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
    Some(label_text(label_of(placement)?, placement.level(), lang))
}

/// The text of `label` for a session of `level` in `lang` (§8.3).
pub fn label_text(label: LevelLabel, level: Level, lang: &str) -> String {
    match (label, level) {
        (LevelLabel::Pc, Level::Project) => t(lang, "PC · senza VM", &[]),
        (LevelLabel::Pc, _) => t(lang, "PC", &[]),
        (LevelLabel::ProjectVm, _) => t(lang, "VM progetto", &[]),
        (LevelLabel::AgentVm, _) => t(lang, "Agente · VM", &[]),
    }
}

/// Where a terminal session runs, as its `PtyInfo` says it (LEVELS.md
/// §4.1, §8.3): the level, the label, and the sandbox in the VM.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionLabel {
    pub level: Level,
    pub label: LevelLabel,
    /// The sandbox's name (`atollo-p-…`, `atollo-a-…`); `None` on the PC.
    pub sandbox: Option<String>,
}

impl SessionLabel {
    /// The label of a session started for `placement`; `None` for a
    /// refusal (nothing starts).
    pub fn of(placement: &Placement) -> Option<SessionLabel> {
        let sandbox = match placement {
            Placement::Vm { sandbox, .. } => Some(sandbox.name().to_owned()),
            _ => None,
        };
        Some(SessionLabel {
            level: placement.level(),
            label: label_of(placement)?,
            sandbox,
        })
    }

    /// A session on the PC at `level` (Manual, or Project without the VM).
    pub fn pc(level: Level) -> SessionLabel {
        SessionLabel {
            level,
            label: LevelLabel::Pc,
            sandbox: None,
        }
    }

    /// A session in the sandbox `name` at `level` (Project or Agent).
    #[cfg(test)]
    pub fn vm(level: Level, name: &str) -> SessionLabel {
        SessionLabel {
            level,
            label: if level == Level::Agent {
                LevelLabel::AgentVm
            } else {
                LevelLabel::ProjectVm
            },
            sandbox: Some(name.to_owned()),
        }
    }

    /// The label's text in `lang`.
    #[cfg(test)]
    pub fn text(&self, lang: &str) -> String {
        label_text(self.label, self.level, lang)
    }
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
        Refusal::NoPathInVm { dir } => t(
            lang,
            "La cartella {0} non ha un percorso nella VM (è su una condivisione di rete): sposta il progetto su un disco locale.",
            &[dir],
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
