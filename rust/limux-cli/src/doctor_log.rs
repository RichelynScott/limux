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
