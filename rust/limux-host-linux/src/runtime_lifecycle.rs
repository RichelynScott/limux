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
}
