use super::*;
use crate::levels::{Agent, Fix, HostAccess, Mounts, Sandbox, VmProgram};
use std::path::PathBuf;

fn pc(level: Level) -> Placement {
    Placement::Pc {
        level,
        project: None,
        dir: PathBuf::from("x"),
    }
}

fn vm(level: Level, sandbox: Sandbox, program: VmProgram) -> Placement {
    Placement::Vm {
        level,
        project: "shop".into(),
        sandbox,
        workdir: "/d/kit/projects/shop/web".into(),
        mounts: Mounts {
            workspace_rw: PathBuf::from("x"),
            tools_ro: vec![],
        },
        host: HostAccess {
            databases: true,
            local_models: false,
        },
        program,
    }
}

fn project_sandbox() -> Sandbox {
    Sandbox::Project {
        name: "atollo-p-shop-x".into(),
    }
}

/// The four labels of §8.3, in Italian and English; none for a refusal.
#[test]
fn every_placement_has_the_label_of_its_level() {
    let pi = vm(
        Level::Agent,
        project_sandbox(),
        VmProgram::AgentBinary { agent: Agent::Pi },
    );
    let refused = Placement::Refused {
        level: Level::Agent,
        reason: Refusal::AgentOutsideProject,
        fix: Fix::OpenFromProject,
    };
    let cases = [
        (pc(Level::Manual), Some(LevelLabel::Pc), "PC", "PC"),
        (
            pc(Level::Project),
            Some(LevelLabel::Pc),
            "PC · senza VM",
            "PC · no VM",
        ),
        (pi, Some(LevelLabel::AgentVm), "Agente · VM", "Agent · VM"),
    ];
    for (p, key, it, en) in cases {
        assert_eq!(label_of(&p), key, "{p:?}");
        assert_eq!(label(&p, "it").as_deref(), Some(it));
        assert_eq!(label(&p, "en").as_deref(), Some(en));
    }
    assert_eq!(label_of(&refused), None);
    assert_eq!(label(&refused, "it"), None);
    assert_eq!(
        serde_json::to_value(LevelLabel::ProjectVm).unwrap(),
        "projectVm"
    );
}

/// The line before a VM session: the agent and its sandbox; nothing on
/// the PC.
#[test]
fn the_banner_names_the_sandbox() {
    let pi = vm(
        Level::Agent,
        project_sandbox(),
        VmProgram::AgentBinary { agent: Agent::Pi },
    );
    assert_eq!(
        banner_line(&pi, "it").as_deref(),
        Some("[Atollo] Agente · VM: pi in atollo-p-shop-x")
    );
    let claude = vm(
        Level::Agent,
        Sandbox::Agent {
            name: "atollo-a-claude-shop-x".into(),
            agent: Agent::Claude,
        },
        VmProgram::SbxAgent {
            agent: Agent::Claude,
        },
    );
    assert_eq!(
        banner_line(&claude, "en").as_deref(),
        Some("[Atollo] Agent · VM: claude in atollo-a-claude-shop-x")
    );
    assert_eq!(banner_line(&pc(Level::Manual), "it"), None);
}

/// A refusal says why and what to do; without the VM, its reasons too.
#[test]
fn a_refusal_says_why_and_what_to_do() {
    let text = refusal_text(
        &Refusal::AgentsNeedVm {
            blockers: vec![crate::isolation::Blocker::KvmMissing],
        },
        "en",
    );
    assert!(
        text.starts_with("Agents start only in the project's VM"),
        "{text}"
    );
    assert!(text.contains("/dev/kvm"), "{text}");
    let share = refusal_text(
        &Refusal::NoPathInVm {
            dir: r"\\nas\p\shop".into(),
        },
        "it",
    );
    assert!(share.contains(r"\\nas\p\shop"), "{share}");
    let pi = refusal_text(&Refusal::NoLinuxBuild { agent: Agent::Pi }, "en");
    assert!(pi.starts_with("Pi has no build"), "{pi}");
}

/// A session's label (`PtyInfo`): the placement's level and label, the
/// sandbox in the VM only, and the text of §8.3.
#[test]
fn a_session_label_names_its_sandbox() {
    let pi = vm(
        Level::Agent,
        project_sandbox(),
        VmProgram::AgentBinary { agent: Agent::Pi },
    );
    let at = SessionLabel::of(&pi).unwrap();
    assert_eq!(at, SessionLabel::vm(Level::Agent, "atollo-p-shop-x"));
    assert_eq!(at.label, LevelLabel::AgentVm);
    assert_eq!(at.text("it"), "Agente · VM");
    let at = SessionLabel::of(&pc(Level::Project)).unwrap();
    assert_eq!(at, SessionLabel::pc(Level::Project));
    assert_eq!(at.sandbox, None);
    assert_eq!(at.text("en"), "PC · no VM");
    assert_eq!(SessionLabel::pc(Level::Manual).text("it"), "PC");
    let claude = vm(
        Level::Agent,
        Sandbox::Agent {
            name: "atollo-a-claude-shop-x".into(),
            agent: Agent::Claude,
        },
        VmProgram::SbxAgent {
            agent: Agent::Claude,
        },
    );
    assert_eq!(
        SessionLabel::of(&claude),
        Some(SessionLabel::vm(Level::Agent, "atollo-a-claude-shop-x"))
    );
    let refused = Placement::Refused {
        level: Level::Agent,
        reason: Refusal::AgentOutsideProject,
        fix: Fix::OpenFromProject,
    };
    assert_eq!(SessionLabel::of(&refused), None);
}
