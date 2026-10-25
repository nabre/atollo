//! The pseudo terminal of the app's terminal on Windows (`platform::pty`):
//! the kit's ConPTY (`conpty.rs`).

use std::path::Path;

use anyhow::Result;

use crate::env::KitEnv;
use crate::platform::pty::{Program, Spawned};

/// Starts `launch` on a new pseudo console (`conpty.rs`) with the
/// kit environment only; without the kit's ConPTY the session says that
/// Windows' own is used.
pub fn spawn(launch: &Program, env: &KitEnv, dir: &Path, lang: &str) -> Result<Spawned> {
    let mut env = env.clone();
    env.set("COLORTERM", "truecolor");
    let spawned = super::conpty::spawn(launch.program, launch.args, &env, dir, launch.conpty)?;
    let note = spawned.system.then(|| {
        let text = crate::common::i18n::t(
            lang,
            "Il ConPTY del kit non è installato: questo terminale usa quello di Windows, e alcuni programmi (Claude Code) possono non ricevere i tasti.",
            &[],
        );
        format!("\x1b[2m{text}\x1b[0m\r\n")
    });
    let process = spawned.process;
    Ok(Spawned {
        io: Box::new(spawned.io),
        reader: Box::new(spawned.reader),
        wait: Box::new(move || process.wait()),
        note,
    })
}

/// A file: Windows runs any (by its extension).
pub fn runnable(program: &Path) -> bool {
    program.is_file()
}
