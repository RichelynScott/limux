//! Where a runtime channel's persisted session state lives on disk.
//!
//! This is the single source of truth for the on-disk layout. Both the host
//! (which reads and writes `session.json`) and the CLI (which lists and prunes
//! profiles) resolve through here, so the two can never disagree about where a
//! profile's state is — a disagreement would make `limux profile list` report
//! profiles the host never uses.

use std::path::{Path, PathBuf};

use crate::socket_path::RuntimeChannel;

pub const PERSISTENCE_DIR_NAME: &str = "limux";
pub const PROFILES_DIR_NAME: &str = "profiles";
pub const SESSION_DIR_NAME: &str = "session";

/// Root of all Limux persisted state: `$XDG_DATA_HOME/limux`, falling back to
/// `$HOME/.local/share/limux`.
pub fn base_persistence_dir() -> PathBuf {
    if let Some(data_dir) = dirs::data_dir() {
        return data_dir.join(PERSISTENCE_DIR_NAME);
    }

    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".local/share").join(PERSISTENCE_DIR_NAME)
}

/// Persisted session directory for a channel, relative to an explicit base.
pub fn channel_persistence_dir_in(base: &Path, channel: &RuntimeChannel) -> PathBuf {
    match channel {
        RuntimeChannel::Stable => base.join("stable").join(SESSION_DIR_NAME),
        RuntimeChannel::Preview(id) => base.join("preview").join(id).join(SESSION_DIR_NAME),
        RuntimeChannel::Profile(id) => profile_persistence_dir_in(base, id),
    }
}

/// Persisted session directory for a channel under the default base.
pub fn channel_persistence_dir(channel: &RuntimeChannel) -> PathBuf {
    channel_persistence_dir_in(&base_persistence_dir(), channel)
}

/// Directory containing every named profile.
pub fn profiles_root_dir_in(base: &Path) -> PathBuf {
    base.join(PROFILES_DIR_NAME)
}

/// Directory containing every named profile, under the default base.
pub fn profiles_root_dir() -> PathBuf {
    profiles_root_dir_in(&base_persistence_dir())
}

/// Persisted session directory for one named profile.
pub fn profile_persistence_dir_in(base: &Path, id: &str) -> PathBuf {
    profiles_root_dir_in(base).join(id).join(SESSION_DIR_NAME)
}

/// Persisted session directory for one named profile, under the default base.
pub fn profile_persistence_dir(id: &str) -> PathBuf {
    profile_persistence_dir_in(&base_persistence_dir(), id)
}

/// Everything belonging to one profile — removed wholesale by `profile rm`.
pub fn profile_root_dir_in(base: &Path, id: &str) -> PathBuf {
    profiles_root_dir_in(base).join(id)
}

/// True when this profile was auto-created by the host rather than named by
/// the user, and is therefore safe to offer for pruning.
pub fn is_auto_profile(id: &str) -> bool {
    id.starts_with(RuntimeChannel::AUTO_PROFILE_PREFIX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_dirs_are_mutually_isolated() {
        let base = Path::new("/data/limux");

        let stable = channel_persistence_dir_in(base, &RuntimeChannel::Stable);
        let preview = channel_persistence_dir_in(base, &RuntimeChannel::Preview("work".into()));
        let profile = channel_persistence_dir_in(base, &RuntimeChannel::Profile("work".into()));

        assert_eq!(stable, PathBuf::from("/data/limux/stable/session"));
        assert_eq!(preview, PathBuf::from("/data/limux/preview/work/session"));
        assert_eq!(profile, PathBuf::from("/data/limux/profiles/work/session"));
        assert_ne!(preview, profile);
    }

    #[test]
    fn profile_dir_is_nested_under_the_profiles_root() {
        let base = Path::new("/data/limux");
        assert!(profile_persistence_dir_in(base, "work").starts_with(profiles_root_dir_in(base)));
        assert_eq!(
            profile_root_dir_in(base, "work"),
            PathBuf::from("/data/limux/profiles/work")
        );
    }

    #[test]
    fn auto_profiles_are_recognizable() {
        assert!(is_auto_profile("auto-2"));
        assert!(is_auto_profile("auto-17"));
        assert!(!is_auto_profile("work"));
        assert!(!is_auto_profile("autopilot"));
    }
}
