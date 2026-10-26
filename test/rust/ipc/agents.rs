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

/// D-178: without the VM no agent starts: the message says the VM is
/// needed, then why it is off and what to do.
#[test]
fn no_agent_starts_without_the_vm() {
    let status = crate::isolation::Status::of(
        vec![
            crate::isolation::Blocker::KvmDenied,
            crate::isolation::Blocker::SbxMissing { failed: None },
        ],
        None,
    );
    let text = no_vm_for_agents(&status, "it");
    assert!(
        text.starts_with("Gli agenti partono solo nella VM"),
        "{text}"
    );
    assert!(text.contains(crate::isolation::KVM_COMMAND), "{text}");
    assert!(text.contains("Installa nel kit"), "{text}");
}
