mod agent_state;
mod app_config;
mod control_bridge;
mod control_registry;
mod cwd_inheritance;
mod durable_atomic;
mod ghostty_config;
mod header_status;
mod host_log;
mod keybind_editor;
mod layout_state;
mod pane;
mod runtime_lifecycle;
mod settings_editor;
mod shortcut_config;
mod split_tree;
mod state_mirror;
mod terminal;
mod window;

use adw::prelude::*;
use libadwaita as adw;
#[cfg(unix)]
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};

pub(crate) const APP_ID: &str = "dev.limux.linux";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
const LIMUX_SOCKET_ENV: &str = "LIMUX_SOCKET";
const LIMUX_SOCKET_PATH_ENV: &str = "LIMUX_SOCKET_PATH";
const LIMUX_TARGET_ENV_REMOVALS: &[&str] = &[
    "LIMUX_SOCKET",
    "LIMUX_SOCKET_PATH",
    limux_control::socket_path::LIMUX_CHANNEL_ENV,
    limux_control::socket_path::LIMUX_PREVIEW_ID_ENV,
    limux_control::socket_path::LIMUX_PROFILE_ID_ENV,
    layout_state::LIMUX_SESSION_DIR_ENV,
    "LIMUX_WORKSPACE_ID",
    "LIMUX_SURFACE_ID",
    "LIMUX_PANE_ID",
    "LIMUX_TAB_ID",
];
const LIMUX_TARGET_ID_ENV_KEYS: &[&str] = &[
    "LIMUX_WORKSPACE_ID",
    "LIMUX_SURFACE_ID",
    "LIMUX_PANE_ID",
    "LIMUX_TAB_ID",
];
const HOST_LOG_ENV: &str = "LIMUX_HOST_LOG";
const HOST_LOG_PATH_ENV: &str = "LIMUX_HOST_LOG_PATH";
const HOST_LOG_DIR_NAME: &str = "limux/logs";
const HOST_LOG_RETAINED_DIR_NAME: &str = "retained";
const HOST_LOG_MAX_ACTIVE_BYTES: u64 = 64 * 1024 * 1024;
const HOST_LOG_MAX_RETAINED_COUNT: usize = 10;
const HOST_LOG_MAX_TOTAL_BYTES: u64 = 640 * 1024 * 1024;
const HOST_LOG_MAX_WARNING_CATEGORIES: usize = 256;
/// Ceiling on how long shutdown and panic paths wait for the bounded-log drain
/// thread to catch up. A healthy drain acks in well under a tick; this only
/// bounds the pathological case (dead or wedged drain thread) so the host can
/// still exit.
#[cfg(unix)]
const HOST_LOG_FLUSH_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(2);

pub(crate) fn build_info() -> limux_control::BuildInfo {
    limux_control::BuildInfo::from_compile_env(
        option_env!("LIMUX_BUILD_SHA"),
        option_env!("LIMUX_BUILD_DIRTY"),
        option_env!("LIMUX_BUILD_PROFILE"),
    )
}

fn render_build_identity(prefix: &str, build: &limux_control::BuildInfo) -> String {
    let dirty = build
        .dirty
        .map(|value| value.to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let install_id = build.install_id.as_deref().unwrap_or("none");
    let channel = build.channel.as_deref().unwrap_or("none");
    format!(
        "{prefix} sha={} dirty={} profile={} install_id={} channel={}",
        build.sha, dirty, build.profile, install_id, channel
    )
}

fn install_panic_identity_hook(build: limux_control::BuildInfo) {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        eprintln!(
            "{}",
            render_build_identity("limux-host panic build", &build)
        );
        default_hook(info);
        // The panic message and build identity are only in the stderr pipe at
        // this point. A panicking process is usually about to die, and exiting
        // kills the drain thread with the pipe still full — so wait for the
        // drain to write them out before anything else can tear the process
        // down.
        #[cfg(unix)]
        host_log::flush_bounded_stderr(HOST_LOG_FLUSH_TIMEOUT);
    }));
}

/// Append a value to an environment variable (comma-separated), or set it.
fn append_env(key: &str, value: &str) {
    match std::env::var(key) {
        Ok(existing) if !existing.is_empty() => {
            std::env::set_var(key, format!("{existing},{value}"));
        }
        _ => {
            std::env::set_var(key, value);
        }
    }
}

// Runtime Ghostty resource-shape contract: a resources dir is valid only when
// it contains shell-integration/ AND a compiled sibling terminfo entry FILE
// (terminfo/{x/xterm-ghostty,g/ghostty} under the resources dir's PARENT,
// never nested inside it). The user-local installer stages bundles to the
// identical contract — keep the two in sync:
// scripts/user-local-install/install-user-local.sh ("Ghostty runtime resource
// resolution" section).
fn has_ghostty_terminfo(path: &Path) -> bool {
    let Some(parent) = path.parent() else {
        return false;
    };

    ["terminfo/g/ghostty", "terminfo/x/xterm-ghostty"]
        .iter()
        .any(|entry| parent.join(entry).is_file())
}

fn is_ghostty_resources_dir(path: &Path) -> bool {
    path.is_dir() && path.join("shell-integration").is_dir() && has_ghostty_terminfo(path)
}

fn ghostty_resources_candidates(exe_dir: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    for ancestor in exe_dir.ancestors() {
        candidates.push(ancestor.join("share/limux/ghostty"));
        candidates.push(ancestor.join("share/ghostty"));
        candidates.push(ancestor.join("ghostty/zig-out/share/ghostty"));
    }

    candidates.push(PathBuf::from("/usr/local/share/ghostty"));
    candidates.push(PathBuf::from("/usr/share/ghostty"));

    candidates
}

fn resolve_ghostty_resources_dir(exe_path: &Path) -> Option<PathBuf> {
    let exe_dir = exe_path.parent()?;
    ghostty_resources_candidates(exe_dir)
        .into_iter()
        .find(|path| is_ghostty_resources_dir(path))
}

fn ghostty_terminfo_dir(resources_dir: &Path) -> Option<PathBuf> {
    resources_dir.parent().map(|parent| parent.join("terminfo"))
}

fn set_env_path_if_missing_or_invalid(
    key: &str,
    path: Option<PathBuf>,
    validator: impl Fn(&Path) -> bool,
) {
    let has_valid_existing = std::env::var_os(key)
        .map(PathBuf::from)
        .is_some_and(|existing| validator(&existing));

    if has_valid_existing {
        return;
    }

    if let Some(path) = path.filter(|candidate| validator(candidate)) {
        std::env::set_var(key, path);
    }
}

fn set_ghostty_runtime_env_for_exe(exe_path: &Path) {
    let Some(resources_dir) = resolve_ghostty_resources_dir(exe_path) else {
        return;
    };

    set_env_path_if_missing_or_invalid(
        "GHOSTTY_RESOURCES_DIR",
        Some(resources_dir.clone()),
        is_ghostty_resources_dir,
    );
    set_env_path_if_missing_or_invalid(
        "TERMINFO",
        ghostty_terminfo_dir(&resources_dir),
        has_ghostty_terminfo,
    );
    set_env_path_if_missing_or_invalid(
        "GHOSTTY_SHELL_INTEGRATION_XDG_DIR",
        Some(resources_dir.join("shell-integration")),
        |candidate| candidate.is_dir(),
    );
}

fn set_ghostty_runtime_env() {
    let Some(exe_path) = std::env::current_exe().ok() else {
        return;
    };

    set_ghostty_runtime_env_for_exe(&exe_path);
}

fn sanitize_terminal_child_env() {
    // Limux is often launched from another TUI agent session. NO_COLOR belongs
    // to that launcher process, not to future shells inside this terminal app.
    std::env::remove_var("NO_COLOR");
}

fn ensure_xdg_data_dirs_defaults() {
    let mut entries = Vec::new();
    if let Some(existing) = std::env::var_os("XDG_DATA_DIRS") {
        for entry in std::env::split_paths(&existing) {
            if !entry.as_os_str().is_empty() && !entries.iter().any(|item| item == &entry) {
                entries.push(entry);
            }
        }
    }

    for fallback in ["/usr/local/share", "/usr/share"] {
        let fallback = PathBuf::from(fallback);
        if !entries.iter().any(|entry| entry == &fallback) {
            entries.push(fallback);
        }
    }

    if let Ok(value) = std::env::join_paths(entries) {
        std::env::set_var("XDG_DATA_DIRS", value);
    }
}

fn sanitize_inherited_limux_target_env_for_host() {
    let launched_from_limux_terminal = LIMUX_TARGET_ID_ENV_KEYS
        .iter()
        .any(|key| std::env::var_os(key).is_some());

    if !launched_from_limux_terminal {
        return;
    }

    for key in LIMUX_TARGET_ENV_REMOVALS {
        std::env::remove_var(key);
    }
}

fn unique_runtime_socket_path(default_path: &Path) -> PathBuf {
    let file_name = format!("limux-{}.sock", std::process::id());
    default_path
        .parent()
        .unwrap_or_else(|| Path::new("/tmp"))
        .join(file_name)
}

fn unique_runtime_session_dir(socket_path: &Path) -> PathBuf {
    let dir_name = socket_path
        .file_stem()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| std::ffi::OsStr::new("limux-runtime"));

    socket_path
        .parent()
        .unwrap_or_else(|| Path::new("/tmp"))
        .join("sessions")
        .join(dir_name)
}

#[cfg(unix)]
fn socket_accepts_connections(path: &Path) -> bool {
    UnixStream::connect(path).is_ok()
}

#[cfg(not(unix))]
fn socket_accepts_connections(_path: &Path) -> bool {
    false
}

fn socket_env_override_present() -> bool {
    [LIMUX_SOCKET_ENV, LIMUX_SOCKET_PATH_ENV]
        .iter()
        .any(|key| std::env::var_os(key).is_some_and(|value| !value.is_empty()))
}

fn session_dir_env_override_present() -> bool {
    std::env::var_os(layout_state::LIMUX_SESSION_DIR_ENV).is_some_and(|value| !value.is_empty())
}

/// Highest `auto-<n>` profile the host will allocate before giving up and
/// falling back to a throwaway session. Bounded so a leaked socket directory
/// cannot make startup scan forever.
const MAX_AUTO_PROFILE_INDEX: u32 = 64;

/// Exclusive claim on one auto-profile slot, held for the life of the process.
///
/// Probing "is this socket connectable?" is check-then-act: two hosts starting
/// together both see auto-2 free and both take it. The loser's socket bind
/// then fails — but that failure is NON-FATAL in `control_bridge::start`, so
/// it keeps running with the winner's session file and silently clobbers it on
/// save. An advisory `flock` makes the claim atomic instead, and the kernel
/// releases it on exit or crash, so a dead host never burns a slot.
///
/// Claims live in the runtime dir, not the data dir, so they cannot show up as
/// profiles in `limux profile list` and never survive a reboot.
struct AutoProfileClaim(std::fs::File);

impl Drop for AutoProfileClaim {
    fn drop(&mut self) {
        use std::os::fd::AsRawFd;
        // Closing the descriptor would release the lock on its own; unlocking
        // explicitly matches `durable_atomic::FileLock` and keeps the release
        // point obvious at the call site.
        // SAFETY: the descriptor remains owned by self until this drop completes.
        unsafe {
            libc::flock(self.0.as_raw_fd(), libc::LOCK_UN);
        }
    }
}

impl AutoProfileClaim {
    fn try_acquire(socket_path: &Path) -> Option<Self> {
        use std::os::fd::AsRawFd;
        use std::os::unix::fs::OpenOptionsExt;

        let dir = socket_path.parent()?;
        std::fs::create_dir_all(dir).ok()?;
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            // O_CLOEXEC is REQUIRED, not optional: flock binds to the open file
            // description, so any child inheriting this fd keeps the slot held
            // after this host dies. Limux spawns a shell per pane, and Rust's
            // `Command` does not close arbitrary inherited fds. Rust's
            // OpenOptions already opens with O_CLOEXEC on Linux — it is named
            // here so the requirement is visible at the call site, not because
            // it adds behavior. Do not "clean up" this comment into the flag.
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(dir.join(".claim.lock"))
            .ok()?;
        // SAFETY: flock only reads the live descriptor and does not retain it.
        // LOCK_NB so a slot held by another host is skipped, never waited on.
        let acquired = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } == 0;
        acquired.then_some(Self(file))
    }
}

/// Claims are intentionally never released before exit: the slot must stay
/// reserved for as long as this host is alive. A host claims exactly once, so
/// in production this holds a single entry; it is a `Vec` rather than a
/// `OnceLock` only so tests (which share one process) can reset between cases
/// without ever dropping a claim early in the real path.
static HELD_AUTO_PROFILE_CLAIMS: std::sync::Mutex<Vec<AutoProfileClaim>> =
    std::sync::Mutex::new(Vec::new());

/// First free `auto-<n>` profile, or `None` if every slot up to the cap is
/// already claimed.
///
/// Numbering starts at 2 because the un-namespaced default session is
/// effectively profile 1 — the one a lone `limux` restores.
fn next_free_auto_profile() -> Option<limux_control::socket_path::RuntimeChannel> {
    for index in 2..=MAX_AUTO_PROFILE_INDEX {
        let channel = limux_control::socket_path::RuntimeChannel::Profile(format!(
            "{}{index}",
            limux_control::socket_path::RuntimeChannel::AUTO_PROFILE_PREFIX
        ));
        let socket = channel.socket_path();
        // Cheap reject first: a slot with a live socket is definitely taken.
        if socket_accepts_connections(&socket) {
            continue;
        }
        // Then claim it atomically. Only the flock winner proceeds, so two
        // hosts racing the same slot cannot both adopt its session file.
        let Some(claim) = AutoProfileClaim::try_acquire(&socket) else {
            continue;
        };
        // Hold the claim for the life of the process.
        match HELD_AUTO_PROFILE_CLAIMS.lock() {
            Ok(mut held) => held.push(claim),
            // A poisoned lock means another thread panicked mid-claim. Dropping
            // `claim` here would release the slot we just won, so refuse the
            // auto-profile path entirely and let the caller fall back.
            Err(_) => return None,
        }
        return Some(channel);
    }
    None
}

fn ensure_runtime_socket_does_not_collide() {
    if socket_env_override_present() {
        return;
    }

    let default_path = limux_control::socket_path::resolve_socket_path(
        None,
        limux_control::socket_path::SocketMode::Runtime,
    );
    if !socket_accepts_connections(&default_path) {
        return;
    }

    // A second concurrent Limux used to get a throwaway session directory
    // under the runtime dir, so everything it had open was discarded on exit.
    // Give it a persistent auto-profile instead; `limux profile list` shows
    // these as `auto` and `limux profile rm` prunes them.
    if let Some(channel) = next_free_auto_profile() {
        eprintln!(
            "limux: default control socket already in use ({}); using profile {} (persistent; see `limux profile list`)",
            default_path.display(),
            channel.label()
        );
        // Setting the channel alone repoints BOTH the control socket and the
        // session file, so the two can never drift apart. An explicit
        // LIMUX_SESSION_DIR still wins inside persistence_dir().
        std::env::set_var(
            limux_control::socket_path::LIMUX_CHANNEL_ENV,
            channel.env_value(),
        );
        std::env::remove_var(limux_control::socket_path::LIMUX_PROFILE_ID_ENV);
        return;
    }

    // Every auto slot is live. Fall back to the historical throwaway session
    // so startup still succeeds rather than refusing to open a window.
    let socket_path = unique_runtime_socket_path(&default_path);
    let session_dir = unique_runtime_session_dir(&socket_path);
    eprintln!(
        "limux: default control socket already in use ({}) and all auto profiles are running; using throwaway session {}",
        default_path.display(),
        socket_path.display()
    );
    std::env::set_var(LIMUX_SOCKET_ENV, &socket_path);
    if !session_dir_env_override_present() {
        std::env::set_var(layout_state::LIMUX_SESSION_DIR_ENV, session_dir);
    }
}

fn host_log_path() -> Option<PathBuf> {
    std::env::var_os(HOST_LOG_PATH_ENV)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            dirs::state_dir().map(|dir| {
                dir.join(HOST_LOG_DIR_NAME)
                    .join(limux_control::DEFAULT_HOST_LOG_FILE_NAME)
            })
        })
}

#[cfg(unix)]
fn install_host_stderr_log() -> Result<Option<PathBuf>, String> {
    if std::env::var_os(HOST_LOG_ENV).is_some_and(|value| value == "off" || value == "0") {
        return Ok(None);
    }

    let Some(path) = host_log_path() else {
        return Ok(None);
    };
    let retained_dir = path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(HOST_LOG_RETAINED_DIR_NAME);
    let config = host_log::HostLogConfig {
        active_path: path,
        retained_dir,
        max_active_bytes: HOST_LOG_MAX_ACTIVE_BYTES,
        max_retained_count: HOST_LOG_MAX_RETAINED_COUNT,
        max_total_bytes: HOST_LOG_MAX_TOTAL_BYTES,
        max_warning_categories: HOST_LOG_MAX_WARNING_CATEGORIES,
    };
    let sequence = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| format!("system clock is before Unix epoch: {error}"))?
        .as_nanos();
    host_log::install_bounded_stderr(&config, sequence)
}

#[cfg(not(unix))]
fn install_host_stderr_log() -> Result<Option<PathBuf>, String> {
    Ok(None)
}

fn gtk_runtime_version() -> (u32, u32, u32) {
    unsafe {
        (
            gtk4::ffi::gtk_get_major_version(),
            gtk4::ffi::gtk_get_minor_version(),
            gtk4::ffi::gtk_get_micro_version(),
        )
    }
}

fn gtk_runtime_at_least(major: u32, minor: u32, micro: u32) -> bool {
    gtk_runtime_version() >= (major, minor, micro)
}

fn main() {
    // Handle --version flag
    if std::env::args().any(|a| a == "--version" || a == "-v") {
        println!(
            "{}",
            limux_control::render_version_line("limux-host", VERSION, &build_info())
        );
        return;
    }

    let build = build_info();
    install_panic_identity_hook(build.clone());

    ensure_xdg_data_dirs_defaults();
    sanitize_inherited_limux_target_env_for_host();

    if let Err(err) = install_host_stderr_log() {
        eprintln!("limux: failed to initialize host log: {err}");
    }
    eprintln!(
        "{}",
        render_build_identity("limux-host start build", &build)
    );
    ensure_runtime_socket_does_not_collide();

    // Ghostty requires desktop OpenGL, not GLES. Must set the GTK renderer
    // environment before GTK initializes, and the exact knobs differ by GTK
    // runtime version. Match Ghostty's GTK logic closely here so modern GTK
    // doesn't warn about removed GDK_DEBUG values.
    if gtk_runtime_at_least(4, 16, 0) {
        append_env("GDK_DISABLE", "gles-api,vulkan");
    } else if gtk_runtime_at_least(4, 14, 0) {
        append_env("GDK_DEBUG", "gl-disable-gles,vulkan-disable");
    } else {
        append_env("GDK_DEBUG", "vulkan-disable");
    }

    // Embedded Ghostty needs a resources directory to resolve named themes,
    // terminfo, and shell integration. Prefer Limux-bundled resources but
    // fall back to common system Ghostty install locations.
    set_ghostty_runtime_env();
    sanitize_terminal_child_env();

    // WebKitGTK's bubblewrap sandbox requires unprivileged user namespaces,
    // which may not be available. Disable it to prevent crashes on launch.
    if std::env::var("WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS").is_err() {
        std::env::set_var("WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS", "1");
    }

    // Initialize Ghostty before GTK app starts
    terminal::init_ghostty();

    let app = adw::Application::builder()
        .application_id(APP_ID)
        .flags(adw::gio::ApplicationFlags::NON_UNIQUE)
        .build();

    app.connect_activate(move |app| {
        window::build_window(app);
    });
    // No flush call here on purpose. Exiting kills the bounded-log drain
    // thread and discards the pipe buffer, but a call sited after `app.run()`
    // does not reliably cover that: measured headless, GTK terminates the
    // process from inside `app.run()`, which never returns. The flush is
    // registered with `atexit` inside `install_bounded_stderr` instead, so it
    // covers this path, GTK's internal exit, and any `std::process::exit`.
    app.run();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[cfg(unix)]
    use std::os::unix::net::UnixListener;
    use std::sync::Mutex;
    use std::time::{SystemTime, UNIX_EPOCH};

    static GHOSTTY_ENV_LOCK: Mutex<()> = Mutex::new(());

    struct GhosttyEnvGuard {
        resources: Option<std::ffi::OsString>,
        terminfo: Option<std::ffi::OsString>,
        shell_integration: Option<std::ffi::OsString>,
    }

    impl GhosttyEnvGuard {
        fn capture() -> Self {
            Self {
                resources: std::env::var_os("GHOSTTY_RESOURCES_DIR"),
                terminfo: std::env::var_os("TERMINFO"),
                shell_integration: std::env::var_os("GHOSTTY_SHELL_INTEGRATION_XDG_DIR"),
            }
        }
    }

    impl Drop for GhosttyEnvGuard {
        fn drop(&mut self) {
            match self.resources.take() {
                Some(value) => std::env::set_var("GHOSTTY_RESOURCES_DIR", value),
                None => std::env::remove_var("GHOSTTY_RESOURCES_DIR"),
            }
            match self.terminfo.take() {
                Some(value) => std::env::set_var("TERMINFO", value),
                None => std::env::remove_var("TERMINFO"),
            }
            match self.shell_integration.take() {
                Some(value) => std::env::set_var("GHOSTTY_SHELL_INTEGRATION_XDG_DIR", value),
                None => std::env::remove_var("GHOSTTY_SHELL_INTEGRATION_XDG_DIR"),
            }
        }
    }

    struct EnvVarGuard {
        key: &'static str,
        old: Option<std::ffi::OsString>,
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: Option<impl AsRef<std::ffi::OsStr>>) -> Self {
            let old = std::env::var_os(key);
            match value {
                Some(value) => std::env::set_var(key, value.as_ref()),
                None => std::env::remove_var(key),
            }
            Self { key, old }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            match &self.old {
                Some(value) => std::env::set_var(self.key, value),
                None => std::env::remove_var(self.key),
            }
        }
    }

    fn with_ghostty_env<R>(test: impl FnOnce() -> R) -> R {
        let _lock = GHOSTTY_ENV_LOCK
            .lock()
            .expect("ghostty env test lock poisoned");
        let _guard = GhosttyEnvGuard::capture();
        test()
    }

    fn temp_path(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_nanos();
        std::env::temp_dir().join(format!("limux-{label}-{}-{nanos}", std::process::id()))
    }

    #[test]
    fn sanitize_terminal_child_env_removes_no_color() {
        let original = std::env::var_os("NO_COLOR");
        std::env::set_var("NO_COLOR", "1");

        sanitize_terminal_child_env();

        assert!(std::env::var_os("NO_COLOR").is_none());
        match original {
            Some(value) => std::env::set_var("NO_COLOR", value),
            None => std::env::remove_var("NO_COLOR"),
        }
    }

    #[test]
    fn ensure_xdg_data_dirs_defaults_preserves_inherited_and_adds_system_dirs() {
        let _lock = GHOSTTY_ENV_LOCK
            .lock()
            .expect("ghostty env test lock poisoned");
        let original = std::env::var_os("XDG_DATA_DIRS");
        std::env::set_var("XDG_DATA_DIRS", "/tmp/limux-private-share");

        ensure_xdg_data_dirs_defaults();

        let updated = std::env::var_os("XDG_DATA_DIRS").expect("updated xdg data dirs");
        let entries = std::env::split_paths(&updated).collect::<Vec<_>>();
        assert!(entries.contains(&PathBuf::from("/tmp/limux-private-share")));
        assert!(entries.contains(&PathBuf::from("/usr/local/share")));
        assert!(entries.contains(&PathBuf::from("/usr/share")));

        match original {
            Some(value) => std::env::set_var("XDG_DATA_DIRS", value),
            None => std::env::remove_var("XDG_DATA_DIRS"),
        }
    }

    #[test]
    fn host_startup_clears_inherited_pane_target_env() {
        let _lock = GHOSTTY_ENV_LOCK
            .lock()
            .expect("ghostty env test lock poisoned");
        let _socket = EnvVarGuard::set(LIMUX_SOCKET_ENV, Some("/tmp/old-runtime.sock"));
        let _socket_path = EnvVarGuard::set(LIMUX_SOCKET_PATH_ENV, Some("/tmp/old-runtime.sock"));
        let _channel = EnvVarGuard::set(
            limux_control::socket_path::LIMUX_CHANNEL_ENV,
            Some("preview:old"),
        );
        let _preview_id = EnvVarGuard::set(
            limux_control::socket_path::LIMUX_PREVIEW_ID_ENV,
            Some("old"),
        );
        let _session_dir = EnvVarGuard::set(
            layout_state::LIMUX_SESSION_DIR_ENV,
            Some("/tmp/old-runtime-session"),
        );
        let _workspace = EnvVarGuard::set("LIMUX_WORKSPACE_ID", Some("workspace-old"));
        let _surface = EnvVarGuard::set("LIMUX_SURFACE_ID", Some("1:terminal-0"));
        let _pane = EnvVarGuard::set("LIMUX_PANE_ID", Some("1"));
        let _tab = EnvVarGuard::set("LIMUX_TAB_ID", Some("terminal-0"));

        sanitize_inherited_limux_target_env_for_host();

        for key in LIMUX_TARGET_ENV_REMOVALS {
            assert!(
                std::env::var_os(key).is_none(),
                "expected {key} to be cleared"
            );
        }
    }

    /// Binds every auto-profile socket in `indices`, returning the listeners
    /// so the caller keeps them alive for the duration of the test.
    #[cfg(unix)]
    fn bind_auto_profile_sockets(indices: impl IntoIterator<Item = u32>) -> Vec<UnixListener> {
        indices
            .into_iter()
            .map(|index| {
                let path = limux_control::socket_path::RuntimeChannel::Profile(format!(
                    "{}{index}",
                    limux_control::socket_path::RuntimeChannel::AUTO_PROFILE_PREFIX
                ))
                .socket_path();
                fs::create_dir_all(path.parent().expect("auto profile socket parent"))
                    .expect("create auto profile socket parent");
                UnixListener::bind(&path).expect("bind auto profile socket")
            })
            .collect()
    }

    /// A second Limux must land on a *persistent* auto profile, not the old
    /// throwaway session directory. Reverting the auto-profile branch in
    /// `ensure_runtime_socket_does_not_collide` fails here.
    #[cfg(unix)]
    #[test]
    fn runtime_socket_uses_persistent_auto_profile_when_default_socket_is_live() {
        let _lock = GHOSTTY_ENV_LOCK
            .lock()
            .expect("ghostty env test lock poisoned");
        let runtime_dir = tempfile::tempdir().expect("runtime tempdir");
        let _xdg = EnvVarGuard::set("XDG_RUNTIME_DIR", Some(runtime_dir.path()));
        let _socket = EnvVarGuard::set(LIMUX_SOCKET_ENV, Option::<&str>::None);
        let _socket_path = EnvVarGuard::set(LIMUX_SOCKET_PATH_ENV, Option::<&str>::None);
        let _channel = EnvVarGuard::set(
            limux_control::socket_path::LIMUX_CHANNEL_ENV,
            Option::<&str>::None,
        );
        let _preview_id = EnvVarGuard::set(
            limux_control::socket_path::LIMUX_PREVIEW_ID_ENV,
            Option::<&str>::None,
        );
        let _profile_id = EnvVarGuard::set(
            limux_control::socket_path::LIMUX_PROFILE_ID_ENV,
            Option::<&str>::None,
        );
        let _session_dir =
            EnvVarGuard::set(layout_state::LIMUX_SESSION_DIR_ENV, Option::<&str>::None);

        let default_path = limux_control::socket_path::SocketMode::default_for(
            limux_control::socket_path::SocketMode::Runtime,
        );
        fs::create_dir_all(default_path.parent().expect("default socket parent"))
            .expect("create socket parent");
        let _listener = UnixListener::bind(&default_path).expect("bind default socket");

        ensure_runtime_socket_does_not_collide();

        assert_eq!(
            std::env::var_os(limux_control::socket_path::LIMUX_CHANNEL_ENV),
            Some(std::ffi::OsString::from("profile:auto-2")),
            "second instance must claim the first free auto profile"
        );
        // The throwaway path must NOT have been taken: no forced session dir,
        // no pid-named socket. Those are what made state disappear on exit.
        assert!(
            std::env::var_os(layout_state::LIMUX_SESSION_DIR_ENV).is_none(),
            "auto profile must persist, not fall back to a throwaway session dir"
        );
        assert!(
            std::env::var_os(LIMUX_SOCKET_ENV).is_none(),
            "auto profile repoints via the channel, not an explicit socket override"
        );
        // And the state it resolves to must be the persistent profile dir.
        assert_eq!(
            layout_state::persistence_dir(),
            limux_control::session_paths::profile_persistence_dir("auto-2")
        );
    }

    /// Allocation must skip auto slots that are already served by a live host,
    /// otherwise two instances would fight over one session file.
    #[cfg(unix)]
    #[test]
    fn runtime_socket_skips_auto_profiles_already_in_use() {
        let _lock = GHOSTTY_ENV_LOCK
            .lock()
            .expect("ghostty env test lock poisoned");
        let runtime_dir = tempfile::tempdir().expect("runtime tempdir");
        let _xdg = EnvVarGuard::set("XDG_RUNTIME_DIR", Some(runtime_dir.path()));
        let _socket = EnvVarGuard::set(LIMUX_SOCKET_ENV, Option::<&str>::None);
        let _socket_path = EnvVarGuard::set(LIMUX_SOCKET_PATH_ENV, Option::<&str>::None);
        let _channel = EnvVarGuard::set(
            limux_control::socket_path::LIMUX_CHANNEL_ENV,
            Option::<&str>::None,
        );
        let _preview_id = EnvVarGuard::set(
            limux_control::socket_path::LIMUX_PREVIEW_ID_ENV,
            Option::<&str>::None,
        );
        let _profile_id = EnvVarGuard::set(
            limux_control::socket_path::LIMUX_PROFILE_ID_ENV,
            Option::<&str>::None,
        );
        let _session_dir =
            EnvVarGuard::set(layout_state::LIMUX_SESSION_DIR_ENV, Option::<&str>::None);

        let default_path = limux_control::socket_path::SocketMode::default_for(
            limux_control::socket_path::SocketMode::Runtime,
        );
        fs::create_dir_all(default_path.parent().expect("default socket parent"))
            .expect("create socket parent");
        let _listener = UnixListener::bind(&default_path).expect("bind default socket");
        let _busy = bind_auto_profile_sockets(2..=4);

        ensure_runtime_socket_does_not_collide();

        assert_eq!(
            std::env::var_os(limux_control::socket_path::LIMUX_CHANNEL_ENV),
            Some(std::ffi::OsString::from("profile:auto-5")),
            "auto-2..4 are live, so the next instance must take auto-5"
        );
    }

    /// The allocator must be atomic, not check-then-act.
    ///
    /// Socket-probing alone is a TOCTOU: two hosts starting together both see
    /// auto-2 with no live socket and both adopt it. The loser's bind then
    /// fails, but that failure is non-fatal in `control_bridge::start`, so it
    /// keeps running against the winner's `session.json` and clobbers it on
    /// save — silently losing a window's workspaces, the exact failure this
    /// feature exists to prevent.
    ///
    /// Here the "other host" holds only the claim lock, with NO socket bound —
    /// the precise window socket-probing cannot see. Reverting the claim to a
    /// bare probe makes this fail.
    #[cfg(unix)]
    #[test]
    fn auto_profile_claim_is_atomic_not_check_then_act() {
        use std::os::fd::AsRawFd;

        let _lock = GHOSTTY_ENV_LOCK
            .lock()
            .expect("ghostty env test lock poisoned");
        let runtime_dir = tempfile::tempdir().expect("runtime tempdir");
        let _xdg = EnvVarGuard::set("XDG_RUNTIME_DIR", Some(runtime_dir.path()));

        let auto2 = limux_control::socket_path::RuntimeChannel::Profile("auto-2".to_string());
        let socket = auto2.socket_path();
        assert!(
            !socket_accepts_connections(&socket),
            "precondition: auto-2 has no live socket, so a naive probe sees it as free"
        );

        // Simulate the racing host mid-startup: claim held, socket not yet bound.
        let rival = AutoProfileClaim::try_acquire(&socket).expect("rival claims auto-2");

        let picked = next_free_auto_profile().expect("an auto profile must still be available");
        assert_eq!(
            picked.profile_id(),
            Some("auto-3"),
            "auto-2 is claimed by a starting host; allocation must move on"
        );

        // And the claim is genuinely exclusive at the OS level.
        let contender =
            std::fs::File::open(socket.parent().expect("socket parent").join(".claim.lock"))
                .expect("open claim lock");
        // SAFETY: flock only reads the live descriptor and does not retain it.
        let taken =
            unsafe { libc::flock(contender.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0;
        assert!(taken, "a held auto-profile claim must block a second flock");

        drop(rival);
    }

    /// The claim must not leak into spawned children.
    ///
    /// `flock` is tied to the open file *description*, not the descriptor, so
    /// any child inheriting the claim fd keeps the slot held after this host
    /// dies — leaking that auto profile until every such child exits. Limux
    /// spawns a shell per pane and several helper processes, and Rust's
    /// `Command` does NOT close arbitrary inherited fds.
    ///
    /// Note on what this does and does not guard: deleting the explicit
    /// `libc::O_CLOEXEC` does NOT fail this test, because Rust's `OpenOptions`
    /// already opens with `O_CLOEXEC` on Linux — that redundancy was measured,
    /// not assumed. What this test does catch is a genuine leak: actively
    /// clearing `FD_CLOEXEC` fails it. So it guards the close-on-exec
    /// *behavior* the claim depends on, whatever supplies it.
    #[cfg(unix)]
    #[test]
    fn auto_profile_claim_is_not_inherited_by_spawned_children() {
        use std::os::fd::AsRawFd;

        let _lock = GHOSTTY_ENV_LOCK
            .lock()
            .expect("ghostty env test lock poisoned");
        let runtime_dir = tempfile::tempdir().expect("runtime tempdir");
        let _xdg = EnvVarGuard::set("XDG_RUNTIME_DIR", Some(runtime_dir.path()));

        let socket =
            limux_control::socket_path::RuntimeChannel::Profile("auto-2".to_string()).socket_path();
        let claim = AutoProfileClaim::try_acquire(&socket).expect("claim auto-2");

        // Mechanism check: the descriptor is marked close-on-exec.
        // SAFETY: fcntl(F_GETFD) only reads flags from the live descriptor.
        let flags = unsafe { libc::fcntl(claim.0.as_raw_fd(), libc::F_GETFD) };
        assert!(flags >= 0, "F_GETFD failed");
        assert_eq!(
            flags & libc::FD_CLOEXEC,
            libc::FD_CLOEXEC,
            "claim fd must be close-on-exec"
        );

        // Behavioral check: a child spawned while the claim is held, which
        // outlives it, must not keep the slot locked.
        let mut child = std::process::Command::new("sleep")
            .arg("30")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("spawn child holding any inherited fds");
        drop(claim);

        let reacquired = AutoProfileClaim::try_acquire(&socket);
        let released = reacquired.is_some();
        let _ = child.kill();
        let _ = child.wait();

        assert!(
            released,
            "a spawned child inherited the claim fd and is still holding the slot"
        );
    }

    /// When every auto slot is live, startup must still succeed by falling
    /// back to the historical throwaway session rather than refusing to open.
    #[cfg(unix)]
    #[test]
    fn runtime_socket_falls_back_to_throwaway_when_all_auto_profiles_are_live() {
        let _lock = GHOSTTY_ENV_LOCK
            .lock()
            .expect("ghostty env test lock poisoned");
        let runtime_dir = tempfile::tempdir().expect("runtime tempdir");
        let _xdg = EnvVarGuard::set("XDG_RUNTIME_DIR", Some(runtime_dir.path()));
        let _socket = EnvVarGuard::set(LIMUX_SOCKET_ENV, Option::<&str>::None);
        let _socket_path = EnvVarGuard::set(LIMUX_SOCKET_PATH_ENV, Option::<&str>::None);
        let _channel = EnvVarGuard::set(
            limux_control::socket_path::LIMUX_CHANNEL_ENV,
            Option::<&str>::None,
        );
        let _preview_id = EnvVarGuard::set(
            limux_control::socket_path::LIMUX_PREVIEW_ID_ENV,
            Option::<&str>::None,
        );
        let _profile_id = EnvVarGuard::set(
            limux_control::socket_path::LIMUX_PROFILE_ID_ENV,
            Option::<&str>::None,
        );
        let _session_dir =
            EnvVarGuard::set(layout_state::LIMUX_SESSION_DIR_ENV, Option::<&str>::None);

        let default_path = limux_control::socket_path::SocketMode::default_for(
            limux_control::socket_path::SocketMode::Runtime,
        );
        fs::create_dir_all(default_path.parent().expect("default socket parent"))
            .expect("create socket parent");
        let _listener = UnixListener::bind(&default_path).expect("bind default socket");
        let _busy = bind_auto_profile_sockets(2..=MAX_AUTO_PROFILE_INDEX);

        ensure_runtime_socket_does_not_collide();

        let expected_socket = unique_runtime_socket_path(&default_path);
        assert_eq!(
            std::env::var_os(LIMUX_SOCKET_ENV),
            Some(expected_socket.clone().into_os_string())
        );
        assert_eq!(
            std::env::var_os(layout_state::LIMUX_SESSION_DIR_ENV),
            Some(unique_runtime_session_dir(&expected_socket).into_os_string())
        );
    }

    #[cfg(unix)]
    #[test]
    fn runtime_socket_collision_check_uses_channel_socket() {
        let _lock = GHOSTTY_ENV_LOCK
            .lock()
            .expect("ghostty env test lock poisoned");
        let runtime_dir = tempfile::tempdir().expect("runtime tempdir");
        let _xdg = EnvVarGuard::set("XDG_RUNTIME_DIR", Some(runtime_dir.path()));
        let _socket = EnvVarGuard::set(LIMUX_SOCKET_ENV, Option::<&str>::None);
        let _socket_path = EnvVarGuard::set(LIMUX_SOCKET_PATH_ENV, Option::<&str>::None);
        let _channel = EnvVarGuard::set(
            limux_control::socket_path::LIMUX_CHANNEL_ENV,
            Some("preview:branch"),
        );
        let _preview_id = EnvVarGuard::set(
            limux_control::socket_path::LIMUX_PREVIEW_ID_ENV,
            Option::<&str>::None,
        );
        let _session_dir =
            EnvVarGuard::set(layout_state::LIMUX_SESSION_DIR_ENV, Option::<&str>::None);

        let legacy_path = limux_control::socket_path::SocketMode::default_for(
            limux_control::socket_path::SocketMode::Runtime,
        );
        fs::create_dir_all(legacy_path.parent().expect("legacy socket parent"))
            .expect("create legacy socket parent");
        let _listener = UnixListener::bind(&legacy_path).expect("bind legacy socket");

        ensure_runtime_socket_does_not_collide();

        assert!(
            std::env::var_os(LIMUX_SOCKET_ENV).is_none(),
            "legacy socket collision must not override explicit preview channel"
        );
        assert!(
            std::env::var_os(layout_state::LIMUX_SESSION_DIR_ENV).is_none(),
            "legacy socket collision must not force a fallback session for preview channel"
        );
    }

    #[cfg(unix)]
    #[test]
    fn runtime_socket_treats_empty_socket_env_as_unset() {
        let _lock = GHOSTTY_ENV_LOCK
            .lock()
            .expect("ghostty env test lock poisoned");
        let runtime_dir = tempfile::tempdir().expect("runtime tempdir");
        let _xdg = EnvVarGuard::set("XDG_RUNTIME_DIR", Some(runtime_dir.path()));
        let _socket = EnvVarGuard::set(LIMUX_SOCKET_ENV, Some(""));
        let _socket_path = EnvVarGuard::set(LIMUX_SOCKET_PATH_ENV, Some(""));
        let _channel = EnvVarGuard::set(
            limux_control::socket_path::LIMUX_CHANNEL_ENV,
            Option::<&str>::None,
        );
        let _preview_id = EnvVarGuard::set(
            limux_control::socket_path::LIMUX_PREVIEW_ID_ENV,
            Option::<&str>::None,
        );
        let _profile_id = EnvVarGuard::set(
            limux_control::socket_path::LIMUX_PROFILE_ID_ENV,
            Option::<&str>::None,
        );
        let _session_dir =
            EnvVarGuard::set(layout_state::LIMUX_SESSION_DIR_ENV, Option::<&str>::None);

        let default_path = limux_control::socket_path::SocketMode::default_for(
            limux_control::socket_path::SocketMode::Runtime,
        );
        fs::create_dir_all(default_path.parent().expect("default socket parent"))
            .expect("create socket parent");
        let _listener = UnixListener::bind(&default_path).expect("bind default socket");

        ensure_runtime_socket_does_not_collide();

        // Empty is treated as unset, so collision handling still runs and
        // claims an auto profile rather than short-circuiting.
        assert_eq!(
            std::env::var_os(limux_control::socket_path::LIMUX_CHANNEL_ENV),
            Some(std::ffi::OsString::from("profile:auto-2"))
        );
    }

    #[cfg(unix)]
    #[test]
    fn runtime_socket_preserves_explicit_session_dir_env() {
        let _lock = GHOSTTY_ENV_LOCK
            .lock()
            .expect("ghostty env test lock poisoned");
        let runtime_dir = tempfile::tempdir().expect("runtime tempdir");
        let _xdg = EnvVarGuard::set("XDG_RUNTIME_DIR", Some(runtime_dir.path()));
        let _socket = EnvVarGuard::set(LIMUX_SOCKET_ENV, Option::<&str>::None);
        let _socket_path = EnvVarGuard::set(LIMUX_SOCKET_PATH_ENV, Option::<&str>::None);
        let _channel = EnvVarGuard::set(
            limux_control::socket_path::LIMUX_CHANNEL_ENV,
            Option::<&str>::None,
        );
        let _preview_id = EnvVarGuard::set(
            limux_control::socket_path::LIMUX_PREVIEW_ID_ENV,
            Option::<&str>::None,
        );
        let _session_dir = EnvVarGuard::set(
            layout_state::LIMUX_SESSION_DIR_ENV,
            Some("/tmp/manual-limux-session"),
        );

        let default_path = limux_control::socket_path::SocketMode::default_for(
            limux_control::socket_path::SocketMode::Runtime,
        );
        fs::create_dir_all(default_path.parent().expect("default socket parent"))
            .expect("create socket parent");
        let _listener = UnixListener::bind(&default_path).expect("bind default socket");

        ensure_runtime_socket_does_not_collide();

        assert_eq!(
            std::env::var_os(layout_state::LIMUX_SESSION_DIR_ENV),
            Some(std::ffi::OsString::from("/tmp/manual-limux-session"))
        );
    }

    #[test]
    fn runtime_socket_preserves_explicit_socket_env() {
        let _lock = GHOSTTY_ENV_LOCK
            .lock()
            .expect("ghostty env test lock poisoned");
        let _socket = EnvVarGuard::set(LIMUX_SOCKET_ENV, Some("/tmp/manual-limux.sock"));
        let _socket_path = EnvVarGuard::set(LIMUX_SOCKET_PATH_ENV, Option::<&str>::None);
        let _channel = EnvVarGuard::set(
            limux_control::socket_path::LIMUX_CHANNEL_ENV,
            Option::<&str>::None,
        );
        let _preview_id = EnvVarGuard::set(
            limux_control::socket_path::LIMUX_PREVIEW_ID_ENV,
            Option::<&str>::None,
        );
        let _session_dir =
            EnvVarGuard::set(layout_state::LIMUX_SESSION_DIR_ENV, Option::<&str>::None);

        ensure_runtime_socket_does_not_collide();

        assert_eq!(
            std::env::var_os(LIMUX_SOCKET_ENV),
            Some(std::ffi::OsString::from("/tmp/manual-limux.sock"))
        );
        assert!(std::env::var_os(layout_state::LIMUX_SESSION_DIR_ENV).is_none());
    }

    #[test]
    fn resolves_app_specific_bundled_resources_next_to_executable() {
        let root = temp_path("resources");
        let exe_dir = root.join("bin");
        let themes_dir = root.join("share/limux/ghostty/themes");
        let shell_integration_dir = root.join("share/limux/ghostty/shell-integration");
        let terminfo_file = root.join("share/limux/terminfo/g/ghostty");
        fs::create_dir_all(&exe_dir).unwrap();
        fs::create_dir_all(&themes_dir).unwrap();
        fs::create_dir_all(&shell_integration_dir).unwrap();
        fs::create_dir_all(terminfo_file.parent().unwrap()).unwrap();
        fs::write(&terminfo_file, b"ghostty").unwrap();

        let exe = exe_dir.join("limux");
        let resolved = resolve_ghostty_resources_dir(&exe).unwrap();
        assert_eq!(resolved, root.join("share/limux/ghostty"));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn resolves_resources_without_optional_themes() {
        let root = temp_path("resources-no-themes");
        let exe_dir = root.join("bin");
        let shell_integration_dir = root.join("share/limux/ghostty/shell-integration");
        let terminfo_file = root.join("share/limux/terminfo/g/ghostty");
        fs::create_dir_all(&exe_dir).unwrap();
        fs::create_dir_all(&shell_integration_dir).unwrap();
        fs::create_dir_all(terminfo_file.parent().unwrap()).unwrap();
        fs::write(&terminfo_file, b"ghostty").unwrap();

        let exe = exe_dir.join("limux");
        let resolved = resolve_ghostty_resources_dir(&exe).unwrap();
        assert_eq!(resolved, root.join("share/limux/ghostty"));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn resolves_dev_checkout_resources_from_target_binary() {
        let root = temp_path("dev-resources");
        let exe_dir = root.join("target/release");
        let themes_dir = root.join("ghostty/zig-out/share/ghostty/themes");
        let shell_integration_dir = root.join("ghostty/zig-out/share/ghostty/shell-integration");
        let terminfo_file = root.join("ghostty/zig-out/share/terminfo/x/xterm-ghostty");
        fs::create_dir_all(&exe_dir).unwrap();
        fs::create_dir_all(&themes_dir).unwrap();
        fs::create_dir_all(&shell_integration_dir).unwrap();
        fs::create_dir_all(terminfo_file.parent().unwrap()).unwrap();
        fs::write(&terminfo_file, b"xterm-ghostty").unwrap();

        let exe = exe_dir.join("limux");
        let resolved = resolve_ghostty_resources_dir(&exe).unwrap();
        assert_eq!(resolved, root.join("ghostty/zig-out/share/ghostty"));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_resource_dirs_without_sibling_terminfo() {
        let root = temp_path("shell-integration-only");
        let exe_dir = root.join("target/release");
        let resources_dir = root.join("ghostty/src");
        let shell_integration_dir = resources_dir.join("shell-integration");
        fs::create_dir_all(&exe_dir).unwrap();
        fs::create_dir_all(&shell_integration_dir).unwrap();

        let exe = exe_dir.join("limux");
        assert!(resolve_ghostty_resources_dir(&exe).is_none());
        assert!(!is_ghostty_resources_dir(&resources_dir));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn resource_env_ignores_shell_integration_without_terminfo() {
        with_ghostty_env(|| {
            let _resources = EnvVarGuard::set("GHOSTTY_RESOURCES_DIR", None::<&str>);
            let _terminfo = EnvVarGuard::set("TERMINFO", None::<&str>);
            let _shell_integration =
                EnvVarGuard::set("GHOSTTY_SHELL_INTEGRATION_XDG_DIR", None::<&str>);
            let root = temp_path("env-shell-only");
            let exe_dir = root.join("target/release");
            let resources_dir = root.join("ghostty/src");
            let shell_integration_dir = resources_dir.join("shell-integration");
            fs::create_dir_all(&exe_dir).unwrap();
            fs::create_dir_all(&shell_integration_dir).unwrap();

            let exe = exe_dir.join("limux");
            set_ghostty_runtime_env_for_exe(&exe);

            assert!(std::env::var_os("GHOSTTY_RESOURCES_DIR").is_none());
            assert!(std::env::var_os("GHOSTTY_SHELL_INTEGRATION_XDG_DIR").is_none());
            assert!(std::env::var_os("TERMINFO").is_none());

            fs::remove_dir_all(root).unwrap();
        });
    }

    #[test]
    fn derives_terminfo_dir_from_resources_dir() {
        let resources_dir = PathBuf::from("/usr/share/limux/ghostty");
        assert_eq!(
            ghostty_terminfo_dir(&resources_dir),
            Some(PathBuf::from("/usr/share/limux/terminfo"))
        );
    }

    #[test]
    fn replaces_invalid_inherited_runtime_env_with_resolved_paths() {
        with_ghostty_env(|| {
            let root = temp_path("env-override");
            let exe_dir = root.join("target/release");
            let resources_dir = root.join("ghostty/zig-out/share/ghostty");
            let themes_dir = resources_dir.join("themes");
            let shell_integration_dir = resources_dir.join("shell-integration");
            let terminfo_dir = root.join("ghostty/zig-out/share/terminfo");
            let terminfo_file = terminfo_dir.join("x/xterm-ghostty");
            fs::create_dir_all(&exe_dir).unwrap();
            fs::create_dir_all(&themes_dir).unwrap();
            fs::create_dir_all(&shell_integration_dir).unwrap();
            fs::create_dir_all(terminfo_file.parent().unwrap()).unwrap();
            fs::write(&terminfo_file, b"xterm-ghostty").unwrap();

            std::env::set_var("GHOSTTY_RESOURCES_DIR", "/app/share/limux/ghostty");
            std::env::set_var("TERMINFO", "/app/share/limux/terminfo");
            std::env::set_var(
                "GHOSTTY_SHELL_INTEGRATION_XDG_DIR",
                "/app/share/limux/ghostty/shell-integration",
            );

            let exe = exe_dir.join("limux");
            set_ghostty_runtime_env_for_exe(&exe);

            assert_eq!(
                std::env::var_os("GHOSTTY_RESOURCES_DIR"),
                Some(resources_dir.into_os_string())
            );
            assert_eq!(
                std::env::var_os("TERMINFO"),
                Some(terminfo_dir.into_os_string())
            );
            assert_eq!(
                std::env::var_os("GHOSTTY_SHELL_INTEGRATION_XDG_DIR"),
                Some(shell_integration_dir.into_os_string())
            );

            fs::remove_dir_all(root).unwrap();
        });
    }

    #[test]
    fn preserves_valid_existing_runtime_env_paths() {
        with_ghostty_env(|| {
            let root = temp_path("env-preserve");
            let exe_dir = root.join("target/release");
            let resources_dir = root.join("ghostty/zig-out/share/ghostty");
            let themes_dir = resources_dir.join("themes");
            let shell_integration_dir = resources_dir.join("shell-integration");
            let terminfo_dir = root.join("ghostty/zig-out/share/terminfo");
            let terminfo_file = terminfo_dir.join("x/xterm-ghostty");
            fs::create_dir_all(&exe_dir).unwrap();
            fs::create_dir_all(&themes_dir).unwrap();
            fs::create_dir_all(&shell_integration_dir).unwrap();
            fs::create_dir_all(terminfo_file.parent().unwrap()).unwrap();
            fs::write(&terminfo_file, b"xterm-ghostty").unwrap();

            std::env::set_var("GHOSTTY_RESOURCES_DIR", &resources_dir);
            std::env::set_var("TERMINFO", &terminfo_dir);
            std::env::set_var("GHOSTTY_SHELL_INTEGRATION_XDG_DIR", &shell_integration_dir);

            let exe = exe_dir.join("limux");
            set_ghostty_runtime_env_for_exe(&exe);

            assert_eq!(
                std::env::var_os("GHOSTTY_RESOURCES_DIR"),
                Some(resources_dir.into_os_string())
            );
            assert_eq!(
                std::env::var_os("TERMINFO"),
                Some(terminfo_dir.into_os_string())
            );
            assert_eq!(
                std::env::var_os("GHOSTTY_SHELL_INTEGRATION_XDG_DIR"),
                Some(shell_integration_dir.into_os_string())
            );

            fs::remove_dir_all(root).unwrap();
        });
    }
}
