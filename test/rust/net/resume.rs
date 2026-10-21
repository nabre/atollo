use super::*;

use std::io::{BufRead, BufReader};
use std::net::TcpListener;
use std::sync::Arc;

#[test]
fn a_range_from_the_part_is_appended() {
    assert_eq!(
        resume_of(206, Some("bytes 100-199/200"), 100),
        Resume::Append
    );
    // From another byte, or a server without Range: from the start.
    assert_eq!(
        resume_of(206, Some("bytes 0-199/200"), 100),
        Resume::Restart
    );
    assert_eq!(resume_of(200, None, 100), Resume::Restart);
    assert_eq!(resume_of(206, None, 100), Resume::Restart);
    // Nothing left: the part is the whole file.
    assert_eq!(resume_of(416, Some("bytes */200"), 200), Resume::Complete);
    assert_eq!(resume_of(416, Some("bytes */300"), 200), Resume::Restart);
    assert_eq!(resume_of(416, None, 200), Resume::Restart);
}

#[test]
fn the_part_is_next_to_the_file() {
    let dest = Path::new("models").join("m.gguf");
    assert_eq!(part_of(&dest), Path::new("models").join("m.gguf.part"));
}

/// How the test server answers.
#[derive(Clone, Copy)]
enum Serve {
    /// Honours `Range`; the first answer is cut after `cut` bytes.
    Ranges { cut: Option<usize> },
    /// Ignores `Range`: always the whole file.
    Whole,
}

/// A server on 127.0.0.1 for `data`, answering `requests` requests; the
/// `Range` header of each is sent on the channel.
fn serve(
    data: Arc<Vec<u8>>,
    how: Serve,
    requests: usize,
) -> (String, std::sync::mpsc::Receiver<Option<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/m.gguf", listener.local_addr().unwrap());
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        for (i, stream) in listener.incoming().take(requests).enumerate() {
            let mut stream = stream.unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut range = None;
            loop {
                let mut line = String::new();
                if reader.read_line(&mut line).unwrap() == 0 || line == "\r\n" {
                    break;
                }
                if let Some(v) = line
                    .to_ascii_lowercase()
                    .strip_prefix("range: bytes=")
                    .map(|v| v.trim().trim_end_matches('-').to_owned())
                {
                    range = Some(v);
                }
            }
            let _ = tx.send(range.clone());
            let start: usize = match how {
                Serve::Ranges { .. } => range.as_deref().map_or(0, |r| r.parse().unwrap()),
                Serve::Whole => 0,
            };
            let body = &data[start..];
            let head = if start > 0 {
                format!(
                    "HTTP/1.1 206 Partial Content\r\nContent-Length: {}\r\nContent-Range: bytes {start}-{}/{}\r\nConnection: close\r\n\r\n",
                    body.len(),
                    data.len() - 1,
                    data.len()
                )
            } else {
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                )
            };
            stream.write_all(head.as_bytes()).unwrap();
            let send = match how {
                Serve::Ranges { cut: Some(cut) } if i == 0 => &body[..cut],
                _ => body,
            };
            let _ = stream.write_all(send);
            // An orderly close: on Windows a socket closed at once can
            // reset the connection before the client read the body.
            let _ = stream.shutdown(std::net::Shutdown::Write);
            let _ = std::io::copy(&mut reader, &mut std::io::sink());
        }
    });
    (url, rx)
}

fn temp(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("atollo-resume-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn net() -> Net {
    Net::new(&crate::settings::Settings::default(), None).unwrap()
}

fn sha256(data: &[u8]) -> String {
    hex(&Sha256::digest(data))
}

fn sample() -> Arc<Vec<u8>> {
    Arc::new((0..300_000u32).map(|i| (i * 7) as u8).collect())
}

/// A transfer cut halfway goes on from the byte where it stopped, and the
/// file is the whole one with its checksum.
#[test]
fn a_cut_download_goes_on_from_where_it_stopped() {
    let data = sample();
    let (url, ranges) = serve(data.clone(), Serve::Ranges { cut: Some(120_000) }, 2);
    let dir = temp("cut");
    let dest = dir.join("m.gguf");
    net()
        .download_resumable(
            &url,
            &dest,
            &sha256(&data),
            Some(data.len() as u64),
            &[],
            &AtomicBool::new(false),
            &mut |_, _| {},
        )
        .unwrap();
    assert_eq!(fs::read(&dest).unwrap(), *data);
    assert!(!part_of(&dest).exists());
    assert_eq!(ranges.recv().unwrap(), None);
    assert_eq!(ranges.recv().unwrap().as_deref(), Some("120000"));
}

/// A part left by an earlier run is resumed, not downloaded again.
#[test]
fn a_part_of_an_earlier_run_is_resumed() {
    let data = sample();
    let (url, ranges) = serve(data.clone(), Serve::Ranges { cut: None }, 1);
    let dir = temp("earlier");
    let dest = dir.join("m.gguf");
    fs::write(part_of(&dest), &data[..50_000]).unwrap();
    assert_eq!(downloaded(&dest), 50_000);
    net()
        .download_resumable(
            &url,
            &dest,
            &sha256(&data),
            None,
            &[],
            &AtomicBool::new(false),
            &mut |_, _| {},
        )
        .unwrap();
    assert_eq!(fs::read(&dest).unwrap(), *data);
    assert_eq!(ranges.recv().unwrap().as_deref(), Some("50000"));
}

/// A server that ignores `Range` sends the whole file: the part starts again.
#[test]
fn a_server_without_ranges_starts_again() {
    let data = sample();
    let (url, _) = serve(data.clone(), Serve::Whole, 1);
    let dir = temp("whole");
    let dest = dir.join("m.gguf");
    fs::write(part_of(&dest), &data[..50_000]).unwrap();
    net()
        .download_resumable(
            &url,
            &dest,
            &sha256(&data),
            None,
            &[],
            &AtomicBool::new(false),
            &mut |_, _| {},
        )
        .unwrap();
    assert_eq!(fs::read(&dest).unwrap(), *data);
}

/// A wrong checksum is refused and nothing is kept.
#[test]
fn a_wrong_checksum_is_refused() {
    let data = sample();
    let (url, _) = serve(data.clone(), Serve::Ranges { cut: None }, 1);
    let dir = temp("wrong");
    let dest = dir.join("m.gguf");
    let err = net()
        .download_resumable(
            &url,
            &dest,
            &sha256(b"other"),
            None,
            &[],
            &AtomicBool::new(false),
            &mut |_, _| {},
        )
        .unwrap_err();
    assert!(err.to_string().contains("checksum"), "{err:#}");
    assert!(!dest.exists());
    assert!(!part_of(&dest).exists());
}

/// Cancel keeps the part for the next attempt.
#[test]
fn cancel_keeps_the_part() {
    let dir = temp("cancel");
    let dest = dir.join("m.gguf");
    fs::write(part_of(&dest), b"abc").unwrap();
    let err = net()
        .download_resumable(
            "http://127.0.0.1:9/never",
            &dest,
            &sha256(b"abcdef"),
            None,
            &[],
            &AtomicBool::new(true),
            &mut |_, _| {},
        )
        .unwrap_err();
    assert!(err.is::<Cancelled>(), "{err:#}");
    assert_eq!(fs::read(part_of(&dest)).unwrap(), b"abc");
}
