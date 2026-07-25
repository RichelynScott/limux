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

/// Directory a build lane owns, relative to an explicit base.
fn channel_dir_in(base: &Path, channel: Option<&RuntimeChannel>) -> PathBuf {
    match channel {
        Some(RuntimeChannel::Stable) => base.join("stable"),
        Some(RuntimeChannel::Preview(id)) => base.join("preview").join(id),
        // No lane: the historical un-namespaced root.
        None => base.to_path_buf(),
    }
}

/// Persisted session directory for a (build lane, session profile) pair.
///
/// Profiles nest UNDER their lane, so a preview build and a stable build can
/// each hold a profile named `work` without sharing a `session.json`.
pub fn session_dir_in(
    base: &Path,
    channel: Option<&RuntimeChannel>,
    profile: Option<&str>,
) -> PathBuf {
    let lane = channel_dir_in(base, channel);
    match profile {
        Some(name) => profiles_root_dir_in(base, channel)
            .join(name)
            .join(SESSION_DIR_NAME),
        // Preserve the historical shapes exactly: an un-namespaced runtime
        // keeps session.json at the root, a lane keeps it under `session/`.
        None if channel.is_none() => lane,
        None => lane.join(SESSION_DIR_NAME),
    }
}

/// Persisted session directory for a channel, relative to an explicit base.
pub fn channel_persistence_dir_in(base: &Path, channel: &RuntimeChannel) -> PathBuf {
    session_dir_in(base, Some(channel), None)
}

/// Persisted session directory for a channel under the default base.
pub fn channel_persistence_dir(channel: &RuntimeChannel) -> PathBuf {
    channel_persistence_dir_in(&base_persistence_dir(), channel)
}

/// Directory containing every profile belonging to one build lane.
pub fn profiles_root_dir_in(base: &Path, channel: Option<&RuntimeChannel>) -> PathBuf {
    channel_dir_in(base, channel).join(PROFILES_DIR_NAME)
}

/// Directory containing every profile of the active lane, under the default base.
pub fn profiles_root_dir(channel: Option<&RuntimeChannel>) -> PathBuf {
    profiles_root_dir_in(&base_persistence_dir(), channel)
}

/// Persisted session directory for one named profile within a lane.
pub fn profile_persistence_dir_in(
    base: &Path,
    channel: Option<&RuntimeChannel>,
    id: &str,
) -> PathBuf {
    session_dir_in(base, channel, Some(id))
}

/// Everything belonging to one profile — removed wholesale by `profile rm`.
pub fn profile_root_dir_in(base: &Path, channel: Option<&RuntimeChannel>, id: &str) -> PathBuf {
    profiles_root_dir_in(base, channel).join(id)
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
