use super::*;
use crate::kit::tests::TempDir;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// D-178: the agents' launchers live in `agents`, out of `bin` (the
/// launchers an older Atollo left there go); `no-agents` has a stand-in
/// for each agent's command, `.cmd` and sh, that starts nothing and says
/// where the agents run.
#[test]
fn the_agents_launchers_and_their_stand_ins() {
    let tmp = TempDir::new("kitfiles-agents");
    let kit = Kit::at(tmp.0.clone(), crate::test_support::kit_arch());
    let dir = kit.kit_files();
    fs::create_dir_all(dir.join("bin")).unwrap();
    for old in OBSOLETE {
        fs::write(join_rel(&dir, old), "old").unwrap();
    }
    write_all(&kit).unwrap();
    for old in OBSOLETE {
        assert!(!join_rel(&dir, old).exists(), "{old}");
    }
    for name in ["agents/agy", "agents/agy.cmd", "agents/claude"] {
        assert!(join_rel(&dir, name).is_file(), "{name}");
    }
    for stem in AGENT_COMMANDS {
        let cmd = fs::read_to_string(dir.join(NO_AGENTS_DIR).join(format!("{stem}.cmd"))).unwrap();
        assert_eq!(cmd.matches('\n').count(), cmd.matches("\r\n").count());
        assert!(cmd.contains(&format!("echo %{AGENTS_NOTICE_VAR}% 1>&2\r\n")));
        // Jumps, never a block: a `)` of the notice would end it.
        assert!(
            cmd.contains("exit /b 1\r\n") && !cmd.contains(") else ("),
            "{stem}"
        );
        assert!(cmd.contains("goto plain\r\n"), "{stem}");
        let sh = fs::read_to_string(dir.join(NO_AGENTS_DIR).join(stem)).unwrap();
        assert!(
            sh.starts_with("#!/bin/sh\n") && !sh.contains('\r'),
            "{stem}"
        );
        assert!(sh.ends_with("exit 1\n") && !sh.contains("exec"), "{stem}");
    }
}

/// The sh stand-in really starts nothing: it prints the notice of the
/// shell (or its own) and fails.
#[cfg(unix)]
#[test]
fn a_stand_in_says_where_the_agents_run() {
    let tmp = TempDir::new("kitfiles-agents-run");
    let kit = Kit::at(tmp.0.clone(), crate::test_support::kit_arch());
    write_all(&kit).unwrap();
    let claude = kit.kit_files().join(NO_AGENTS_DIR).join("claude");
    let out = Command::new(&claude)
        .arg("--version")
        .env(AGENTS_NOTICE_VAR, "only in the VM")
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(out.stdout.is_empty());
    assert_eq!(String::from_utf8_lossy(&out.stderr), "only in the VM\n");
    let out = Command::new(&claude)
        .env_remove(AGENTS_NOTICE_VAR)
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("VM"));
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
    assert!(is_shell("bashrc") && is_shell("agents/agy") && is_shell("no-agents/claude"));
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
    for (name, _) in FILES {
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
    for name in ["agents/agy", "agents/claude", "bin/gradle", "bin/atollo"] {
        let text = fs::read_to_string(join_rel(&dir, name)).unwrap();
        assert!(text.starts_with("#!/bin/sh\n"), "{name}");
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for name in FILES.iter().map(|(n, _)| *n).filter(|n| is_unix_bin(n)) {
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
