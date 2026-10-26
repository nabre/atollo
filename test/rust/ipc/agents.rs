use super::*;

/// D-179 step 7 (R13): Antigravity from the project's page is admitted
/// like Pi, in the project's own VM with its build for Linux, never on the
/// PC; an id Atollo does not know is an error.
#[test]
fn antigravity_starts_on_a_project_in_its_vm() {
    use crate::levels::{Agent, Level, Placement, Sandbox, VmProgram};
    let (_tmp, kit) = crate::test_support::prepared_kit("agents-agy");
    let dir = kit.projects().join("shop");
    std::fs::create_dir_all(&dir).unwrap();
    let req = agent_request(&kit, "antigravity", &dir).unwrap();
    let facts = crate::levels::Facts {
        vm: crate::levels::VmState::On,
        guest_dir: crate::levels::facts::guest_dir(&kit, &req.place),
        restart_pending: false,
        linux_build: true,
        guest_tools: Vec::new(),
    };
    match super::super::levels::admitted(&req, &facts, "it").unwrap() {
        Placement::Vm {
            level,
            sandbox,
            program,
            ..
        } => {
            assert_eq!(level, Level::Agent);
            assert_eq!(
                sandbox,
                Sandbox::Project {
                    name: crate::isolation::vm::name("shop")
                }
            );
            assert_eq!(
                program,
                VmProgram::AgentBinary {
                    agent: Agent::Antigravity
                }
            );
        }
        other => panic!("{other:?}"),
    }
    assert!(agent_request(&kit, "gemini", &dir).is_err());
}

/// D-178, R1 of D-179: without the VM no agent starts from the project's
/// page: the error says the VM is needed, then why it is off and what to
/// do.
#[test]
fn no_agent_starts_without_the_vm() {
    let (_tmp, kit) = crate::test_support::prepared_kit("agents-no-vm");
    let dir = kit.projects().join("shop");
    std::fs::create_dir_all(&dir).unwrap();
    let req = agent_request(&kit, "claude", &dir).unwrap();
    let facts = crate::levels::Facts {
        vm: crate::levels::VmState::Off {
            blockers: vec![
                crate::isolation::Blocker::KvmDenied,
                crate::isolation::Blocker::SbxMissing { failed: None },
            ],
        },
        guest_dir: crate::levels::facts::guest_dir(&kit, &req.place),
        restart_pending: false,
        linux_build: false,
        guest_tools: Vec::new(),
    };
    let text = super::super::levels::admitted(&req, &facts, "it")
        .unwrap_err()
        .to_string();
    assert!(
        text.starts_with("Gli agenti partono solo nella VM"),
        "{text}"
    );
    assert!(text.contains(crate::isolation::KVM_COMMAND), "{text}");
    assert!(text.contains("Installa nel kit"), "{text}");
}
