//! What many Rust tests share, mounted once by `main.rs` as
//! `crate::test_support`: a temporary folder, a prepared kit, a component
//! faked on disk, and the wait for a terminal session's output.

use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::editor::pty::Sessions;
use crate::kit::{Arch, Kit};
use crate::setup::Catalog;

/// A fresh folder under the system temp, removed on drop.
pub struct TempDir(pub PathBuf);
impl TempDir {
    pub fn new(name: &str) -> TempDir {
        let dir = std::env::temp_dir().join(format!(
            "atollo-test-{}-{}-{name}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        TempDir(dir)
    }
}
impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// `path` (a `PATH` of a shell or program on the PC) past the stand-ins of
/// the agents' commands that come first (D-178, `levels::env::forward_first`),
/// which must be there.
pub fn past_stand_ins(kit: &Kit, path: Option<&str>) -> String {
    let path = path.expect("a PATH");
    let first = crate::levels::env::forward_dir(kit).display().to_string();
    let rest = path
        .strip_prefix(&first)
        .unwrap_or_else(|| panic!("the stand-ins first: {path}"));
    rest.get(1..).unwrap_or_default().to_owned()
}

/// The processor of a kit for the system running the tests, one of the
/// supported targets whatever the runner's CPU (D-176): Apple silicon on
/// macOS, x64 elsewhere.
pub fn kit_arch() -> Arch {
    if crate::kit::Os::current().is_macos() {
        Arch::Arm64
    } else {
        Arch::X64
    }
}

/// A kit ([`kit_arch`]) prepared in a fresh folder named after `name`; the
/// folder goes when the returned [`TempDir`] is dropped.
pub fn prepared_kit(name: &str) -> (TempDir, Kit) {
    let tmp = TempDir::new(name);
    let kit = Kit::at(tmp.0.clone(), kit_arch());
    kit.prepare().unwrap();
    (tmp, kit)
}

/// `id` installed at `version`, the default folder of its line, with the
/// key paths of the kit's target (`caddy.exe` or `bin/mysqld.exe` on
/// Windows, `caddy` or `bin/mysqld` elsewhere).
pub fn fake_server(kit: &Kit, id: &str, version: &str) {
    let c = Catalog::get().find(id).unwrap();
    let dir = crate::versions::install_dir(kit, c, version);
    let folder = c.folder.as_deref().unwrap();
    let target = kit.target();
    for k in std::iter::once(c.key_for(target)).chain(c.keys_for(target)) {
        if let Some(rel) = k.strip_prefix(folder) {
            let file = crate::common::fsx::join_rel(&dir, rel);
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(&file, b"").unwrap();
        }
    }
    c.save_version(kit, version).unwrap();
}

/// The output of the terminal session `id` once it holds `text`, or as it
/// is after 20 seconds.
pub fn wait_for(sessions: &Sessions, id: u64, text: &str) -> String {
    let until = Instant::now() + Duration::from_secs(20);
    loop {
        let data = sessions.backlog(id).unwrap().data;
        if data.contains(text) || Instant::now() > until {
            return data;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}
