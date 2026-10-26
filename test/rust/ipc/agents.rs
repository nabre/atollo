use super::*;

/// D-178: on a project Claude Code, Codex and Pi start (in the VM);
/// Antigravity, a desktop editor, never does, with a message that says
/// why; an id without a way into the VM is refused, never run on the PC.
#[test]
fn antigravity_never_starts_on_a_project() {
    for agent in ["claude", "codex", "pi"] {
        assert_eq!(refused_on_project(agent, "it"), None, "{agent}");
    }
    let why = refused_on_project("antigravity", "it").unwrap();
    assert!(why.contains("microVM") && why.contains("Pi"), "{why}");
    assert_ne!(refused_on_project("antigravity", "en").unwrap(), why);
    assert!(refused_on_project("gemini", "it").is_some());
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
