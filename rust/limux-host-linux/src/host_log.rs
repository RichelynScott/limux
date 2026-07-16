#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn config(root: &std::path::Path) -> HostLogConfig {
        HostLogConfig {
            active_path: root.join("managed/limux-host.current.log"),
            retained_dir: root.join("managed/retained"),
            max_active_bytes: 64,
            max_retained_count: 2,
            max_total_bytes: 128,
            max_warning_categories: 2,
        }
    }

    #[test]
    fn retained_name_collision_never_clobbers_existing_file() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let config = config(tmp.path());
        fs::create_dir_all(&config.retained_dir).expect("retained dir");
        fs::create_dir_all(config.active_path.parent().expect("active parent"))
            .expect("active dir");
        fs::write(&config.active_path, b"new managed log").expect("active fixture");
        let collision = config.retained_dir.join("limux-host.42.log");
        fs::write(&collision, b"preserve me").expect("collision fixture");

        let outcome = rotate_managed_active(&config, 42).expect("rotation result");

        assert_eq!(
            fs::read(&collision).expect("collision contents"),
            b"preserve me"
        );
        assert_eq!(
            outcome,
            RotationOutcome::Rotated(config.retained_dir.join("limux-host.42.1.log"))
        );
        assert_eq!(
            fs::read(config.retained_dir.join("limux-host.42.1.log")).expect("rotated contents"),
            b"new managed log"
        );
    }

    #[test]
    fn exhausted_retention_fails_closed_without_removing_files() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let config = config(tmp.path());
        fs::create_dir_all(&config.retained_dir).expect("retained dir");
        fs::create_dir_all(config.active_path.parent().expect("active parent"))
            .expect("active dir");
        fs::write(&config.active_path, vec![b'a'; 32]).expect("active fixture");
        fs::write(config.retained_dir.join("limux-host.1.log"), vec![b'1'; 48])
            .expect("retained one");
        fs::write(config.retained_dir.join("limux-host.2.log"), vec![b'2'; 48])
            .expect("retained two");

        let outcome = rotate_managed_active(&config, 43).expect("rotation result");

        assert!(matches!(outcome, RotationOutcome::StderrFallback { .. }));
        assert_eq!(
            fs::read(&config.active_path).expect("active preserved"),
            vec![b'a'; 32]
        );
        assert_eq!(
            fs::read(config.retained_dir.join("limux-host.1.log")).expect("one preserved"),
            vec![b'1'; 48]
        );
        assert_eq!(
            fs::read(config.retained_dir.join("limux-host.2.log")).expect("two preserved"),
            vec![b'2'; 48]
        );
    }

    #[test]
    fn setup_failure_returns_stderr_fallback_without_blocking_startup() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let config = config(tmp.path());
        fs::write(tmp.path().join("managed"), b"parent is a file").expect("blocked parent");

        let outcome = prepare_host_logging(&config, 44);

        assert!(matches!(outcome, HostLogSetup::StderrFallback { .. }));
    }

    #[test]
    fn setup_never_reads_or_mutates_legacy_incident_log() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let legacy_incident = tmp.path().join("limux-host.log");
        let sentinel = b"legacy incident bytes must remain exact";
        fs::write(&legacy_incident, sentinel).expect("legacy fixture");
        let before = fs::metadata(&legacy_incident).expect("legacy metadata");
        let config = config(tmp.path());

        let _ = prepare_host_logging(&config, 45);

        let after = fs::metadata(&legacy_incident).expect("legacy metadata after setup");
        assert_eq!(
            fs::read(&legacy_incident).expect("legacy contents"),
            sentinel
        );
        assert_eq!(after.len(), before.len());
        assert_eq!(
            after.modified().expect("after mtime"),
            before.modified().expect("before mtime")
        );
    }

    #[test]
    fn repeated_warning_is_aggregated_and_recovery_reports_count() {
        let mut warnings = WarningAggregator::new(2);

        assert_eq!(
            warnings.record("renderer_context_lost", "context lost"),
            WarningEvent::First {
                message: "context lost".to_string()
            }
        );
        assert_eq!(
            warnings.record("renderer_context_lost", "context lost again"),
            WarningEvent::Suppressed { count: 2 }
        );
        assert_eq!(
            warnings.recover("renderer_context_lost"),
            WarningEvent::Recovered {
                total_count: 2,
                repeated_count: 1
            }
        );
    }

    #[test]
    fn warning_category_count_is_bounded() {
        let mut warnings = WarningAggregator::new(2);
        let _ = warnings.record("renderer_context_lost", "one");
        let _ = warnings.record("wsl_vhd_wait_timeout", "two");

        assert_eq!(
            warnings.record("third_category", "three"),
            WarningEvent::CategoryLimitReached { max_categories: 2 }
        );
        assert_eq!(warnings.category_count(), 2);
    }
}
