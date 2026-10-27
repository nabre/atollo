//! The output of a program on its way to the window, in batches: a build
//! or an `npm install` writes thousands of small pieces a second, and one
//! event each floods the webview (every event is serialized, posted to the
//! page and handled there). The terminal (`editor/pty.rs`) and the log of
//! a run (`projects/run/output.rs`) read through here.
//!
//! - what comes after a quiet moment goes at once: the echo of a key typed
//!   is never held back;
//! - what comes while output streams waits for the next flush, at most
//!   [`Pace::every`] after the last one, or goes as soon as [`Pace::cap`]
//!   bytes are waiting;
//! - the end of the stream sends what is left before [`pump`] returns, so
//!   nothing the caller tells after it (the program ended) overtakes the
//!   output. Bytes go out in the order read; a batch may end inside a
//!   character, as a read could: the caller decodes them as one stream.

use std::io::Read;
use std::sync::mpsc::{sync_channel, Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

/// How a stream is batched.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pace {
    /// The least time between two flushes while output streams.
    pub every: Duration,
    /// The bytes that are flushed at once, however soon after the last.
    pub cap: usize,
}

/// Reads not handed over yet: the reader stops reading (and the program,
/// once its pipe is full, stops writing) while the window is behind.
const DEPTH: usize = 16;

/// The bytes waiting for the next flush and when the last one went.
#[derive(Debug)]
pub struct Batch {
    pace: Pace,
    bytes: Vec<u8>,
    last: Option<Instant>,
}

impl Batch {
    pub fn new(pace: Pace) -> Batch {
        Batch {
            pace,
            bytes: Vec::new(),
            last: None,
        }
    }

    /// Adds `bytes` read at `now`: whether the batch is to go now (the
    /// first output after a quiet moment, or a full batch).
    pub fn add(&mut self, bytes: &[u8], now: Instant) -> bool {
        self.bytes.extend_from_slice(bytes);
        self.bytes.len() >= self.pace.cap
            || self
                .last
                .is_none_or(|last| now.saturating_duration_since(last) >= self.pace.every)
    }

    /// When the bytes waiting are to go; `None` with nothing waiting.
    pub fn due(&self) -> Option<Instant> {
        if self.bytes.is_empty() {
            return None;
        }
        Some(self.last.map_or_else(Instant::now, |last| last + self.pace.every))
    }

    /// Hands the bytes waiting to `flush`, if any, as gone at `now`.
    pub fn flush(&mut self, now: Instant, flush: &mut impl FnMut(&[u8])) {
        if self.bytes.is_empty() {
            return;
        }
        flush(&self.bytes);
        self.bytes.clear();
        self.last = Some(now);
    }
}

/// Reads `reader` to its end (`read_size` bytes at most at a time) and
/// hands what it read to `flush` in batches paced by `pace`; returns once
/// the last batch was handed over. The reading blocks, so it runs on a
/// thread of its own: the caller's thread waits for the bytes and for the
/// time a batch is due, and is the only one to call `flush`, in order.
pub fn pump(
    reader: impl Read + Send + 'static,
    read_size: usize,
    pace: Pace,
    mut flush: impl FnMut(&[u8]),
) {
    let (tx, rx) = sync_channel::<Vec<u8>>(DEPTH);
    std::thread::spawn(move || {
        let mut reader = reader;
        let mut buf = vec![0u8; read_size.max(1)];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    if tx.send(buf[..n].to_vec()).is_err() {
                        break;
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
                // EIO on Linux once the program and its children closed
                // the PTY; a broken pipe.
                Err(_) => break,
            }
        }
    });
    drain(&rx, pace, &mut flush);
}

/// The caller's side of [`pump`]: the bytes of `rx` until the reader
/// ended, batched.
fn drain(rx: &Receiver<Vec<u8>>, pace: Pace, flush: &mut impl FnMut(&[u8])) {
    let mut batch = Batch::new(pace);
    loop {
        let read = match batch.due() {
            None => rx.recv().map_err(|_| RecvTimeoutError::Disconnected),
            Some(at) => rx.recv_timeout(at.saturating_duration_since(Instant::now())),
        };
        match read {
            Ok(bytes) => {
                let now = Instant::now();
                if batch.add(&bytes, now) {
                    batch.flush(now, flush);
                }
            }
            Err(RecvTimeoutError::Timeout) => batch.flush(Instant::now(), flush),
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
    batch.flush(Instant::now(), flush);
}

#[cfg(test)]
#[path = "../../../test/rust/common/batch.rs"]
mod tests;
