use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct TailRead {
    pub(crate) lines: Vec<String>,
    pub(crate) bytes_read: u64,
    pub(crate) truncated: bool,
}

pub(crate) fn read_backward_tail(
    path: &Path,
    line_limit: usize,
    byte_limit: usize,
) -> io::Result<TailRead> {
    if line_limit == 0 || byte_limit == 0 {
        return Ok(TailRead {
            lines: Vec::new(),
            bytes_read: 0,
            truncated: false,
        });
    }
    let mut file = File::open(path)?;
    let file_len = file.metadata()?.len();
    let byte_limit = u64::try_from(byte_limit).unwrap_or(u64::MAX);
    let start = file_len.saturating_sub(byte_limit);
    file.seek(SeekFrom::Start(start))?;
    let bytes_to_read = usize::try_from(file_len - start).unwrap_or(usize::MAX);
    let mut bytes = Vec::with_capacity(bytes_to_read.min(byte_limit as usize));
    file.take(byte_limit).read_to_end(&mut bytes)?;
    let bytes_read = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    let text = String::from_utf8_lossy(&bytes);
    let mut candidate_lines = text.lines();
    if start > 0 {
        let _ = candidate_lines.next();
    }
    let lines = candidate_lines
        .rev()
        .take(line_limit)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(str::to_string)
        .collect();
    Ok(TailRead {
        lines,
        bytes_read,
        truncated: start > 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, OpenOptions};
    use std::io::Write;

    #[test]
    fn backward_tail_reads_only_the_bounded_suffix() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("tiny-host-log.fixture");
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .expect("fixture file");
        for index in 0..2_000 {
            writeln!(file, "line-{index:04}").expect("fixture line");
        }
        file.flush().expect("fixture flush");
        let file_len = fs::metadata(&path).expect("fixture metadata").len();

        let tail = read_backward_tail(&path, 3, 256).expect("bounded tail");

        assert_eq!(tail.lines, ["line-1997", "line-1998", "line-1999"]);
        assert!(tail.bytes_read <= 256, "read {} bytes", tail.bytes_read);
        assert!(tail.bytes_read < file_len);
        assert!(tail.truncated);
    }

    #[test]
    fn backward_tail_handles_a_partial_first_line_without_growing_the_budget() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("partial-line.fixture");
        fs::write(
            &path,
            b"prefix-that-must-be-discarded\nkeep-one\nkeep-two\n",
        )
        .expect("fixture");

        let tail = read_backward_tail(&path, 2, 20).expect("bounded tail");

        assert_eq!(tail.lines, ["keep-one", "keep-two"]);
        assert!(tail.bytes_read <= 20, "read {} bytes", tail.bytes_read);
    }
}
