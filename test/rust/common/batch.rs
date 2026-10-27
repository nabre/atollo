use super::*;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};

const MS: Duration = Duration::from_millis(1);
const HOUR: Duration = Duration::from_secs(3600);

fn pace(every: Duration, cap: usize) -> Pace {
    Pace { every, cap }
}

fn taken(b: &mut Batch, now: Instant) -> Vec<String> {
    let mut out = Vec::new();
    b.flush(now, &mut |bytes| {
        out.push(String::from_utf8(bytes.to_vec()).unwrap())
    });
    out
}

/// The first output after a quiet moment goes at once (a key's echo);
/// what follows it within `every` waits and goes as one batch.
#[test]
fn quiet_output_goes_at_once_streaming_output_waits() {
    let t0 = Instant::now();
    let mut b = Batch::new(pace(16 * MS, 1024));
    assert_eq!(b.due(), None);
    assert!(b.add(b"a", t0), "nothing went yet");
    assert_eq!(taken(&mut b, t0), ["a"]);
    assert_eq!(b.due(), None);
    assert!(!b.add(b"b", t0 + 5 * MS));
    assert!(!b.add(b"c", t0 + 10 * MS));
    assert_eq!(b.due(), Some(t0 + 16 * MS));
    assert_eq!(taken(&mut b, t0 + 16 * MS), ["bc"]);
    // Quiet again for longer than `every`.
    assert!(b.add(b"d", t0 + 40 * MS));
    assert_eq!(taken(&mut b, t0 + 40 * MS), ["d"]);
    // Nothing waiting: nothing is handed over, and the clock stays.
    assert!(taken(&mut b, t0 + 41 * MS).is_empty());
    assert!(!b.add(b"e", t0 + 50 * MS));
    assert_eq!(b.due(), Some(t0 + 56 * MS));
}

/// A full batch goes at once, however soon after the last.
#[test]
fn a_full_batch_goes_at_once() {
    let t0 = Instant::now();
    let mut b = Batch::new(pace(HOUR, 4));
    assert!(b.add(b"a", t0));
    assert_eq!(taken(&mut b, t0), ["a"]);
    assert!(!b.add(b"bc", t0 + MS));
    assert!(b.add(b"de", t0 + 2 * MS));
    assert_eq!(taken(&mut b, t0 + 2 * MS), ["bcde"]);
}

/// A reader fed by the test, piece by piece; it ends when the sender is
/// dropped.
struct Feed(std::sync::mpsc::Receiver<Vec<u8>>);

impl Read for Feed {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self.0.recv() {
            Ok(piece) => {
                buf[..piece.len()].copy_from_slice(&piece);
                Ok(piece.len())
            }
            Err(_) => Ok(0),
        }
    }
}

/// [`pump`] on a thread of the test: the feed, what was flushed, a
/// signal at every flush, and the pump's thread.
#[allow(clippy::type_complexity)]
fn pumping(
    pace: Pace,
) -> (
    Sender<Vec<u8>>,
    Arc<Mutex<Vec<String>>>,
    std::sync::mpsc::Receiver<()>,
    std::thread::JoinHandle<()>,
) {
    let (feed, rx) = channel();
    let (flushed, flushes) = channel();
    let sent = Arc::new(Mutex::new(Vec::new()));
    let out = Arc::clone(&sent);
    let handle = std::thread::spawn(move || {
        pump(Feed(rx), 1024, pace, |bytes| {
            out.lock()
                .unwrap()
                .push(String::from_utf8(bytes.to_vec()).unwrap());
            let _ = flushed.send(());
        });
    });
    (feed, sent, flushes, handle)
}

/// What streams in within `every` goes as one batch, once the time is
/// up, with nothing more read.
#[test]
fn pump_flushes_a_waiting_batch_when_its_time_is_up() {
    let (feed, sent, flushes, handle) = pumping(pace(150 * MS, 1 << 20));
    feed.send(b"a".to_vec()).unwrap();
    flushes.recv_timeout(Duration::from_secs(5)).unwrap();
    assert_eq!(*sent.lock().unwrap(), ["a"]);
    feed.send(b"b".to_vec()).unwrap();
    feed.send(b"c".to_vec()).unwrap();
    flushes.recv_timeout(Duration::from_secs(5)).unwrap();
    assert_eq!(*sent.lock().unwrap(), ["a", "bc"]);
    drop(feed);
    handle.join().unwrap();
    assert_eq!(*sent.lock().unwrap(), ["a", "bc"]);
}

/// The end of the stream sends what waits before `pump` returns: the
/// caller's "ended" never comes before the last output.
#[test]
fn pump_sends_what_waits_before_it_returns() {
    let (feed, sent, flushes, handle) = pumping(pace(HOUR, 1 << 20));
    feed.send(b"a".to_vec()).unwrap();
    flushes.recv_timeout(Duration::from_secs(5)).unwrap();
    feed.send(b"b".to_vec()).unwrap();
    feed.send(b"c".to_vec()).unwrap();
    drop(feed);
    handle.join().unwrap();
    assert_eq!(*sent.lock().unwrap(), ["a", "bc"]);
}

/// A full batch goes without waiting for its time.
#[test]
fn pump_sends_a_full_batch_at_once() {
    let (feed, sent, flushes, handle) = pumping(pace(HOUR, 4));
    feed.send(b"a".to_vec()).unwrap();
    flushes.recv_timeout(Duration::from_secs(5)).unwrap();
    feed.send(b"bc".to_vec()).unwrap();
    feed.send(b"de".to_vec()).unwrap();
    flushes.recv_timeout(Duration::from_secs(5)).unwrap();
    assert_eq!(*sent.lock().unwrap(), ["a", "bcde"]);
    feed.send(b"f".to_vec()).unwrap();
    drop(feed);
    handle.join().unwrap();
    assert_eq!(*sent.lock().unwrap(), ["a", "bcde", "f"]);
}

/// Many pieces in a stream: fewer batches, every byte once, in order,
/// none larger than the cap and one read.
#[test]
fn pump_keeps_every_byte_in_order() {
    let (feed, sent, _flushes, handle) = pumping(pace(5 * MS, 256));
    let mut all = String::new();
    for i in 0..2000 {
        let piece = format!("{i};");
        all.push_str(&piece);
        feed.send(piece.into_bytes()).unwrap();
    }
    drop(feed);
    handle.join().unwrap();
    let sent = sent.lock().unwrap();
    assert_eq!(sent.concat(), all);
    assert!(sent.len() < 2000, "{} batches", sent.len());
    assert!(sent.iter().all(|b| b.len() < 256 + 8), "{sent:?}");
}
