use std::collections::HashMap;
use std::ffi::CString;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::fd::{AsRawFd, FromRawFd, RawFd};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

const MAX_WARNING_LINE_BYTES: u64 = 16 * 1024;
const STDERR_READ_CHUNK_BYTES: usize = 8 * 1024;
const STDERR_IDLE_TICK: Duration = Duration::from_millis(25);
/// Written to stderr by [`flush_bounded_stderr`] and recognised (never logged)
/// by the drain loop. The leading newline terminates any partial line already
/// in flight, so the barrier is always parsed as a line of its own.
const FLUSH_BARRIER: &[u8] = b"\nlimux-log-flush-barrier\n";
/// Body of [`FLUSH_BARRIER`], i.e. what one drained line has to equal.
const FLUSH_BARRIER_BODY: &[u8] = b"limux-log-flush-barrier";
/// How often [`flush_bounded_stderr`] re-checks for the drain thread's ack.
const FLUSH_POLL_TICK: Duration = Duration::from_millis(1);
/// Last line the drain writes when it gives up on an unusable read end, so the
/// log explains why it stops rather than just ending mid-stream.
const DRAIN_STOPPED_MARKER: &[u8] =
    b"limux-log-drain-stopped read end unusable; stderr is now discarded\n";
/// Written once when the byte cap first drops output. Its length is held
/// back from the usable budget so it always fits inside the cap.
const CAP_MARKER: &[u8] = b"limux-log-cap-reached\n";

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
    states: HashMap<String, WarningState>,
    max_categories: usize,
}

#[derive(Debug)]
struct WarningState {
    total_count: u64,
    summarized_count: u64,
}

impl WarningAggregator {
    pub(crate) fn new(max_categories: usize) -> Self {
        Self {
            states: HashMap::new(),
            max_categories,
        }
    }

    pub(crate) fn record(&mut self, category: &str, message: &str) -> WarningEvent {
        if let Some(state) = self.states.get_mut(category) {
            state.total_count = state.total_count.saturating_add(1);
            return WarningEvent::Suppressed {
                count: state.total_count,
            };
        }
        if self.states.len() >= self.max_categories {
            return WarningEvent::CategoryLimitReached {
                max_categories: self.max_categories,
            };
        }
        self.states.insert(
            category.to_string(),
            WarningState {
                total_count: 1,
                summarized_count: 1,
            },
        );
        WarningEvent::First {
            message: message.to_string(),
        }
    }

    pub(crate) fn recover(&mut self, category: &str) -> WarningEvent {
        let Some(state) = self.states.remove(category) else {
            return WarningEvent::NotTracked;
        };
        WarningEvent::Recovered {
            total_count: state.total_count,
            repeated_count: state.total_count.saturating_sub(1),
        }
    }

    #[cfg(test)]
    pub(crate) fn category_count(&self) -> usize {
        self.states.len()
    }

    fn take_summaries(&mut self) -> Vec<(String, u64)> {
        let mut summaries = self
            .states
            .iter_mut()
            .filter(|(_, state)| {
                state.total_count > 1 && state.total_count > state.summarized_count
            })
            .map(|(category, state)| {
                state.summarized_count = state.total_count;
                (category.clone(), state.total_count)
            })
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
    summary_interval: Duration,
    last_summary_flush: Duration,
    finished: bool,
    cap_reached: bool,
}

impl BoundedLogWriter {
    pub(crate) fn new(file: File, max_bytes: u64, warnings: WarningAggregator) -> Self {
        Self::new_with_summary_interval(file, max_bytes, warnings, Duration::from_secs(60))
    }

    pub(crate) fn new_with_summary_interval(
        file: File,
        max_bytes: u64,
        warnings: WarningAggregator,
        summary_interval: Duration,
    ) -> Self {
        let bytes_written = file.metadata().map(|metadata| metadata.len()).unwrap_or(0);
        Self {
            file,
            max_bytes,
            bytes_written,
            warnings,
            summary_interval,
            last_summary_flush: Duration::ZERO,
            finished: false,
            cap_reached: false,
        }
    }

    /// Content budget, i.e. the cap minus the headroom reserved so the
    /// cap marker is always writable without breaching `max_bytes`.
    fn content_budget(&self) -> u64 {
        self.max_bytes
            .saturating_sub(u64::try_from(CAP_MARKER.len()).unwrap_or(u64::MAX))
    }

    fn write_bounded(&mut self, bytes: &[u8]) -> io::Result<bool> {
        let incoming = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
        if self.bytes_written.saturating_add(incoming) > self.content_budget() {
            self.note_cap_reached()?;
            return Ok(false);
        }
        self.file.write_all(bytes)?;
        self.bytes_written = self.bytes_written.saturating_add(incoming);
        Ok(true)
    }

    /// True once the byte cap has rejected a write.
    pub(crate) fn cap_reached(&self) -> bool {
        self.cap_reached
    }

    /// A2: the cap used to be silent. Rotation only happens at startup, so a
    /// log that fills up simply stops recording for the rest of the process
    /// lifetime with nothing to say why. Leave exactly one short marker
    /// behind the first time output is dropped.
    ///
    /// Headroom for this one line is reserved up front by
    /// [`Self::content_budget`], so "bounded" keeps meaning bounded: the
    /// marker never pushes the file past `max_bytes`.
    fn note_cap_reached(&mut self) -> io::Result<()> {
        if self.cap_reached {
            return Ok(());
        }
        self.cap_reached = true;
        let len = u64::try_from(CAP_MARKER.len()).unwrap_or(u64::MAX);
        if self.bytes_written.saturating_add(len) > self.max_bytes {
            return Ok(());
        }
        self.file.write_all(CAP_MARKER)?;
        self.bytes_written = self.bytes_written.saturating_add(len);
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn write_warning(
        &mut self,
        category: &str,
        message: &str,
    ) -> io::Result<WarningEvent> {
        self.write_warning_at(category, message, self.last_summary_flush)
    }

    pub(crate) fn write_warning_at(
        &mut self,
        category: &str,
        message: &str,
        elapsed: Duration,
    ) -> io::Result<WarningEvent> {
        let event = self.warnings.record(category, message);
        if matches!(event, WarningEvent::First { .. }) {
            let mut line = message.as_bytes().to_vec();
            line.push(b'\n');
            let _ = self.write_bounded(&line)?;
        }
        let _ = self.flush_due(elapsed)?;
        Ok(event)
    }

    pub(crate) fn recover_warning_at(
        &mut self,
        category: &str,
        message: &str,
        elapsed: Duration,
    ) -> io::Result<WarningEvent> {
        let event = self.warnings.recover(category);
        match &event {
            WarningEvent::Recovered {
                total_count,
                repeated_count,
            } => {
                let recovery = format!(
                    "limux-warning-recovery category={category} total={total_count} repeated={repeated_count} {message}\n"
                );
                let _ = self.write_bounded(recovery.as_bytes())?;
            }
            WarningEvent::NotTracked => {
                let mut raw = message.as_bytes().to_vec();
                raw.push(b'\n');
                let _ = self.write_bounded(&raw)?;
            }
            _ => {}
        }
        let _ = self.flush_due(elapsed)?;
        Ok(event)
    }

    pub(crate) fn write_raw(&mut self, bytes: &[u8]) -> io::Result<bool> {
        self.write_bounded(bytes)
    }

    /// One-shot notice that the sink failed and stderr is now being discarded.
    ///
    /// Best-effort by construction: it is written through the same file that
    /// just failed, so it only lands when the failure was transient.
    pub(crate) fn write_degraded_marker(&mut self) -> io::Result<bool> {
        self.write_bounded(b"limux-log-degraded sink write failed; stderr is being discarded\n")
    }

    /// Closing tally of what a degraded sink cost, written when the drain
    /// loop stops. Also best-effort, for the same reason.
    pub(crate) fn write_degraded_summary(
        &mut self,
        failures: u64,
        discarded_bytes: u64,
    ) -> io::Result<bool> {
        let summary = format!(
            "limux-log-degraded-summary sink_failures={failures} discarded_bytes={discarded_bytes}\n"
        );
        self.write_bounded(summary.as_bytes())
    }

    fn flush_summaries(&mut self) -> io::Result<()> {
        for (category, total) in self.warnings.take_summaries() {
            let repeated = total.saturating_sub(1);
            let summary = format!(
                "limux-warning-summary category={category} total={total} repeated={repeated}\n"
            );
            let _ = self.write_bounded(summary.as_bytes())?;
        }
        self.file.flush()
    }

    pub(crate) fn flush_due(&mut self, elapsed: Duration) -> io::Result<bool> {
        if elapsed.saturating_sub(self.last_summary_flush) < self.summary_interval {
            return Ok(false);
        }
        self.flush_summaries()?;
        self.last_summary_flush = elapsed;
        Ok(true)
    }

    /// Push anything held in user space out to the log descriptor.
    ///
    /// `File` is unbuffered, so this is a no-op today; it is called on the
    /// flush-barrier path so the durability guarantee survives someone later
    /// wrapping the sink in a `BufWriter`.
    pub(crate) fn flush_sink(&mut self) -> io::Result<()> {
        self.file.flush()
    }

    pub(crate) fn finish(&mut self) -> io::Result<()> {
        if self.finished {
            return Ok(());
        }
        self.flush_summaries()?;
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

#[derive(Debug, PartialEq, Eq)]
enum LogAction {
    Warning { category: String },
    Recovery { category: String },
    Raw,
}

fn source_category(line: &str) -> String {
    line.split_whitespace()
        .next()
        .unwrap_or("warning")
        .chars()
        .take(64)
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}

fn classify_log_action(line: &str) -> LogAction {
    let lowercase = line.to_ascii_lowercase();
    let category = if lowercase.contains("renderer") && lowercase.contains("context") {
        Some("renderer_context_lost".to_string())
    } else if lowercase.contains("wsl")
        && lowercase.contains("vhd")
        && lowercase.contains("timeout")
    {
        Some("wsl_vhd_wait_timeout".to_string())
    } else if lowercase.contains("warning") || lowercase.contains("critical") {
        Some(source_category(line))
    } else {
        None
    };
    let recovered = lowercase.contains("recovered") || lowercase.contains("restored");
    match (category, recovered) {
        (Some(category), true) => LogAction::Recovery { category },
        (Some(category), false) => LogAction::Warning { category },
        (None, _) => LogAction::Raw,
    }
}

fn process_stderr_line(
    writer: &mut BoundedLogWriter,
    line: &[u8],
    elapsed: Duration,
) -> io::Result<()> {
    let text = String::from_utf8_lossy(line);
    let message = text.trim_end_matches(['\r', '\n']);
    match classify_log_action(message) {
        LogAction::Warning { category } => {
            let event = writer.write_warning_at(&category, message, elapsed)?;
            if matches!(event, WarningEvent::CategoryLimitReached { .. }) {
                let _ = writer.write_raw(line)?;
            }
        }
        LogAction::Recovery { category } => {
            let _ = writer.recover_warning_at(&category, message, elapsed)?;
        }
        LogAction::Raw => {
            let _ = writer.write_raw(line)?;
        }
    }
    Ok(())
}

/// True for the one line [`flush_bounded_stderr`] injects as a barrier.
fn is_flush_barrier(line: &[u8]) -> bool {
    let trimmed = line
        .iter()
        .rposition(|byte| *byte != b'\n' && *byte != b'\r')
        .map(|end| &line[..=end])
        .unwrap_or_default();
    trimmed == FLUSH_BARRIER_BODY
}

fn drain_complete_lines(
    writer: &mut BoundedLogWriter,
    pending: &mut Vec<u8>,
    elapsed: Duration,
    acks: &AtomicU64,
) -> io::Result<()> {
    loop {
        let newline = pending.iter().position(|byte| *byte == b'\n');
        let take = newline.map(|index| index + 1).or_else(|| {
            (pending.len() >= MAX_WARNING_LINE_BYTES as usize)
                .then_some(MAX_WARNING_LINE_BYTES as usize)
        });
        let Some(take) = take else {
            break;
        };
        let line = pending.drain(..take).collect::<Vec<_>>();
        if is_flush_barrier(&line) {
            // Reaching this line means every earlier byte has already been
            // handed to `write(2)` on the log fd, so publishing the ack tells
            // the waiter its output is durable. The barrier itself is never
            // logged.
            writer.flush_sink()?;
            acks.fetch_add(1, Ordering::Release);
            continue;
        }
        process_stderr_line(writer, &line, elapsed)?;
    }
    Ok(())
}

fn set_nonblocking(file: &File) -> io::Result<()> {
    let flags = unsafe { libc::fcntl(file.as_raw_fd(), libc::F_GETFL) };
    if flags < 0 {
        return Err(io::Error::last_os_error());
    }
    if unsafe { libc::fcntl(file.as_raw_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// What the drain loop should do with the outcome of one `read` call.
///
/// Split out from the loop so the control-flow decision is unit-testable
/// without spawning a GUI or a real pipe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DrainAction {
    /// `n` bytes are readable in the chunk buffer.
    Process(usize),
    /// Nothing readable right now; idle briefly, then retry.
    Idle,
    /// Interrupted by a signal; retry immediately.
    Retry,
    /// Every write end is closed; finish up and stop.
    Eof,
    /// The read end itself is unusable; nothing is left to drain.
    ReaderFailed,
}

pub(crate) fn classify_read(result: &io::Result<usize>) -> DrainAction {
    match result {
        Ok(0) => DrainAction::Eof,
        Ok(read) => DrainAction::Process(*read),
        Err(error) if error.kind() == io::ErrorKind::WouldBlock => DrainAction::Idle,
        Err(error) if error.kind() == io::ErrorKind::Interrupted => DrainAction::Retry,
        Err(_) => DrainAction::ReaderFailed,
    }
}

/// Health of the bounded-log sink as observed by the drain thread.
///
/// A failing sink must never stop the drain loop: the write end of the pipe
/// is `STDERR_FILENO` for the whole process, including the GTK main thread.
/// If nobody reads the pipe, the 64 KiB kernel buffer fills and the next
/// `write(2)` from the main thread blocks forever, freezing the GUI. So a
/// sink failure degrades to read-and-discard, and is recorded here instead.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DrainState {
    degraded: bool,
    discarded_bytes: u64,
    sink_failures: u64,
    cap_reached: bool,
}

impl DrainState {
    pub(crate) fn is_degraded(&self) -> bool {
        self.degraded
    }

    pub(crate) fn discarded_bytes(&self) -> u64 {
        self.discarded_bytes
    }

    pub(crate) fn sink_failures(&self) -> u64 {
        self.sink_failures
    }

    /// Record that the log sink failed. Returns `true` the first time, so the
    /// caller can emit a one-shot marker.
    pub(crate) fn note_sink_failure(&mut self) -> bool {
        self.sink_failures = self.sink_failures.saturating_add(1);
        let first = !self.degraded;
        self.degraded = true;
        first
    }

    /// True when the byte cap silently started dropping output.
    pub(crate) fn cap_reached(&self) -> bool {
        self.cap_reached
    }

    pub(crate) fn note_cap_reached(&mut self) {
        self.cap_reached = true;
    }

    pub(crate) fn note_discarded(&mut self, bytes: usize) {
        self.discarded_bytes = self
            .discarded_bytes
            .saturating_add(u64::try_from(bytes).unwrap_or(u64::MAX));
    }
}

/// Point stderr at /dev/null, discarding whatever it referred to before.
///
/// Used when the drain thread stops: from that moment fd 2 is the write end of
/// a pipe nobody will ever read again.
#[cfg(unix)]
fn detach_stderr_to_null() {
    let Ok(dev_null) = CString::new("/dev/null") else {
        return;
    };
    let opened = unsafe { libc::open(dev_null.as_ptr(), libc::O_WRONLY | libc::O_CLOEXEC) };
    if opened < 0 {
        return;
    }
    unsafe {
        libc::dup2(opened, libc::STDERR_FILENO);
        libc::close(opened);
    }
}

/// M2: before PR #88 a broken log sink could not take the host down, because
/// stderr *was* the log file. Now stderr is a pipe, so the moment the drain
/// thread stops — `DrainAction::ReaderFailed`, or a panic inside the thread —
/// the read end closes and the very next `eprintln!` from the GTK main thread
/// hits EPIPE. `eprintln!` panics on a failed write (verified: a probe whose
/// reader is gone dies with rc=101), and that panic's own message goes to the
/// same dead pipe, so the host dies with no diagnostic at all. Children that
/// inherited fd 2 get SIGPIPE.
///
/// Repointing stderr at /dev/null on the way out turns all of that into
/// "stderr is silently discarded", which is survivable. Written as a guard so
/// it also runs when the drain thread unwinds.
///
/// Residual, deliberately not fixed here: diagnostics stop. Redirecting to the
/// log file instead would keep them, but that bypasses the byte cap this
/// module exists to enforce, and in the sink-failure case it would be writing
/// to the descriptor that just failed. `DRAIN_STOPPED_MARKER` records the
/// transition in the log so the silence is explained.
#[cfg(unix)]
struct StderrDetachGuard;

#[cfg(unix)]
impl Drop for StderrDetachGuard {
    fn drop(&mut self) {
        detach_stderr_to_null();
    }
}

fn drain_stderr(mut reader: File, mut writer: BoundedLogWriter, acks: &AtomicU64) -> DrainState {
    let started = Instant::now();
    let mut pending = Vec::with_capacity(MAX_WARNING_LINE_BYTES as usize);
    let mut chunk = [0u8; STDERR_READ_CHUNK_BYTES];
    let mut state = DrainState::default();
    loop {
        let elapsed = started.elapsed();
        if !state.is_degraded() && writer.flush_due(elapsed).is_err() {
            note_sink_failure(&mut state, &mut writer, &mut pending);
        }
        let result = reader.read(&mut chunk);
        match classify_read(&result) {
            DrainAction::Eof => {
                if !state.is_degraded() && !pending.is_empty() {
                    let _ = process_stderr_line(&mut writer, &pending, elapsed);
                }
                break;
            }
            DrainAction::Process(read) => {
                if state.is_degraded() {
                    // Sink is dead, but the pipe MUST keep being drained or
                    // the GTK main thread blocks on write forever.
                    state.note_discarded(read);
                } else {
                    pending.extend_from_slice(&chunk[..read]);
                    if drain_complete_lines(&mut writer, &mut pending, elapsed, acks).is_err() {
                        note_sink_failure(&mut state, &mut writer, &mut pending);
                    }
                }
            }
            DrainAction::Idle => thread::sleep(STDERR_IDLE_TICK),
            DrainAction::Retry => {}
            // The read end is gone; there is nothing left to drain and the
            // loop would otherwise spin at 100% CPU. Leave a note, because
            // from here on stderr is discarded and the log would otherwise
            // just stop mid-sentence with no explanation.
            DrainAction::ReaderFailed => {
                if !state.is_degraded() {
                    let _ = writer.write_raw(DRAIN_STOPPED_MARKER);
                }
                break;
            }
        }
    }
    if writer.cap_reached() {
        state.note_cap_reached();
    }
    if state.is_degraded() || state.cap_reached() {
        let _ = writer.write_degraded_summary(state.sink_failures(), state.discarded_bytes());
    }
    let _ = writer.finish();
    state
}

/// Mark the sink degraded and make that fact observable, best-effort.
fn note_sink_failure(state: &mut DrainState, writer: &mut BoundedLogWriter, pending: &mut Vec<u8>) {
    let first = state.note_sink_failure();
    state.note_discarded(pending.len());
    pending.clear();
    if first {
        // Best-effort: the sink just failed, so this may well fail too. It
        // lands when the failure was transient (e.g. one bad line).
        let _ = writer.write_degraded_marker();
    }
}

#[cfg(unix)]
pub(crate) fn install_bounded_stderr(
    config: &HostLogConfig,
    sequence: u128,
) -> Result<Option<PathBuf>, String> {
    // No descriptor reservation or relocation happens here, deliberately.
    //
    // Both used to exist to defend against "the host was started with stderr
    // closed, so the first open(2) lands on fd 2 and the dup2 below clobbers
    // it". That precondition cannot occur in a Rust binary: the standard
    // library runs `sanitize_standard_fds()` before `main`, so 0/1/2 are
    // always open (on /dev/null) no matter how the process was launched, and
    // the first open(2) therefore lands on fd 3 or above. Verified by
    // launching a probe with `2>&-` and with `0<&- 1>&- 2>&-`: fd 2 was
    // /dev/null (ino 4, mode 020666) in both cases and the first open returned
    // fd 3. `standard_descriptors_are_open_before_main_even_when_launched_closed`
    // pins that invariant.
    //
    // Roughly 90 lines of unsafe fd surgery guarding an unreachable branch is
    // a liability, not insurance — it can only ever misfire.
    let (path, file, warnings) = match prepare_host_logging(config, sequence) {
        HostLogSetup::Active {
            path,
            file,
            warnings,
        } => (path, file, warnings),
        HostLogSetup::StderrFallback { reason } => return Err(reason),
    };

    let mut pipe_fds = [0 as RawFd; 2];
    // O_CLOEXEC is load-bearing and stays: without it every spawned child
    // inherits the read end, which keeps the pipe from ever reaching EOF and
    // leaks host stderr into shells. Covered by
    // `spawned_children_do_not_inherit_the_stderr_pipe_read_end`.
    if unsafe { libc::pipe2(pipe_fds.as_mut_ptr(), libc::O_CLOEXEC) } < 0 {
        return Err(format!(
            "could not create bounded stderr pipe: {}",
            io::Error::last_os_error()
        ));
    }
    let (read_fd, write_fd) = (pipe_fds[0], pipe_fds[1]);
    let read_file = unsafe { File::from_raw_fd(read_fd) };
    if let Err(error) = set_nonblocking(&read_file) {
        unsafe {
            libc::close(write_fd);
        }
        return Err(format!(
            "could not make bounded log drain nonblocking: {error}"
        ));
    }
    let writer = BoundedLogWriter::new(file, config.max_active_bytes, warnings);
    let acks = Arc::new(AtomicU64::new(0));
    let drain_acks = Arc::clone(&acks);
    let drain = thread::Builder::new()
        .name(DRAIN_THREAD_NAME.to_string())
        // The guard lives in the installed thread only, never in
        // `drain_stderr` itself: unit tests drive that function directly and
        // must not have the test binary's stderr yanked out from under them.
        .spawn(move || {
            let _detach_stderr = StderrDetachGuard;
            drain_stderr(read_file, writer, &drain_acks)
        });
    // The drain thread outlives this function by design: fd 2 is process-wide
    // and every spawned child inherits it, so the pipe cannot reach EOF while
    // the app is running and `join()` here would block forever. Detaching is
    // therefore correct — but detaching alone loses everything still in the
    // 64 KiB pipe buffer when `exit(2)` kills the thread, which is why
    // `flush_bounded_stderr` exists. Bind the handle explicitly so the
    // detachment stays a decision rather than a dropped `Result` variant.
    let _detached_drain = match drain {
        Ok(handle) => handle,
        Err(error) => {
            unsafe {
                libc::close(write_fd);
            }
            return Err(format!("could not start bounded log drain: {error}"));
        }
    };
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
    // Publish only once fd 2 really is the pipe, so a flush can never write a
    // barrier nobody will read and then wait out the whole timeout.
    let _ = DRAIN_ACKS.set(acks);
    Ok(Some(path))
}

/// Ack counter of the installed drain thread, published by
/// [`install_bounded_stderr`] once stderr is actually wired to the pipe.
static DRAIN_ACKS: OnceLock<Arc<AtomicU64>> = OnceLock::new();

const DRAIN_THREAD_NAME: &str = "limux-bounded-log";

/// Block (up to `timeout`) until everything already written to stderr has
/// reached the managed log file.
///
/// Before PR #88 stderr *was* the log file: `dup2(log_fd, 2)` made every write
/// synchronous, so `exit(2)` could not lose anything. The pipe + drain-thread
/// design broke that — `std::process::exit` kills the drain thread wherever it
/// happens to be, typically mid `STDERR_IDLE_TICK` sleep, and the pipe buffer
/// plus the thread's `pending` buffer die with it. Shutdown and panic paths
/// call this to restore the old guarantee.
///
/// Implemented as a barrier rather than "close the write end and join": fd 2
/// is inherited by every child, so closing the parent's copy does not produce
/// EOF and a join would stall for the full timeout on every clean exit. The
/// ack instead proves positively that the drain consumed past this point.
///
/// Returns `true` when the drain acknowledged, or when no bounded stderr is
/// installed (nothing to flush). Costs one blank line in the log per call.
#[cfg(unix)]
pub(crate) fn flush_bounded_stderr(timeout: Duration) -> bool {
    let Some(acks) = DRAIN_ACKS.get() else {
        return true;
    };
    // A drain-thread panic runs the panic hook on the drain thread itself;
    // waiting there would be waiting on ourselves for the whole timeout.
    if thread::current().name() == Some(DRAIN_THREAD_NAME) {
        return false;
    }
    let start = acks.load(Ordering::Acquire);
    // Raw `write(2)`: `eprintln!` panics on EPIPE, and this runs on the
    // shutdown and panic paths where a fresh panic would be fatal. The barrier
    // is far below PIPE_BUF, so the write is atomic — no interleaving with
    // another thread's stderr.
    let written = unsafe {
        libc::write(
            libc::STDERR_FILENO,
            FLUSH_BARRIER.as_ptr().cast::<libc::c_void>(),
            FLUSH_BARRIER.len(),
        )
    };
    if written != isize::try_from(FLUSH_BARRIER.len()).unwrap_or(isize::MAX) {
        return false;
    }
    let deadline = Instant::now() + timeout;
    loop {
        if acks.load(Ordering::Acquire) > start {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        thread::sleep(FLUSH_POLL_TICK);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;
    use std::time::Duration;

    /// A sink whose every write fails: an existing file opened read-only.
    fn failing_sink(root: &std::path::Path) -> File {
        let path = root.join("readonly-sink.log");
        fs::write(&path, b"").expect("sink fixture");
        File::open(&path).expect("open read-only sink")
    }

    // ---------------------------------------------------------------------
    // Subprocess probe harness.
    //
    // `install_bounded_stderr` takes over fd 2 for the whole process, so any
    // test that drives the *real* installer in-process corrupts descriptors
    // owned by the ~650 tests running alongside it — which is why the earlier
    // end-to-end test had to be `#[ignore]`d. Running the installer in a child
    // process instead gives each probe its own descriptor table, so these
    // tests are parallel-safe and run by default.
    // ---------------------------------------------------------------------

    const PROBE_MODE_ENV: &str = "LIMUX_HOST_LOG_PROBE_MODE";
    const PROBE_DIR_ENV: &str = "LIMUX_HOST_LOG_PROBE_DIR";
    /// Lines the loss probe emits before it exits. Large enough that far more
    /// than one 64 KiB pipe buffer is still unread at exit, so "the drain got
    /// lucky and had already consumed everything" is not a realistic outcome.
    const PROBE_LOSS_LINES: usize = 20_000;
    const PROBE_TAIL_MARKER: &str = "limux-loss-probe-tail";
    const PROBE_EXIT_CODE: i32 = 3;
    const PROBE_FD_LISTING: &str = "child-fds.txt";
    const PROBE_STD_FDS: &str = "std-fds.txt";
    const PROBE_SURVIVED: &str = "survived.txt";
    /// Dumps `fd<TAB>target` for every descriptor the shell inherited. Parsed
    /// rather than `ls -l` so the format is not locale-dependent.
    const FD_LISTING_SCRIPT: &str =
        r#"for f in /proc/self/fd/*; do printf '%s\t%s\n' "${f##*/}" "$(readlink "$f")"; done"#;

    fn probe_config(root: &Path) -> HostLogConfig {
        HostLogConfig {
            active_path: root.join("managed/limux-host.current.log"),
            retained_dir: root.join("managed/retained"),
            max_active_bytes: 16 * 1024 * 1024,
            max_retained_count: 4,
            max_total_bytes: 64 * 1024 * 1024,
            max_warning_categories: 16,
        }
    }

    fn probe_command(mode: &str, dir: &Path) -> Command {
        let exe = std::env::current_exe().expect("test binary path");
        let mut command = Command::new(exe);
        command
            .args([
                "--exact",
                "host_log::tests::probe_child",
                "--nocapture",
                "--test-threads=1",
            ])
            .env(PROBE_MODE_ENV, mode)
            .env(PROBE_DIR_ENV, dir);
        command
    }

    /// Re-invoke this test binary, running only [`probe_child`], in `mode`.
    fn run_probe(mode: &str, dir: &Path) -> std::process::Output {
        probe_command(mode, dir)
            .output()
            .expect("probe child must spawn")
    }

    /// As [`run_probe`], but the child is exec'd with fds 0, 1 and 2 genuinely
    /// closed — the launch condition the deleted fd-reservation layer claimed
    /// to defend against.
    #[cfg(unix)]
    fn run_probe_with_closed_std_fds(mode: &str, dir: &Path) -> std::process::Output {
        use std::os::unix::process::CommandExt;

        let mut command = probe_command(mode, dir);
        // SAFETY: between fork and exec only async-signal-safe calls are made.
        unsafe {
            command.pre_exec(|| {
                for fd in [libc::STDIN_FILENO, libc::STDOUT_FILENO, libc::STDERR_FILENO] {
                    libc::close(fd);
                }
                Ok(())
            });
        }
        command.output().expect("probe child must spawn")
    }

    /// `0=open 1=open 2=open` for whichever standard descriptors are live.
    #[cfg(unix)]
    fn standard_fd_state() -> String {
        [libc::STDIN_FILENO, libc::STDOUT_FILENO, libc::STDERR_FILENO]
            .iter()
            .map(|fd| {
                let open = unsafe { libc::fcntl(*fd, libc::F_GETFD) } >= 0;
                format!("{fd}={}", if open { "open" } else { "closed" })
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// The child half of the probe harness. Inert (and trivially passing)
    /// unless [`PROBE_MODE_ENV`] is set, so a normal test run ignores it.
    #[test]
    fn probe_child() {
        let Ok(mode) = std::env::var(PROBE_MODE_ENV) else {
            return;
        };
        let dir = PathBuf::from(std::env::var_os(PROBE_DIR_ENV).expect("probe dir"));
        // Sampled before this process opens anything of its own.
        let inherited_std_fds = standard_fd_state();
        let config = probe_config(&dir);
        fs::create_dir_all(config.active_path.parent().expect("parent")).expect("active dir");
        fs::create_dir_all(&config.retained_dir).expect("retained dir");
        install_bounded_stderr(&config, 1).expect("probe install must succeed");

        match mode.as_str() {
            // H1: everything written before the flush must survive exit(2).
            "loss" => {
                for index in 0..PROBE_LOSS_LINES {
                    eprintln!("limux-loss-probe {index:05}");
                }
                eprintln!("{PROBE_TAIL_MARKER}");
                // The call site under test. Reverting *this line* is what the
                // regression check reverts.
                flush_bounded_stderr(Duration::from_secs(10));
                std::process::exit(PROBE_EXIT_CODE);
            }
            // H2: a spawned child must inherit the write end (fd 2, so its
            // stderr is logged) and nothing else belonging to the pipe.
            "cloexec" => {
                let listing = Command::new("/bin/sh")
                    .arg("-c")
                    .arg(FD_LISTING_SCRIPT)
                    .stderr(std::process::Stdio::inherit())
                    .output()
                    .expect("fd listing child must spawn");
                fs::write(dir.join(PROBE_FD_LISTING), listing.stdout).expect("fd listing");
                flush_bounded_stderr(Duration::from_secs(10));
                std::process::exit(PROBE_EXIT_CODE);
            }
            // M2: kill the drain thread's read end and check the host lives.
            "deaddrain" => {
                let pipe = fs::read_link("/proc/self/fd/2").expect("stderr link");
                let read_fd = fs::read_dir("/proc/self/fd")
                    .expect("fd dir")
                    .filter_map(|entry| entry.ok())
                    .filter_map(|entry| {
                        let fd = entry.file_name().to_string_lossy().parse::<RawFd>().ok()?;
                        (fd != libc::STDERR_FILENO && fs::read_link(entry.path()).ok()? == pipe)
                            .then_some(fd)
                    })
                    .next()
                    .expect("the drain thread's read end must be findable");
                // Fault injection: atomically replace the pipe's only read end
                // with /dev/null. `dup2` closes the target as part of the same
                // call, so unlike a bare `close` there is no window in which
                // the descriptor number can be recycled under the drain
                // thread. The drain's next read(2) returns 0 (EOF) and the
                // thread stops — with a live write end still on fd 2, which is
                // precisely the M2 hazard.
                let dev_null = CString::new("/dev/null").expect("path");
                let opened = unsafe { libc::open(dev_null.as_ptr(), libc::O_RDONLY) };
                assert!(opened >= 0, "open /dev/null");
                assert!(unsafe { libc::dup2(opened, read_fd) } >= 0, "dup2");
                unsafe { libc::close(opened) };

                // Wait (ceiling only — no timing assertion) for the drain to
                // notice and for its guard to repoint stderr.
                let deadline = Instant::now() + Duration::from_secs(5);
                while Instant::now() < deadline
                    && fs::read_link("/proc/self/fd/2").ok().as_deref() == Some(pipe.as_path())
                {
                    thread::sleep(Duration::from_millis(5));
                }

                // The load-bearing line: before the guard existed this
                // panicked on EPIPE and took the whole host down.
                eprintln!("limux-deaddrain-probe");
                fs::write(dir.join(PROBE_SURVIVED), b"survived").expect("survival report");
                std::process::exit(PROBE_EXIT_CODE);
            }
            // H3: report the standard descriptors this process was exec'd
            // with, and prove the installer works without reserving them.
            "stdfds" => {
                eprintln!("{PROBE_TAIL_MARKER}");
                flush_bounded_stderr(Duration::from_secs(10));
                fs::write(dir.join(PROBE_STD_FDS), inherited_std_fds).expect("std fd report");
                std::process::exit(PROBE_EXIT_CODE);
            }
            other => panic!("unknown probe mode {other:?}"),
        }
    }

    /// H1 regression: PR #88 replaced `dup2(log_fd, 2)` — where every stderr
    /// write was synchronous and loss was impossible — with a pipe drained by
    /// a detached thread. `std::process::exit` kills that thread wherever it
    /// is (usually mid `STDERR_IDLE_TICK` sleep), and the 64 KiB pipe buffer
    /// plus the thread's pending line die with it. Measured before the fix:
    /// runs that captured 0 of 200 emitted lines.
    #[cfg(unix)]
    #[test]
    fn stderr_written_before_shutdown_survives_process_exit() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let output = run_probe("loss", tmp.path());

        assert_eq!(
            output.status.code(),
            Some(PROBE_EXIT_CODE),
            "probe child must reach its exit: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let log = fs::read_to_string(probe_config(tmp.path()).active_path).expect("probe log");
        assert!(
            log.contains(PROBE_TAIL_MARKER),
            "the last line written before exit never reached the log — stderr is \
             being lost at shutdown (PR #88 regression)"
        );
        let captured = log.matches("limux-loss-probe ").count();
        assert_eq!(
            captured,
            PROBE_LOSS_LINES,
            "lost {} of {PROBE_LOSS_LINES} lines at shutdown",
            PROBE_LOSS_LINES.saturating_sub(captured)
        );
    }

    /// H2 regression: the `O_CLOEXEC` on the installer's `pipe2` had no test
    /// defence — removing it left the suite fully green, because the two
    /// pre-existing pipe tests build their own `libc::pipe` and never touch
    /// `install_bounded_stderr`. Without it, every spawned child inherits the
    /// pipe's READ end, so the pipe can never reach EOF and host stderr leaks
    /// into shells.
    ///
    /// Asserts against a real child of the real installer: fd 2 must be the
    /// pipe (that inheritance is deliberate — child stderr belongs in the log)
    /// and no descriptor above 2 may be one.
    #[cfg(unix)]
    #[test]
    fn spawned_children_do_not_inherit_the_stderr_pipe_read_end() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let output = run_probe("cloexec", tmp.path());
        assert_eq!(
            output.status.code(),
            Some(PROBE_EXIT_CODE),
            "probe child must reach its exit: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let listing =
            fs::read_to_string(tmp.path().join(PROBE_FD_LISTING)).expect("child fd listing");
        let descriptors = listing
            .lines()
            .filter_map(|line| line.split_once('\t'))
            .filter_map(|(fd, target)| fd.parse::<i32>().ok().map(|fd| (fd, target)))
            .collect::<Vec<_>>();
        assert!(
            !descriptors.is_empty(),
            "fd listing must not be empty: {listing:?}"
        );

        // Both ends of one pipe share an inode, so `pipe:[N]` identifies the
        // bounded-log pipe specifically. Matching on that rather than on "any
        // pipe" keeps the test immune to unrelated descriptors the harness
        // leaks in from tests running in parallel.
        let (_, log_pipe) = descriptors
            .iter()
            .find(|(fd, target)| *fd == libc::STDERR_FILENO && target.starts_with("pipe:"))
            .unwrap_or_else(|| {
                panic!(
                    "the child's stderr must still be the log pipe, otherwise this \
                     test is not observing the installed pipe at all: {listing:?}"
                )
            });

        let leaked = descriptors
            .iter()
            .filter(|(fd, target)| *fd > libc::STDERR_FILENO && target == log_pipe)
            .collect::<Vec<_>>();
        assert!(
            leaked.is_empty(),
            "a spawned child inherited the bounded-log pipe at {leaked:?} as well as \
             stderr — the installer's pipe is missing O_CLOEXEC, so the pipe can \
             never reach EOF and host stderr leaks into children: {listing:?}"
        );
    }

    /// M2 regression: before PR #88 a broken log sink could not kill the host,
    /// because stderr *was* the log file. Now stderr is a pipe, so when the
    /// drain thread stops the read end closes and the next `eprintln!` from
    /// the GTK main thread hits EPIPE — which `eprintln!` turns into a panic
    /// (verified independently: a probe whose reader is gone exits rc=101),
    /// with the panic message going to the same dead pipe. The host would die
    /// silently.
    ///
    /// Fault-injects exactly that (close the drain's read end so its next
    /// `read(2)` returns EBADF) and asserts the process survives a subsequent
    /// `eprintln!` and still exits normally.
    #[cfg(unix)]
    #[test]
    fn a_dead_drain_thread_does_not_kill_the_host() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let output = run_probe("deaddrain", tmp.path());

        assert_ne!(
            output.status.code(),
            Some(101),
            "the probe panicked after the drain thread died — writing to stderr \
             with no reader is fatal again: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            output.status.code(),
            Some(PROBE_EXIT_CODE),
            "probe child must reach its exit: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            fs::read_to_string(tmp.path().join(PROBE_SURVIVED)).unwrap_or_default(),
            "survived",
            "the probe never got past the eprintln that follows a dead drain"
        );
    }

    #[test]
    fn flush_barrier_is_recognised_only_as_a_whole_line() {
        assert!(is_flush_barrier(b"limux-log-flush-barrier\n"));
        assert!(is_flush_barrier(b"limux-log-flush-barrier\r\n"));
        assert!(is_flush_barrier(b"limux-log-flush-barrier"));
        assert!(!is_flush_barrier(b"prefix limux-log-flush-barrier\n"));
        assert!(!is_flush_barrier(b"limux-log-flush-barrier suffix\n"));
        assert!(!is_flush_barrier(b"\n"));
        assert!(!is_flush_barrier(b""));
    }

    #[test]
    fn drain_acks_a_flush_barrier_without_logging_it() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let mut fds = [0 as RawFd; 2];
        assert_eq!(unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC) }, 0);
        let reader = unsafe { File::from_raw_fd(fds[0]) };
        let mut write_end = unsafe { File::from_raw_fd(fds[1]) };
        set_nonblocking(&reader).expect("nonblocking reader");

        let path = tmp.path().join("barrier.log");
        let file = OpenOptions::new()
            .create_new(true)
            .append(true)
            .open(&path)
            .expect("active fixture");
        let writer = BoundedLogWriter::new(file, u64::MAX, WarningAggregator::new(4));
        let acks = Arc::new(AtomicU64::new(0));
        let drain_acks = Arc::clone(&acks);
        let drain = thread::spawn(move || drain_stderr(reader, writer, &drain_acks));

        write_end.write_all(b"before-barrier\n").expect("feed");
        write_end.write_all(FLUSH_BARRIER).expect("feed barrier");
        drop(write_end);
        drain.join().expect("drain thread");

        assert_eq!(
            acks.load(Ordering::Acquire),
            1,
            "barrier must be acked once"
        );
        let body = fs::read_to_string(&path).expect("barrier log");
        assert!(body.contains("before-barrier"), "log: {body:?}");
        assert!(
            !body.contains("limux-log-flush-barrier"),
            "the barrier is a control line and must never be logged: {body:?}"
        );
    }

    #[test]
    fn classify_read_maps_outcomes_to_drain_actions() {
        assert_eq!(classify_read(&Ok(0)), DrainAction::Eof);
        assert_eq!(classify_read(&Ok(17)), DrainAction::Process(17));
        assert_eq!(
            classify_read(&Err(io::Error::from(io::ErrorKind::WouldBlock))),
            DrainAction::Idle
        );
        assert_eq!(
            classify_read(&Err(io::Error::from(io::ErrorKind::Interrupted))),
            DrainAction::Retry
        );
        assert_eq!(
            classify_read(&Err(io::Error::from(io::ErrorKind::BrokenPipe))),
            DrainAction::ReaderFailed
        );
    }

    #[test]
    fn drain_state_reports_first_failure_once_and_accumulates_discards() {
        let mut state = DrainState::default();
        assert!(!state.is_degraded());

        assert!(state.note_sink_failure(), "first failure must be reported");
        assert!(
            !state.note_sink_failure(),
            "subsequent failures must not re-report"
        );
        assert!(state.is_degraded());
        assert_eq!(state.sink_failures(), 2);

        state.note_discarded(100);
        state.note_discarded(23);
        assert_eq!(state.discarded_bytes(), 123);
    }

    /// A1 regression: the drain loop must never stop draining while a write
    /// end is open. The write end is process-wide stderr, so if the loop
    /// stops, the 64 KiB pipe buffer fills and the next write from the GTK
    /// main thread blocks forever — a full GUI freeze.
    #[cfg(unix)]
    #[test]
    fn sink_failure_never_stops_draining_the_pipe() {
        use std::sync::mpsc;

        let tmp = tempfile::tempdir().expect("tempdir");
        let mut fds = [0i32; 2];
        assert_eq!(
            unsafe { libc::pipe(fds.as_mut_ptr()) },
            0,
            "pipe() must succeed"
        );
        let reader = unsafe { File::from_raw_fd(fds[0]) };
        let mut write_end = unsafe { File::from_raw_fd(fds[1]) };
        set_nonblocking(&reader).expect("nonblocking reader");

        let writer = BoundedLogWriter::new(failing_sink(tmp.path()), u64::MAX, {
            WarningAggregator::new(4)
        });
        let acks = Arc::new(AtomicU64::new(0));
        let drain = thread::spawn(move || drain_stderr(reader, writer, &acks));

        // Push far more than one pipe buffer (64 KiB on Linux) through the
        // write end, exactly as the GTK main thread would.
        let (tx, rx) = mpsc::channel();
        let feeder = thread::spawn(move || {
            let line = vec![b'x'; 4096];
            for _ in 0..64 {
                if write_end.write_all(&line).is_err() || write_end.write_all(b"\n").is_err() {
                    break;
                }
            }
            drop(write_end);
            let _ = tx.send(());
        });

        assert!(
            rx.recv_timeout(Duration::from_secs(15)).is_ok(),
            "writes to stderr blocked: the drain loop stopped draining after a sink \
             failure, which freezes the GTK main thread (GUI-hang regression)"
        );

        feeder.join().expect("feeder thread");
        let state = drain.join().expect("drain thread");
        assert!(state.is_degraded(), "sink failure must be recorded");
        assert!(state.sink_failures() >= 1);
        assert!(
            state.discarded_bytes() >= 64 * 1024,
            "drain must have kept consuming past one pipe buffer, discarded {}",
            state.discarded_bytes()
        );
    }

    /// A1 regression, production shape: in the real host the pipe fds are
    /// inherited by every spawned child (they are created without
    /// `O_CLOEXEC`), so the read end stays open even after the drain thread
    /// stops. Nobody is left draining, the 64 KiB buffer fills, and the next
    /// stderr write from the GTK main thread blocks forever.
    ///
    /// Holding a duplicate of the read end reproduces exactly that shape.
    #[cfg(unix)]
    #[test]
    fn sink_failure_does_not_block_stderr_writers_while_read_end_stays_open() {
        use std::sync::mpsc;

        let tmp = tempfile::tempdir().expect("tempdir");
        let mut fds = [0i32; 2];
        assert_eq!(
            unsafe { libc::pipe(fds.as_mut_ptr()) },
            0,
            "pipe() must succeed"
        );
        let reader = unsafe { File::from_raw_fd(fds[0]) };
        let mut write_end = unsafe { File::from_raw_fd(fds[1]) };
        set_nonblocking(&reader).expect("nonblocking reader");

        // Stand in for a forked child that inherited the read end: the pipe
        // now never reaches EOF just because the drain thread went away.
        let inherited = unsafe { libc::dup(fds[0]) };
        assert!(inherited >= 0, "dup of read end must succeed");
        let _inherited_read_end = unsafe { File::from_raw_fd(inherited) };

        let writer = BoundedLogWriter::new(failing_sink(tmp.path()), u64::MAX, {
            WarningAggregator::new(4)
        });
        let acks = Arc::new(AtomicU64::new(0));
        let drain = thread::spawn(move || drain_stderr(reader, writer, &acks));

        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let line = vec![b'x'; 4096];
            for _ in 0..64 {
                if write_end.write_all(&line).is_err() || write_end.write_all(b"\n").is_err() {
                    break;
                }
            }
            drop(write_end);
            let _ = tx.send(());
        });

        assert!(
            rx.recv_timeout(Duration::from_secs(15)).is_ok(),
            "stderr writes blocked with the pipe read end still open: this is the \
             GUI freeze — the GTK main thread would be stuck in write(2) forever"
        );

        let state = drain.join().expect("drain thread");
        assert!(state.is_degraded());
        assert!(state.discarded_bytes() >= 64 * 1024);
    }

    /// A2 regression: hitting the byte cap used to be completely silent, so a
    /// full log just stopped recording for the rest of the process lifetime.
    #[test]
    fn cap_reached_leaves_exactly_one_marker_and_stays_within_the_cap() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("capped.log");
        let file = OpenOptions::new()
            .create_new(true)
            .append(true)
            .open(&path)
            .expect("active fixture");
        let mut writer = BoundedLogWriter::new(file, 128, WarningAggregator::new(4));

        assert!(!writer.cap_reached(), "cap must start unreached");

        let line = vec![b'y'; 40];
        let mut rejected = 0;
        for _ in 0..20 {
            if !writer.write_raw(&line).expect("bounded write") {
                rejected += 1;
            }
        }

        assert!(rejected > 0, "the cap must actually reject writes");
        assert!(writer.cap_reached(), "cap state must be observable");

        let contents = fs::read_to_string(&path).expect("capped contents");
        assert_eq!(
            contents.matches("limux-log-cap-reached").count(),
            1,
            "exactly one marker, not one per dropped write: {contents:?}"
        );
        assert!(
            fs::metadata(&path).expect("capped metadata").len() <= 128,
            "the marker must not push the log past its cap"
        );
    }

    #[test]
    fn drain_reports_a_capped_sink() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let mut fds = [0 as RawFd; 2];
        assert_eq!(unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC) }, 0);
        let reader = unsafe { File::from_raw_fd(fds[0]) };
        let mut write_end = unsafe { File::from_raw_fd(fds[1]) };
        set_nonblocking(&reader).expect("nonblocking reader");

        let path = tmp.path().join("drain-capped.log");
        let file = OpenOptions::new()
            .create_new(true)
            .append(true)
            .open(&path)
            .expect("active fixture");
        let writer = BoundedLogWriter::new(file, 96, WarningAggregator::new(4));
        let acks = Arc::new(AtomicU64::new(0));
        let drain = thread::spawn(move || drain_stderr(reader, writer, &acks));

        for _ in 0..40 {
            write_end.write_all(&[b'z'; 32]).expect("feed");
            write_end.write_all(b"\n").expect("feed newline");
        }
        drop(write_end);

        let state = drain.join().expect("drain thread");
        assert!(
            state.cap_reached(),
            "the drain thread must report that the sink hit its cap"
        );
        assert!(
            fs::read_to_string(&path)
                .expect("capped contents")
                .contains("limux-log-cap-reached"),
            "the cap marker must reach the log"
        );
    }

    /// Replaces the deleted fd-reservation layer's tests.
    ///
    /// That layer (`reserve_standard_fds`, `relocate_above_stderr`,
    /// `relocate_file_above_stderr` — ~90 lines of unsafe fd surgery) existed
    /// for one documented scenario: the host started with stderr closed, so
    /// the pipe's read end lands on fd 2 and the installing `dup2` destroys
    /// it. That scenario is unreachable in a Rust binary, because the standard
    /// library reopens 0/1/2 on /dev/null before `main` runs. Its tests only
    /// passed because they manufactured the condition mid-process with
    /// `close(2)`, which nothing in this binary does.
    ///
    /// This pins the real invariant instead, on the real launch path: exec the
    /// probe with 0, 1 and 2 genuinely closed, and assert the child still finds
    /// them open — and that the installer logs correctly with no reservation.
    #[cfg(unix)]
    #[test]
    fn standard_descriptors_are_open_before_main_even_when_launched_closed() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let output = run_probe_with_closed_std_fds("stdfds", tmp.path());
        assert_eq!(
            output.status.code(),
            Some(PROBE_EXIT_CODE),
            "probe child must reach its exit"
        );

        let observed =
            fs::read_to_string(tmp.path().join(PROBE_STD_FDS)).expect("std fd observation");
        assert_eq!(
            observed.trim(),
            "0=open 1=open 2=open",
            "the Rust runtime is expected to reopen every standard descriptor \
             before main, which is what makes the fd-reservation layer dead code. \
             If this ever fails, that layer has to come back."
        );

        let log = fs::read_to_string(probe_config(tmp.path()).active_path).expect("probe log");
        assert!(
            log.contains(PROBE_TAIL_MARKER),
            "installing bounded stderr in a process launched with closed standard \
             descriptors must still log: {log:?}"
        );
    }

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

    #[test]
    fn periodic_summary_flushes_at_deadline_without_waiting_for_eof() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("periodic-summary.log");
        let file = OpenOptions::new()
            .create_new(true)
            .append(true)
            .open(&path)
            .expect("active fixture");
        let mut writer = BoundedLogWriter::new_with_summary_interval(
            file,
            512,
            WarningAggregator::new(2),
            Duration::from_secs(60),
        );
        writer
            .write_warning_at(
                "renderer_context_lost",
                "renderer context lost",
                Duration::ZERO,
            )
            .expect("first warning");
        writer
            .write_warning_at(
                "renderer_context_lost",
                "renderer context lost",
                Duration::from_secs(1),
            )
            .expect("repeated warning");

        assert!(!writer
            .flush_due(Duration::from_secs(59))
            .expect("early flush"));
        assert!(writer
            .flush_due(Duration::from_secs(60))
            .expect("deadline flush"));

        let content = fs::read_to_string(&path).expect("periodic contents");
        assert!(content.contains("category=renderer_context_lost total=2 repeated=1"));
    }

    #[test]
    fn recovery_writes_accounting_and_releases_category_capacity() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("recovery.log");
        let file = OpenOptions::new()
            .create_new(true)
            .append(true)
            .open(&path)
            .expect("active fixture");
        let mut writer = BoundedLogWriter::new_with_summary_interval(
            file,
            512,
            WarningAggregator::new(1),
            Duration::from_secs(60),
        );
        writer
            .write_warning_at("renderer_context_lost", "lost", Duration::ZERO)
            .expect("first warning");
        writer
            .write_warning_at(
                "renderer_context_lost",
                "lost again",
                Duration::from_secs(1),
            )
            .expect("repeat warning");

        assert_eq!(
            writer
                .recover_warning_at(
                    "renderer_context_lost",
                    "renderer context restored",
                    Duration::from_secs(2),
                )
                .expect("recovery"),
            WarningEvent::Recovered {
                total_count: 2,
                repeated_count: 1,
            }
        );
        assert!(matches!(
            writer.write_warning_at(
                "wsl_vhd_wait_timeout",
                "new category after recovery",
                Duration::from_secs(3),
            ),
            Ok(WarningEvent::First { .. })
        ));

        let content = fs::read_to_string(&path).expect("recovery contents");
        assert!(content.contains(
            "limux-warning-recovery category=renderer_context_lost total=2 repeated=1 renderer context restored"
        ));
    }

    #[test]
    fn stderr_line_classification_pairs_warning_and_recovery_categories() {
        assert_eq!(
            classify_log_action("Gtk-WARNING renderer context lost"),
            LogAction::Warning {
                category: "renderer_context_lost".to_string(),
            }
        );
        assert_eq!(
            classify_log_action("renderer context restored"),
            LogAction::Recovery {
                category: "renderer_context_lost".to_string(),
            }
        );
        assert_eq!(
            classify_log_action("ordinary diagnostic line"),
            LogAction::Raw
        );
    }

    #[test]
    fn preview_smoke_repeated_warning_growth_remains_bounded() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("repeated-warning-smoke.log");
        let file = OpenOptions::new()
            .create_new(true)
            .append(true)
            .open(&path)
            .expect("active fixture");
        let mut writer = BoundedLogWriter::new_with_summary_interval(
            file,
            512,
            WarningAggregator::new(4),
            Duration::from_secs(60),
        );

        for second in 0..10_000 {
            writer
                .write_warning_at(
                    "renderer_context_lost",
                    "Gtk-WARNING renderer context lost",
                    Duration::from_secs(second),
                )
                .expect("warning write");
        }
        writer.finish().expect("finish writer");

        let content = fs::read_to_string(&path).expect("bounded content");
        assert_eq!(
            content.matches("Gtk-WARNING renderer context lost").count(),
            1
        );
        assert!(fs::metadata(&path).expect("bounded metadata").len() <= 512);
    }

    #[test]
    fn preview_smoke_multi_start_retention_never_clobbers_and_fails_closed() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let mut config = config(tmp.path());
        config.max_total_bytes = 192;

        for (sequence, payload) in [(50, b"first".as_slice()), (50, b"second"), (50, b"third")] {
            let HostLogSetup::Active { mut file, .. } = prepare_host_logging(&config, sequence)
            else {
                panic!("start {sequence} should receive a bounded active log");
            };
            file.write_all(payload).expect("active payload");
            file.flush().expect("active flush");
        }

        assert_eq!(
            fs::read(config.retained_dir.join("limux-host.50.log")).expect("first retained"),
            b"first"
        );
        assert_eq!(
            fs::read(config.retained_dir.join("limux-host.50.1.log")).expect("collision retained"),
            b"second"
        );
        assert!(matches!(
            prepare_host_logging(&config, 52),
            HostLogSetup::StderrFallback { .. }
        ));
        assert_eq!(
            fs::read(&config.active_path).expect("active preserved on fallback"),
            b"third"
        );
    }

    #[test]
    fn preview_smoke_setup_failure_keeps_startup_on_stderr_fallback() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let config = config(tmp.path());
        fs::write(tmp.path().join("managed"), b"not a directory").expect("blocked parent");

        let setup = prepare_host_logging(&config, 60);

        assert!(matches!(setup, HostLogSetup::StderrFallback { .. }));
    }
}
