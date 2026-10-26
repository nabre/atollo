use super::*;
use crate::kit::tests::TempDir;
use crate::kit::Target;

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
    let kit = Kit::at(tmp.0.join("k (1) & co"), crate::test_support::kit_arch()).for_target(target);
    kit.prepare().unwrap();
    for rel in installed {
        let f = crate::common::fsx::join_rel(kit.apps(), rel);
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


/// What a fake PTY was asked to do.
#[derive(Default)]
struct Calls {
    written: Vec<u8>,
    sizes: Vec<(u16, u16)>,
    hung_up: u32,
    /// The pids of every kill.
    killed: Vec<Vec<u32>>,
    released: u32,
    /// The processes left in the session besides the program (its jobs).
    left: Vec<u32>,
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

    /// A program whose jobs `left` stay in its session.
    fn with_jobs(pid: u32, left: &[u32]) -> FakeIo {
        let io = FakeIo::new(pid);
        io.calls().left = left.to_vec();
        io
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
    fn left(&self, _leader_running: bool) -> Vec<u32> {
        self.calls().left.clone()
    }
    fn kill(&self, pids: &[u32]) {
        let mut c = self.calls();
        c.killed.push(pids.to_vec());
        c.left.retain(|p| !pids.contains(p));
    }
    fn release(&self) {
        self.calls().released += 1;
    }
}

/// `PtyInfo` says where the session runs (LEVELS.md §4.1, §8.3): its
/// level, the label of its tab and its sandbox, as the UI reads them.
#[test]
fn pty_info_serializes_its_level_label_and_sandbox() {
    use crate::levels::labels::SessionLabel;
    use crate::levels::Level;
    let sessions = Sessions::default();
    let vm = sessions.insert_at(
        "shop · VM".into(),
        Path::new("/p/shop"),
        SessionLabel::vm(Level::Project, "atollo-p-shop-1a2b"),
        Box::new(FakeIo::new(1)),
    );
    let json = serde_json::to_value(vm.info()).unwrap();
    assert_eq!(json["level"], "project");
    assert_eq!(json["label"], "projectVm");
    assert_eq!(json["sandbox"], "atollo-p-shop-1a2b");
    let agent = sessions.insert_at(
        "shop · claude".into(),
        Path::new("/p/shop"),
        SessionLabel::vm(Level::Agent, "atollo-a-claude-shop-1a2b"),
        Box::new(FakeIo::new(2)),
    );
    assert_eq!(
        serde_json::to_value(agent.info()).unwrap()["label"],
        "agentVm"
    );
    let pc = sessions.insert_at(
        "shop · pwsh".into(),
        Path::new("/p/shop"),
        SessionLabel::pc(Level::Project),
        Box::new(FakeIo::new(3)),
    );
    let json = serde_json::to_value(pc.info()).unwrap();
    assert_eq!(json["level"], "project");
    assert_eq!(json["label"], "pc");
    assert!(json["sandbox"].is_null());
    // The kit's own sessions (`shell`, `command`, `program`): the PC, level 1.
    let kit = sessions.insert("x".into(), Path::new("/p"), Box::new(FakeIo::new(4)));
    let json = serde_json::to_value(kit.info()).unwrap();
    assert_eq!(
        (json["level"].as_str(), json["label"].as_str()),
        (Some("manual"), Some("pc"))
    );
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

    // "Rimuovi l'unità": the jobs too, also those of a shell that ended.
    io.calls().left = vec![4300];
    let running = FakeIo::with_jobs(5000, &[5001]);
    sessions.insert("b".into(), Path::new("/p"), Box::new(running));
    assert_eq!(sessions.pids(), [4300, 5000, 5001]);
}

/// D-149: a session moves into a window of its own and back; it keeps
/// running and keeps its output meanwhile.
#[test]
fn sessions_detach_and_dock() {
    let sessions = Sessions::default();
    sessions.insert(
        "app · bash".into(),
        Path::new("/p"),
        Box::new(FakeIo::new(1)),
    );
    assert!(!sessions.info(1).unwrap().detached);
    let moved = sessions.set_detached(1, true).unwrap();
    assert!(moved.detached && moved.running);
    assert!(sessions.list()[0].detached);
    assert!(!sessions.set_detached(1, false).unwrap().detached);
    assert!(sessions.set_detached(9, true).is_err(), "no such session");
    assert!(sessions.info(9).is_err());
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
    assert_eq!(stubborn.calls().killed, [vec![10]]);
    assert_eq!(stubborn.calls().released, 1);
    assert!(sessions.list().is_empty());
    assert!(sessions.close(1, Duration::ZERO).is_err());

    // A shell ends on the hang-up: not killed.
    let polite = FakeIo::new(11);
    let s = sessions.insert("b".into(), Path::new("/p"), Box::new(polite.clone()));
    *polite.on_hang_up.lock().unwrap() = Some(Arc::clone(&s));
    sessions.close(s.info().id, Duration::from_secs(5)).unwrap();
    assert_eq!(polite.calls().hung_up, 1);
    assert!(polite.calls().killed.is_empty());
    assert_eq!(polite.calls().released, 1);

    // Already ended: only let go.
    let ended = FakeIo::new(12);
    let s = sessions.insert("c".into(), Path::new("/p"), Box::new(ended.clone()));
    s.exited(None);
    sessions.close(s.info().id, Duration::from_secs(5)).unwrap();
    assert_eq!(ended.calls().hung_up, 0);
    assert!(ended.calls().killed.is_empty());
    assert_eq!(ended.calls().released, 1);
}

#[test]
fn jobs_left_by_a_shell_that_ended_are_killed() {
    // An interactive bash ends at once on the hang-up; a job of it that
    // ignores SIGHUP (`caddy run`, `nohup x &`) stays in its session.
    let sessions = Sessions::default();
    let shell = FakeIo::with_jobs(20, &[21, 22]);
    let s = sessions.insert("a".into(), Path::new("/p"), Box::new(shell.clone()));
    *shell.on_hang_up.lock().unwrap() = Some(Arc::clone(&s));
    let started = Instant::now();
    sessions
        .close(s.info().id, Duration::from_millis(200))
        .unwrap();
    assert!(
        started.elapsed() >= Duration::from_millis(200),
        "grace for the jobs"
    );
    let c = shell.calls();
    assert_eq!(c.hung_up, 1);
    assert_eq!(c.killed, [vec![21, 22]], "the shell itself was reaped");
    assert_eq!(c.released, 1);
    drop(c);

    // `x &` then `exit`: the tab shows the shell ended, its job runs on;
    // closing the tab kills it at once, with no grace.
    let ended = FakeIo::with_jobs(30, &[31]);
    let s = sessions.insert("b".into(), Path::new("/p"), Box::new(ended.clone()));
    s.exited(Some(0));
    let started = Instant::now();
    sessions.close(s.info().id, Duration::from_secs(5)).unwrap();
    assert!(started.elapsed() < Duration::from_secs(2));
    assert_eq!(ended.calls().hung_up, 0);
    assert_eq!(ended.calls().killed, [vec![31]]);

    // Still running past the grace: the program with its jobs.
    let stubborn = FakeIo::with_jobs(40, &[41]);
    let s = sessions.insert("c".into(), Path::new("/p"), Box::new(stubborn.clone()));
    sessions
        .close(s.info().id, Duration::from_millis(20))
        .unwrap();
    assert_eq!(stubborn.calls().killed, [vec![40, 41]]);
}

#[test]
fn close_all_at_exit_ends_every_session() {
    let sessions = Sessions::default();
    let ios: Vec<FakeIo> = (1..=3).map(FakeIo::new).collect();
    for io in &ios {
        sessions.insert("x".into(), Path::new("/p"), Box::new(io.clone()));
    }
    // A shell that ended on the hang-up, leaving a job.
    let shell = FakeIo::with_jobs(9, &[90]);
    let s = sessions.insert("y".into(), Path::new("/p"), Box::new(shell.clone()));
    *shell.on_hang_up.lock().unwrap() = Some(s);
    sessions.close_all(Duration::from_millis(20));
    assert!(sessions.list().is_empty());
    for io in &ios {
        let c = io.calls();
        assert_eq!((c.hung_up, c.released), (1, 1));
        assert_eq!(c.killed, [vec![io.pid]]);
    }
    assert_eq!(shell.calls().killed, [vec![90]]);
}

#[test]
fn binary_input_is_one_byte_per_character() {
    assert_eq!(
        latin1("\x1b[M \u{ff}\u{a0}"),
        Some(vec![0x1b, b'[', b'M', b' ', 0xff, 0xa0])
    );
    assert_eq!(latin1(""), Some(Vec::new()));
    // Never text in UTF-8 by mistake.
    assert_eq!(latin1("à€"), None);
    let sessions = Sessions::default();
    let io = FakeIo::new(42);
    let id = sessions
        .insert("t".into(), Path::new("/x"), Box::new(io.clone()))
        .info()
        .id;
    sessions.write_bytes(id, &[0xff, 0x00]).unwrap();
    assert_eq!(io.calls().written, [0xff, 0x00]);
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
            conpty: None,
            at: crate::levels::labels::SessionLabel::pc(crate::levels::Level::Manual),
        }
    }

    use crate::test_support::wait_for;

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
                "it",
                |_| {},
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
                "it",
                |_| {},
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
                "it",
                |_| {},
                |_, _| {},
                |_| {},
            )
            .unwrap();
        let bg = pid_after(&sessions, info.id, "bg=");
        // The program and its job, for "Rimuovi l'unità".
        let pids = sessions.pids();
        assert_eq!(pids.len(), 2, "{pids:?}");
        assert!(pids.contains(&u32::try_from(bg).unwrap()), "{pids:?}");
        sessions.close(info.id, Duration::from_millis(300)).unwrap();
        assert_gone(bg);
    }

    /// The first number written after `tag` (the echo of a typed line
    /// holds the tag too, followed by its `$$`).
    fn pid_after(sessions: &Sessions, id: u64, tag: &str) -> i32 {
        let until = Instant::now() + Duration::from_secs(20);
        loop {
            let out = sessions.backlog(id).unwrap().data;
            let found = out.split(tag).skip(1).find_map(|s| {
                let digits: String = s.chars().take_while(char::is_ascii_digit).collect();
                digits.parse().ok()
            });
            match found {
                Some(pid) => return pid,
                None if Instant::now() > until => panic!("no {tag} in {out:?}"),
                None => std::thread::sleep(Duration::from_millis(20)),
            }
        }
    }

    fn assert_gone(pid: i32) {
        let until = Instant::now() + Duration::from_secs(5);
        // SAFETY: signal 0 only checks that the process exists.
        while unsafe { libc::kill(pid, 0) } == 0 && Instant::now() < until {
            std::thread::sleep(Duration::from_millis(50));
        }
        // SAFETY: as above.
        assert_ne!(unsafe { libc::kill(pid, 0) }, 0, "{pid} still running");
    }

    fn interactive_bash() -> Launch {
        Launch {
            program: PathBuf::from("/bin/bash"),
            args: vec!["--norc".into(), "--noprofile".into(), "-i".into()],
            shell: PathBuf::from("/bin/bash"),
            title: "test".into(),
            conpty: None,
            at: crate::levels::labels::SessionLabel::pc(crate::levels::Level::Manual),
        }
    }

    fn open_bash(sessions: &Sessions, dir: &Path) -> u64 {
        sessions
            .open(
                &interactive_bash(),
                &env(),
                dir,
                "it",
                |_| {},
                |_, _| {},
                |_| {},
            )
            .unwrap()
            .id
    }

    #[test]
    fn close_ends_a_job_of_an_interactive_bash_that_ignores_the_hang_up() {
        // Job control: the job has a group of its own, bash ends at once
        // on the hang-up, the job stays (reparented) until the session's
        // processes are killed.
        let tmp = TempDir::new("pty-bash-fg");
        let sessions = Sessions::default();
        let id = open_bash(&sessions, &tmp.0);
        sessions
            .write(id, "sh -c 'trap \"\" HUP; echo fg=$$; exec sleep 60'\r")
            .unwrap();
        let fg = pid_after(&sessions, id, "fg=");
        sessions.close(id, Duration::from_millis(300)).unwrap();
        assert_gone(fg);
    }

    #[test]
    fn closing_a_bash_that_ended_ends_its_background_job() {
        // `x &` then `exit`: bash does not hang up its running jobs.
        let tmp = TempDir::new("pty-bash-bg");
        let sessions = Sessions::default();
        let id = open_bash(&sessions, &tmp.0);
        sessions
            .write(id, "sh -c 'trap \"\" HUP; exec sleep 60' & echo bg=$!\r")
            .unwrap();
        let bg = pid_after(&sessions, id, "bg=");
        sessions.write(id, "exit\r").unwrap();
        let until = Instant::now() + Duration::from_secs(20);
        while sessions.running() > 0 && Instant::now() < until {
            std::thread::sleep(Duration::from_millis(20));
        }
        assert_eq!(sessions.running(), 0, "bash did not exit");
        // SAFETY: signal 0 only checks that the process exists.
        assert_eq!(unsafe { libc::kill(bg, 0) }, 0, "the job runs on");
        sessions.close(id, Duration::from_millis(300)).unwrap();
        assert_gone(bg);
    }

    #[test]
    fn exit_never_comes_before_open() {
        let tmp = TempDir::new("pty-order");
        let sessions = Sessions::default();
        let events = Arc::new(Mutex::new(Vec::new()));
        let (tx, rx) = channel();
        let (opened, ended) = (Arc::clone(&events), Arc::clone(&events));
        sessions
            .open(
                &sh("exit 5"),
                &env(),
                &tmp.0,
                "it",
                move |info| {
                    opened
                        .lock()
                        .unwrap()
                        .push(format!("open {}", info.running))
                },
                |_, _| {},
                move |info| {
                    ended.lock().unwrap().push(format!("exit {:?}", info.code));
                    tx.send(()).unwrap();
                },
            )
            .unwrap();
        rx.recv_timeout(Duration::from_secs(20)).unwrap();
        assert_eq!(*events.lock().unwrap(), ["open true", "exit Some(5)"]);
    }

    #[test]
    fn a_missing_folder_or_a_program_that_cannot_run_is_told() {
        let tmp = TempDir::new("pty-ready");
        let sessions = Sessions::default();
        let missing = tmp.0.join("nope");
        let e = sessions
            .open(
                &sh("true"),
                &env(),
                &missing,
                "it",
                |_| {},
                |_, _| {},
                |_| {},
            )
            .unwrap_err();
        assert!(e.to_string().contains("non esiste"), "{e:#}");
        // In the language of the interface.
        let e = sessions
            .open(
                &sh("true"),
                &env(),
                &missing,
                "en",
                |_| {},
                |_, _| {},
                |_| {},
            )
            .unwrap_err();
        assert_eq!(
            e.to_string(),
            crate::common::i18n::t(
                "en",
                "La cartella {0} non esiste.",
                &[&missing.display().to_string()]
            )
        );
        assert!(!e.to_string().contains("non esiste"), "{e:#}");
        // Without its execute bit (as on a copy that lost it).
        let script = tmp.0.join("agy");
        std::fs::write(&script, "#!/bin/sh\necho hi\n").unwrap();
        let e = sessions
            .open(
                &Launch {
                    program: script.clone(),
                    args: Vec::new(),
                    shell: PathBuf::from(BASH),
                    title: "agy".into(),
                    conpty: None,
                    at: crate::levels::labels::SessionLabel::pc(crate::levels::Level::Manual),
                },
                &env(),
                &tmp.0,
                "it",
                |_| {},
                |_, _| {},
                |_| {},
            )
            .unwrap_err();
        assert!(e.to_string().contains("non è installato"), "{e:#}");
        assert!(sessions.list().is_empty());
    }

    #[test]
    fn a_session_without_a_locale_gets_a_utf8_one() {
        // An app started from Finder: no LANG, LC_ALL or LC_CTYPE.
        let tmp = TempDir::new("pty-locale");
        let sessions = Sessions::default();
        let (tx, rx) = channel();
        let info = sessions
            .open(
                &sh("printf 'loc=%s|%s|%s\\n' \"$LANG\" \"$LC_ALL\" \"$LC_CTYPE\""),
                &env(),
                &tmp.0,
                "it",
                |_| {},
                |_, _| {},
                move |info| tx.send(info).unwrap(),
            )
            .unwrap();
        rx.recv_timeout(Duration::from_secs(20)).unwrap();
        let out = wait_for(&sessions, info.id, "loc=");
        let expected = if cfg!(target_os = "macos") {
            "loc=en_US.UTF-8||"
        } else {
            "loc=C.UTF-8||"
        };
        assert!(out.contains(expected), "{out:?}");
        sessions.close(info.id, Duration::ZERO).unwrap();
    }

    #[test]
    fn the_exit_comes_after_the_last_output() {
        let tmp = TempDir::new("pty-tail");
        let sessions = Sessions::default();
        let events = Arc::new(Mutex::new(Vec::<String>::new()));
        let (tx, rx) = channel();
        let (out, ended) = (Arc::clone(&events), Arc::clone(&events));
        sessions
            .open(
                &sh("i=0; while [ $i -lt 3000 ]; do echo line$i; i=$((i+1)); done; echo tail-end"),
                &env(),
                &tmp.0,
                "it",
                |_| {},
                move |_, chunk| out.lock().unwrap().push(chunk.data.clone()),
                move |_| {
                    ended.lock().unwrap().push("<exit>".into());
                    tx.send(()).unwrap();
                },
            )
            .unwrap();
        rx.recv_timeout(Duration::from_secs(20)).unwrap();
        let events = events.lock().unwrap();
        assert_eq!(events.last().map(String::as_str), Some("<exit>"));
        let text: String = events[..events.len() - 1].concat();
        assert!(text.contains("tail-end"), "{text:?}");
    }
}
