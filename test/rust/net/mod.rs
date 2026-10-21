use super::*;

#[test]
fn retry_stops_on_cancel() {
    let mut calls = 0;
    let result: Result<()> = retry(|| {
        calls += 1;
        Err(Cancelled.into())
    });
    assert!(result.is_err());
    assert_eq!(calls, 1);
}

/// A body that gives its first bytes, then nothing: each read waits until
/// the test drops the sender (a proxy that stops sending).
struct Stalls {
    first: Option<Vec<u8>>,
    wait: std::sync::mpsc::Receiver<()>,
}

impl Read for Stalls {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if let Some(first) = self.first.take() {
            buf[..first.len()].copy_from_slice(&first);
            return Ok(first.len());
        }
        let _ = self.wait.recv();
        Err(std::io::Error::other("closed"))
    }
}

fn stalls() -> (Stalls, std::sync::mpsc::Sender<()>) {
    let (tx, rx) = std::sync::mpsc::channel();
    let body = Stalls {
        first: Some(b"abc".to_vec()),
        wait: rx,
    };
    (body, tx)
}

/// A transfer that stalls ends after the inactivity limit, with the bytes
/// that arrived written (BUG-V6-13).
#[test]
fn a_stalled_download_ends() {
    let (body, _hold) = stalls();
    let mut file = Vec::new();
    let start = std::time::Instant::now();
    let err = copy_body(
        body,
        &mut file,
        &AtomicBool::new(false),
        Duration::from_millis(300),
        &mut |_| {},
    )
    .unwrap_err();
    assert!(err.to_string().contains("no data"), "{err:#}");
    assert!(start.elapsed() < Duration::from_secs(5));
    assert_eq!(file, b"abc");
}

/// Cancel stops a download whose read never returns (BUG-V6-13).
#[test]
fn cancel_stops_a_stalled_download() {
    let (body, _hold) = stalls();
    let cancel = Arc::new(AtomicBool::new(false));
    let flag = cancel.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(200));
        flag.store(true, Ordering::Relaxed);
    });
    let start = std::time::Instant::now();
    let err = copy_body(
        body,
        &mut Vec::new(),
        &cancel,
        Duration::from_secs(600),
        &mut |_| {},
    )
    .unwrap_err();
    assert!(err.is::<Cancelled>(), "{err:#}");
    assert!(start.elapsed() < Duration::from_secs(5));
}

#[test]
fn copy_body_copies_to_the_end() {
    let data: Vec<u8> = (0..700_000u32).map(|i| i as u8).collect();
    let mut file = Vec::new();
    let mut seen = 0;
    let done = copy_body(
        std::io::Cursor::new(data.clone()),
        &mut file,
        &AtomicBool::new(false),
        Duration::from_secs(5),
        &mut |d| seen = d,
    )
    .unwrap();
    assert_eq!(done, data.len() as u64);
    assert_eq!(seen, done);
    assert_eq!(file, data);
}
