use std::env;
use std::fs;
use std::io;
use std::os::unix::fs::FileTypeExt;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener as StdUnixListener, UnixStream as StdUnixStream};
use std::path::Path;
use std::path::PathBuf;

const LIMUX_SOCKET_ENV: &str = "LIMUX_SOCKET";
const LIMUX_SOCKET_PATH_ENV: &str = "LIMUX_SOCKET_PATH";
pub const LIMUX_CHANNEL_ENV: &str = "LIMUX_CHANNEL";
pub const LIMUX_PREVIEW_ID_ENV: &str = "LIMUX_PREVIEW_ID";
pub const LIMUX_PROFILE_ID_ENV: &str = "LIMUX_PROFILE_ID";
const RUNTIME_SUBDIR: &str = "limux";
const PROFILES_SUBDIR: &str = "profiles";
const RUNTIME_SOCKET_NAME: &str = "limux.sock";
const FALLBACK_RUNTIME_SOCKET: &str = "/tmp/limux.sock";
const DEBUG_SOCKET: &str = "/tmp/limux-debug.sock";
const PRIVATE_DIR_MODE: u32 = 0o700;
const SOCKET_FILE_MODE: u32 = 0o600;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketMode {
    Runtime,
    Debug,
}

/// Which BUILD lane a runtime belongs to. Orthogonal to a session profile:
/// installed launchers pin the lane (`--channel stable`), while the operator
/// selects a session set (`--profile work`). Folding the two into one value
/// made them mutually exclusive, which made `--profile` unreachable from every
/// installed launcher — see `profile` below.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeChannel {
    Stable,
    Preview(String),
}

impl RuntimeChannel {
    pub const DEFAULT_PREVIEW_ID: &'static str = "default";
    /// Prefix for profiles the host assigns itself when a bare `limux`
    /// launch finds the default socket busy. Named so `profile list` can
    /// flag them as auto-created and therefore safe to prune.
    pub const AUTO_PROFILE_PREFIX: &'static str = "auto-";

    pub fn parse(raw: &str) -> Option<Self> {
        let raw = raw.trim();
        match raw {
            "stable" => Some(Self::Stable),
            "preview" => Some(Self::Preview(Self::DEFAULT_PREVIEW_ID.to_string())),
            _ => raw
                .strip_prefix("preview:")
                .or_else(|| raw.strip_prefix("preview/"))
                .and_then(sanitize_channel_id)
                .map(Self::Preview),
        }
    }

    pub fn from_env() -> Option<Self> {
        let channel = env::var(LIMUX_CHANNEL_ENV).ok()?;
        match channel.trim() {
            "preview" => env::var(LIMUX_PREVIEW_ID_ENV)
                .ok()
                .and_then(|value| sanitize_channel_id(&value))
                .map(Self::Preview)
                .or_else(|| Some(Self::Preview(Self::DEFAULT_PREVIEW_ID.to_string()))),
            _ => Self::parse(&channel),
        }
    }

    pub fn env_value(&self) -> String {
        match self {
            Self::Stable => "stable".to_string(),
            Self::Preview(id) => format!("preview:{id}"),
        }
    }

    pub fn label(&self) -> String {
        match self {
            Self::Stable => "stable".to_string(),
            Self::Preview(id) => format!("preview/{id}"),
        }
    }

    /// Path segments this lane contributes, under the `limux` root.
    fn segments(&self) -> Vec<String> {
        match self {
            Self::Stable => vec!["stable".to_string()],
            Self::Preview(id) => vec!["preview".to_string(), id.clone()],
        }
    }

    fn runtime_socket_path(&self) -> PathBuf {
        runtime_socket_path_for(Some(self), None)
    }

    /// Public accessor for a channel's runtime socket path.
    pub fn socket_path(&self) -> PathBuf {
        self.runtime_socket_path()
    }
}

/// Validate a user-supplied session profile name, rejecting anything that
/// could escape the profiles directory.
pub fn sanitize_profile_id(raw: &str) -> Option<String> {
    sanitize_channel_id(raw)
}

/// The session profile selected by the environment, independent of the build
/// lane in `LIMUX_CHANNEL`. Read separately precisely so a launcher-pinned
/// channel and an operator-chosen profile can coexist.
pub fn profile_from_env() -> Option<String> {
    env::var(LIMUX_PROFILE_ID_ENV)
        .ok()
        .and_then(|value| sanitize_profile_id(&value))
}

/// Runtime socket for a (build lane, session profile) pair.
///
/// Profiles nest UNDER their lane, so a preview build and a stable build can
/// each have a profile named `work` without sharing a socket or a session
/// file. Neither dimension is required: no lane and no profile resolves to the
/// historical default socket.
pub fn runtime_socket_path_for(channel: Option<&RuntimeChannel>, profile: Option<&str>) -> PathBuf {
    match env::var_os("XDG_RUNTIME_DIR") {
        Some(runtime_dir) if !runtime_dir.is_empty() => {
            let mut path = PathBuf::from(runtime_dir);
            path.push(RUNTIME_SUBDIR);
            if let Some(channel) = channel {
                for segment in channel.segments() {
                    path.push(segment);
                }
            }
            if let Some(profile) = profile {
                path.push(PROFILES_SUBDIR);
                path.push(profile);
            }
            path.push(RUNTIME_SOCKET_NAME);
            path
        }
        _ => {
            let lane = match channel {
                Some(RuntimeChannel::Stable) => "-stable".to_string(),
                Some(RuntimeChannel::Preview(id)) => format!("-preview-{id}"),
                None => String::new(),
            };
            let profile = profile
                .map(|name| format!("-profile-{name}"))
                .unwrap_or_default();
            PathBuf::from("/tmp").join(format!("limux{lane}{profile}.sock"))
        }
    }
}

impl SocketMode {
    pub fn default_for(mode: Self) -> PathBuf {
        match mode {
            Self::Runtime => default_runtime_socket_path(),
            Self::Debug => PathBuf::from(DEBUG_SOCKET),
        }
    }
}

pub fn resolve_socket_path(explicit: Option<PathBuf>, mode: SocketMode) -> PathBuf {
    if let Some(path) = explicit {
        return path;
    }

    if mode == SocketMode::Runtime {
        if let Some(path) = get_env_path(LIMUX_SOCKET_ENV) {
            return path;
        }
        if let Some(path) = get_env_path(LIMUX_SOCKET_PATH_ENV) {
            return path;
        }
        let channel = RuntimeChannel::from_env();
        let profile = profile_from_env();
        if channel.is_some() || profile.is_some() {
            return runtime_socket_path_for(channel.as_ref(), profile.as_deref());
        }
    }

    SocketMode::default_for(mode)
}

pub fn resolve_socket_path_for_channel(
    explicit: Option<PathBuf>,
    mode: SocketMode,
    channel: Option<&RuntimeChannel>,
) -> PathBuf {
    if let Some(path) = explicit {
        return path;
    }
    if mode == SocketMode::Runtime {
        if let Some(channel) = channel {
            return channel.runtime_socket_path();
        }
    }
    resolve_socket_path(None, mode)
}

pub fn prepare_socket_path(path: &Path, mode: SocketMode, owner_only: bool) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
        if owner_only && should_lock_down_parent(path, mode) {
            fs::set_permissions(parent, PermissionsExt::from_mode(PRIVATE_DIR_MODE))?;
        }
    }
    remove_existing_socket(path)?;
    Ok(())
}

pub fn finalize_socket_permissions(path: &Path, owner_only: bool) -> io::Result<()> {
    if owner_only {
        fs::set_permissions(path, PermissionsExt::from_mode(SOCKET_FILE_MODE))?;
    }
    Ok(())
}

pub fn bind_listener(
    path: &Path,
    mode: SocketMode,
    owner_only: bool,
) -> io::Result<StdUnixListener> {
    prepare_socket_path(path, mode, owner_only)?;
    let listener = StdUnixListener::bind(path)?;
    finalize_socket_permissions(path, owner_only)?;
    Ok(listener)
}

pub fn bind_tokio_listener(
    path: &Path,
    mode: SocketMode,
    owner_only: bool,
) -> io::Result<tokio::net::UnixListener> {
    prepare_socket_path(path, mode, owner_only)?;
    let listener = tokio::net::UnixListener::bind(path)?;
    finalize_socket_permissions(path, owner_only)?;
    Ok(listener)
}

fn default_runtime_socket_path() -> PathBuf {
    match env::var_os("XDG_RUNTIME_DIR") {
        Some(runtime_dir) if !runtime_dir.is_empty() => {
            let mut path = PathBuf::from(runtime_dir);
            path.push(RUNTIME_SUBDIR);
            path.push(RUNTIME_SOCKET_NAME);
            path
        }
        _ => PathBuf::from(FALLBACK_RUNTIME_SOCKET),
    }
}

fn default_runtime_socket_dir() -> Option<PathBuf> {
    let runtime_dir = env::var_os("XDG_RUNTIME_DIR")?;
    if runtime_dir.is_empty() {
        return None;
    }

    let mut path = PathBuf::from(runtime_dir);
    path.push(RUNTIME_SUBDIR);
    Some(path)
}

fn should_lock_down_parent(path: &Path, mode: SocketMode) -> bool {
    matches!(mode, SocketMode::Runtime) && path.parent() == default_runtime_socket_dir().as_deref()
}

fn remove_existing_socket(path: &Path) -> io::Result<()> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };

    if !metadata.file_type().is_socket() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("refusing to overwrite non-socket path {}", path.display()),
        ));
    }

    match StdUnixStream::connect(path) {
        Ok(_) => Err(io::Error::new(
            io::ErrorKind::AddrInUse,
            format!("socket already in use at {}", path.display()),
        )),
        Err(error) if is_stale_socket_error(&error) => {
            fs::remove_file(path)?;
            Ok(())
        }
        Err(error) => Err(io::Error::new(
            io::ErrorKind::AddrInUse,
            format!("refusing to replace socket at {}: {error}", path.display()),
        )),
    }
}

fn is_stale_socket_error(error: &io::Error) -> bool {
    matches!(
        error.kind(),
        io::ErrorKind::ConnectionRefused | io::ErrorKind::NotFound
    )
}

fn get_env_path(key: &str) -> Option<PathBuf> {
    env::var_os(key).and_then(|value| {
        if value.is_empty() {
            None
        } else {
            Some(PathBuf::from(value))
        }
    })
}

fn sanitize_channel_id(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed == "." || trimmed == ".." {
        return None;
    }
    let mut out = String::with_capacity(trimmed.len());
    for ch in trimmed.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            out.push(ch);
        } else {
            return None;
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::os::unix::fs::PermissionsExt;
    use std::os::unix::net::UnixListener;
    use std::sync::Mutex;
    use tempfile::TempDir;

    static ENV_TEST_LOCK: Mutex<()> = Mutex::new(());

    struct EnvGuard {
        key: &'static str,
        old: Option<std::ffi::OsString>,
    }

    impl EnvGuard {
        fn set(key: &'static str, value: Option<&str>) -> Self {
            let old = env::var_os(key);
            match value {
                Some(value) => unsafe { env::set_var(key, value) },
                None => unsafe { env::remove_var(key) },
            }
            Self { key, old }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match &self.old {
                Some(value) => unsafe { env::set_var(self.key, value) },
                None => unsafe { env::remove_var(self.key) },
            }
        }
    }

    #[test]
    fn explicit_path_has_highest_precedence() {
        let _lock = ENV_TEST_LOCK.lock().expect("env test lock");
        let _socket = EnvGuard::set(LIMUX_SOCKET_ENV, Some("/tmp/from-env.sock"));
        let _socket_path = EnvGuard::set(LIMUX_SOCKET_PATH_ENV, Some("/tmp/from-env-path.sock"));

        let resolved = resolve_socket_path(
            Some(PathBuf::from("/tmp/from-arg.sock")),
            SocketMode::Runtime,
        );
        assert_eq!(resolved, PathBuf::from("/tmp/from-arg.sock"));
    }

    #[test]
    fn limux_socket_has_higher_precedence_than_limux_socket_path() {
        let _lock = ENV_TEST_LOCK.lock().expect("env test lock");
        let _socket = EnvGuard::set(LIMUX_SOCKET_ENV, Some("/tmp/from-limux-socket.sock"));
        let _socket_path = EnvGuard::set(
            LIMUX_SOCKET_PATH_ENV,
            Some("/tmp/from-limux-socket-path.sock"),
        );

        let resolved = resolve_socket_path(None, SocketMode::Runtime);
        assert_eq!(resolved, PathBuf::from("/tmp/from-limux-socket.sock"));
    }

    #[test]
    fn limux_socket_path_used_when_limux_socket_missing() {
        let _lock = ENV_TEST_LOCK.lock().expect("env test lock");
        let _socket = EnvGuard::set(LIMUX_SOCKET_ENV, None);
        let _socket_path = EnvGuard::set(
            LIMUX_SOCKET_PATH_ENV,
            Some("/tmp/from-limux-socket-path.sock"),
        );

        let resolved = resolve_socket_path(None, SocketMode::Runtime);
        assert_eq!(resolved, PathBuf::from("/tmp/from-limux-socket-path.sock"));
    }

    #[test]
    fn runtime_mode_defaults_to_xdg_runtime_dir() {
        let _lock = ENV_TEST_LOCK.lock().expect("env test lock");
        let _socket = EnvGuard::set(LIMUX_SOCKET_ENV, None);
        let _socket_path = EnvGuard::set(LIMUX_SOCKET_PATH_ENV, None);
        let _channel = EnvGuard::set(LIMUX_CHANNEL_ENV, None);
        let _preview_id = EnvGuard::set(LIMUX_PREVIEW_ID_ENV, None);
        let xdg = TempDir::new().expect("xdg runtime dir temp path");
        let _xdg = EnvGuard::set("XDG_RUNTIME_DIR", Some(xdg.path().to_str().expect("utf8")));

        let resolved = resolve_socket_path(None, SocketMode::Runtime);
        assert_eq!(
            resolved,
            xdg.path().join(RUNTIME_SUBDIR).join(RUNTIME_SOCKET_NAME)
        );
    }

    #[test]
    fn runtime_channel_parses_stable_and_preview_ids() {
        assert_eq!(
            RuntimeChannel::parse("stable"),
            Some(RuntimeChannel::Stable)
        );
        assert_eq!(
            RuntimeChannel::parse("preview"),
            Some(RuntimeChannel::Preview("default".to_string()))
        );
        assert_eq!(
            RuntimeChannel::parse("preview:branch_123"),
            Some(RuntimeChannel::Preview("branch_123".to_string()))
        );
        assert_eq!(RuntimeChannel::parse("preview:bad/id"), None);
        assert_eq!(RuntimeChannel::parse("preview:.."), None);
    }






    /// The #92 revert defect: a launcher-pinned lane and a user profile must
    /// resolve TOGETHER. Modelling them as one value made `--profile`
    /// unreachable from every installed launcher.
    #[test]
    fn lane_and_profile_resolve_together() {
        let _lock = ENV_TEST_LOCK.lock().expect("env test lock");
        let xdg = TempDir::new().expect("xdg runtime dir temp path");
        let _xdg = EnvGuard::set("XDG_RUNTIME_DIR", Some(xdg.path().to_str().expect("utf8")));

        assert_eq!(
            runtime_socket_path_for(Some(&RuntimeChannel::Stable), Some("work")),
            xdg.path().join("limux/stable/profiles/work/limux.sock")
        );
        assert_eq!(
            runtime_socket_path_for(Some(&RuntimeChannel::Preview("lab".into())), Some("work")),
            xdg.path().join("limux/preview/lab/profiles/work/limux.sock")
        );
        // Either dimension alone still resolves.
        assert_eq!(
            runtime_socket_path_for(None, Some("work")),
            xdg.path().join("limux/profiles/work/limux.sock")
        );
        assert_eq!(
            runtime_socket_path_for(Some(&RuntimeChannel::Stable), None),
            xdg.path().join("limux/stable/limux.sock")
        );
    }

    /// Two BUILDS must never share a profile's socket.
    #[test]
    fn same_profile_in_different_lanes_gets_different_sockets() {
        let _lock = ENV_TEST_LOCK.lock().expect("env test lock");
        let xdg = TempDir::new().expect("xdg runtime dir temp path");
        let _xdg = EnvGuard::set("XDG_RUNTIME_DIR", Some(xdg.path().to_str().expect("utf8")));

        let stable = runtime_socket_path_for(Some(&RuntimeChannel::Stable), Some("work"));
        let preview =
            runtime_socket_path_for(Some(&RuntimeChannel::Preview("lab".into())), Some("work"));
        assert_ne!(stable, preview);
    }

    /// NEW traversal surface: making the lane a path component means the
    /// channel/preview-id needs the same allowlist rigor as a profile name.
    /// This surface did not exist before profiles nested under lanes.
    #[test]
    fn neither_lane_nor_profile_can_traverse_out_of_their_directory() {
        for bad in ["..", "../escape", "a/b", "", ".", "x\0y"] {
            assert_eq!(
                sanitize_profile_id(bad),
                None,
                "profile id {bad:?} must be rejected"
            );
            assert_eq!(
                RuntimeChannel::parse(&format!("preview:{bad}")),
                None,
                "preview id {bad:?} must be rejected"
            );
        }
        // And the allowlist still admits the shapes we rely on.
        assert_eq!(sanitize_profile_id("auto-2").as_deref(), Some("auto-2"));
        assert_eq!(sanitize_profile_id("work_1").as_deref(), Some("work_1"));
    }

    /// The profile is read from its OWN env var, independent of the lane, so
    /// a launcher's `LIMUX_CHANNEL` and an operator's profile coexist.
    #[test]
    fn profile_env_is_independent_of_channel_env() {
        let _lock = ENV_TEST_LOCK.lock().expect("env test lock");
        let _channel = EnvGuard::set(LIMUX_CHANNEL_ENV, Some("stable"));
        let _preview = EnvGuard::set(LIMUX_PREVIEW_ID_ENV, None);
        let _profile = EnvGuard::set(LIMUX_PROFILE_ID_ENV, Some("work"));

        assert_eq!(RuntimeChannel::from_env(), Some(RuntimeChannel::Stable));
        assert_eq!(profile_from_env().as_deref(), Some("work"));
    }

    /// A profile socket must be no more permissive than any other. Profiles
    /// multiply live sockets, so 0600 drift here multiplies exposure.
    #[test]
    fn profile_socket_is_owner_only_like_every_other_channel() {
        let _lock = ENV_TEST_LOCK.lock().expect("env test lock");
        let _socket = EnvGuard::set(LIMUX_SOCKET_ENV, None);
        let _socket_path = EnvGuard::set(LIMUX_SOCKET_PATH_ENV, None);
        let xdg = TempDir::new().expect("xdg runtime dir temp path");
        let _xdg = EnvGuard::set("XDG_RUNTIME_DIR", Some(xdg.path().to_str().expect("utf8")));

        let path = runtime_socket_path_for(Some(&RuntimeChannel::Stable), Some("work"));
        let listener =
            bind_listener(&path, SocketMode::Runtime, true).expect("bind profile socket");

        let mode = std::fs::metadata(&path)
            .expect("profile socket metadata")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, SOCKET_FILE_MODE);
        drop(listener);
    }

    #[test]
    fn channel_socket_paths_use_isolated_runtime_namespaces() {
        let _lock = ENV_TEST_LOCK.lock().expect("env test lock");
        let _socket = EnvGuard::set(LIMUX_SOCKET_ENV, None);
        let _socket_path = EnvGuard::set(LIMUX_SOCKET_PATH_ENV, None);
        let _channel = EnvGuard::set(LIMUX_CHANNEL_ENV, None);
        let _preview_id = EnvGuard::set(LIMUX_PREVIEW_ID_ENV, None);
        let xdg = TempDir::new().expect("xdg runtime dir temp path");
        let _xdg = EnvGuard::set("XDG_RUNTIME_DIR", Some(xdg.path().to_str().expect("utf8")));

        let stable = resolve_socket_path_for_channel(
            None,
            SocketMode::Runtime,
            Some(&RuntimeChannel::Stable),
        );
        let preview = resolve_socket_path_for_channel(
            None,
            SocketMode::Runtime,
            Some(&RuntimeChannel::Preview("test".to_string())),
        );

        assert_eq!(stable, xdg.path().join("limux/stable/limux.sock"));
        assert_eq!(preview, xdg.path().join("limux/preview/test/limux.sock"));
    }

    #[test]
    fn explicit_channel_overrides_inherited_socket_env_for_cli_targeting() {
        let _lock = ENV_TEST_LOCK.lock().expect("env test lock");
        let _socket = EnvGuard::set(LIMUX_SOCKET_ENV, Some("/tmp/inherited-stable.sock"));
        let _socket_path = EnvGuard::set(LIMUX_SOCKET_PATH_ENV, None);
        let _channel = EnvGuard::set(LIMUX_CHANNEL_ENV, None);
        let _preview_id = EnvGuard::set(LIMUX_PREVIEW_ID_ENV, None);
        let _xdg = EnvGuard::set("XDG_RUNTIME_DIR", None);

        let resolved = resolve_socket_path_for_channel(
            None,
            SocketMode::Runtime,
            Some(&RuntimeChannel::Preview("branch".to_string())),
        );

        assert_eq!(resolved, PathBuf::from("/tmp/limux-preview-branch.sock"));
    }

    #[test]
    fn env_channel_is_lower_precedence_than_inherited_socket() {
        let _lock = ENV_TEST_LOCK.lock().expect("env test lock");
        let _socket = EnvGuard::set(LIMUX_SOCKET_ENV, Some("/tmp/from-env.sock"));
        let _socket_path = EnvGuard::set(LIMUX_SOCKET_PATH_ENV, None);
        let _channel = EnvGuard::set(LIMUX_CHANNEL_ENV, Some("preview:env"));
        let _preview_id = EnvGuard::set(LIMUX_PREVIEW_ID_ENV, None);

        let resolved = resolve_socket_path(None, SocketMode::Runtime);

        assert_eq!(resolved, PathBuf::from("/tmp/from-env.sock"));
    }

    #[test]
    fn debug_mode_defaults_to_debug_socket() {
        let _lock = ENV_TEST_LOCK.lock().expect("env test lock");
        let _socket = EnvGuard::set(LIMUX_SOCKET_ENV, None);
        let _socket_path = EnvGuard::set(LIMUX_SOCKET_PATH_ENV, None);
        let _xdg = EnvGuard::set("XDG_RUNTIME_DIR", None);

        let resolved = resolve_socket_path(None, SocketMode::Debug);
        assert_eq!(resolved, PathBuf::from(DEBUG_SOCKET));
    }

    #[test]
    fn debug_mode_ignores_inherited_runtime_socket_env() {
        let _lock = ENV_TEST_LOCK.lock().expect("env test lock");
        let _socket = EnvGuard::set(LIMUX_SOCKET_ENV, Some("/tmp/from-runtime.sock"));
        let _socket_path =
            EnvGuard::set(LIMUX_SOCKET_PATH_ENV, Some("/tmp/from-runtime-path.sock"));

        let resolved = resolve_socket_path(None, SocketMode::Debug);
        assert_eq!(resolved, PathBuf::from(DEBUG_SOCKET));
    }

    #[test]
    fn prepare_socket_path_locks_down_runtime_parent_dir() {
        let _lock = ENV_TEST_LOCK.lock().expect("env test lock");
        let _socket = EnvGuard::set(LIMUX_SOCKET_ENV, None);
        let _socket_path = EnvGuard::set(LIMUX_SOCKET_PATH_ENV, None);
        let _channel = EnvGuard::set(LIMUX_CHANNEL_ENV, None);
        let _preview_id = EnvGuard::set(LIMUX_PREVIEW_ID_ENV, None);
        let xdg = TempDir::new().expect("xdg runtime dir temp path");
        let _xdg = EnvGuard::set("XDG_RUNTIME_DIR", Some(xdg.path().to_str().expect("utf8")));

        let path = resolve_socket_path(None, SocketMode::Runtime);
        prepare_socket_path(&path, SocketMode::Runtime, true).expect("prepare socket path");

        let mode = std::fs::metadata(path.parent().expect("socket parent"))
            .expect("parent metadata")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, PRIVATE_DIR_MODE);
    }

    #[test]
    fn finalize_socket_permissions_sets_socket_mode() {
        let temp_dir = TempDir::new().expect("temp dir");
        let socket_path = temp_dir.path().join("limux.sock");
        let listener = UnixListener::bind(&socket_path).expect("bind listener");

        finalize_socket_permissions(&socket_path, true).expect("set socket permissions");

        let mode = std::fs::metadata(&socket_path)
            .expect("socket metadata")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, SOCKET_FILE_MODE);

        drop(listener);
    }

    #[test]
    fn prepare_socket_path_does_not_force_private_parent_for_allow_all() {
        let _lock = ENV_TEST_LOCK.lock().expect("env test lock");
        let _socket = EnvGuard::set(LIMUX_SOCKET_ENV, None);
        let _socket_path = EnvGuard::set(LIMUX_SOCKET_PATH_ENV, None);
        let _channel = EnvGuard::set(LIMUX_CHANNEL_ENV, None);
        let _preview_id = EnvGuard::set(LIMUX_PREVIEW_ID_ENV, None);
        let xdg = TempDir::new().expect("xdg runtime dir temp path");
        let _xdg = EnvGuard::set("XDG_RUNTIME_DIR", Some(xdg.path().to_str().expect("utf8")));

        let path = resolve_socket_path(None, SocketMode::Runtime);
        std::fs::create_dir_all(path.parent().expect("socket parent")).expect("create parent");
        std::fs::set_permissions(
            path.parent().expect("socket parent"),
            PermissionsExt::from_mode(0o755),
        )
        .expect("set parent permissions");

        prepare_socket_path(&path, SocketMode::Runtime, false).expect("prepare socket path");

        let mode = std::fs::metadata(path.parent().expect("socket parent"))
            .expect("parent metadata")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o755);
    }

    #[test]
    fn prepare_socket_path_refuses_to_overwrite_non_socket_path() {
        let temp_dir = TempDir::new().expect("temp dir");
        let socket_path = temp_dir.path().join("limux.sock");
        std::fs::write(&socket_path, b"not a socket").expect("write placeholder");

        let error = prepare_socket_path(&socket_path, SocketMode::Runtime, true)
            .expect_err("non-socket path should fail");
        assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
    }

    #[test]
    fn prepare_socket_path_rejects_live_socket() {
        let temp_dir = TempDir::new().expect("temp dir");
        let socket_path = temp_dir.path().join("limux.sock");
        let _listener = UnixListener::bind(&socket_path).expect("bind listener");

        let error = prepare_socket_path(&socket_path, SocketMode::Runtime, true)
            .expect_err("live socket should fail");
        assert_eq!(error.kind(), io::ErrorKind::AddrInUse);
    }

    #[test]
    fn prepare_socket_path_removes_stale_socket() {
        let temp_dir = TempDir::new().expect("temp dir");
        let socket_path = temp_dir.path().join("limux.sock");
        let listener = UnixListener::bind(&socket_path).expect("bind listener");
        drop(listener);

        prepare_socket_path(&socket_path, SocketMode::Runtime, true)
            .expect("stale socket should be removed");
        assert!(!socket_path.exists());
    }
}
