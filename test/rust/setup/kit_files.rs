use super::*;
use crate::kit::tests::TempDir;
use crate::levels::requests::{self, Asked, Reply, Taken, Where};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// D-178, D-179: the agents' launchers live in `agents`, out of `bin`;
/// what an older Atollo left (the launchers of `bin`, `vm.ps1`, the
/// stand-ins of `no-agents`) goes; `forward` has a forwarder for each
/// agent's command, `.cmd`, `.ps1` and sh, that asks the app through the
/// request clients (LEVELS.md §6.8).
#[test]
fn the_agents_launchers_and_their_forwarders() {
    let tmp = TempDir::new("kitfiles-agents");
    let kit = Kit::at(tmp.0.clone(), crate::test_support::kit_arch());
    let dir = kit.kit_files();
    fs::create_dir_all(dir.join("bin")).unwrap();
    fs::create_dir_all(dir.join("no-agents")).unwrap();
    for old in OBSOLETE {
        if old != "no-agents" {
            fs::write(join_rel(&dir, old), "old").unwrap();
        }
    }
    fs::write(dir.join("no-agents").join("claude"), "old").unwrap();
    write_all(&kit).unwrap();
    for old in OBSOLETE {
        assert!(!join_rel(&dir, old).exists(), "{old}");
    }
    for name in [
        "agents/agy",
        "agents/agy.cmd",
        "agents/claude",
        "launch.ps1",
        "launch.sh",
    ] {
        assert!(join_rel(&dir, name).is_file(), "{name}");
    }
    let mut commands = 0;
    for agent in crate::levels::Agent::ALL {
        for cmd in agent.commands() {
            commands += 1;
            let id = agent.id();
            let bat = fs::read_to_string(dir.join(FORWARD_DIR).join(format!("{cmd}.cmd"))).unwrap();
            assert_eq!(bat.matches('\n').count(), bat.matches("\r\n").count());
            assert!(
                bat.contains(&format!(
                    "pwsh -NoLogo -NoProfile -ExecutionPolicy Bypass -File \"%~dp0..\\launch.ps1\" -Origin forwarded -Agent {id} -- %*\r\n"
                )),
                "{bat}"
            );
            // Jumps, never a block: a `)` of the arguments would end it.
            assert!(!bat.contains('('), "{cmd}");
            assert!(
                bat.contains("goto nopwsh\r\n") && bat.ends_with("exit /b 1\r\n"),
                "{cmd}"
            );
            let ps1 = fs::read_to_string(dir.join(FORWARD_DIR).join(format!("{cmd}.ps1"))).unwrap();
            assert!(
                ps1.contains(&format!("-Origin forwarded -Agent {id} -- @args\n")),
                "{ps1}"
            );
            // In a PowerShell of its own: `launch.ps1` exits and sets variables.
            assert!(ps1.contains("& $pwsh -NoLogo -NoProfile"), "{ps1}");
            let sh = fs::read_to_string(dir.join(FORWARD_DIR).join(cmd)).unwrap();
            assert!(sh.starts_with("#!/bin/sh\n") && !sh.contains('\r'), "{cmd}");
            assert!(
                sh.ends_with(&format!(
                    "exec /bin/sh \"$(dirname \"$0\")/../launch.sh\" forwarded {id} \"$@\"\n"
                )),
                "{sh}"
            );
            assert!(is_unix_bin(&format!("{FORWARD_DIR}/{cmd}")));
            assert!(!is_shell(&format!("{FORWARD_DIR}/{cmd}.ps1")));
        }
    }
    assert_eq!(commands, 5, "claude, codex, pi, agy, antigravity");
    assert_eq!(
        crate::levels::env::forward_dir(&kit),
        dir.join(FORWARD_DIR),
        "the folder first on every PC PATH"
    );
}

/// LEVELS.md §4.3: `launch.ps1` decides nothing. It reads its words itself
/// (what follows an agent's command is passed on as it is), writes the
/// request whole then renames it, waits for the reply, takes the request
/// back when no app took it, prints a refusal and exits 1, applies the
/// reply's environment, runs its program in this terminal and exits with
/// its code; Pi's marker carries its pid while Pi runs.
#[test]
fn launch_ps1_writes_waits_and_runs() {
    let script = include_str!("../../../runtime/launch.ps1").replace("\r\n", "\n");
    assert!(!script.contains("param("), "no binding of the user's words");
    for part in [
        "$words = @($args)",
        "if ($word -eq '--' -or ($Origin -eq 'forwarded' -and $Agent)) {",
        "{ $_ -eq '-Command' -or $_ -eq '-c' } { $Command = $value }",
        "[IO.Path]::Combine($PSScriptRoot, '..', 'app', 'levels', 'requests')",
        "$id = [guid]::NewGuid().ToString('N')",
        "[IO.File]::WriteAllText($tmp, $body, [Text.UTF8Encoding]::new($false))",
        "[IO.File]::Move($tmp, $request)",
        "Remove-Item -LiteralPath $request -ErrorAction Stop; $gone = $true",
        "Atollo non e' aperto: apri il terminale da Atollo.",
        "if ($reply.placement -ne 'vm' -and $reply.placement -ne 'pc') {\n    Say $reply.text\n    exit 1\n}",
        "[Environment]::SetEnvironmentVariable($var.Name, [string]$var.Value)",
        "[Environment]::SetEnvironmentVariable($name, $null)",
        "$marker = Join-Path $requests \"$($reply.marker).$PID\"",
        "exit $LASTEXITCODE",
    ] {
        assert!(script.contains(part), "{part}");
    }
    // The marker is written before Pi and removed after it, whatever the way.
    let set = script
        .find("Set-Content -LiteralPath $marker -Value $PID")
        .unwrap();
    let run = script[set..].find("& $reply.program @argv").unwrap() + set;
    let finally = script[run..].find("} finally {").unwrap() + run;
    assert!(script[finally..].starts_with(
        "} finally {\n        Remove-Item -LiteralPath $marker -ErrorAction SilentlyContinue"
    ));
    assert!(
        script.is_ascii(),
        "the console's code page never mangles it"
    );
}

/// The same for `launch.sh`, the POSIX sh client: no bash syntax, the reply
/// read as sh (`set --`), `exec` but for Pi, whose marker goes when it ends.
#[test]
fn launch_sh_writes_waits_and_runs() {
    let script = include_str!("../../../runtime/launch.sh").replace("\r\n", "\n");
    assert!(script.starts_with("#!/bin/sh\n"));
    for part in [
        "requests=\"$kit/../app/levels/requests\"",
        "id=$(od -An -N16 -tx1 /dev/urandom | tr -d ' \\n')",
        ">\"$requests/$id.tmp\" &&\n    mv \"$requests/$id.tmp\" \"$request\"",
        "rm \"$request\" 2>/dev/null; then",
        ". \"$reply\"",
        "atollo_env\n",
        "trap 'rm -f \"$mark\"' EXIT",
        "[ -r \"/proc/$$/winpid\" ] && pid=$(cat \"/proc/$$/winpid\")",
        "exec \"$program\" \"$@\"\n",
    ] {
        assert!(script.contains(part), "{part}");
    }
    let code: String = script
        .lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    for bash in ["[[", "local ", "function ", "$'", "<<<", "${!", "declare "] {
        assert!(!code.contains(bash), "launch.sh uses {bash:?}");
    }
}

/// Takes the first request of `kit` (within a minute) and answers it with
/// `answer_with`; the request it answered.
fn responder(
    kit: &Kit,
    answer_with: impl Fn(&Asked) -> Reply + Send + 'static,
) -> std::thread::JoinHandle<Option<Asked>> {
    let kit = kit.clone();
    std::thread::spawn(move || {
        let until = Instant::now() + Duration::from_secs(60);
        while Instant::now() < until {
            match requests::take(&kit).into_iter().next() {
                Some(Taken::Asked(asked)) => {
                    requests::answer(&kit, &asked.id, &answer_with(&asked)).unwrap();
                    return Some(asked);
                }
                Some(Taken::Malformed { id, why }) => panic!("{id}: {why}"),
                None => {}
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        None
    })
}

/// A reply that runs `sh -c <script>` with `words` on the PC.
fn sh_reply(script: &str, words: &[&str]) -> Reply {
    let mut reply = Reply::error("");
    reply.placement = Where::Pc;
    reply.text = None;
    reply.program = Some("sh".into());
    reply.argv = ["-c", script, "sh"]
        .iter()
        .chain(words)
        .map(|w| (*w).to_owned())
        .collect();
    reply.banner = Some("[Atollo] test".into());
    reply
        .env
        .set
        .insert("ATOLLO_TEST_SET".into(), "set by the reply".into());
    reply
}

/// A kit with the project `shop` and the kit's files, its folder named as
/// the shell names it (macOS: `/private/var` for `/var`).
fn kit_with_shop(name: &str) -> (TempDir, Kit) {
    let tmp = TempDir::new(name);
    #[cfg(unix)]
    let root = fs::canonicalize(&tmp.0).unwrap();
    #[cfg(not(unix))]
    let root = tmp.0.clone();
    let kit = Kit::at(root, crate::test_support::kit_arch());
    fs::create_dir_all(kit.projects().join("shop").join("web")).unwrap();
    write_all(&kit).unwrap();
    (tmp, kit)
}

/// `path` as sh takes it on this host (Git Bash: `/`).
fn for_sh(path: &Path) -> String {
    path.display().to_string().replace('\\', "/")
}

/// `bash` running the sh file `file` in `dir` with `args`, a terminal's
/// variables only.
fn sh_in(bash: &Path, file: &Path, dir: &Path, args: &[&str]) -> Command {
    let mut cmd = bash_cmd(bash, dir);
    cmd.arg(for_sh(file)).args(args).current_dir(dir);
    cmd.env_remove("ATOLLO_LEVEL")
        .env_remove("ATOLLO_LAUNCH_WAIT");
    #[cfg(unix)]
    cmd.env("PWD", dir);
    cmd
}

/// LEVELS.md §6.8: `claude` typed in a project's folder writes a request
/// (forwarded, from the terminal's level, the words as typed) through
/// `launch.sh` and runs what the reply says in the same terminal, with its
/// environment, exiting with its code.
#[test]
fn a_forwarder_asks_the_app() {
    let Some(bash) = host_bash() else {
        eprintln!("no bash on this host: skipped");
        return;
    };
    let (_tmp, kit) = kit_with_shop("kitfiles-forward");
    let web = kit.projects().join("shop").join("web");
    let app = responder(&kit, |_| {
        sh_reply(
            "echo \"ran $* with $ATOLLO_TEST_SET\"; exit 3",
            &["a b", "c"],
        )
    });
    let forwarder = kit.kit_files().join(FORWARD_DIR).join("claude");
    let out = sh_in(&bash, &forwarder, &web, &["--resume", "it's \"mine\""])
        .env("ATOLLO_LEVEL", "project")
        .stdin(Stdio::null())
        .output()
        .unwrap();
    let asked = app.join().unwrap().expect("a request");
    let (stdout, stderr) = (
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
    assert_eq!(out.status.code(), Some(3), "{stdout}\n{stderr}");
    assert_eq!(
        stdout.trim_end(),
        "ran a b c with set by the reply",
        "{stderr}"
    );
    assert!(stderr.contains("[Atollo] test"), "{stderr}");
    assert_eq!(
        asked.origin,
        crate::levels::Origin::Forwarded {
            agent: crate::levels::Agent::Claude,
            from: crate::levels::Level::Project,
        }
    );
    assert_eq!(asked.args, ["--resume", "it's \"mine\""]);
    match requests::request_of(&kit, &asked).place {
        crate::levels::Place::Project { name, below, .. } => {
            assert_eq!((name.as_str(), below.as_deref()), ("shop", Some("web")))
        }
        other => panic!("{other:?} for {}", asked.cwd.display()),
    }
    let left = fs::read_dir(requests::dir(&kit)).unwrap().count();
    assert_eq!(left, 0, "the client removes its reply");
}

/// LEVELS.md §6.8: outside a project's folder (or without the VM) the
/// forwarder prints the refusal's reason and what to do, and exits 1:
/// nothing runs.
#[test]
fn a_forwarder_outside_a_project_says_why() {
    let Some(bash) = host_bash() else {
        eprintln!("no bash on this host: skipped");
        return;
    };
    let (_tmp, kit) = kit_with_shop("kitfiles-forward-out");
    let mine = kit.clone();
    let app = responder(&kit, move |asked| {
        let req = requests::request_of(&mine, asked);
        let facts = crate::levels::Facts {
            vm: crate::levels::VmState::On,
            guest_dir: None,
            restart_pending: false,
            linux_build: true,
            guest_tools: Vec::new(),
        };
        Reply::refused(&crate::levels::decide(&req, &facts), "it")
    });
    let forwarder = kit.kit_files().join(FORWARD_DIR).join("codex");
    let out = sh_in(&bash, &forwarder, kit.root(), &["--version"])
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(app.join().unwrap().is_some());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(1), "{stderr}");
    assert!(out.stdout.is_empty());
    assert!(stderr.contains("cartella di un progetto"), "{stderr}");
}

/// An editor's task through `launch.sh` (VSCodium adds `-c <line>`), and a
/// Pi session: its marker, with the client's pid, is there while Pi runs
/// and gone after.
#[test]
fn launch_sh_runs_a_task_and_marks_pi() {
    let Some(bash) = host_bash() else {
        eprintln!("no bash on this host: skipped");
        return;
    };
    let (_tmp, kit) = kit_with_shop("kitfiles-launch-pi");
    let web = kit.projects().join("shop").join("web");
    let markers = for_sh(&requests::dir(&kit));
    let app = responder(&kit, move |_| {
        let mut reply = sh_reply("ls \"$1\" | grep '^shop[.]pi[.][0-9][0-9]*$'", &[&markers]);
        reply.placement = Where::Vm;
        reply.marker = Some("shop.pi".into());
        reply
    });
    let launch = kit.kit_files().join("launch.sh");
    let out = sh_in(&bash, &launch, &web, &["editorTask", "-c", "npm test"])
        .stdin(Stdio::null())
        .output()
        .unwrap();
    let asked = app.join().unwrap().expect("a request");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "{stdout}\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(stdout.trim_end().starts_with("shop.pi."), "{stdout}");
    assert_eq!(
        asked.origin,
        crate::levels::Origin::EditorTask {
            command: "npm test".into()
        }
    );
    assert!(
        requests::pi_markers(&kit, "shop").is_empty(),
        "removed when Pi ends"
    );
}

/// Without Atollo, nobody takes the request: after the wait the client
/// takes it back, says Atollo is not open and exits 1.
#[test]
fn launch_sh_says_when_atollo_is_not_open() {
    let Some(bash) = host_bash() else {
        eprintln!("no bash on this host: skipped");
        return;
    };
    let (_tmp, kit) = kit_with_shop("kitfiles-launch-closed");
    let launch = kit.kit_files().join("launch.sh");
    let out = sh_in(&bash, &launch, kit.root(), &["editorAgentProfile", "pi"])
        .env("ATOLLO_LAUNCH_WAIT", "1")
        .stdin(Stdio::null())
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("Atollo non è aperto"), "{stderr}");
    let left = fs::read_dir(requests::dir(&kit)).unwrap().count();
    assert_eq!(left, 0, "the request taken back");
}

/// The PowerShell 7 to run `launch.ps1` with on this host: `ATOLLO_TEST_PWSH`,
/// or `pwsh` on the PATH; `None` without one (the test is skipped).
fn host_pwsh() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("ATOLLO_TEST_PWSH") {
        return Some(PathBuf::from(p));
    }
    let ok = Command::new("pwsh")
        .args(["-NoProfile", "-Command", "exit 0"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success());
    ok.then(|| PathBuf::from("pwsh"))
}

/// `launch.ps1` for real: a forwarder's request with the words after `--`
/// as typed (`-c` too), the reply run in the same console with its
/// environment and its exit code; a refusal prints its text, exits 1.
#[test]
fn launch_ps1_forwards_and_refuses() {
    let Some(pwsh) = host_pwsh() else {
        eprintln!("no pwsh on this host: skipped");
        return;
    };
    let (_tmp, kit) = kit_with_shop("kitfiles-launch-ps1");
    let web = kit.projects().join("shop").join("web");
    let launch = kit.kit_files().join("launch.ps1");
    let run = |answer: Reply| {
        let app = responder(&kit, move |_| answer.clone());
        let out = Command::new(&pwsh)
            .args([
                "-NoLogo",
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
            ])
            .arg(&launch)
            .args([
                "-Origin",
                "forwarded",
                "-Agent",
                "codex",
                "--",
                "-c",
                "it's \"mine\"",
            ])
            .current_dir(&web)
            .env_remove("ATOLLO_LEVEL")
            .stdin(Stdio::null())
            .output()
            .unwrap();
        (app.join().unwrap().expect("a request"), out)
    };
    let mut reply = Reply::error("");
    reply.placement = Where::Pc;
    reply.text = None;
    reply.banner = Some("[Atollo] test".into());
    reply
        .env
        .set
        .insert("ATOLLO_TEST_SET".into(), "from the reply".into());
    if cfg!(windows) {
        reply.program = Some("cmd.exe".into());
        reply.argv = ["/d", "/c", "echo ran %ATOLLO_TEST_SET%& exit 3"]
            .map(String::from)
            .into();
    } else {
        reply.program = Some("sh".into());
        reply.argv = ["-c", "echo \"ran $ATOLLO_TEST_SET\"; exit 3"]
            .map(String::from)
            .into();
    }
    let (asked, out) = run(reply);
    let (stdout, stderr) = (
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
    assert_eq!(out.status.code(), Some(3), "{stdout}\n{stderr}");
    assert_eq!(stdout.trim_end(), "ran from the reply", "{stderr}");
    assert!(stderr.contains("[Atollo] test"), "{stderr}");
    assert_eq!(
        asked.origin,
        crate::levels::Origin::Forwarded {
            agent: crate::levels::Agent::Codex,
            from: crate::levels::Level::Manual,
        }
    );
    assert_eq!(asked.args, ["-c", "it's \"mine\""]);
    assert!(matches!(
        requests::request_of(&kit, &asked).place,
        crate::levels::Place::Project { ref name, .. } if name == "shop"
    ));

    let refusal = Reply::error("Gli agenti partono solo nella VM.");
    let (_, out) = run(refusal);
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("solo nella VM"));
    assert!(out.stdout.is_empty());
}

#[test]
fn written_and_restored() {
    let tmp = TempDir::new("kitfiles");
    let kit = Kit::at(tmp.0.clone(), crate::test_support::kit_arch());
    write_all(&kit).unwrap();
    let path = kit.kit_files().join("loopback.js");
    assert!(fs::read_to_string(&path)
        .unwrap()
        .contains("atolloLoopback"));
    assert!(kit.kit_files().join("php-router.php").is_file());
    assert!(kit.kit_files().join("fcgi-guard.php").is_file());
    fs::write(&path, "changed").unwrap();
    write_all(&kit).unwrap();
    assert!(fs::read_to_string(&path)
        .unwrap()
        .contains("atolloLoopback"));
}

/// The MySQL launchers (D-110): `.cmd` in CRLF and sh in LF, written with
/// the other files, each passing the kit's login first, a message on a
/// kit without MySQL.
#[test]
fn mysql_launchers_written() {
    let tmp = TempDir::new("kitfiles-mysql");
    let kit = Kit::at(tmp.0.clone(), crate::test_support::kit_arch());
    write_all(&kit).unwrap();
    let dir = kit.kit_files();
    for stem in MYSQL_LAUNCHERS {
        let cmd = fs::read_to_string(dir.join("bin").join(format!("{stem}.cmd"))).unwrap();
        assert_eq!(cmd.matches('\n').count(), cmd.matches("\r\n").count());
        assert!(cmd.contains(&format!(
            "\"%CP_MYSQL%\\bin\\{stem}.exe\" \"--defaults-extra-file=%ATOLLO_MYSQL_CNF%\" %*\r\n"
        )));
        assert!(cmd.contains("echo MySQL is not installed"));
        assert!(!cmd.contains('('), "no block around %*: {stem}");
        let sh = fs::read_to_string(dir.join("bin").join(stem)).unwrap();
        assert!(sh.starts_with("#!/bin/sh\n") && !sh.contains('\r'));
        assert!(sh.contains(&format!(
            "exec \"$CP_MYSQL/bin/{stem}\" --defaults-extra-file=\"$ATOLLO_MYSQL_CNF\" \"$@\"\n"
        )));
        assert!(sh.contains("MySQL is not installed"));
    }
}

/// The sh launcher really runs MySQL's program with the login first, the
/// arguments as given, and a message without MySQL.
#[cfg(unix)]
#[test]
fn mysql_launcher_runs_the_right_program() {
    use std::os::unix::fs::PermissionsExt;
    let tmp = TempDir::new("kitfiles-mysql-run");
    let kit = Kit::at(tmp.0.clone(), crate::test_support::kit_arch());
    write_all(&kit).unwrap();
    let fake = |dir: &Path, name: &str| {
        fs::create_dir_all(dir.join("bin")).unwrap();
        let p = dir.join("bin").join(name);
        fs::write(&p, format!("#!/bin/sh\necho \"{name} $*\"\n")).unwrap();
        fs::set_permissions(&p, fs::Permissions::from_mode(0o755)).unwrap();
    };
    let mysql = tmp.0.join("my");
    let cnf = tmp.0.join("mysql.cnf");
    let launcher = kit.kit_files().join("bin").join("mysql");
    let run = || {
        let out = Command::new(&launcher)
            .args(["-e", "SELECT 1"])
            .env("CP_MYSQL", &mysql)
            .env("ATOLLO_MYSQL_CNF", &cnf)
            .output()
            .unwrap();
        String::from_utf8_lossy(&out.stdout).trim().to_owned()
    };
    assert_eq!(run(), "", "no MySQL: only a message on stderr");
    fake(&mysql, "mysql");
    assert_eq!(run(), "mysql -e SELECT 1", "no login yet");
    fs::write(&cnf, "[client]\n").unwrap();
    assert_eq!(
        run(),
        format!("mysql --defaults-extra-file={} -e SELECT 1", cnf.display())
    );
}

#[test]
fn line_endings_by_kind() {
    let text = "a\r\nb\nc\r\n";
    assert_eq!(with_line_endings("bin/agy.cmd", text), "a\r\nb\r\nc\r\n");
    assert_eq!(with_line_endings("bashrc", text), "a\nb\nc\n");
    assert_eq!(with_line_endings("bin/gradle", text), "a\nb\nc\n");
    assert_eq!(with_line_endings("profile.ps1", text), text);
    assert!(is_shell("bashrc") && is_shell("agents/agy") && is_shell("forward/claude"));
    assert!(is_shell("launch.sh") && is_unix_bin("launch.sh"));
    assert!(!is_shell("launch.ps1") && !is_shell("forward/claude.ps1"));
    assert_eq!(
        with_line_endings("forward/claude.cmd", text),
        "a\r\nb\r\nc\r\n"
    );
    assert!(!is_shell("agents/agy.cmd") && !is_shell("profile.ps1"));
    assert!(!is_unix_bin("bashrc") && is_unix_bin("bin/gradle"));
}

/// The shell files the kit writes: no CR (sh reads it as part of the word),
/// launchers executable on Unix, batch files in CRLF.
#[test]
fn shell_files_lf_and_executable() {
    let tmp = TempDir::new("kitfiles-sh");
    let kit = Kit::at(tmp.0.clone(), crate::test_support::kit_arch());
    write_all(&kit).unwrap();
    let dir = kit.kit_files();
    let generated = generated();
    let names: Vec<&str> = FILES
        .iter()
        .map(|(n, _)| *n)
        .chain(generated.iter().map(|(n, _)| n.as_str()))
        .collect();
    for name in names.iter().copied() {
        let bytes = fs::read(join_rel(&dir, name)).unwrap();
        if is_shell(name) {
            assert!(!bytes.contains(&b'\r'), "{name} has CR");
        }
        if name.ends_with(".cmd") {
            let text = String::from_utf8(bytes).unwrap();
            assert_eq!(
                text.matches('\n').count(),
                text.matches("\r\n").count(),
                "{name}"
            );
        }
    }
    for name in [
        "agents/agy",
        "agents/claude",
        "bin/gradle",
        "bin/atollo",
        "launch.sh",
        "forward/pi",
    ] {
        let text = fs::read_to_string(join_rel(&dir, name)).unwrap();
        assert!(text.starts_with("#!/bin/sh\n"), "{name}");
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for name in names.iter().copied().filter(|n| is_unix_bin(n)) {
            let mode = fs::metadata(join_rel(&dir, name))
                .unwrap()
                .permissions()
                .mode();
            assert_eq!(mode & 0o777, 0o755, "{name}");
        }
        // A wrong mode left by a copy is put back at the next start.
        let agy = dir.join("agents/agy");
        fs::set_permissions(&agy, fs::Permissions::from_mode(0o644)).unwrap();
        write_all(&kit).unwrap();
        assert_eq!(
            fs::metadata(&agy).unwrap().permissions().mode() & 0o777,
            0o755
        );
    }
}

/// `atollo` in the kit's terminals runs the kit's exe (`ATOLLO_EXE`) with
/// the words given; without the variable it says so and fails.
#[test]
fn atollo_runs_the_kits_exe() {
    let tmp = TempDir::new("kitfiles-atollo");
    let kit = Kit::at(tmp.0.clone(), crate::test_support::kit_arch());
    write_all(&kit).unwrap();
    let dir = kit.kit_files();
    #[cfg(windows)]
    let (mut with, mut without, exe) = {
        let launcher = join_rel(&dir, "bin/atollo.cmd");
        let cmd = |l: &std::path::Path| {
            let mut c = Command::new("cmd");
            c.args(["/d", "/c"]).arg(l);
            c
        };
        let exe = std::path::PathBuf::from(std::env::var("SystemRoot").unwrap())
            .join("System32")
            .join("cmd.exe");
        (cmd(&launcher), cmd(&launcher), exe)
    };
    #[cfg(unix)]
    let (mut with, mut without, exe) = {
        let launcher = join_rel(&dir, "bin/atollo");
        (
            Command::new(&launcher),
            Command::new(&launcher),
            std::path::PathBuf::from("/bin/echo"),
        )
    };
    #[cfg(windows)]
    with.args(["/d", "/c", "echo", "components"]);
    #[cfg(unix)]
    with.arg("components");
    let out = with.env("ATOLLO_EXE", &exe).output().unwrap();
    assert!(out.status.success(), "{out:?}");
    assert!(String::from_utf8_lossy(&out.stdout).contains("components"));
    let out = without.env_remove("ATOLLO_EXE").output().unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("ATOLLO_EXE"));
}

/// The bashrc is also read by the bash 3.2 of macOS: none of the syntax that
/// came with bash 4.
#[test]
fn bashrc_has_no_bash4_syntax() {
    let rc = include_str!("../../../runtime/bashrc");
    let code: String = rc
        .lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");
    for bash4 in [
        ",,}",
        "^^}",
        ",}",
        "^}",
        "declare -A",
        "local -A",
        "typeset -A",
        "mapfile",
        "readarray",
        ";&",
        "declare -n",
        "local -n",
        "coproc",
        "${!name[@]}",
        "&>>",
        "|&",
    ] {
        assert!(!code.contains(bash4), "bashrc uses {bash4:?}");
    }
}

/// Git Bash on Windows (never `C:\Windows\System32\bash.exe`, which is WSL),
/// the `bash` on PATH elsewhere; `None` when there is none.
fn host_bash() -> Option<PathBuf> {
    if cfg!(windows) {
        let out = Command::new("git").arg("--exec-path").output().ok()?;
        // <Git>\mingw64\libexec\git-core
        let exec = PathBuf::from(String::from_utf8_lossy(&out.stdout).trim());
        let bash = exec.ancestors().nth(3)?.join("bin").join("bash.exe");
        bash.is_file().then_some(bash)
    } else {
        let ok = Command::new("bash")
            .arg("--version")
            .stdout(Stdio::null())
            .status()
            .is_ok_and(|s| s.success());
        ok.then(|| PathBuf::from("bash"))
    }
}

fn run(cmd: &mut Command) -> (bool, String, String) {
    let out = cmd.stdin(Stdio::null()).output().unwrap();
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// A bash with only the variables of the test: no CP_* of a kit terminal.
fn bash_cmd(bash: &Path, home: &Path) -> Command {
    let mut cmd = Command::new(bash);
    for name in [
        "CP_ROOT",
        "CP_DATA",
        "ATOLLO_TERMINAL",
        "BASH_ENV",
        "ENV",
        "PWD",
    ] {
        cmd.env_remove(name);
    }
    cmd.env("HOME", home);
    cmd
}

/// `bash -n` on the files as written, and the plan's acceptance run:
/// `bash --rcfile bashrc -i -c 'echo ok'`.
#[test]
fn bash_parses_written_files() {
    let Some(bash) = host_bash() else {
        eprintln!("no bash on this host: skipped");
        return;
    };
    let tmp = TempDir::new("kitfiles-bash");
    let kit = Kit::at(tmp.0.clone(), crate::test_support::kit_arch());
    write_all(&kit).unwrap();
    let dir = kit.kit_files();
    for name in FILES.iter().map(|(n, _)| *n).filter(|n| is_shell(n)) {
        let (ok, _, err) = run(bash_cmd(&bash, &tmp.0).arg("-n").arg(join_rel(&dir, name)));
        assert!(ok, "bash -n {name}: {err}");
    }
    let rc = dir.join("bashrc");
    let (ok, out, err) = run(bash_cmd(&bash, &tmp.0)
        .arg("--rcfile")
        .arg(&rc)
        .args(["-i", "-c", "echo ok"]));
    assert!(ok && out.trim_end().ends_with("ok"), "{out}\n{err}");
}

/// The versions of a project swapped in by the bashrc (D-80), compared
/// ignoring case on Windows and macOS, exactly on Linux.
#[test]
fn bashrc_swaps_project_versions() {
    let Some(bash) = host_bash() else {
        eprintln!("no bash on this host: skipped");
        return;
    };
    let tmp = TempDir::new("kitfiles-versions");
    let kit = Kit::at(tmp.0.clone(), crate::test_support::kit_arch());
    write_all(&kit).unwrap();
    let root = tmp.0.join("root");
    let data = tmp.0.join("data");
    let here = root.join("projects").join("demo").join("src");
    fs::create_dir_all(&here).unwrap();
    fs::create_dir_all(data.join("projects").join("demo")).unwrap();
    let (from, to, value) = if cfg!(windows) {
        (r"C:\Apps\Jdk\17", r"C:\Apps\Jdk\21", r"c:\apps\jdk\17\bin")
    } else if cfg!(target_os = "macos") {
        ("/Apps/Jdk/17", "/Apps/Jdk/21", "/apps/jdk/17/bin")
    } else {
        ("/apps/jdk/17", "/apps/jdk/21", "/apps/jdk/17/bin")
    };
    fs::write(
        data.join("projects").join("demo").join("versions.txt"),
        format!("{from}\t{to}\r\n/none\t/other\n"),
    )
    .unwrap();
    // The shell's working folder as the system names it (/private/var on
    // macOS for /var).
    #[cfg(unix)]
    let (root, data, here) = (
        fs::canonicalize(&root).unwrap(),
        fs::canonicalize(&data).unwrap(),
        fs::canonicalize(&here).unwrap(),
    );
    let probe = "echo \"J=$JAVA_HOME\"; shopt -q nocasematch && echo nocase; \
                 type __atollo_p >/dev/null 2>&1 && echo leaked; true";
    let (ok, out, err) = run(bash_cmd(&bash, &tmp.0)
        .current_dir(&here)
        .env("CP_ROOT", &root)
        .env("CP_DATA", &data)
        .env("JAVA_HOME", value)
        .arg("--rcfile")
        .arg(kit.kit_files().join("bashrc"))
        .args(["-i", "-c", probe]));
    assert!(ok, "{out}\n{err}");
    assert!(
        out.contains(&format!("J={to}{}", &value[from.len()..])),
        "{out}\n{err}"
    );
    // nocasematch is put back for the user's shell, the helpers removed.
    assert!(!out.contains("nocase"), "{out}");
    assert!(!out.contains("leaked"), "{out}");
    // A sibling folder that only starts with the same text is not swapped.
    let (_, out, _) = run(bash_cmd(&bash, &tmp.0)
        .current_dir(&here)
        .env("CP_ROOT", &root)
        .env("CP_DATA", &data)
        .env("JAVA_HOME", format!("{from}0"))
        .arg("--rcfile")
        .arg(kit.kit_files().join("bashrc"))
        .args(["-i", "-c", "echo \"J=$JAVA_HOME\""]));
    assert!(out.contains(&format!("J={from}0")), "{out}");
}
