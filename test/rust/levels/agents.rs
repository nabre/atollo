use super::*;
use crate::isolation::Blocker;
use crate::levels::VmState;

fn off() -> VmState {
    VmState::Off {
        blockers: vec![Blocker::HypervisorOff, Blocker::SbxSignedOut],
    }
}

fn component(id: &str) -> &'static Component {
    Catalog::get().find(id).unwrap()
}

/// LEVELS.md §7: the agents are enabled exactly while the VM is on.
#[test]
fn agents_are_enabled_only_with_the_vm() {
    assert!(agents_enabled(&VmState::On));
    assert!(!agents_enabled(&off()));
    assert!(!agents_enabled(&VmState::Off {
        blockers: Vec::new()
    }));
    let status = crate::isolation::Status::of(vec![Blocker::KvmMissing], None);
    assert_eq!(
        vm_state(&status),
        VmState::Off {
            blockers: vec![Blocker::KvmMissing]
        }
    );
    assert_eq!(
        vm_state(&crate::isolation::Status::of(Vec::new(), None)),
        VmState::On
    );
}

/// Q11 on every system and VM state: every agent only for the VM's Linux
/// while the VM is on (D-184); any other component as before.
#[test]
fn installable_on_every_target_and_vm_state() {
    for target in Target::ALL {
        for vm in [VmState::On, off()] {
            let on = agents_enabled(&vm);
            for id in ["claude", "codex", "pi", "antigravity"] {
                assert_eq!(
                    installable(component(id), target, &vm),
                    on && target.is_linux(),
                    "{id} {target} {vm:?}"
                );
                assert_eq!(obsolete(component(id), target), !target.is_linux());
            }
            for id in ["node", "git", "python"] {
                assert!(installable(component(id), target, &vm), "{id}");
                assert!(!obsolete(component(id), target), "{id}");
            }
        }
    }
    assert_eq!(agent_of(component("pi")), Some(Agent::Pi));
    assert_eq!(agent_of(component("node")), None);
    assert_eq!(catalog_agents().count(), Agent::ALL.len());
}

/// The rows of the Tools page: «serve la VM» with the blockers' text for
/// every agent without it; nothing with it, nothing for the others.
#[test]
fn the_rows_of_the_agents() {
    for vm in [VmState::On, off()] {
        assert_eq!(agent_row(component("node"), &vm, "it"), None);
    }
    for id in ["claude", "codex", "pi", "antigravity"] {
        assert_eq!(agent_row(component(id), &VmState::On, "it"), None, "{id}");
        assert!(
            matches!(
                agent_row(component(id), &off(), "it"),
                Some(AgentRow::NeedsVm { .. })
            ),
            "{id}"
        );
    }
    let Some(AgentRow::NeedsVm { reason }) = agent_row(component("antigravity"), &off(), "it")
    else {
        panic!("not a row of the VM")
    };
    assert!(
        reason.starts_with("Gli agenti girano solo nella VM"),
        "{reason}"
    );
    assert!(reason.contains("Windows Hypervisor Platform"), "{reason}");
    assert!(reason.contains("sbx login"), "{reason}");
    let en = off_reason(&off(), "en").unwrap();
    assert!(!en.contains("Gli agenti"), "{en}");
    assert_eq!(off_reason(&VmState::On, "it"), None);
    assert!(off_reason(&VmState::Off { blockers: vec![] }, "it").is_some());
    let row = serde_json::to_value(AgentRow::NeedsVm { reason: "x".into() }).unwrap();
    assert_eq!(row, serde_json::json!({"kind": "needsVm", "reason": "x"}));
}

/// An install asked anyway says why not.
#[test]
fn a_refused_install_says_why() {
    let claude = refused_install(component("claude"), &VmState::On, "it");
    assert!(claude.contains("Linux"), "{claude}");
    let pi = refused_install(component("pi"), &off(), "it");
    assert!(pi.contains("Windows Hypervisor Platform"), "{pi}");
    let pi = refused_install(component("pi"), &VmState::On, "it");
    assert!(pi.contains("Linux"), "{pi}");
}

/// `levels-changed` (LEVELS.md §4.1, §4.2).
#[test]
fn the_levels_state_as_the_interface_reads_it() {
    let on = LevelsState::of(&VmState::On, "it");
    assert_eq!(
        serde_json::to_value(&on).unwrap(),
        serde_json::json!({"vm": "on", "reason": null, "agentsEnabled": true})
    );
    let off = LevelsState::of(&off(), "it");
    assert_eq!(off.vm, "off");
    assert!(!off.agents_enabled && off.reason.is_some());
}

/// Re-enabling: only a flip of the availability is told (the first look
/// too), not every status.
#[test]
fn only_a_flip_is_told() {
    let told = Told::default();
    assert!(told.flipped(&off()), "the first look");
    assert!(!told.flipped(&off()));
    assert!(
        !told.flipped(&VmState::Off { blockers: vec![] }),
        "other reasons, still off"
    );
    assert!(told.flipped(&VmState::On));
    assert!(!told.flipped(&VmState::On));
    assert!(told.flipped(&off()));
}
