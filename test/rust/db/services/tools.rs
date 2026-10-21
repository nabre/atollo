use super::*;

/// Far more than a pipe holds (64 KiB on Windows and Linux): the write
/// would block, or break, before the tool ends.
fn big_input() -> String {
    "SELECT 1;\n".repeat(800_000)
}

/// A tool that fails without reading its stdin, with `boom` on stderr.
fn failing_tool() -> Command {
    if cfg!(windows) {
        let mut cmd = Command::new("cmd");
        cmd.args(["/d", "/c", "echo boom 1>&2 & exit /b 3"]);
        cmd
    } else {
        let mut cmd = Command::new("sh");
        cmd.args(["-c", "echo boom >&2; exit 3"]);
        cmd
    }
}

/// A tool that never reads its stdin and takes 30 seconds.
fn stuck_tool() -> Command {
    if cfg!(windows) {
        let mut cmd = Command::new("ping");
        cmd.args(["-n", "30", "127.0.0.1"]);
        cmd
    } else {
        let mut cmd = Command::new("sleep");
        cmd.arg("30");
        cmd
    }
}

/// A tool that reads its whole stdin and ends well.
fn reading_tool() -> Command {
    if cfg!(windows) {
        Command::new("sort")
    } else {
        Command::new("cat")
    }
}

/// The tool's own error, not the "broken pipe" of the input it never read
/// (BUG-V6-05).
#[test]
fn a_failing_tool_tells_its_own_error() {
    let e = run_with_input(failing_tool(), &big_input(), Duration::from_secs(60))
        .unwrap_err()
        .to_string();
    assert!(e.contains("boom"), "{e}");
    assert!(!e.to_lowercase().contains("pipe"), "{e}");
}

/// A tool that stops reading still meets the limit (BUG-V6-05).
#[test]
fn a_tool_that_stops_reading_meets_the_limit() {
    let start = Instant::now();
    let e = run_with_input(stuck_tool(), &big_input(), Duration::from_secs(1))
        .unwrap_err()
        .to_string();
    assert!(e.contains("no answer"), "{e}");
    assert!(
        start.elapsed() < Duration::from_secs(20),
        "{:?}",
        start.elapsed()
    );
}

/// An input that cannot be read to its end (a copy on a stick pulled out)
/// is never a whole run, even when the tool ends well on what it got.
#[test]
fn an_input_read_halfway_is_an_error() {
    struct Halfway(usize);
    impl std::io::Read for Halfway {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            if self.0 == 0 {
                return Err(std::io::Error::other("the copy is gone"));
            }
            let n = buf.len().min(self.0);
            // Short lines: Windows' `sort` refuses long ones.
            buf[..n].fill(b'\n');
            self.0 -= n;
            Ok(n)
        }
    }
    let e = run_with_reader(reading_tool(), Halfway(200_000), Duration::from_secs(60))
        .unwrap_err()
        .to_string();
    assert!(e.contains("the copy is gone"), "{e}");
    run_with_input(reading_tool(), &big_input(), Duration::from_secs(60)).unwrap();
}
