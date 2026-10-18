use super::*;
use crate::kit::{tests::TempDir, Arch};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[test]
fn written_and_restored() {
    let tmp = TempDir::new("kitfiles");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    write_all(&kit).unwrap();
    let path = kit.kit_files().join("loopback.js");
    assert!(fs::read_to_string(&path)
        .unwrap()
        .contains("devkitLoopback"));
    assert!(kit.kit_files().join("php-router.php").is_file());
    assert!(kit.kit_files().join("fcgi-guard.php").is_file());
    fs::write(&path, "changed").unwrap();
    write_all(&kit).unwrap();
    assert!(fs::read_to_string(&path)
        .unwrap()
        .contains("devkitLoopback"));
}

/// The MySQL launchers (D-110): `.cmd` in CRLF and sh in LF, written with
/// the other files, each passing the kit's login first, MariaDB's program
/// on a kit without MySQL.
#[test]
fn mysql_launchers_written() {
    let tmp = TempDir::new("kitfiles-mysql");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    write_all(&kit).unwrap();
    let dir = kit.kit_files();
    for (stem, mariadb) in MYSQL_LAUNCHERS {
        let cmd = fs::read_to_string(dir.join("bin").join(format!("{stem}.cmd"))).unwrap();
        assert_eq!(cmd.matches('\n').count(), cmd.matches("\r\n").count());
        assert!(cmd.contains(&format!(
            "\"%CP_MYSQL%\\bin\\{stem}.exe\" \"--defaults-extra-file=%DEVKIT_MYSQL_CNF%\" %*\r\n"
        )));
        assert!(cmd.contains(&format!("\"%CP_MARIADB%\\bin\\{mariadb}.exe\" %*\r\n")));
        assert!(!cmd.contains('('), "no block around %*: {stem}");
        let sh = fs::read_to_string(dir.join("bin").join(stem)).unwrap();
        assert!(sh.starts_with("#!/bin/sh\n") && !sh.contains('\r'));
        assert!(sh.contains(&format!(
            "exec \"$CP_MYSQL/bin/{stem}\" --defaults-extra-file=\"$DEVKIT_MYSQL_CNF\" \"$@\"\n"
        )));
        assert!(sh.contains(&format!("exec \"$CP_MARIADB/bin/{mariadb}\" \"$@\"\n")));
    }
}

/// The sh launcher really runs MySQL's program with the login first, the
/// arguments as given, and MariaDB's without MySQL.
#[cfg(unix)]
#[test]
fn mysql_launcher_runs_the_right_program() {
    use std::os::unix::fs::PermissionsExt;
    let tmp = TempDir::new("kitfiles-mysql-run");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    write_all(&kit).unwrap();
    let fake = |dir: &Path, name: &str| {
        fs::create_dir_all(dir.join("bin")).unwrap();
        let p = dir.join("bin").join(name);
        fs::write(&p, format!("#!/bin/sh\necho \"{name} $*\"\n")).unwrap();
        fs::set_permissions(&p, fs::Permissions::from_mode(0o755)).unwrap();
    };
    let (mysql, mariadb) = (tmp.0.join("my"), tmp.0.join("ma"));
    fake(&mariadb, "mariadb");
    let cnf = tmp.0.join("mysql.cnf");
    let launcher = kit.kit_files().join("bin").join("mysql");
    let run = || {
        let out = Command::new(&launcher)
            .args(["-e", "SELECT 1"])
            .env("CP_MYSQL", &mysql)
            .env("CP_MARIADB", &mariadb)
            .env("DEVKIT_MYSQL_CNF", &cnf)
            .output()
            .unwrap();
        String::from_utf8_lossy(&out.stdout).trim().to_owned()
    };
    assert_eq!(run(), "mariadb -e SELECT 1", "no MySQL: MariaDB's");
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
    assert!(is_shell("bashrc") && is_shell("bin/agy"));
    assert!(!is_shell("bin/agy.cmd") && !is_shell("profile.ps1"));
    assert!(!is_unix_bin("bashrc") && is_unix_bin("bin/gradle"));
}

/// The shell files the kit writes: no CR (sh reads it as part of the word),
/// launchers executable on Unix, batch files in CRLF.
#[test]
fn shell_files_lf_and_executable() {
    let tmp = TempDir::new("kitfiles-sh");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
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
    for name in ["bin/agy", "bin/gradle"] {
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
        let agy = dir.join("bin/agy");
        fs::set_permissions(&agy, fs::Permissions::from_mode(0o644)).unwrap();
        write_all(&kit).unwrap();
        assert_eq!(
            fs::metadata(&agy).unwrap().permissions().mode() & 0o777,
            0o755
        );
    }
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
        "DEVKIT_TERMINAL",
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
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
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
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
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
                 type __devkit_p >/dev/null 2>&1 && echo leaked; true";
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
