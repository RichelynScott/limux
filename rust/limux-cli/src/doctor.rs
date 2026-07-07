use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::os::unix::net::UnixStream as StdUnixStream;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use limux_control::{BuildInfo, InstallInfo};
use limux_protocol::{V2Request, V2Response};
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

const DEFAULT_LOG_LINES: usize = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CheckStatus {
    Ok,
    Warn,
    Fail,
}

impl CheckStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Warn => "warn",
            Self::Fail => "fail",
        }
    }
}

#[derive(Debug, Clone)]
struct Check {
    name: &'static str,
    status: CheckStatus,
    message: String,
    data: Value,
}

impl Check {
    fn new(name: &'static str, status: CheckStatus, message: impl Into<String>) -> Self {
        Self {
            name,
            status,
            message: message.into(),
            data: Value::Null,
        }
    }

    fn with_data(mut self, data: Value) -> Self {
        self.data = data;
        self
    }

    fn to_json(&self) -> Value {
        json!({
            "name": self.name,
            "status": self.status.as_str(),
            "message": self.message,
            "data": self.data,
        })
    }
}

#[derive(Debug, Clone)]
struct LauncherInfo {
    name: String,
    link_path: PathBuf,
    target_path: PathBuf,
    target_error: Option<String>,
    install_root: Option<PathBuf>,
    channel: Option<String>,
    install_info: Option<InstallInfo>,
    install_info_error: Option<String>,
}

#[derive(Debug, Clone)]
struct DoctorOptions {
    json_output: bool,
    log_triage: bool,
    lines: usize,
}

pub struct DoctorRun {
    pub json_output: bool,
    pub payload: Value,
    pub text: String,
    pub exit_code: i32,
}

pub fn wants_json(args: &[String], global_json: bool) -> bool {
    global_json || args.iter().any(|arg| arg == "--json")
}

pub async fn run(
    args: &[String],
    global_json: bool,
    socket: PathBuf,
    cli_build: BuildInfo,
) -> Result<DoctorRun> {
    let options = parse_options(args, global_json)?;
    let prefix = env::var_os("LIMUX_USER_PREFIX")
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|home| home.join(".local")))
        .ok_or_else(|| anyhow!("could not resolve user prefix"))?;

    let launchers = discover_launchers(&prefix);
    let install_roots = known_install_roots(&launchers);

    let mut checks = Vec::new();
    checks.push(check_launchers(&prefix, &launchers));
    checks.push(check_processes(&install_roots));
    checks.push(check_socket(&socket, &cli_build).await);
    checks.push(check_stale_sockets(&socket));
    checks.push(check_ghostty_resources(&install_roots));

    let log_triage = if options.log_triage {
        Some(run_log_triage(options.lines))
    } else {
        None
    };

    let exit_code = if checks.iter().any(|check| check.status == CheckStatus::Fail) {
        1
    } else if checks.iter().any(|check| check.status == CheckStatus::Warn) {
        2
    } else {
        0
    };

    let payload = json!({
        "ok": exit_code == 0,
        "exit_code": exit_code,
        "prefix": prefix.to_string_lossy(),
        "socket": socket.to_string_lossy(),
        "cli_build": cli_build,
        "launchers": launchers_json(&launchers),
        "install_roots": install_roots
            .iter()
            .map(|path| Value::String(path.to_string_lossy().to_string()))
            .collect::<Vec<_>>(),
        "checks": checks.iter().map(Check::to_json).collect::<Vec<_>>(),
        "log_triage": log_triage,
    });
    let text = render_text_report(&payload);

    Ok(DoctorRun {
        json_output: options.json_output,
        payload,
        text,
        exit_code,
    })
}

fn parse_options(args: &[String], global_json: bool) -> Result<DoctorOptions> {
    let mut options = DoctorOptions {
        json_output: global_json,
        log_triage: false,
        lines: DEFAULT_LOG_LINES,
    };
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--json" => {
                options.json_output = true;
                index += 1;
            }
            "--log-triage" => {
                options.log_triage = true;
                index += 1;
            }
            "--lines" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| anyhow!("doctor --lines requires a value"))?;
                options.lines = value
                    .parse::<usize>()
                    .context("doctor --lines must be a positive integer")?;
                index += 2;
            }
            other => return Err(anyhow!("unknown doctor argument: {other}")),
        }
    }
    Ok(options)
}

fn discover_launchers(prefix: &Path) -> Vec<LauncherInfo> {
    let bin_dir = prefix.join("bin");
    let mut launchers = Vec::new();
    let Ok(entries) = fs::read_dir(&bin_dir) else {
        return launchers;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !name.starts_with("limux") {
            continue;
        }
        let Ok(metadata) = fs::symlink_metadata(&path) else {
            continue;
        };
        if !metadata.file_type().is_symlink() {
            continue;
        }
        let Ok(raw_target) = fs::read_link(&path) else {
            continue;
        };
        let target_path = if raw_target.is_absolute() {
            raw_target
        } else {
            path.parent().unwrap_or(&bin_dir).join(raw_target)
        };
        let wrapper_result = fs::read_to_string(&target_path);
        let target_error = wrapper_result.as_ref().err().map(|error| error.to_string());
        let wrapper_text = wrapper_result.unwrap_or_default();
        let install_root = parse_wrapper_install_root(&wrapper_text)
            .map(PathBuf::from)
            .or_else(|| infer_install_root(prefix, &target_path));
        let channel = parse_wrapper_channel(&wrapper_text);
        let (install_info, install_info_error) = install_root
            .as_ref()
            .map(|root| parse_install_info_detail(&root.join("install-info.json")))
            .unwrap_or((None, None));
        launchers.push(LauncherInfo {
            name: name.to_string(),
            link_path: path,
            target_path,
            target_error,
            install_root,
            channel,
            install_info,
            install_info_error,
        });
    }
    launchers.sort_by(|left, right| left.name.cmp(&right.name));
    launchers
}

fn parse_wrapper_install_root(text: &str) -> Option<String> {
    text.lines()
        .find_map(|line| parse_shell_assignment(line, "INSTALL_ROOT"))
}

fn parse_wrapper_channel(text: &str) -> Option<String> {
    text.lines().find_map(|line| {
        parse_shell_assignment(line.trim_start_matches("export "), "LIMUX_CHANNEL")
    })
}

fn parse_shell_assignment(line: &str, key: &str) -> Option<String> {
    let raw = line.trim().strip_prefix(key)?.trim_start();
    let raw = raw.strip_prefix('=')?.trim();
    let raw = raw.strip_suffix(';').unwrap_or(raw).trim();
    let raw = raw
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(raw);
    (!raw.is_empty()).then_some(raw.to_string())
}

fn infer_install_root(prefix: &Path, target_path: &Path) -> Option<PathBuf> {
    let reviewed = prefix.join("limux-reviewed");
    let mut current = target_path;
    loop {
        if current.parent() == Some(reviewed.as_path()) {
            return Some(current.to_path_buf());
        }
        current = current.parent()?;
    }
}

fn parse_install_info_detail(path: &Path) -> (Option<InstallInfo>, Option<String>) {
    if !path.exists() {
        return (None, None);
    }
    match fs::read_to_string(path) {
        Ok(text) => match serde_json::from_str::<InstallInfo>(&text) {
            Ok(info) => (Some(info), None),
            Err(error) => (None, Some(error.to_string())),
        },
        Err(error) => (None, Some(error.to_string())),
    }
}

fn known_install_roots(launchers: &[LauncherInfo]) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    for launcher in launchers {
        if let Some(root) = &launcher.install_root {
            if !roots.iter().any(|existing| existing == root) {
                roots.push(root.clone());
            }
        }
    }
    roots.sort();
    roots
}

fn check_launchers(prefix: &Path, launchers: &[LauncherInfo]) -> Check {
    if launchers.is_empty() {
        return Check::new(
            "launchers",
            CheckStatus::Warn,
            format!(
                "no limux* launcher symlinks found under {}",
                prefix.join("bin").display()
            ),
        )
        .with_data(json!([]));
    }

    let malformed = launchers
        .iter()
        .filter(|launcher| launcher.install_info_error.is_some())
        .count();
    let broken_targets = launchers
        .iter()
        .filter(|launcher| launcher.target_error.is_some())
        .count();
    let missing_roots = launchers
        .iter()
        .filter(|launcher| launcher.install_root.is_none())
        .count();
    let status = if malformed > 0 || broken_targets > 0 || missing_roots > 0 {
        CheckStatus::Warn
    } else {
        CheckStatus::Ok
    };
    Check::new(
        "launchers",
        status,
        format!("found {} limux launcher symlink(s)", launchers.len()),
    )
    .with_data(launchers_json(launchers))
}

fn launchers_json(launchers: &[LauncherInfo]) -> Value {
    Value::Array(
        launchers
            .iter()
            .map(|launcher| {
                json!({
                    "name": launcher.name.clone(),
                    "link_path": launcher.link_path.to_string_lossy().to_string(),
                    "target_path": launcher.target_path.to_string_lossy().to_string(),
                    "target_error": launcher.target_error.clone(),
                    "install_root": launcher
                        .install_root
                        .as_ref()
                        .map(|path| path.to_string_lossy().to_string()),
                    "channel": launcher
                        .install_info
                        .as_ref()
                        .and_then(|info| info.channel.clone())
                        .or_else(|| launcher.channel.clone()),
                    "install_id": launcher
                        .install_info
                        .as_ref()
                        .and_then(|info| info.install_id.clone()),
                    "install_info_error": launcher.install_info_error.clone(),
                })
            })
            .collect(),
    )
}

#[cfg(target_os = "linux")]
fn check_processes(install_roots: &[PathBuf]) -> Check {
    let current_uid = proc_uid(std::process::id());
    let mut rows = Vec::new();
    let Ok(entries) = fs::read_dir("/proc") else {
        return Check::new("processes", CheckStatus::Warn, "could not read /proc");
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(pid) = name.to_str().and_then(|value| value.parse::<u32>().ok()) else {
            continue;
        };
        if current_uid.is_some() && proc_uid(pid) != current_uid {
            continue;
        }
        let exe = match fs::read_link(entry.path().join("exe")) {
            Ok(path) => path,
            Err(_) => continue,
        };
        let basename = exe.file_name().and_then(|name| name.to_str()).unwrap_or("");
        if basename != "limux-host" && basename != "limux" {
            continue;
        }
        let exe_text = exe.to_string_lossy();
        if !exe_text.contains("limux-reviewed/") && !exe_text.contains("/target/") {
            continue;
        }
        let known_root = install_roots
            .iter()
            .find(|root| exe.starts_with(root))
            .map(|root| root.to_string_lossy().to_string());
        rows.push(json!({
            "pid": pid,
            "exe": exe_text,
            "known_install_root": known_root,
            "dev_target": exe_text.contains("/target/"),
        }));
    }

    if rows.is_empty() {
        Check::new(
            "processes",
            CheckStatus::Warn,
            "no running Limux host process found",
        )
        .with_data(Value::Array(rows))
    } else {
        let unknown = rows.iter().filter(|row| {
            matches!(row.get("known_install_root"), None | Some(Value::Null))
                && !row
                    .get("dev_target")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
        });
        let status = if unknown.count() > 0 {
            CheckStatus::Warn
        } else {
            CheckStatus::Ok
        };
        Check::new(
            "processes",
            status,
            format!("found {} running Limux host process(es)", rows.len()),
        )
        .with_data(Value::Array(rows))
    }
}

#[cfg(not(target_os = "linux"))]
fn check_processes(_install_roots: &[PathBuf]) -> Check {
    Check::new(
        "processes",
        CheckStatus::Warn,
        "process inspection is Linux-only",
    )
}

#[cfg(target_os = "linux")]
fn proc_uid(pid: u32) -> Option<u32> {
    let status = fs::read_to_string(format!("/proc/{pid}/status")).ok()?;
    status.lines().find_map(|line| {
        let rest = line.strip_prefix("Uid:")?;
        rest.split_whitespace().next()?.parse::<u32>().ok()
    })
}

async fn check_socket(socket: &Path, cli_build: &BuildInfo) -> Check {
    match request(socket, "system.identify", json!({})).await {
        Ok(identify) => {
            let host_sha = identify
                .get("build")
                .and_then(|build| build.get("sha"))
                .and_then(Value::as_str);
            let status = match host_sha {
                Some(host_sha)
                    if host_sha != "unknown"
                        && cli_build.sha != "unknown"
                        && host_sha != cli_build.sha =>
                {
                    CheckStatus::Fail
                }
                Some(_) => CheckStatus::Ok,
                None => CheckStatus::Warn,
            };
            let message = match status {
                CheckStatus::Fail => "connected host build SHA differs from CLI build SHA",
                CheckStatus::Warn => "connected host did not report build identity",
                CheckStatus::Ok => "control socket responded to system.identify",
            };
            Check::new("socket", status, message).with_data(json!({
                "socket": socket.to_string_lossy(),
                "identify": identify,
            }))
        }
        Err(error) => {
            let message = if socket.exists() {
                format!("control socket exists but is not connectable: {error}")
            } else {
                format!("control socket does not exist or is not connectable: {error}")
            };
            Check::new("socket", CheckStatus::Warn, message)
                .with_data(json!({"socket": socket.to_string_lossy()}))
        }
    }
}

async fn request(socket: &Path, method: &str, params: Value) -> Result<Value> {
    let stream = UnixStream::connect(socket)
        .await
        .with_context(|| format!("failed to connect to {}", socket.display()))?;
    let (reader_half, mut writer_half) = stream.into_split();
    let request = V2Request {
        id: Some(Value::String(format!("doctor-{method}"))),
        method: method.to_string(),
        params,
    };
    let mut encoded = serde_json::to_string(&request)?;
    encoded.push('\n');
    writer_half.write_all(encoded.as_bytes()).await?;
    writer_half.flush().await?;

    let mut reader = BufReader::new(reader_half);
    let mut line = String::new();
    reader.read_line(&mut line).await?;
    if line.trim().is_empty() {
        return Err(anyhow!("empty response"));
    }
    let response: V2Response = serde_json::from_str(line.trim())?;
    if response.ok {
        Ok(response.result.unwrap_or_else(|| json!({})))
    } else {
        let message = response
            .error
            .map(|error| error.message)
            .unwrap_or_else(|| "unknown error".to_string());
        Err(anyhow!(message))
    }
}

fn check_stale_sockets(active_socket: &Path) -> Check {
    let mut stale = Vec::new();
    for socket in candidate_socket_paths(active_socket) {
        if socket == active_socket {
            continue;
        }
        if StdUnixStream::connect(&socket).is_err() {
            stale.push(socket);
        }
    }
    if stale.is_empty() {
        Check::new(
            "stale_sockets",
            CheckStatus::Ok,
            "no stale Limux sockets found",
        )
        .with_data(json!([]))
    } else {
        Check::new(
            "stale_sockets",
            CheckStatus::Warn,
            format!("found {} stale Limux socket(s)", stale.len()),
        )
        .with_data(Value::Array(
            stale
                .into_iter()
                .map(|path| Value::String(path.to_string_lossy().to_string()))
                .collect(),
        ))
    }
}

fn candidate_socket_paths(active_socket: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(parent) = active_socket.parent() {
        collect_socket_paths(parent, &mut paths);
    }
    if let Some(uid) = proc_uid(std::process::id()) {
        let runtime_dir = Path::new("/run/user").join(uid.to_string()).join("limux");
        collect_socket_paths_recursive(&runtime_dir, &mut paths, 3);
    }
    collect_socket_paths(Path::new("/tmp"), &mut paths);
    paths.sort();
    paths.dedup();
    paths
}

fn collect_socket_paths(dir: &Path, paths: &mut Vec<PathBuf>) {
    collect_socket_paths_recursive(dir, paths, 0);
}

fn collect_socket_paths_recursive(dir: &Path, paths: &mut Vec<PathBuf>, max_depth: usize) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if name.starts_with("limux") && name.ends_with(".sock") {
            paths.push(path.clone());
        }
        let is_dir = entry
            .file_type()
            .map(|file_type| file_type.is_dir())
            .unwrap_or(false);
        if max_depth > 0 && is_dir {
            collect_socket_paths_recursive(&path, paths, max_depth - 1);
        }
    }
}

fn check_ghostty_resources(install_roots: &[PathBuf]) -> Check {
    if install_roots.is_empty() {
        return Check::new(
            "ghostty_resources",
            CheckStatus::Warn,
            "no install roots available for Ghostty resource check",
        );
    }
    let rows: Vec<Value> = install_roots
        .iter()
        .map(|root| {
            let resources = root.join("share/limux/ghostty");
            let terminfo = root.join("share/limux/terminfo");
            let present = resources.join("shell-integration").is_dir()
                && (terminfo.join("g/ghostty").is_file()
                    || terminfo.join("x/xterm-ghostty").is_file());
            json!({
                "install_root": root.to_string_lossy(),
                "resources_dir": resources.to_string_lossy(),
                "terminfo_dir": terminfo.to_string_lossy(),
                "present": present,
            })
        })
        .collect();
    let missing = rows
        .iter()
        .filter(|row| !row.get("present").and_then(Value::as_bool).unwrap_or(false))
        .count();
    let status = if missing > 0 {
        CheckStatus::Warn
    } else {
        CheckStatus::Ok
    };
    Check::new(
        "ghostty_resources",
        status,
        if missing > 0 {
            format!("{missing} install root(s) missing bundled Ghostty resource shape")
        } else {
            "bundled Ghostty resource shape present".to_string()
        },
    )
    .with_data(Value::Array(rows))
}

fn run_log_triage(lines: usize) -> Value {
    let path = env::var_os("LIMUX_HOST_LOG_PATH")
        .map(PathBuf::from)
        .or_else(|| dirs::state_dir().map(|dir| dir.join("limux/logs/limux-host.log")));
    let Some(path) = path else {
        return json!({"status": "warn", "message": "could not resolve host log path"});
    };
    let Ok(text) = fs::read_to_string(&path) else {
        return json!({
            "status": "warn",
            "path": path.to_string_lossy(),
            "message": "host log not found",
        });
    };
    let mut summary: BTreeMap<&'static str, usize> = BTreeMap::new();
    let triaged = text
        .lines()
        .rev()
        .take(lines)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .filter_map(|line| {
            let cleaned = strip_terminal_controls(line);
            let class = classify_log_line(&cleaned)?;
            *summary.entry(class).or_default() += 1;
            Some(json!({"class": class, "text": cleaned}))
        })
        .collect::<Vec<_>>();
    json!({
        "status": "ok",
        "path": path.to_string_lossy(),
        "lines_scanned": lines,
        "summary": summary,
        "matches": triaged,
    })
}

fn classify_log_line(line: &str) -> Option<&'static str> {
    if line.contains("EGL")
        || line.contains("MESA-LOADER")
        || line.contains("ZINK")
        || line.contains("dri2")
        || line.contains("Compositor doesn't support moving popups")
    {
        Some("benign-env")
    } else if line.contains("Gtk-CRITICAL") || line.contains("GLib-GIO-CRITICAL") {
        Some("limux-error")
    } else if line.contains("Gtk-WARNING") || line.contains("Gdk-WARNING") {
        Some("limux-warning")
    } else if !line.trim().is_empty() {
        Some("unknown")
    } else {
        None
    }
}

fn strip_terminal_controls(input: &str) -> String {
    let mut output = String::new();
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            if chars.peek() == Some(&'[') {
                chars.next();
                for next in chars.by_ref() {
                    if ('@'..='~').contains(&next) {
                        break;
                    }
                }
            }
            continue;
        }
        if ch == '\t' || (!ch.is_control() && ch != '\u{7f}') {
            output.push(ch);
        }
    }
    output
}

fn render_text_report(payload: &Value) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "Limux doctor: {}",
        if payload.get("ok").and_then(Value::as_bool).unwrap_or(false) {
            "ok"
        } else {
            "issues found"
        }
    ));
    if let Some(checks) = payload.get("checks").and_then(Value::as_array) {
        for check in checks {
            let status = check
                .get("status")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            let name = check.get("name").and_then(Value::as_str).unwrap_or("check");
            let message = check.get("message").and_then(Value::as_str).unwrap_or("");
            lines.push(format!("[{status}] {name}: {message}"));
        }
    }
    if let Some(log) = payload.get("log_triage").filter(|value| !value.is_null()) {
        if let Some(path) = log.get("path").and_then(Value::as_str) {
            lines.push(format!("log-triage: {path}"));
        }
        if let Some(summary) = log.get("summary").and_then(Value::as_object) {
            let rendered = summary
                .iter()
                .map(|(key, value)| format!("{key}={}", value.as_u64().unwrap_or(0)))
                .collect::<Vec<_>>()
                .join(" ");
            lines.push(format!("log-triage-summary: {rendered}"));
        }
        if let Some(matches) = log.get("matches").and_then(Value::as_array) {
            for item in matches.iter().rev().take(20).rev() {
                let class = item
                    .get("class")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown");
                let text = item.get("text").and_then(Value::as_str).unwrap_or("");
                lines.push(format!("[{class}] {text}"));
            }
        }
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_real_log_fixtures() {
        assert_eq!(
            classify_log_line("MESA: error: ZINK: vkCreateInstance failed"),
            Some("benign-env")
        );
        assert_eq!(
            classify_log_line("Gdk-WARNING **: Compositor doesn't support moving popups"),
            Some("benign-env")
        );
        assert_eq!(
            classify_log_line("Gtk-WARNING **: Failed to load icon"),
            Some("limux-warning")
        );
        assert_eq!(
            classify_log_line("GLib-GIO-CRITICAL **: g_settings_schema_source_lookup"),
            Some("limux-error")
        );
    }

    #[test]
    fn strip_terminal_controls_removes_escape_sequences() {
        assert_eq!(
            strip_terminal_controls("\u{1b}[31mGtk-WARNING\u{1b}[0m\tok\u{7f}"),
            "Gtk-WARNING\tok"
        );
    }

    #[test]
    fn parses_wrapper_identity_fields() {
        let wrapper = r#"INSTALL_ROOT="/tmp/root"
export LIMUX_CHANNEL="preview:test"
exec "${INSTALL_ROOT}/libexec/limux-cli" "$@"
"#;
        assert_eq!(
            parse_wrapper_install_root(wrapper).as_deref(),
            Some("/tmp/root")
        );
        assert_eq!(
            parse_wrapper_channel(wrapper).as_deref(),
            Some("preview:test")
        );
    }

    #[cfg(unix)]
    #[test]
    fn reports_broken_launcher_target() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let prefix = tmp.path();
        let bin_dir = prefix.join("bin");
        let install_root = prefix.join("limux-reviewed/test-install");
        let target = install_root.join("bin/limux");
        fs::create_dir_all(&bin_dir).expect("bin dir");
        fs::create_dir_all(target.parent().unwrap()).expect("install bin dir");
        std::os::unix::fs::symlink(&target, bin_dir.join("limux")).expect("launcher symlink");

        let launchers = discover_launchers(prefix);

        assert_eq!(launchers.len(), 1);
        assert_eq!(launchers[0].install_root.as_ref(), Some(&install_root));
        assert!(
            launchers[0].target_error.is_some(),
            "broken wrapper target should be reported"
        );
        let payload = launchers_json(&launchers);
        assert!(
            payload[0]
                .get("target_error")
                .and_then(Value::as_str)
                .is_some(),
            "launcher JSON should include target_error"
        );
    }

    #[test]
    fn collects_nested_channel_socket_paths() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let socket = tmp.path().join("preview/default/limux.sock");
        fs::create_dir_all(socket.parent().unwrap()).expect("socket parent");
        fs::write(&socket, b"not a real socket").expect("socket placeholder");
        let mut paths = Vec::new();

        collect_socket_paths_recursive(tmp.path(), &mut paths, 3);

        assert_eq!(paths, vec![socket]);
    }
}
