//! What MySQL needs that MariaDB does not (D-110), as functions of the
//! target and the folders, so the tests cover every system on Windows:
//!
//! - a new data folder from `mysqld --initialize-insecure` (no
//!   `mariadb-install-db`), always with `lower_case_table_names=1`: MySQL
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
/// server in `basedir` (with `root@localhost` and no password: the kit sets
/// it right after, `access::secure`). Output on stderr (`--console`).
pub fn init_args(basedir: &Path, data: &Path) -> Vec<String> {
    vec![
        // It must be the first option.
        "--no-defaults".to_owned(),
        "--initialize-insecure".to_owned(),
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
/// Linux) or the server does not start: deep kits get one in `fallback`
/// (the PC's temporary folder), named by the port.
const SOCKET_MAX: usize = 100;

/// The socket of the server of `data` on `port`: none on Windows (TCP
/// only), in the data folder when its path is short enough.
pub fn socket_path(os: Os, data: &Path, port: u16, fallback: &Path) -> Option<PathBuf> {
    if os == Os::Win32 {
        return None;
    }
    let inside = data.join("mysql.sock");
    if inside.as_os_str().len() <= SOCKET_MAX {
        Some(inside)
    } else {
        Some(fallback.join(format!("devkit-mysql-{port}.sock")))
    }
}

/// The kit's `my.cnf` of the server: the folders, its port on 127.0.0.1
/// only, no X Protocol (its port 33060 would be taken by every version),
/// `lower_case_table_names=1` as at the initialization.
pub fn server_cnf(os: Os, basedir: &Path, data: &Path, port: u16, socket: Option<&Path>) -> String {
    let mut text = format!(
        "# Dev-Kit: options of the kit's MySQL (written by the app at every start).\n\
         [mysqld]\n\
         basedir={}\n\
         datadir={}\n\
         port={port}\n\
         bind-address=127.0.0.1\n\
         mysqlx=OFF\n\
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

/// The files a MySQL build carries that the kit never runs (T3.7.4): the
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

#[cfg(test)]
#[path = "../../../../test/rust/db/services/mysql.rs"]
mod tests;
