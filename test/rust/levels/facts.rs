use super::*;
use crate::kit::{Kit, Target};
use crate::levels::Agent;
use std::path::PathBuf;

fn kit(target: Target) -> Kit {
    Kit::at_target(std::env::temp_dir().join("atollo-levels-facts"), target)
}

/// The folder as the VM mounts it: the drive in lower case on Windows, the
/// same path elsewhere; a network share has none.
#[test]
fn the_guest_dir_is_the_projects_folder_in_the_vm() {
    let win = kit(Target::WIN32_X64);
    let place = Place::Project {
        name: "shop".into(),
        dir: PathBuf::from(r"D:\kit\projects\shop"),
        below: Some("web".into()),
    };
    assert_eq!(
        guest_dir(&win, &place).as_deref(),
        Some("/d/kit/projects/shop")
    );
    let share = Place::Folder {
        dir: PathBuf::from(r"\\server\share\shop"),
    };
    assert_eq!(guest_dir(&win, &share), None);
    let linux = kit(Target::LINUX_X64);
    let place = Place::Folder {
        dir: PathBuf::from("/home/u/kit/projects/shop"),
    };
    assert_eq!(
        guest_dir(&linux, &place).as_deref(),
        Some("/home/u/kit/projects/shop")
    );
}

/// The kit's tools for the VM's Linux, by the PC's processor, and the
/// global packages of npm (pnpm, yarn, D-180).
#[test]
fn the_guest_tools_are_the_kits_linux_ones() {
    for (host, guest) in [
        (Target::WIN32_X64, Target::LINUX_X64),
        (Target::LINUX_X64, Target::LINUX_X64),
        (Target::DARWIN_ARM64, Target::LINUX_ARM64),
        (Target::LINUX_ARM64, Target::LINUX_ARM64),
    ] {
        let k = kit(host);
        let linux = k.for_target(guest);
        assert_eq!(
            guest_tools(&k),
            vec![
                linux.apps(),
                linux.data().join("npm-global"),
                linux.data().join("php").join(guest.as_str())
            ],
            "{host:?}"
        );
    }
}

/// Asked for Pi and Antigravity only, from the catalog of the VM's Linux.
#[test]
fn a_linux_build_is_asked_for_project_vm_agents_only() {
    let k = kit(Target::WIN32_X64);
    let guest = Target::LINUX_X64;
    for agent in Agent::ALL {
        let origin = Origin::AgentButton { agent };
        let expected = agent.runner() == AgentRunner::ProjectVm
            && crate::setup::Catalog::get()
                .find(agent.id())
                .is_some_and(|c| c.is_available_on(guest));
        assert_eq!(linux_build(&k, &origin), expected, "{agent:?}");
    }
    assert!(!linux_build(&k, &Origin::ProjectTerminal));
    // Pi has its Linux build (D-178).
    assert!(linux_build(&k, &Origin::AgentButton { agent: Agent::Pi }));
}
