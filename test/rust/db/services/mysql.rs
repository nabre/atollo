use super::*;
use crate::kit::Target;

#[test]
fn initialization_with_the_same_case_rule_everywhere() {
    let args = init_args(Path::new("base"), Path::new("data"), Path::new("init.sql"));
    assert_eq!(
        args[0], "--no-defaults",
        "first, or the PC's files are read"
    );
    // root never without a password: the init file sets the kit's.
    assert!(args.contains(&"--initialize".to_owned()));
    assert!(!args.iter().any(|a| a.contains("insecure")), "{args:?}");
    assert!(args.contains(&"--init-file=init.sql".to_owned()));
    assert!(args.contains(&"--lower_case_table_names=1".to_owned()));
    assert!(args.contains(&"--basedir=base".to_owned()));
    assert!(args.contains(&"--datadir=data".to_owned()));
}

/// Paths in option files: `/` on Windows, quoted, Unix escapes kept.
#[test]
fn paths_in_option_files() {
    assert_eq!(
        cnf_path(Os::Win32, Path::new(r"E:\Dev Kit\data\db\mysql\8.4")),
        r#""E:/Dev Kit/data/db/mysql/8.4""#
    );
    assert_eq!(
        cnf_path(Os::Linux, Path::new("/media/u/kit #1/data")),
        r#""/media/u/kit #1/data""#
    );
    assert_eq!(
        cnf_path(Os::Darwin, Path::new(r#"/Users/a"b\c"#)),
        r#""/Users/a\"b\\c""#
    );
}

#[test]
fn server_cnf_on_every_system() {
    for t in Target::ALL {
        let (base, data) = match t.os {
            Os::Win32 => (
                r"E:\kit\data\apps\win32-x64\mysql\8.4",
                r"E:\kit\data\db\mysql\8.4",
            ),
            _ => ("/kit/data/apps/x/mysql/8.4", "/kit/data/db/mysql/8.4"),
        };
        let (base, data) = (Path::new(base), Path::new(data));
        let socket = socket_path(t.os, data, 3306, None).unwrap();
        let text = server_cnf(t.os, base, data, 3306, socket.as_deref());
        assert!(text.contains("[mysqld]\n"), "{}", t.as_str());
        for line in [
            "port=3306\n",
            "bind-address=127.0.0.1\n",
            "mysqlx=OFF\n",
            "skip-log-bin\n",
            "lower_case_table_names=1\n",
        ] {
            assert!(text.contains(line), "{}: {line}", t.as_str());
        }
        assert!(!text.contains('\r'), "LF only");
        // The files inside the data folder, as this host joins them.
        let inside = |name: &str| cnf_path(t.os, &data.join(name));
        assert!(
            text.contains(&format!("pid-file={}\n", inside("mysqld.pid"))),
            "{text}"
        );
        match t.os {
            Os::Win32 => {
                assert!(!text.contains('\\'), "no backslash on Windows: {text}");
                assert!(socket.is_none());
                assert!(!text.contains("socket="));
                assert!(text.contains(r#"datadir="E:/kit/data/db/mysql/8.4""#));
                assert!(text.contains(r#"pid-file="E:/kit/data/db/mysql/8.4/mysqld.pid""#));
            }
            Os::Darwin | Os::Linux => {
                assert!(text.contains(r#"datadir="/kit/data/db/mysql/8.4""#));
                assert!(text.contains(&format!("socket={}\n", inside("mysql.sock"))));
            }
        }
    }
}

/// A socket path that does not fit `sun_path` goes to the kit's private
/// folder of the PC's temporary one, never `/tmp` itself; without one that
/// fits, an error.
#[test]
fn long_socket_paths_move() {
    let deep = Path::new("/media/someone/a-very-long-volume-name/Dev-Kit/data/db/mysql/9.7")
        .join("x".repeat(60));
    let own = socket_dir(Some(Path::new("/tmp/Dev-Kit/k1"))).unwrap();
    assert_eq!(own, Path::new("/tmp/Dev-Kit/k1").join("mysql"));
    assert_eq!(socket_dir(None), None);
    assert_eq!(
        socket_path(Os::Linux, &deep, 3307, Some(&own))
            .unwrap()
            .unwrap(),
        own.join("devkit-mysql-3307.sock")
    );
    assert!(socket_path(Os::Linux, &deep, 3307, None).is_err());
    assert!(socket_path(Os::Linux, &deep, 3307, Some(&deep)).is_err());
    let short = Path::new("/k/data/db/mysql/9.7");
    assert_eq!(
        socket_path(Os::Darwin, short, 3307, Some(&own))
            .unwrap()
            .unwrap(),
        short.join("mysql.sock")
    );
    assert_eq!(socket_path(Os::Win32, &deep, 3307, None).unwrap(), None);
}

/// The client's own commands are off on a restore (`\!`, `system`,
/// `source`): MySQL `--commands=OFF`; a client
/// without the option gets no file (None).
#[test]
fn restores_without_client_commands() {
    use crate::db::DbEngine;
    let mysql_help =
        "  --commands          Enable or disable processing of local mysql commands.\n  \
                      --system-command    Enable or disable (by default) the 'system' mysql\n";
    let other_help = "  --sandbox           Disallow commands that access the file system (except \\P without an argument and \\e).\n";
    assert_eq!(
        restore_guard(DbEngine::MySql, mysql_help),
        Some("--commands=OFF")
    );
    assert_eq!(restore_guard(DbEngine::MySql, other_help), None);
    assert_eq!(restore_guard(DbEngine::MySql, ""), None, "an old client");
    assert_eq!(restore_guard(DbEngine::MongoDb, mysql_help), None);
}

/// Ubuntu 24.04: `libaio.so.1t64` only, linked by the kit; older systems:
/// `libaio.so.1`; libnuma missing is told.
#[test]
fn linux_server_libraries() {
    let have =
        |files: &'static [&'static str]| move |p: &Path| files.iter().any(|f| Path::new(f) == p);
    let noble = linux_server_libs(
        Arch::X64,
        have(&[
            "/usr/lib/x86_64-linux-gnu/libaio.so.1t64",
            "/usr/lib/x86_64-linux-gnu/libnuma.so.1",
        ]),
    );
    assert_eq!(
        noble,
        ServerLibs {
            link: Some(PathBuf::from("/usr/lib/x86_64-linux-gnu/libaio.so.1t64")),
            missing: vec![],
        }
    );
    let jammy = linux_server_libs(
        Arch::X64,
        have(&[
            "/lib/x86_64-linux-gnu/libaio.so.1",
            "/lib/x86_64-linux-gnu/libnuma.so.1",
        ]),
    );
    assert_eq!(jammy, ServerLibs::default());
    let fedora = linux_server_libs(Arch::Arm64, have(&["/usr/lib64/libaio.so.1"]));
    assert_eq!(fedora.missing, ["libnuma"]);
    let arm = linux_server_libs(
        Arch::Arm64,
        have(&[
            "/usr/lib/aarch64-linux-gnu/libaio.so.1t64",
            "/usr/lib/aarch64-linux-gnu/libnuma.so.1",
        ]),
    );
    assert_eq!(
        arm.link.as_deref(),
        Some(Path::new("/usr/lib/aarch64-linux-gnu/libaio.so.1t64"))
    );
    // x64 folders are never taken for ARM64
    let wrong = linux_server_libs(
        Arch::Arm64,
        have(&[
            "/usr/lib/x86_64-linux-gnu/libaio.so.1t64",
            "/usr/lib/x86_64-linux-gnu/libnuma.so.1",
        ]),
    );
    assert_eq!(wrong.missing, ["libaio", "libnuma"]);
    let text = SystemLibsMissing(wrong.missing).to_string();
    assert!(text.contains("libaio and libnuma"), "{text}");
    assert!(
        text.contains("sudo apt install libaio1t64 libnuma1"),
        "{text}"
    );
}

#[test]
fn nothing_to_do_off_linux() {
    let dir = Path::new("nowhere");
    ensure_server_libs(Os::Win32, Arch::X64, dir).unwrap();
    ensure_server_libs(Os::Darwin, Arch::Arm64, dir).unwrap();
}
