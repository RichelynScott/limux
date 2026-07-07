mod app_config;
mod control_bridge;
mod ghostty_config;
mod keybind_editor;
mod layout_state;
mod pane;
mod settings_editor;
mod shortcut_config;
mod split_tree;
mod terminal;
mod window;

use adw::prelude::*;
use libadwaita as adw;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::os::fd::AsRawFd;
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
const HOST_LOG_FILE_NAME: &str = "limux-host.log";

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

    let socket_path = unique_runtime_socket_path(&default_path);
    let session_dir = unique_runtime_session_dir(&socket_path);
    eprintln!(
        "limux: default control socket already in use ({}); using {}",
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
            dirs::state_dir().map(|dir| dir.join(HOST_LOG_DIR_NAME).join(HOST_LOG_FILE_NAME))
        })
}

#[cfg(unix)]
fn install_host_stderr_log() -> io::Result<Option<PathBuf>> {
    if std::env::var_os(HOST_LOG_ENV).is_some_and(|value| value == "off" || value == "0") {
        return Ok(None);
    }

    let Some(path) = host_log_path() else {
        return Ok(None);
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = OpenOptions::new().create(true).append(true).open(&path)?;
    writeln!(
        file,
        "\n--- limux-host start version={} pid={} ---",
        VERSION,
        std::process::id()
    )?;
    file.flush()?;

    let rc = unsafe { libc::dup2(file.as_raw_fd(), libc::STDERR_FILENO) };
    if rc < 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(Some(path))
}

#[cfg(not(unix))]
fn install_host_stderr_log() -> io::Result<Option<PathBuf>> {
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
        println!("Limux {VERSION}");
        return;
    }

    ensure_xdg_data_dirs_defaults();
    sanitize_inherited_limux_target_env_for_host();

    if let Err(err) = install_host_stderr_log() {
        eprintln!("limux: failed to initialize host log: {err}");
    }
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

    #[cfg(unix)]
    #[test]
    fn runtime_socket_uses_unique_path_when_default_socket_is_live() {
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
        let _session_dir =
            EnvVarGuard::set(layout_state::LIMUX_SESSION_DIR_ENV, Option::<&str>::None);

        let default_path = limux_control::socket_path::SocketMode::default_for(
            limux_control::socket_path::SocketMode::Runtime,
        );
        fs::create_dir_all(default_path.parent().expect("default socket parent"))
            .expect("create socket parent");
        let _listener = UnixListener::bind(&default_path).expect("bind default socket");

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
        let _session_dir =
            EnvVarGuard::set(layout_state::LIMUX_SESSION_DIR_ENV, Option::<&str>::None);

        let default_path = limux_control::socket_path::SocketMode::default_for(
            limux_control::socket_path::SocketMode::Runtime,
        );
        fs::create_dir_all(default_path.parent().expect("default socket parent"))
            .expect("create socket parent");
        let _listener = UnixListener::bind(&default_path).expect("bind default socket");

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
