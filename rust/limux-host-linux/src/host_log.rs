use std::collections::HashMap;
use std::ffi::CString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::os::fd::FromRawFd;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::thread;

const MAX_WARNING_LINE_BYTES: u64 = 16 * 1024;

#[derive(Debug, Clone)]
pub(crate) struct HostLogConfig {
    pub(crate) active_path: PathBuf,
    pub(crate) retained_dir: PathBuf,
    pub(crate) max_active_bytes: u64,
    pub(crate) max_retained_count: usize,
    pub(crate) max_total_bytes: u64,
    pub(crate) max_warning_categories: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum RotationOutcome {
    NoActiveLog,
    Rotated(PathBuf),
    StderrFallback { reason: String },
}

#[derive(Debug)]
pub(crate) enum HostLogSetup {
    Active {
        path: PathBuf,
        file: File,
        warnings: WarningAggregator,
    },
    StderrFallback {
        reason: String,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum WarningEvent {
    First {
        message: String,
    },
    Suppressed {
        count: u64,
    },
    #[cfg(test)]
    Recovered {
        total_count: u64,
        repeated_count: u64,
    },
    CategoryLimitReached {
        max_categories: usize,
    },
    NotTracked,
}

#[derive(Debug)]
pub(crate) struct WarningAggregator {
    counts: HashMap<String, u64>,
    max_categories: usize,
}

impl WarningAggregator {
    pub(crate) fn new(max_categories: usize) -> Self {
        Self {
            counts: HashMap::new(),
            max_categories,
        }
    }

    pub(crate) fn record(&mut self, category: &str, message: &str) -> WarningEvent {
        if let Some(count) = self.counts.get_mut(category) {
            *count = count.saturating_add(1);
            return WarningEvent::Suppressed { count: *count };
        }
        if self.counts.len() >= self.max_categories {
            return WarningEvent::CategoryLimitReached {
                max_categories: self.max_categories,
            };
        }
        self.counts.insert(category.to_string(), 1);
        WarningEvent::First {
            message: message.to_string(),
        }
    }

    #[cfg(test)]
    pub(crate) fn recover(&mut self, category: &str) -> WarningEvent {
        let Some(total_count) = self.counts.remove(category) else {
            return WarningEvent::NotTracked;
        };
        WarningEvent::Recovered {
            total_count,
            repeated_count: total_count.saturating_sub(1),
        }
    }

    #[cfg(test)]
    pub(crate) fn category_count(&self) -> usize {
        self.counts.len()
    }

    fn summaries(&self) -> Vec<(String, u64)> {
        let mut summaries = self
            .counts
            .iter()
            .filter(|(_, count)| **count > 1)
            .map(|(category, count)| (category.clone(), *count))
            .collect::<Vec<_>>();
        summaries.sort_by(|left, right| left.0.cmp(&right.0));
        summaries
    }
}

pub(crate) struct BoundedLogWriter {
    file: File,
    max_bytes: u64,
    bytes_written: u64,
    warnings: WarningAggregator,
    finished: bool,
}

impl BoundedLogWriter {
    pub(crate) fn new(file: File, max_bytes: u64, warnings: WarningAggregator) -> Self {
        let bytes_written = file.metadata().map(|metadata| metadata.len()).unwrap_or(0);
        Self {
            file,
            max_bytes,
            bytes_written,
            warnings,
            finished: false,
        }
    }

    fn write_bounded(&mut self, bytes: &[u8]) -> io::Result<bool> {
        let incoming = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
        if self.bytes_written.saturating_add(incoming) > self.max_bytes {
            return Ok(false);
        }
        self.file.write_all(bytes)?;
        self.bytes_written = self.bytes_written.saturating_add(incoming);
        Ok(true)
    }

    pub(crate) fn write_warning(
        &mut self,
        category: &str,
        message: &str,
    ) -> io::Result<WarningEvent> {
        let event = self.warnings.record(category, message);
        if matches!(event, WarningEvent::First { .. }) {
            let mut line = message.as_bytes().to_vec();
            line.push(b'\n');
            let _ = self.write_bounded(&line)?;
        }
        Ok(event)
    }

    pub(crate) fn write_raw(&mut self, bytes: &[u8]) -> io::Result<bool> {
        self.write_bounded(bytes)
    }

    pub(crate) fn finish(&mut self) -> io::Result<()> {
        if self.finished {
            return Ok(());
        }
        for (category, total) in self.warnings.summaries() {
            let repeated = total.saturating_sub(1);
            let summary = format!(
                "limux-warning-summary category={category} total={total} repeated={repeated}\n"
            );
            let _ = self.write_bounded(summary.as_bytes())?;
        }
        self.file.flush()?;
        self.finished = true;
        Ok(())
    }
}

fn retained_usage(retained_dir: &Path) -> io::Result<(usize, u64)> {
    let mut count = 0usize;
    let mut bytes = 0u64;
    if !retained_dir.exists() {
        return Ok((count, bytes));
    }
    for entry in fs::read_dir(retained_dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !name.starts_with("limux-host.") || !name.ends_with(".log") {
            continue;
        }
        let metadata = entry.metadata()?;
        if metadata.is_file() {
            count = count.saturating_add(1);
            bytes = bytes.saturating_add(metadata.len());
        }
    }
    Ok((count, bytes))
}

fn available_retained_path(
    retained_dir: &Path,
    sequence: u128,
    attempts: usize,
) -> Option<PathBuf> {
    for collision in 0..=attempts {
        let name = if collision == 0 {
            format!("limux-host.{sequence}.log")
        } else {
            format!("limux-host.{sequence}.{collision}.log")
        };
        let candidate = retained_dir.join(name);
        if !candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

fn rename_no_replace(source: &Path, destination: &Path) -> io::Result<()> {
    let source = CString::new(source.as_os_str().as_bytes())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "source path contains NUL"))?;
    let destination = CString::new(destination.as_os_str().as_bytes()).map_err(|_| {
        io::Error::new(io::ErrorKind::InvalidInput, "destination path contains NUL")
    })?;
    let result = unsafe {
        libc::renameat2(
            libc::AT_FDCWD,
            source.as_ptr(),
            libc::AT_FDCWD,
            destination.as_ptr(),
            libc::RENAME_NOREPLACE,
        )
    };
    if result < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

pub(crate) fn rotate_managed_active(
    config: &HostLogConfig,
    sequence: u128,
) -> io::Result<RotationOutcome> {
    let active = match fs::metadata(&config.active_path) {
        Ok(metadata) if metadata.is_file() => metadata,
        Ok(_) => {
            return Ok(RotationOutcome::StderrFallback {
                reason: "managed active log path is not a regular file".to_string(),
            });
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(RotationOutcome::NoActiveLog);
        }
        Err(error) => return Err(error),
    };
    let (retained_count, retained_bytes) = retained_usage(&config.retained_dir)?;
    if retained_count >= config.max_retained_count {
        return Ok(RotationOutcome::StderrFallback {
            reason: format!(
                "retained log count limit reached ({})",
                config.max_retained_count
            ),
        });
    }
    if retained_bytes
        .saturating_add(active.len())
        .saturating_add(config.max_active_bytes)
        > config.max_total_bytes
    {
        return Ok(RotationOutcome::StderrFallback {
            reason: format!(
                "retained log byte budget would exceed {}",
                config.max_total_bytes
            ),
        });
    }
    fs::create_dir_all(&config.retained_dir)?;
    for _ in 0..=config.max_retained_count {
        let Some(retained_path) =
            available_retained_path(&config.retained_dir, sequence, config.max_retained_count)
        else {
            break;
        };
        match rename_no_replace(&config.active_path, &retained_path) {
            Ok(()) => return Ok(RotationOutcome::Rotated(retained_path)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    Ok(RotationOutcome::StderrFallback {
        reason: "could not reserve a non-clobbering retained log name".to_string(),
    })
}

pub(crate) fn prepare_host_logging(config: &HostLogConfig, sequence: u128) -> HostLogSetup {
    if config.max_active_bytes == 0
        || config.max_retained_count == 0
        || config.max_total_bytes < config.max_active_bytes
        || config.max_warning_categories == 0
    {
        return HostLogSetup::StderrFallback {
            reason: "bounded host log limits are invalid".to_string(),
        };
    }
    let Some(parent) = config.active_path.parent() else {
        return HostLogSetup::StderrFallback {
            reason: "managed active log has no parent directory".to_string(),
        };
    };
    if let Err(error) = fs::create_dir_all(parent) {
        return HostLogSetup::StderrFallback {
            reason: format!("could not create managed log directory: {error}"),
        };
    }
    match rotate_managed_active(config, sequence) {
        Ok(RotationOutcome::NoActiveLog) => match retained_usage(&config.retained_dir) {
            Ok((retained_count, retained_bytes))
                if retained_count < config.max_retained_count
                    && retained_bytes.saturating_add(config.max_active_bytes)
                        <= config.max_total_bytes => {}
            Ok(_) => {
                return HostLogSetup::StderrFallback {
                    reason: "retained logs leave no budget for a new bounded active log"
                        .to_string(),
                };
            }
            Err(error) => {
                return HostLogSetup::StderrFallback {
                    reason: format!("could not inspect retained log budget: {error}"),
                };
            }
        },
        Ok(RotationOutcome::Rotated(_)) => {}
        Ok(RotationOutcome::StderrFallback { reason }) => {
            return HostLogSetup::StderrFallback { reason };
        }
        Err(error) => {
            return HostLogSetup::StderrFallback {
                reason: format!("could not rotate managed active log: {error}"),
            };
        }
    }
    match OpenOptions::new()
        .create_new(true)
        .append(true)
        .open(&config.active_path)
    {
        Ok(file) => HostLogSetup::Active {
            path: config.active_path.clone(),
            file,
            warnings: WarningAggregator::new(config.max_warning_categories),
        },
        Err(error) => HostLogSetup::StderrFallback {
            reason: format!("could not create managed active log: {error}"),
        },
    }
}

fn warning_key(line: &str) -> Option<String> {
    let class = if line.contains("CRITICAL") {
        "critical"
    } else if line.contains("WARNING") || line.contains("warning") {
        "warning"
    } else {
        return None;
    };
    let normalized = line.trim().chars().take(160).collect::<String>();
    Some(format!("{class}:{normalized}"))
}

fn drain_stderr(reader: File, mut writer: BoundedLogWriter) {
    let mut reader = BufReader::new(reader);
    loop {
        let mut line = Vec::new();
        let read = match reader
            .by_ref()
            .take(MAX_WARNING_LINE_BYTES)
            .read_until(b'\n', &mut line)
        {
            Ok(read) => read,
            Err(_) => break,
        };
        if read == 0 {
            break;
        }
        let text = String::from_utf8_lossy(&line);
        let result = if let Some(category) = warning_key(&text) {
            writer.write_warning(&category, text.trim_end_matches('\n'))
        } else {
            writer.write_raw(&line).map(|_| WarningEvent::NotTracked)
        };
        if result.is_err() {
            break;
        }
    }
    let _ = writer.finish();
}

#[cfg(unix)]
pub(crate) fn install_bounded_stderr(
    config: &HostLogConfig,
    sequence: u128,
) -> Result<Option<PathBuf>, String> {
    let (path, file, warnings) = match prepare_host_logging(config, sequence) {
        HostLogSetup::Active {
            path,
            file,
            warnings,
        } => (path, file, warnings),
        HostLogSetup::StderrFallback { reason } => return Err(reason),
    };

    let mut pipe_fds = [0; 2];
    if unsafe { libc::pipe(pipe_fds.as_mut_ptr()) } < 0 {
        return Err(format!(
            "could not create bounded stderr pipe: {}",
            io::Error::last_os_error()
        ));
    }
    let read_file = unsafe { File::from_raw_fd(pipe_fds[0]) };
    let write_fd = pipe_fds[1];
    let writer = BoundedLogWriter::new(file, config.max_active_bytes, warnings);
    let drain = thread::Builder::new()
        .name("limux-bounded-log".to_string())
        .spawn(move || drain_stderr(read_file, writer));
    if let Err(error) = drain {
        unsafe {
            libc::close(write_fd);
        }
        return Err(format!("could not start bounded log drain: {error}"));
    }
    if unsafe { libc::dup2(write_fd, libc::STDERR_FILENO) } < 0 {
        let error = io::Error::last_os_error();
        unsafe {
            libc::close(write_fd);
        }
        return Err(format!("could not redirect stderr: {error}"));
    }
    unsafe {
        libc::close(write_fd);
    }
    Ok(Some(path))
}

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
    fn setup_without_active_log_fails_closed_when_future_budget_is_unavailable() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let config = config(tmp.path());
        fs::create_dir_all(&config.retained_dir).expect("retained dir");
        fs::write(config.retained_dir.join("limux-host.1.log"), vec![b'1'; 80])
            .expect("retained fixture");

        let outcome = prepare_host_logging(&config, 44);

        assert!(matches!(outcome, HostLogSetup::StderrFallback { .. }));
        assert!(!config.active_path.exists());
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

    #[test]
    fn bounded_writer_suppresses_repeats_and_never_exceeds_active_limit() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("bounded-active.log");
        let file = OpenOptions::new()
            .create_new(true)
            .append(true)
            .open(&path)
            .expect("active fixture");
        let mut writer = BoundedLogWriter::new(file, 128, WarningAggregator::new(2));

        assert!(matches!(
            writer.write_warning("gtk_warning", "Gtk-WARNING repeated warning"),
            Ok(WarningEvent::First { .. })
        ));
        assert_eq!(
            writer
                .write_warning("gtk_warning", "Gtk-WARNING repeated warning")
                .expect("repeat two"),
            WarningEvent::Suppressed { count: 2 }
        );
        assert_eq!(
            writer
                .write_warning("gtk_warning", "Gtk-WARNING repeated warning")
                .expect("repeat three"),
            WarningEvent::Suppressed { count: 3 }
        );
        writer.finish().expect("summary flush");

        let content = fs::read_to_string(&path).expect("bounded contents");
        assert_eq!(content.matches("Gtk-WARNING repeated warning").count(), 1);
        assert!(content.contains("category=gtk_warning total=3 repeated=2"));
        assert!(fs::metadata(&path).expect("bounded metadata").len() <= 128);
    }
}
