use std::collections::BTreeMap;
use std::env;
use std::fs::{self, OpenOptions};
use std::hash::{Hash, Hasher};
use std::io::{ErrorKind, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context, Result};
use limux_control::socket_path::{
    resolve_socket_path, resolve_socket_path_for_channel, RuntimeChannel, SocketMode,
};
use limux_protocol::{validate_terminal_text_payload, V2Request, V2Response};
use serde_json::{json, Map, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

mod agent_hooks;
mod doctor;

const CLI_STATE_LOCK_TIMEOUT: Duration = Duration::from_secs(2);
const CLI_STATE_LOCK_RETRY: Duration = Duration::from_millis(25);
const AGENT_HOOK_NOTIFY_BUDGET: Duration = Duration::from_millis(500);
const AGENT_TEAM_BOOTSTRAP_LAUNCH_SETTLE: Duration = Duration::from_millis(1000);
const AGENT_TEAM_BOOTSTRAP_RETRY_ATTEMPTS: usize = 50;
const AGENT_TEAM_BOOTSTRAP_RETRY_INTERVAL: Duration = Duration::from_millis(100);
const AGENT_TEAM_PROTOCOL_MARKER: &str = "<!-- limux-agent-team-protocol generated:v1 -->";
const AGENT_TEAM_ROSTER_MARKER: &str = "<!-- limux-team-roster durable:create-if-missing:v1 -->";
const AGENT_TEAM_LEDGER_MARKER: &str = "<!-- limux-review-ledger durable:v1 -->";
const REVIEW_REQUEST_MARKER: &str = "<!-- limux-review-request generated:v1 -->";
const REVIEW_EVIDENCE_MARKER: &str = "<!-- limux-review-evidence pointer:v1 -->";
const HOST_LAUNCH_SOCKET_ENV_REMOVALS: &[&str] = &[
    "LIMUX_SOCKET",
    "LIMUX_SOCKET_PATH",
    limux_control::socket_path::LIMUX_CHANNEL_ENV,
    limux_control::socket_path::LIMUX_PREVIEW_ID_ENV,
];
const HOST_LAUNCH_SESSION_ENV_REMOVALS: &[&str] = &["LIMUX_SESSION_DIR"];
const HOST_LAUNCH_TARGET_ENV_REMOVALS: &[&str] = &[
    "LIMUX_WORKSPACE_ID",
    "LIMUX_SURFACE_ID",
    "LIMUX_PANE_ID",
    "LIMUX_TAB_ID",
];
const AGENT_TEAM_DEFAULT_PROTOCOL_FILE: &str = "LIMUX_AGENTS.md";
const AGENT_TEAM_DEFAULT_ROSTER_FILE: &str = "LIMUX_TEAM_ROSTER.md";
const AGENT_TEAM_DEFAULT_LEDGER_FILE: &str = "LIMUX_REVIEW_LEDGER.md";
const AGENT_TEAM_LOCAL_POLICY_FILE: &str = "LIMUX_AGENTS.local.md";
const AGENT_TEAM_INSTRUCTION_FILES: &[&str] = &["AGENTS.md", "CLAUDE.md", "GEMINI.md"];
const REVIEW_DEFAULT_REVIEWS_DIR: &str = "reviews";
const REVIEW_REVIEWERS: &[&str] = &["codex", "claude", "gemini", "opencode", "hermes", "manual"];
type HookInstallSpec = (&'static str, &'static str, Option<&'static str>);
const REVIEW_LENSES: &[&str] = &[
    "security",
    "correctness",
    "maintainability",
    "ux",
    "release",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IdFormat {
    Refs,
    Both,
    Uuids,
}

impl IdFormat {
    fn parse(raw: &str) -> Result<Self> {
        match raw {
            "refs" => Ok(Self::Refs),
            "both" => Ok(Self::Both),
            "uuids" => Ok(Self::Uuids),
            _ => bail!("--id-format must be one of refs|both|uuids"),
        }
    }
}

#[derive(Debug, Clone)]
struct GlobalOptions {
    socket: Option<PathBuf>,
    channel: Option<RuntimeChannel>,
    socket_mode: SocketMode,
    json_output: bool,
    id_format: IdFormat,
    request: Option<String>,
    pretty: bool,
    command_args: Vec<String>,
}

#[derive(Debug)]
enum CommandOutput {
    Text(String),
    Json(Value),
    TextWithExit(String, i32),
    JsonWithExit(Value, i32),
}

struct Client {
    socket: PathBuf,
    seq: u64,
}

impl Client {
    fn new(socket: PathBuf) -> Self {
        Self { socket, seq: 0 }
    }

    async fn call(&mut self, method: &str, params: Value) -> Result<Value> {
        self.seq = self.seq.saturating_add(1);
        let request = V2Request {
            id: Some(Value::String(format!("cli-{}", self.seq))),
            method: method.to_string(),
            params,
        };
        self.send_request(request).await
    }

    async fn send_request(&self, request: V2Request) -> Result<Value> {
        let stream = UnixStream::connect(&self.socket)
            .await
            .with_context(|| format!("failed to connect to socket {}", self.socket.display()))?;
        let (reader_half, mut writer_half) = stream.into_split();

        let mut payload = serde_json::to_string(&request).context("failed to encode request")?;
        payload.push('\n');

        writer_half
            .write_all(payload.as_bytes())
            .await
            .context("failed to write request")?;
        writer_half
            .flush()
            .await
            .context("failed to flush request")?;

        let mut reader = BufReader::new(reader_half);
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .await
            .context("failed to read response")?;

        if line.trim().is_empty() {
            bail!("server returned an empty response");
        }

        let response: V2Response =
            serde_json::from_str(line.trim()).context("response was not valid v2 JSON")?;

        if response.ok {
            Ok(response.result.unwrap_or_else(|| json!({})))
        } else {
            let err = response
                .error
                .ok_or_else(|| anyhow!("server returned !ok without error payload"))?;
            if err.code == -32004 {
                bail!("not_found: {}", err.message);
            }
            bail!("{}: {}", err.code, err.message);
        }
    }
}

fn parse_global_args() -> Result<GlobalOptions> {
    let mut args: Vec<String> = env::args().skip(1).collect();
    let mut socket: Option<PathBuf> = None;
    let mut channel: Option<RuntimeChannel> = None;
    let mut socket_mode = SocketMode::Runtime;
    let mut json_output = false;
    let mut id_format = IdFormat::Refs;
    let mut request: Option<String> = None;
    let mut pretty = false;

    let mut command_start = 0usize;
    while command_start < args.len() {
        let arg = args[command_start].clone();
        if !arg.starts_with('-') {
            break;
        }
        match arg.as_str() {
            "--socket" => {
                let value = args
                    .get(command_start + 1)
                    .ok_or_else(|| anyhow!("--socket requires a value"))?;
                socket = Some(PathBuf::from(value));
                command_start += 2;
            }
            "--socket-mode" => {
                let value = args
                    .get(command_start + 1)
                    .ok_or_else(|| anyhow!("--socket-mode requires runtime|debug"))?;
                socket_mode = match value.as_str() {
                    "runtime" => SocketMode::Runtime,
                    "debug" => SocketMode::Debug,
                    _ => bail!("--socket-mode must be runtime or debug"),
                };
                command_start += 2;
            }
            "--channel" => {
                let value = args
                    .get(command_start + 1)
                    .ok_or_else(|| anyhow!("--channel requires stable|preview[:id]"))?;
                channel = RuntimeChannel::parse(value)
                    .ok_or_else(|| anyhow!("--channel requires stable|preview[:id]"))?
                    .into();
                command_start += 2;
            }
            "--json" => {
                json_output = true;
                command_start += 1;
            }
            "--id-format" => {
                let value = args
                    .get(command_start + 1)
                    .ok_or_else(|| anyhow!("--id-format requires refs|both|uuids"))?;
                id_format = IdFormat::parse(value)?;
                command_start += 2;
            }
            "--request" => {
                let value = args
                    .get(command_start + 1)
                    .ok_or_else(|| anyhow!("--request requires a JSON value"))?;
                request = Some(value.clone());
                command_start += 2;
            }
            "--pretty" => {
                pretty = true;
                command_start += 1;
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            "--version" | "-V" => {
                println!("{}", render_cli_version());
                std::process::exit(0);
            }
            _ => break,
        }
    }

    let command_args = args.split_off(command_start);

    Ok(GlobalOptions {
        socket,
        channel,
        socket_mode,
        json_output,
        id_format,
        request,
        pretty,
        command_args,
    })
}

fn print_help() {
    println!(
        "limux CLI\n\nUsage: limux [--socket <path>] [--channel stable|preview[:id]] [--json] [--id-format refs|both|uuids] <command> [args...]\n       limux\n\nRunning `limux` with no arguments launches the GTK app.\n\nCommon commands:\n  --version\n  identify [--workspace <id|ref>] [--surface <id|ref>]\n  doctor [--json] [--log-triage [--lines <n>]]\n  list-panels [--workspace <id|ref>]\n  list-panes [--workspace <id|ref>]\n  list-workspaces\n  surface-health [--workspace <id|ref>]\n  send [--workspace <id|ref>] [--surface <id|ref>] <text>\n  send-key [--workspace <id|ref>] [--surface <id|ref>] <key>\n  new-workspace [--cwd <path>] [--command <text>]\n  close-workspace --workspace <id|ref>\n  sidebar-state --workspace <id|ref>\n  new-surface [--workspace <id|ref>]\n  new-pane [--workspace <id|ref>] [--pane <id|ref>] [--surface <id|ref>] [--direction <left|right|up|down>] [--type <terminal|browser>] [--command <text>] [--url <url>]\n      Live GTK self-spawn currently supports terminal panes only; browser panes remain deferred.\n  rename-workspace [--workspace <id|ref>] <title>\n  rename-window [--workspace <id|ref>] <title>\n  rename-tab [--workspace <id|ref>] [--tab <id|ref>] <title>\n  read-screen [--workspace <id|ref>] [--surface <id|ref>] [--scrollback] [--lines <n>]\n  capture-pane (alias of read-screen)\n  tab-action --action <name> [--workspace <id|ref>] [--tab <id|ref>] [--title <text>] [--url <url>]\n  pane-action --action set_flag_color --color <orange|red|purple|pink|green|yellow|teal|cyan> [--workspace <id|ref>] [--pane <id|ref>]\n  pane-action --action clear_flag_color [--workspace <id|ref>] [--pane <id|ref>]\n  target-info (alias: socket-info) prints the resolved socket/channel without connecting\n  browser [--surface <id|ref>|<surface>] <subcommand> ...\n\nAgent integrations:\n  notify [--workspace <id|ref>] [--subtitle <text>] [--body <text>] <title>\n  hooks setup [agent] | hooks uninstall [agent] | hooks <agent> <event>\n  claude-hook | opencode-hook | gemini-hook --event <name> [--subtitle <text>] [--body <text>] [--title <text>]\n  agent-team [--agents codex,claude[,opencode,gemini]] [--launch-mode direct|hcom] [--cwd <path>] [--protocol-path <path>] [--roster-path <path>] [--ledger-path <path>] [--force-protocol-overwrite] [--force-roster-overwrite] [--no-launch] [--no-bootstrap] [--dry-run]\n      Splits the active workspace into one pane per agent (caller's pane stays\n      as the orchestrator on the left, peers stack down the right), launches\n      each CLI in its pane, or hcom with --run-here when requested, writes\n      LIMUX_AGENTS.md, and seeds LIMUX_TEAM_ROSTER.md plus\n      LIMUX_REVIEW_LEDGER.md when missing so peers can coordinate via durable\n      files and `limux send --surface <peer-surface-id> <envelope>`.\n  review prepare --artifact <path-or-ref> --reviewer <agent|manual> --lens <name> --summary <text> [--cwd <path>] [--ledger-path <path>] [--reviews-dir <path>] [--review-id <id>] [--dry-run]\n      Creates a durable review request file, appends a pending review-ledger\n      entry, and prints the reviewer prompt without launching a reviewer pane.\n"
    );
    println!(
        "  agent-team extra flags: --no-bootstrap skips the post-launch bootstrap prompt while still launching panes; --dry-run skips host contact but still materializes the protocol and seeds missing roster/ledger files."
    );
    println!(
        "  review spawn: review spawn --review-id <id> [--cwd <path>] [--reviews-dir <path>] [--ledger-path <path>] [--evidence-path <path>] [--workspace <id|ref>] [--surface <id|ref>] [--direction <left|right|up|down>] [--launch-mode direct|hcom] [--no-launch] [--dry-run]"
    );
    println!(
        "  hermes-hook: alias of `limux hooks hermes <event>`; Hermes-side lifecycle plugin install remains external."
    );
    println!(
        "  agent-team supports `--agents hermes` and hcom launch as `hcom hermes --run-here`."
    );
}

fn current_cli_build_info() -> limux_control::BuildInfo {
    limux_control::BuildInfo::from_compile_env(
        option_env!("LIMUX_BUILD_SHA"),
        option_env!("LIMUX_BUILD_DIRTY"),
        option_env!("LIMUX_BUILD_PROFILE"),
    )
}

fn render_cli_version() -> String {
    let build = current_cli_build_info();
    limux_control::render_version_line("limux-cli", env!("CARGO_PKG_VERSION"), &build)
}

fn should_launch_host(opts: &GlobalOptions) -> bool {
    opts.command_args.is_empty()
        && opts.request.is_none()
        && opts.socket.is_none()
        && opts.socket_mode == SocketMode::Runtime
        && !opts.json_output
        && !opts.pretty
        && opts.id_format == IdFormat::Refs
}

fn host_binary_candidates(exe: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(bin_dir) = exe.parent() {
        if let Some(prefix) = bin_dir.parent() {
            candidates.push(prefix.join("libexec/limux/limux-host"));
        }

        let sibling_host = bin_dir.join("limux-host");
        if sibling_host != exe {
            candidates.push(sibling_host);
        }

        let sibling_dev_host = bin_dir.join("limux");
        if sibling_dev_host != exe {
            candidates.push(sibling_dev_host);
        }
    }

    candidates
}

fn resolve_host_binary() -> Result<PathBuf> {
    if let Ok(raw) = env::var("LIMUX_HOST_BIN") {
        let path = PathBuf::from(raw);
        if path.is_file() {
            return Ok(path);
        }
    }

    let exe = env::current_exe().context("failed to resolve current executable")?;
    host_binary_candidates(&exe)
        .into_iter()
        .find(|path| path.is_file())
        .ok_or_else(|| {
            anyhow!(
                "could not find limux host binary; expected limux-host next to the installed CLI"
            )
        })
}

fn host_launch_command_with_inherited_target_env(
    host: &Path,
    inherited_target_env: bool,
    channel: Option<&RuntimeChannel>,
) -> Command {
    let mut command = Command::new(host);
    for key in host_launch_env_removals(inherited_target_env, channel.is_some()) {
        command.env_remove(key);
    }
    if let Some(channel) = channel {
        command.env(
            limux_control::socket_path::LIMUX_CHANNEL_ENV,
            channel.env_value(),
        );
    }
    command
}

fn host_launch_command(host: &Path, channel: Option<&RuntimeChannel>) -> Command {
    host_launch_command_with_inherited_target_env(
        host,
        host_launch_has_inherited_target_env(),
        channel,
    )
}

fn host_launch_has_inherited_target_env() -> bool {
    HOST_LAUNCH_TARGET_ENV_REMOVALS
        .iter()
        .any(|key| env::var_os(key).is_some())
}

fn host_launch_env_removals(
    inherited_target_env: bool,
    explicit_channel: bool,
) -> Vec<&'static str> {
    let mut removals = HOST_LAUNCH_TARGET_ENV_REMOVALS.to_vec();
    if inherited_target_env || explicit_channel {
        removals.extend_from_slice(HOST_LAUNCH_SOCKET_ENV_REMOVALS);
        removals.extend_from_slice(HOST_LAUNCH_SESSION_ENV_REMOVALS);
    }
    removals
}

fn launch_host(channel: Option<&RuntimeChannel>) -> Result<()> {
    let host = resolve_host_binary()?;
    let err = host_launch_command(&host, channel)
        .spawn()
        .with_context(|| format!("failed to launch {}", host.display()))?
        .wait()
        .with_context(|| format!("failed to wait for {}", host.display()))?;
    if err.success() {
        Ok(())
    } else {
        bail!("{} exited with {}", host.display(), err)
    }
}

fn get_string(value: &Value, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(raw) = value.get(*key) {
            match raw {
                Value::String(s) if !s.is_empty() => return Some(s.clone()),
                Value::Number(n) => return Some(n.to_string()),
                _ => {}
            }
        }
    }
    None
}

fn handle_from_payload(value: &Value, id_key: &str, ref_key: &str) -> String {
    get_string(value, &[ref_key])
        .or_else(|| get_string(value, &[id_key]))
        .unwrap_or_default()
}

fn apply_id_format(value: &mut Value, id_format: IdFormat) {
    match value {
        Value::Object(map) => {
            let keys: Vec<String> = map.keys().cloned().collect();
            for key in &keys {
                if key.ends_with("_id") {
                    let prefix = key.trim_end_matches("_id");
                    let ref_key = format!("{}_ref", prefix);
                    match id_format {
                        IdFormat::Refs => {
                            if map.contains_key(&ref_key) {
                                map.remove(key);
                            }
                        }
                        IdFormat::Uuids => {
                            if map.contains_key(key) {
                                map.remove(&ref_key);
                            }
                        }
                        IdFormat::Both => {}
                    }
                }
            }

            match id_format {
                IdFormat::Refs => {
                    if map.contains_key("ref") {
                        map.remove("id");
                    }
                }
                IdFormat::Uuids => {
                    if map.contains_key("id") {
                        map.remove("ref");
                    }
                }
                IdFormat::Both => {}
            }

            let child_keys: Vec<String> = map.keys().cloned().collect();
            for key in child_keys {
                if let Some(child) = map.get_mut(&key) {
                    apply_id_format(child, id_format);
                }
            }
        }
        Value::Array(list) => {
            for item in list {
                apply_id_format(item, id_format);
            }
        }
        _ => {}
    }
}

fn parse_opt(args: &[String], name: &str) -> Option<String> {
    args.windows(2).find_map(|w| {
        if w[0] == name {
            Some(w[1].clone())
        } else {
            None
        }
    })
}

fn parse_flag(args: &[String], name: &str) -> bool {
    args.iter().any(|a| a == name)
}

fn positional_arg(args: &[String], index: usize) -> Option<String> {
    let mut position = 0usize;
    let mut skip = false;
    for arg in args {
        if skip {
            skip = false;
            continue;
        }
        if arg == "--agent" {
            skip = true;
            continue;
        }
        if arg.starts_with('-') {
            continue;
        }
        if position == index {
            return Some(arg.clone());
        }
        position += 1;
    }
    None
}

fn trailing_title(args: &[String]) -> Option<String> {
    let mut filtered: Vec<String> = Vec::new();
    let mut skip = false;
    for arg in args {
        if skip {
            skip = false;
            continue;
        }
        if arg == "--workspace"
            || arg == "--tab"
            || arg == "--surface"
            || arg == "--pane"
            || arg == "--target-pane"
            || arg == "--action"
            || arg == "--title"
            || arg == "--url"
            || arg == "--cwd"
            || arg == "--command"
            || arg == "--direction"
            || arg == "--type"
            || arg == "--lines"
            || arg == "--timeout"
            || arg == "--timeout-ms"
            || arg == "--name"
            || arg == "--out"
            || arg == "--subtitle"
            || arg == "--body"
            || arg == "--message"
            || arg == "--event"
            || arg == "--agents"
            || arg == "--artifact"
            || arg == "--reviewer"
            || arg == "--lens"
            || arg == "--summary"
            || arg == "--ledger-path"
            || arg == "--reviews-dir"
            || arg == "--review-id"
            || arg == "--selector"
            || arg == "--text"
            || arg == "--attr"
            || arg == "--property"
            || arg == "--value"
            || arg == "--amount"
            || arg == "--color"
            || arg == "--unset"
        {
            skip = true;
            continue;
        }
        if arg.starts_with('-') {
            continue;
        }
        filtered.push(arg.clone());
    }
    if filtered.is_empty() {
        None
    } else {
        Some(filtered.join(" "))
    }
}

fn validate_terminal_text_arg(label: &str, text: &str) -> Result<()> {
    validate_terminal_text_payload(label, text).map_err(anyhow::Error::from)
}

fn wait_signal_path(name: &str) -> PathBuf {
    let sanitized: String = name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect();
    PathBuf::from(format!("/tmp/limux-wait-for-{}.sig", sanitized))
}

fn read_json_map(path: &str) -> BTreeMap<String, String> {
    let raw = fs::read_to_string(path).unwrap_or_default();
    serde_json::from_str::<BTreeMap<String, String>>(&raw).unwrap_or_default()
}

fn write_json_map(path: &Path, map: &BTreeMap<String, String>) -> Result<()> {
    let encoded = serde_json::to_string_pretty(map).context("failed to encode json map")?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let tmp = path.with_extension(format!("tmp-{}-{}", std::process::id(), nonce));
    fs::write(&tmp, encoded).with_context(|| format!("failed to write {}", tmp.display()))?;
    fs::rename(&tmp, path).with_context(|| format!("failed to replace {}", path.display()))?;
    Ok(())
}

fn socket_state_namespace(socket: &Path) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    socket.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn cli_state_dir(socket: &Path) -> PathBuf {
    env::temp_dir()
        .join("limux-cli")
        .join(socket_state_namespace(socket))
}

fn cli_state_path(socket: &Path, kind: &str) -> PathBuf {
    cli_state_dir(socket).join(format!("{kind}.json"))
}

fn cli_state_lock_path(socket: &Path, kind: &str) -> PathBuf {
    cli_state_dir(socket).join(format!("{kind}.lock"))
}

struct CliStateLock {
    path: PathBuf,
}

impl Drop for CliStateLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn acquire_cli_state_lock(socket: &Path, kind: &str) -> Result<CliStateLock> {
    let dir = cli_state_dir(socket);
    fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;
    let lock_path = cli_state_lock_path(socket, kind);
    let deadline = Instant::now() + CLI_STATE_LOCK_TIMEOUT;
    loop {
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock_path)
        {
            Ok(_) => return Ok(CliStateLock { path: lock_path }),
            Err(err) if err.kind() == ErrorKind::AlreadyExists => {
                if Instant::now() >= deadline {
                    bail!("timed out acquiring CLI state lock {}", lock_path.display());
                }
                std::thread::sleep(CLI_STATE_LOCK_RETRY);
            }
            Err(err) => {
                return Err(err).with_context(|| {
                    format!("failed to create CLI state lock {}", lock_path.display())
                });
            }
        }
    }
}

fn with_locked_json_map<T, F>(socket: &Path, kind: &str, update: F) -> Result<T>
where
    F: FnOnce(&mut BTreeMap<String, String>, &Path) -> Result<T>,
{
    let _lock = acquire_cli_state_lock(socket, kind)?;
    let path = cli_state_path(socket, kind);
    let path_str = path.to_string_lossy().to_string();
    let mut map = read_json_map(&path_str);
    update(&mut map, &path)
}

async fn resolve_current_workspace(client: &mut Client) -> Result<String> {
    let current = client.call("workspace.current", json!({})).await?;
    get_string(&current, &["workspace_id", "workspace_ref"])
        .ok_or_else(|| anyhow!("workspace.current returned no workspace handle"))
}

async fn call_in_workspace_scope(
    client: &mut Client,
    workspace: Option<String>,
    method: &str,
    params: Value,
) -> Result<Value> {
    if let Some(target) = workspace {
        let mut map = match params {
            Value::Object(map) => map,
            Value::Null => Map::new(),
            _ => bail!("{method} requires object params for workspace-scoped calls"),
        };
        map.entry("workspace_id".to_string())
            .or_insert(Value::String(target));
        return client.call(method, Value::Object(map)).await;
    }
    client.call(method, params).await
}

async fn browser_call(
    client: &mut Client,
    surface: Option<String>,
    method: &str,
    mut params: Map<String, Value>,
) -> Result<Value> {
    if let Some(surface) = surface {
        params.insert("surface_id".to_string(), Value::String(surface));
    }
    client.call(method, Value::Object(params)).await
}

async fn selected_surface_for_pane(
    client: &mut Client,
    workspace: Option<String>,
    pane_id: &str,
) -> Result<String> {
    let payload = call_in_workspace_scope(
        client,
        workspace,
        "pane.surfaces",
        json!({ "pane_id": pane_id }),
    )
    .await?;
    let rows = payload
        .get("surfaces")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("pane.surfaces returned no surfaces"))?;

    for row in rows {
        let focused = row.get("focused").and_then(Value::as_bool).unwrap_or(false)
            || row
                .get("selected")
                .and_then(Value::as_bool)
                .unwrap_or(false);
        if focused {
            let handle = handle_from_payload(row, "surface_id", "surface_ref");
            if !handle.is_empty() {
                return Ok(handle);
            }
        }
    }

    let first = rows
        .first()
        .ok_or_else(|| anyhow!("pane has no surfaces"))?;
    let handle = handle_from_payload(first, "surface_id", "surface_ref");
    if handle.is_empty() {
        bail!("pane.surfaces returned an empty surface handle");
    }
    Ok(handle)
}

async fn run_identify(client: &mut Client, args: &[String]) -> Result<Value> {
    let workspace = parse_opt(args, "--workspace");
    let surface = parse_opt(args, "--surface");
    let no_caller = parse_flag(args, "--no-caller");

    let mut params = Map::new();
    if workspace.is_some() || surface.is_some() {
        let mut caller = Map::new();
        if let Some(workspace) = workspace {
            caller.insert("workspace_id".to_string(), Value::String(workspace));
        }
        if let Some(surface) = surface {
            caller.insert("surface_id".to_string(), Value::String(surface));
        }
        params.insert("caller".to_string(), Value::Object(caller));
    }

    let mut payload = client
        .call("system.identify", Value::Object(params))
        .await?;
    if no_caller {
        if let Some(map) = payload.as_object_mut() {
            map.remove("caller");
        }
    }
    Ok(payload)
}

async fn run_list(client: &mut Client, command: &str, args: &[String]) -> Result<Value> {
    let workspace = parse_opt(args, "--workspace")
        .or_else(|| env::var("LIMUX_WORKSPACE_ID").ok())
        .filter(|value| !value.trim().is_empty());
    let params = if let Some(workspace) = workspace.as_ref() {
        json!({ "workspace_id": workspace })
    } else {
        json!({})
    };
    let method = match command {
        "list-panels" => "surface.list",
        "list-panes" => "pane.list",
        "list-workspaces" => "workspace.list",
        "surface-health" => "surface.health",
        _ => bail!("unsupported list command"),
    };
    let mut payload = client.call(method, params).await?;
    if let Some(workspace) = workspace.as_ref() {
        if let Some(map) = payload.as_object_mut() {
            if workspace.contains(':') {
                map.entry("workspace_ref".to_string())
                    .or_insert_with(|| Value::String(workspace.clone()));
            } else {
                map.entry("workspace_id".to_string())
                    .or_insert_with(|| Value::String(workspace.clone()));
            }
        }
    }
    Ok(payload)
}

fn render_list_text(command: &str, payload: &Value) -> String {
    match command {
        "list-panels" => {
            let rows = payload
                .get("surfaces")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            if rows.is_empty() {
                return "No surfaces".to_string();
            }
            rows.iter()
                .map(|row| {
                    let handle = handle_from_payload(row, "surface_id", "surface_ref");
                    let title = get_string(row, &["title"]).unwrap_or_default();
                    format!("{} {}", handle, title)
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
        "list-panes" => {
            let rows = payload
                .get("panes")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            if rows.is_empty() {
                return "No panes".to_string();
            }
            rows.iter()
                .map(|row| {
                    let handle = handle_from_payload(row, "pane_id", "pane_ref");
                    let count = row
                        .get("surface_count")
                        .and_then(Value::as_u64)
                        .unwrap_or(0);
                    format!("{} surfaces={}", handle, count)
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
        "list-workspaces" => {
            let rows = payload
                .get("workspaces")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            if rows.is_empty() {
                return "No workspaces".to_string();
            }
            rows.iter()
                .map(|row| {
                    let handle = handle_from_payload(row, "workspace_id", "workspace_ref");
                    let title = get_string(row, &["title", "name"]).unwrap_or_default();
                    let selected = row
                        .get("selected")
                        .and_then(Value::as_bool)
                        .unwrap_or(false);
                    if selected {
                        format!("* {} {}", handle, title)
                    } else {
                        format!("  {} {}", handle, title)
                    }
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
        "surface-health" => {
            let rows = payload
                .get("surfaces")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            if rows.is_empty() {
                return "No surfaces".to_string();
            }
            rows.iter()
                .map(|row| {
                    let handle = handle_from_payload(row, "surface_id", "surface_ref");
                    let healthy = row.get("healthy").and_then(Value::as_bool).unwrap_or(true);
                    format!("{} healthy={}", handle, healthy)
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
        _ => "".to_string(),
    }
}

async fn run_send(client: &mut Client, args: &[String]) -> Result<Value> {
    let workspace = parse_opt(args, "--workspace")
        .or_else(|| env::var("LIMUX_WORKSPACE_ID").ok())
        .filter(|s| !s.is_empty());
    let surface = parse_opt(args, "--surface").filter(|s| !s.is_empty());

    let text = trailing_title(args).ok_or_else(|| anyhow!("send requires text"))?;
    validate_terminal_text_arg("surface.send_text text", &text)?;

    let mut params = Map::new();
    params.insert("text".to_string(), Value::String(text));
    if let Some(surface) = surface {
        params.insert("surface_id".to_string(), Value::String(surface));
    }

    call_in_workspace_scope(
        client,
        workspace,
        "surface.send_text",
        Value::Object(params),
    )
    .await
}

async fn run_send_key(client: &mut Client, args: &[String]) -> Result<Value> {
    let workspace = parse_opt(args, "--workspace")
        .or_else(|| env::var("LIMUX_WORKSPACE_ID").ok())
        .filter(|s| !s.is_empty());
    let surface = parse_opt(args, "--surface").filter(|s| !s.is_empty());
    let key = trailing_title(args).ok_or_else(|| anyhow!("send-key requires key"))?;

    let mut params = Map::new();
    params.insert("key".to_string(), Value::String(key));
    if let Some(surface) = surface {
        params.insert("surface_id".to_string(), Value::String(surface));
    }

    call_in_workspace_scope(client, workspace, "surface.send_key", Value::Object(params)).await
}

/// `limux notify` — post a notification into the sidebar + toast overlay.
///
/// Usage:
///   limux notify [--workspace <id|ref>] [--subtitle <text>] [--body <text>] <title>
///   limux notify --title "..." --subtitle "..." --body "..."
///
/// Mirrors the `cmux notify` shape (title / subtitle / body). Title is
/// required; subtitle and body are optional. Falls back to the current
/// workspace via LIMUX_WORKSPACE_ID when --workspace isn't given.
async fn run_notify(client: &mut Client, args: &[String]) -> Result<Value> {
    let workspace = parse_opt(args, "--workspace")
        .or_else(|| env::var("LIMUX_WORKSPACE_ID").ok())
        .filter(|s| !s.is_empty());

    // Title can be provided either via --title or as the trailing positional
    // (matching `limux send`'s ergonomics).
    let title = parse_opt(args, "--title")
        .or_else(|| trailing_title(args))
        .ok_or_else(|| anyhow!("notify requires a title"))?;

    let subtitle = parse_opt(args, "--subtitle").unwrap_or_default();
    let body = parse_opt(args, "--body")
        .or_else(|| parse_opt(args, "--message"))
        .unwrap_or_default();

    let mut params = Map::new();
    params.insert("title".to_string(), Value::String(title));
    if !subtitle.is_empty() {
        params.insert("subtitle".to_string(), Value::String(subtitle));
    }
    if !body.is_empty() {
        params.insert("body".to_string(), Value::String(body));
    }

    call_in_workspace_scope(
        client,
        workspace,
        "notification.create",
        Value::Object(params),
    )
    .await
}

// ---------------------------------------------------------------------------
// Agent hooks (claude-hook / opencode-hook / gemini-hook / hermes-hook)
// ---------------------------------------------------------------------------
//
// These subcommands read a JSON hook event from stdin and translate it into
// a `notify` (and, eventually, log / progress) call so the GUI reflects
// agent activity in real time. Designed for direct wiring into Claude Code,
// OpenCode, and Gemini CLI's hook settings.
//
// Claude Code stdin schema (what we rely on):
//   {
//     "session_id": "...",
//     "transcript_path": "...",
//     "cwd": "...",
//     "hook_event_name": "Notification" | "Stop" | "SessionStart" | ...,
//     "message": "agent is waiting for input",     // Notification only
//     "tool_name": "...", "tool_input": {...},     // PreToolUse/PostToolUse
//     "tool_response": {...},                       // PostToolUse
//     "prompt": "..."                               // UserPromptSubmit
//   }
//
// OpenCode and Gemini use slightly different names; we fall back gracefully
// when fields are missing. Hermes-side lifecycle payloads may also nest useful
// fields under `extra` or `metadata`.

/// Pull a string field from the hook JSON, trying multiple keys.
fn hook_str<'a>(payload: &'a Value, keys: &[&str]) -> Option<&'a str> {
    keys.iter()
        .find_map(|k| {
            payload
                .get(*k)
                .and_then(Value::as_str)
                .or_else(|| {
                    payload
                        .get("extra")
                        .and_then(|value| value.get(*k))
                        .and_then(Value::as_str)
                })
                .or_else(|| {
                    payload
                        .get("metadata")
                        .and_then(|value| value.get(*k))
                        .and_then(Value::as_str)
                })
        })
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

fn parse_hook_event(args: &[String], payload: &Value) -> String {
    parse_opt(args, "--event")
        .or_else(|| trailing_title(args))
        .or_else(|| hook_str(payload, &["hook_event_name", "event"]).map(str::to_owned))
        .unwrap_or_else(|| "event".to_string())
}

/// Run an agent hook: read JSON from stdin, synthesize a notification.
///
/// Args:
///   [event_name] — optional positional, e.g. "Notification", "Stop".
///                  If omitted, we read `hook_event_name` from the JSON.
async fn run_agent_hook(
    client: &mut Client,
    agent: agent_hooks::AgentKind,
    args: &[String],
) -> Result<Value> {
    use std::io::Read;

    // Read stdin (hook JSON). If stdin is empty or not JSON, treat as
    // minimal event so we still post *something*.
    let mut raw = String::new();
    let _ = std::io::stdin().read_to_string(&mut raw);
    let raw = raw.trim();
    let payload: Value = if raw.is_empty() {
        Value::Object(Map::new())
    } else {
        serde_json::from_str(raw).unwrap_or_else(|_| json!({ "raw": raw }))
    };

    // Explicit --event or positional event beats the JSON field.
    let event = parse_hook_event(args, &payload);

    // Build a human-friendly title + body depending on event + agent.
    let agent_label = agent.label();
    persist_agent_hook_session(agent, args, &payload, &event)?;
    let tool_name = hook_str(&payload, &["tool_name", "toolName", "name"]).unwrap_or("");
    let (title, body) = match canonical_agent_hook_display_event(&event) {
        AgentHookDisplayEvent::Notification => (
            format!("{agent_label} needs you"),
            hook_str(
                &payload,
                &["message", "notification", "description", "pattern_key"],
            )
            .unwrap_or("waiting for input")
            .to_owned(),
        ),
        AgentHookDisplayEvent::Stop => (
            format!("{agent_label} finished"),
            hook_str(&payload, &["message", "reason"])
                .unwrap_or("task complete")
                .to_owned(),
        ),
        AgentHookDisplayEvent::SessionStart => (
            format!("{agent_label} session started"),
            hook_str(&payload, &["cwd", "directory", "source"])
                .unwrap_or("")
                .to_owned(),
        ),
        AgentHookDisplayEvent::SessionEnd => (
            format!("{agent_label} session ended"),
            hook_str(&payload, &["reason"]).unwrap_or("").to_owned(),
        ),
        AgentHookDisplayEvent::ToolUse => (
            format!("{agent_label}: {}", non_empty_or(tool_name, "tool")),
            hook_str(&payload, &["tool_input", "summary", "command", "path"])
                .unwrap_or("")
                .to_owned(),
        ),
        AgentHookDisplayEvent::UserPromptSubmit => (
            format!("{agent_label}: new prompt"),
            hook_str(&payload, &["prompt"])
                .unwrap_or("")
                .chars()
                .take(120)
                .collect(),
        ),
        AgentHookDisplayEvent::Other => {
            let event_label = event.trim();
            let event_label = if event_label.is_empty() {
                "event"
            } else {
                event_label
            };
            (
                format!("{agent_label}: {event_label}"),
                hook_str(&payload, &["message", "summary"])
                    .unwrap_or("")
                    .to_owned(),
            )
        }
    };

    let subtitle = hook_str(&payload, &["session_id"])
        .map(|s| {
            // Show only a short prefix of the session id to keep sidebar tidy.
            s.chars().take(8).collect::<String>()
        })
        .unwrap_or_default();

    let workspace = parse_opt(args, "--workspace")
        .or_else(|| env::var("LIMUX_WORKSPACE_ID").ok())
        .filter(|s| !s.is_empty());

    let mut params = Map::new();
    params.insert("title".to_string(), Value::String(title));
    if !subtitle.is_empty() {
        params.insert("subtitle".to_string(), Value::String(subtitle));
    }
    if !body.is_empty() {
        params.insert("body".to_string(), Value::String(body));
    }

    let workspace_for_debug = workspace.clone();
    let notify_outcome = send_agent_hook_notification_with_budget(client, workspace, params).await;
    match notify_outcome {
        AgentHookNotifyOutcome::Delivered => write_agent_hook_debug(
            agent,
            &event,
            "notify_ok",
            &agent_hook_notify_debug_details(&client.socket, workspace_for_debug.as_deref(), None),
        ),
        AgentHookNotifyOutcome::Error(error) => {
            write_agent_hook_debug(
                agent,
                &event,
                "notify_error",
                &agent_hook_notify_debug_details(
                    &client.socket,
                    workspace_for_debug.as_deref(),
                    Some(&error),
                ),
            );
        }
        AgentHookNotifyOutcome::Timeout => {
            let error = format!("timed out after {}ms", AGENT_HOOK_NOTIFY_BUDGET.as_millis());
            write_agent_hook_debug(
                agent,
                &event,
                "notify_timeout",
                &agent_hook_notify_debug_details(
                    &client.socket,
                    workspace_for_debug.as_deref(),
                    Some(&error),
                ),
            );
        }
    }

    Ok(agent_hook_output(&event, &payload))
}

#[derive(Debug, Eq, PartialEq)]
enum AgentHookNotifyOutcome {
    Delivered,
    Error(String),
    Timeout,
}

async fn send_agent_hook_notification_with_budget(
    client: &mut Client,
    workspace: Option<String>,
    params: Map<String, Value>,
) -> AgentHookNotifyOutcome {
    let notification = call_in_workspace_scope(
        client,
        workspace,
        "notification.create",
        Value::Object(params),
    );
    match tokio::time::timeout(AGENT_HOOK_NOTIFY_BUDGET, notification).await {
        Ok(Ok(_)) => AgentHookNotifyOutcome::Delivered,
        Ok(Err(error)) => AgentHookNotifyOutcome::Error(format!("{error:#}")),
        Err(_) => AgentHookNotifyOutcome::Timeout,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AgentHookDisplayEvent {
    Notification,
    Stop,
    SessionStart,
    SessionEnd,
    ToolUse,
    UserPromptSubmit,
    Other,
}

fn canonical_agent_hook_display_event(event: &str) -> AgentHookDisplayEvent {
    match event.trim() {
        "Notification" | "notification" | "pre_approval_request" | "pre-approval-request" => {
            AgentHookDisplayEvent::Notification
        }
        "Stop" | "stop" | "SubagentStop" | "subagent-stop" | "subagent_stop" | "post_llm_call"
        | "post-llm-call" => AgentHookDisplayEvent::Stop,
        "SessionStart" | "session-start" | "session_start" | "on_session_start"
        | "session-started" => AgentHookDisplayEvent::SessionStart,
        "SessionEnd"
        | "session-end"
        | "session_end"
        | "on_session_end"
        | "on_session_finalize"
        | "session_finalize" => AgentHookDisplayEvent::SessionEnd,
        "PreToolUse" | "pre-tool-use" | "pre_tool_use" | "PostToolUse" | "post-tool-use"
        | "post_tool_use" | "pre_tool_call" | "pre-tool-call" | "post_tool_call"
        | "post-tool-call" => AgentHookDisplayEvent::ToolUse,
        "UserPromptSubmit" | "prompt-submit" | "user-prompt-submit" | "user_prompt_submit"
        | "pre_llm_call" | "pre-llm-call" => AgentHookDisplayEvent::UserPromptSubmit,
        _ => AgentHookDisplayEvent::Other,
    }
}

fn agent_hook_notify_debug_details(
    socket: &Path,
    workspace: Option<&str>,
    error: Option<&str>,
) -> Value {
    let mut details = json!({
        "workspace": workspace,
        "surface_id": limux_env_value("LIMUX_SURFACE_ID"),
        "socket": limux_env_value("LIMUX_SOCKET"),
        "resolved_socket": socket.to_string_lossy().to_string(),
    });
    if let Some(error) = error {
        details["error"] = Value::String(error.to_string());
    }
    details
}

fn non_empty_or<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    if value.trim().is_empty() {
        fallback
    } else {
        value
    }
}

fn agent_hook_output(event: &str, payload: &Value) -> Value {
    let canonical_event = canonical_hook_event_name(event);
    let mut output = Map::new();
    output.insert("continue".to_string(), Value::Bool(true));
    output.insert("suppressOutput".to_string(), Value::Bool(false));

    if matches!(canonical_event, Some("SessionStart" | "UserPromptSubmit")) {
        let mut specific = Map::new();
        specific.insert(
            "hookEventName".to_string(),
            Value::String(
                canonical_event
                    .expect("matched canonical event")
                    .to_string(),
            ),
        );
        if let Some(context) = hook_additional_context(payload) {
            specific.insert("additionalContext".to_string(), Value::String(context));
        }
        output.insert("hookSpecificOutput".to_string(), Value::Object(specific));
    }

    Value::Object(output)
}

fn canonical_hook_event_name(event: &str) -> Option<&'static str> {
    match event {
        "SessionStart" | "session-start" | "session_start" | "on_session_start" => {
            Some("SessionStart")
        }
        "UserPromptSubmit" | "prompt-submit" | "user_prompt_submit" | "pre_llm_call" => {
            Some("UserPromptSubmit")
        }
        "Stop" | "stop" | "Notification" | "notification" | "post_llm_call" => Some("Stop"),
        "SessionEnd" | "session-end" | "session_end" | "on_session_end" | "on_session_finalize" => {
            None
        }
        "Cleanup" | "cleanup" | "restore-exit" => None,
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AgentHookPersistenceAction {
    Upsert,
    Preserve,
    Remove,
}

fn agent_hook_persistence_action(event: &str) -> AgentHookPersistenceAction {
    match event {
        "Cleanup" | "cleanup" | "restore-exit" => AgentHookPersistenceAction::Remove,
        "SessionEnd" | "session-end" | "session_end" | "on_session_end" | "on_session_finalize" => {
            AgentHookPersistenceAction::Preserve
        }
        _ => AgentHookPersistenceAction::Upsert,
    }
}

fn hook_additional_context(payload: &Value) -> Option<String> {
    hook_str(payload, &["additional_context", "additionalContext"])
        .map(str::to_owned)
        .filter(|value| !value.trim().is_empty())
}

fn persist_agent_hook_session(
    agent: agent_hooks::AgentKind,
    args: &[String],
    payload: &Value,
    event: &str,
) -> Result<()> {
    let Some(session_id) = hook_session_id(payload) else {
        write_agent_hook_debug(
            agent,
            event,
            "skip_missing_session_id",
            &json!({
                "payload_keys": payload_keys(payload),
                "has_claude_code_session_env": limux_env_value("CLAUDE_CODE_SESSION_ID").is_some(),
                "has_claude_session_env": limux_env_value("CLAUDE_SESSION_ID").is_some(),
                "has_hermes_session_env": limux_env_value("HERMES_SESSION_ID").is_some(),
            }),
        );
        return Ok(());
    };

    let store = agent_hooks::AgentHookSessionStore::new(agent);
    match agent_hook_persistence_action(event) {
        AgentHookPersistenceAction::Remove => {
            let result = store.remove(&session_id);
            if result.is_ok() {
                write_agent_hook_debug(
                    agent,
                    event,
                    "removed",
                    &json!({
                        "session_id": session_id,
                        "payload_keys": payload_keys(payload),
                    }),
                );
            }
            return result;
        }
        AgentHookPersistenceAction::Preserve => {
            write_agent_hook_debug(
                agent,
                event,
                "preserved",
                &json!({
                    "session_id": session_id,
                    "payload_keys": payload_keys(payload),
                }),
            );
            return Ok(());
        }
        AgentHookPersistenceAction::Upsert => {}
    }

    let workspace_id = parse_opt(args, "--workspace")
        .or_else(|| limux_env_value("LIMUX_WORKSPACE_ID"))
        .filter(|value| !value.trim().is_empty());
    let surface_id = parse_opt(args, "--surface")
        .or_else(|| limux_env_value("LIMUX_SURFACE_ID"))
        .filter(|value| !value.trim().is_empty());
    let (Some(workspace_id), Some(surface_id)) = (workspace_id, surface_id) else {
        write_agent_hook_debug(
            agent,
            event,
            "skip_missing_limux_target",
            &json!({
                "session_id": session_id,
                "has_workspace_arg": parse_opt(args, "--workspace").is_some(),
                "has_surface_arg": parse_opt(args, "--surface").is_some(),
                "has_workspace_env": limux_env_value("LIMUX_WORKSPACE_ID").is_some(),
                "has_surface_env": limux_env_value("LIMUX_SURFACE_ID").is_some(),
                "payload_keys": payload_keys(payload),
            }),
        );
        return Ok(());
    };

    let existing = store.lookup(&session_id)?;
    let cwd = hook_str(payload, &["cwd", "working_directory", "directory"])
        .map(str::to_string)
        .or_else(|| existing.as_ref().and_then(|record| record.cwd.clone()));
    let pid = hook_str(payload, &["pid"])
        .and_then(|value| value.parse::<u32>().ok())
        .or_else(|| agent_ancestor_pid(agent))
        .or_else(|| existing.as_ref().and_then(|record| record.pid));
    let launch_command = agent_hooks::launch_record_from_env(agent, cwd.as_deref()).or_else(|| {
        existing
            .as_ref()
            .and_then(|record| record.launch_command.clone())
    });

    let record = agent_hooks::AgentHookSessionRecord {
        session_id,
        workspace_id,
        surface_id,
        cwd,
        pid,
        launch_command,
        updated_at: agent_hooks::now_seconds(),
    };
    let result = store.upsert(record);
    if result.is_ok() {
        write_agent_hook_debug(
            agent,
            event,
            "upserted",
            &json!({
                "payload_keys": payload_keys(payload),
            }),
        );
    }
    result
}

fn hook_session_id(payload: &Value) -> Option<String> {
    hook_str(payload, &["session_id", "sessionId", "sessionID"])
        .map(str::to_string)
        .or_else(|| limux_env_value("CLAUDE_CODE_SESSION_ID"))
        .or_else(|| limux_env_value("CLAUDE_SESSION_ID"))
        .or_else(|| limux_env_value("HERMES_SESSION_ID"))
        .or_else(|| hook_session_id_from_transcript(payload))
        .filter(|value| !value.trim().is_empty())
}

fn hook_session_id_from_transcript(payload: &Value) -> Option<String> {
    let transcript = hook_str(
        payload,
        &["transcript_path", "transcriptPath", "transcript"],
    )?;
    Path::new(transcript)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(str::to_string)
        .filter(|value| !value.trim().is_empty())
}

fn payload_keys(payload: &Value) -> Vec<String> {
    payload
        .as_object()
        .map(|object| object.keys().cloned().collect())
        .unwrap_or_default()
}

fn write_agent_hook_debug(
    agent: agent_hooks::AgentKind,
    event: &str,
    outcome: &str,
    details: &Value,
) {
    let Some(dir) = agent_hook_debug_dir() else {
        return;
    };
    if fs::create_dir_all(&dir).is_err() {
        return;
    }
    let path = dir.join("agent-hook-debug.jsonl");
    let line = json!({
        "time": agent_hooks::now_seconds(),
        "agent": agent.store_name(),
        "event": event,
        "outcome": outcome,
        "details": details,
    });
    if let Ok(mut encoded) = serde_json::to_vec(&line) {
        encoded.push(b'\n');
        let _ = append_debug_line(&path, &encoded);
    }
}

fn agent_hook_debug_dir() -> Option<PathBuf> {
    if let Some(dir) = env::var_os("LIMUX_AGENT_HOOK_STATE_DIR") {
        return Some(PathBuf::from(dir));
    }
    dirs::state_dir()
        .map(|dir| dir.join("limux"))
        .or_else(|| dirs::home_dir().map(|home| home.join(".local/state/limux")))
}

fn append_debug_line(path: &Path, bytes: &[u8]) -> Result<()> {
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("failed to open {}", path.display()))?;
    file.write_all(bytes)
        .with_context(|| format!("failed to append {}", path.display()))
}

fn limux_env_value(name: &str) -> Option<String> {
    env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| ancestor_env_value(name))
}

#[cfg(target_os = "linux")]
fn agent_ancestor_pid(agent: agent_hooks::AgentKind) -> Option<u32> {
    let needle = agent.store_name();
    let mut pid = std::process::id();
    for _ in 0..8 {
        let parent = proc_parent_pid(pid)?;
        if parent <= 1 || parent == pid {
            return None;
        }
        if proc_identity_contains(parent, needle) {
            return Some(parent);
        }
        pid = parent;
    }
    None
}

#[cfg(not(target_os = "linux"))]
fn agent_ancestor_pid(_agent: agent_hooks::AgentKind) -> Option<u32> {
    None
}

#[cfg(target_os = "linux")]
fn proc_identity_contains(pid: u32, needle: &str) -> bool {
    let needle = needle.to_ascii_lowercase();
    proc_cmdline(pid)
        .or_else(|| fs::read_to_string(format!("/proc/{pid}/comm")).ok())
        .map(|value| value.to_ascii_lowercase().contains(&needle))
        .unwrap_or(false)
}

#[cfg(target_os = "linux")]
fn proc_cmdline(pid: u32) -> Option<String> {
    let raw = fs::read(format!("/proc/{pid}/cmdline")).ok()?;
    let parts = raw
        .split(|byte| *byte == 0)
        .filter(|part| !part.is_empty())
        .filter_map(|part| std::str::from_utf8(part).ok())
        .collect::<Vec<_>>();
    (!parts.is_empty()).then(|| parts.join(" "))
}

#[cfg(target_os = "linux")]
fn ancestor_env_value(name: &str) -> Option<String> {
    let mut pid = std::process::id();
    for _ in 0..8 {
        let parent = proc_parent_pid(pid)?;
        if parent <= 1 || parent == pid {
            return None;
        }
        if let Some(value) = proc_env_value(parent, name) {
            return Some(value);
        }
        pid = parent;
    }
    None
}

#[cfg(not(target_os = "linux"))]
fn ancestor_env_value(_name: &str) -> Option<String> {
    None
}

#[cfg(target_os = "linux")]
fn proc_parent_pid(pid: u32) -> Option<u32> {
    let stat = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    parse_proc_stat_parent_pid(&stat)
}

#[cfg(target_os = "linux")]
fn parse_proc_stat_parent_pid(stat: &str) -> Option<u32> {
    let close = stat.rfind(')')?;
    let mut fields = stat.get(close + 2..)?.split_whitespace();
    fields.next()?;
    fields.next()?.parse().ok()
}

#[cfg(target_os = "linux")]
fn proc_env_value(pid: u32, name: &str) -> Option<String> {
    let environ = fs::read(format!("/proc/{pid}/environ")).ok()?;
    env_value_from_environ(&environ, name)
}

fn env_value_from_environ(environ: &[u8], name: &str) -> Option<String> {
    let prefix = format!("{name}=");
    environ
        .split(|byte| *byte == 0)
        .filter_map(|part| std::str::from_utf8(part).ok())
        .find_map(|entry| entry.strip_prefix(&prefix).map(str::to_string))
        .filter(|value| !value.trim().is_empty())
}

async fn run_hooks_command(
    client: &mut Client,
    args: &[String],
    json_output: bool,
) -> Result<CommandOutput> {
    let Some(first) = args.first().map(String::as_str) else {
        bail!(
            "Usage: limux hooks setup [agent]|uninstall [agent]|<agent> install|uninstall|<event>"
        );
    };

    match first {
        "setup" | "install" => {
            let target = parse_opt(args, "--agent").or_else(|| positional_arg(args, 1));
            let installed = install_hook_targets(target.as_deref())?;
            return hooks_summary_output("installed", installed, json_output);
        }
        "uninstall" => {
            let target = parse_opt(args, "--agent").or_else(|| positional_arg(args, 1));
            let changed = uninstall_hook_targets(target.as_deref())?;
            return hooks_summary_output("uninstalled", changed, json_output);
        }
        _ => {}
    }

    let agent = agent_hooks::AgentKind::from_hook_name(first)
        .ok_or_else(|| anyhow!("unknown hooks target: {first}"))?;
    let rest = &args[1..];
    match rest.first().map(String::as_str) {
        Some("install") => {
            install_hook_target(agent)?;
            hooks_summary_output(
                "installed",
                vec![agent.store_name().to_string()],
                json_output,
            )
        }
        Some("uninstall") => {
            uninstall_hook_target(agent)?;
            hooks_summary_output(
                "uninstalled",
                vec![agent.store_name().to_string()],
                json_output,
            )
        }
        _ => {
            let payload = run_agent_hook(client, agent, rest).await?;
            if json_output {
                Ok(CommandOutput::Json(payload))
            } else {
                Ok(CommandOutput::Text("OK".to_string()))
            }
        }
    }
}

fn hooks_summary_output(
    action: &str,
    agents: Vec<String>,
    json_output: bool,
) -> Result<CommandOutput> {
    if json_output {
        Ok(CommandOutput::Json(json!({
            "action": action,
            "agents": agents,
        })))
    } else {
        Ok(CommandOutput::Text(format!(
            "OK {action}: {}",
            if agents.is_empty() {
                "none".to_string()
            } else {
                agents.join(", ")
            }
        )))
    }
}

fn install_hook_targets(target: Option<&str>) -> Result<Vec<String>> {
    let agents = target
        .map(|name| {
            agent_hooks::AgentKind::from_hook_name(name)
                .ok_or_else(|| anyhow!("unknown hooks target: {name}"))
                .map(|agent| vec![agent])
        })
        .transpose()?
        .unwrap_or_else(default_hook_targets);

    let mut installed = Vec::new();
    for agent in agents {
        install_hook_target(agent)?;
        installed.push(agent.store_name().to_string());
    }
    Ok(installed)
}

fn uninstall_hook_targets(target: Option<&str>) -> Result<Vec<String>> {
    let agents = target
        .map(|name| {
            agent_hooks::AgentKind::from_hook_name(name)
                .ok_or_else(|| anyhow!("unknown hooks target: {name}"))
                .map(|agent| vec![agent])
        })
        .transpose()?
        .unwrap_or_else(default_hook_targets);

    let mut changed = Vec::new();
    for agent in agents {
        uninstall_hook_target(agent)?;
        changed.push(agent.store_name().to_string());
    }
    Ok(changed)
}

fn default_hook_targets() -> Vec<agent_hooks::AgentKind> {
    vec![
        agent_hooks::AgentKind::Codex,
        agent_hooks::AgentKind::Claude,
        agent_hooks::AgentKind::Gemini,
    ]
}

fn install_hook_target(agent: agent_hooks::AgentKind) -> Result<()> {
    match agent {
        agent_hooks::AgentKind::Codex => install_json_hooks(
            &codex_hooks_path(),
            agent,
            &[
                ("SessionStart", "session-start", None),
                ("UserPromptSubmit", "prompt-submit", None),
                ("Stop", "stop", None),
            ],
        ),
        agent_hooks::AgentKind::Claude => install_json_hooks(
            &claude_settings_path(),
            agent,
            &[
                ("SessionStart", "session-start", None),
                ("UserPromptSubmit", "prompt-submit", None),
                ("Stop", "stop", None),
                ("Notification", "stop", None),
                ("SessionEnd", "session-end", None),
            ],
        ),
        agent_hooks::AgentKind::OpenCode => install_opencode_plugin(),
        agent_hooks::AgentKind::Gemini => install_json_hooks(
            &gemini_settings_path(),
            agent,
            &[
                ("SessionStart", "session-start", None),
                ("BeforeAgent", "prompt-submit", None),
                ("AfterAgent", "stop", None),
                ("SessionEnd", "session-end", None),
            ],
        ),
        agent_hooks::AgentKind::Hermes => bail!(
            "Hermes hook installation is owned by the Hermes-side lifecycle plugin; Limux only provides `limux hooks hermes <event>` and `limux hermes-hook` receivers"
        ),
    }
}

fn uninstall_hook_target(agent: agent_hooks::AgentKind) -> Result<()> {
    match agent {
        agent_hooks::AgentKind::Codex => uninstall_json_hooks(&codex_hooks_path(), agent),
        agent_hooks::AgentKind::Claude => uninstall_json_hooks(&claude_settings_path(), agent),
        agent_hooks::AgentKind::OpenCode => {
            let path = opencode_plugin_path();
            if path.exists() {
                fs::remove_file(&path)
                    .with_context(|| format!("failed to remove {}", path.display()))?;
            }
            opencode_config_unregister_plugin()
        }
        agent_hooks::AgentKind::Gemini => uninstall_json_hooks(&gemini_settings_path(), agent),
        agent_hooks::AgentKind::Hermes => Ok(()),
    }
}

fn install_json_hooks(
    path: &Path,
    agent: agent_hooks::AgentKind,
    events: &[HookInstallSpec],
) -> Result<()> {
    let mut root = read_json_object(path)?;
    let hooks = root
        .entry("hooks".to_string())
        .or_insert_with(|| Value::Object(Map::new()));
    let hooks = hooks
        .as_object_mut()
        .ok_or_else(|| anyhow!("{} has non-object hooks field", path.display()))?;
    let marker = hook_marker(agent);
    for value in hooks.values_mut() {
        if let Some(entries) = value.as_array_mut() {
            entries.retain(|entry| !json_value_contains(entry, marker));
        }
    }
    hooks.retain(|_, value| {
        value
            .as_array()
            .map(|entries| !entries.is_empty())
            .unwrap_or(true)
    });

    for (agent_event, limux_event, matcher) in events {
        let entries = hooks
            .entry((*agent_event).to_string())
            .or_insert_with(|| Value::Array(Vec::new()));
        let entries = entries
            .as_array_mut()
            .ok_or_else(|| anyhow!("{} hook {agent_event} is not an array", path.display()))?;
        entries.retain(|entry| !json_value_contains(entry, marker));
        let mut entry = json!({
            "hooks": [{
                "type": "command",
                "command": hook_command(agent, limux_event)?,
                "statusMessage": hook_status_message(agent_event),
                "timeout": hook_timeout(agent)
            }]
        });
        if let Some(matcher) = matcher {
            entry["matcher"] = Value::String((*matcher).to_string());
        } else if matches!(agent, agent_hooks::AgentKind::Claude) {
            entry["matcher"] = Value::String("*".to_string());
        }
        entries.push(entry);
    }

    write_json_object(path, &root)
}

fn hook_status_message(agent_event: &str) -> &'static str {
    match canonical_agent_hook_display_event(agent_event) {
        AgentHookDisplayEvent::Notification => "limux notify",
        AgentHookDisplayEvent::Stop => "limux stop hook",
        AgentHookDisplayEvent::SessionStart => "limux session start",
        AgentHookDisplayEvent::SessionEnd => "limux session end",
        AgentHookDisplayEvent::ToolUse => "limux tool hook",
        AgentHookDisplayEvent::UserPromptSubmit => "limux prompt hook",
        AgentHookDisplayEvent::Other => "limux hook",
    }
}

fn hook_timeout(agent: agent_hooks::AgentKind) -> u64 {
    match agent {
        agent_hooks::AgentKind::Claude => 2,
        agent_hooks::AgentKind::Codex | agent_hooks::AgentKind::Gemini => 5000,
        agent_hooks::AgentKind::OpenCode => 0,
        agent_hooks::AgentKind::Hermes => 5000,
    }
}

fn uninstall_json_hooks(path: &Path, agent: agent_hooks::AgentKind) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let mut root = read_json_object(path)?;
    if let Some(hooks) = root.get_mut("hooks").and_then(Value::as_object_mut) {
        let marker = hook_marker(agent);
        for value in hooks.values_mut() {
            if let Some(entries) = value.as_array_mut() {
                entries.retain(|entry| !json_value_contains(entry, marker));
            }
        }
        hooks.retain(|_, value| {
            value
                .as_array()
                .map(|entries| !entries.is_empty())
                .unwrap_or(true)
        });
    }
    write_json_object(path, &root)
}

fn install_opencode_plugin() -> Result<()> {
    let path = opencode_plugin_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(&path, opencode_plugin_source()?).context("failed to write OpenCode plugin")?;
    opencode_config_register_plugin(&path)
}

fn opencode_config_register_plugin(plugin_path: &Path) -> Result<()> {
    let config_path = opencode_config_path();
    let mut root = read_json_object(&config_path)?;
    let plugins = root
        .entry("plugin".to_string())
        .or_insert_with(|| Value::Array(Vec::new()));
    let plugins = plugins
        .as_array_mut()
        .ok_or_else(|| anyhow!("{} has non-array plugin field", config_path.display()))?;
    let plugin_str = plugin_path.to_string_lossy().into_owned();
    if !plugins.iter().any(|v| v.as_str() == Some(&plugin_str)) {
        plugins.push(Value::String(plugin_str));
    }
    write_json_object(&config_path, &root)
}

fn opencode_config_unregister_plugin() -> Result<()> {
    let config_path = opencode_config_path();
    if !config_path.exists() {
        return Ok(());
    }
    let plugin_path = opencode_plugin_path();
    let plugin_str = plugin_path.to_string_lossy().into_owned();
    let mut root = read_json_object(&config_path)?;
    if let Some(plugins) = root.get_mut("plugin").and_then(Value::as_array_mut) {
        plugins.retain(|v| v.as_str() != Some(&plugin_str));
    }
    write_json_object(&config_path, &root)
}

fn hook_command(agent: agent_hooks::AgentKind, event: &str) -> Result<String> {
    let disable_var = format!(
        "LIMUX_{}_HOOKS_DISABLED",
        agent.store_name().to_ascii_uppercase()
    );
    let limux_command = hook_cli_command()?;
    Ok(format!(
        "[ \"${{{disable_var}:-}}\" != \"1\" ] && {limux_command} --json hooks {} {} || echo '{{\"continue\":true,\"suppressOutput\":false}}'",
        agent.store_name(),
        event
    ))
}

fn hook_cli_command() -> Result<String> {
    let exe = env::current_exe().context("failed to resolve current executable")?;
    let file_name = exe
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    if file_name == "limux-cli" {
        return Ok(shell_single_quote(&exe.to_string_lossy()));
    }
    Ok("limux".to_string())
}

fn opencode_plugin_cli_command() -> Result<String> {
    let exe = env::current_exe().context("failed to resolve current executable")?;
    let file_name = exe
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    if file_name == "limux-cli" {
        return Ok(exe.to_string_lossy().to_string());
    }
    Ok("limux".to_string())
}

fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn shell_command_arg(value: &str) -> String {
    if value.chars().any(|ch| ch.is_ascii_control()) {
        return shell_ansi_c_quote(value);
    }
    shell_single_quote(value)
}

fn shell_ansi_c_quote(value: &str) -> String {
    let mut out = String::from("$'");
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\'' => out.push_str("\\'"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_ascii_control() => {
                use std::fmt::Write as _;
                let _ = write!(out, "\\x{:02x}", ch as u32);
            }
            ch => out.push(ch),
        }
    }
    out.push('\'');
    out
}

fn new_pane_shell_command(direction: &str, command: &str) -> String {
    format!(
        "limux new-pane --direction {} --command {}",
        direction,
        shell_command_arg(command)
    )
}

fn bootstrap_prompt_value(value: &str) -> String {
    value.chars().flat_map(char::escape_default).collect()
}

fn is_agent_team_bootstrap_display_spoof_char(ch: char) -> bool {
    matches!(
        ch,
        '\u{200b}'
            | '\u{200c}'
            | '\u{200d}'
            | '\u{200e}'
            | '\u{200f}'
            | '\u{202a}'..='\u{202e}'
            | '\u{2066}'..='\u{2069}'
    )
}

fn validate_agent_team_bootstrap_prompt(text: &str) -> Result<()> {
    validate_terminal_text_arg("agent-team bootstrap prompt", text)?;
    if text.contains('\r') {
        bail!("agent-team bootstrap prompt contains CR");
    }
    if text.chars().any(|ch| ch == '\n' || ch == '\t') {
        bail!("agent-team bootstrap prompt must be a single line without LF or tab");
    }
    if let Some(ch) = text
        .chars()
        .find(|ch| is_agent_team_bootstrap_display_spoof_char(*ch))
    {
        bail!(
            "agent-team bootstrap prompt contains disallowed display-spoofing character U+{:04X}",
            ch as u32
        );
    }
    Ok(())
}

fn build_agent_team_bootstrap_prompt(
    agent: &str,
    protocol_path: &Path,
    roster_path: &Path,
    ledger_path: &Path,
) -> Result<String> {
    let prompt = format!(
        "You are {agent} in a Limux agent-team pane. Read the generated runtime protocol file at {protocol_path}, the durable ownership/team roster at {roster_path}, and the durable review ledger at {ledger_path}, then read the authoritative instruction sources listed in the protocol file; use the protocol file for current surface IDs; do not treat LIMUX_AGENTS.md as copied AGENTS.md content; use limux send --surface for peer messages and limux notify for human input; record durable review decisions in the ledger; reply to the orchestrator when ready.",
        agent = bootstrap_prompt_value(agent),
        protocol_path = bootstrap_prompt_value(&protocol_path.to_string_lossy()),
        roster_path = bootstrap_prompt_value(&roster_path.to_string_lossy()),
        ledger_path = bootstrap_prompt_value(&ledger_path.to_string_lossy()),
    );
    validate_agent_team_bootstrap_prompt(&prompt)?;
    Ok(prompt)
}

fn hook_marker(agent: agent_hooks::AgentKind) -> &'static str {
    match agent {
        agent_hooks::AgentKind::Claude => "hooks claude",
        agent_hooks::AgentKind::Codex => "hooks codex",
        agent_hooks::AgentKind::OpenCode => "hooks opencode",
        agent_hooks::AgentKind::Gemini => "hooks gemini",
        agent_hooks::AgentKind::Hermes => "hooks hermes",
    }
}

fn read_json_object(path: &Path) -> Result<Map<String, Value>> {
    if !path.exists() {
        return Ok(Map::new());
    }
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    if raw.trim().is_empty() {
        return Ok(Map::new());
    }
    let value: Value = serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    value
        .as_object()
        .cloned()
        .ok_or_else(|| anyhow!("{} must contain a JSON object", path.display()))
}

fn write_json_object(path: &Path, object: &Map<String, Value>) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let temp = path.with_extension(format!("json.{}.tmp", std::process::id()));
    let encoded = serde_json::to_vec_pretty(object).context("failed to encode hook config")?;
    fs::write(&temp, encoded).with_context(|| format!("failed to write {}", temp.display()))?;
    fs::rename(&temp, path).with_context(|| format!("failed to replace {}", path.display()))
}

fn json_value_contains(value: &Value, needle: &str) -> bool {
    match value {
        Value::String(value) => value.contains(needle),
        Value::Array(values) => values
            .iter()
            .any(|value| json_value_contains(value, needle)),
        Value::Object(map) => map.values().any(|value| json_value_contains(value, needle)),
        _ => false,
    }
}

fn codex_hooks_path() -> PathBuf {
    env::var_os("CODEX_HOME")
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|home| home.join(".codex")))
        .unwrap_or_else(|| PathBuf::from(".codex"))
        .join("hooks.json")
}

fn claude_settings_path() -> PathBuf {
    env::var_os("CLAUDE_CONFIG_DIR")
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|home| home.join(".claude")))
        .unwrap_or_else(|| PathBuf::from(".claude"))
        .join("settings.json")
}

fn gemini_settings_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".gemini/settings.json")
}

fn opencode_config_dir() -> PathBuf {
    env::var_os("OPENCODE_CONFIG_DIR")
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|home| home.join(".config/opencode")))
        .unwrap_or_else(|| PathBuf::from(".config/opencode"))
}

fn opencode_plugin_path() -> PathBuf {
    opencode_config_dir().join("plugins/limux-session.js")
}

fn opencode_config_path() -> PathBuf {
    opencode_config_dir().join("config.json")
}

fn opencode_plugin_source() -> Result<String> {
    opencode_plugin_source_with_command(&opencode_plugin_cli_command()?)
}

fn opencode_plugin_source_with_command(limux_command: &str) -> Result<String> {
    let limux_command_json =
        serde_json::to_string(limux_command).context("failed to encode OpenCode hook command")?;
    Ok(
        r#"// limux-opencode-session-plugin v2
// Installed by `limux hooks opencode install`. Do not edit manually.

import { spawnSync } from "node:child_process";
import { appendFileSync, mkdirSync } from "node:fs";
import { join } from "node:path";

const LIMUX_COMMAND = __LIMUX_COMMAND__;

function debug(outcome, details = {}) {
  if (process.env.LIMUX_OPENCODE_HOOK_DEBUG !== "1" && outcome !== "spawn_failed") return;
  try {
    const dir = process.env.LIMUX_AGENT_HOOK_STATE_DIR || (process.env.XDG_STATE_HOME ? join(process.env.XDG_STATE_HOME, "limux") : join(process.env.HOME || ".", ".local/state/limux"));
    mkdirSync(dir, { recursive: true });
    appendFileSync(join(dir, "opencode-plugin-debug.jsonl"), JSON.stringify({
      time: Date.now() / 1000,
      outcome,
      details
    }) + "\n");
  } catch (_) {}
}

function firstString(...values) {
  for (const value of values) {
    if (typeof value === "string" && value.trim().length > 0) return value.trim();
  }
  return null;
}

function props(event) {
  return (event && typeof event === "object" && event.properties) || {};
}

function data(event) {
  return (event && typeof event === "object" && event.data) || {};
}

function info(event) {
  const p = props(event);
  const d = data(event);
  return (p.info && typeof p.info === "object" && p.info) || (d.info && typeof d.info === "object" && d.info) || {};
}

function eventType(event) {
  const raw = firstString(event && event.type, event && event.name);
  if (!raw) return null;
  if (raw === "sync") return firstString(event && event.name);
  return raw.endsWith(".1") ? raw.slice(0, -2) : raw;
}

function sessionId(event) {
  const p = props(event);
  const d = data(event);
  const i = info(event);
  return firstString(p.sessionID, p.sessionId, p.session_id, d.sessionID, d.sessionId, d.session_id, i.id, event && event.sessionID, event && event.sessionId);
}

function cwd(ctx, event) {
  const p = props(event);
  const d = data(event);
  const i = info(event);
  return firstString(p.cwd, p.directory, d.cwd, d.directory, i.directory, i.path, ctx && ctx.directory, process.cwd());
}

function launchExecutable() {
  return firstString(process.env.LIMUX_OPENCODE_EXECUTABLE, "opencode");
}

function send(kind, ctx, event) {
  if (process.env.LIMUX_OPENCODE_HOOKS_DISABLED === "1") {
    debug("skip_disabled", { kind });
    return;
  }
  if (!process.env.LIMUX_SURFACE_ID) {
    debug("skip_missing_surface", { kind, type: eventType(event), hasWorkspace: !!process.env.LIMUX_WORKSPACE_ID });
    return;
  }
  const sid = sessionId(event);
  if (!sid) {
    debug("skip_missing_session", { kind, type: eventType(event), keys: Object.keys(event || {}) });
    return;
  }
  const type = eventType(event);
  const payload = {
    session_id: sid,
    cwd: cwd(ctx, event),
    hook_event_name: type,
    event: type
  };
  try {
    const command = process.env.LIMUX_BIN || LIMUX_COMMAND;
    const result = spawnSync(command, ["hooks", "opencode", kind], {
      input: JSON.stringify(payload),
      encoding: "utf8",
      stdio: ["pipe", "ignore", "ignore"],
      timeout: 5000,
      env: {
        ...process.env,
        LIMUX_AGENT_LAUNCH_ARGV: launchExecutable(),
        LIMUX_AGENT_LAUNCH_EXECUTABLE: launchExecutable(),
        LIMUX_AGENT_LAUNCH_CWD: cwd(ctx, event)
      }
    });
    debug("spawned", { kind, type, status: result.status, error: result.error && String(result.error), command });
  } catch (error) {
    debug("spawn_failed", { kind, type, error: String(error) });
  }
}

const limuxSessionRestore = async (ctx) => {
  debug("plugin_started", { directory: ctx && ctx.directory, hasSurface: !!process.env.LIMUX_SURFACE_ID, hasWorkspace: !!process.env.LIMUX_WORKSPACE_ID });
  return {
    event: async ({ event }) => {
    const type = eventType(event);
    debug("event", { type, rawType: event && event.type, rawName: event && event.name });
    if (!type) return;
    if (type === "session.created") send("session-start", ctx, event);
    if (type === "session.idle" || type === "session.updated" || type === "session.status" || type === "session.compacted") send("prompt-submit", ctx, event);
    if (type === "session.error") send("session-end", ctx, event);
    if (type === "session.deleted") send("cleanup", ctx, event);
    }
  };
};

export const LimuxSessionRestore = limuxSessionRestore;
export default limuxSessionRestore;
"#
        .replace("__LIMUX_COMMAND__", &limux_command_json),
    )
}

async fn run_new_workspace(client: &mut Client, args: &[String]) -> Result<Value> {
    let cwd = parse_opt(args, "--cwd");
    let command = parse_opt(args, "--command");
    if let Some(command) = command.as_ref() {
        validate_terminal_text_arg("workspace.create command", command)?;
    }
    let original = resolve_current_workspace(client).await?;

    let mut params = Map::new();
    if let Some(cwd_value) = cwd.as_ref() {
        params.insert("cwd".to_string(), Value::String(cwd_value.clone()));
    }
    if let Some(command) = command.clone() {
        params.insert("command".to_string(), Value::String(command));
    }

    let created = client
        .call("workspace.create", Value::Object(params))
        .await
        .context("workspace.create failed")?;

    let _ = client
        .call("workspace.select", json!({ "workspace_id": original }))
        .await;

    Ok(created)
}

// ---------------------------------------------------------------------------
// `limux agent-team` — spin up a multi-agent collaboration workspace.
// ---------------------------------------------------------------------------
//
// Creates ONE workspace and one pane per requested agent (codex / claude /
// opencode / gemini / hermes), launches each agent's CLI in its pane, captures the
// pane/surface IDs, and writes LIMUX_AGENTS.md by default in the shared cwd
// describing the XML-tagged message protocol and the peer directory so agents
// can message each other. Use --protocol-path to choose a different output.
//
// The protocol file codifies:
//   To send a message to a peer, run from any terminal:
//     limux send --surface <peer-surface-id> \\
//       $'<agent-msg from="<me>" to="<peer>" ts="<iso-8601>">\\n...\\n</agent-msg>\\n'
//
// Peers read their own terminals normally — the text appears at the prompt.
// Each agent should watch for <agent-msg from="..."> blocks and reply with
// the same envelope targeted back.

fn agent_team_help_text() -> &'static str {
    "Usage: limux agent-team [--agents codex,claude[,opencode,gemini,hermes]] [--launch-mode direct|hcom] [--cwd <path>] [--protocol-path <path>] [--roster-path <path>] [--ledger-path <path>] [--force-protocol-overwrite] [--force-roster-overwrite] [--no-launch] [--no-bootstrap] [--dry-run]\n\nSplits the active Limux workspace into one pane per agent, writes LIMUX_AGENTS.md, and seeds LIMUX_TEAM_ROSTER.md plus LIMUX_REVIEW_LEDGER.md when missing.\n\nSafety:\n  --help is informational only and never contacts the running host.\n  --dry-run previews files without contacting the running host."
}

/// Built-in agent launcher commands. Chosen to match the CLIs the user
/// actually has installed (see README); the launch command is what gets
/// typed into the new workspace's terminal, so it also works as a fallback
/// shell command if the CLI isn't in PATH yet.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AgentLaunchMode {
    Direct,
    Hcom,
}

impl AgentLaunchMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Direct => "direct",
            Self::Hcom => "hcom",
        }
    }
}

fn parse_agent_launch_mode(args: &[String], command_name: &str) -> Result<AgentLaunchMode> {
    match parse_opt(args, "--launch-mode")
        .unwrap_or_else(|| "direct".to_string())
        .to_ascii_lowercase()
        .as_str()
    {
        "direct" => Ok(AgentLaunchMode::Direct),
        "hcom" => Ok(AgentLaunchMode::Hcom),
        other => bail!("{command_name}: --launch-mode must be one of direct|hcom, got {other:?}"),
    }
}

fn agent_launch_command_for_mode(
    agent: &str,
    launch_mode: AgentLaunchMode,
) -> Option<(&'static str, String)> {
    match agent.to_lowercase().as_str() {
        "codex" => Some(("codex", agent_launch_command_text("codex", launch_mode))),
        "claude" | "claude-code" => {
            Some(("claude", agent_launch_command_text("claude", launch_mode)))
        }
        "opencode" => Some((
            "opencode",
            agent_launch_command_text("opencode", launch_mode),
        )),
        "gemini" | "gemini-cli" => {
            Some(("gemini", agent_launch_command_text("gemini", launch_mode)))
        }
        "hermes" | "hermes-agent" | "hermes-cli" => {
            Some(("hermes", agent_launch_command_text("hermes", launch_mode)))
        }
        _ => None,
    }
}

fn agent_launch_command_text(agent: &str, launch_mode: AgentLaunchMode) -> String {
    match launch_mode {
        AgentLaunchMode::Direct => agent.to_string(),
        AgentLaunchMode::Hcom => format!("hcom {agent} --run-here"),
    }
}

async fn send_agent_team_bootstrap_prompt(
    client: &mut Client,
    workspace_id: &str,
    surface_id: &str,
    agent: &str,
    prompt: &str,
) -> Result<()> {
    validate_agent_team_bootstrap_prompt(prompt)?;

    let mut text_params = Map::new();
    text_params.insert(
        "workspace_id".to_string(),
        Value::String(workspace_id.to_string()),
    );
    text_params.insert(
        "surface_id".to_string(),
        Value::String(surface_id.to_string()),
    );
    text_params.insert("text".to_string(), Value::String(prompt.to_string()));

    let mut key_params = Map::new();
    key_params.insert(
        "workspace_id".to_string(),
        Value::String(workspace_id.to_string()),
    );
    key_params.insert(
        "surface_id".to_string(),
        Value::String(surface_id.to_string()),
    );
    key_params.insert("key".to_string(), Value::String("enter".to_string()));

    let mut last_error = None;
    for attempt in 0..AGENT_TEAM_BOOTSTRAP_RETRY_ATTEMPTS {
        match client
            .call("surface.send_text", Value::Object(text_params.clone()))
            .await
        {
            Ok(_) => {
                tokio::time::sleep(Duration::from_millis(100)).await;
                client
                    .call("surface.send_key", Value::Object(key_params.clone()))
                    .await
                    .with_context(|| {
                        format!(
                            "agent-team: bootstrap prompt submit failed for '{agent}' on surface {surface_id}"
                        )
                    })?;
                return Ok(());
            }
            Err(error) => {
                let message = format!("{error:#}");
                if message.contains("not ready for text input")
                    && attempt + 1 < AGENT_TEAM_BOOTSTRAP_RETRY_ATTEMPTS
                {
                    last_error = Some(message);
                    tokio::time::sleep(AGENT_TEAM_BOOTSTRAP_RETRY_INTERVAL).await;
                    continue;
                }
                return Err(error).with_context(|| {
                    format!(
                        "agent-team: bootstrap prompt failed for '{agent}' on surface {surface_id}"
                    )
                });
            }
        }
    }

    bail!(
        "agent-team: bootstrap prompt failed for '{agent}' on surface {surface_id}: {}",
        last_error.unwrap_or_else(|| "surface did not become ready".to_string())
    )
}

async fn run_agent_team(client: &mut Client, args: &[String]) -> Result<Value> {
    if parse_flag(args, "--help") {
        return Ok(json!({ "help": agent_team_help_text() }));
    }

    // Parse --agents codex,claude (default: codex,claude).
    let agents_raw = parse_opt(args, "--agents").unwrap_or_else(|| "codex,claude".to_string());
    let agents: Vec<String> = agents_raw
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if agents.is_empty() {
        bail!("agent-team: --agents is empty");
    }

    let cwd = parse_opt(args, "--cwd")
        .or_else(|| {
            env::current_dir()
                .ok()
                .map(|p| p.to_string_lossy().to_string())
        })
        .ok_or_else(|| anyhow!("agent-team: could not resolve --cwd"))?;

    // Optional: skip launching the CLIs (useful when the user wants to open
    // the agents manually) — still splits the panes + writes the protocol file.
    let no_launch = parse_flag(args, "--no-launch");
    let no_bootstrap = parse_flag(args, "--no-bootstrap");
    let dry_run = parse_flag(args, "--dry-run");
    let force_protocol_overwrite = parse_flag(args, "--force-protocol-overwrite");
    let force_roster_overwrite = parse_flag(args, "--force-roster-overwrite");
    let launch_mode = parse_agent_launch_mode(args, "agent-team")?;
    let bootstrap_enabled = !no_launch && !no_bootstrap;

    // Resolve the agent list up front so --dry-run can build a deterministic
    // peer table without touching the host.
    let resolved: Vec<(String, &'static str, String)> = agents
        .iter()
        .filter_map(|agent| {
            agent_launch_command_for_mode(agent, launch_mode)
                .map(|(name, launch)| (agent.clone(), name, launch))
        })
        .collect();
    for agent in &agents {
        if agent_launch_command_for_mode(agent, launch_mode).is_none() {
            eprintln!("agent-team: unknown agent '{agent}', skipping");
        }
    }
    if resolved.is_empty() {
        bail!("agent-team: no valid agents spawned");
    }

    let agents_md_path = resolve_agent_team_protocol_path(&cwd, args);
    let roster_path = resolve_agent_team_roster_path(&cwd, args);
    let ledger_path = resolve_agent_team_ledger_path(&cwd, args);
    validate_agent_team_output_paths_are_distinct(&agents_md_path, &roster_path, &ledger_path)?;
    validate_agent_team_protocol_file(&agents_md_path, force_protocol_overwrite)?;
    validate_agent_team_roster_file(&roster_path, force_roster_overwrite)?;
    validate_agent_team_durable_file_path(&ledger_path, "review ledger")?;
    let instruction_sources = discover_instruction_sources(Path::new(&cwd));

    if dry_run {
        let peers: Vec<(String, String, String, String)> = resolved
            .iter()
            .enumerate()
            .map(|(i, (_, name, launch))| {
                (
                    name.to_string(),
                    format!("<dry-run-pane-{i}>"),
                    format!("<dry-run-surface-{name}>"),
                    launch.clone(),
                )
            })
            .collect();
        let body = build_agents_md(
            &peers,
            &cwd,
            "<active-workspace>",
            "<dry-run-workspace>",
            "<dry-run-orchestrator>",
            AgentTeamCoordinationFiles {
                roster_path: &roster_path,
                ledger_path: &ledger_path,
            },
            &instruction_sources,
        );
        let roster_body = build_agent_team_roster_md(
            &peers,
            &cwd,
            "<active-workspace>",
            "<dry-run-workspace>",
            "<dry-run-orchestrator>",
            &agents_md_path,
            &ledger_path,
        );
        let ledger_body = build_agent_team_ledger_md(&cwd, &agents_md_path, &roster_path);
        write_agent_team_protocol_file(&agents_md_path, &body, force_protocol_overwrite)?;
        let roster_status = write_agent_team_roster_file_if_missing(
            &roster_path,
            &roster_body,
            force_roster_overwrite,
        )?;
        let ledger_status = create_agent_team_ledger_file_if_missing(&ledger_path, &ledger_body)?;
        return Ok(json!({
            "ok": true,
            "cwd": cwd,
            "workspace_name": "<active-workspace>",
            "workspace_id": Value::Null,
            "orchestrator_surface_id": Value::Null,
            "agents_md": agents_md_path.to_string_lossy(),
            "protocol_path": agents_md_path.to_string_lossy(),
            "roster_path": roster_path.to_string_lossy(),
            "ledger_path": ledger_path.to_string_lossy(),
            "roster": {
                "status": roster_status,
            },
            "ledger": {
                "status": ledger_status,
            },
            "dry_run": true,
            "no_launch": no_launch,
            "launch_mode": launch_mode.as_str(),
            "bootstrap": {
                "enabled": false,
                "status": "skipped",
            },
            "peers": peers
                .iter()
                .map(|(name, pane, surface, launch)| {
                    json!({
                        "agent": name,
                        "pane_id": pane,
                        "surface_id": surface,
                        "launch_command": launch,
                        "bootstrap": { "status": "skipped" },
                    })
                })
                .collect::<Vec<_>>(),
        }));
    }

    // 1. Resolve the orchestrator's workspace + pane. Prefer LIMUX_* env (set
    //    in every limux-spawned terminal) and fall back to the host's active
    //    focus so callers from a regular shell still work.
    let orchestrator_workspace = env::var("LIMUX_WORKSPACE_ID")
        .ok()
        .filter(|s| !s.is_empty());
    let orchestrator_surface_env = env::var("LIMUX_SURFACE_ID").ok().filter(|s| !s.is_empty());
    let orchestrator_pane_env = env::var("LIMUX_PANE_ID").ok().filter(|s| !s.is_empty());

    let workspace_id = match orchestrator_workspace.clone() {
        Some(id) => id,
        None => resolve_current_workspace(client)
            .await
            .context("agent-team: could not resolve active workspace; run from inside a limux pane or pass --workspace")?,
    };

    // 2. Discover the orchestrator pane's surface_id. If env didn't tell us,
    //    use the focused/first surface in the workspace.
    let surfaces = client
        .call(
            "surface.list",
            json!({ "workspace_id": workspace_id.clone() }),
        )
        .await
        .context("surface.list failed for active workspace")?;
    let surface_rows = surfaces
        .get("surfaces")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if surface_rows.is_empty() {
        bail!("agent-team: active workspace has no surfaces");
    }
    let orchestrator_surface = orchestrator_surface_env.clone().unwrap_or_else(|| {
        surface_rows
            .iter()
            .find(|row| row.get("focused").and_then(Value::as_bool) == Some(true))
            .and_then(|row| get_string(row, &["surface_id"]))
            .or_else(|| get_string(&surface_rows[0], &["surface_id"]))
            .unwrap_or_default()
    });
    if orchestrator_surface.is_empty() {
        bail!("agent-team: could not determine orchestrator surface");
    }
    let orchestrator_pane = orchestrator_pane_env.unwrap_or_else(|| {
        surface_rows
            .iter()
            .find(|row| {
                get_string(row, &["surface_id"]).as_deref() == Some(orchestrator_surface.as_str())
            })
            .and_then(|row| get_string(row, &["pane_id"]))
            .unwrap_or_default()
    });

    // 3. Workspace name (for the protocol header) — best-effort lookup.
    let workspace_name = client
        .call("workspace.list", json!({}))
        .await
        .ok()
        .and_then(|v| v.get("workspaces").and_then(Value::as_array).cloned())
        .and_then(|rows| {
            rows.into_iter().find(|row| {
                get_string(row, &["workspace_id", "id"]).as_deref() == Some(workspace_id.as_str())
            })
        })
        .and_then(|row| get_string(&row, &["name", "title"]))
        .unwrap_or_else(|| "active workspace".to_string());

    // 4. Split a pane per agent. Layout: agent[0] splits RIGHT of orchestrator,
    //    each subsequent agent splits DOWN of the previous agent — orchestrator
    //    keeps its full height on the left, peers stack top-to-bottom on the right.
    let mut peers: Vec<(String, String, String, String)> = Vec::new();
    let mut parent_surface = orchestrator_surface.clone();

    for (i, (_alias, name, launch)) in resolved.iter().enumerate() {
        let direction = if i == 0 { "right" } else { "down" };

        let mut params = Map::new();
        params.insert(
            "workspace_id".to_string(),
            Value::String(workspace_id.clone()),
        );
        params.insert(
            "surface_id".to_string(),
            Value::String(parent_surface.clone()),
        );
        params.insert(
            "direction".to_string(),
            Value::String(direction.to_string()),
        );
        params.insert("type".to_string(), Value::String("terminal".to_string()));
        if !no_launch {
            params.insert("command".to_string(), Value::String(launch.clone()));
        }

        let created = client
            .call("pane.create", Value::Object(params))
            .await
            .with_context(|| format!("pane.create failed for agent '{name}'"))?;
        let pane_id = get_string(&created, &["pane_id"])
            .ok_or_else(|| anyhow!("agent-team: pane.create for '{name}' returned no pane_id"))?;
        let surface_id = get_string(&created, &["surface_id"]).ok_or_else(|| {
            anyhow!("agent-team: pane.create for '{name}' returned no surface_id")
        })?;

        parent_surface = surface_id.clone();
        peers.push((name.to_string(), pane_id, surface_id, launch.clone()));
    }

    // 5. Write the generated protocol. Existing repo AGENTS.md files are
    //    preserved by default; pass --protocol-path to choose an explicit path.
    let body = build_agents_md(
        &peers,
        &cwd,
        &workspace_name,
        &workspace_id,
        &orchestrator_surface,
        AgentTeamCoordinationFiles {
            roster_path: &roster_path,
            ledger_path: &ledger_path,
        },
        &instruction_sources,
    );
    let roster_body = build_agent_team_roster_md(
        &peers,
        &cwd,
        &workspace_name,
        &workspace_id,
        &orchestrator_surface,
        &agents_md_path,
        &ledger_path,
    );
    let ledger_body = build_agent_team_ledger_md(&cwd, &agents_md_path, &roster_path);
    write_agent_team_protocol_file(&agents_md_path, &body, force_protocol_overwrite)?;
    let roster_status = write_agent_team_roster_file_if_missing(
        &roster_path,
        &roster_body,
        force_roster_overwrite,
    )?;
    let ledger_status = create_agent_team_ledger_file_if_missing(&ledger_path, &ledger_body)?;

    let mut bootstrap_results: Vec<(String, Option<String>)> = peers
        .iter()
        .map(|_| ("skipped".to_string(), None))
        .collect();
    if bootstrap_enabled {
        tokio::time::sleep(AGENT_TEAM_BOOTSTRAP_LAUNCH_SETTLE).await;
        for (index, (name, _pane_id, surface_id, _launch)) in peers.iter().enumerate() {
            let prompt = build_agent_team_bootstrap_prompt(
                name,
                &agents_md_path,
                &roster_path,
                &ledger_path,
            )?;
            match send_agent_team_bootstrap_prompt(client, &workspace_id, surface_id, name, &prompt)
                .await
            {
                Ok(()) => {
                    bootstrap_results[index] = ("sent".to_string(), None);
                }
                Err(error) => {
                    let message = format!("{error:#}");
                    bail!(
                        "{message}; bootstrap aborted after protocol write and pane creation, so earlier peers may already be launched or bootstrapped"
                    );
                }
            }
        }
    }
    let bootstrap_status = if !bootstrap_enabled {
        "skipped"
    } else {
        "sent"
    };

    Ok(json!({
        "ok": true,
        "cwd": cwd,
        "workspace_name": workspace_name,
        "workspace_id": workspace_id,
        "orchestrator_pane_id": orchestrator_pane,
        "orchestrator_surface_id": orchestrator_surface,
        "agents_md": agents_md_path.to_string_lossy(),
        "protocol_path": agents_md_path.to_string_lossy(),
        "roster_path": roster_path.to_string_lossy(),
        "ledger_path": ledger_path.to_string_lossy(),
        "roster": {
            "status": roster_status,
        },
        "ledger": {
            "status": ledger_status,
        },
        "dry_run": false,
        "no_launch": no_launch,
        "launch_mode": launch_mode.as_str(),
        "bootstrap": {
            "enabled": bootstrap_enabled,
            "status": bootstrap_status,
        },
        "peers": peers
            .iter()
            .enumerate()
            .map(|(index, (name, pane, surface, launch))| {
                let (status, error) = &bootstrap_results[index];
                json!({
                    "agent": name,
                    "pane_id": pane,
                    "surface_id": surface,
                    "launch_command": launch,
                    "bootstrap": {
                        "status": status,
                        "error": error,
                    },
                })
            })
            .collect::<Vec<_>>(),
    }))
}

fn resolve_agent_team_protocol_path(cwd: &str, args: &[String]) -> PathBuf {
    resolve_agent_team_output_path(
        cwd,
        args,
        "--protocol-path",
        AGENT_TEAM_DEFAULT_PROTOCOL_FILE,
    )
}

fn resolve_agent_team_roster_path(cwd: &str, args: &[String]) -> PathBuf {
    resolve_agent_team_output_path(cwd, args, "--roster-path", AGENT_TEAM_DEFAULT_ROSTER_FILE)
}

fn resolve_agent_team_ledger_path(cwd: &str, args: &[String]) -> PathBuf {
    resolve_agent_team_output_path(cwd, args, "--ledger-path", AGENT_TEAM_DEFAULT_LEDGER_FILE)
}

fn resolve_agent_team_output_path(
    cwd: &str,
    args: &[String],
    flag: &str,
    default_file: &str,
) -> PathBuf {
    let cwd_path = Path::new(cwd);
    if let Some(raw_path) = parse_opt(args, flag) {
        let path = PathBuf::from(raw_path);
        return if path.is_absolute() {
            path
        } else {
            cwd_path.join(path)
        };
    }

    cwd_path.join(default_file)
}

fn validate_agent_team_output_paths_are_distinct(
    protocol_path: &Path,
    roster_path: &Path,
    ledger_path: &Path,
) -> Result<()> {
    let paths = [
        (
            "protocol",
            protocol_path,
            comparable_agent_team_path(protocol_path),
        ),
        (
            "team roster",
            roster_path,
            comparable_agent_team_path(roster_path),
        ),
        (
            "review ledger",
            ledger_path,
            comparable_agent_team_path(ledger_path),
        ),
    ];
    for (left_index, (left_label, left_path, left_comparable)) in paths.iter().enumerate() {
        for (right_label, _right_path, right_comparable) in paths.iter().skip(left_index + 1) {
            if left_comparable == right_comparable {
                bail!(
                    "agent-team output paths must be distinct; {left_label} and {right_label} both resolve to {}",
                    left_path.display()
                );
            }
        }
    }
    Ok(())
}

fn comparable_agent_team_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    normalized.push(component.as_os_str());
                }
            }
            Component::Normal(part) => normalized.push(part),
            Component::RootDir | Component::Prefix(_) => normalized.push(component.as_os_str()),
        }
    }
    normalized
}

fn validate_agent_team_protocol_file(path: &Path, force: bool) -> Result<()> {
    validate_agent_team_generated_file(
        path,
        AGENT_TEAM_PROTOCOL_MARKER,
        force,
        "protocol",
        "--force-protocol-overwrite",
    )
}

fn validate_agent_team_roster_file(path: &Path, force: bool) -> Result<()> {
    validate_agent_team_durable_file_path(path, "team roster")?;
    if force && path.exists() {
        let existing = fs::read_to_string(path).with_context(|| {
            format!("failed to inspect existing team roster {}", path.display())
        })?;
        if !existing.contains(AGENT_TEAM_ROSTER_MARKER) {
            bail!(
                "refusing to replace existing unmarked team roster {}; move it aside or add the Limux roster marker before using --force-roster-overwrite",
                path.display()
            );
        }
    }
    Ok(())
}

fn validate_agent_team_generated_file(
    path: &Path,
    marker: &str,
    force: bool,
    label: &str,
    force_flag: &str,
) -> Result<()> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(()),
        Err(err) => {
            return Err(err).with_context(|| format!("failed to inspect {}", path.display()));
        }
    };

    let file_type = metadata.file_type();
    if file_type.is_symlink() {
        bail!(
            "refusing to write {label} path because it is a symlink: {}",
            path.display()
        );
    }
    if !file_type.is_file() {
        bail!(
            "refusing to write {label} path because it is not a regular file: {}",
            path.display()
        );
    }
    if force {
        return Ok(());
    }

    let existing = fs::read_to_string(path)
        .with_context(|| format!("failed to inspect existing {label} file {}", path.display()))?;
    if !existing.contains(marker) {
        bail!(
            "refusing to overwrite existing unmarked {label} file {}; rerun with {force_flag} only if this file is safe to replace",
            path.display()
        );
    }

    Ok(())
}

fn temporary_agent_team_output_path(path: &Path) -> Result<PathBuf> {
    let file_name = path
        .file_name()
        .ok_or_else(|| anyhow!("output path has no file name: {}", path.display()))?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let mut temp_name = file_name.to_os_string();
    temp_name.push(format!(".limux-tmp-{}-{nonce}", std::process::id()));
    Ok(path.with_file_name(temp_name))
}

fn write_agent_team_protocol_file(path: &Path, body: &str, force: bool) -> Result<()> {
    write_agent_team_generated_file(
        path,
        body,
        force,
        validate_agent_team_protocol_file,
        "protocol",
    )
}

fn write_agent_team_generated_file(
    path: &Path,
    body: &str,
    force: bool,
    validate: fn(&Path, bool) -> Result<()>,
    label: &str,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
    }
    validate(path, force)?;

    let temp_path = temporary_agent_team_output_path(path)?;
    fs::write(&temp_path, body)
        .with_context(|| format!("failed to write {}", temp_path.display()))?;
    validate(path, force)?;
    if let Err(err) = fs::rename(&temp_path, path) {
        let _ = fs::remove_file(&temp_path);
        return Err(err).with_context(|| {
            format!(
                "failed to move generated {label} from {} to {}",
                temp_path.display(),
                path.display()
            )
        });
    }
    Ok(())
}

fn create_agent_team_ledger_file_if_missing(path: &Path, body: &str) -> Result<&'static str> {
    create_agent_team_durable_file_if_missing(path, body, "review ledger")
}

fn create_agent_team_durable_file_if_missing(
    path: &Path,
    body: &str,
    label: &str,
) -> Result<&'static str> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
    }

    match validate_agent_team_durable_file_path(path, label) {
        Ok(()) if path.exists() => Ok("existing"),
        Ok(()) => match OpenOptions::new().write(true).create_new(true).open(path) {
            Ok(mut file) => {
                if let Err(err) = file.write_all(body.as_bytes()) {
                    let _ = fs::remove_file(path);
                    return Err(err)
                        .with_context(|| format!("failed to write {label} {}", path.display()));
                }
                Ok("created")
            }
            Err(err) if err.kind() == ErrorKind::AlreadyExists => {
                validate_agent_team_durable_file_path(path, label)?;
                Ok("existing")
            }
            Err(err) => {
                Err(err).with_context(|| format!("failed to create {label} {}", path.display()))
            }
        },
        Err(error) => Err(error),
    }
}

fn validate_agent_team_durable_file_path(path: &Path, label: &str) -> Result<()> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(()),
        Err(err) => {
            return Err(err).with_context(|| format!("failed to inspect {}", path.display()));
        }
    };
    let file_type = metadata.file_type();
    if file_type.is_symlink() {
        bail!(
            "refusing to use {label} path because it is a symlink: {}",
            path.display()
        );
    }
    if !file_type.is_file() {
        bail!(
            "refusing to use {label} path because it is not a regular file: {}",
            path.display()
        );
    }
    Ok(())
}

fn write_agent_team_roster_file_if_missing(
    path: &Path,
    body: &str,
    force: bool,
) -> Result<&'static str> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
    }
    validate_agent_team_roster_file(path, force)?;
    let existed = path.exists();

    if !force {
        return create_agent_team_durable_file_if_missing(path, body, "team roster");
    }

    let temp_path = temporary_agent_team_output_path(path)?;
    fs::write(&temp_path, body)
        .with_context(|| format!("failed to write {}", temp_path.display()))?;
    validate_agent_team_roster_file(path, force)?;
    if let Err(err) = fs::rename(&temp_path, path) {
        let _ = fs::remove_file(&temp_path);
        return Err(err).with_context(|| {
            format!(
                "failed to move generated roster seed from {} to {}",
                temp_path.display(),
                path.display()
            )
        });
    }

    if existed && force {
        Ok("replaced")
    } else {
        Ok("created")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct InstructionSource {
    name: &'static str,
    display_path: String,
    kind: String,
    modified_unix: Option<u64>,
    hash: Option<String>,
    note: Option<String>,
}

#[derive(Debug, Clone, Copy)]
struct AgentTeamCoordinationFiles<'a> {
    roster_path: &'a Path,
    ledger_path: &'a Path,
}

struct ReviewRequestMd<'a> {
    review_id: &'a str,
    artifact: &'a str,
    reviewer: &'a str,
    lens: &'a str,
    summary: &'a str,
    cwd: &'a Path,
    ledger_path: &'a Path,
    prompt: &'a str,
}

#[derive(Debug, Clone)]
struct PreparedReviewRequest {
    review_id: String,
    artifact: String,
    reviewer: String,
    lens: String,
    summary: String,
    ledger_path: PathBuf,
    prompt: String,
}

struct ReviewSpawnEvidenceMd<'a> {
    request: &'a PreparedReviewRequest,
    request_path: &'a Path,
    ledger_path: &'a Path,
    reviewer_pane_id: &'a str,
    reviewer_surface_id: &'a str,
    prompt_status: &'a str,
    capture_command: &'a str,
}

struct ReviewSpawnLedgerUpdate<'a> {
    evidence_path: &'a Path,
    reviewer_pane_id: &'a str,
    reviewer_surface_id: &'a str,
    prompt_status: &'a str,
    capture_command: &'a str,
}

fn discover_instruction_sources(cwd: &Path) -> Vec<InstructionSource> {
    AGENT_TEAM_INSTRUCTION_FILES
        .iter()
        .filter_map(|name| inspect_instruction_source(cwd, name))
        .collect()
}

fn inspect_instruction_source(cwd: &Path, name: &'static str) -> Option<InstructionSource> {
    let path = cwd.join(name);
    let display_path = format!("./{name}");
    let metadata = match fs::symlink_metadata(&path) {
        Ok(metadata) => metadata,
        Err(err) if err.kind() == ErrorKind::NotFound => return None,
        Err(err) => {
            return Some(InstructionSource {
                name,
                display_path,
                kind: "unreadable".to_string(),
                modified_unix: None,
                hash: None,
                note: Some(format!("metadata unavailable: {err}")),
            });
        }
    };

    let modified_unix = metadata.modified().ok().and_then(system_time_unix_seconds);
    let file_type = metadata.file_type();

    if file_type.is_symlink() {
        return Some(InstructionSource {
            name,
            display_path,
            kind: "symlink".to_string(),
            modified_unix,
            hash: None,
            note: Some("not hashed; symlink target was not read".to_string()),
        });
    }

    if !file_type.is_file() {
        return Some(InstructionSource {
            name,
            display_path,
            kind: "not a regular file".to_string(),
            modified_unix,
            hash: None,
            note: Some("not hashed".to_string()),
        });
    }

    let (hash, note) = match fs::read(&path) {
        Ok(bytes) => (Some(fnv1a64_hash(&bytes)), None),
        Err(err) => (None, Some(format!("not hashed: {err}"))),
    };

    Some(InstructionSource {
        name,
        display_path,
        kind: "regular file".to_string(),
        modified_unix,
        hash,
        note,
    })
}

fn system_time_unix_seconds(time: SystemTime) -> Option<u64> {
    time.duration_since(UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_secs())
}

fn fnv1a64_hash(bytes: &[u8]) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("fnv1a64:{hash:016x}")
}

fn markdown_table_cell(value: &str) -> String {
    value
        .replace('\n', " ")
        .replace('|', "\\|")
        .replace('`', "'")
}

fn markdown_path(path: &Path) -> String {
    markdown_table_cell(&path.to_string_lossy())
}

fn build_agent_team_roster_md(
    peers: &[(String, String, String, String)],
    cwd: &str,
    workspace_name: &str,
    _workspace_id: &str,
    _orchestrator_surface: &str,
    protocol_path: &Path,
    ledger_path: &Path,
) -> String {
    let mut out = String::new();
    out.push_str(AGENT_TEAM_ROSTER_MARKER);
    out.push('\n');
    out.push_str("# Limux Team Roster\n\n");
    out.push_str(
        "This durable file is seeded by `limux agent-team` when missing. Edit it\n\
         to track project owners, hcom names, related teams, durable handoffs,\n\
         and routing rules. Limux does not overwrite it during normal\n\
         regeneration.\n\n",
    );

    out.push_str("## Coordination Files\n\n");
    out.push_str("| Purpose | Path | Notes |\n");
    out.push_str("|---|---|---|\n");
    out.push_str(&format!(
        "| Runtime protocol | `{}` | Generated; current workspace peers and message protocol |\n",
        markdown_path(protocol_path)
    ));
    out.push_str(&format!(
        "| Local policy | `{}` | Optional durable team policy; Limux never creates or overwrites it |\n",
        markdown_table_cell(AGENT_TEAM_LOCAL_POLICY_FILE)
    ));
    out.push_str(&format!(
        "| Review ledger | `{}` | Append-only review findings, consensus decisions, and open risks |\n\n",
        markdown_path(ledger_path)
    ));

    out.push_str("## Projects\n\n");
    out.push_str("| Project ID | Name | Repo Path | Runtime Source | Lead | hcom Thread | Related Teams | Status |\n");
    out.push_str("|---|---|---|---|---|---|---|---|\n");
    out.push_str(&format!(
        "| `current` | `{}` | `{}` | `{}` | `unassigned` | `limux-agent-team` | `unassigned` | active |\n\n",
        markdown_table_cell(workspace_name),
        markdown_table_cell(cwd),
        markdown_path(protocol_path)
    ));

    out.push_str("## Agents\n\n");
    out.push_str(
        "| Project ID | Role | Runtime | hcom Name | Owner | Scope | Status | Handoff |\n",
    );
    out.push_str("|---|---|---|---|---|---|---|---|\n");
    out.push_str("| `current` | orchestrator | unknown | `unassigned` | `unassigned` | repo orchestration | active | `HANDOFF.md` |\n");
    for (name, _pane_id, _surface_id, _launch) in peers {
        out.push_str(&format!(
            "| `current` | peer | `{}` | `unassigned` | `unassigned` | repo peer work | active | `HANDOFF.md` |\n",
            markdown_table_cell(name)
        ));
    }
    out.push('\n');
    out.push_str(
        "Live workspace IDs, pane IDs, surface IDs, and launch commands are\n\
         volatile. Use the current generated runtime protocol file listed above\n\
         for live surface routing; keep durable ownership and cross-team routing\n\
         here.\n\n",
    );

    out.push_str("## Routing Rules\n\n");
    out.push_str("| Path/Topic | Owner | Preferred Channel | Fallback | Notes |\n");
    out.push_str("|---|---|---|---|---|\n");
    out.push_str("| `.` | `unassigned` | Limux surface / hcom 1:1 | review ledger entry | Replace this row with project-specific ownership. |\n\n");

    out.push_str("## Privacy Rules\n\n");
    out.push_str("- Do not store secrets, tokens, credentials, env dumps, browser cookies, or raw terminal transcripts here.\n");
    out.push_str(
        "- Prefer repo-relative paths before sharing this file outside the local machine.\n",
    );
    out.push_str(
        "- Treat surface IDs, pane IDs, workspace IDs, and hcom names as local topology.\n",
    );

    out
}

fn build_agent_team_ledger_md(cwd: &str, protocol_path: &Path, roster_path: &Path) -> String {
    let mut out = String::new();
    out.push_str(AGENT_TEAM_LEDGER_MARKER);
    out.push('\n');
    out.push_str("# Limux Review And Consensus Ledger\n\n");
    out.push_str(
        "Append review findings, consensus decisions, accepted risks, and\n\
         cross-team notification records here. Limux creates this file only\n\
         when it is missing and never overwrites existing entries.\n\n",
    );
    out.push_str("## Coordination\n\n");
    out.push_str(&format!("- Shared cwd: `{}`\n", markdown_table_cell(cwd)));
    out.push_str(&format!(
        "- Runtime protocol: `{}`\n",
        markdown_path(protocol_path)
    ));
    out.push_str(&format!(
        "- Team roster: `{}`\n\n",
        markdown_path(roster_path)
    ));

    out.push_str("## Entry Template\n\n");
    out.push_str("```markdown\n");
    out.push_str("## <UTC timestamp> - <review-id>\n\n");
    out.push_str("Status: pending | go | wait | no-go | superseded\n");
    out.push_str("Project: `current`\n");
    out.push_str("Thread: `limux-agent-team`\n");
    out.push_str("Artifact: `<path or commit/ref>`\n");
    out.push_str("Coordinator: `<agent or human>`\n\n");
    out.push_str("### Reviewers\n\n");
    out.push_str("| Reviewer | Runtime | hcom Name | Lens | Verdict | Evidence |\n");
    out.push_str("|---|---|---|---|---|---|\n\n");
    out.push_str("### Findings\n\n");
    out.push_str("| Severity | File | Line | Summary | Owner | Status |\n");
    out.push_str("|---|---|---:|---|---|---|\n\n");
    out.push_str("### Consensus\n\n");
    out.push_str("Decision: `<GO/WAIT/NO-GO plus rationale>`\n");
    out.push_str("Open risks: `<short list or none>`\n");
    out.push_str("Next actions: `<owner/action/status>`\n");
    out.push_str("Cross-team notifications: `<hcom targets or none>`\n");
    out.push_str("```\n\n");
    out.push_str("## Entries\n\n");
    out
}

async fn run_review_command(client: &mut Client, args: &[String]) -> Result<Value> {
    match args.first().map(String::as_str) {
        Some("prepare") => run_review_prepare(args),
        Some("spawn") => run_review_spawn(client, args).await,
        Some(subcommand) => bail!("unknown review subcommand: {subcommand}"),
        None => bail!("review requires a subcommand; try `review prepare` or `review spawn`"),
    }
}

fn run_review_prepare(raw_args: &[String]) -> Result<Value> {
    let args = if raw_args.first().map(String::as_str) == Some("prepare") {
        &raw_args[1..]
    } else {
        raw_args
    };

    let cwd_raw = parse_opt(args, "--cwd")
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let cwd = cwd_raw.canonicalize().with_context(|| {
        format!(
            "review prepare: could not resolve --cwd {}",
            cwd_raw.display()
        )
    })?;
    let cwd_string = cwd.to_string_lossy().to_string();

    let artifact = required_review_opt(args, "--artifact")?;
    let reviewer = required_review_opt(args, "--reviewer")?.to_ascii_lowercase();
    let lens = required_review_opt(args, "--lens")?.to_ascii_lowercase();
    let summary = required_review_opt(args, "--summary")?;
    let dry_run = parse_flag(args, "--dry-run");

    validate_review_field("review artifact", &artifact)?;
    validate_review_choice("reviewer", &reviewer, REVIEW_REVIEWERS)?;
    validate_review_choice("review lens", &lens, REVIEW_LENSES)?;
    validate_review_field("review summary", &summary)?;

    let review_id = match parse_opt(args, "--review-id") {
        Some(raw) => {
            validate_review_id(&raw)?;
            raw
        }
        None => generate_review_id(&reviewer, &lens),
    };

    let ledger_path =
        resolve_review_output_path(&cwd, args, "--ledger-path", AGENT_TEAM_DEFAULT_LEDGER_FILE);
    let reviews_dir =
        resolve_review_output_path(&cwd, args, "--reviews-dir", REVIEW_DEFAULT_REVIEWS_DIR);
    let request_path = reviews_dir.join(format!("{review_id}.md"));

    validate_review_output_paths_are_distinct(&request_path, &ledger_path)?;
    validate_review_output_dir_path(&reviews_dir)?;
    validate_review_request_path(&request_path)?;
    validate_agent_team_durable_file_path(&ledger_path, "review ledger")?;

    let prompt = build_review_prepare_prompt(&review_id, &artifact, &reviewer, &lens, &summary);
    validate_terminal_text_arg("review prompt", &prompt)?;
    let request_body = build_review_request_md(ReviewRequestMd {
        review_id: &review_id,
        artifact: &artifact,
        reviewer: &reviewer,
        lens: &lens,
        summary: &summary,
        cwd: &cwd,
        ledger_path: &ledger_path,
        prompt: &prompt,
    });
    let ledger_entry = build_review_ledger_entry(
        &review_id,
        &artifact,
        &reviewer,
        &lens,
        &summary,
        &request_path,
    );

    let (request_status, ledger_status, ledger_seed_status) = if dry_run {
        ("planned", "planned", Value::Null)
    } else {
        create_review_request_file(&request_path, &request_body)?;
        let protocol_path = cwd.join(AGENT_TEAM_DEFAULT_PROTOCOL_FILE);
        let roster_path = cwd.join(AGENT_TEAM_DEFAULT_ROSTER_FILE);
        let ledger_body = build_agent_team_ledger_md(&cwd_string, &protocol_path, &roster_path);
        let seed_status = create_agent_team_ledger_file_if_missing(&ledger_path, &ledger_body)?;
        append_review_ledger_entry(&ledger_path, &ledger_entry)?;
        (
            "created",
            "appended",
            Value::String(seed_status.to_string()),
        )
    };

    Ok(json!({
        "ok": true,
        "review_command": "prepare",
        "dry_run": dry_run,
        "cwd": cwd_string,
        "review_id": review_id,
        "artifact": artifact,
        "reviewer": reviewer,
        "lens": lens,
        "summary": summary,
        "request_path": request_path.to_string_lossy().to_string(),
        "ledger_path": ledger_path.to_string_lossy().to_string(),
        "reviews_dir": reviews_dir.to_string_lossy().to_string(),
        "request": {
            "status": request_status,
            "path": request_path.to_string_lossy().to_string(),
        },
        "ledger": {
            "status": ledger_status,
            "path": ledger_path.to_string_lossy().to_string(),
            "seed_status": ledger_seed_status,
        },
        "prompt": prompt,
        "request_markdown": request_body,
        "ledger_entry": ledger_entry,
    }))
}

async fn run_review_spawn(client: &mut Client, raw_args: &[String]) -> Result<Value> {
    let args = if raw_args.first().map(String::as_str) == Some("spawn") {
        &raw_args[1..]
    } else {
        raw_args
    };

    let cwd_raw = parse_opt(args, "--cwd")
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let cwd = cwd_raw.canonicalize().with_context(|| {
        format!(
            "review spawn: could not resolve --cwd {}",
            cwd_raw.display()
        )
    })?;
    let cwd_string = cwd.to_string_lossy().to_string();

    let review_id = required_review_spawn_opt(args, "--review-id")?;
    validate_review_id(&review_id)?;
    let dry_run = parse_flag(args, "--dry-run");
    let no_launch = parse_flag(args, "--no-launch");
    let launch_mode = parse_agent_launch_mode(args, "review spawn")?;
    let direction = parse_opt(args, "--direction")
        .unwrap_or_else(|| "right".to_string())
        .to_ascii_lowercase();
    validate_review_spawn_direction(&direction)?;

    let reviews_dir =
        resolve_review_output_path(&cwd, args, "--reviews-dir", REVIEW_DEFAULT_REVIEWS_DIR);
    let request_path = reviews_dir.join(format!("{review_id}.md"));
    validate_review_output_dir_path(&reviews_dir)?;
    validate_review_existing_request_path(&request_path)?;
    let request = read_prepared_review_request(&request_path)?;
    if request.review_id != review_id {
        bail!(
            "review spawn: request id mismatch; expected {review_id}, found {}",
            request.review_id
        );
    }
    if request.reviewer == "manual" {
        bail!(
            "review spawn cannot launch manual reviews; use review prepare for manual review files"
        );
    }
    let (_, launch_command) = agent_launch_command_for_mode(&request.reviewer, launch_mode)
        .ok_or_else(|| {
            anyhow!(
                "review spawn: reviewer {} is not launchable",
                request.reviewer
            )
        })?;

    let ledger_path = parse_opt(args, "--ledger-path")
        .map(PathBuf::from)
        .map(|path| {
            if path.is_absolute() {
                path
            } else {
                cwd.join(path)
            }
        })
        .unwrap_or_else(|| request.ledger_path.clone());
    let evidence_path = parse_opt(args, "--evidence-path")
        .map(PathBuf::from)
        .map(|path| {
            if path.is_absolute() {
                path
            } else {
                cwd.join(path)
            }
        })
        .unwrap_or_else(|| reviews_dir.join(format!("{review_id}.evidence.md")));
    validate_review_spawn_output_paths_are_distinct(&request_path, &ledger_path, &evidence_path)?;
    validate_agent_team_durable_file_path(&ledger_path, "review ledger")?;
    validate_review_evidence_path(&evidence_path)?;
    ensure_review_ledger_pending_entry(&ledger_path, &review_id)?;
    validate_terminal_text_arg("review prompt", &request.prompt)?;

    if dry_run {
        let planned_surface = parse_opt(args, "--surface")
            .or_else(|| {
                env::var("LIMUX_SURFACE_ID")
                    .ok()
                    .filter(|value| !value.is_empty())
            })
            .unwrap_or_else(|| "<active-surface>".to_string());
        let planned_workspace = parse_opt(args, "--workspace")
            .or_else(|| {
                env::var("LIMUX_WORKSPACE_ID")
                    .ok()
                    .filter(|value| !value.is_empty())
            })
            .unwrap_or_else(|| "<active-workspace>".to_string());
        let capture_command = build_review_capture_command("<planned-reviewer-surface>", 120);
        return Ok(json!({
            "ok": true,
            "review_command": "spawn",
            "dry_run": true,
            "no_launch": no_launch,
            "launch_mode": launch_mode.as_str(),
            "cwd": cwd_string,
            "review_id": request.review_id,
            "artifact": request.artifact,
            "reviewer": request.reviewer,
            "lens": request.lens,
            "summary": request.summary,
            "request_path": request_path.to_string_lossy().to_string(),
            "ledger_path": ledger_path.to_string_lossy().to_string(),
            "reviews_dir": reviews_dir.to_string_lossy().to_string(),
            "evidence_path": evidence_path.to_string_lossy().to_string(),
            "workspace_id": planned_workspace,
            "source_surface_id": planned_surface,
            "reviewer_pane_id": Value::Null,
            "reviewer_surface_id": "<planned-reviewer-surface>",
            "direction": direction,
            "launch_command": launch_command,
            "capture_command": capture_command,
            "spawn": { "status": "planned" },
            "prompt": { "status": "planned", "text": request.prompt },
            "evidence": { "status": "planned" },
            "ledger": { "status": "planned" },
        }));
    }

    let (workspace_id, source_surface_id) = resolve_review_spawn_source(client, args).await?;
    let (reviewer_pane_id, reviewer_surface_id) = create_review_spawn_pane(
        client,
        &workspace_id,
        &source_surface_id,
        &direction,
        if no_launch {
            None
        } else {
            Some(launch_command.as_str())
        },
    )
    .await?;

    let prompt_status = if no_launch {
        "skipped"
    } else {
        tokio::time::sleep(AGENT_TEAM_BOOTSTRAP_LAUNCH_SETTLE).await;
        send_review_prompt(
            client,
            &workspace_id,
            &reviewer_surface_id,
            &request.reviewer,
            &request.prompt,
        )
        .await?;
        "sent"
    };
    let capture_command = build_review_capture_command(&reviewer_surface_id, 120);
    let evidence_body = build_review_evidence_pointer_md(ReviewSpawnEvidenceMd {
        request: &request,
        request_path: &request_path,
        ledger_path: &ledger_path,
        reviewer_pane_id: &reviewer_pane_id,
        reviewer_surface_id: &reviewer_surface_id,
        prompt_status,
        capture_command: &capture_command,
    });
    create_review_evidence_file(&evidence_path, &evidence_body)?;
    let ledger_spawn_block = build_review_spawn_ledger_update(ReviewSpawnLedgerUpdate {
        evidence_path: &evidence_path,
        reviewer_pane_id: &reviewer_pane_id,
        reviewer_surface_id: &reviewer_surface_id,
        prompt_status,
        capture_command: &capture_command,
    });
    update_review_ledger_entry_for_spawn(&ledger_path, &review_id, &ledger_spawn_block)?;

    Ok(json!({
        "ok": true,
        "review_command": "spawn",
        "dry_run": false,
        "no_launch": no_launch,
        "launch_mode": launch_mode.as_str(),
        "cwd": cwd_string,
        "review_id": request.review_id,
        "artifact": request.artifact,
        "reviewer": request.reviewer,
        "lens": request.lens,
        "summary": request.summary,
        "request_path": request_path.to_string_lossy().to_string(),
        "ledger_path": ledger_path.to_string_lossy().to_string(),
        "reviews_dir": reviews_dir.to_string_lossy().to_string(),
        "evidence_path": evidence_path.to_string_lossy().to_string(),
        "workspace_id": workspace_id,
        "source_surface_id": source_surface_id,
        "reviewer_pane_id": reviewer_pane_id,
        "reviewer_surface_id": reviewer_surface_id,
        "direction": direction,
        "launch_command": launch_command,
        "capture_command": capture_command,
        "spawn": { "status": "created" },
        "prompt": { "status": prompt_status, "text": request.prompt },
        "evidence": { "status": "created" },
        "ledger": { "status": "updated" },
    }))
}

fn required_review_spawn_opt(args: &[String], flag: &str) -> Result<String> {
    parse_opt(args, flag)
        .filter(|value| !value.trim().is_empty() && !value.starts_with("--"))
        .ok_or_else(|| anyhow!("review spawn requires {flag}"))
}

fn validate_review_spawn_direction(direction: &str) -> Result<()> {
    match direction {
        "left" | "right" | "up" | "down" => Ok(()),
        _ => bail!("review spawn --direction must be one of left|right|up|down"),
    }
}

fn required_review_opt(args: &[String], flag: &str) -> Result<String> {
    let value = parse_opt(args, flag)
        .filter(|value| !value.trim().is_empty() && !value.starts_with("--"))
        .ok_or_else(|| anyhow!("review prepare requires {flag}"))?;
    Ok(value)
}

fn validate_review_choice(label: &str, value: &str, allowed: &[&str]) -> Result<()> {
    validate_review_field(label, value)?;
    if !allowed.iter().any(|candidate| candidate == &value) {
        bail!("{label} must be one of {}", allowed.join("|"));
    }
    Ok(())
}

fn validate_review_field(label: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        bail!("{label} cannot be empty");
    }
    for (index, ch) in value.char_indices() {
        if ch.is_control() {
            bail!(
                "{label} contains unsupported control character U+{:04X} at byte {}",
                ch as u32,
                index
            );
        }
    }
    Ok(())
}

fn validate_review_id(review_id: &str) -> Result<()> {
    validate_review_field("review id", review_id)?;
    if review_id.starts_with('.') {
        bail!("review id cannot start with '.'");
    }
    if !review_id
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
    {
        bail!("review id may contain only ASCII letters, numbers, '-', '_', and '.'");
    }
    Ok(())
}

fn generate_review_id(reviewer: &str, lens: &str) -> String {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    format!("review-{nonce}-{}-{reviewer}-{lens}", std::process::id())
}

fn resolve_review_output_path(
    cwd: &Path,
    args: &[String],
    flag: &str,
    default_file: &str,
) -> PathBuf {
    if let Some(raw_path) = parse_opt(args, flag) {
        let path = PathBuf::from(raw_path);
        if path.is_absolute() {
            path
        } else {
            cwd.join(path)
        }
    } else {
        cwd.join(default_file)
    }
}

fn validate_review_output_paths_are_distinct(
    request_path: &Path,
    ledger_path: &Path,
) -> Result<()> {
    if comparable_agent_team_path(request_path) == comparable_agent_team_path(ledger_path) {
        bail!(
            "review output paths must be distinct; request and review ledger both resolve to {}",
            request_path.display()
        );
    }
    Ok(())
}

fn validate_review_output_dir_path(path: &Path) -> Result<()> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(()),
        Err(err) => {
            return Err(err).with_context(|| format!("failed to inspect {}", path.display()));
        }
    };
    let file_type = metadata.file_type();
    if file_type.is_symlink() {
        bail!(
            "refusing to use reviews directory because it is a symlink: {}",
            path.display()
        );
    }
    if !file_type.is_dir() {
        bail!(
            "refusing to use reviews directory because it is not a directory: {}",
            path.display()
        );
    }
    Ok(())
}

fn validate_review_request_path(path: &Path) -> Result<()> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(()),
        Err(err) => {
            return Err(err).with_context(|| format!("failed to inspect {}", path.display()));
        }
    };
    let file_type = metadata.file_type();
    if file_type.is_symlink() {
        bail!(
            "refusing to write review request because it is a symlink: {}",
            path.display()
        );
    }
    if !file_type.is_file() {
        bail!(
            "refusing to write review request because it is not a regular file: {}",
            path.display()
        );
    }
    bail!("review request already exists: {}", path.display())
}

fn create_review_request_file(path: &Path, body: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
    }
    validate_review_request_path(path)?;
    match OpenOptions::new().write(true).create_new(true).open(path) {
        Ok(mut file) => {
            if let Err(err) = file.write_all(body.as_bytes()) {
                let _ = fs::remove_file(path);
                return Err(err)
                    .with_context(|| format!("failed to write review request {}", path.display()));
            }
            Ok(())
        }
        Err(err) if err.kind() == ErrorKind::AlreadyExists => {
            validate_review_request_path(path)?;
            bail!("review request already exists: {}", path.display())
        }
        Err(err) => {
            Err(err).with_context(|| format!("failed to create review request {}", path.display()))
        }
    }
}

fn validate_review_existing_request_path(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("failed to inspect review request {}", path.display()))?;
    let file_type = metadata.file_type();
    if file_type.is_symlink() {
        bail!(
            "refusing to read review request because it is a symlink: {}",
            path.display()
        );
    }
    if !file_type.is_file() {
        bail!(
            "refusing to read review request because it is not a regular file: {}",
            path.display()
        );
    }
    Ok(())
}

fn read_prepared_review_request(path: &Path) -> Result<PreparedReviewRequest> {
    validate_review_existing_request_path(path)?;
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read review request {}", path.display()))?;
    if !raw.contains(REVIEW_REQUEST_MARKER) {
        bail!(
            "review spawn: refusing unmarked review request {}; run `review prepare` first",
            path.display()
        );
    }
    let review_id = review_markdown_backtick_field(&raw, "Review ID")
        .ok_or_else(|| anyhow!("review spawn: request missing Review ID"))?;
    let reviewer = review_markdown_backtick_field(&raw, "Reviewer")
        .ok_or_else(|| anyhow!("review spawn: request missing Reviewer"))?;
    let lens = review_markdown_backtick_field(&raw, "Lens")
        .ok_or_else(|| anyhow!("review spawn: request missing Lens"))?;
    let artifact = review_markdown_backtick_field(&raw, "Artifact")
        .ok_or_else(|| anyhow!("review spawn: request missing Artifact"))?;
    let summary = review_markdown_backtick_field(&raw, "Summary")
        .ok_or_else(|| anyhow!("review spawn: request missing Summary"))?;
    let ledger_path = review_markdown_backtick_field(&raw, "Review ledger")
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("review spawn: request missing Review ledger"))?;
    let prompt = review_markdown_prompt_block(&raw)
        .ok_or_else(|| anyhow!("review spawn: request missing prompt block"))?;

    validate_review_id(&review_id)?;
    validate_review_choice("reviewer", &reviewer, REVIEW_REVIEWERS)?;
    validate_review_choice("review lens", &lens, REVIEW_LENSES)?;
    validate_review_field("review artifact", &artifact)?;
    validate_review_field("review summary", &summary)?;
    validate_terminal_text_arg("review prompt", &prompt)?;

    Ok(PreparedReviewRequest {
        review_id,
        artifact,
        reviewer,
        lens,
        summary,
        ledger_path,
        prompt,
    })
}

fn review_markdown_backtick_field(markdown: &str, label: &str) -> Option<String> {
    let prefix = format!("{label}: `");
    markdown.lines().find_map(|line| {
        let rest = line.strip_prefix(&prefix)?;
        let end = rest.find('`')?;
        Some(rest[..end].to_string())
    })
}

fn review_markdown_prompt_block(markdown: &str) -> Option<String> {
    let start = markdown.find("```text\n")? + "```text\n".len();
    let rest = &markdown[start..];
    let end = rest.find("\n```")?;
    Some(rest[..end].to_string())
}

fn validate_review_spawn_output_paths_are_distinct(
    request_path: &Path,
    ledger_path: &Path,
    evidence_path: &Path,
) -> Result<()> {
    let paths = [
        (
            "request",
            request_path,
            comparable_agent_team_path(request_path),
        ),
        (
            "review ledger",
            ledger_path,
            comparable_agent_team_path(ledger_path),
        ),
        (
            "review evidence",
            evidence_path,
            comparable_agent_team_path(evidence_path),
        ),
    ];
    for (left_index, (left_label, left_path, left_comparable)) in paths.iter().enumerate() {
        for (right_label, _right_path, right_comparable) in paths.iter().skip(left_index + 1) {
            if left_comparable == right_comparable {
                bail!(
                    "review spawn output paths must be distinct; {left_label} and {right_label} both resolve to {}",
                    left_path.display()
                );
            }
        }
    }
    Ok(())
}

fn validate_review_evidence_path(path: &Path) -> Result<()> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(()),
        Err(err) => {
            return Err(err).with_context(|| format!("failed to inspect {}", path.display()));
        }
    };
    let file_type = metadata.file_type();
    if file_type.is_symlink() {
        bail!(
            "refusing to write review evidence because it is a symlink: {}",
            path.display()
        );
    }
    if !file_type.is_file() {
        bail!(
            "refusing to write review evidence because it is not a regular file: {}",
            path.display()
        );
    }
    bail!("review evidence already exists: {}", path.display())
}

fn create_review_evidence_file(path: &Path, body: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
    }
    validate_review_evidence_path(path)?;
    match OpenOptions::new().write(true).create_new(true).open(path) {
        Ok(mut file) => file
            .write_all(body.as_bytes())
            .with_context(|| format!("failed to write review evidence {}", path.display())),
        Err(err) if err.kind() == ErrorKind::AlreadyExists => {
            validate_review_evidence_path(path)?;
            bail!("review evidence already exists: {}", path.display())
        }
        Err(err) => {
            Err(err).with_context(|| format!("failed to create review evidence {}", path.display()))
        }
    }
}

fn append_review_ledger_entry(path: &Path, entry: &str) -> Result<()> {
    validate_agent_team_durable_file_path(path, "review ledger")?;
    let needs_leading_newline = match fs::read(path) {
        Ok(bytes) => !bytes.is_empty() && bytes.last() != Some(&b'\n'),
        Err(err) if err.kind() == ErrorKind::NotFound => false,
        Err(err) => {
            return Err(err)
                .with_context(|| format!("failed to inspect review ledger {}", path.display()));
        }
    };

    let mut file = OpenOptions::new()
        .append(true)
        .open(path)
        .with_context(|| format!("failed to open review ledger {}", path.display()))?;
    let mut body = String::new();
    if needs_leading_newline {
        body.push('\n');
    }
    body.push_str(entry);
    if !body.ends_with('\n') {
        body.push('\n');
    }
    file.write_all(body.as_bytes())
        .with_context(|| format!("failed to append review ledger {}", path.display()))?;
    Ok(())
}

async fn resolve_review_spawn_source(
    client: &mut Client,
    args: &[String],
) -> Result<(String, String)> {
    let workspace_id = if let Some(workspace_id) = parse_opt(args, "--workspace").or_else(|| {
        env::var("LIMUX_WORKSPACE_ID")
            .ok()
            .filter(|value| !value.is_empty())
    }) {
        workspace_id
    } else {
        let payload = client
            .call("workspace.current", json!({}))
            .await
            .context("review spawn: workspace.current failed")?;
        get_string(&payload, &["workspace_id", "id"])
            .ok_or_else(|| anyhow!("review spawn: workspace.current returned no workspace_id"))?
    };

    if let Some(surface_id) = parse_opt(args, "--surface").or_else(|| {
        env::var("LIMUX_SURFACE_ID")
            .ok()
            .filter(|value| !value.is_empty())
    }) {
        return Ok((workspace_id, surface_id));
    }

    let surfaces = client
        .call(
            "surface.list",
            json!({ "workspace_id": workspace_id.clone() }),
        )
        .await
        .context("review spawn: surface.list failed for target workspace")?;
    let surface_rows = surfaces
        .get("surfaces")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if surface_rows.is_empty() {
        bail!("review spawn: target workspace has no surfaces");
    }
    let source_surface_id = surface_rows
        .iter()
        .find(|row| row.get("focused").and_then(Value::as_bool) == Some(true))
        .and_then(|row| get_string(row, &["surface_id"]))
        .or_else(|| get_string(&surface_rows[0], &["surface_id"]))
        .ok_or_else(|| anyhow!("review spawn: could not determine source surface"))?;

    Ok((workspace_id, source_surface_id))
}

async fn create_review_spawn_pane(
    client: &mut Client,
    workspace_id: &str,
    source_surface_id: &str,
    direction: &str,
    command: Option<&str>,
) -> Result<(String, String)> {
    let mut params = Map::new();
    params.insert(
        "workspace_id".to_string(),
        Value::String(workspace_id.to_string()),
    );
    params.insert(
        "surface_id".to_string(),
        Value::String(source_surface_id.to_string()),
    );
    params.insert(
        "direction".to_string(),
        Value::String(direction.to_string()),
    );
    params.insert("type".to_string(), Value::String("terminal".to_string()));
    if let Some(command) = command {
        validate_terminal_text_arg("review spawn command", command)?;
        params.insert("command".to_string(), Value::String(command.to_string()));
    }

    let created = client
        .call("pane.create", Value::Object(params))
        .await
        .context("review spawn: pane.create failed")?;
    let pane_id = get_string(&created, &["pane_id", "pane_ref"])
        .ok_or_else(|| anyhow!("review spawn: pane.create returned no pane_id"))?;
    let surface_id = get_string(&created, &["surface_id", "surface_ref"])
        .ok_or_else(|| anyhow!("review spawn: pane.create returned no surface_id"))?;
    Ok((pane_id, surface_id))
}

async fn send_review_prompt(
    client: &mut Client,
    workspace_id: &str,
    surface_id: &str,
    reviewer: &str,
    prompt: &str,
) -> Result<()> {
    validate_terminal_text_arg("review prompt", prompt)?;
    let mut text_params = Map::new();
    text_params.insert(
        "workspace_id".to_string(),
        Value::String(workspace_id.to_string()),
    );
    text_params.insert(
        "surface_id".to_string(),
        Value::String(surface_id.to_string()),
    );
    text_params.insert("text".to_string(), Value::String(prompt.to_string()));

    let mut key_params = Map::new();
    key_params.insert(
        "workspace_id".to_string(),
        Value::String(workspace_id.to_string()),
    );
    key_params.insert(
        "surface_id".to_string(),
        Value::String(surface_id.to_string()),
    );
    key_params.insert("key".to_string(), Value::String("enter".to_string()));

    let mut last_error = None;
    for attempt in 0..AGENT_TEAM_BOOTSTRAP_RETRY_ATTEMPTS {
        match client
            .call("surface.send_text", Value::Object(text_params.clone()))
            .await
        {
            Ok(_) => {
                tokio::time::sleep(Duration::from_millis(100)).await;
                client
                    .call("surface.send_key", Value::Object(key_params.clone()))
                    .await
                    .with_context(|| {
                        format!(
                            "review spawn: prompt submit failed for '{reviewer}' on surface {surface_id}"
                        )
                    })?;
                return Ok(());
            }
            Err(error) => {
                let message = format!("{error:#}");
                if message.contains("not ready for text input")
                    && attempt + 1 < AGENT_TEAM_BOOTSTRAP_RETRY_ATTEMPTS
                {
                    last_error = Some(message);
                    tokio::time::sleep(AGENT_TEAM_BOOTSTRAP_RETRY_INTERVAL).await;
                    continue;
                }
                return Err(error).with_context(|| {
                    format!(
                        "review spawn: prompt send failed for '{reviewer}' on surface {surface_id}"
                    )
                });
            }
        }
    }

    bail!(
        "review spawn: prompt send failed for '{reviewer}' on surface {surface_id}: {}",
        last_error.unwrap_or_else(|| "surface did not become ready".to_string())
    )
}

fn build_review_capture_command(surface_id: &str, lines: u64) -> String {
    format!("limux read-screen --surface {surface_id} --scrollback --lines {lines}")
}

fn build_review_prepare_prompt(
    review_id: &str,
    artifact: &str,
    reviewer: &str,
    lens: &str,
    summary: &str,
) -> String {
    format!(
        "Review request: {review_id}\n\
         Reviewer: {reviewer}\n\
         Lens: {lens}\n\
         Artifact: {artifact}\n\
         Goal: {summary}\n\n\
         Please review the artifact using the named lens. Return a verdict of GO, WAIT, or NO-GO; list concrete findings with file/line evidence when applicable; call out residual risks; and keep raw terminal transcripts out of the response."
    )
}

fn build_review_request_md(request: ReviewRequestMd<'_>) -> String {
    let mut out = String::new();
    out.push_str(REVIEW_REQUEST_MARKER);
    out.push('\n');
    out.push_str("# Limux Review Request\n\n");
    out.push_str(&format!(
        "Review ID: `{}`\n",
        markdown_table_cell(request.review_id)
    ));
    out.push_str("Status: `pending`\n");
    out.push_str(&format!(
        "Reviewer: `{}`\n",
        markdown_table_cell(request.reviewer)
    ));
    out.push_str(&format!("Lens: `{}`\n", markdown_table_cell(request.lens)));
    out.push_str(&format!(
        "Artifact: `{}`\n",
        markdown_table_cell(request.artifact)
    ));
    out.push_str(&format!(
        "Summary: `{}`\n",
        markdown_table_cell(request.summary)
    ));
    out.push_str(&format!("Shared cwd: `{}`\n", markdown_path(request.cwd)));
    out.push_str(&format!(
        "Review ledger: `{}`\n\n",
        markdown_path(request.ledger_path)
    ));
    out.push_str("## Instructions\n\n");
    out.push_str(
        "- Inspect the artifact and relevant surrounding context before giving a verdict.\n",
    );
    out.push_str("- Return `GO`, `WAIT`, or `NO-GO` with concrete rationale.\n");
    out.push_str("- Include file and line evidence for code findings when possible.\n");
    out.push_str(
        "- Do not paste raw terminal transcripts, secrets, credentials, or unrelated logs.\n\n",
    );
    out.push_str("## Prompt\n\n");
    out.push_str("```text\n");
    out.push_str(request.prompt);
    out.push_str("\n```\n");
    out
}

fn build_review_ledger_entry(
    review_id: &str,
    artifact: &str,
    reviewer: &str,
    lens: &str,
    summary: &str,
    request_path: &Path,
) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "## pending - {}\n\n",
        markdown_table_cell(review_id)
    ));
    out.push_str("Status: pending\n");
    out.push_str("Project: `current`\n");
    out.push_str("Thread: `limux-agent-team`\n");
    out.push_str(&format!("Artifact: `{}`\n", markdown_table_cell(artifact)));
    out.push_str(&format!("Reviewer: `{}`\n", markdown_table_cell(reviewer)));
    out.push_str(&format!("Lens: `{}`\n", markdown_table_cell(lens)));
    out.push_str(&format!("Request: `{}`\n", markdown_path(request_path)));
    out.push_str(&format!("Summary: `{}`\n\n", markdown_table_cell(summary)));
    out.push_str("### Pending Review\n\n");
    out.push_str("| Reviewer | Lens | Verdict | Evidence |\n");
    out.push_str("|---|---|---|---|\n");
    out.push_str(&format!(
        "| `{}` | `{}` | pending | `{}` |\n\n",
        markdown_table_cell(reviewer),
        markdown_table_cell(lens),
        markdown_path(request_path)
    ));
    out
}

fn build_review_evidence_pointer_md(evidence: ReviewSpawnEvidenceMd<'_>) -> String {
    let mut out = String::new();
    out.push_str(REVIEW_EVIDENCE_MARKER);
    out.push('\n');
    out.push_str("# Limux Review Evidence Pointer\n\n");
    out.push_str(&format!(
        "Review ID: `{}`\n",
        markdown_table_cell(&evidence.request.review_id)
    ));
    out.push_str("Status: `in-progress`\n");
    out.push_str(&format!(
        "Reviewer: `{}`\n",
        markdown_table_cell(&evidence.request.reviewer)
    ));
    out.push_str(&format!(
        "Lens: `{}`\n",
        markdown_table_cell(&evidence.request.lens)
    ));
    out.push_str(&format!(
        "Artifact: `{}`\n",
        markdown_table_cell(&evidence.request.artifact)
    ));
    out.push_str(&format!(
        "Request: `{}`\n",
        markdown_path(evidence.request_path)
    ));
    out.push_str(&format!(
        "Review ledger: `{}`\n",
        markdown_path(evidence.ledger_path)
    ));
    out.push_str(&format!(
        "Reviewer pane: `{}`\n",
        markdown_table_cell(evidence.reviewer_pane_id)
    ));
    out.push_str(&format!(
        "Reviewer surface: `{}`\n",
        markdown_table_cell(evidence.reviewer_surface_id)
    ));
    out.push_str(&format!(
        "Prompt status: `{}`\n",
        markdown_table_cell(evidence.prompt_status)
    ));
    out.push_str(&format!(
        "Capture command: `{}`\n\n",
        markdown_table_cell(evidence.capture_command)
    ));
    out.push_str(
        "This file points to the live reviewer pane and the request file. Do not\n\
         paste raw terminal transcripts here unless the capture has been\n\
         reviewed for secrets and unrelated logs.\n",
    );
    out
}

fn build_review_spawn_ledger_update(update: ReviewSpawnLedgerUpdate<'_>) -> String {
    let mut out = String::new();
    out.push_str("### Spawn\n\n");
    out.push_str("Spawn status: in-progress\n");
    out.push_str(&format!(
        "Reviewer pane: `{}`\n",
        markdown_table_cell(update.reviewer_pane_id)
    ));
    out.push_str(&format!(
        "Reviewer surface: `{}`\n",
        markdown_table_cell(update.reviewer_surface_id)
    ));
    out.push_str(&format!(
        "Prompt status: `{}`\n",
        markdown_table_cell(update.prompt_status)
    ));
    out.push_str(&format!(
        "Evidence pointer: `{}`\n",
        markdown_path(update.evidence_path)
    ));
    out.push_str(&format!(
        "Capture command: `{}`\n\n",
        markdown_table_cell(update.capture_command)
    ));
    out
}

fn ensure_review_ledger_pending_entry(path: &Path, review_id: &str) -> Result<()> {
    validate_agent_team_durable_file_path(path, "review ledger")?;
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read review ledger {}", path.display()))?;
    let pending_header = format!("## pending - {}", markdown_table_cell(review_id));
    if raw.contains(&pending_header) {
        return Ok(());
    }
    let in_progress_header = format!("## in-progress - {}", markdown_table_cell(review_id));
    if raw.contains(&in_progress_header) {
        bail!("review ledger entry already in-progress for {review_id}");
    }
    bail!("review ledger has no pending entry for {review_id}")
}

fn update_review_ledger_entry_for_spawn(
    path: &Path,
    review_id: &str,
    spawn_block: &str,
) -> Result<()> {
    validate_agent_team_durable_file_path(path, "review ledger")?;
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read review ledger {}", path.display()))?;
    let pending_header = format!("## pending - {}", markdown_table_cell(review_id));
    let in_progress_header = format!("## in-progress - {}", markdown_table_cell(review_id));
    if raw.contains(&in_progress_header) {
        bail!("review ledger entry already in-progress for {review_id}");
    }
    let header_start = raw
        .find(&pending_header)
        .ok_or_else(|| anyhow!("review ledger has no pending entry for {review_id}"))?;
    let search_from = header_start + 1;
    let block_end = raw[search_from..]
        .find("\n## ")
        .map(|index| search_from + index)
        .unwrap_or(raw.len());
    let mut block = raw[header_start..block_end].to_string();
    if block.contains("\n### Spawn\n") {
        bail!("review ledger entry already has spawn metadata for {review_id}");
    }
    block = block.replacen(&pending_header, &in_progress_header, 1);
    if !block.contains("Status: pending") {
        bail!("review ledger entry for {review_id} has no pending status");
    }
    block = block.replacen("Status: pending", "Status: in-progress", 1);
    if !block.ends_with('\n') {
        block.push('\n');
    }
    block.push('\n');
    block.push_str(spawn_block);
    if !block.ends_with('\n') {
        block.push('\n');
    }

    let mut updated = String::with_capacity(raw.len() + spawn_block.len() + 64);
    updated.push_str(&raw[..header_start]);
    updated.push_str(&block);
    updated.push_str(&raw[block_end..]);
    replace_review_ledger_file(path, &updated)
}

fn replace_review_ledger_file(path: &Path, body: &str) -> Result<()> {
    validate_agent_team_durable_file_path(path, "review ledger")?;
    let temp_path = temporary_agent_team_output_path(path)?;
    fs::write(&temp_path, body)
        .with_context(|| format!("failed to write {}", temp_path.display()))?;
    validate_agent_team_durable_file_path(path, "review ledger")?;
    fs::rename(&temp_path, path).with_context(|| {
        format!(
            "failed to move updated review ledger from {} to {}",
            temp_path.display(),
            path.display()
        )
    })
}

fn build_agents_md(
    peers: &[(String, String, String, String)],
    cwd: &str,
    workspace_name: &str,
    workspace_id: &str,
    orchestrator_surface: &str,
    coordination_files: AgentTeamCoordinationFiles<'_>,
    instruction_sources: &[InstructionSource],
) -> String {
    let mut out = String::new();
    out.push_str(AGENT_TEAM_PROTOCOL_MARKER);
    out.push('\n');
    out.push_str("# LIMUX_AGENTS.md — agent-to-agent message protocol\n\n");
    out.push_str(
        "This file is auto-generated by `limux agent-team`. It defines how the\n\
         agents running in this workspace team communicate with each other via\n\
         the limux control socket. Humans should feel free to edit the\n\
         'Policies' section below; everything else is mechanical.\n\n",
    );

    out.push_str("## Instruction Sources\n\n");
    out.push_str(
        "Project instruction files remain authoritative. Limux does not copy,\n\
         merge, or reinterpret their contents; each agent should read these\n\
         files directly before acting.\n\n",
    );
    if instruction_sources.is_empty() {
        out.push_str(
            "No `AGENTS.md`, `CLAUDE.md`, or `GEMINI.md` files were detected in\n\
             the shared cwd at generation time.\n\n",
        );
    } else {
        out.push_str("| Source | Path | Type | Modified (unix seconds) | Hash | Notes |\n");
        out.push_str("|--------|------|------|-------------------------|------|-------|\n");
        for source in instruction_sources {
            let modified = source
                .modified_unix
                .map(|value| value.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            let hash = source.hash.as_deref().unwrap_or("not hashed");
            let note = source.note.as_deref().unwrap_or("");
            out.push_str(&format!(
                "| `{}` | `{}` | `{}` | `{}` | `{}` | {} |\n",
                markdown_table_cell(source.name),
                markdown_table_cell(&source.display_path),
                markdown_table_cell(&source.kind),
                modified,
                markdown_table_cell(hash),
                markdown_table_cell(note)
            ));
        }
        out.push('\n');
    }

    out.push_str("## Local Policy Extension\n\n");
    out.push_str(&format!(
        "Use `{}` for durable team policy notes that should survive\n\
         regeneration. Limux does not create or overwrite it; if present,\n\
         agents should read it after this generated runtime protocol.\n\n",
        AGENT_TEAM_LOCAL_POLICY_FILE
    ));

    out.push_str("## Durable Coordination Files\n\n");
    out.push_str(
        "Use these files for project/team state that should survive protocol\n\
         regeneration. Limux seeds the roster and ledger only when missing;\n\
         it does not overwrite existing durable entries by default.\n\n",
    );
    out.push_str(&format!(
        "- Team roster: `{}`\n",
        markdown_path(coordination_files.roster_path)
    ));
    out.push_str(&format!(
        "- Review and consensus ledger: `{}`\n\n",
        markdown_path(coordination_files.ledger_path)
    ));

    out.push_str(&format!(
        "## Team workspace\n\n\
         The orchestrator (the pane that ran `limux agent-team`) and all\n\
         spawned peers share one workspace:\n\n\
         - Workspace name: `{workspace_name}`\n\
         - Workspace ID: `{workspace_id}`\n\
         - Orchestrator surface: `{orchestrator_surface}`\n\
         - Shared cwd: `{cwd}`\n\n",
    ));

    out.push_str("## Peers in this team\n\n");
    out.push_str("| Agent | Pane | Surface | Launch command |\n");
    out.push_str("|-------|------|---------|----------------|\n");
    for (name, pane_id, surface_id, launch) in peers {
        out.push_str(&format!(
            "| `{name}` | `{pane_id}` | `{surface_id}` | `{launch}` |\n"
        ));
    }
    out.push('\n');
    out.push_str(
        "The orchestrator is not in the table — message it back using its\n\
         `Orchestrator surface` from the block above. This table is the current\n\
         runtime snapshot; durable ownership and cross-project routing belong\n\
         in the team roster.\n\n",
    );

    out.push_str("## How to send a message\n\n");
    out.push_str(
        "Messages use the `<agent-msg>` XML envelope so they're easy to\n\
         extract from the terminal scrollback. To send a message to a peer,\n\
         look up their `Surface` in the peers table above and run (from any\n\
         shell, including the agent's own terminal — `limux` is on PATH):\n\n",
    );
    out.push_str("```bash\n");
    out.push_str("limux send --surface <peer-surface-id> $'<agent-msg from=\"<me>\" to=\"<peer>\" id=\"<uuid>\" ts=\"<iso8601>\">\\n<body/>\\n</agent-msg>\\n'\n");
    out.push_str("```\n\n");
    out.push_str(
        "The message appears at the peer's prompt as plain stdin, so the\n\
         peer's agent CLI picks it up like a normal user message. Trailing\n\
         newline is required so the agent's read-line actually fires.\n\n",
    );

    out.push_str("### Envelope format\n\n");
    out.push_str("```xml\n");
    out.push_str("<agent-msg from=\"codex\" to=\"claude\" id=\"<uuid>\" ts=\"2026-04-19T16:48:00Z\" reply-to=\"<parent-uuid>\">\n");
    out.push_str(
        "  <context>optional: one or two sentences about what the request is for</context>\n",
    );
    out.push_str("  <request>the actual ask, in prose or code</request>\n");
    out.push_str("  <expect>how you want the peer to reply (\"inline code diff\" / \"short summary\" / etc.)</expect>\n");
    out.push_str("</agent-msg>\n");
    out.push_str("```\n\n");

    out.push_str("Rules:\n");
    out.push_str("- `from` / `to` MUST be one of the agent names in the peers table.\n");
    out.push_str("- `id` is a fresh UUID (e.g. `uuidgen`); peers echo it in `reply-to`.\n");
    out.push_str("- `ts` is ISO-8601 UTC (`date -u +%Y-%m-%dT%H:%M:%SZ`).\n");
    out.push_str("- Inner tags are guidance, not required — `<request>` alone is fine.\n");
    out.push_str("- Keep bodies short; link to files in the shared cwd for anything long.\n\n");

    out.push_str("### Replying\n\n");
    out.push_str("Reply with the envelope reversed and `reply-to` set to the original `id`:\n\n");
    out.push_str("```bash\n");
    out.push_str("limux send --surface <orig-sender-surface-id> $'<agent-msg from=\"claude\" to=\"codex\" id=\"<new-uuid>\" reply-to=\"<orig-uuid>\" ts=\"<iso8601>\">\\n<response>...</response>\\n</agent-msg>\\n'\n");
    out.push_str("```\n\n");

    out.push_str("## Pinging the human\n\n");
    out.push_str(
        "When you need human input, use `limux notify` — it pops a toast\n\
         and lights up the workspace in the sidebar. Example:\n\n",
    );
    out.push_str("```bash\n");
    out.push_str("limux notify --subtitle 'needs review' --body 'Claude blocked on auth choice' 'Input needed'\n");
    out.push_str("```\n\n");

    out.push_str("## Environment contract\n\n");
    out.push_str(
        "Every pane spawned by limux inherits:\n\
         - `LIMUX_WORKSPACE_ID` — the team workspace's UUID\n\
         - `LIMUX_SURFACE_ID` — this pane's surface id (this is your `from`)\n\
         - `LIMUX_PANE_ID`, `LIMUX_TAB_ID`\n\
         - `LIMUX_SOCKET` — the control socket path\n\n\
         This means `limux identify`, `limux send` (with `--surface`), and\n\
         `limux notify` all auto-target the right thing with no flags needed\n\
         from inside the agent's own terminal.\n\n",
    );

    out.push_str("## Splitting your own pane\n\n");
    out.push_str("If you need a scratch terminal next to you, split your own pane:\n\n");
    out.push_str("```bash\n");
    out.push_str(&new_pane_shell_command("right", "bash"));
    out.push('\n');
    out.push_str("```\n\n");
    out.push_str(
        "`new-pane` reads `LIMUX_WORKSPACE_ID`, `LIMUX_SURFACE_ID`, and\n\
         `LIMUX_PANE_ID`, so it splits your current pane even if GTK focus has\n\
         moved elsewhere. Live GTK self-spawn currently supports terminal\n\
         panes only; browser pane creation is deferred.\n\n",
    );

    out.push_str("## Policies (edit these freely)\n\n");
    out.push_str(
        "- If a peer is silent for more than 60 seconds, re-send with `reply-to` = your last id.\n",
    );
    out.push_str(
        "- Never send more than 200 lines at once; write to a file and send the path instead.\n",
    );
    out.push_str("- Record reviewer findings, consensus decisions, accepted risks, and cross-team notifications in the review ledger before relying on terminal scrollback.\n");
    out.push_str("- If two agents disagree on an approach, both message the human via `limux notify` and stop.\n");
    out.push_str("- Before taking destructive actions (rm, git push, kubectl apply), ask the human via `limux notify`.\n\n");

    out.push_str("---\n");
    out.push_str(
        "_Generated by `limux agent-team`. Safe to edit the Policies\n\
         section; regenerating will overwrite everything above it._\n",
    );

    out
}

async fn run_close_workspace(client: &mut Client, args: &[String]) -> Result<Value> {
    let workspace = parse_opt(args, "--workspace")
        .or_else(|| env::var("LIMUX_WORKSPACE_ID").ok())
        .ok_or_else(|| anyhow!("close-workspace requires --workspace <id|ref>"))?;
    client
        .call("workspace.close", json!({ "workspace_id": workspace }))
        .await
}

async fn run_sidebar_state(client: &mut Client, args: &[String]) -> Result<Value> {
    let workspace = parse_opt(args, "--workspace")
        .or_else(|| env::var("LIMUX_WORKSPACE_ID").ok())
        .ok_or_else(|| anyhow!("sidebar-state requires --workspace <id|ref>"))?;

    let listed = client.call("workspace.list", json!({})).await?;
    let rows = listed
        .get("workspaces")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let matched = rows.into_iter().find(|row| {
        let id = get_string(row, &["workspace_id", "id"]).unwrap_or_default();
        let rf = get_string(row, &["workspace_ref", "ref"]).unwrap_or_default();
        workspace == id || workspace == rf
    });

    let cwd = matched
        .as_ref()
        .and_then(|row| get_string(row, &["cwd"]))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "none".to_string());

    let git_branch = if cwd != "none" {
        let output = Command::new("git")
            .arg("-C")
            .arg(&cwd)
            .arg("rev-parse")
            .arg("--abbrev-ref")
            .arg("HEAD")
            .output();
        match output {
            Ok(out) if out.status.success() => {
                String::from_utf8_lossy(&out.stdout).trim().to_string()
            }
            _ => "none".to_string(),
        }
    } else {
        "none".to_string()
    };

    Ok(json!({
        "workspace": workspace,
        "cwd": cwd,
        "git_branch": git_branch,
    }))
}

async fn run_new_surface(client: &mut Client, args: &[String]) -> Result<Value> {
    let workspace = parse_opt(args, "--workspace");
    call_in_workspace_scope(client, workspace, "surface.create", json!({})).await
}

fn env_opt(name: &str) -> Option<String> {
    env::var(name).ok()
}

fn nonempty(value: Option<String>) -> Option<String> {
    value.filter(|s| !s.trim().is_empty())
}

fn build_new_pane_request(
    args: &[String],
    env_lookup: impl Fn(&str) -> Option<String>,
) -> (Option<String>, Value) {
    let workspace =
        nonempty(parse_opt(args, "--workspace").or_else(|| env_lookup("LIMUX_WORKSPACE_ID")));
    let surface = nonempty(parse_opt(args, "--surface").or_else(|| env_lookup("LIMUX_SURFACE_ID")));
    let pane = nonempty(parse_opt(args, "--pane").or_else(|| env_lookup("LIMUX_PANE_ID")));
    let direction = parse_opt(args, "--direction").unwrap_or_else(|| "right".to_string());
    let pane_type = parse_opt(args, "--type").unwrap_or_else(|| "terminal".to_string());
    let command = nonempty(parse_opt(args, "--command"));
    let url = nonempty(parse_opt(args, "--url"));

    let mut params = Map::new();
    params.insert("direction".to_string(), Value::String(direction));
    params.insert("type".to_string(), Value::String(pane_type));
    if let Some(surface) = surface {
        params.insert("surface_id".to_string(), Value::String(surface));
    }
    if let Some(pane) = pane {
        params.insert("pane_id".to_string(), Value::String(pane));
    }
    if let Some(command) = command {
        params.insert("command".to_string(), Value::String(command));
    }
    if let Some(url) = url {
        params.insert("url".to_string(), Value::String(url));
    }

    (workspace, Value::Object(params))
}

fn new_pane_unexpected_positionals(args: &[String]) -> Vec<String> {
    const VALUE_OPTIONS: &[&str] = &[
        "--workspace",
        "--surface",
        "--pane",
        "--direction",
        "--type",
        "--command",
        "--url",
    ];
    let mut unexpected = Vec::new();
    let mut skip_value = false;

    for arg in args {
        if skip_value {
            skip_value = false;
            continue;
        }
        if VALUE_OPTIONS.contains(&arg.as_str()) {
            skip_value = true;
            continue;
        }
        if arg.starts_with('-') {
            continue;
        }
        unexpected.push(arg.clone());
    }

    unexpected
}

async fn run_new_pane(client: &mut Client, args: &[String]) -> Result<Value> {
    // `pane.create` contract shared with the core dispatcher and live GTK host:
    // direction/type are validated by the server, and responses keep
    // pane_id/pane_ref/surface_id/surface_ref. Inside a Limux terminal,
    // LIMUX_* defaults make `limux new-pane --command 'claude'` split the
    // caller's pane; outside Limux, omitting workspace preserves active-focus
    // server behavior.
    let unexpected = new_pane_unexpected_positionals(args);
    if !unexpected.is_empty() {
        bail!(
            "new-pane received unexpected positional argument(s): {}. Quote multi-word launch commands, for example: --command 'codex --ask-for-approval never'. Send arbitrary prompts later with limux send.",
            unexpected.join(", ")
        );
    }
    if let Some(command) = parse_opt(args, "--command") {
        validate_terminal_text_arg("pane.create command", &command)?;
    }
    let (workspace, params) = build_new_pane_request(args, env_opt);
    call_in_workspace_scope(client, workspace, "pane.create", params).await
}

async fn run_read_screen(client: &mut Client, args: &[String]) -> Result<Value> {
    if let Some(lines) = parse_opt(args, "--lines") {
        if lines.parse::<u64>().unwrap_or(0) == 0 {
            bail!("--lines must be greater than 0");
        }
    }

    let workspace = parse_opt(args, "--workspace");
    let surface = parse_opt(args, "--surface");
    let mut params = Map::new();
    if let Some(workspace) = workspace {
        params.insert("workspace_id".to_string(), Value::String(workspace));
    }
    if let Some(surface) = surface {
        params.insert("surface_id".to_string(), Value::String(surface));
    }

    client
        .call("surface.read_text", Value::Object(params))
        .await
}

async fn run_rename_workspace_like(
    client: &mut Client,
    command: &str,
    args: &[String],
) -> Result<Value> {
    let workspace = parse_opt(args, "--workspace").or_else(|| env::var("LIMUX_WORKSPACE_ID").ok());
    let title = trailing_title(args).ok_or_else(|| {
        if command == "rename-window" {
            anyhow!("rename-window requires a title")
        } else {
            anyhow!("rename-workspace requires a title")
        }
    })?;

    let mut params = Map::new();
    params.insert("title".to_string(), Value::String(title));
    if let Some(workspace) = workspace {
        params.insert("workspace_id".to_string(), Value::String(workspace));
    }

    client.call("workspace.rename", Value::Object(params)).await
}

async fn run_rename_tab(client: &mut Client, args: &[String]) -> Result<Value> {
    let workspace = parse_opt(args, "--workspace")
        .or_else(|| env::var("LIMUX_WORKSPACE_ID").ok())
        .unwrap_or_default();
    let tab = parse_opt(args, "--tab")
        .or_else(|| env::var("LIMUX_TAB_ID").ok())
        .unwrap_or_default();
    let title = trailing_title(args).ok_or_else(|| anyhow!("rename-tab requires a title"))?;

    let mut params = Map::new();
    params.insert("action".to_string(), Value::String("rename".to_string()));
    params.insert("title".to_string(), Value::String(title));
    if !workspace.is_empty() {
        params.insert("workspace_id".to_string(), Value::String(workspace));
    }
    if !tab.is_empty() {
        params.insert("surface_id".to_string(), Value::String(tab));
    }

    client.call("tab.action", Value::Object(params)).await
}

async fn run_tab_action(client: &mut Client, args: &[String]) -> Result<Value> {
    if parse_flag(args, "--help") {
        return Ok(json!({
            "help": "Usage: limux tab-action --action <name> [--workspace <id|ref>] [--tab <id|ref>] [--title <text>] [--url <url>]\nTarget tab:\n  --tab tab:<n>       Stable tab reference alias\n  --tab surface:<n>   Surface alias (legacy-compatible)\nExamples:\n  limux tab-action --workspace workspace:2 --tab tab:1 --action pin\n  limux tab-action --tab tab:3 --action mark-unread"
        }));
    }

    let action = parse_opt(args, "--action")
        .ok_or_else(|| anyhow!("tab-action requires --action <name>"))?;
    let workspace = parse_opt(args, "--workspace").or_else(|| env::var("LIMUX_WORKSPACE_ID").ok());
    let tab = parse_opt(args, "--tab").or_else(|| env::var("LIMUX_TAB_ID").ok());
    let title = parse_opt(args, "--title").or_else(|| trailing_title(args));
    let url = parse_opt(args, "--url");

    if action == "new-terminal-right" || action == "new-browser-right" {
        let pane_type = if action == "new-browser-right" {
            "browser"
        } else {
            "terminal"
        };
        let mut params = vec![
            "--direction".to_string(),
            "right".to_string(),
            "--type".to_string(),
            pane_type.to_string(),
        ];
        if let Some(workspace) = workspace.clone() {
            params.push("--workspace".to_string());
            params.push(workspace);
        }
        if let Some(url) = url {
            params.push("--url".to_string());
            params.push(url);
        }
        let created = run_new_pane(client, &params).await?;
        let tab_ref = tab.unwrap_or_else(|| "tab:1".to_string());
        return Ok(json!({
            "tab_ref": tab_ref,
            "surface_id": created.get("surface_id").cloned().unwrap_or(Value::Null),
            "surface_ref": created.get("surface_ref").cloned().unwrap_or(Value::Null),
        }));
    }

    let mut params = Map::new();
    params.insert("action".to_string(), Value::String(action.clone()));
    if let Some(workspace) = workspace {
        params.insert("workspace_id".to_string(), Value::String(workspace));
    }
    if let Some(tab) = tab.clone() {
        params.insert("surface_id".to_string(), Value::String(tab));
    }
    if let Some(title) = title {
        params.insert("title".to_string(), Value::String(title));
    }

    let mut payload = client.call("tab.action", Value::Object(params)).await?;
    if let Some(obj) = payload.as_object_mut() {
        if !obj.contains_key("tab_ref") {
            obj.insert(
                "tab_ref".to_string(),
                Value::String(tab.unwrap_or_else(|| "tab:1".to_string())),
            );
        }
        if action == "pin" {
            obj.insert("pinned".to_string(), Value::Bool(true));
        }
        if action == "unpin" {
            obj.insert("pinned".to_string(), Value::Bool(false));
        }
    }
    Ok(payload)
}

fn build_pane_action_request(
    args: &[String],
    env_lookup: impl Fn(&str) -> Option<String>,
) -> Result<Value> {
    let action = parse_opt(args, "--action")
        .ok_or_else(|| anyhow!("pane-action requires --action <name>"))?;
    let workspace =
        nonempty(parse_opt(args, "--workspace").or_else(|| env_lookup("LIMUX_WORKSPACE_ID")));
    let pane = nonempty(parse_opt(args, "--pane").or_else(|| env_lookup("LIMUX_PANE_ID")));
    let color = nonempty(parse_opt(args, "--color"));

    let mut params = Map::new();
    params.insert("action".to_string(), Value::String(action));
    if let Some(workspace) = workspace {
        params.insert("workspace_id".to_string(), Value::String(workspace));
    }
    if let Some(pane) = pane {
        params.insert("pane_id".to_string(), Value::String(pane));
    }
    if let Some(color) = color {
        params.insert("color".to_string(), Value::String(color));
    }

    Ok(Value::Object(params))
}

async fn run_pane_action(client: &mut Client, args: &[String]) -> Result<Value> {
    if parse_flag(args, "--help") {
        return Ok(json!({
            "help": "Usage: limux pane-action --action set_flag_color --color <orange|red|purple|pink|green|yellow|teal|cyan> [--workspace <id|ref>] [--pane <id|ref>]\n       limux pane-action --action clear_flag_color [--workspace <id|ref>] [--pane <id|ref>]\nDefaults:\n  --workspace falls back to LIMUX_WORKSPACE_ID\n  --pane falls back to LIMUX_PANE_ID"
        }));
    }

    let params = build_pane_action_request(args, env_opt)?;
    client.call("pane.action", params).await
}

async fn run_browser(
    client: &mut Client,
    args: &[String],
    json_output: bool,
) -> Result<CommandOutput> {
    let mut browser_args = args.to_vec();
    let mut local_json = json_output;

    loop {
        if browser_args.last().map(|s| s.as_str()) == Some("--json") {
            local_json = true;
            browser_args.pop();
            continue;
        }
        break;
    }

    let workspace = parse_opt(&browser_args, "--workspace");
    let mut surface = parse_opt(&browser_args, "--surface");

    let mut positional: Vec<String> = Vec::new();
    let mut skip = false;
    for (idx, arg) in browser_args.iter().enumerate() {
        if skip {
            skip = false;
            continue;
        }
        match arg.as_str() {
            "--workspace" | "--surface" | "--id-format" | "--timeout-ms" | "--load-state"
            | "--out" => {
                if idx + 1 < browser_args.len() {
                    skip = true;
                }
            }
            value if value.starts_with('-') => {}
            _ => positional.push(arg.clone()),
        }
    }

    if positional.is_empty() {
        bail!("browser requires a subcommand");
    }

    let mut pos_idx = 0usize;
    let first = positional[0].clone();
    let verbs_without_surface = ["open", "open-split", "new", "identify"];

    if !verbs_without_surface.contains(&first.as_str()) {
        if !first.contains(':') && !first.contains('-') {
            // probably still subcommand
        } else {
            surface = Some(first);
            pos_idx = 1;
        }
    }

    if pos_idx >= positional.len() {
        bail!("browser requires a subcommand");
    }
    let sub = positional[pos_idx].clone();
    let rest = positional[(pos_idx + 1)..].to_vec();

    let output = match sub.as_str() {
        "open" | "open-split" | "new" => {
            let url = rest
                .first()
                .cloned()
                .unwrap_or_else(|| "about:blank".to_string());
            if let Some(surface) = surface.clone() {
                let payload = browser_call(client, Some(surface), "browser.navigate", {
                    let mut p = Map::new();
                    p.insert("url".to_string(), Value::String(url));
                    p
                })
                .await?;
                CommandOutput::Json(payload)
            } else {
                let payload = call_in_workspace_scope(
                    client,
                    workspace.clone(),
                    "browser.open_split",
                    json!({ "url": url }),
                )
                .await?;
                CommandOutput::Json(payload)
            }
        }
        "url" | "get-url" => {
            let sid = surface
                .clone()
                .ok_or_else(|| anyhow!("browser url requires a surface"))?;
            let payload = browser_call(client, Some(sid), "browser.url.get", Map::new()).await?;
            if local_json {
                CommandOutput::Json(payload)
            } else {
                CommandOutput::Text(get_string(&payload, &["url"]).unwrap_or_default())
            }
        }
        "goto" | "navigate" => {
            let sid = surface
                .clone()
                .ok_or_else(|| anyhow!("browser navigate requires a surface"))?;
            let url = rest
                .first()
                .cloned()
                .ok_or_else(|| anyhow!("browser navigate requires a URL"))?;
            let payload = browser_call(client, Some(sid.clone()), "browser.navigate", {
                let mut p = Map::new();
                p.insert("url".to_string(), Value::String(url));
                p
            })
            .await?;
            if parse_flag(&browser_args, "--snapshot-after") {
                let snap = browser_call(client, Some(sid), "browser.snapshot", Map::new()).await?;
                if local_json {
                    let mut merged = payload;
                    if let Some(obj) = merged.as_object_mut() {
                        obj.insert("post_action_snapshot".to_string(), snap);
                    }
                    CommandOutput::Json(merged)
                } else {
                    CommandOutput::Text(
                        get_string(&snap, &["snapshot", "text"])
                            .unwrap_or_else(|| "OK".to_string()),
                    )
                }
            } else {
                CommandOutput::Json(payload)
            }
        }
        "wait" => {
            let sid = surface
                .clone()
                .ok_or_else(|| anyhow!("browser wait requires a surface"))?;
            let mut p = Map::new();
            if let Some(selector) = parse_opt(&browser_args, "--selector") {
                p.insert("selector".to_string(), Value::String(selector));
            }
            if let Some(timeout_ms) = parse_opt(&browser_args, "--timeout-ms") {
                if let Ok(ms) = timeout_ms.parse::<u64>() {
                    p.insert("timeout_ms".to_string(), Value::Number(ms.into()));
                }
            }
            let payload = browser_call(client, Some(sid), "browser.wait", p).await?;
            if local_json {
                CommandOutput::Json(payload)
            } else {
                CommandOutput::Text("OK".to_string())
            }
        }
        "snapshot" => {
            let sid = surface
                .clone()
                .ok_or_else(|| anyhow!("browser snapshot requires a surface"))?;
            let payload = browser_call(client, Some(sid), "browser.snapshot", Map::new()).await?;
            if local_json {
                CommandOutput::Json(payload)
            } else {
                let url = get_string(&payload, &["url"]).unwrap_or_default();
                if parse_flag(&browser_args, "--interactive") && url == "about:blank" {
                    CommandOutput::Text("about:blank\nNo interactive elements found; try `browser <surface> get url`.".to_string())
                } else if parse_flag(&browser_args, "--interactive") {
                    let mut text = get_string(&payload, &["snapshot", "text"])
                        .unwrap_or_else(|| "OK".to_string());
                    if let Some(refs) = payload.get("refs").and_then(Value::as_object) {
                        for key in refs.keys() {
                            text.push_str(&format!("\nref={}", key));
                        }
                    }
                    CommandOutput::Text(text)
                } else {
                    CommandOutput::Text(
                        get_string(&payload, &["snapshot", "text"])
                            .unwrap_or_else(|| "OK".to_string()),
                    )
                }
            }
        }
        "screenshot" => {
            let sid = surface
                .clone()
                .ok_or_else(|| anyhow!("browser screenshot requires a surface"))?;
            let mut payload =
                browser_call(client, Some(sid), "browser.screenshot", Map::new()).await?;
            let out = parse_opt(&browser_args, "--out");
            let mut path = get_string(&payload, &["path"])
                .unwrap_or_else(|| "/tmp/limux-browser-shot.png".to_string());
            if let Some(out_path) = out {
                path = out_path;
            }
            if !Path::new(&path).exists() {
                if let Some(parent) = Path::new(&path).parent() {
                    fs::create_dir_all(parent).with_context(|| {
                        format!("failed to create screenshot directory {}", parent.display())
                    })?;
                }
                fs::write(&path, [])
                    .with_context(|| format!("failed to create screenshot {}", path))?;
            }
            let url = format!("file://{}", path);
            if let Some(obj) = payload.as_object_mut() {
                obj.insert("path".to_string(), Value::String(path.clone()));
                obj.insert("url".to_string(), Value::String(url.clone()));
                obj.remove("png_base64");
            }
            if parse_opt(&browser_args, "--out").is_some() {
                CommandOutput::Text(format!("OK {}", path))
            } else if local_json {
                CommandOutput::Json(payload)
            } else {
                CommandOutput::Text(path)
            }
        }
        "find" => {
            let sid = surface
                .clone()
                .ok_or_else(|| anyhow!("browser find requires a surface"))?;
            let locator = rest.first().cloned().unwrap_or_else(|| "text".to_string());
            let value = rest.get(1).cloned().unwrap_or_default();
            let method = format!("browser.find.{}", locator);
            let mut params = Map::new();
            match locator.as_str() {
                "role" => {
                    params.insert("role".to_string(), Value::String(value));
                }
                "nth" => {
                    params.insert(
                        "selector".to_string(),
                        Value::String(rest.get(1).cloned().unwrap_or_default()),
                    );
                    let index = rest.get(2).and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
                    params.insert("index".to_string(), Value::Number(index.into()));
                }
                "first" | "last" => {
                    params.insert("selector".to_string(), Value::String(value));
                }
                _ => {
                    params.insert(locator.clone(), Value::String(value));
                }
            }
            let payload = browser_call(client, Some(sid), &method, params).await?;
            if local_json {
                CommandOutput::Json(payload)
            } else {
                CommandOutput::Text(
                    get_string(&payload, &["element_ref"]).unwrap_or_else(|| "@e1".to_string()),
                )
            }
        }
        "frame" => {
            let sid = surface
                .clone()
                .ok_or_else(|| anyhow!("browser frame requires a surface"))?;
            let target = rest.first().cloned().unwrap_or_else(|| "main".to_string());
            let payload = if target == "main" {
                browser_call(client, Some(sid), "browser.frame.main", Map::new()).await?
            } else {
                browser_call(client, Some(sid), "browser.frame.select", {
                    let mut p = Map::new();
                    p.insert("selector".to_string(), Value::String(target));
                    p
                })
                .await?
            };
            CommandOutput::Json(payload)
        }
        "click" => {
            let sid = surface
                .clone()
                .ok_or_else(|| anyhow!("browser click requires a surface"))?;
            let selector = parse_opt(&browser_args, "--selector")
                .or_else(|| rest.first().cloned())
                .ok_or_else(|| anyhow!("browser click requires a selector"))?;
            let payload = browser_call(client, Some(sid), "browser.click", {
                let mut p = Map::new();
                p.insert("selector".to_string(), Value::String(selector));
                p
            })
            .await?;
            CommandOutput::Json(payload)
        }
        "fill" => {
            let sid = surface
                .clone()
                .ok_or_else(|| anyhow!("browser fill requires a surface"))?;
            let selector = parse_opt(&browser_args, "--selector")
                .or_else(|| rest.first().cloned())
                .unwrap_or_default();
            let text = parse_opt(&browser_args, "--text")
                .or_else(|| rest.get(1).cloned())
                .unwrap_or_default();
            let payload = browser_call(client, Some(sid), "browser.fill", {
                let mut p = Map::new();
                p.insert("selector".to_string(), Value::String(selector));
                p.insert("text".to_string(), Value::String(text));
                p
            })
            .await?;
            if parse_flag(&browser_args, "--snapshot-after") {
                let snap =
                    browser_call(client, surface.clone(), "browser.snapshot", Map::new()).await?;
                if local_json {
                    let mut merged = payload;
                    if let Some(obj) = merged.as_object_mut() {
                        obj.insert("post_action_snapshot".to_string(), snap);
                    }
                    CommandOutput::Json(merged)
                } else {
                    CommandOutput::Text(
                        get_string(&snap, &["snapshot", "text"])
                            .unwrap_or_else(|| "OK".to_string()),
                    )
                }
            } else {
                CommandOutput::Json(payload)
            }
        }
        "get" => {
            let sid = surface
                .clone()
                .ok_or_else(|| anyhow!("browser get requires a surface"))?;
            let get_verb = rest.first().cloned().unwrap_or_else(|| "url".to_string());
            let method = match get_verb.as_str() {
                "url" => "browser.url.get".to_string(),
                "title" => "browser.get.title".to_string(),
                "text" => "browser.get.text".to_string(),
                "html" => "browser.get.html".to_string(),
                "value" => "browser.get.value".to_string(),
                "attr" => "browser.get.attr".to_string(),
                "count" => "browser.get.count".to_string(),
                "box" => "browser.get.box".to_string(),
                "styles" => "browser.get.styles".to_string(),
                other => bail!("Unsupported browser get subcommand: {}", other),
            };
            let selector = rest
                .get(1)
                .cloned()
                .or_else(|| parse_opt(&browser_args, "--selector"));
            let mut p = Map::new();
            if let Some(selector) = selector {
                p.insert("selector".to_string(), Value::String(selector));
            }
            if let Some(attr) = parse_opt(&browser_args, "--attr") {
                p.insert("name".to_string(), Value::String(attr));
            }
            if let Some(property) = parse_opt(&browser_args, "--property") {
                p.insert("property".to_string(), Value::String(property));
            }
            let payload = browser_call(client, Some(sid), &method, p).await?;
            if local_json {
                CommandOutput::Json(payload)
            } else {
                let text = get_string(&payload, &["url", "title", "text", "value", "html"])
                    .unwrap_or_else(|| "OK".to_string());
                CommandOutput::Text(text)
            }
        }
        "cookies" => {
            let sid = surface
                .clone()
                .ok_or_else(|| anyhow!("browser cookies requires a surface"))?;
            let op = rest.first().cloned().unwrap_or_else(|| "get".to_string());
            let method = match op.as_str() {
                "get" => "browser.cookies.get",
                "set" => "browser.cookies.set",
                "clear" => "browser.cookies.clear",
                _ => bail!("Unsupported browser cookies subcommand: {}", op),
            };
            let mut p = Map::new();
            if let Some(name) = rest
                .get(1)
                .cloned()
                .or_else(|| parse_opt(&browser_args, "--name"))
            {
                p.insert("name".to_string(), Value::String(name));
            }
            if let Some(value) = rest
                .get(2)
                .cloned()
                .or_else(|| parse_opt(&browser_args, "--value"))
            {
                p.insert("value".to_string(), Value::String(value));
            }
            let payload = browser_call(client, Some(sid), method, p).await?;
            CommandOutput::Json(payload)
        }
        "storage" => {
            let sid = surface
                .clone()
                .ok_or_else(|| anyhow!("browser storage requires a surface"))?;
            if rest.len() < 2 {
                bail!("browser storage requires <local|session> <get|set|clear>");
            }
            let storage_type = rest[0].clone();
            let op = rest[1].clone();
            let method = match op.as_str() {
                "get" => "browser.storage.get",
                "set" => "browser.storage.set",
                "clear" => "browser.storage.clear",
                _ => bail!("Unsupported browser storage subcommand: {}", op),
            };
            let mut p = Map::new();
            p.insert("type".to_string(), Value::String(storage_type));
            if let Some(key) = rest.get(2) {
                p.insert("key".to_string(), Value::String(key.clone()));
            }
            if let Some(value) = rest.get(3) {
                p.insert("value".to_string(), Value::String(value.clone()));
            }
            let payload = browser_call(client, Some(sid), method, p).await?;
            CommandOutput::Json(payload)
        }
        "tab" => {
            let sid = surface
                .clone()
                .ok_or_else(|| anyhow!("browser tab requires a surface"))?;
            let tab_verb = rest.first().cloned().unwrap_or_else(|| "list".to_string());
            let (method, p) = match tab_verb.as_str() {
                "list" => ("browser.tab.list", Map::new()),
                "new" => {
                    let mut p = Map::new();
                    if let Some(url) = rest.get(1) {
                        p.insert("url".to_string(), Value::String(url.clone()));
                    }
                    ("browser.tab.new", p)
                }
                "switch" => {
                    let mut p = Map::new();
                    if let Some(target) = rest.get(1) {
                        p.insert(
                            "target_surface_id".to_string(),
                            Value::String(target.clone()),
                        );
                    }
                    ("browser.tab.switch", p)
                }
                "close" => {
                    let mut p = Map::new();
                    if let Some(target) = rest.get(1) {
                        p.insert(
                            "target_surface_id".to_string(),
                            Value::String(target.clone()),
                        );
                    }
                    ("browser.tab.close", p)
                }
                _ => bail!("Unsupported browser tab subcommand: {}", tab_verb),
            };
            let payload = browser_call(client, Some(sid), method, p).await?;
            CommandOutput::Json(payload)
        }
        "addscript" | "addinitscript" | "addstyle" => {
            let sid = surface
                .clone()
                .ok_or_else(|| anyhow!("browser {} requires a surface", sub))?;
            let content = rest.join(" ");
            if content.trim().is_empty() {
                bail!("browser {} requires content", sub);
            }
            let field = if sub == "addstyle" { "css" } else { "script" };
            let method = format!("browser.{}", sub);
            let mut p = Map::new();
            p.insert(field.to_string(), Value::String(content));
            let payload = browser_call(client, Some(sid), &method, p).await?;
            CommandOutput::Json(payload)
        }
        "console" | "errors" => {
            let sid = surface
                .clone()
                .ok_or_else(|| anyhow!("browser {} requires a surface", sub))?;
            let op = rest.first().cloned().unwrap_or_else(|| "list".to_string());
            let method = format!("browser.{}.{}", sub, op);
            let payload = browser_call(client, Some(sid), &method, Map::new()).await?;
            CommandOutput::Json(payload)
        }
        "highlight" => {
            let sid = surface
                .clone()
                .ok_or_else(|| anyhow!("browser highlight requires a surface"))?;
            let selector = rest.first().cloned().unwrap_or_default();
            let payload = browser_call(client, Some(sid), "browser.highlight", {
                let mut p = Map::new();
                p.insert("selector".to_string(), Value::String(selector));
                p
            })
            .await?;
            CommandOutput::Json(payload)
        }
        "state" => {
            let sid = surface
                .clone()
                .ok_or_else(|| anyhow!("browser state requires a surface"))?;
            let op = rest.first().cloned().unwrap_or_else(|| "save".to_string());
            let path = rest
                .get(1)
                .cloned()
                .ok_or_else(|| anyhow!("browser state {} requires a file path", op))?;
            let method = match op.as_str() {
                "save" => "browser.state.save",
                "load" => "browser.state.load",
                _ => bail!("Unsupported browser state subcommand: {}", op),
            };
            let payload = browser_call(client, Some(sid), method, {
                let mut p = Map::new();
                p.insert("path".to_string(), Value::String(path));
                p
            })
            .await?;
            CommandOutput::Json(payload)
        }
        "viewport" => {
            bail!("not_supported: browser viewport is not supported in linux mock");
        }
        _ => {
            // Generic passthrough to browser.<sub>
            let sid = surface
                .clone()
                .ok_or_else(|| anyhow!("browser {} requires a surface", sub))?;
            let method = format!("browser.{}", sub);
            let payload = browser_call(client, Some(sid), &method, Map::new()).await?;
            CommandOutput::Json(payload)
        }
    };

    Ok(output)
}

fn is_unsupported_tmux_cmd(cmd: &str) -> bool {
    matches!(cmd, "popup" | "bind-key" | "unbind-key" | "copy-mode")
}

async fn run_tmux_compat(client: &mut Client, command: &str, args: &[String]) -> Result<Value> {
    if is_unsupported_tmux_cmd(command) {
        bail!("not supported");
    }

    match command {
        "capture-pane" => run_read_screen(client, args).await,
        "pipe-pane" => {
            let capture = run_read_screen(client, args).await?;
            let text = get_string(&capture, &["text"]).unwrap_or_default();
            let shell_cmd = parse_opt(args, "--command")
                .ok_or_else(|| anyhow!("pipe-pane requires --command"))?;
            let mut child = Command::new("bash")
                .arg("-lc")
                .arg(shell_cmd)
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .context("failed to spawn pipe-pane command")?;
            if let Some(stdin) = child.stdin.as_mut() {
                use std::io::Write;
                stdin
                    .write_all(text.as_bytes())
                    .context("failed to write pipe-pane stdin")?;
            }
            let status = child
                .wait()
                .context("failed waiting for pipe-pane command")?;
            if !status.success() {
                bail!("pipe-pane command failed");
            }
            Ok(json!({"ok": true}))
        }
        "wait-for" => {
            let signal = parse_flag(args, "-S") || parse_flag(args, "--signal");
            let name = trailing_title(args).ok_or_else(|| anyhow!("wait-for requires a name"))?;
            let timeout = parse_opt(args, "--timeout")
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(5);
            let path = wait_signal_path(&name);
            if signal {
                fs::write(&path, b"1").context("failed to write wait-for signal")?;
                Ok(json!({"ok": true, "name": name}))
            } else {
                let deadline = Instant::now() + Duration::from_secs(timeout);
                loop {
                    if path.exists() {
                        let _ = fs::remove_file(&path);
                        return Ok(json!({"ok": true, "name": name}));
                    }
                    if Instant::now() >= deadline {
                        bail!("wait-for timed out waiting for '{}'", name);
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
            }
        }
        "find-window" => {
            let needle = trailing_title(args).unwrap_or_default();
            let listed = client.call("workspace.list", json!({})).await?;
            let rows = listed
                .get("workspaces")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let mut out = String::new();
            for row in rows {
                let title = get_string(&row, &["title", "name"]).unwrap_or_default();
                if title.contains(&needle) {
                    let handle = handle_from_payload(&row, "workspace_id", "workspace_ref");
                    out = format!("{} {}", handle, title);
                    break;
                }
            }
            Ok(json!({"text": out}))
        }
        "last-window" => client.call("workspace.last", json!({})).await,
        "next-window" => client.call("workspace.next", json!({})).await,
        "previous-window" => client.call("workspace.previous", json!({})).await,
        "swap-pane" => {
            let workspace = parse_opt(args, "--workspace");
            let pane =
                parse_opt(args, "--pane").ok_or_else(|| anyhow!("swap-pane requires --pane"))?;
            let target = parse_opt(args, "--target-pane")
                .ok_or_else(|| anyhow!("swap-pane requires --target-pane"))?;

            let source_surface =
                selected_surface_for_pane(client, workspace.clone(), &pane).await?;
            let target_surface =
                selected_surface_for_pane(client, workspace.clone(), &target).await?;

            let _ = call_in_workspace_scope(
                client,
                workspace.clone(),
                "surface.move",
                json!({"surface_id": source_surface, "target_pane_id": target, "index": 0}),
            )
            .await?;
            let _ = call_in_workspace_scope(
                client,
                workspace.clone(),
                "surface.move",
                json!({"surface_id": target_surface, "target_pane_id": pane, "index": 0}),
            )
            .await?;

            Ok(json!({"ok": true}))
        }
        "break-pane" => {
            let workspace = parse_opt(args, "--workspace");
            let pane = parse_opt(args, "--pane");
            let surface = parse_opt(args, "--surface");
            let mut p = Map::new();
            if let Some(pane) = pane {
                p.insert("pane_id".to_string(), Value::String(pane));
            }
            if let Some(surface) = surface {
                p.insert("surface_id".to_string(), Value::String(surface));
            }
            call_in_workspace_scope(client, workspace, "pane.break", Value::Object(p)).await
        }
        "join-pane" => {
            let workspace = parse_opt(args, "--workspace");
            let pane = parse_opt(args, "--pane");
            let surface = parse_opt(args, "--surface");
            let target = parse_opt(args, "--target-pane")
                .ok_or_else(|| anyhow!("join-pane requires --target-pane"))?;
            let mut p = Map::new();
            p.insert("target_pane_id".to_string(), Value::String(target));
            if let Some(pane) = pane {
                p.insert("pane_id".to_string(), Value::String(pane));
            }
            if let Some(surface) = surface {
                p.insert("surface_id".to_string(), Value::String(surface));
            }
            call_in_workspace_scope(client, workspace, "pane.join", Value::Object(p)).await
        }
        "last-pane" => {
            let workspace = parse_opt(args, "--workspace");
            call_in_workspace_scope(client, workspace, "pane.last", json!({})).await
        }
        "clear-history" => {
            let workspace = parse_opt(args, "--workspace");
            let surface = parse_opt(args, "--surface");
            let mut p = Map::new();
            if let Some(surface) = surface {
                p.insert("surface_id".to_string(), Value::String(surface));
            }
            call_in_workspace_scope(client, workspace, "surface.clear_history", Value::Object(p))
                .await
        }
        "set-hook" => {
            let list_mode = parse_flag(args, "--list");
            let unset = parse_opt(args, "--unset");
            with_locked_json_map(&client.socket, "hooks", |hooks, path| {
                if list_mode {
                    let text = hooks
                        .iter()
                        .map(|(k, v)| format!("{}={}", k, v))
                        .collect::<Vec<_>>()
                        .join("\n");
                    return Ok(json!({
                        "text": text,
                        "path": path.display().to_string(),
                    }));
                }
                if let Some(name) = unset {
                    hooks.remove(&name);
                    write_json_map(path, hooks)?;
                    return Ok(json!({"ok": true}));
                }
                let name = args
                    .iter()
                    .find(|a| !a.starts_with('-'))
                    .cloned()
                    .unwrap_or_default();
                let body = trailing_title(args).unwrap_or_default();
                if name.is_empty() || body.is_empty() {
                    bail!("set-hook requires <name> <command>");
                }
                hooks.insert(name, body);
                write_json_map(path, hooks)?;
                Ok(json!({"ok": true}))
            })
        }
        "resize-pane" => {
            let workspace = parse_opt(args, "--workspace");
            let pane =
                parse_opt(args, "--pane").ok_or_else(|| anyhow!("resize-pane requires --pane"))?;
            let direction = if parse_flag(args, "-R") {
                "right"
            } else if parse_flag(args, "-L") {
                "left"
            } else if parse_flag(args, "-D") {
                "down"
            } else if parse_flag(args, "-U") {
                "up"
            } else {
                "right"
            };
            let amount = parse_opt(args, "--amount")
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(1);
            call_in_workspace_scope(
                client,
                workspace,
                "pane.resize",
                json!({"pane_id": pane, "direction": direction, "amount": amount}),
            )
            .await
        }
        "set-buffer" => {
            let name =
                parse_opt(args, "--name").ok_or_else(|| anyhow!("set-buffer requires --name"))?;
            let body = trailing_title(args).unwrap_or_default();
            with_locked_json_map(&client.socket, "buffers", |buffers, path| {
                buffers.insert(name, body);
                write_json_map(path, buffers)?;
                Ok(json!({"ok": true}))
            })
        }
        "list-buffers" => with_locked_json_map(&client.socket, "buffers", |buffers, _path| {
            let text = buffers.keys().cloned().collect::<Vec<_>>().join("\n");
            Ok(json!({"text": text}))
        }),
        "paste-buffer" => {
            let name =
                parse_opt(args, "--name").ok_or_else(|| anyhow!("paste-buffer requires --name"))?;
            let workspace = parse_opt(args, "--workspace");
            let surface = parse_opt(args, "--surface");
            let text = with_locked_json_map(&client.socket, "buffers", |buffers, _path| {
                Ok(buffers.get(&name).cloned().unwrap_or_default())
            })?;
            validate_terminal_text_arg("paste-buffer text", &text)?;
            let mut p = Map::new();
            if let Some(surface) = surface {
                p.insert("surface_id".to_string(), Value::String(surface));
            }
            p.insert("text".to_string(), Value::String(text));
            call_in_workspace_scope(client, workspace, "surface.send_text", Value::Object(p)).await
        }
        "respawn-pane" => {
            let workspace = parse_opt(args, "--workspace");
            let surface = parse_opt(args, "--surface");
            let command = parse_opt(args, "--command").unwrap_or_default();
            validate_terminal_text_arg("respawn-pane command", &command)?;
            let mut p = Map::new();
            if let Some(surface) = surface {
                p.insert("surface_id".to_string(), Value::String(surface));
            }
            p.insert("text".to_string(), Value::String(format!("{}\n", command)));
            call_in_workspace_scope(client, workspace, "surface.send_text", Value::Object(p)).await
        }
        "display-message" => {
            let msg = trailing_title(args).unwrap_or_default();
            Ok(json!({"text": msg}))
        }
        _ => bail!("unknown tmux command"),
    }
}

fn socket_mode_label(mode: SocketMode) -> &'static str {
    match mode {
        SocketMode::Runtime => "runtime",
        SocketMode::Debug => "debug",
    }
}

fn target_info_payload(client: &Client, opts: &GlobalOptions) -> Value {
    json!({
        "resolved_socket": client.socket.to_string_lossy().to_string(),
        "socket_mode": socket_mode_label(opts.socket_mode),
        "explicit_socket": opts
            .socket
            .as_ref()
            .map(|socket| socket.to_string_lossy().to_string()),
        "explicit_channel": opts.channel.as_ref().map(RuntimeChannel::env_value),
        "inherited": {
            "LIMUX_SOCKET": limux_env_value("LIMUX_SOCKET"),
            "LIMUX_SOCKET_PATH": limux_env_value("LIMUX_SOCKET_PATH"),
            "LIMUX_CHANNEL": limux_env_value(limux_control::socket_path::LIMUX_CHANNEL_ENV),
            "LIMUX_PREVIEW_ID": limux_env_value(limux_control::socket_path::LIMUX_PREVIEW_ID_ENV),
        },
        "connects": false,
    })
}

fn render_target_info_text(payload: &Value) -> String {
    let resolved_socket = payload
        .get("resolved_socket")
        .and_then(Value::as_str)
        .unwrap_or("");
    let socket_mode = payload
        .get("socket_mode")
        .and_then(Value::as_str)
        .unwrap_or("");
    let explicit_socket = payload
        .get("explicit_socket")
        .and_then(Value::as_str)
        .unwrap_or("none");
    let explicit_channel = payload
        .get("explicit_channel")
        .and_then(Value::as_str)
        .unwrap_or("none");

    format!(
        "resolved_socket={resolved_socket}\nsocket_mode={socket_mode}\nexplicit_socket={explicit_socket}\nexplicit_channel={explicit_channel}\nconnects=false"
    )
}

async fn execute_command(client: &mut Client, opts: &GlobalOptions) -> Result<CommandOutput> {
    if let Some(raw_request) = &opts.request {
        let request: V2Request =
            serde_json::from_str(raw_request).context("request must be a valid v2 JSON object")?;
        let mut payload = client.send_request(request).await?;
        apply_id_format(&mut payload, opts.id_format);
        return Ok(CommandOutput::Json(payload));
    }

    if opts.command_args.is_empty() {
        print_help();
        bail!("missing command");
    }

    let command = opts.command_args[0].as_str();
    let args = &opts.command_args[1..];
    let mut effective_id_format = opts.id_format;
    if command == "browser" {
        if let Some(raw) = parse_opt(args, "--id-format") {
            effective_id_format = IdFormat::parse(&raw)?;
        }
    }

    let mut out = match command {
        "target-info" | "socket-info" => {
            let payload = target_info_payload(client, opts);
            if opts.json_output {
                CommandOutput::Json(payload)
            } else {
                CommandOutput::Text(render_target_info_text(&payload))
            }
        }
        "identify" => CommandOutput::Json(run_identify(client, args).await?),
        "doctor" => {
            let run = doctor::run(
                args,
                opts.json_output || doctor::wants_json(args, false),
                client.socket.clone(),
                current_cli_build_info(),
            )
            .await?;
            if run.json_output {
                CommandOutput::JsonWithExit(run.payload, run.exit_code)
            } else {
                CommandOutput::TextWithExit(run.text, run.exit_code)
            }
        }
        "list-panels" | "list-panes" | "list-workspaces" | "surface-health" => {
            let payload = run_list(client, command, args).await?;
            if opts.json_output {
                CommandOutput::Json(payload)
            } else {
                CommandOutput::Text(render_list_text(command, &payload))
            }
        }
        "send" => {
            let payload = run_send(client, args).await?;
            if opts.json_output {
                CommandOutput::Json(payload)
            } else {
                let handle = handle_from_payload(&payload, "surface_id", "surface_ref");
                CommandOutput::Text(format!("OK {}", handle.trim()))
            }
        }
        "send-key" => {
            let payload = run_send_key(client, args).await?;
            if opts.json_output {
                CommandOutput::Json(payload)
            } else {
                let handle = handle_from_payload(&payload, "surface_id", "surface_ref");
                CommandOutput::Text(format!("OK {}", handle.trim()))
            }
        }
        "notify" => {
            let payload = run_notify(client, args).await?;
            if opts.json_output {
                CommandOutput::Json(payload)
            } else {
                CommandOutput::Text("OK".to_string())
            }
        }
        "claude-hook" | "opencode-hook" | "gemini-hook" | "hermes-hook" => {
            let agent = match command {
                "claude-hook" => agent_hooks::AgentKind::Claude,
                "opencode-hook" => agent_hooks::AgentKind::OpenCode,
                "gemini-hook" => agent_hooks::AgentKind::Gemini,
                "hermes-hook" => agent_hooks::AgentKind::Hermes,
                _ => unreachable!(),
            };
            let payload = run_agent_hook(client, agent, args).await?;
            if opts.json_output {
                CommandOutput::Json(payload)
            } else {
                CommandOutput::Text("OK".to_string())
            }
        }
        "hooks" => return run_hooks_command(client, args, opts.json_output).await,
        "new-workspace" => {
            let payload = run_new_workspace(client, args).await?;
            if opts.json_output {
                CommandOutput::Json(payload)
            } else {
                let handle = handle_from_payload(&payload, "workspace_id", "workspace_ref");
                CommandOutput::Text(format!("OK {}", handle))
            }
        }
        "close-workspace" => {
            let payload = run_close_workspace(client, args).await?;
            if opts.json_output {
                CommandOutput::Json(payload)
            } else {
                CommandOutput::Text("OK".to_string())
            }
        }
        "agent-team" => {
            let payload = run_agent_team(client, args).await?;
            if opts.json_output {
                CommandOutput::Json(payload)
            } else if let Some(help) = get_string(&payload, &["help"]) {
                CommandOutput::Text(help)
            } else {
                let agents_md = payload
                    .get("agents_md")
                    .and_then(|v| v.as_str())
                    .unwrap_or("LIMUX_AGENTS.md");
                let workspace = payload
                    .get("workspace_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let peers = payload
                    .get("peers")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|p| p.get("agent").and_then(|v| v.as_str()))
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or_default();
                let bootstrap = payload
                    .get("bootstrap")
                    .and_then(|v| v.get("status"))
                    .and_then(Value::as_str)
                    .unwrap_or("unknown");
                CommandOutput::Text(format!(
                    "OK agent-team workspace={workspace} peers=[{peers}] agents_md={agents_md} bootstrap={bootstrap}"
                ))
            }
        }
        "review" => {
            let payload = run_review_command(client, args).await?;
            if opts.json_output {
                CommandOutput::Json(payload)
            } else {
                let review_command = payload
                    .get("review_command")
                    .and_then(Value::as_str)
                    .unwrap_or("prepare");
                if review_command == "spawn" {
                    let request = payload
                        .get("request_path")
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    let ledger = payload
                        .get("ledger_path")
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    let evidence = payload
                        .get("evidence_path")
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    let surface = payload
                        .get("reviewer_surface_id")
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    let prompt_status = payload
                        .get("prompt")
                        .and_then(|value| value.get("status"))
                        .and_then(Value::as_str)
                        .unwrap_or("unknown");
                    let dry_run = payload
                        .get("dry_run")
                        .and_then(Value::as_bool)
                        .unwrap_or(false);
                    CommandOutput::Text(format!(
                        "OK review spawn request={request} ledger={ledger} evidence={evidence} surface={surface} prompt={prompt_status} dry_run={dry_run}"
                    ))
                } else {
                    let request = payload
                        .get("request_path")
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    let ledger = payload
                        .get("ledger_path")
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    let dry_run = payload
                        .get("dry_run")
                        .and_then(Value::as_bool)
                        .unwrap_or(false);
                    let prompt = payload.get("prompt").and_then(Value::as_str).unwrap_or("");
                    CommandOutput::Text(format!(
                        "OK review prepare request={request} ledger={ledger} dry_run={dry_run}\n\n{prompt}"
                    ))
                }
            }
        }
        "sidebar-state" => {
            let payload = run_sidebar_state(client, args).await?;
            if opts.json_output {
                CommandOutput::Json(payload)
            } else {
                let workspace =
                    get_string(&payload, &["workspace"]).unwrap_or_else(|| "none".to_string());
                let cwd = get_string(&payload, &["cwd"]).unwrap_or_else(|| "none".to_string());
                let git_branch =
                    get_string(&payload, &["git_branch"]).unwrap_or_else(|| "none".to_string());
                CommandOutput::Text(format!(
                    "workspace={}\ncwd={}\ngit_branch={}",
                    workspace, cwd, git_branch
                ))
            }
        }
        "new-surface" => {
            let payload = run_new_surface(client, args).await?;
            if opts.json_output {
                CommandOutput::Json(payload)
            } else {
                let handle = handle_from_payload(&payload, "surface_id", "surface_ref");
                CommandOutput::Text(format!("OK {}", handle))
            }
        }
        "new-pane" => {
            let payload = run_new_pane(client, args).await?;
            if opts.json_output {
                CommandOutput::Json(payload)
            } else {
                let handle = handle_from_payload(&payload, "surface_id", "surface_ref");
                CommandOutput::Text(format!("OK {}", handle))
            }
        }
        "tab-action" => {
            let payload = run_tab_action(client, args).await?;
            if opts.json_output {
                CommandOutput::Json(payload)
            } else if let Some(help) = get_string(&payload, &["help"]) {
                CommandOutput::Text(help)
            } else {
                CommandOutput::Text("OK".to_string())
            }
        }
        "pane-action" => {
            let payload = run_pane_action(client, args).await?;
            if opts.json_output {
                CommandOutput::Json(payload)
            } else if let Some(help) = get_string(&payload, &["help"]) {
                CommandOutput::Text(help)
            } else {
                CommandOutput::Text("OK".to_string())
            }
        }
        "rename-workspace" | "rename-window" => {
            let payload = run_rename_workspace_like(client, command, args).await?;
            if opts.json_output {
                CommandOutput::Json(payload)
            } else {
                CommandOutput::Text("OK".to_string())
            }
        }
        "rename-tab" => {
            let payload = run_rename_tab(client, args).await?;
            if opts.json_output {
                CommandOutput::Json(payload)
            } else {
                CommandOutput::Text("OK".to_string())
            }
        }
        "read-screen" | "capture-pane" => {
            let payload = run_read_screen(client, args).await?;
            if opts.json_output {
                CommandOutput::Json(payload)
            } else {
                CommandOutput::Text(get_string(&payload, &["text"]).unwrap_or_default())
            }
        }
        "browser" => return run_browser(client, args, opts.json_output).await,
        "open-browser" => {
            let mut bridged = vec!["open".to_string()];
            bridged.extend(args.iter().cloned());
            return run_browser(client, &bridged, opts.json_output).await;
        }
        "navigate-browser" => {
            let mut bridged = vec!["navigate".to_string()];
            bridged.extend(args.iter().cloned());
            return run_browser(client, &bridged, opts.json_output).await;
        }
        "browser-back" => {
            let mut bridged = vec!["back".to_string()];
            bridged.extend(args.iter().cloned());
            return run_browser(client, &bridged, opts.json_output).await;
        }
        "browser-forward" => {
            let mut bridged = vec!["forward".to_string()];
            bridged.extend(args.iter().cloned());
            return run_browser(client, &bridged, opts.json_output).await;
        }
        "browser-reload" => {
            let mut bridged = vec!["reload".to_string()];
            bridged.extend(args.iter().cloned());
            return run_browser(client, &bridged, opts.json_output).await;
        }
        "pipe-pane" | "wait-for" | "find-window" | "last-window" | "next-window"
        | "previous-window" | "swap-pane" | "break-pane" | "join-pane" | "last-pane"
        | "clear-history" | "set-hook" | "resize-pane" | "set-buffer" | "list-buffers"
        | "paste-buffer" | "respawn-pane" | "display-message" | "popup" | "bind-key"
        | "unbind-key" | "copy-mode" => {
            let payload = run_tmux_compat(client, command, args).await?;
            if opts.json_output {
                CommandOutput::Json(payload)
            } else if let Some(text) = get_string(&payload, &["text"]) {
                CommandOutput::Text(text)
            } else {
                CommandOutput::Text("OK".to_string())
            }
        }
        _ => bail!("unknown command: {}", command),
    };

    if let CommandOutput::Json(ref mut payload) = out {
        apply_id_format(payload, effective_id_format);
    }

    Ok(out)
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts = parse_global_args()?;
    if should_launch_host(&opts) {
        return launch_host(opts.channel.as_ref());
    }

    let socket = if opts.channel.is_some() {
        resolve_socket_path_for_channel(
            opts.socket.clone(),
            opts.socket_mode,
            opts.channel.as_ref(),
        )
    } else {
        resolve_socket_path(opts.socket.clone(), opts.socket_mode)
    };

    let mut client = Client::new(socket);
    let output = execute_command(&mut client, &opts).await;

    match output {
        Ok(CommandOutput::Text(text)) => {
            println!("{}", text);
            Ok(())
        }
        Ok(CommandOutput::Json(value)) => {
            if opts.pretty {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&value)
                        .context("failed to pretty print response")?
                );
            } else {
                println!(
                    "{}",
                    serde_json::to_string(&value).context("failed to encode json output")?
                );
            }
            Ok(())
        }
        Ok(CommandOutput::TextWithExit(text, code)) => {
            println!("{}", text);
            if code != 0 {
                std::process::exit(code);
            }
            Ok(())
        }
        Ok(CommandOutput::JsonWithExit(value, code)) => {
            if opts.pretty {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&value)
                        .context("failed to pretty print response")?
                );
            } else {
                println!(
                    "{}",
                    serde_json::to_string(&value).context("failed to encode json output")?
                );
            }
            if code != 0 {
                std::process::exit(code);
            }
            Ok(())
        }
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod cli_arg_tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    fn default_opts(command_args: Vec<String>) -> GlobalOptions {
        GlobalOptions {
            socket: None,
            channel: None,
            socket_mode: SocketMode::Runtime,
            json_output: false,
            id_format: IdFormat::Refs,
            request: None,
            pretty: false,
            command_args,
        }
    }

    #[test]
    fn no_args_launches_host_but_cli_flags_do_not() {
        assert!(should_launch_host(&default_opts(Vec::new())));

        let mut json_only = default_opts(Vec::new());
        json_only.json_output = true;
        assert!(!should_launch_host(&json_only));

        assert!(!should_launch_host(&default_opts(args(&[
            "list-workspaces"
        ]))));
    }

    #[test]
    fn host_binary_candidates_cover_installed_and_dev_layouts() {
        let installed = Path::new("/usr/bin/limux");
        let candidates = host_binary_candidates(installed);
        assert!(candidates.contains(&PathBuf::from("/usr/libexec/limux/limux-host")));
        assert!(!candidates.contains(&PathBuf::from("/usr/bin/limux")));

        let dev = Path::new("/repo/target/debug/limux-cli");
        let candidates = host_binary_candidates(dev);
        assert!(candidates.contains(&PathBuf::from("/repo/target/debug/limux")));
    }

    #[test]
    fn host_launch_command_clears_runtime_target_env_but_preserves_explicit_socket() {
        let command = host_launch_command_with_inherited_target_env(
            Path::new("/tmp/limux-host"),
            false,
            None,
        );
        let removals = command
            .get_envs()
            .filter_map(|(key, value)| value.is_none().then_some(key.to_string_lossy()))
            .collect::<Vec<_>>();

        for key in HOST_LAUNCH_TARGET_ENV_REMOVALS {
            assert!(
                removals.iter().any(|removed| removed == key),
                "missing env removal for {key}"
            );
        }
        for key in HOST_LAUNCH_SOCKET_ENV_REMOVALS
            .iter()
            .chain(HOST_LAUNCH_SESSION_ENV_REMOVALS.iter())
        {
            assert!(
                !removals.iter().any(|removed| removed == key),
                "explicit runtime env should be preserved for {key}"
            );
        }
    }

    #[test]
    fn host_launch_command_sets_explicit_runtime_channel() {
        let channel = RuntimeChannel::Preview("branch".to_string());
        let command = host_launch_command_with_inherited_target_env(
            Path::new("/tmp/limux-host"),
            false,
            Some(&channel),
        );
        let channel_env = command
            .get_envs()
            .find_map(|(key, value)| {
                (key == limux_control::socket_path::LIMUX_CHANNEL_ENV)
                    .then(|| value.map(|value| value.to_string_lossy().to_string()))
            })
            .flatten();

        assert_eq!(channel_env.as_deref(), Some("preview:branch"));
    }

    #[test]
    fn host_launch_command_clears_inherited_channel_without_explicit_channel() {
        let command =
            host_launch_command_with_inherited_target_env(Path::new("/tmp/limux-host"), true, None);
        let removals = command
            .get_envs()
            .filter_map(|(key, value)| value.is_none().then_some(key.to_string_lossy()))
            .collect::<Vec<_>>();

        for key in [
            limux_control::socket_path::LIMUX_CHANNEL_ENV,
            limux_control::socket_path::LIMUX_PREVIEW_ID_ENV,
        ] {
            assert!(
                removals.iter().any(|removed| removed == key),
                "missing inherited channel env removal for {key}"
            );
        }
    }

    #[test]
    fn target_info_reports_resolved_preview_socket_without_connecting() {
        let mut opts = default_opts(args(&["target-info"]));
        opts.channel = Some(RuntimeChannel::Preview("branch".to_string()));
        let client = Client::new(PathBuf::from("/tmp/limux-preview-branch.sock"));

        let payload = target_info_payload(&client, &opts);

        assert_eq!(payload["resolved_socket"], "/tmp/limux-preview-branch.sock");
        assert_eq!(payload["socket_mode"], "runtime");
        assert_eq!(payload["explicit_channel"], "preview:branch");
        assert_eq!(payload["connects"], false);
    }

    #[test]
    fn target_info_text_includes_channel_and_no_connect_marker() {
        let mut opts = default_opts(args(&["socket-info"]));
        opts.channel = Some(RuntimeChannel::Stable);
        let client = Client::new(PathBuf::from("/tmp/limux-stable.sock"));

        let payload = target_info_payload(&client, &opts);
        let rendered = render_target_info_text(&payload);

        assert!(rendered.contains("resolved_socket=/tmp/limux-stable.sock"));
        assert!(rendered.contains("explicit_channel=stable"));
        assert!(rendered.contains("connects=false"));
    }

    #[test]
    fn host_launch_env_removals_clear_socket_when_target_env_is_inherited() {
        let removals = host_launch_env_removals(true, false);
        for key in HOST_LAUNCH_TARGET_ENV_REMOVALS
            .iter()
            .chain(HOST_LAUNCH_SOCKET_ENV_REMOVALS.iter())
            .chain(HOST_LAUNCH_SESSION_ENV_REMOVALS.iter())
        {
            assert!(
                removals.iter().any(|removed| removed == key),
                "missing inherited env removal for {key}"
            );
        }
    }

    #[test]
    fn host_launch_env_removals_preserve_socket_without_inherited_target() {
        let removals = host_launch_env_removals(false, false);
        for key in HOST_LAUNCH_TARGET_ENV_REMOVALS {
            assert!(
                removals.iter().any(|removed| removed == key),
                "missing target env removal for {key}"
            );
        }
        for key in HOST_LAUNCH_SOCKET_ENV_REMOVALS
            .iter()
            .chain(HOST_LAUNCH_SESSION_ENV_REMOVALS.iter())
        {
            assert!(
                !removals.iter().any(|removed| removed == key),
                "runtime env should not be removed without inherited target env for {key}"
            );
        }
    }

    #[test]
    fn host_launch_env_removals_clear_socket_for_explicit_channel() {
        let removals = host_launch_env_removals(false, true);
        for key in HOST_LAUNCH_TARGET_ENV_REMOVALS
            .iter()
            .chain(HOST_LAUNCH_SOCKET_ENV_REMOVALS.iter())
            .chain(HOST_LAUNCH_SESSION_ENV_REMOVALS.iter())
        {
            assert!(
                removals.iter().any(|removed| removed == key),
                "missing explicit channel env removal for {key}"
            );
        }
    }

    #[tokio::test]
    async fn send_rejects_disallowed_terminal_control_before_socket_contact() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let mut client = Client::new(tmp.path().join("unused.sock"));

        let err = run_send(&mut client, &args(&["hello\u{1b}[31m"]))
            .await
            .expect_err("escape should fail before socket contact");

        let msg = format!("{err:#}");
        assert!(
            msg.contains("surface.send_text text"),
            "unexpected error: {msg}"
        );
        assert!(msg.contains("U+001B"), "unexpected error: {msg}");
    }

    #[tokio::test]
    async fn send_allows_multiline_agent_messages() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let mut client = Client::new(tmp.path().join("unused.sock"));
        let err = run_send(
            &mut client,
            &args(&["<agent-msg>\n\t<request>ok</request>\r\n</agent-msg>\n"]),
        )
        .await
        .expect_err("valid text should reach the socket layer");

        assert!(
            format!("{err:#}").contains("failed to connect"),
            "unexpected error: {err:#}"
        );
    }

    #[tokio::test]
    async fn new_workspace_rejects_disallowed_terminal_control_command_before_socket_contact() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let mut client = Client::new(tmp.path().join("unused.sock"));

        let err = run_new_workspace(&mut client, &args(&["--command", "claude\u{7}"]))
            .await
            .expect_err("BEL should fail before socket contact");

        let msg = format!("{err:#}");
        assert!(
            msg.contains("workspace.create command"),
            "unexpected error: {msg}"
        );
    }

    #[tokio::test]
    async fn new_pane_rejects_disallowed_terminal_control_command_before_socket_contact() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let mut client = Client::new(tmp.path().join("unused.sock"));

        let err = run_new_pane(&mut client, &args(&["--command", "codex\u{9b}0m"]))
            .await
            .expect_err("C1 control should fail before socket contact");

        let msg = format!("{err:#}");
        assert!(
            msg.contains("pane.create command"),
            "unexpected error: {msg}"
        );
        assert!(msg.contains("U+009B"), "unexpected error: {msg}");
    }

    #[tokio::test]
    async fn respawn_pane_rejects_disallowed_terminal_control_command_before_socket_contact() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let mut client = Client::new(tmp.path().join("unused.sock"));

        let err = run_tmux_compat(
            &mut client,
            "respawn-pane",
            &args(&["--command", "echo bad\u{1b}"]),
        )
        .await
        .expect_err("ESC should fail before socket contact");

        let msg = format!("{err:#}");
        assert!(
            msg.contains("respawn-pane command"),
            "unexpected error: {msg}"
        );
    }

    #[tokio::test]
    async fn paste_buffer_rejects_stored_disallowed_terminal_control_before_socket_contact() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let mut client = Client::new(tmp.path().join("limux.sock"));

        run_tmux_compat(
            &mut client,
            "set-buffer",
            &args(&["--name", "bad", "hello\u{1b}[31m"]),
        )
        .await
        .expect("set-buffer should store text without terminal injection");

        let err = run_tmux_compat(&mut client, "paste-buffer", &args(&["--name", "bad"]))
            .await
            .expect_err("stored ESC should fail before socket contact");

        let msg = format!("{err:#}");
        assert!(msg.contains("paste-buffer text"), "unexpected error: {msg}");
    }

    #[test]
    fn notify_positional_title_skips_option_values() {
        let args = args(&[
            "--subtitle",
            "needs review",
            "--body",
            "blocked",
            "Input needed",
        ]);

        assert_eq!(trailing_title(&args).as_deref(), Some("Input needed"));
    }

    #[test]
    fn hook_event_comes_from_json_after_option_values() {
        let args = args(&["--workspace", "codex"]);
        let payload = json!({ "hook_event_name": "Notification" });

        assert_eq!(parse_hook_event(&args, &payload), "Notification");
    }

    #[test]
    fn hook_event_prefers_explicit_event_flag() {
        let args = args(&["--workspace", "codex", "--event", "Stop"]);
        let payload = json!({ "hook_event_name": "Notification" });

        assert_eq!(parse_hook_event(&args, &payload), "Stop");
    }

    #[test]
    fn hook_event_accepts_positional_event_after_options() {
        let args = args(&["--workspace", "codex", "Stop"]);
        let payload = json!({ "hook_event_name": "Notification" });

        assert_eq!(parse_hook_event(&args, &payload), "Stop");
    }

    #[test]
    fn external_session_end_preserves_restorable_hook_session() {
        assert_eq!(
            agent_hook_persistence_action("SessionEnd"),
            AgentHookPersistenceAction::Preserve
        );
        assert_eq!(
            agent_hook_persistence_action("session-end"),
            AgentHookPersistenceAction::Preserve
        );
        assert_eq!(
            agent_hook_persistence_action("on_session_end"),
            AgentHookPersistenceAction::Preserve
        );
        assert_eq!(
            agent_hook_persistence_action("on_session_finalize"),
            AgentHookPersistenceAction::Preserve
        );
    }

    #[test]
    fn internal_cleanup_removes_restorable_hook_session() {
        assert_eq!(
            agent_hook_persistence_action("cleanup"),
            AgentHookPersistenceAction::Remove
        );
        assert_eq!(
            agent_hook_persistence_action("restore-exit"),
            AgentHookPersistenceAction::Remove
        );
    }

    #[test]
    fn default_hook_setup_omits_opencode_until_supported() {
        assert_eq!(
            default_hook_targets(),
            vec![
                agent_hooks::AgentKind::Codex,
                agent_hooks::AgentKind::Claude,
                agent_hooks::AgentKind::Gemini,
            ]
        );
        assert!(!default_hook_targets().contains(&agent_hooks::AgentKind::OpenCode));
        assert!(!default_hook_targets().contains(&agent_hooks::AgentKind::Hermes));
    }

    #[test]
    fn opencode_plugin_embeds_installer_cli_command() {
        let source = opencode_plugin_source_with_command("/tmp/limux-cli").expect("plugin source");

        assert!(source.contains("const LIMUX_COMMAND = \"/tmp/limux-cli\";"));
        assert!(source.contains("process.env.LIMUX_BIN || LIMUX_COMMAND"));
        assert!(!source.contains("process.env.LIMUX_BIN || \"limux\""));
    }

    #[test]
    fn opencode_plugin_removes_only_deleted_sessions() {
        let source = opencode_plugin_source_with_command("/tmp/limux-cli").expect("plugin source");

        assert!(
            source.contains("if (type === \"session.error\") send(\"session-end\", ctx, event);")
        );
        assert!(source.contains("if (type === \"session.deleted\") send(\"cleanup\", ctx, event);"));
        assert!(source.contains("type === \"session.status\""));
        assert!(source.contains("type === \"session.compacted\""));
    }

    #[test]
    fn stop_hook_output_matches_codex_schema_shape() {
        let output = agent_hook_output("stop", &json!({ "session_id": "session-a" }));

        assert_eq!(
            output,
            json!({
                "continue": true,
                "suppressOutput": false
            })
        );
    }

    #[test]
    fn hermes_lifecycle_events_map_to_human_notification_types() {
        assert_eq!(
            canonical_agent_hook_display_event("pre_approval_request"),
            AgentHookDisplayEvent::Notification
        );
        assert_eq!(
            canonical_agent_hook_display_event("post_llm_call"),
            AgentHookDisplayEvent::Stop
        );
        assert_eq!(
            canonical_agent_hook_display_event("on_session_start"),
            AgentHookDisplayEvent::SessionStart
        );
        assert_eq!(
            canonical_agent_hook_display_event("pre_tool_call"),
            AgentHookDisplayEvent::ToolUse
        );
    }

    #[test]
    fn hermes_session_start_hook_output_uses_canonical_schema() {
        let output = agent_hook_output(
            "on_session_start",
            &json!({ "additionalContext": "Hermes session restore tracking active." }),
        );

        assert_eq!(
            output,
            json!({
                "continue": true,
                "suppressOutput": false,
                "hookSpecificOutput": {
                    "hookEventName": "SessionStart",
                    "additionalContext": "Hermes session restore tracking active."
                }
            })
        );
    }

    #[test]
    fn session_start_hook_output_uses_camel_case_specific_output() {
        let output = agent_hook_output(
            "session-start",
            &json!({ "additionalContext": "Limux session restore tracking active." }),
        );

        assert_eq!(
            output,
            json!({
                "continue": true,
                "suppressOutput": false,
                "hookSpecificOutput": {
                    "hookEventName": "SessionStart",
                    "additionalContext": "Limux session restore tracking active."
                }
            })
        );
    }

    #[test]
    fn hook_notify_debug_details_include_resolved_socket_path() {
        let details = agent_hook_notify_debug_details(
            Path::new("/tmp/resolved.sock"),
            Some("workspace-a"),
            None,
        );

        assert_eq!(details["workspace"], "workspace-a");
        assert_eq!(details["resolved_socket"], "/tmp/resolved.sock");
    }

    #[test]
    fn claude_hook_install_writes_required_matcher() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("settings.json");

        install_json_hooks(
            &path,
            agent_hooks::AgentKind::Claude,
            &[("SessionStart", "session-start", None)],
        )
        .expect("install hooks");

        let root: Value =
            serde_json::from_slice(&fs::read(&path).expect("read settings")).expect("json");
        let entry = &root["hooks"]["SessionStart"][0];
        assert_eq!(entry["matcher"], "*");
        assert_eq!(entry["hooks"][0]["timeout"], 2);
        assert_eq!(entry["hooks"][0]["statusMessage"], "limux session start");
        assert!(entry["hooks"][0]["command"]
            .as_str()
            .expect("command")
            .contains("hooks claude session-start"));
    }

    #[test]
    fn claude_hook_install_writes_event_specific_status_messages() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("settings.json");

        install_json_hooks(
            &path,
            agent_hooks::AgentKind::Claude,
            &[
                ("Notification", "stop", None),
                ("Stop", "stop", None),
                ("SessionEnd", "session-end", None),
            ],
        )
        .expect("install hooks");

        let root: Value =
            serde_json::from_slice(&fs::read(&path).expect("read settings")).expect("json");
        assert_eq!(
            root["hooks"]["Notification"][0]["hooks"][0]["statusMessage"],
            "limux notify"
        );
        assert_eq!(
            root["hooks"]["Stop"][0]["hooks"][0]["statusMessage"],
            "limux stop hook"
        );
        assert_eq!(
            root["hooks"]["SessionEnd"][0]["hooks"][0]["statusMessage"],
            "limux session end"
        );
    }

    #[tokio::test]
    async fn agent_hook_notification_budget_returns_fast_when_socket_hangs() {
        let dir = tempfile::tempdir().expect("tempdir");
        let socket = dir.path().join("limux.sock");
        let listener = tokio::net::UnixListener::bind(&socket).expect("bind listener");
        let server = tokio::spawn(async move {
            let (_stream, _) = listener.accept().await.expect("accept client");
            tokio::time::sleep(Duration::from_secs(5)).await;
        });

        let mut client = Client::new(socket);
        let mut params = Map::new();
        params.insert("title".to_string(), Value::String("hook".to_string()));
        let started = Instant::now();

        let outcome = send_agent_hook_notification_with_budget(&mut client, None, params).await;

        assert_eq!(outcome, AgentHookNotifyOutcome::Timeout);
        assert!(
            started.elapsed() < Duration::from_secs(2),
            "hook notification waited too long: {:?}",
            started.elapsed()
        );
        server.abort();
    }

    #[test]
    fn codex_hook_install_keeps_codex_schema_without_matcher() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("hooks.json");

        install_json_hooks(
            &path,
            agent_hooks::AgentKind::Codex,
            &[("SessionStart", "session-start", None)],
        )
        .expect("install hooks");

        let root: Value =
            serde_json::from_slice(&fs::read(&path).expect("read hooks")).expect("json");
        let entry = &root["hooks"]["SessionStart"][0];
        assert!(entry.get("matcher").is_none());
        assert_eq!(entry["hooks"][0]["timeout"], 5000);
        assert!(entry["hooks"][0]["command"]
            .as_str()
            .expect("command")
            .contains("hooks codex session-start"));
    }

    #[test]
    fn environ_parser_reads_requested_limux_value() {
        let environ = b"PATH=/bin\0LIMUX_WORKSPACE_ID=ws-1\0LIMUX_SURFACE_ID=7:tab-a\0";

        assert_eq!(
            env_value_from_environ(environ, "LIMUX_WORKSPACE_ID").as_deref(),
            Some("ws-1")
        );
        assert_eq!(
            env_value_from_environ(environ, "LIMUX_SURFACE_ID").as_deref(),
            Some("7:tab-a")
        );
        assert_eq!(env_value_from_environ(environ, "LIMUX_PANE_ID"), None);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn proc_stat_parser_handles_process_names_with_spaces() {
        let stat = "1234 (claude hook sh) S 987 1 1 0 -1 4194560";

        assert_eq!(parse_proc_stat_parent_pid(stat), Some(987));
    }

    #[test]
    fn hook_session_id_falls_back_to_transcript_stem() {
        let payload = json!({
            "transcript_path": "/home/amwill/.claude/projects/-home-amwill-Applications-limux/268746f1-5a8f-471c-85db-dc50649c2f9c.jsonl"
        });

        assert_eq!(
            hook_session_id(&payload).as_deref(),
            Some("268746f1-5a8f-471c-85db-dc50649c2f9c")
        );
    }

    #[test]
    fn hook_session_id_prefers_explicit_session_id() {
        let payload = json!({
            "session_id": "explicit-session",
            "transcript_path": "/tmp/transcript-session.jsonl"
        });

        assert_eq!(
            hook_session_id(&payload).as_deref(),
            Some("explicit-session")
        );
    }

    #[test]
    fn hook_session_id_reads_nested_hermes_metadata() {
        let payload = json!({
            "extra": {
                "session_id": "hermes-session",
                "cwd": "/tmp/project"
            }
        });

        assert_eq!(hook_session_id(&payload).as_deref(), Some("hermes-session"));
        assert_eq!(hook_str(&payload, &["cwd"]), Some("/tmp/project"));
    }
}

#[cfg(test)]
mod agent_team_tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};
    use tokio::net::UnixListener;

    #[derive(Clone, Debug)]
    struct RecordedRequest {
        method: String,
        params: Value,
        protocol_exists_at_request: bool,
        roster_exists_at_request: bool,
        ledger_exists_at_request: bool,
    }

    #[derive(Default)]
    struct FakeAgentTeamServerOptions {
        fail_bootstrap_surface: Option<String>,
    }

    #[test]
    fn agent_launch_known() {
        for agent in [
            "codex",
            "claude",
            "claude-code",
            "opencode",
            "gemini",
            "gemini-cli",
            "hermes",
            "hermes-agent",
        ] {
            assert!(
                agent_launch_command_for_mode(agent, AgentLaunchMode::Direct).is_some(),
                "expected '{agent}' to be a known agent"
            );
        }
    }

    #[test]
    fn hcom_launch_mode_uses_hermes_run_here_command() {
        assert_eq!(
            agent_launch_command_for_mode("hermes", AgentLaunchMode::Hcom),
            Some(("hermes", "hcom hermes --run-here".to_string()))
        );
    }

    #[test]
    fn agent_launch_unknown_returns_none() {
        assert!(agent_launch_command_for_mode("nonsense-cli", AgentLaunchMode::Direct).is_none());
    }

    #[tokio::test]
    async fn agent_team_dry_run_hcom_launch_mode_uses_run_here_commands() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = tmp.path();
        let args = vec![
            "--dry-run".to_string(),
            "--cwd".to_string(),
            cwd.to_string_lossy().to_string(),
            "--agents".to_string(),
            "codex,claude".to_string(),
            "--launch-mode".to_string(),
            "hcom".to_string(),
        ];
        let mut client = Client::new(cwd.join("unused.sock"));

        let payload = run_agent_team(&mut client, &args)
            .await
            .expect("dry run should not contact host");

        assert_eq!(payload["launch_mode"], "hcom");
        assert_eq!(
            payload["peers"][0]["launch_command"],
            "hcom codex --run-here"
        );
        assert_eq!(
            payload["peers"][1]["launch_command"],
            "hcom claude --run-here"
        );

        let md = std::fs::read_to_string(cwd.join("LIMUX_AGENTS.md")).expect("read protocol");
        assert!(md.contains("| `codex` | `<dry-run-pane-0>` | `<dry-run-surface-codex>` | `hcom codex --run-here` |"));
        assert!(md.contains("| `claude` | `<dry-run-pane-1>` | `<dry-run-surface-claude>` | `hcom claude --run-here` |"));
    }

    #[tokio::test]
    async fn agent_team_help_is_side_effect_free() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = tmp.path();
        let args = vec![
            "--help".to_string(),
            "--cwd".to_string(),
            cwd.to_string_lossy().to_string(),
        ];
        let mut client = Client::new(cwd.join("missing.sock"));

        let payload = run_agent_team(&mut client, &args)
            .await
            .expect("help should not contact host");

        let help = payload["help"].as_str().expect("help text");
        assert!(help.contains("Usage: limux agent-team"));
        assert!(!cwd.join("LIMUX_AGENTS.md").exists());
        assert!(!cwd.join("LIMUX_TEAM_ROSTER.md").exists());
        assert!(!cwd.join("LIMUX_REVIEW_LEDGER.md").exists());
    }

    #[tokio::test]
    async fn agent_team_hcom_launch_mode_launches_run_here_commands() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = tmp.path();
        let socket = cwd.join("limux.sock");
        let protocol_path = cwd.join("LIMUX_AGENTS.md");
        let (requests, server) =
            spawn_agent_team_fake_server(socket.clone(), protocol_path.clone()).await;
        let mut client = Client::new(socket);
        let args = vec![
            "--agents".to_string(),
            "codex,claude".to_string(),
            "--cwd".to_string(),
            cwd.to_string_lossy().to_string(),
            "--launch-mode".to_string(),
            "hcom".to_string(),
        ];

        let payload = run_agent_team(&mut client, &args)
            .await
            .expect("agent-team should complete against fake host");
        server.abort();

        assert_eq!(payload["launch_mode"], "hcom");
        let requests = requests.lock().expect("lock requests").clone();
        let pane_creates: Vec<_> = requests
            .iter()
            .filter(|request| request.method == "pane.create")
            .collect();
        assert_eq!(pane_creates.len(), 2);
        assert_eq!(
            pane_creates[0].params["command"].as_str(),
            Some("hcom codex --run-here")
        );
        assert_eq!(
            pane_creates[1].params["command"].as_str(),
            Some("hcom claude --run-here")
        );
    }

    #[tokio::test]
    async fn agent_team_rejects_unknown_launch_mode() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = tmp.path();
        let args = vec![
            "--dry-run".to_string(),
            "--cwd".to_string(),
            cwd.to_string_lossy().to_string(),
            "--launch-mode".to_string(),
            "spaceship".to_string(),
        ];
        let mut client = Client::new(cwd.join("unused.sock"));

        let err = run_agent_team(&mut client, &args)
            .await
            .expect_err("unknown launch mode should fail");
        let msg = format!("{err:#}");
        assert!(msg.contains("--launch-mode"), "unexpected error: {msg}");
        assert!(msg.contains("direct"), "unexpected error: {msg}");
        assert!(msg.contains("hcom"), "unexpected error: {msg}");
    }

    async fn spawn_agent_team_fake_server(
        socket: PathBuf,
        protocol_path: PathBuf,
    ) -> (
        Arc<Mutex<Vec<RecordedRequest>>>,
        tokio::task::JoinHandle<()>,
    ) {
        spawn_agent_team_fake_server_with_options(
            socket,
            protocol_path,
            FakeAgentTeamServerOptions::default(),
        )
        .await
    }

    async fn spawn_agent_team_fake_server_with_options(
        socket: PathBuf,
        protocol_path: PathBuf,
        options: FakeAgentTeamServerOptions,
    ) -> (
        Arc<Mutex<Vec<RecordedRequest>>>,
        tokio::task::JoinHandle<()>,
    ) {
        let _ = std::fs::remove_file(&socket);
        let listener = UnixListener::bind(&socket).expect("bind fake limux socket");
        let roster_path = protocol_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(AGENT_TEAM_DEFAULT_ROSTER_FILE);
        let ledger_path = protocol_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(AGENT_TEAM_DEFAULT_LEDGER_FILE);
        let requests = Arc::new(Mutex::new(Vec::new()));
        let pane_count = Arc::new(AtomicUsize::new(0));
        let server_requests = Arc::clone(&requests);
        let server_pane_count = Arc::clone(&pane_count);
        let fail_bootstrap_surface = options.fail_bootstrap_surface;

        let handle = tokio::spawn(async move {
            loop {
                let Ok((stream, _addr)) = listener.accept().await else {
                    break;
                };
                let requests = Arc::clone(&server_requests);
                let pane_count = Arc::clone(&server_pane_count);
                let protocol_path = protocol_path.clone();
                let roster_path = roster_path.clone();
                let ledger_path = ledger_path.clone();
                let fail_bootstrap_surface = fail_bootstrap_surface.clone();

                tokio::spawn(async move {
                    let (reader_half, mut writer_half) = stream.into_split();
                    let mut reader = BufReader::new(reader_half);
                    let mut line = String::new();
                    if reader.read_line(&mut line).await.expect("read request") == 0 {
                        return;
                    }

                    let request: V2Request =
                        serde_json::from_str(line.trim()).expect("parse fake request");
                    let method = request.method.clone();
                    let params = request.params.clone();
                    let protocol_exists_at_request = protocol_path.exists();
                    let roster_exists_at_request = roster_path.exists();
                    let ledger_exists_at_request = ledger_path.exists();
                    requests
                        .lock()
                        .expect("lock requests")
                        .push(RecordedRequest {
                            method: method.clone(),
                            params: params.clone(),
                            protocol_exists_at_request,
                            roster_exists_at_request,
                            ledger_exists_at_request,
                        });

                    let response = match method.as_str() {
                        "workspace.current" => V2Response::success(
                            request.id.clone(),
                            json!({ "workspace_id": "workspace:team" }),
                        ),
                        "surface.list" => V2Response::success(
                            request.id.clone(),
                            json!({
                                "surfaces": [{
                                    "pane_id": "pane:1",
                                    "surface_id": "surface:1:orchestrator",
                                    "focused": true,
                                    "title": "orchestrator"
                                }]
                            }),
                        ),
                        "workspace.list" => V2Response::success(
                            request.id.clone(),
                            json!({
                                "workspaces": [{
                                    "workspace_id": "workspace:team",
                                    "id": "workspace:team",
                                    "name": "limux",
                                    "title": "limux"
                                }]
                            }),
                        ),
                        "pane.create" => {
                            let index = pane_count.fetch_add(1, Ordering::SeqCst);
                            let (agent, pane_id, surface_id) = if index == 0 {
                                ("codex", "pane:2", "surface:2:codex")
                            } else {
                                ("claude", "pane:3", "surface:3:claude")
                            };
                            V2Response::success(
                                request.id.clone(),
                                json!({
                                    "pane_id": pane_id,
                                    "surface_id": surface_id,
                                    "surface_ref": surface_id,
                                    "surface_title": agent,
                                    "ok": true
                                }),
                            )
                        }
                        "surface.send_text" => {
                            let surface_id = params
                                .get("surface_id")
                                .and_then(Value::as_str)
                                .unwrap_or("surface:unknown");
                            if fail_bootstrap_surface.as_deref() == Some(surface_id) {
                                V2Response::error(
                                    request.id.clone(),
                                    -32009,
                                    format!("bootstrap rejected for terminal surface {surface_id}"),
                                    None,
                                )
                            } else {
                                V2Response::success(
                                    request.id.clone(),
                                    json!({
                                        "ok": true,
                                        "surface_id": surface_id
                                    }),
                                )
                            }
                        }
                        "surface.send_key" => {
                            let surface_id = params
                                .get("surface_id")
                                .and_then(Value::as_str)
                                .unwrap_or("surface:unknown");
                            V2Response::success(
                                request.id.clone(),
                                json!({
                                    "ok": true,
                                    "surface_id": surface_id
                                }),
                            )
                        }
                        other => panic!("unexpected fake request method {other}: {params:?}"),
                    };
                    let mut payload = serde_json::to_string(&response).expect("encode response");
                    payload.push('\n');
                    writer_half
                        .write_all(payload.as_bytes())
                        .await
                        .expect("write response");
                });
            }
        });

        (requests, handle)
    }

    #[tokio::test]
    async fn agent_team_live_bootstrap_launches_binary_then_sends_prompt() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = tmp.path();
        let socket = cwd.join("limux.sock");
        let protocol_path = cwd.join("LIMUX_AGENTS.md");
        let (requests, server) =
            spawn_agent_team_fake_server(socket.clone(), protocol_path.clone()).await;
        let mut client = Client::new(socket);
        let args = vec![
            "--agents".to_string(),
            "codex,claude".to_string(),
            "--cwd".to_string(),
            cwd.to_string_lossy().to_string(),
        ];

        let payload = run_agent_team(&mut client, &args)
            .await
            .expect("agent-team should complete against fake host");
        server.abort();

        assert_eq!(payload["no_launch"], false);
        assert_eq!(payload["bootstrap"]["enabled"], true);
        assert_eq!(payload["bootstrap"]["status"], "sent");
        assert_eq!(payload["roster"]["status"], "created");
        assert_eq!(payload["ledger"]["status"], "created");

        let requests = requests.lock().expect("lock requests").clone();
        let pane_creates: Vec<_> = requests
            .iter()
            .filter(|request| request.method == "pane.create")
            .collect();
        assert_eq!(pane_creates.len(), 2, "expected two pane.create calls");
        assert_eq!(pane_creates[0].params["command"].as_str(), Some("codex"));
        assert_eq!(pane_creates[1].params["command"].as_str(), Some("claude"));
        assert!(
            pane_creates.iter().all(|request| !request.params["command"]
                .as_str()
                .unwrap_or("")
                .contains("LIMUX_AGENTS.md")),
            "pane.create command must remain only the launch binary"
        );

        let bootstrap_sends: Vec<_> = requests
            .iter()
            .filter(|request| request.method == "surface.send_text")
            .collect();
        assert_eq!(
            bootstrap_sends.len(),
            2,
            "expected one post-launch bootstrap prompt per peer"
        );
        let bootstrap_submits: Vec<_> = requests
            .iter()
            .filter(|request| request.method == "surface.send_key")
            .collect();
        assert_eq!(
            bootstrap_submits.len(),
            2,
            "expected one explicit Enter submit per bootstrap prompt"
        );
        assert!(bootstrap_submits.iter().all(|request| {
            request.params["key"].as_str() == Some("enter")
                && request.protocol_exists_at_request
                && request.roster_exists_at_request
                && request.ledger_exists_at_request
        }));
        assert!(
            bootstrap_sends
                .iter()
                .all(|request| request.protocol_exists_at_request
                    && request.roster_exists_at_request
                    && request.ledger_exists_at_request),
            "coordination files should exist before bootstrap prompts are sent"
        );
        for request in bootstrap_sends {
            let text = request.params["text"].as_str().expect("bootstrap text");
            validate_agent_team_bootstrap_prompt(text)
                .expect("bootstrap prompt should pass bootstrap text policy");
            assert!(text.contains("Read the generated runtime protocol file"));
            assert!(text.contains(protocol_path.to_string_lossy().as_ref()));
            assert!(text.contains(AGENT_TEAM_DEFAULT_ROSTER_FILE));
            assert!(text.contains(AGENT_TEAM_DEFAULT_LEDGER_FILE));
            assert!(text.contains("authoritative instruction sources"));
            assert!(
                !text.contains('\n'),
                "bootstrap prompt text should be submitted only by explicit Enter"
            );
        }
    }

    #[test]
    fn agent_team_bootstrap_prompt_is_single_line_and_escapes_dynamic_values() {
        let protocol_path = PathBuf::from("/tmp/limux\nteam/\u{202e}\u{200b}LIMUX_AGENTS.md");
        let roster_path = PathBuf::from("/tmp/limux\nteam/LIMUX_TEAM_ROSTER.md");
        let ledger_path = PathBuf::from("/tmp/limux\nteam/LIMUX_REVIEW_LEDGER.md");

        let prompt =
            build_agent_team_bootstrap_prompt("codex", &protocol_path, &roster_path, &ledger_path)
                .expect("prompt should be valid");

        validate_agent_team_bootstrap_prompt(&prompt).expect("prompt policy should pass");
        assert!(prompt.contains("LIMUX_AGENTS.md"));
        assert!(prompt.contains("LIMUX_TEAM_ROSTER.md"));
        assert!(prompt.contains("LIMUX_REVIEW_LEDGER.md"));
        assert!(prompt.contains("authoritative instruction sources"));
        assert!(!prompt.contains('\r'));
        assert!(!prompt.contains('\t'));
        assert!(
            !prompt.contains('\n'),
            "bootstrap prompt should avoid LF because explicit Enter submits the line"
        );
        assert!(!prompt.contains('\u{202e}'));
        assert!(!prompt.contains('\u{200b}'));
        assert!(prompt.contains("\\n"));
        assert!(prompt.contains("\\u{202e}"));
        assert!(prompt.contains("\\u{200b}"));
    }

    #[tokio::test]
    async fn agent_team_no_bootstrap_launches_panes_without_prompt_send() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = tmp.path();
        let socket = cwd.join("limux.sock");
        let protocol_path = cwd.join("LIMUX_AGENTS.md");
        let (requests, server) = spawn_agent_team_fake_server(socket.clone(), protocol_path).await;
        let mut client = Client::new(socket);
        let args = vec![
            "--agents".to_string(),
            "codex,claude".to_string(),
            "--cwd".to_string(),
            cwd.to_string_lossy().to_string(),
            "--no-bootstrap".to_string(),
        ];

        let payload = run_agent_team(&mut client, &args)
            .await
            .expect("agent-team should complete with bootstrap disabled");
        server.abort();

        assert_eq!(payload["bootstrap"]["enabled"], false);
        assert_eq!(payload["bootstrap"]["status"], "skipped");
        let requests = requests.lock().expect("lock requests").clone();
        assert_eq!(
            requests
                .iter()
                .filter(|request| request.method == "pane.create")
                .count(),
            2
        );
        assert!(
            requests.iter().all(|request| !matches!(
                request.method.as_str(),
                "surface.send_text" | "surface.send_key"
            )),
            "--no-bootstrap should skip only post-launch prompt injection"
        );
    }

    #[tokio::test]
    async fn agent_team_no_launch_implies_no_bootstrap_prompt_send() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = tmp.path();
        let socket = cwd.join("limux.sock");
        let protocol_path = cwd.join("LIMUX_AGENTS.md");
        let (requests, server) = spawn_agent_team_fake_server(socket.clone(), protocol_path).await;
        let mut client = Client::new(socket);
        let args = vec![
            "--agents".to_string(),
            "codex,claude".to_string(),
            "--cwd".to_string(),
            cwd.to_string_lossy().to_string(),
            "--no-launch".to_string(),
        ];

        let payload = run_agent_team(&mut client, &args)
            .await
            .expect("agent-team should complete with launch disabled");
        server.abort();

        assert_eq!(payload["no_launch"], true);
        assert_eq!(payload["bootstrap"]["enabled"], false);
        let requests = requests.lock().expect("lock requests").clone();
        let pane_creates: Vec<_> = requests
            .iter()
            .filter(|request| request.method == "pane.create")
            .collect();
        assert_eq!(pane_creates.len(), 2);
        assert!(
            pane_creates
                .iter()
                .all(|request| request.params.get("command").is_none()),
            "--no-launch should not type launch commands"
        );
        assert!(
            requests.iter().all(|request| !matches!(
                request.method.as_str(),
                "surface.send_text" | "surface.send_key"
            )),
            "--no-launch implies no bootstrap"
        );
    }

    #[tokio::test]
    async fn agent_team_bootstrap_send_failure_reports_peer_and_surface() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = tmp.path();
        let socket = cwd.join("limux.sock");
        let protocol_path = cwd.join("LIMUX_AGENTS.md");
        let (requests, server) = spawn_agent_team_fake_server_with_options(
            socket.clone(),
            protocol_path,
            FakeAgentTeamServerOptions {
                fail_bootstrap_surface: Some("surface:3:claude".to_string()),
            },
        )
        .await;
        let mut client = Client::new(socket);
        let args = vec![
            "--agents".to_string(),
            "codex,claude".to_string(),
            "--cwd".to_string(),
            cwd.to_string_lossy().to_string(),
        ];

        let err = run_agent_team(&mut client, &args)
            .await
            .expect_err("failed bootstrap send should fail the command");
        server.abort();

        let msg = format!("{err:#}");
        assert!(msg.contains("claude"), "unexpected error: {msg}");
        assert!(msg.contains("surface:3:claude"), "unexpected error: {msg}");
        let requests = requests.lock().expect("lock requests").clone();
        assert!(requests
            .iter()
            .any(|request| request.method == "surface.send_text"
                && request.params["surface_id"] == "surface:3:claude"));
    }

    #[test]
    fn agents_md_contains_protocol_and_peers() {
        let peers = vec![
            (
                "codex".to_string(),
                "10".to_string(),
                "10:tab-a".to_string(),
                "codex".to_string(),
            ),
            (
                "claude".to_string(),
                "11".to_string(),
                "11:tab-a".to_string(),
                "claude".to_string(),
            ),
        ];
        let md = build_agents_md(
            &peers,
            "/tmp/team",
            "active-ws",
            "ws-uuid-123",
            "9:terminal-orch",
            AgentTeamCoordinationFiles {
                roster_path: Path::new("/tmp/team/LIMUX_TEAM_ROSTER.md"),
                ledger_path: Path::new("/tmp/team/LIMUX_REVIEW_LEDGER.md"),
            },
            &[],
        );

        // Header & generation marker
        assert!(md.contains("LIMUX_AGENTS.md — agent-to-agent message protocol"));
        assert!(md.contains("Generated by `limux agent-team`"));

        // Team workspace block
        assert!(md.contains("Workspace name: `active-ws`"));
        assert!(md.contains("Workspace ID: `ws-uuid-123`"));
        assert!(md.contains("Orchestrator surface: `9:terminal-orch`"));
        assert!(md.contains("Shared cwd: `/tmp/team`"));

        // Peer table rows (Agent | Pane | Surface | Launch)
        assert!(md.contains("| `codex` | `10` | `10:tab-a` | `codex` |"));
        assert!(md.contains("| `claude` | `11` | `11:tab-a` | `claude` |"));

        // Protocol envelope spec uses --surface, not --workspace
        assert!(md.contains("<agent-msg from=\"codex\" to=\"claude\""));
        assert!(md.contains("limux send --surface"));
        assert!(!md.contains("limux send --workspace"));
        assert!(md.contains("reply-to"));

        // Notify + env contract
        assert!(md.contains("limux notify"));
        assert!(md.contains("LIMUX_WORKSPACE_ID"));
        assert!(md.contains("LIMUX_SURFACE_ID"));
        assert!(md.contains("limux new-pane --direction right --command 'bash'"));
        assert!(md.contains("Live GTK self-spawn currently supports terminal"));
    }

    #[tokio::test]
    async fn agent_team_dry_run_preserves_existing_agents_md() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = tmp.path();
        let existing_agents_md = cwd.join("AGENTS.md");
        std::fs::write(&existing_agents_md, "repo instructions\n").expect("write AGENTS.md");

        let args = vec![
            "--dry-run".to_string(),
            "--cwd".to_string(),
            cwd.to_string_lossy().to_string(),
        ];
        let mut client = Client::new(cwd.join("unused.sock"));

        let payload = run_agent_team(&mut client, &args)
            .await
            .expect("dry run should not contact host");

        assert_eq!(
            std::fs::read_to_string(&existing_agents_md).expect("read AGENTS.md"),
            "repo instructions\n"
        );

        let protocol_path = cwd.join("LIMUX_AGENTS.md");
        assert!(
            protocol_path.exists(),
            "expected generated protocol at {}",
            protocol_path.display()
        );
        assert_eq!(
            payload.get("agents_md").and_then(Value::as_str),
            Some(protocol_path.to_string_lossy().as_ref())
        );
    }

    #[tokio::test]
    async fn agent_team_dry_run_uses_sidecar_protocol_path_by_default() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = tmp.path();
        let args = vec![
            "--dry-run".to_string(),
            "--cwd".to_string(),
            cwd.to_string_lossy().to_string(),
        ];
        let mut client = Client::new(cwd.join("unused.sock"));

        let payload = run_agent_team(&mut client, &args)
            .await
            .expect("dry run should not contact host");

        assert!(!cwd.join("AGENTS.md").exists());

        let protocol_path = cwd.join("LIMUX_AGENTS.md");
        assert!(
            protocol_path.exists(),
            "expected generated protocol at {}",
            protocol_path.display()
        );
        assert_eq!(
            payload.get("protocol_path").and_then(Value::as_str),
            Some(protocol_path.to_string_lossy().as_ref())
        );
    }

    #[tokio::test]
    async fn agent_team_dry_run_honors_relative_protocol_path() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = tmp.path();
        let args = vec![
            "--dry-run".to_string(),
            "--cwd".to_string(),
            cwd.to_string_lossy().to_string(),
            "--protocol-path".to_string(),
            ".limux/team-protocol.md".to_string(),
        ];
        let mut client = Client::new(cwd.join("unused.sock"));

        let payload = run_agent_team(&mut client, &args)
            .await
            .expect("dry run should not contact host");

        let protocol_path = cwd.join(".limux/team-protocol.md");
        assert!(
            protocol_path.exists(),
            "expected generated protocol at {}",
            protocol_path.display()
        );
        assert_eq!(
            payload.get("agents_md").and_then(Value::as_str),
            Some(protocol_path.to_string_lossy().as_ref())
        );
        assert_eq!(
            payload.get("protocol_path").and_then(Value::as_str),
            Some(protocol_path.to_string_lossy().as_ref())
        );
    }

    #[tokio::test]
    async fn agent_team_generated_protocol_includes_marker_and_local_policy_pointer() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = tmp.path();
        let args = vec![
            "--dry-run".to_string(),
            "--cwd".to_string(),
            cwd.to_string_lossy().to_string(),
        ];
        let mut client = Client::new(cwd.join("unused.sock"));

        run_agent_team(&mut client, &args)
            .await
            .expect("dry run should not contact host");

        let md = std::fs::read_to_string(cwd.join("LIMUX_AGENTS.md")).expect("read protocol");
        assert!(
            md.starts_with("<!-- limux-agent-team-protocol generated:v1 -->\n# LIMUX_AGENTS.md"),
            "generated protocol should start with a stable generated marker"
        );
        assert!(md.contains("LIMUX_AGENTS.local.md"));
        assert!(md.contains("Limux does not create or overwrite it"));
        assert!(md.contains("## Durable Coordination Files"));
        assert!(md.contains(AGENT_TEAM_DEFAULT_ROSTER_FILE));
        assert!(md.contains(AGENT_TEAM_DEFAULT_LEDGER_FILE));
    }

    #[tokio::test]
    async fn agent_team_dry_run_creates_roster_and_review_ledger_without_host() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = tmp.path();
        let args = vec![
            "--dry-run".to_string(),
            "--cwd".to_string(),
            cwd.to_string_lossy().to_string(),
        ];
        let mut client = Client::new(cwd.join("unused.sock"));

        let payload = run_agent_team(&mut client, &args)
            .await
            .expect("dry run should not contact host");

        let roster_path = cwd.join(AGENT_TEAM_DEFAULT_ROSTER_FILE);
        let ledger_path = cwd.join(AGENT_TEAM_DEFAULT_LEDGER_FILE);
        assert_eq!(
            payload.get("roster_path").and_then(Value::as_str),
            Some(roster_path.to_string_lossy().as_ref())
        );
        assert_eq!(
            payload.get("ledger_path").and_then(Value::as_str),
            Some(ledger_path.to_string_lossy().as_ref())
        );
        assert_eq!(payload["roster"]["status"], "created");
        assert_eq!(payload["ledger"]["status"], "created");

        let roster = std::fs::read_to_string(&roster_path).expect("read roster");
        assert!(roster.starts_with(AGENT_TEAM_ROSTER_MARKER));
        assert!(roster.contains("# Limux Team Roster"));
        assert!(roster.contains("| `current` | peer | `codex`"));
        assert!(roster.contains("| `current` | peer | `claude`"));
        assert!(roster.contains(AGENT_TEAM_DEFAULT_LEDGER_FILE));
        assert!(roster.contains("Use the current generated runtime protocol"));
        assert!(!roster.contains("<dry-run-orchestrator>"));

        let ledger = std::fs::read_to_string(&ledger_path).expect("read ledger");
        assert!(ledger.starts_with(AGENT_TEAM_LEDGER_MARKER));
        assert!(ledger.contains("# Limux Review And Consensus Ledger"));
        assert!(ledger.contains("## Entry Template"));
    }

    #[tokio::test]
    async fn agent_team_dry_run_preserves_existing_roster_and_review_ledger() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = tmp.path();
        let roster_path = cwd.join(AGENT_TEAM_DEFAULT_ROSTER_FILE);
        let ledger_path = cwd.join(AGENT_TEAM_DEFAULT_LEDGER_FILE);
        std::fs::write(&roster_path, "manual roster\n").expect("write roster");
        std::fs::write(&ledger_path, "manual ledger\n").expect("write ledger");
        let args = vec![
            "--dry-run".to_string(),
            "--cwd".to_string(),
            cwd.to_string_lossy().to_string(),
        ];
        let mut client = Client::new(cwd.join("unused.sock"));

        let payload = run_agent_team(&mut client, &args)
            .await
            .expect("dry run should preserve durable files");

        assert_eq!(payload["roster"]["status"], "existing");
        assert_eq!(payload["ledger"]["status"], "existing");
        assert_eq!(
            std::fs::read_to_string(&roster_path).expect("read roster"),
            "manual roster\n"
        );
        assert_eq!(
            std::fs::read_to_string(&ledger_path).expect("read ledger"),
            "manual ledger\n"
        );
    }

    #[tokio::test]
    async fn agent_team_dry_run_force_overwrites_existing_roster_file_only() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = tmp.path();
        let roster_path = cwd.join(AGENT_TEAM_DEFAULT_ROSTER_FILE);
        let ledger_path = cwd.join(AGENT_TEAM_DEFAULT_LEDGER_FILE);
        std::fs::write(
            &roster_path,
            format!("{AGENT_TEAM_ROSTER_MARKER}\nold generated roster\n"),
        )
        .expect("write roster");
        std::fs::write(&ledger_path, "manual ledger\n").expect("write ledger");
        let args = vec![
            "--dry-run".to_string(),
            "--cwd".to_string(),
            cwd.to_string_lossy().to_string(),
            "--force-roster-overwrite".to_string(),
        ];
        let mut client = Client::new(cwd.join("unused.sock"));

        let payload = run_agent_team(&mut client, &args)
            .await
            .expect("force should reseed roster without touching ledger");

        assert_eq!(payload["roster"]["status"], "replaced");
        assert_eq!(payload["ledger"]["status"], "existing");
        let roster = std::fs::read_to_string(&roster_path).expect("read roster");
        assert!(roster.contains(AGENT_TEAM_ROSTER_MARKER));
        assert!(!roster.contains("old generated roster"));
        assert_eq!(
            std::fs::read_to_string(&ledger_path).expect("read ledger"),
            "manual ledger\n"
        );
    }

    #[tokio::test]
    async fn agent_team_dry_run_force_refuses_unmarked_roster_file() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = tmp.path();
        let roster_path = cwd.join(AGENT_TEAM_DEFAULT_ROSTER_FILE);
        std::fs::write(&roster_path, "manual roster\n").expect("write roster");
        let args = vec![
            "--dry-run".to_string(),
            "--cwd".to_string(),
            cwd.to_string_lossy().to_string(),
            "--force-roster-overwrite".to_string(),
        ];
        let mut client = Client::new(cwd.join("unused.sock"));

        let error = run_agent_team(&mut client, &args)
            .await
            .expect_err("force should not replace unmarked roster files");
        assert!(
            error.to_string().contains("unmarked team roster"),
            "unexpected error: {error:#}"
        );
        assert_eq!(
            std::fs::read_to_string(&roster_path).expect("read roster"),
            "manual roster\n"
        );
    }

    #[tokio::test]
    async fn agent_team_dry_run_refuses_overlapping_output_paths() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = tmp.path();
        let shared_path = cwd.join("shared.md");
        let args = vec![
            "--dry-run".to_string(),
            "--cwd".to_string(),
            cwd.to_string_lossy().to_string(),
            "--roster-path".to_string(),
            shared_path.to_string_lossy().to_string(),
            "--ledger-path".to_string(),
            shared_path.to_string_lossy().to_string(),
        ];
        let mut client = Client::new(cwd.join("unused.sock"));

        let error = run_agent_team(&mut client, &args)
            .await
            .expect_err("roster and ledger paths must not overlap");
        assert!(
            error.to_string().contains("output paths must be distinct"),
            "unexpected error: {error:#}"
        );
        assert!(!shared_path.exists());
    }

    #[tokio::test]
    async fn agent_team_dry_run_refuses_lexically_overlapping_output_paths() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = tmp.path();
        let args = vec![
            "--dry-run".to_string(),
            "--cwd".to_string(),
            cwd.to_string_lossy().to_string(),
            "--roster-path".to_string(),
            format!("./{AGENT_TEAM_DEFAULT_PROTOCOL_FILE}"),
        ];
        let mut client = Client::new(cwd.join("unused.sock"));

        let error = run_agent_team(&mut client, &args)
            .await
            .expect_err("protocol and roster paths must not overlap through ./ alias");
        assert!(
            error.to_string().contains("output paths must be distinct"),
            "unexpected error: {error:#}"
        );
        assert!(!cwd.join(AGENT_TEAM_DEFAULT_PROTOCOL_FILE).exists());
    }

    #[tokio::test]
    async fn agent_team_generated_protocol_lists_instruction_sources_without_copying() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = tmp.path();
        std::fs::write(
            cwd.join("AGENTS.md"),
            "AGENT_SOURCE_SECRET: repo instructions\n",
        )
        .expect("write AGENTS.md");
        std::fs::write(
            cwd.join("CLAUDE.md"),
            "CLAUDE_SOURCE_SECRET: claude instructions\n",
        )
        .expect("write CLAUDE.md");

        let args = vec![
            "--dry-run".to_string(),
            "--cwd".to_string(),
            cwd.to_string_lossy().to_string(),
        ];
        let mut client = Client::new(cwd.join("unused.sock"));

        run_agent_team(&mut client, &args)
            .await
            .expect("dry run should not contact host");

        let md = std::fs::read_to_string(cwd.join("LIMUX_AGENTS.md")).expect("read protocol");
        assert!(md.contains("## Instruction Sources"));
        assert!(md.contains("Project instruction files remain authoritative"));
        assert!(md.contains("| `AGENTS.md` | `./AGENTS.md` | `regular file` |"));
        assert!(md.contains("| `CLAUDE.md` | `./CLAUDE.md` | `regular file` |"));
        assert!(md.contains("Modified (unix seconds)"));
        assert!(md.contains("fnv1a64:"));
        assert!(!md.contains("AGENT_SOURCE_SECRET"));
        assert!(!md.contains("CLAUDE_SOURCE_SECRET"));
    }

    #[tokio::test]
    async fn agent_team_dry_run_refuses_unmarked_existing_protocol_file() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = tmp.path();
        let protocol_path = cwd.join("LIMUX_AGENTS.md");
        std::fs::write(&protocol_path, "manual protocol\n").expect("write protocol");
        let args = vec![
            "--dry-run".to_string(),
            "--cwd".to_string(),
            cwd.to_string_lossy().to_string(),
        ];
        let mut client = Client::new(cwd.join("unused.sock"));

        let err = run_agent_team(&mut client, &args)
            .await
            .expect_err("dry run should refuse an unmarked protocol file");
        assert!(
            format!("{err:#}").contains("refusing to overwrite existing unmarked protocol file"),
            "unexpected error: {err:#}"
        );
        assert_eq!(
            std::fs::read_to_string(&protocol_path).expect("read protocol"),
            "manual protocol\n"
        );
    }

    #[tokio::test]
    async fn agent_team_dry_run_force_overwrites_unmarked_protocol_file() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = tmp.path();
        let protocol_path = cwd.join("LIMUX_AGENTS.md");
        std::fs::write(&protocol_path, "manual protocol\n").expect("write protocol");
        let args = vec![
            "--dry-run".to_string(),
            "--cwd".to_string(),
            cwd.to_string_lossy().to_string(),
            "--force-protocol-overwrite".to_string(),
        ];
        let mut client = Client::new(cwd.join("unused.sock"));

        run_agent_team(&mut client, &args)
            .await
            .expect("force should allow replacing an unmarked protocol file");

        let md = std::fs::read_to_string(&protocol_path).expect("read protocol");
        assert!(md.contains("<!-- limux-agent-team-protocol generated:v1 -->"));
        assert!(!md.contains("manual protocol"));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn agent_team_dry_run_refuses_protocol_symlink() {
        use std::os::unix::fs::symlink;

        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = tmp.path();
        let target = cwd.join("target.md");
        std::fs::write(&target, "target protocol\n").expect("write target");
        symlink(&target, cwd.join("LIMUX_AGENTS.md")).expect("create protocol symlink");
        let args = vec![
            "--dry-run".to_string(),
            "--cwd".to_string(),
            cwd.to_string_lossy().to_string(),
            "--force-protocol-overwrite".to_string(),
        ];
        let mut client = Client::new(cwd.join("unused.sock"));

        let err = run_agent_team(&mut client, &args)
            .await
            .expect_err("dry run should refuse symlink protocol path");
        assert!(
            format!("{err:#}").contains("refusing to write protocol path because it is a symlink"),
            "unexpected error: {err:#}"
        );
        assert_eq!(
            std::fs::read_to_string(&target).expect("read target"),
            "target protocol\n"
        );
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn agent_team_dry_run_refuses_roster_and_ledger_symlinks() {
        use std::os::unix::fs::symlink;

        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = tmp.path();
        let roster_target = cwd.join("roster-target.md");
        let ledger_target = cwd.join("ledger-target.md");
        std::fs::write(&roster_target, "target roster\n").expect("write roster target");
        std::fs::write(&ledger_target, "target ledger\n").expect("write ledger target");
        symlink(&roster_target, cwd.join(AGENT_TEAM_DEFAULT_ROSTER_FILE))
            .expect("create roster symlink");
        symlink(&ledger_target, cwd.join(AGENT_TEAM_DEFAULT_LEDGER_FILE))
            .expect("create ledger symlink");
        let args = vec![
            "--dry-run".to_string(),
            "--cwd".to_string(),
            cwd.to_string_lossy().to_string(),
            "--force-roster-overwrite".to_string(),
        ];
        let mut client = Client::new(cwd.join("unused.sock"));

        let err = run_agent_team(&mut client, &args)
            .await
            .expect_err("dry run should refuse symlink roster before ledger");
        assert!(
            format!("{err:#}").contains("refusing to use team roster path because it is a symlink"),
            "unexpected error: {err:#}"
        );

        std::fs::remove_file(cwd.join(AGENT_TEAM_DEFAULT_ROSTER_FILE))
            .expect("remove roster symlink");
        let err = run_agent_team(&mut client, &args)
            .await
            .expect_err("dry run should refuse symlink ledger");
        assert!(
            format!("{err:#}")
                .contains("refusing to use review ledger path because it is a symlink"),
            "unexpected error: {err:#}"
        );
        assert_eq!(
            std::fs::read_to_string(&roster_target).expect("read roster target"),
            "target roster\n"
        );
        assert_eq!(
            std::fs::read_to_string(&ledger_target).expect("read ledger target"),
            "target ledger\n"
        );
    }

    #[test]
    fn shell_command_arg_round_trips_launch_metacharacters() {
        for launch in [
            "",
            "--help",
            "codex review spaces",
            "codex \"review spaces\" '$PATH' `whoami`; echo nope",
            "codex $HOME $(whoami) \\slash\ttab & wait | cat < in > out (paren) * ? ~ # !",
            "codex first line\nsecond line with '$PATH' and `whoami`; echo nope",
            "codex carriage\rreturn",
            "codex escape \x1b bell \x07",
        ] {
            let quoted = shell_command_arg(launch);

            assert_bash_word_round_trip(&quoted, launch);
            assert!(
                !quoted.contains('\n'),
                "generated shell argument should not insert literal newlines: {quoted:?}"
            );
        }
    }

    #[test]
    fn shell_command_arg_keeps_newline_launches_single_line() {
        let launch = "codex first line\nsecond line with '$PATH' and `whoami`; echo nope";
        let quoted = shell_command_arg(launch);

        assert_bash_word_round_trip(&quoted, launch);
        assert!(
            !quoted.contains('\n'),
            "generated shell argument should not insert literal newlines: {quoted:?}"
        );
    }

    #[test]
    fn new_pane_shell_command_parses_as_single_argv_without_outer_side_effects() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let capture = tmp.path().join("captured-command");
        let bad_substitution = tmp.path().join("bad-substitution");
        let bad_backtick = tmp.path().join("bad-backtick");
        let bad_semicolon = tmp.path().join("bad-semicolon");
        let launch = format!(
            "codex \"review spaces\" 'single' $HOME $(touch {}) `touch {}` ; touch {} \\\\slash\ttab\nsecond",
            bad_substitution.display(),
            bad_backtick.display(),
            bad_semicolon.display()
        );
        let command = new_pane_shell_command("right", &launch);
        let capture_arg = shell_command_arg(&capture.to_string_lossy());
        let bad_substitution_arg = shell_command_arg(&bad_substitution.to_string_lossy());
        let bad_backtick_arg = shell_command_arg(&bad_backtick.to_string_lossy());
        let bad_semicolon_arg = shell_command_arg(&bad_semicolon.to_string_lossy());

        let script = format!(
            r#"
set -euo pipefail
limux() {{
  [[ "$#" -eq 5 ]] || {{ printf 'argc=%s\n' "$#" >&2; return 90; }}
  [[ "$1" == "new-pane" ]] || return 91
  [[ "$2" == "--direction" ]] || return 92
  [[ "$3" == "right" ]] || return 93
  [[ "$4" == "--command" ]] || return 94
  printf '%s' "$5" > {capture_arg}
}}
{command}
[[ ! -e {bad_substitution_arg} ]]
[[ ! -e {bad_backtick_arg} ]]
[[ ! -e {bad_semicolon_arg} ]]
"#
        );

        let output = std::process::Command::new("bash")
            .arg("-lc")
            .arg(script)
            .output()
            .expect("run bash");

        assert!(
            output.status.success(),
            "generated command should parse as one --command argv without outer-shell side effects: stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            std::fs::read_to_string(&capture).expect("read captured command"),
            launch
        );
    }

    #[test]
    fn new_pane_shell_command_quotes_command_argument() {
        let launch = "codex \"review this\" '$PATH' `whoami`; echo nope\nnext";
        let command = new_pane_shell_command("right", launch);
        let expected_launch_arg = shell_command_arg(launch);

        assert_eq!(
            command,
            format!("limux new-pane --direction right --command {expected_launch_arg}")
        );
        assert!(
            !command.contains('\n'),
            "generated new-pane command should remain one physical line"
        );
    }

    #[test]
    fn agents_md_uses_shell_quoted_new_pane_command_example() {
        let md = build_agents_md(
            &[],
            "/tmp/team",
            "active-ws",
            "ws-uuid-123",
            "9:terminal-orch",
            AgentTeamCoordinationFiles {
                roster_path: Path::new("/tmp/team/LIMUX_TEAM_ROSTER.md"),
                ledger_path: Path::new("/tmp/team/LIMUX_REVIEW_LEDGER.md"),
            },
            &[],
        );

        assert!(md.contains("limux new-pane --direction right --command 'bash'"));
        assert!(!md.contains("limux new-pane --direction right --command bash\n"));
    }

    fn assert_bash_word_round_trip(quoted: &str, expected: &str) {
        let output = std::process::Command::new("bash")
            .arg("-lc")
            .arg(format!("set -- {quoted}; printf '%s\\0%s' \"$#\" \"$1\""))
            .output()
            .expect("run bash");

        assert!(
            output.status.success(),
            "bash rejected quoted word {quoted:?}: stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
        let split = output
            .stdout
            .iter()
            .position(|byte| *byte == 0)
            .expect("round trip output should include NUL separator");
        assert_eq!(
            std::str::from_utf8(&output.stdout[..split]).expect("utf8 argc"),
            "1",
            "quoted word should parse as exactly one shell word"
        );
        assert_eq!(
            String::from_utf8(output.stdout[(split + 1)..].to_vec()).expect("utf8 stdout"),
            expected
        );
    }
}

#[cfg(test)]
mod review_prepare_tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use tokio::net::UnixListener;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[derive(Clone, Debug)]
    struct ReviewRecordedRequest {
        method: String,
        params: Value,
    }

    async fn spawn_review_fake_server(
        socket: PathBuf,
    ) -> (
        Arc<Mutex<Vec<ReviewRecordedRequest>>>,
        tokio::task::JoinHandle<()>,
    ) {
        let listener = UnixListener::bind(&socket).expect("bind fake review socket");
        let requests = Arc::new(Mutex::new(Vec::new()));
        let server_requests = Arc::clone(&requests);

        let handle = tokio::spawn(async move {
            loop {
                let Ok((stream, _addr)) = listener.accept().await else {
                    break;
                };
                let requests = Arc::clone(&server_requests);
                tokio::spawn(async move {
                    let (reader_half, mut writer_half) = stream.into_split();
                    let mut reader = BufReader::new(reader_half);
                    let mut line = String::new();
                    if reader.read_line(&mut line).await.expect("read request") == 0 {
                        return;
                    }

                    let request: V2Request =
                        serde_json::from_str(line.trim()).expect("parse fake request");
                    let method = request.method.clone();
                    let params = request.params.clone();
                    requests
                        .lock()
                        .expect("lock requests")
                        .push(ReviewRecordedRequest {
                            method: method.clone(),
                            params: params.clone(),
                        });

                    let response = match method.as_str() {
                        "workspace.current" => V2Response::success(
                            request.id.clone(),
                            json!({ "workspace_id": "workspace:team" }),
                        ),
                        "surface.list" => V2Response::success(
                            request.id.clone(),
                            json!({
                                "surfaces": [{
                                    "pane_id": "pane:1",
                                    "surface_id": "surface:1:orchestrator",
                                    "focused": true,
                                    "title": "orchestrator"
                                }]
                            }),
                        ),
                        "pane.create" => V2Response::success(
                            request.id.clone(),
                            json!({
                                "pane_id": "pane:7",
                                "surface_id": "surface:7:claude",
                                "surface_ref": "surface:7:claude",
                                "ok": true
                            }),
                        ),
                        "surface.send_text" => V2Response::success(
                            request.id.clone(),
                            json!({
                                "ok": true,
                                "surface_id": params
                                    .get("surface_id")
                                    .and_then(Value::as_str)
                                    .unwrap_or("surface:unknown")
                            }),
                        ),
                        "surface.send_key" => V2Response::success(
                            request.id.clone(),
                            json!({
                                "ok": true,
                                "surface_id": params
                                    .get("surface_id")
                                    .and_then(Value::as_str)
                                    .unwrap_or("surface:unknown")
                            }),
                        ),
                        other => {
                            panic!("unexpected fake review request method {other}: {params:?}")
                        }
                    };
                    let mut payload = serde_json::to_string(&response).expect("encode response");
                    payload.push('\n');
                    writer_half
                        .write_all(payload.as_bytes())
                        .await
                        .expect("write response");
                });
            }
        });

        (requests, handle)
    }

    #[test]
    fn review_prepare_dry_run_reports_paths_without_writing_files() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = tmp.path();
        let payload = run_review_prepare(&args(&[
            "prepare",
            "--cwd",
            cwd.to_str().expect("utf8 cwd"),
            "--artifact",
            "rust/limux-cli/src/main.rs",
            "--reviewer",
            "claude",
            "--lens",
            "security",
            "--summary",
            "Review Phase 5D1 scaffold",
            "--review-id",
            "phase5d1-test",
            "--dry-run",
        ]))
        .expect("dry-run should succeed without host");

        let request_path = cwd.join("reviews/phase5d1-test.md");
        let ledger_path = cwd.join(AGENT_TEAM_DEFAULT_LEDGER_FILE);
        assert_eq!(payload["dry_run"], true);
        assert_eq!(payload["request"]["status"], "planned");
        assert_eq!(payload["ledger"]["status"], "planned");
        assert_eq!(
            payload.get("request_path").and_then(Value::as_str),
            Some(request_path.to_string_lossy().as_ref())
        );
        assert_eq!(
            payload.get("ledger_path").and_then(Value::as_str),
            Some(ledger_path.to_string_lossy().as_ref())
        );
        assert!(payload["prompt"]
            .as_str()
            .unwrap()
            .contains("phase5d1-test"));
        assert!(!request_path.exists());
        assert!(!ledger_path.exists());
    }

    #[test]
    fn review_prepare_creates_request_file_and_appends_pending_ledger_entry() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = tmp.path();
        let payload = run_review_prepare(&args(&[
            "prepare",
            "--cwd",
            cwd.to_str().expect("utf8 cwd"),
            "--artifact",
            "rust/limux-cli/src/main.rs",
            "--reviewer",
            "claude",
            "--lens",
            "security",
            "--summary",
            "Review Phase 5D1 scaffold",
            "--review-id",
            "phase5d1-test",
        ]))
        .expect("review prepare should create durable artifacts");

        let request_path = cwd.join("reviews/phase5d1-test.md");
        let ledger_path = cwd.join(AGENT_TEAM_DEFAULT_LEDGER_FILE);
        assert_eq!(payload["request"]["status"], "created");
        assert_eq!(payload["ledger"]["status"], "appended");
        assert_eq!(
            payload.get("request_path").and_then(Value::as_str),
            Some(request_path.to_string_lossy().as_ref())
        );

        let request = std::fs::read_to_string(&request_path).expect("read request");
        assert!(request.starts_with("<!-- limux-review-request generated:v1 -->"));
        assert!(request.contains("Review ID: `phase5d1-test`"));
        assert!(request.contains("Reviewer: `claude`"));
        assert!(request.contains("Lens: `security`"));
        assert!(request.contains("Do not paste raw terminal transcripts"));

        let ledger = std::fs::read_to_string(&ledger_path).expect("read ledger");
        assert!(ledger.starts_with(AGENT_TEAM_LEDGER_MARKER));
        assert!(ledger.contains("## pending - phase5d1-test"));
        assert!(ledger.contains("Status: pending"));
        assert!(ledger.contains("Request: `"));
        assert!(ledger.contains("Reviewer: `claude`"));
    }

    #[test]
    fn review_prepare_appends_without_rewriting_existing_ledger() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = tmp.path();
        let ledger_path = cwd.join(AGENT_TEAM_DEFAULT_LEDGER_FILE);
        std::fs::write(&ledger_path, "manual ledger header\n").expect("seed ledger");

        run_review_prepare(&args(&[
            "prepare",
            "--cwd",
            cwd.to_str().expect("utf8 cwd"),
            "--artifact",
            "README.md",
            "--reviewer",
            "manual",
            "--lens",
            "correctness",
            "--summary",
            "Check docs",
            "--review-id",
            "manual-review",
        ]))
        .expect("review prepare should append");

        let ledger = std::fs::read_to_string(&ledger_path).expect("read ledger");
        assert!(ledger.starts_with("manual ledger header\n"));
        assert!(ledger.contains("## pending - manual-review"));
    }

    #[cfg(unix)]
    #[test]
    fn review_prepare_refuses_symlink_ledger_path() {
        use std::os::unix::fs::symlink;

        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = tmp.path();
        let target = cwd.join("target-ledger.md");
        std::fs::write(&target, "target ledger\n").expect("write target");
        let symlink_path = cwd.join("ledger-link.md");
        symlink(&target, &symlink_path).expect("create ledger symlink");

        let error = run_review_prepare(&args(&[
            "prepare",
            "--cwd",
            cwd.to_str().expect("utf8 cwd"),
            "--ledger-path",
            symlink_path.to_str().expect("utf8 symlink"),
            "--artifact",
            "README.md",
            "--reviewer",
            "manual",
            "--lens",
            "correctness",
            "--summary",
            "Check docs",
            "--review-id",
            "manual-review",
        ]))
        .expect_err("ledger symlink should be refused");

        assert!(
            error
                .to_string()
                .contains("refusing to use review ledger path because it is a symlink"),
            "unexpected error: {error:#}"
        );
        assert_eq!(
            std::fs::read_to_string(&target).expect("read target"),
            "target ledger\n"
        );
    }

    #[cfg(unix)]
    #[test]
    fn review_prepare_refuses_symlink_reviews_dir() {
        use std::os::unix::fs::symlink;

        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = tmp.path();
        let target = cwd.join("real-reviews");
        std::fs::create_dir(&target).expect("create target dir");
        let symlink_path = cwd.join("reviews-link");
        symlink(&target, &symlink_path).expect("create reviews symlink");

        let error = run_review_prepare(&args(&[
            "prepare",
            "--cwd",
            cwd.to_str().expect("utf8 cwd"),
            "--reviews-dir",
            symlink_path.to_str().expect("utf8 symlink"),
            "--artifact",
            "README.md",
            "--reviewer",
            "manual",
            "--lens",
            "correctness",
            "--summary",
            "Check docs",
            "--review-id",
            "manual-review",
        ]))
        .expect_err("reviews dir symlink should be refused");

        assert!(
            error
                .to_string()
                .contains("refusing to use reviews directory because it is a symlink"),
            "unexpected error: {error:#}"
        );
    }

    #[test]
    fn review_prepare_refuses_non_regular_ledger_path() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = tmp.path();
        let ledger_path = cwd.join("ledger-dir");
        std::fs::create_dir(&ledger_path).expect("create ledger dir");

        let error = run_review_prepare(&args(&[
            "prepare",
            "--cwd",
            cwd.to_str().expect("utf8 cwd"),
            "--ledger-path",
            ledger_path.to_str().expect("utf8 ledger dir"),
            "--artifact",
            "README.md",
            "--reviewer",
            "manual",
            "--lens",
            "correctness",
            "--summary",
            "Check docs",
            "--review-id",
            "manual-review",
        ]))
        .expect_err("directory ledger path should be refused");

        assert!(
            error
                .to_string()
                .contains("refusing to use review ledger path because it is not a regular file"),
            "unexpected error: {error:#}"
        );
    }

    #[test]
    fn review_prepare_refuses_existing_request_file() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = tmp.path();
        let reviews_dir = cwd.join("reviews");
        std::fs::create_dir(&reviews_dir).expect("create reviews dir");
        let request_path = reviews_dir.join("manual-review.md");
        std::fs::write(&request_path, "existing review\n").expect("seed request");

        let error = run_review_prepare(&args(&[
            "prepare",
            "--cwd",
            cwd.to_str().expect("utf8 cwd"),
            "--artifact",
            "README.md",
            "--reviewer",
            "manual",
            "--lens",
            "correctness",
            "--summary",
            "Check docs",
            "--review-id",
            "manual-review",
        ]))
        .expect_err("existing request should be refused");

        assert!(
            error.to_string().contains("review request already exists"),
            "unexpected error: {error:#}"
        );
        assert_eq!(
            std::fs::read_to_string(&request_path).expect("read request"),
            "existing review\n"
        );
    }

    #[test]
    fn review_prepare_refuses_overlapping_request_and_ledger_paths() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = tmp.path();

        let error = run_review_prepare(&args(&[
            "prepare",
            "--cwd",
            cwd.to_str().expect("utf8 cwd"),
            "--ledger-path",
            "reviews/manual-review.md",
            "--artifact",
            "README.md",
            "--reviewer",
            "manual",
            "--lens",
            "correctness",
            "--summary",
            "Check docs",
            "--review-id",
            "manual-review",
        ]))
        .expect_err("overlapping request and ledger paths should be refused");

        assert!(
            error
                .to_string()
                .contains("review output paths must be distinct"),
            "unexpected error: {error:#}"
        );
    }

    #[test]
    fn review_prepare_rejects_invalid_reviewer_and_lens() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = tmp.path();

        let reviewer_error = run_review_prepare(&args(&[
            "prepare",
            "--cwd",
            cwd.to_str().expect("utf8 cwd"),
            "--artifact",
            "README.md",
            "--reviewer",
            "unknown",
            "--lens",
            "correctness",
            "--summary",
            "Check docs",
        ]))
        .expect_err("invalid reviewer should fail");
        assert!(
            reviewer_error
                .to_string()
                .contains("reviewer must be one of"),
            "unexpected error: {reviewer_error:#}"
        );

        let lens_error = run_review_prepare(&args(&[
            "prepare",
            "--cwd",
            cwd.to_str().expect("utf8 cwd"),
            "--artifact",
            "README.md",
            "--reviewer",
            "manual",
            "--lens",
            "unknown",
            "--summary",
            "Check docs",
        ]))
        .expect_err("invalid lens should fail");
        assert!(
            lens_error
                .to_string()
                .contains("review lens must be one of"),
            "unexpected error: {lens_error:#}"
        );
    }

    #[tokio::test]
    async fn review_command_dispatches_prepare_without_host_contact() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = tmp.path();
        let mut client = Client::new(cwd.join("missing.sock"));
        let opts = GlobalOptions {
            socket: None,
            channel: None,
            socket_mode: SocketMode::Runtime,
            json_output: true,
            id_format: IdFormat::Refs,
            request: None,
            pretty: false,
            command_args: args(&[
                "review",
                "prepare",
                "--cwd",
                cwd.to_str().expect("utf8 cwd"),
                "--artifact",
                "README.md",
                "--reviewer",
                "manual",
                "--lens",
                "correctness",
                "--summary",
                "Check docs",
                "--review-id",
                "manual-review",
                "--dry-run",
            ]),
        };

        let output = execute_command(&mut client, &opts)
            .await
            .expect("review dispatch should not contact host");
        let CommandOutput::Json(payload) = output else {
            panic!("expected json payload");
        };
        assert_eq!(payload["dry_run"], true);
        assert_eq!(payload["review_id"], "manual-review");
        assert!(!cwd.join("reviews/manual-review.md").exists());
    }

    #[tokio::test]
    async fn review_spawn_dry_run_uses_existing_request_without_host_contact() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = tmp.path();
        run_review_prepare(&args(&[
            "prepare",
            "--cwd",
            cwd.to_str().expect("utf8 cwd"),
            "--artifact",
            "README.md",
            "--reviewer",
            "claude",
            "--lens",
            "correctness",
            "--summary",
            "Check docs",
            "--review-id",
            "phase5d2-dry",
        ]))
        .expect("prepare should create request");

        let mut client = Client::new(cwd.join("missing.sock"));
        let payload = run_review_command(
            &mut client,
            &args(&[
                "spawn",
                "--cwd",
                cwd.to_str().expect("utf8 cwd"),
                "--review-id",
                "phase5d2-dry",
                "--dry-run",
            ]),
        )
        .await
        .expect("dry-run spawn should not contact host");

        let evidence_path = cwd.join("reviews/phase5d2-dry.evidence.md");
        assert_eq!(payload["review_command"], "spawn");
        assert_eq!(payload["dry_run"], true);
        assert_eq!(payload["spawn"]["status"], "planned");
        assert_eq!(payload["prompt"]["status"], "planned");
        assert_eq!(payload["ledger"]["status"], "planned");
        assert_eq!(payload["evidence"]["status"], "planned");
        assert!(!evidence_path.exists());

        let ledger =
            std::fs::read_to_string(cwd.join(AGENT_TEAM_DEFAULT_LEDGER_FILE)).expect("read ledger");
        assert!(ledger.contains("## pending - phase5d2-dry"));
        assert!(!ledger.contains("## in-progress - phase5d2-dry"));
    }

    #[tokio::test]
    async fn review_spawn_dry_run_hcom_launch_mode_reports_run_here_command() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = tmp.path();
        run_review_prepare(&args(&[
            "prepare",
            "--cwd",
            cwd.to_str().expect("utf8 cwd"),
            "--artifact",
            "README.md",
            "--reviewer",
            "claude",
            "--lens",
            "correctness",
            "--summary",
            "Check docs",
            "--review-id",
            "phase5d2-hcom",
        ]))
        .expect("prepare should create request");

        let mut client = Client::new(cwd.join("missing.sock"));
        let payload = run_review_command(
            &mut client,
            &args(&[
                "spawn",
                "--cwd",
                cwd.to_str().expect("utf8 cwd"),
                "--review-id",
                "phase5d2-hcom",
                "--launch-mode",
                "hcom",
                "--dry-run",
            ]),
        )
        .await
        .expect("dry-run spawn should not contact host");

        assert_eq!(payload["launch_mode"], "hcom");
        assert_eq!(payload["launch_command"], "hcom claude --run-here");
        assert_eq!(payload["spawn"]["status"], "planned");
    }

    #[tokio::test]
    async fn review_spawn_requires_matching_pending_ledger_entry_before_host_contact() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = tmp.path();
        run_review_prepare(&args(&[
            "prepare",
            "--cwd",
            cwd.to_str().expect("utf8 cwd"),
            "--artifact",
            "README.md",
            "--reviewer",
            "claude",
            "--lens",
            "correctness",
            "--summary",
            "Check docs",
            "--review-id",
            "phase5d2-missing-ledger",
        ]))
        .expect("prepare should create request");
        std::fs::write(
            cwd.join(AGENT_TEAM_DEFAULT_LEDGER_FILE),
            "manual ledger without the pending entry\n",
        )
        .expect("replace ledger");

        let mut client = Client::new(cwd.join("missing.sock"));
        let error = run_review_command(
            &mut client,
            &args(&[
                "spawn",
                "--cwd",
                cwd.to_str().expect("utf8 cwd"),
                "--review-id",
                "phase5d2-missing-ledger",
                "--dry-run",
            ]),
        )
        .await
        .expect_err("missing pending ledger entry should fail before host contact");

        assert!(
            error
                .to_string()
                .contains("review ledger has no pending entry"),
            "unexpected error: {error:#}"
        );
        assert!(!cwd
            .join("reviews/phase5d2-missing-ledger.evidence.md")
            .exists());
    }

    #[tokio::test]
    async fn review_spawn_launches_reviewer_sends_prompt_and_updates_ledger_entry() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = tmp.path();
        run_review_prepare(&args(&[
            "prepare",
            "--cwd",
            cwd.to_str().expect("utf8 cwd"),
            "--artifact",
            "rust/limux-cli/src/main.rs",
            "--reviewer",
            "claude",
            "--lens",
            "security",
            "--summary",
            "Review Phase 5D2 wrapper",
            "--review-id",
            "phase5d2-live",
        ]))
        .expect("prepare should create request");
        let ledger_path = cwd.join(AGENT_TEAM_DEFAULT_LEDGER_FILE);
        {
            let mut ledger = OpenOptions::new()
                .append(true)
                .open(&ledger_path)
                .expect("open ledger");
            ledger
                .write_all(b"\nmanual suffix that must survive\n")
                .expect("append sentinel");
        }

        let socket = cwd.join("limux.sock");
        let (requests, server) = spawn_review_fake_server(socket.clone()).await;
        let mut client = Client::new(socket);
        let payload = run_review_command(
            &mut client,
            &args(&[
                "spawn",
                "--cwd",
                cwd.to_str().expect("utf8 cwd"),
                "--review-id",
                "phase5d2-live",
                "--workspace",
                "workspace:team",
                "--surface",
                "surface:1:orchestrator",
                "--direction",
                "down",
            ]),
        )
        .await
        .expect("review spawn should complete against fake host");
        server.abort();

        let evidence_path = cwd.join("reviews/phase5d2-live.evidence.md");
        assert_eq!(payload["review_command"], "spawn");
        assert_eq!(payload["dry_run"], false);
        assert_eq!(payload["spawn"]["status"], "created");
        assert_eq!(payload["prompt"]["status"], "sent");
        assert_eq!(payload["ledger"]["status"], "updated");
        assert_eq!(payload["evidence"]["status"], "created");
        assert_eq!(payload["reviewer_surface_id"], "surface:7:claude");
        assert!(evidence_path.exists());

        let requests = requests.lock().expect("lock requests").clone();
        let pane_create = requests
            .iter()
            .find(|request| request.method == "pane.create")
            .expect("pane.create request");
        assert_eq!(pane_create.params["command"].as_str(), Some("claude"));
        assert_eq!(pane_create.params["direction"].as_str(), Some("down"));
        assert_eq!(
            pane_create.params["surface_id"].as_str(),
            Some("surface:1:orchestrator")
        );

        let send_text = requests
            .iter()
            .find(|request| request.method == "surface.send_text")
            .expect("surface.send_text request");
        assert_eq!(
            send_text.params["surface_id"].as_str(),
            Some("surface:7:claude")
        );
        let prompt = send_text.params["text"].as_str().expect("prompt text");
        assert!(prompt.contains("Review request: phase5d2-live"));
        assert!(prompt.contains("Reviewer: claude"));
        assert!(prompt.contains("Lens: security"));

        let send_key = requests
            .iter()
            .find(|request| request.method == "surface.send_key")
            .expect("surface.send_key request");
        assert_eq!(send_key.params["key"].as_str(), Some("enter"));
        assert_eq!(
            send_key.params["surface_id"].as_str(),
            Some("surface:7:claude")
        );

        let evidence = std::fs::read_to_string(&evidence_path).expect("read evidence pointer");
        assert!(evidence.contains("Review ID: `phase5d2-live`"));
        assert!(evidence.contains("Reviewer surface: `surface:7:claude`"));
        assert!(evidence.contains("Capture command: `limux read-screen --surface surface:7:claude --scrollback --lines 120`"));

        let ledger = std::fs::read_to_string(&ledger_path).expect("read ledger");
        assert!(ledger.contains("## in-progress - phase5d2-live"));
        assert!(!ledger.contains("## pending - phase5d2-live"));
        assert!(ledger.contains("Reviewer surface: `surface:7:claude`"));
        assert!(ledger.contains("Evidence pointer: `"));
        assert!(ledger.contains("manual suffix that must survive"));
    }

    #[test]
    fn review_prepare_rejects_missing_required_arguments() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = tmp.path();
        let error = run_review_prepare(&args(&[
            "prepare",
            "--cwd",
            cwd.to_str().expect("utf8 cwd"),
            "--reviewer",
            "claude",
            "--lens",
            "security",
            "--summary",
            "Missing artifact",
        ]))
        .expect_err("missing artifact should fail");

        assert!(
            error
                .to_string()
                .contains("review prepare requires --artifact"),
            "unexpected error: {error:#}"
        );
    }

    #[test]
    fn review_prepare_rejects_control_characters_in_generated_prompt_fields() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = tmp.path();
        let error = run_review_prepare(&args(&[
            "prepare",
            "--cwd",
            cwd.to_str().expect("utf8 cwd"),
            "--artifact",
            "README.md",
            "--reviewer",
            "manual",
            "--lens",
            "correctness",
            "--summary",
            "Check docs\u{1b}[31m",
        ]))
        .expect_err("terminal control characters should fail");

        assert!(
            error.to_string().contains("review summary"),
            "unexpected error: {error:#}"
        );
    }
}

#[cfg(test)]
mod new_pane_tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    fn test_env(name: &str) -> Option<String> {
        match name {
            "LIMUX_WORKSPACE_ID" => Some("workspace:agent".to_string()),
            "LIMUX_SURFACE_ID" => Some("surface:11:tab-a".to_string()),
            "LIMUX_PANE_ID" => Some("pane:11".to_string()),
            _ => None,
        }
    }

    #[test]
    fn new_pane_serializes_env_defaults_and_command() {
        let (workspace, params) = build_new_pane_request(&args(&["--command", "claude"]), test_env);

        assert_eq!(workspace.as_deref(), Some("workspace:agent"));
        assert_eq!(
            params,
            json!({
                "direction": "right",
                "type": "terminal",
                "surface_id": "surface:11:tab-a",
                "pane_id": "pane:11",
                "command": "claude"
            })
        );
    }

    #[test]
    fn new_pane_flags_override_env_and_preserve_raw_refs() {
        let (workspace, params) = build_new_pane_request(
            &args(&[
                "--workspace",
                "raw-workspace",
                "--surface",
                "7:tab-b",
                "--pane",
                "7",
                "--direction",
                "down",
                "--type",
                "terminal",
                "--command",
                "codex --ask-for-approval never",
            ]),
            test_env,
        );

        assert_eq!(workspace.as_deref(), Some("raw-workspace"));
        assert_eq!(
            params,
            json!({
                "direction": "down",
                "type": "terminal",
                "surface_id": "7:tab-b",
                "pane_id": "7",
                "command": "codex --ask-for-approval never"
            })
        );
    }

    #[test]
    fn new_pane_command_preserves_metacharacters_in_request_json() {
        let payload =
            "codex \"review\" 'single' $HOME $(whoami) `id`; echo nope \\\\slash\ttab\nsecond";

        let (_workspace, params) = build_new_pane_request(&args(&["--command", payload]), test_env);

        assert_eq!(params["command"].as_str(), Some(payload));
    }

    #[test]
    fn new_pane_command_accepts_leading_hyphen_value() {
        let (_workspace, params) =
            build_new_pane_request(&args(&["--command", "--help"]), test_env);

        assert_eq!(params["command"].as_str(), Some("--help"));
        assert!(new_pane_unexpected_positionals(&args(&["--command", "--help"])).is_empty());
    }

    #[tokio::test]
    async fn new_pane_rejects_unquoted_extra_command_tokens_before_contacting_host() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let mut client = Client::new(tmp.path().join("unused.sock"));

        let err = run_new_pane(
            &mut client,
            &args(&["--command", "codex", "review this diff"]),
        )
        .await
        .expect_err("unquoted extra command tokens should fail before socket use");

        let msg = format!("{err:#}");
        assert!(
            msg.contains("unexpected positional argument"),
            "unexpected error: {msg}"
        );
        assert!(
            msg.contains("Quote multi-word launch commands"),
            "unexpected error: {msg}"
        );
        assert!(msg.contains("limux send"), "unexpected error: {msg}");
    }

    #[test]
    fn new_pane_allows_shell_quoted_multiword_command_as_one_argv() {
        assert!(new_pane_unexpected_positionals(&args(&[
            "--command",
            "codex \"review this diff\""
        ]))
        .is_empty());
    }

    #[test]
    fn new_pane_without_env_preserves_active_workspace_fallback() {
        let (workspace, params) = build_new_pane_request(&args(&[]), |_| None);

        assert_eq!(workspace, None);
        assert_eq!(
            params,
            json!({
                "direction": "right",
                "type": "terminal"
            })
        );
    }

    #[test]
    fn pane_action_serializes_env_defaults_and_color() {
        let params = build_pane_action_request(
            &args(&["--action", "set_flag_color", "--color", "orange"]),
            test_env,
        )
        .expect("pane action request");

        assert_eq!(
            params,
            json!({
                "action": "set_flag_color",
                "workspace_id": "workspace:agent",
                "pane_id": "pane:11",
                "color": "orange"
            })
        );
    }

    #[test]
    fn pane_action_flags_override_env_and_clear_omits_color() {
        let params = build_pane_action_request(
            &args(&[
                "--action",
                "clear_flag_color",
                "--workspace",
                "workspace:manual",
                "--pane",
                "pane:77",
            ]),
            test_env,
        )
        .expect("pane action request");

        assert_eq!(
            params,
            json!({
                "action": "clear_flag_color",
                "workspace_id": "workspace:manual",
                "pane_id": "pane:77"
            })
        );
    }
}
