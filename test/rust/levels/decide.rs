use super::*;
use crate::isolation::Blocker;

/// A kit's root for the system running the tests (nothing is read: the
/// decision touches no disk).
fn root() -> PathBuf {
    std::env::temp_dir().join("atollo-levels-decide")
}

fn shop_dir() -> PathBuf {
    root().join("projects").join("shop")
}

fn in_shop(below: Option<&str>) -> Place {
    Place::Project {
        name: "shop".into(),
        dir: shop_dir(),
        below: below.map(str::to_owned),
    }
}

fn elsewhere() -> Place {
    Place::Folder {
        dir: root().join("elsewhere"),
    }
}

fn tools() -> Vec<PathBuf> {
    vec![root().join("data").join("apps").join("linux-x64")]
}

/// The VM on, everything ready.
fn on() -> Facts {
    Facts {
        vm: VmState::On,
        guest_dir: Some("/k/projects/shop".into()),
        restart_pending: false,
        linux_build: true,
        guest_tools: tools(),
    }
}

/// The VM off, for a reason.
fn off() -> Facts {
    Facts {
        vm: VmState::Off {
            blockers: vec![Blocker::HypervisorOff],
        },
        ..on()
    }
}

fn req(origin: Origin, place: Place) -> Request {
    Request { origin, place }
}

/// Every origin: each kind once, each level-3 one with every agent.
fn origins() -> Vec<Origin> {
    let mut all = vec![
        Origin::KitTerminal {
            terminal: "all".into(),
        },
        Origin::EditorTerminal,
        Origin::ProjectTerminal,
        Origin::EditorTask {
            command: "npm test".into(),
        },
        Origin::Action {
            action: "dev".into(),
            web: true,
        },
        Origin::Action {
            action: "build".into(),
            web: false,
        },
        Origin::Site {
            action: "site".into(),
        },
        Origin::Pool {
            pool: "apache-2.4".into(),
        },
        Origin::Scaffold {
            base: "laravel".into(),
        },
        Origin::Sandbox,
    ];
    for agent in Agent::ALL {
        all.push(Origin::AgentButton { agent });
        all.push(Origin::EditorAgentProfile { agent });
        for from in [Level::Manual, Level::Project] {
            all.push(Origin::Forwarded { agent, from });
        }
        all.push(Origin::EditorExtension {
            extension: "anthropic.claude-code".into(),
            agent,
        });
    }
    all
}

/// The level of each origin, written as §3.4's table says it (a new
/// origin does not compile until it is added here).
fn level_in_the_table(origin: &Origin) -> Level {
    match origin {
        Origin::KitTerminal { .. } => Level::Manual,
        Origin::EditorTerminal => Level::Manual,
        Origin::ProjectTerminal => Level::Project,
        Origin::EditorTask { .. } => Level::Project,
        Origin::Action { .. } => Level::Project,
        Origin::Site { .. } => Level::Project,
        Origin::Pool { .. } => Level::Project,
        Origin::Scaffold { .. } => Level::Project,
        Origin::Sandbox => Level::Agent,
        Origin::AgentButton { .. } => Level::Agent,
        Origin::EditorAgentProfile { .. } => Level::Agent,
        Origin::Forwarded { .. } => Level::Agent,
        Origin::EditorExtension { .. } => Level::Agent,
    }
}

/// The rows of §3.4 written one by one, as the document has them: the
/// first that matches.
fn first_row(req: &Request, facts: &Facts) -> Option<Rule> {
    let agent = req.origin.level() == Level::Agent;
    let tools = matches!(req.origin, Origin::Sandbox);
    let project = matches!(req.place, Place::Project { .. });
    let folder = !project;
    let on = facts.vm == VmState::On;
    let off = !on;
    let wants_agent = req.origin.agent().is_some();
    let rows = [
        (Rule::R1, agent && off),
        (Rule::R2, !agent && project),
        (Rule::R3, !agent && folder),
        (Rule::R4, agent && folder && on),
        (Rule::R5, agent && project && on && facts.restart_pending),
        (
            Rule::R6,
            agent && project && on && facts.guest_dir.is_none(),
        ),
        (Rule::R7, wants_agent && project && on && !facts.linux_build),
        (Rule::R8, (tools || wants_agent) && project && on),
    ];
    rows.into_iter().find(|(_, m)| *m).map(|(r, _)| r)
}

/// Every shape of the facts: the VM on, off with a reason, off with none
/// listed; each other fact both ways.
fn every_facts() -> Vec<Facts> {
    let mut all = Vec::new();
    for vm in [
        VmState::On,
        VmState::Off {
            blockers: vec![Blocker::SbxSignedOut],
        },
        VmState::Off { blockers: vec![] },
    ] {
        for restart_pending in [false, true] {
            for guest_dir in [None, Some("/k/projects/shop".to_owned())] {
                for linux_build in [false, true] {
                    for guest_tools in [vec![], tools()] {
                        all.push(Facts {
                            vm: vm.clone(),
                            guest_dir: guest_dir.clone(),
                            restart_pending,
                            linux_build,
                            guest_tools,
                        });
                    }
                }
            }
        }
    }
    all
}

/// The invariants of §2 that hold for every placement (I3, I7) and what
/// the table says each placement carries.
fn check_invariants(req: &Request, facts: &Facts, rule: Rule, p: &Placement) {
    let at = format!("{rule:?} for {req:?} with {facts:?}: {p:?}");
    assert_eq!(p.level(), req.origin.level(), "{at}");
    match p {
        Placement::Pc {
            level,
            project,
            dir,
        } => {
            assert_ne!(*level, Level::Agent, "I3: {at}");
            assert_eq!(dir, &req.place.dir(), "{at}");
            match &req.place {
                Place::Project { name, .. } => assert_eq!(project.as_ref(), Some(name), "{at}"),
                Place::Folder { .. } => assert_eq!(project, &None, "{at}"),
            }
        }
        Placement::Vm {
            level,
            project,
            sandbox,
            workdir,
            mounts,
            host,
            program,
            ..
        } => {
            assert_eq!(*level, Level::Agent, "D-183: {at}");
            let Place::Project { name, dir, .. } = &req.place else {
                panic!("a VM outside a project: {at}");
            };
            assert_eq!(project, name, "{at}");
            assert_eq!(&mounts.workspace_rw, dir, "{at}");
            assert_eq!(mounts.tools_ro, facts.guest_tools, "{at}");
            assert!(
                workdir.starts_with(facts.guest_dir.as_deref().unwrap()),
                "{at}"
            );
            assert!(
                !host.local_models || req.origin.agent() == Some(Agent::Pi),
                "{at}"
            );
            // One sandbox per project, every agent in it (D-184).
            let Sandbox::Project { name: sandbox } = sandbox;
            assert_eq!(sandbox, &crate::isolation::vm::name(name), "{at}");
            assert_eq!(program.agent(), req.origin.agent(), "{at}");
            assert_eq!(
                matches!(program, VmProgram::Tools),
                req.origin == Origin::Sandbox,
                "{at}"
            );
        }
        Placement::Refused { reason, fix, .. } => {
            assert_eq!(*fix, reason.fix(), "I7: {at}");
            for lang in crate::common::i18n::LANGS {
                assert!(
                    !labels::refusal_text(reason, lang).trim().is_empty(),
                    "I7: {at}"
                );
            }
        }
    }
}

#[test]
fn the_level_comes_from_the_origin() {
    for origin in origins() {
        assert_eq!(origin.level(), level_in_the_table(&origin), "{origin:?}");
        assert_eq!(
            origin.agent().is_some(),
            origin.level() == Level::Agent && origin != Origin::Sandbox,
            "{origin:?}"
        );
    }
    // The agents and their commands.
    for agent in Agent::ALL {
        assert_eq!(Agent::from_id(agent.id()), Some(agent));
        for cmd in agent.commands() {
            assert_eq!(Agent::from_command(cmd), Some(agent));
        }
    }
    assert_eq!(Agent::from_command("agy"), Some(Agent::Antigravity));
    assert_eq!(Agent::from_id("gemini"), None);
    assert_eq!(Agent::from_command("bash"), None);
}

/// I8: every origin × {project, project subfolder, folder} × every facts
/// shape has a row, the first row of the table is the one applied, and
/// the placement keeps the invariants. The same request gives the same
/// placement.
#[test]
fn every_request_has_exactly_one_placement() {
    let mut seen = std::collections::BTreeSet::new();
    let mut count = 0;
    for origin in origins() {
        for place in [in_shop(None), in_shop(Some("web/src")), elsewhere()] {
            let r = req(origin.clone(), place);
            for facts in every_facts() {
                let row = first_row(&r, &facts)
                    .unwrap_or_else(|| panic!("no row for {r:?} with {facts:?}"));
                let (rule, placement) = decided(&r, &facts);
                assert_eq!(rule, row, "{r:?} with {facts:?}");
                assert_eq!(decide(&r, &facts), placement, "pure: {r:?}");
                check_invariants(&r, &facts, rule, &placement);
                seen.insert(rule);
                count += 1;
            }
        }
    }
    assert_eq!(seen.len(), 8, "every row is reachable: {seen:?}");
    assert!(count > 3000, "{count}");
}

#[test]
fn r1_agents_need_the_vm() {
    for agent in Agent::ALL {
        for place in [in_shop(None), elsewhere()] {
            let r = req(Origin::AgentButton { agent }, place);
            assert_eq!(
                decided(&r, &off()),
                (
                    Rule::R1,
                    Placement::Refused {
                        level: Level::Agent,
                        reason: Refusal::AgentsNeedVm {
                            blockers: vec![Blocker::HypervisorOff],
                        },
                        fix: Fix::OpenVmNotice,
                    }
                )
            );
        }
    }
}

#[test]
fn r2_everything_but_an_agent_in_a_project_is_on_the_pc() {
    let origins = [
        (
            Origin::KitTerminal {
                terminal: "all".into(),
            },
            Level::Manual,
        ),
        (Origin::EditorTerminal, Level::Manual),
        (Origin::ProjectTerminal, Level::Project),
        (
            Origin::EditorTask {
                command: "npm test".into(),
            },
            Level::Project,
        ),
        (
            Origin::Action {
                action: "dev".into(),
                web: true,
            },
            Level::Project,
        ),
        (
            Origin::Site {
                action: "site".into(),
            },
            Level::Project,
        ),
        (
            Origin::Scaffold {
                base: "laravel".into(),
            },
            Level::Project,
        ),
    ];
    for (origin, level) in origins {
        // Whatever the VM says (D-183): it runs the agents only.
        for facts in [on(), off()] {
            assert_eq!(
                decided(&req(origin.clone(), in_shop(Some("web/src"))), &facts),
                (
                    Rule::R2,
                    Placement::Pc {
                        level,
                        project: Some("shop".into()),
                        dir: shop_dir().join("web").join("src"),
                    }
                ),
                "{origin:?}"
            );
        }
    }
}

#[test]
fn r3_everything_but_an_agent_elsewhere_is_on_the_pc() {
    for facts in [on(), off()] {
        assert_eq!(
            decided(&req(Origin::EditorTerminal, elsewhere()), &facts),
            (
                Rule::R3,
                Placement::Pc {
                    level: Level::Manual,
                    project: None,
                    dir: root().join("elsewhere"),
                }
            )
        );
        // A pool, in its own folder, also with the VM on: sealed for no
        // project.
        let dir = root().join("data").join("servers").join("apache-2.4");
        let pool = Request {
            origin: Origin::Pool {
                pool: "apache-2.4".into(),
            },
            place: Place::Folder { dir: dir.clone() },
        };
        assert_eq!(
            decided(&pool, &facts),
            (
                Rule::R3,
                Placement::Pc {
                    level: Level::Project,
                    project: None,
                    dir,
                }
            )
        );
    }
}

#[test]
fn r4_an_agent_needs_a_project() {
    for agent in Agent::ALL {
        let r = req(
            Origin::Forwarded {
                agent,
                from: Level::Manual,
            },
            elsewhere(),
        );
        assert_eq!(
            decided(&r, &on()),
            (
                Rule::R4,
                Placement::Refused {
                    level: Level::Agent,
                    reason: Refusal::AgentOutsideProject,
                    fix: Fix::OpenFromProject,
                }
            )
        );
    }
}

#[test]
fn r5_a_pending_restart_starts_no_agent() {
    let facts = Facts {
        restart_pending: true,
        guest_dir: None,
        ..on()
    };
    let r = req(
        Origin::AgentButton {
            agent: Agent::Claude,
        },
        in_shop(None),
    );
    assert_eq!(
        decided(&r, &facts),
        (
            Rule::R5,
            Placement::Refused {
                level: Level::Agent,
                reason: Refusal::DaemonRestartPending,
                fix: Fix::RestartDaemon,
            }
        )
    );
    // The project's own terminal does not wait for the VM.
    assert_eq!(
        decided(&req(Origin::ProjectTerminal, in_shop(None)), &facts).0,
        Rule::R2
    );
}

#[test]
fn r6_a_network_share_never_gets_a_sandbox() {
    let facts = Facts {
        guest_dir: None,
        ..on()
    };
    assert_eq!(
        decided(
            &req(
                Origin::AgentButton { agent: Agent::Pi },
                in_shop(Some("web"))
            ),
            &facts
        ),
        (
            Rule::R6,
            Placement::Refused {
                level: Level::Agent,
                reason: Refusal::NoPathInVm {
                    dir: shop_dir().join("web").display().to_string(),
                },
                fix: Fix::UseLocalDisk,
            }
        )
    );
}

#[test]
fn r7_no_linux_build_no_agent() {
    let facts = Facts {
        linux_build: false,
        ..on()
    };
    // Every agent is the kit's build for the VM's Linux (D-184).
    for agent in Agent::ALL {
        assert_eq!(
            decided(&req(Origin::AgentButton { agent }, in_shop(None)), &facts),
            (
                Rule::R7,
                Placement::Refused {
                    level: Level::Agent,
                    reason: Refusal::NoLinuxBuild { agent },
                    fix: Fix::None,
                }
            )
        );
    }
}

#[test]
fn r8_every_agent_runs_in_the_project_vm() {
    for agent in Agent::ALL {
        let r = req(
            Origin::EditorExtension {
                extension: "x.y".into(),
                agent,
            },
            in_shop(Some("web")),
        );
        assert_eq!(
            decided(&r, &on()),
            (
                Rule::R8,
                Placement::Vm {
                    level: Level::Agent,
                    project: "shop".into(),
                    sandbox: Sandbox::Project {
                        name: crate::isolation::vm::name("shop"),
                    },
                    workdir: "/k/projects/shop/web".into(),
                    mounts: Mounts {
                        workspace_rw: shop_dir(),
                        tools_ro: tools(),
                    },
                    host: HostAccess {
                        databases: true,
                        local_models: agent == Agent::Pi,
                    },
                    program: VmProgram::Agent { agent },
                }
            )
        );
    }
}

#[test]
fn r8_the_sandbox_itself_is_the_projects() {
    assert_eq!(
        decided(&req(Origin::Sandbox, in_shop(None)), &on()),
        (
            Rule::R8,
            Placement::Vm {
                level: Level::Agent,
                project: "shop".into(),
                sandbox: Sandbox::Project {
                    name: crate::isolation::vm::name("shop"),
                },
                workdir: "/k/projects/shop".into(),
                mounts: Mounts {
                    workspace_rw: shop_dir(),
                    tools_ro: tools(),
                },
                host: HostAccess {
                    databases: true,
                    local_models: false,
                },
                program: VmProgram::Tools,
            }
        )
    );
    // No build of an agent is asked for the tools alone.
    let facts = Facts {
        linux_build: false,
        ..on()
    };
    assert_eq!(
        decided(&req(Origin::Sandbox, in_shop(None)), &facts).0,
        Rule::R8
    );
}

/// I3: whatever the facts and the place, an agent never runs on the PC.
#[test]
fn an_agent_is_never_on_the_pc() {
    for origin in origins().into_iter().filter(|o| o.agent().is_some()) {
        for place in [in_shop(None), elsewhere()] {
            for facts in every_facts() {
                let p = decide(&req(origin.clone(), place.clone()), &facts);
                assert!(!matches!(p, Placement::Pc { .. }), "{origin:?}: {p:?}");
            }
        }
    }
}

/// I7: a refusal has its reason and the fix of that reason, and a text
/// in every language that says what to do.
#[test]
fn a_refusal_always_has_a_reason_and_a_fix() {
    let reasons = [
        Refusal::AgentsNeedVm {
            blockers: vec![Blocker::KvmMissing],
        },
        Refusal::AgentOutsideProject,
        Refusal::NoPathInVm {
            dir: r"\\server\share\shop".into(),
        },
        Refusal::NoLinuxBuild { agent: Agent::Pi },
        Refusal::DaemonRestartPending,
    ];
    let fixes: Vec<Fix> = reasons.iter().map(Refusal::fix).collect();
    assert_eq!(
        fixes,
        [
            Fix::OpenVmNotice,
            Fix::OpenFromProject,
            Fix::UseLocalDisk,
            Fix::None,
            Fix::RestartDaemon,
        ]
    );
    for origin in origins() {
        for place in [in_shop(None), elsewhere()] {
            for facts in every_facts() {
                if let Placement::Refused { reason, fix, .. } =
                    decide(&req(origin.clone(), place.clone()), &facts)
                {
                    assert_eq!(fix, reason.fix(), "{origin:?}");
                }
            }
        }
    }
}

/// What the UI and the request files read (§4.1, §4.3).
#[test]
fn placements_serialize_as_the_ipc_says() {
    let refused = decide(
        &req(Origin::AgentButton { agent: Agent::Pi }, elsewhere()),
        &off(),
    );
    let json = serde_json::to_value(&refused).unwrap();
    assert_eq!(json["where"], "refused");
    assert_eq!(json["level"], "agent");
    assert_eq!(json["reason"]["code"], "agents_need_vm");
    assert_eq!(json["fix"], "open_vm_notice");
    let vm = decide(
        &req(Origin::AgentButton { agent: Agent::Pi }, in_shop(None)),
        &on(),
    );
    let json = serde_json::to_value(&vm).unwrap();
    assert_eq!(json["where"], "vm");
    assert_eq!(json["level"], "agent");
    assert_eq!(json["sandbox"]["kind"], "project");
    assert_eq!(json["program"]["kind"], "agent");
    let pc = decide(&req(Origin::ProjectTerminal, in_shop(None)), &on());
    let json = serde_json::to_value(&pc).unwrap();
    assert_eq!(json["where"], "pc");
    assert_eq!(json["level"], "project");
    let origin: Origin =
        serde_json::from_str(r#"{"kind":"forwarded","agent":"claude","from":"manual"}"#).unwrap();
    assert_eq!(
        origin,
        Origin::Forwarded {
            agent: Agent::Claude,
            from: Level::Manual
        }
    );
    let task: Origin =
        serde_json::from_str(r#"{"kind":"editorTask","command":"npm test"}"#).unwrap();
    assert_eq!(task.level(), Level::Project);
}
