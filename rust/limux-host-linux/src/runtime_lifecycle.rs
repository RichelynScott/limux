use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const RUNTIME_MARKER_FILE_NAME: &str = "runtime-incarnation.json";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RuntimeMarker {
    pub incarnation_id: String,
    pub pid: u32,
    pub started_at_unix_ms: u64,
    pub version: String,
    pub clean_shutdown: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeMarkerSeed<'a> {
    pub pid: u32,
    pub started_at_unix_ms: u64,
    pub version: &'a str,
}

impl<'a> RuntimeMarkerSeed<'a> {
    pub fn current(version: &'a str) -> io::Result<Self> {
        let started_at_unix_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?
            .as_millis()
            .try_into()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "timestamp overflow"))?;
        Ok(Self {
            pid: std::process::id(),
            started_at_unix_ms,
            version,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UncleanStartupReason {
    MissingMarker,
    MalformedMarker,
    PreviousRunUnclean,
    MarkerReadFailed,
}

impl UncleanStartupReason {
    pub fn name(self) -> &'static str {
        match self {
            Self::MissingMarker => "missing_marker",
            Self::MalformedMarker => "malformed_marker",
            Self::PreviousRunUnclean => "previous_run_unclean",
            Self::MarkerReadFailed => "marker_read_failed",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "status", content = "reason", rename_all = "snake_case")]
pub enum StartupClassification {
    Clean,
    Unclean(UncleanStartupReason),
}

impl StartupClassification {
    pub fn is_unclean(&self) -> bool {
        matches!(self, Self::Unclean(_))
    }
}

#[derive(Clone, Debug)]
pub struct RuntimeLifecycle {
    marker_path: PathBuf,
    marker: RuntimeMarker,
    previous_shutdown: StartupClassification,
}

impl RuntimeLifecycle {
    #[cfg(test)]
    pub fn marker_path(&self) -> &Path {
        &self.marker_path
    }

    #[cfg(test)]
    pub fn marker(&self) -> &RuntimeMarker {
        &self.marker
    }

    pub fn previous_shutdown(&self) -> &StartupClassification {
        &self.previous_shutdown
    }

    pub fn mark_clean(&self) -> io::Result<bool> {
        crate::durable_atomic::update_bytes_atomic_durable(&self.marker_path, |bytes| {
            let mut current_marker = serde_json::from_slice::<RuntimeMarker>(bytes)
                .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
            if current_marker.incarnation_id != self.marker.incarnation_id {
                return Ok(None);
            }
            current_marker.clean_shutdown = true;
            serde_json::to_vec_pretty(&current_marker)
                .map(Some)
                .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
        })
    }

    pub fn mark_clean_if_session_saved(&self, session_saved: bool) -> io::Result<bool> {
        if !session_saved {
            return Ok(false);
        }
        self.mark_clean()
    }
}

pub fn begin_runtime_in(dir: &Path, seed: RuntimeMarkerSeed<'_>) -> io::Result<RuntimeLifecycle> {
    fs::create_dir_all(dir)?;
    let marker_path = dir.join(RUNTIME_MARKER_FILE_NAME);
    let previous_shutdown = classify_previous_marker(&marker_path);
    let marker = RuntimeMarker {
        incarnation_id: uuid::Uuid::new_v4().to_string(),
        pid: seed.pid,
        started_at_unix_ms: seed.started_at_unix_ms,
        version: seed.version.to_string(),
        clean_shutdown: false,
    };
    write_marker_atomic(&marker_path, &marker)?;
    Ok(RuntimeLifecycle {
        marker_path,
        marker,
        previous_shutdown,
    })
}

fn classify_previous_marker(path: &Path) -> StartupClassification {
    match fs::read(path) {
        Ok(bytes) => match serde_json::from_slice::<RuntimeMarker>(&bytes) {
            Ok(marker) if marker.clean_shutdown => StartupClassification::Clean,
            Ok(_) => StartupClassification::Unclean(UncleanStartupReason::PreviousRunUnclean),
            Err(_) => StartupClassification::Unclean(UncleanStartupReason::MalformedMarker),
        },
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            StartupClassification::Unclean(UncleanStartupReason::MissingMarker)
        }
        Err(_) => StartupClassification::Unclean(UncleanStartupReason::MarkerReadFailed),
    }
}

fn write_marker_atomic(path: &Path, marker: &RuntimeMarker) -> io::Result<()> {
    let bytes = serde_json::to_vec_pretty(marker)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    crate::durable_atomic::write_bytes_atomic_durable(path, &bytes)
}

#[cfg(test)]
mod tests {
    use super::{begin_runtime_in, RuntimeMarkerSeed, StartupClassification, UncleanStartupReason};
    use std::fs;
    use tempfile::tempdir;

    fn seed(started_at_unix_ms: u64) -> RuntimeMarkerSeed<'static> {
        RuntimeMarkerSeed {
            pid: 4242,
            started_at_unix_ms,
            version: "0.2.2",
        }
    }

    #[test]
    fn missing_marker_is_unclean_and_new_incarnation_starts_dirty() {
        let dir = tempdir().expect("tempdir");

        let lifecycle = begin_runtime_in(dir.path(), seed(1000)).expect("begin runtime");

        assert_eq!(
            lifecycle.previous_shutdown(),
            &StartupClassification::Unclean(UncleanStartupReason::MissingMarker)
        );
        assert!(!lifecycle.marker().clean_shutdown);
        assert_eq!(lifecycle.marker().pid, 4242);
        assert_eq!(lifecycle.marker().started_at_unix_ms, 1000);
        assert_eq!(lifecycle.marker().version, "0.2.2");
        assert!(lifecycle.marker_path().is_file());
    }

    #[test]
    fn clean_shutdown_is_observed_before_next_incarnation_is_written_dirty() {
        let dir = tempdir().expect("tempdir");
        let first = begin_runtime_in(dir.path(), seed(1000)).expect("first runtime");
        let first_id = first.marker().incarnation_id.clone();
        first.mark_clean().expect("mark clean");

        let second = begin_runtime_in(dir.path(), seed(2000)).expect("second runtime");

        assert_eq!(second.previous_shutdown(), &StartupClassification::Clean);
        assert!(!second.marker().clean_shutdown);
        assert_ne!(second.marker().incarnation_id, first_id);
        assert_eq!(second.marker().started_at_unix_ms, 2000);
    }

    #[test]
    fn dirty_and_malformed_markers_fail_closed_to_unclean() {
        let dirty_dir = tempdir().expect("dirty tempdir");
        let first = begin_runtime_in(dirty_dir.path(), seed(1000)).expect("first runtime");
        drop(first);
        let dirty_restart = begin_runtime_in(dirty_dir.path(), seed(2000)).expect("dirty restart");
        assert_eq!(
            dirty_restart.previous_shutdown(),
            &StartupClassification::Unclean(UncleanStartupReason::PreviousRunUnclean)
        );

        let malformed_dir = tempdir().expect("malformed tempdir");
        fs::write(
            malformed_dir.path().join("runtime-incarnation.json"),
            b"not-json",
        )
        .expect("write malformed marker");
        let malformed_restart =
            begin_runtime_in(malformed_dir.path(), seed(3000)).expect("malformed restart");
        assert_eq!(
            malformed_restart.previous_shutdown(),
            &StartupClassification::Unclean(UncleanStartupReason::MalformedMarker)
        );
    }

    #[test]
    fn failed_session_save_never_marks_the_runtime_clean() {
        let dir = tempdir().expect("tempdir");
        let first = begin_runtime_in(dir.path(), seed(1000)).expect("first runtime");

        assert!(!first
            .mark_clean_if_session_saved(false)
            .expect("skip clean marker"));

        let second = begin_runtime_in(dir.path(), seed(2000)).expect("second runtime");
        assert_eq!(
            second.previous_shutdown(),
            &StartupClassification::Unclean(UncleanStartupReason::PreviousRunUnclean)
        );
    }

    #[test]
    fn stale_incarnation_cannot_mark_a_newer_runtime_clean() {
        let dir = tempdir().expect("tempdir");
        let first = begin_runtime_in(dir.path(), seed(1000)).expect("first runtime");
        let second = begin_runtime_in(dir.path(), seed(2000)).expect("second runtime");

        assert!(!first
            .mark_clean_if_session_saved(true)
            .expect("stale clean attempt"));
        assert!(second
            .mark_clean_if_session_saved(true)
            .expect("current clean attempt"));

        let third = begin_runtime_in(dir.path(), seed(3000)).expect("third runtime");
        assert_eq!(third.previous_shutdown(), &StartupClassification::Clean);
    }
}
