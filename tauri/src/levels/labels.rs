//! What the user reads about a placement (LEVELS.md §8.3): the label of a
//! terminal, the banner line before a VM session, and the text of a
//! refusal with what to do.

use serde::Serialize;

use crate::common::i18n::t;

use super::{Level, Placement, Refusal};

/// The label of every terminal (`PtyInfo.label`, the UI's `LevelLabel`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LevelLabel {
    Pc,
    ProjectVm,
    AgentVm,
}

/// The label of `placement`; `None` for a refusal (nothing starts).
pub fn label_of(placement: &Placement) -> Option<LevelLabel> {
    match placement {
        Placement::Pc { .. } => Some(LevelLabel::Pc),
        Placement::Vm {
            level: Level::Agent,
            ..
        } => Some(LevelLabel::AgentVm),
        Placement::Vm { .. } => Some(LevelLabel::ProjectVm),
        Placement::Refused { .. } => None,
    }
}

/// The text of the label of `placement` in `lang` (§8.3): «PC» for level 1,
/// «PC · senza VM» for level 2 on the PC, «VM progetto», «Agente · VM».
pub fn label(placement: &Placement, lang: &str) -> Option<String> {
    Some(match (label_of(placement)?, placement.level()) {
        (LevelLabel::Pc, Level::Manual) => t(lang, "PC", &[]),
        (LevelLabel::Pc, _) => t(lang, "PC · senza VM", &[]),
        (LevelLabel::ProjectVm, _) => t(lang, "VM progetto", &[]),
        (LevelLabel::AgentVm, _) => t(lang, "Agente · VM", &[]),
    })
}

/// The line printed on stderr before a VM session starts (by the PC side,
/// so it does not depend on the VM's shell); `None` on the PC, whose shells
/// print their own banner (profile.ps1, bashrc), and for a refusal.
pub fn banner_line(placement: &Placement, lang: &str) -> Option<String> {
    let Placement::Vm {
        level,
        sandbox,
        workdir,
        program,
        ..
    } = placement
    else {
        return None;
    };
    Some(match (level, program) {
        (
            Level::Agent,
            super::VmProgram::SbxAgent { agent } | super::VmProgram::AgentBinary { agent },
        ) => t(
            lang,
            "[Atollo] Agente · VM: {0} in {1}",
            &[agent.id(), sandbox.name()],
        ),
        _ => t(
            lang,
            "[Atollo] VM progetto: {0} · {1}",
            &[sandbox.name(), workdir],
        ),
    })
}

/// Why `reason` refused a request and what to do, in `lang` (the reply's
/// `text`, the error of a refused command).
pub fn refusal_text(reason: &Refusal, lang: &str) -> String {
    match reason {
        Refusal::AgentsNeedVm { blockers } => format!(
            "{}\n{}",
            t(
                lang,
                "Gli agenti partono solo nella VM del progetto, e la VM non è attiva.",
                &[]
            ),
            crate::isolation::why_no_vm(
                &crate::isolation::Status::of(blockers.clone(), None),
                lang
            )
        ),
        Refusal::AgentOutsideProject => t(
            lang,
            "Gli agenti partono solo nella cartella di un progetto: apri il terminale nella cartella del progetto, o avvia l'agente dalla pagina del progetto.",
            &[],
        ),
        Refusal::NotAProject => t(
            lang,
            "Con la VM attiva il codice di un progetto parte solo nella cartella di un progetto: aprila dalla pagina del progetto.",
            &[],
        ),
        Refusal::NoPathInVm { dir } => t(
            lang,
            "La cartella {0} non ha un percorso nella VM (è su una condivisione di rete): sposta il progetto su un disco locale.",
            &[dir],
        ),
        Refusal::PoolsOnlyWithoutVm => t(
            lang,
            "Con la VM attiva ogni sito gira nella VM del suo progetto: avvialo dalla pagina del progetto. I server condivisi servono quando si lavora senza VM.",
            &[],
        ),
        Refusal::NoLinuxBuild { agent } => t(
            lang,
            "{0} non ha una versione per il Linux della VM: per ora non può partire.",
            &[agent.name()],
        ),
        Refusal::DaemonRestartPending => t(
            lang,
            "Docker Sandboxes deve ripartire per un'impostazione di sicurezza: chiudi le sessioni nella VM, poi «Ricontrolla» nella finestra della VM e riprova.",
            &[],
        ),
    }
}

#[cfg(test)]
#[path = "../../../test/rust/levels/labels.rs"]
mod tests;
