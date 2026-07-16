use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::os::fd::AsRawFd;
use std::path::Path;

struct FileLock(File);

impl FileLock {
    fn acquire(path: &Path) -> io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)?;
        // SAFETY: flock only reads the live file descriptor and does not retain it.
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX) } != 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(Self(file))
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        // SAFETY: the descriptor remains owned by self until this drop completes.
        unsafe {
            libc::flock(self.0.as_raw_fd(), libc::LOCK_UN);
        }
    }
}

pub fn write_bytes_atomic_durable(path: &Path, bytes: &[u8]) -> io::Result<()> {
    with_target_lock(path, || commit_bytes_locked(path, bytes))
}

pub fn update_bytes_atomic_durable<F>(path: &Path, update: F) -> io::Result<bool>
where
    F: FnOnce(&[u8]) -> io::Result<Option<Vec<u8>>>,
{
    with_target_lock(path, || {
        let current = fs::read(path)?;
        let Some(updated) = update(&current)? else {
            return Ok(false);
        };
        commit_bytes_locked(path, &updated)?;
        Ok(true)
    })
}

fn with_target_lock<T>(path: &Path, operation: impl FnOnce() -> io::Result<T>) -> io::Result<T> {
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "target has no parent"))?;
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid target file name"))?;
    fs::create_dir_all(parent)?;

    let lock_path = parent.join(format!(".{file_name}.lock"));
    let _lock = FileLock::acquire(&lock_path)?;
    operation()
}

fn commit_bytes_locked(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "target has no parent"))?;
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid target file name"))?;
    let pending_path = parent.join(format!(".{file_name}.pending"));

    let mut pending = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&pending_path)?;
    pending.write_all(bytes)?;
    pending.sync_all()?;
    drop(pending);

    fs::rename(&pending_path, path)?;
    OpenOptions::new().read(true).open(parent)?.sync_all()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::write_bytes_atomic_durable;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn repeated_commit_failures_leave_one_bounded_pending_file() {
        let dir = tempdir().expect("tempdir");
        let target = dir.path().join("state.json");
        fs::create_dir(&target).expect("blocking target directory");

        assert!(write_bytes_atomic_durable(&target, b"first").is_err());
        assert!(write_bytes_atomic_durable(&target, b"second").is_err());

        let pending = dir.path().join(".state.json.pending");
        assert_eq!(fs::read(&pending).expect("bounded pending file"), b"second");
        let pending_count = fs::read_dir(dir.path())
            .expect("read tempdir")
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().ends_with(".pending"))
            .count();
        assert_eq!(pending_count, 1);
    }
}
