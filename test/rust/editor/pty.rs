use super::*;
use crate::kit::tests::TempDir;
use crate::kit::{Arch, Target};

#[test]
fn utf8_split_between_reads_is_kept() {
    let mut d = Utf8::default();
    let text = "già ✓ 😀";
    let bytes = text.as_bytes();
    let mut out = String::new();
    // One byte at a time: every character is split somewhere.
    for b in bytes {
        out.push_str(&d.push(std::slice::from_ref(b)));
    }
    assert_eq!(out, text);
    assert_eq!(d.finish(), "");
}

#[test]
fn utf8_invalid_bytes_become_replacement() {
    let mut d = Utf8::default();
    assert_eq!(d.push(b"a\xffb"), "a\u{fffd}b");
    // An incomplete character at the end of the stream.
    assert_eq!(d.push(b"c\xe2\x9c"), "c");
    assert_eq!(d.finish(), "\u{fffd}");
}

#[test]
fn backlog_keeps_the_tail_at_a_line_start() {
    let mut b = Backlog::with_cap(10);
    assert_eq!(b.push("abc\n"), 1);
    assert_eq!(b.push("defgh\nij"), 2);
    let s = b.snapshot();
    assert_eq!(s.seq, 2);
    // 12 bytes, cap 10: cut after the first new line.
    assert_eq!(s.data, "defgh\nij");
    b.push("klmno");
    assert_eq!(b.snapshot().data, "ijklmno");
    assert_eq!(b.snapshot().seq, 3);
    // Never inside a character.
    let mut b = Backlog::with_cap(3);
    b.push("ààà");
    assert!(b.snapshot().data.len() <= 4);
    assert!(b.snapshot().data.ends_with('à'));
}

fn kit_on(name: &str, target: Target, installed: &[&str]) -> (TempDir, Kit) {
    let tmp = TempDir::new(name);
    let kit = Kit::at(tmp.0.join("k (1) & co"), Arch::X64).for_target(target);
    kit.prepare().unwrap();
    for rel in installed {
        let f = crate::fsx::join_rel(kit.apps(), rel);
        std::fs::create_dir_all(f.parent().unwrap()).unwrap();
        std::fs::write(f, b"").unwrap();
    }
    (tmp, kit)
}

#[test]
fn shell_is_bash_with_the_kit_bashrc_without_pwsh() {
    for target in [Target::LINUX_X64, Target::DARWIN_ARM64] {
        let (_t, kit) = kit_on("pty-bash", target, &[]);
        let dir = kit.projects().join("app");
        let l = shell(&kit, &dir);
        assert_eq!(l.program, Path::new("/bin/bash"));
        assert_eq!(l.shell, Path::new("/bin/bash"));
        assert_eq!(l.args[0], "--rcfile");
        let rc = l.args[1].replace('\\', "/");
        assert!(rc.ends_with("data/kit/bashrc"), "{rc}");
        assert_eq!(l.args[2], "-i");
        assert_eq!(l.title, "app · bash");
    }
}

#[test]
fn shell_is_the_kit_pwsh_when_installed() {
    let (_t, kit) = kit_on("pty-pwsh", Target::LINUX_ARM64, &["pwsh/pwsh"]);
    let l = shell(&kit, &kit.projects());
    let program = l.program.display().to_string().replace('\\', "/");
    assert!(program.ends_with("linux-arm64/pwsh/pwsh"), "{program}");
    assert_eq!(l.shell, l.program);
    assert!(l.args.iter().any(|a| a.ends_with("profile.ps1")));
    assert!(l.title.ends_with(" · pwsh"));
}

#[test]
fn agents_run_directly_with_bash_as_shell() {
    let l = command(
        Path::new("/k/data/apps/linux-x64/claude/claude"),
        Path::new("/k/projects/my app"),
    );
    assert_eq!(l.program, Path::new("/k/data/apps/linux-x64/claude/claude"));
    assert!(l.args.is_empty());
    assert_eq!(l.shell, Path::new("/bin/bash"));
    assert_eq!(l.title, "my app · claude");
}

#[test]
fn output_events_are_named_by_session() {
    assert_eq!(output_event(7), "pty://7");
}

/// What a fake PTY was asked to do.
#[derive(Default)]
struct Calls {
    written: Vec<u8>,
    sizes: Vec<(u16, u16)>,
    hung_up: u32,
    killed: u32,
    released: u32,
}

#[derive(Clone)]
struct FakeIo {
    calls: Arc<Mutex<Calls>>,
    pid: u32,
    /// Ends the session when hung up, as a shell does.
    on_hang_up: Arc<Mutex<Option<Arc<Session>>>>,
}

impl FakeIo {
    fn new(pid: u32) -> FakeIo {
        FakeIo {
            calls: Arc::default(),
            pid,
            on_hang_up: Arc::default(),
        }
    }

    fn calls(&self) -> MutexGuard<'_, Calls> {
        self.calls.lock().unwrap()
    }
}

impl Io for FakeIo {
    fn write(&self, data: &[u8]) -> Result<()> {
        self.calls().written.extend_from_slice(data);
        Ok(())
    }
    fn resize(&self, cols: u16, rows: u16) -> Result<()> {
        self.calls().sizes.push((cols, rows));
        Ok(())
    }
    fn pid(&self) -> Option<u32> {
        Some(self.pid)
    }
    fn hang_up(&self) {
        self.calls().hung_up += 1;
        if let Some(s) = self.on_hang_up.lock().unwrap().take() {
            s.exited(Some(0));
        }
    }
    fn kill(&self) {
        self.calls().killed += 1;
    }
    fn release(&self) {
        self.calls().released += 1;
    }
}

#[test]
fn sessions_write_resize_and_count() {
    let sessions = Sessions::default();
    let io = FakeIo::new(4242);
    let s = sessions.insert(
        "app · bash".into(),
        Path::new("/p/app"),
        Box::new(io.clone()),
    );
    assert_eq!(s.info().id, 1);
    assert_eq!(sessions.running(), 1);
    assert_eq!(sessions.pids(), [4242]);
    sessions.write(1, "ls\r").unwrap();
    sessions.resize(1, 120, 30).unwrap();
    assert!(sessions.resize(1, 0, 30).is_err());
    assert!(sessions.write(9, "x").is_err(), "no such session");
    assert_eq!(io.calls().written, b"ls\r");
    assert_eq!(io.calls().sizes, [(120, 30)]);

    // Ended: still listed (its tab stays), not counted, not written to.
    s.exited(Some(3));
    assert_eq!(sessions.running(), 0);
    assert!(sessions.pids().is_empty());
    sessions.write(1, "more").unwrap();
    assert_eq!(io.calls().written, b"ls\r");
    let info = &sessions.list()[0];
    assert!(!info.running);
    assert_eq!(info.code, Some(3));
    assert_eq!(info.dir, Path::new("/p/app").display().to_string());
}

#[test]
fn pump_numbers_chunks_and_fills_the_backlog() {
    let sessions = Sessions::default();
    let s = sessions.insert("t".into(), Path::new("/p"), Box::new(FakeIo::new(1)));
    let sent = Mutex::new(Vec::new());
    // A reader returning a character split over two reads.
    let bytes = "ok ✓\n".as_bytes().to_vec();
    struct Split(Vec<Vec<u8>>);
    impl Read for Split {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            if self.0.is_empty() {
                return Ok(0);
            }
            let part = self.0.remove(0);
            buf[..part.len()].copy_from_slice(&part);
            Ok(part.len())
        }
    }
    let reader = Split(vec![bytes[..4].to_vec(), bytes[4..].to_vec()]);
    pump(&s, reader, |c| sent.lock().unwrap().push(c.clone()));
    let sent = sent.into_inner().unwrap();
    assert_eq!(
        sent,
        [
            Chunk {
                seq: 1,
                data: "ok ".into()
            },
            Chunk {
                seq: 2,
                data: "✓\n".into()
            }
        ]
    );
    assert_eq!(
        sessions.backlog(1).unwrap(),
        Chunk {
            seq: 2,
            data: "ok ✓\n".into()
        }
    );
}

#[test]
fn close_hangs_up_then_kills_what_did_not_end() {
    let sessions = Sessions::default();
    // A program that ignores the hang-up.
    let stubborn = FakeIo::new(10);
    sessions.insert("a".into(), Path::new("/p"), Box::new(stubborn.clone()));
    sessions.close(1, Duration::from_millis(50)).unwrap();
    assert_eq!(stubborn.calls().hung_up, 1);
    assert_eq!(stubborn.calls().killed, 1);
    assert_eq!(stubborn.calls().released, 1);
    assert!(sessions.list().is_empty());
    assert!(sessions.close(1, Duration::ZERO).is_err());

    // A shell ends on the hang-up: not killed.
    let polite = FakeIo::new(11);
    let s = sessions.insert("b".into(), Path::new("/p"), Box::new(polite.clone()));
    *polite.on_hang_up.lock().unwrap() = Some(Arc::clone(&s));
    sessions.close(s.info().id, Duration::from_secs(5)).unwrap();
    assert_eq!(polite.calls().hung_up, 1);
    assert_eq!(polite.calls().killed, 0);
    assert_eq!(polite.calls().released, 1);

    // Already ended: only let go.
    let ended = FakeIo::new(12);
    let s = sessions.insert("c".into(), Path::new("/p"), Box::new(ended.clone()));
    s.exited(None);
    sessions.close(s.info().id, Duration::from_secs(5)).unwrap();
    assert_eq!(ended.calls().hung_up, 0);
    assert_eq!(ended.calls().released, 1);
}

#[test]
fn close_all_at_exit_ends_every_session() {
    let sessions = Sessions::default();
    let ios: Vec<FakeIo> = (0..3).map(FakeIo::new).collect();
    for io in &ios {
        sessions.insert("x".into(), Path::new("/p"), Box::new(io.clone()));
    }
    sessions.close_all(Duration::from_millis(20));
    assert!(sessions.list().is_empty());
    for io in &ios {
        let c = io.calls();
        assert_eq!((c.hung_up, c.killed, c.released), (1, 1, 1));
    }
}

/// Real PTYs: a program's output comes back, the size reaches it, closing
/// ends its whole group.
#[cfg(unix)]
mod real {
    use super::*;
    use std::sync::mpsc::channel;

    fn env() -> crate::env::KitEnv {
        let mut env = crate::env::KitEnv::default();
        env.set("PATH", "/usr/bin:/bin");
        env.set("ATOLLO_PROBE", "kit-env");
        env
    }

    fn sh(script: &str) -> Launch {
        Launch {
            program: PathBuf::from("/bin/sh"),
            args: vec!["-c".into(), script.into()],
            shell: PathBuf::from("/bin/sh"),
            title: "test".into(),
        }
    }

    fn wait_for(sessions: &Sessions, id: u64, text: &str) -> String {
        let until = Instant::now() + Duration::from_secs(20);
        loop {
            let data = sessions.backlog(id).unwrap().data;
            if data.contains(text) || Instant::now() > until {
                return data;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    #[test]
    fn echo_comes_back_with_the_kit_env() {
        let tmp = TempDir::new("pty-echo");
        let sessions = Sessions::default();
        let (tx, rx) = channel();
        let info = sessions
            .open(
                &sh("printf 'pty-%s|%s|%s\\n' \"$ATOLLO_PROBE\" \"$TERM\" \"$(pwd)\"; exit 3"),
                &env(),
                &tmp.0,
                |_, _| {},
                move |info| tx.send(info).unwrap(),
            )
            .unwrap();
        assert!(info.running);
        let ended = rx.recv_timeout(Duration::from_secs(20)).unwrap();
        assert_eq!(ended.code, Some(3));
        let out = wait_for(&sessions, info.id, "pty-kit-env");
        let dir = tmp.0.canonicalize().unwrap();
        assert!(
            out.contains(&format!("pty-kit-env|xterm-256color|{}", dir.display())),
            "{out:?}"
        );
        sessions.close(info.id, Duration::ZERO).unwrap();
    }

    #[test]
    fn typed_input_and_resize_reach_the_program() {
        let tmp = TempDir::new("pty-size");
        let sessions = Sessions::default();
        let info = sessions
            .open(
                &sh("read line; stty size; echo \"got-$line\""),
                &env(),
                &tmp.0,
                |_, _| {},
                |_| {},
            )
            .unwrap();
        sessions.resize(info.id, 100, 40).unwrap();
        sessions.write(info.id, "hello\r").unwrap();
        let out = wait_for(&sessions, info.id, "got-hello");
        assert!(out.contains("40 100"), "{out:?}");
        assert!(out.contains("got-hello"), "{out:?}");
        sessions.close(info.id, Duration::ZERO).unwrap();
    }

    #[test]
    fn close_ends_the_whole_group() {
        let tmp = TempDir::new("pty-close");
        let sessions = Sessions::default();
        // A child that ignores the hang-up, in the program's group.
        let info = sessions
            .open(
                &sh("trap '' HUP; sleep 60 & echo \"bg=$!\"; wait"),
                &env(),
                &tmp.0,
                |_, _| {},
                |_| {},
            )
            .unwrap();
        let out = wait_for(&sessions, info.id, "bg=");
        let bg: i32 = out
            .split("bg=")
            .nth(1)
            .and_then(|s| s.split_whitespace().next())
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(|| panic!("{out:?}"));
        assert_eq!(sessions.pids().len(), 1);
        sessions.close(info.id, Duration::from_millis(300)).unwrap();
        let until = Instant::now() + Duration::from_secs(5);
        // SAFETY: signal 0 only checks that the process exists.
        while unsafe { libc::kill(bg, 0) } == 0 && Instant::now() < until {
            std::thread::sleep(Duration::from_millis(50));
        }
        // SAFETY: as above.
        assert_ne!(unsafe { libc::kill(bg, 0) }, 0, "sleep {bg} still running");
    }
}
