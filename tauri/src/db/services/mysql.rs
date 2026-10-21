//! What the kit's MySQL needs (D-110), as functions of the
//! target and the folders, so the tests cover every system on Windows:
//!
//! - a new data folder from `mysqld --initialize --init-file`, always with `lower_case_table_names=1`: MySQL
//!   refuses another value after the initialization, and with the same one
//!   everywhere the folder opens on Windows, macOS and Linux alike;
//! - `my.cnf` in the data folder, written by the kit at every start and
//!   read alone (`--defaults-file`): no option file of the PC
//!   (`C:\my.ini`, `/etc/mysql/my.cnf`) reaches the kit's server;
//! - on Linux the server needs `libaio.so.1` and `libnuma.so.1` from the
//!   system. Ubuntu 24.04 and Debian 13 renamed libaio's to `libaio.so.1t64`
//!   (package `libaio1t64`): the kit links `lib/private/libaio.so.1` (in the
//!   server's RUNPATH) to it, no root needed. libnuma cannot come from the
//!   kit: a missing one is told with the command that installs it.

use std::path::{Path, PathBuf};

use crate::kit::{Arch, Os};

/// The arguments of `mysqld` that create the data folder `data` of the
/// server in `basedir`. `--initialize` (never `--initialize-insecure`):
/// `root` is never without a password, not even for a moment, and
/// `init_file` (a private file of the kit, removed after) gives it the
/// kit's own, on `localhost` and `127.0.0.1` ([`auth::mysql_root_sql`],
/// run by the bootstrap before any connection is possible). Output on
/// stderr (`--console`).
///
/// [`auth::mysql_root_sql`]: crate::db::auth::mysql_root_sql
pub fn init_args(basedir: &Path, data: &Path, init_file: &Path) -> Vec<String> {
    vec![
        // It must be the first option.
        "--no-defaults".to_owned(),
        "--initialize".to_owned(),
        format!("--init-file={}", init_file.display()),
        format!("--basedir={}", basedir.display()),
        format!("--datadir={}", data.display()),
        "--lower_case_table_names=1".to_owned(),
        "--console".to_owned(),
    ]
}

/// A path as a value of MySQL's option files: `/` on Windows too (a `\`
/// starts an escape there), quoted (a `#` would start a comment), with `\`
/// and `"` of a Unix name escaped.
pub fn cnf_path(os: Os, path: &Path) -> String {
    let text = path.to_string_lossy();
    let text = match os {
        Os::Win32 => text.replace('\\', "/"),
        Os::Darwin | Os::Linux => text.into_owned(),
    };
    format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""))
}

/// A Unix socket's path must fit in `sun_path` (104 bytes on macOS, 108 on
/// Linux) or the server does not start: deep kits get one in `fallback`,
/// the kit's own private folder (0700) in the PC's temporary one
/// (`<TEMP>/Atollo/<kit id>/mysql`, [`socket_dir`]), named by the port;
/// never a shared folder such as `/tmp` itself.
const SOCKET_MAX: usize = 100;

/// The kit's private folder of the sockets that do not fit in their data
/// folder: `mysql` in the kit's temporary folder (`Kit::system_temp`).
pub fn socket_dir(system_temp: Option<&Path>) -> Option<PathBuf> {
    system_temp.map(|t| t.join("mysql"))
}

/// The socket of the server of `data` on `port`: none on Windows (TCP
/// only), in the data folder when its path is short enough, otherwise in
/// `fallback` ([`socket_dir`]); an error when neither fits (the server
/// would not start, and no shared folder is used instead).
pub fn socket_path(
    os: Os,
    data: &Path,
    port: u16,
    fallback: Option<&Path>,
) -> anyhow::Result<Option<PathBuf>> {
    if os == Os::Win32 {
        return Ok(None);
    }
    let inside = data.join("mysql.sock");
    if inside.as_os_str().len() <= SOCKET_MAX {
        return Ok(Some(inside));
    }
    match fallback.map(|f| f.join(format!("atollo-mysql-{port}.sock"))) {
        Some(s) if s.as_os_str().len() <= SOCKET_MAX => Ok(Some(s)),
        _ => anyhow::bail!(
            "{}: the folder of the kit is too deep for MySQL's socket and the PC has \
             no short temporary folder: move the kit to a shorter path",
            data.display()
        ),
    }
}

/// The kit's `my.cnf` of the server: the folders, its port on 127.0.0.1
/// only, no X Protocol (its port 33060 would be taken by every version),
/// `lower_case_table_names=1` as at the initialization, no binary log
/// (`skip-log-bin`): on since 8.0, it would refuse
/// the triggers and functions of a project's own user without `SUPER`
/// (error 1419, seen on 9.7 with a Laravel-style trigger) and fill the
/// disk of a stick with logs a development server never replays.
pub fn server_cnf(os: Os, basedir: &Path, data: &Path, port: u16, socket: Option<&Path>) -> String {
    let mut text = format!(
        "# Atollo: options of the kit's MySQL (written by the app at every start).\n\
         [mysqld]\n\
         basedir={}\n\
         datadir={}\n\
         port={port}\n\
         bind-address=127.0.0.1\n\
         mysqlx=OFF\n\
         skip-log-bin\n\
         lower_case_table_names=1\n\
         pid-file={}\n",
        cnf_path(os, basedir),
        cnf_path(os, data),
        cnf_path(os, &data.join("mysqld.pid")),
    );
    if let Some(s) = socket {
        text.push_str(&format!("socket={}\n", cnf_path(os, s)));
    }
    text
}

/// The options that stop the SQL client from running its own commands
/// while it reads a file given to it (a copy restored, D-78): a dump from
/// elsewhere could hold `\! cmd`, `system`, `source`, `tee` or `pager` and
/// run programs or write files as the user. MySQL: `--commands=OFF` (in the
/// clients of 8.4 and 9.7, checked with `mysql --help`). `help` is the
/// client's `--help`; None: this client has no such option, and the file
/// is not given to it.
pub fn restore_guard(engine: crate::db::DbEngine, help: &str) -> Option<&'static str> {
    use crate::db::DbEngine;
    let has = |opt: &str| {
        help.lines()
            .any(|l| l.split_whitespace().next() == Some(opt))
    };
    match engine {
        DbEngine::MySql if has("--commands") => Some("--commands=OFF"),
        DbEngine::MySql | DbEngine::MongoDb => None,
    }
}

/// The folders where the dynamic loader finds the system's libraries for
/// `arch` (Debian/Ubuntu multiarch first, then Fedora/RHEL, Arch, others).
pub fn lib_dirs(arch: Arch) -> [&'static str; 6] {
    match arch {
        Arch::X64 => [
            "/lib/x86_64-linux-gnu",
            "/usr/lib/x86_64-linux-gnu",
            "/lib64",
            "/usr/lib64",
            "/usr/lib",
            "/lib",
        ],
        Arch::Arm64 => [
            "/lib/aarch64-linux-gnu",
            "/usr/lib/aarch64-linux-gnu",
            "/lib64",
            "/usr/lib64",
            "/usr/lib",
            "/lib",
        ],
    }
}

/// What the server of Linux `arch` needs from the system, as `exists` sees
/// it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ServerLibs {
    /// The `libaio.so.1t64` that `lib/private/libaio.so.1` must point to,
    /// when the system has only that name.
    pub link: Option<PathBuf>,
    /// Libraries the system lacks (`libaio`, `libnuma`).
    pub missing: Vec<&'static str>,
}

pub fn linux_server_libs(arch: Arch, exists: impl Fn(&Path) -> bool) -> ServerLibs {
    let find = |name: &str| {
        lib_dirs(arch)
            .iter()
            .map(|d| Path::new(d).join(name))
            .find(|p| exists(p))
    };
    let mut out = ServerLibs::default();
    if find("libaio.so.1").is_none() {
        match find("libaio.so.1t64") {
            Some(t64) => out.link = Some(t64),
            None => out.missing.push("libaio"),
        }
    }
    if find("libnuma.so.1").is_none() {
        out.missing.push("libnuma");
    }
    out
}

/// The system libraries MySQL needs on Linux are missing, with the commands
/// that install them (they need the administrator).
#[derive(Debug)]
pub struct SystemLibsMissing(pub Vec<&'static str>);

impl std::fmt::Display for SystemLibsMissing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MySQL needs the system libraries {} of this PC: install them with \
             `sudo apt install libaio1t64 libnuma1` (Ubuntu 24.04+, Debian 13; \
             `libaio1` on older releases) or `sudo dnf install libaio numactl-libs` \
             (Fedora, RHEL), then start it again",
            self.0.join(" and ")
        )
    }
}

impl std::error::Error for SystemLibsMissing {}

/// The files a MySQL build carries that the kit never runs (D-110): the
/// test suite, debug symbols (`.pdb`), the debug plugins and server (most of
/// the Linux ARM64 build's 1.8 GB), relative to the build's folder.
pub const STRIP_DIRS: [&str; 2] = ["mysql-test", "lib/plugin/debug"];
pub const STRIP_FILES: [&str; 2] = ["bin/mysqld-debug", "bin/mysqld-debug.exe"];

/// The `lib/private/libaio.so.1` of the server in `basedir` points to the
/// system's `libaio.so.1t64` when that is the only one, and is not there
/// otherwise (a kit moved to a PC with `libaio.so.1`); a missing library is
/// an error ([`SystemLibsMissing`]). Nothing off Linux.
#[cfg_attr(not(unix), allow(unused_variables))]
pub fn ensure_server_libs(os: Os, arch: Arch, basedir: &Path) -> anyhow::Result<()> {
    if os != Os::Linux {
        return Ok(());
    }
    let libs = linux_server_libs(arch, |p| p.exists());
    #[cfg(unix)]
    {
        let ours = basedir.join("lib").join("private").join("libaio.so.1");
        let is_link = std::fs::symlink_metadata(&ours).is_ok_and(|m| m.file_type().is_symlink());
        match &libs.link {
            Some(target) => {
                if std::fs::read_link(&ours).ok().as_deref() != Some(target.as_path()) {
                    if is_link || ours.is_file() {
                        std::fs::remove_file(&ours)?;
                    }
                    if let Some(parent) = ours.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    // A disk without links (exFAT): a copy of the library.
                    if std::os::unix::fs::symlink(target, &ours).is_err() {
                        std::fs::copy(target, &ours)?;
                    }
                }
            }
            None if is_link => std::fs::remove_file(&ours)?,
            None => {}
        }
    }
    if !libs.missing.is_empty() {
        return Err(SystemLibsMissing(libs.missing).into());
    }
    Ok(())
}

/// A line of a dump without its `DEFINER=<user>@<host>` clauses
/// (BUG-V6-01): the dump runs as `root`, the restore as the project's user
/// (D-78), which may not create a view, trigger, routine or event for
/// another user (`ERROR 1227`, the client stops halfway). Without the
/// clause each object is the restoring user's. `/*!50017 DEFINER=… */`
/// becomes an empty comment; the rows (`INSERT`) are never touched.
pub fn strip_definer(line: &[u8]) -> std::borrow::Cow<'_, [u8]> {
    const KEY: &[u8] = b"DEFINER=";
    if line.starts_with(b"INSERT ") || !line.windows(KEY.len()).any(|w| w == KEY) {
        return std::borrow::Cow::Borrowed(line);
    }
    let mut out = Vec::with_capacity(line.len());
    let mut i = 0;
    while i < line.len() {
        if line[i..].starts_with(KEY) {
            let user = i + KEY.len();
            if let Some(at) = account_part(line, user).filter(|&e| line.get(e) == Some(&b'@')) {
                if let Some(mut end) = account_part(line, at + 1) {
                    if line.get(end) == Some(&b' ') {
                        end += 1;
                    }
                    i = end;
                    continue;
                }
            }
        }
        out.push(line[i]);
        i += 1;
    }
    std::borrow::Cow::Owned(out)
}

/// The end of the user or host of an account at `from`: a name in
/// backticks (a doubled one inside), in single quotes (doubled or after a
/// backslash inside) or bare (letters, digits, `_ $ . % -`).
fn account_part(line: &[u8], from: usize) -> Option<usize> {
    match *line.get(from)? {
        q @ (b'`' | b'\'') => {
            let mut i = from + 1;
            while i < line.len() {
                if q == b'\'' && line[i] == b'\\' {
                    i += 2;
                } else if line[i] == q {
                    if line.get(i + 1) == Some(&q) {
                        i += 2;
                    } else {
                        return Some(i + 1);
                    }
                } else {
                    i += 1;
                }
            }
            None
        }
        _ => {
            let bare = |c: &u8| c.is_ascii_alphanumeric() || b"_$.%-".contains(c);
            let len = line[from..].iter().take_while(|c| bare(c)).count();
            (len > 0).then_some(from + len)
        }
    }
}

/// A dump read line by line through [`strip_definer`], as bytes: the
/// copy is never whole in memory nor read as UTF-8 text (BUG-V6-04).
pub struct WithoutDefiner<R> {
    input: R,
    line: Vec<u8>,
    at: usize,
}

impl<R: std::io::BufRead> WithoutDefiner<R> {
    pub fn new(input: R) -> Self {
        WithoutDefiner {
            input,
            line: Vec::new(),
            at: 0,
        }
    }
}

impl<R: std::io::BufRead> std::io::Read for WithoutDefiner<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.at == self.line.len() {
            let mut raw = Vec::new();
            if self.input.read_until(b'\n', &mut raw)? == 0 {
                return Ok(0);
            }
            self.line = match strip_definer(&raw) {
                std::borrow::Cow::Borrowed(_) => raw,
                std::borrow::Cow::Owned(line) => line,
            };
            self.at = 0;
        }
        let n = buf.len().min(self.line.len() - self.at);
        buf[..n].copy_from_slice(&self.line[self.at..self.at + n]);
        self.at += n;
        Ok(n)
    }
}

#[cfg(test)]
#[path = "../../../../test/rust/db/services/mysql.rs"]
mod tests;
