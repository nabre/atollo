//! Components: the choice, the installation in progress, files downloaded
//! by hand (D-82).

use super::*;
use crate::router::availability::{not_needed_reason, unavailable_reason};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentInfo {
    pub id: String,
    pub name: String,
    pub group: Group,
    pub desc: String,
    pub mb: u32,
    pub chosen: bool,
    pub installed: bool,
    /// Something of it is on disk, even half installed: it can be removed.
    pub present: bool,
    /// A system component, or needed by one: always installed.
    pub locked: bool,
    pub version: Option<String>,
    /// This version of the app can install it on this kit.
    pub available: bool,
    /// Why it is not available on this kit's target, already translated:
    /// the catalog's own reason (`unavailable`, Android on Linux ARM64), or
    /// not made for this system (Windows only, on macOS or Linux).
    pub unavailable_reason: Option<String>,
    /// Not needed on this kit's target, with the catalog's reason already
    /// translated (ConPTY on macOS and Linux, whose terminal uses the
    /// system's PTY, D-177): `available` false, no `unavailable_reason`.
    pub not_needed: Option<String>,
    /// Components it needs on this kit's target (Apache needs PHP, Kotlin
    /// needs Java; C and C++ need Zig on macOS and Linux only).
    pub needs: Vec<String>,
    /// C and C++ on a Mac without Xcode Command Line Tools (`xcode-select
    /// -p` fails): C and C++ work, the Apple frameworks do not (D-112).
    /// Only the Mac itself can tell: false for a kit inspected or prepared
    /// from another system.
    pub xcode_missing: bool,
    /// One folder per minor line, side by side (versions/mod.rs); the others
    /// are always at their latest version.
    pub versioned: bool,
    /// Its official site.
    pub home: String,
    /// Bootstrap archives pre-downloaded for this target; finishes installing
    /// on first start on that OS.
    pub bootstrap_staged: bool,
    /// An agent's row (LEVELS.md §7, Q11): «serve la VM» with the reason
    /// (without the VM); `None` for every other component. With one, `available` is
    /// false: nothing to install or update.
    pub agent: Option<crate::levels::agents::AgentRow>,
    /// An agent's folder for a system where the kit no longer keeps it (an
    /// install for the PC of an earlier Atollo): never on a `PATH`, removed
    /// by the next «Applica» (`levels::agents::obsolete`).
    pub leftover: bool,
}

/// When the Tools page opens: what was deleted by hand from the kit's
/// folder is no longer chosen (versions::sync_with_disk); the names of
/// those. Not while an installation runs (its folders come and go), nor when
/// inspecting a non-host target. Off the window's thread: it reads every
/// component's folders, and rebuilds the kit's environment. It writes
/// `components.txt`: in order with the page's clicks ([`INSTALLS`]).
#[tauri::command]
pub async fn components_sync<R: Runtime>(
    webview: Webview<R>,
    target: Option<String>,
) -> CmdResult<Vec<String>> {
    ensure_main(&webview)?;
    let work = off_main(&webview, "components", move |_, state| {
        if state.setup.snapshot().running {
            return Ok(Vec::new());
        }
        let kit = kit_for(&state.kit, target.as_deref())?;
        if kit.target() != state.kit.target() {
            return Ok(Vec::new());
        }
        let off = crate::versions::sync_with_disk(&kit).or_fail_chain("components")?;
        if !off.is_empty() {
            apply_env(state);
        }
        Ok(off)
    });
    in_order(&INSTALLS, work).await
}

/// The catalog as the Tools page shows it; with `ids`, only those (a row
/// whose install just ended, read again without the whole list). Off the
/// window's thread: it reads every component's folders.
#[tauri::command]
pub async fn components_list<R: Runtime>(
    webview: Webview<R>,
    target: Option<String>,
    ids: Option<Vec<String>>,
) -> CmdResult<Vec<ComponentInfo>> {
    ensure_main(&webview)?;
    off_main(&webview, "components", move |_, state| {
        let kit = kit_for(&state.kit, target.as_deref())?;
        let vm = crate::levels::agents::known_vm();
        let mut infos = component_infos(&kit, state.lang(), &vm);
        if let Some(ids) = ids {
            infos.retain(|c| ids.contains(&c.id));
        }
        Ok(infos)
    })
    .await
}

/// Verifies every installed or bootstrap-staged component on `target` (live
/// version commands when executable on the host, static key/version check
/// when probing a foreign target).
#[tauri::command]
pub async fn components_check<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    target: Option<String>,
) -> CmdResult<Vec<crate::setup::checkup::Check>> {
    ensure_main(&webview)?;
    let kit = kit_for(&state.kit, target.as_deref())?;
    let env = state.env();
    tauri::async_runtime::spawn_blocking(move || {
        Ok(crate::router::Router::new(kit).probe_health(&env))
    })
    .await
    .or_fail("check")?
}

/// The component × target grid of the Tools page («Matrice», D-121): what
/// each of the four systems can run from this PC, the state of every
/// component on each of them, and the system chosen last there.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetGrid {
    /// In the order of the columns (`Target::ALL`).
    pub profiles: [crate::router::target::TargetProfile; 4],
    /// The same order, one per system.
    pub targets: Vec<crate::router::probe::TargetSummary>,
    /// `tools_target` of settings.json, when it is a system the app knows.
    pub chosen: Option<String>,
    /// The VM's Linux when the VM is on (D-174): its column is the VM's.
    pub vm: Option<String>,
}

/// The grid: off the window's thread, it reads the disk of the four systems
/// (and may wait for `sbx`'s answer on the VM, [`vm_tools_target`]).
#[tauri::command]
pub async fn target_profiles<R: Runtime>(webview: Webview<R>) -> CmdResult<TargetGrid> {
    ensure_main(&webview)?;
    off_main(&webview, "targets", |_, state| {
        let vm = vm_tools_target(state);
        Ok(target_grid(&state.kit, state.lang(), vm))
    })
    .await
}

pub(super) fn target_grid(kit: &Kit, lang: &str, vm: Option<crate::kit::Target>) -> TargetGrid {
    let chosen = Settings::load(kit)
        .settings
        .tools_target
        .and_then(|t| crate::kit::Target::parse(&t))
        .map(|t| t.as_str().to_owned());
    let targets = crate::router::probe::probe_all_targets(kit, lang);
    TargetGrid {
        profiles: crate::router::target::all_profiles(),
        targets,
        chosen,
        vm: vm.map(|t| t.as_str().to_owned()),
    }
}

/// Rust's compiler on Windows, chosen on Rust's page in Tools (D-137):
/// `msvc`, or `None` for the default (GNU, everything in the kit). It
/// applies from the next install or update of Rust.
#[tauri::command]
pub fn rust_toolchain_set<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    toolchain: Option<String>,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let toolchain = toolchain.filter(|t| !t.is_empty() && t != "gnu");
    if let Some(t) = toolchain.as_deref().filter(|t| *t != "msvc") {
        return Err(tr(&state, "Valore non valido: {0}", &[t]).into());
    }
    Settings::update(&state.kit, |settings, problem| {
        if let Some(problem) = problem {
            anyhow::bail!("{problem}");
        }
        settings.rust_toolchain = toolchain;
        Ok(())
    })
    .map(drop)
    .or_fail_chain("settings")
}

/// «Piattaforma» of the Tools page, remembered between sessions in
/// settings.json (`None`: this PC's system again). Only that value
/// changes: the kit's environment is not rebuilt.
#[tauri::command]
pub fn tools_target_set<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    target: Option<String>,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    save_tools_target(&state.kit, target.as_deref()).or_fail_chain("settings")
}

/// Writes `tools_target` into settings.json, keeping the rest of the file;
/// one that cannot be read is not overwritten (common/settings.rs). An unknown
/// system is refused.
pub(super) fn save_tools_target(kit: &Kit, target: Option<&str>) -> anyhow::Result<()> {
    let target = match target.map(str::trim).filter(|s| !s.is_empty()) {
        None => None,
        Some(raw) => Some(
            crate::kit::Target::parse(raw)
                .ok_or_else(|| anyhow::anyhow!("unknown target: {raw}"))?
                .as_str()
                .to_owned(),
        ),
    };
    Settings::update(kit, |settings, problem| {
        if let Some(problem) = problem {
            anyhow::bail!("{problem}");
        }
        settings.tools_target = target;
        Ok(())
    })
    .map(drop)
}

/// The Xcode Command Line Tools are missing on this Mac, for a kit of
/// `target`: only a darwin kit on a Mac, where `xcode-select -p` fails.
/// The answer is the installer's, kept for a while (`xcode_selected`):
/// the page asks at every load.
fn xcode_missing_on_host(target: crate::kit::Target) -> bool {
    xcode_missing(
        target,
        crate::kit::Os::current(),
        crate::install::xcode_selected,
    )
}

/// [`xcode_missing_on_host`] with the host's system and `xcode-select -p`
/// given (`None`: it failed), so that every target is tested anywhere.
pub(super) fn xcode_missing(
    target: crate::kit::Target,
    host: crate::kit::Os,
    selected: impl FnOnce() -> Option<String>,
) -> bool {
    target.is_macos() && host.is_macos() && selected().is_none_or(|s| s.trim().is_empty())
}

/// The catalog as the "Linguaggi e strumenti" page shows it, from this PC:
/// a component with an `unavailableReason` is not available on this
/// target (see [`unavailable_reason`]). The agents with the VM as `vm`
/// says (LEVELS.md §7): as installed for the VM's Linux (their only build
/// the kit keeps, Q11, D-184); nothing of them installable without it.
pub(super) fn component_infos(
    kit: &Kit,
    lang: &str,
    vm: &crate::levels::VmState,
) -> Vec<ComponentInfo> {
    let catalog = Catalog::get();
    let choice = Choice::read_or_defaults(kit, catalog);
    let target = kit.target();
    let locked = catalog.locked_on(target);
    // Chosen here also when a chosen one needs it on this target only (Zig
    // for C and C++ on Linux): the run installs it, the choice does not
    // save it (see `choice_text`).
    let here = catalog.needs_only_on(&choice.chosen_ids(catalog), target);
    let xcode_missing = xcode_missing_on_host(target);
    // The order of the page, which an installation follows too.
    catalog
        .in_page_order()
        .into_iter()
        .map(|c| {
            let off = !c.is_available_on(target);
            let unavailable_reason = unavailable_reason(c, target, lang);
            let not_needed = not_needed_reason(c, target, lang);
            let agent = crate::levels::agents::agent_row(c, vm, lang);
            // An agent's state is the one of its build for the VM's Linux.
            let shown = agent_kit(kit, c);
            ComponentInfo {
                id: c.id.clone(),
                name: c.name.clone(),
                group: c.group,
                desc: crate::common::i18n::t(lang, &c.desc, &[]),
                mb: c.mb,
                chosen: choice.is_chosen(&c.id) || here.contains(c.id.as_str()),
                installed: c.is_installed(&shown),
                present: c.is_present(&shown),
                locked: locked.contains(c.id.as_str()),
                version: c.installed_version(&shown),
                available: install::installer(&c.id).is_some() && !off && agent.is_none(),
                unavailable_reason,
                not_needed,
                needs: c.needs_list_on(target),
                xcode_missing: c.id == "cc" && xcode_missing,
                versioned: c.versioned,
                home: c.home.clone(),
                bootstrap_staged: crate::router::bootstrap::is_staged(&shown, c),
                leftover: crate::levels::agents::obsolete(c, target) && c.is_present(kit),
                agent,
            }
        })
        .collect()
}

/// Where the Tools page reads `c`'s state for `kit`: an agent's on the
/// VM's Linux of `kit`'s system (the only build the kit keeps, Q11, D-184;
/// the same kit on Linux), `kit` for anything else.
fn agent_kit(kit: &Kit, c: &crate::setup::Component) -> Kit {
    let guest = crate::levels::agents::agent_of(c)
        .and_then(|_| crate::isolation::vm::guest_target(kit.target()))
        .filter(|g| *g != kit.target());
    match guest {
        Some(g) => kit.for_target(g),
        None => kit.clone(),
    }
}

/// Sends every change of the preparation to the app window.
/// At the end of a run the kit environment is built again: its PATH lists
/// only the programs present (Git, Node.js...), so one just installed is
/// seen by the editor, the consoles and the projects without a restart.
/// Called without the snapshot's lock, on the run's thread, before the
/// snapshot says the run ended (`SetupTask::end`): `setup_state` does not
/// wait for the rebuild, and `ensure` reads the new environment.
pub fn setup_notify<R: Runtime>(app: AppHandle<R>) -> Notify {
    Box::new(move |snapshot: &Snapshot| {
        if !snapshot.running && !snapshot.steps.is_empty() {
            use tauri::Manager;
            if let Some(state) = app.try_state::<AppState>() {
                apply_env(&state);
            }
        }
        let _ = app.emit_to(MAIN, super::events::SETUP, snapshot);
    })
}

#[tauri::command]
pub fn setup_state<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> CmdResult<Snapshot> {
    ensure_main(&webview)?;
    Ok(state.setup.snapshot())
}

/// Installs the components chosen (or only `only`); `update_all` also
/// updates what is installed. Returns at once; progress comes as `setup`
/// and `job`. While another run runs it waits in the queue. In the order
/// clicked with the page's other installs ([`INSTALLS`]): as a `fn` it
/// waited on the window's thread for the one clicked before.
#[tauri::command]
pub async fn setup_start<R: Runtime>(
    webview: Webview<R>,
    only: Option<Vec<String>>,
    update_all: bool,
    target: Option<String>,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let work = off_main(&webview, "setup", move |app, state| {
        let kit = kit_for(&state.kit, target.as_deref())?;
        let only = only.unwrap_or_default();
        if let Some(bad) = only.iter().find(|id| Catalog::get().find(id).is_none()) {
            return Err(format!("unknown component: {bad}").into());
        }
        let opts = Options {
            target: Some(kit.target()),
            only,
            update_all,
            ..Options::default()
        };
        state.setup.start(opts, false, setup_notify(app.clone()))?;
        Ok(())
    });
    in_order(&INSTALLS, work).await
}

/// The components with a new version (N-9), from the versions asked last;
/// nothing is asked here. Off the window's thread: it reads the installed
/// version of every component.
#[tauri::command]
pub async fn updates_list<R: Runtime>(
    webview: Webview<R>,
) -> CmdResult<crate::setup::updates::Updates> {
    ensure_main(&webview)?;
    off_main(&webview, "updates", |_, state| {
        Ok(crate::setup::updates::pending(
            &state.kit,
            &crate::levels::agents::known_vm(),
        ))
    })
    .await
}

/// «Controlla ora»: each installed component's source asked now.
#[tauri::command]
pub async fn updates_check<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> CmdResult<crate::setup::updates::Updates> {
    ensure_main(&webview)?;
    let (kit, env) = (state.kit.clone(), state.env());
    let token = super::app::github_token(&state.host);
    tauri::async_runtime::spawn_blocking(move || {
        crate::setup::updates::check(&kit, &env, token)?;
        anyhow::Ok(crate::setup::updates::pending(
            &kit,
            &crate::levels::agents::known_vm(),
        ))
    })
    .await
    .or_fail("updates")?
    .or_fail_chain("updates")
}

/// A component's site (`home`) or the notes of its new version (`notes`),
/// in the PC's browser: only the catalog's addresses, never one from the
/// page.
#[tauri::command]
pub fn component_link_open<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    id: String,
    kind: String,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let c = Catalog::get()
        .find(&id)
        .ok_or_else(|| format!("unknown component: {id}"))?;
    let url = match kind.as_str() {
        "home" => c.home.clone(),
        "notes" => {
            let latest = crate::setup::updates::load(&state.kit);
            let version = latest.versions.get(&id).cloned().unwrap_or_default();
            crate::setup::updates::notes_url(c, &version)
        }
        other => return Err(format!("unknown link: {other}").into()),
    };
    if !url.starts_with("https://") {
        return Err(format!("not an https address: {url}").into());
    }
    super::app::open_in_browser(&url)
}

/// «Verifica i file» (N-5): every installed component of `target` against
/// the manifest of its installation.
#[tauri::command]
pub async fn components_integrity<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    target: Option<String>,
) -> CmdResult<Vec<crate::setup::integrity::Integrity>> {
    ensure_main(&webview)?;
    let kit = kit_for(&state.kit, target.as_deref())?;
    tauri::async_runtime::spawn_blocking(move || crate::setup::integrity::check_all(&kit))
        .await
        .or_fail("integrity")
}

/// One damaged folder to install again: the component, and its version's
/// folder (a saved line, or the default).
#[derive(Debug, serde::Deserialize)]
pub struct Repair {
    pub id: String,
    pub line: Option<String>,
}

/// «Reinstalla» of a damaged component: marked to be installed again at
/// its version (D-92), then prepared now. Off the window's thread, in the
/// order clicked, as [`setup_start`].
#[tauri::command]
pub async fn components_repair<R: Runtime>(
    webview: Webview<R>,
    items: Vec<Repair>,
    target: Option<String>,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let work = off_main(&webview, "repair", move |app, state| {
        let kit = kit_for(&state.kit, target.as_deref())?;
        let items: Vec<(String, Option<String>)> =
            items.into_iter().map(|r| (r.id, r.line)).collect();
        let only =
            crate::setup::integrity::mark_for_repair(&kit, &items).or_fail_chain("repair")?;
        let opts = Options {
            target: Some(kit.target()),
            only,
            ..Options::default()
        };
        state.setup.start(opts, false, setup_notify(app.clone()))?;
        Ok(())
    });
    in_order(&INSTALLS, work).await
}

/// Installs `id` now, at its latest version, from the table of the Tools
/// page: it becomes chosen, the other changes of the page stay pending.
/// Off the window's thread: where it goes asks `sbx` ([`vm_tools_target`]);
/// in the order clicked ([`INSTALLS`]).
#[tauri::command]
pub async fn components_install<R: Runtime>(
    webview: Webview<R>,
    id: String,
    target: Option<String>,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    in_order(
        &INSTALLS,
        off_main(&webview, "components", move |app, state| {
            super::versions::install_now(app.clone(), state, id, None, target)
        }),
    )
    .await
}

/// The choice of the "Linguaggi e strumenti" page: `chosen` are the ids to
/// keep installed (required ones always are, dependencies follow). Saved in
/// `components.txt`, then installed and removed in the background. Off the
/// window's thread: where the tools go asks `sbx` ([`vm_tools_target`]);
/// in the order clicked ([`INSTALLS`]): the last choice is the one saved.
#[tauri::command]
pub async fn components_apply<R: Runtime>(
    webview: Webview<R>,
    chosen: Vec<String>,
    update_all: bool,
    target: Option<String>,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let work = off_main(&webview, "components", move |app, state| {
        let kit = kit_for(&state.kit, target.as_deref())?;
        let catalog = Catalog::get();
        if let Some(bad) = chosen.iter().find(|id| catalog.find(id).is_none()) {
            return Err(format!("unknown component: {bad}").into());
        }
        {
            let _one = crate::setup::catalog::choice_lock();
            let text = choice_text(&kit, &chosen);
            let choice = Choice::parse(&text, catalog);
            choice.write(&kit, catalog).or_fail_chain("components")?;
        }
        // With the VM on (D-183): every chosen component on the PC, then the
        // agents' tools for the VM's Linux.
        let guest = (kit.target() == state.kit.target())
            .then(|| vm_tools_target(state))
            .flatten();
        let opts = Options {
            target: Some(kit.target()),
            update_all,
            then_tools_for: guest,
            ..Options::default()
        };
        state.setup.start(opts, false, setup_notify(app.clone()))?;
        Ok(())
    });
    in_order(&INSTALLS, work).await
}

/// The VM's Linux when the VM is on (D-173): where the chosen tools go,
/// the PC keeping the system components only; `None` without it.
///
/// It asks the VM's status ([`crate::isolation::status`]): `sbx`'s answer
/// of the last minute, or a new one, which takes seconds (minutes when its
/// daemon starts). Never on the window's thread: its callers are `async`
/// commands that call it off it. The last known status
/// ([`crate::levels::agents::known_vm`]) is not enough here: before the
/// first answer it says the VM is off, and the tools would be installed on
/// the PC instead of the VM's Linux (and a project's missing tools counted
/// there).
pub(super) fn vm_tools_target(state: &AppState) -> Option<crate::kit::Target> {
    let guest = crate::isolation::vm::guest_target(state.kit.target())?;
    crate::isolation::vm_on(&state.kit, &state.host, &state.env()).then_some(guest)
}

/// `components.txt` for the ids in `chosen`. What this target cannot install
/// (`unavailable` or outside `targets`) keeps its saved choice: the same kit
/// on another OS/arch must not lose it. Nor does what a chosen one needs on
/// this target only (Zig for C and C++ on Linux, ticked with them by the
/// page): the run on this target installs it anyway, and saving it would
/// install it on Windows too. The needs of the saved choice count as well:
/// the page seeds Zig as ticked next to C and C++, and unticking C and C++
/// leaves it in `chosen`, which must not turn it into a saved choice.
pub(super) fn choice_text(kit: &Kit, chosen: &[String]) -> String {
    let catalog = Catalog::get();
    let saved = Choice::read_or_defaults(kit, catalog);
    let target = kit.target();
    let ids: Vec<&str> = chosen.iter().map(String::as_str).collect();
    let mut here = catalog.needs_only_on(&ids, target);
    here.extend(catalog.needs_only_on(&saved.chosen_ids(catalog), target));
    catalog
        .all()
        .iter()
        .map(|c| {
            let on = if !crate::router::target::installable_on(c, target) {
                saved.is_chosen(&c.id)
            } else if here.contains(c.id.as_str()) {
                // Saved on and still ticked: an untick on the page wins
                // (Zig unticked on Linux stays off).
                saved.is_chosen(&c.id) && chosen.contains(&c.id)
            } else {
                chosen.contains(&c.id)
            };
            format!("{}={}\n", c.id, u8::from(on))
        })
        .collect()
}

/// The jobs kept (LEVELS.md §8.6): the installs running and queued, the
/// VMs being prepared, and the last ones that ended; then `job` events.
#[tauri::command]
pub fn jobs_list<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> CmdResult<Vec<crate::common::jobs::Job>> {
    ensure_main(&webview)?;
    Ok(state.setup.jobs().list())
}

/// Cancels job `id` alone: a queued install leaves the queue, the running
/// one stops (during its download, else at the end of its step); the
/// others go on. False when it cannot be cancelled (ended, or a VM's
/// preparation, which finishes its step).
#[tauri::command]
pub fn job_cancel<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    id: String,
) -> CmdResult<bool> {
    ensure_main(&webview)?;
    Ok(state.setup.cancel_job(&id, &setup_notify(app)))
}

#[tauri::command]
pub fn setup_cancel<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    state.setup.cancel(&setup_notify(app));
    Ok(())
}

/// Files downloaded by hand (setup/downloads.rs, D-82), dropped on the window:
/// kept in `data\downloads` for the next installation. Their names.
#[tauri::command]
pub async fn downloads_add<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    paths: Vec<String>,
) -> CmdResult<Vec<String>> {
    ensure_main(&webview)?;
    let kit = state.kit.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let files: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
        crate::setup::downloads::add(&kit, &files)
    })
    .await
    .or_fail("downloads")?
    .or_fail_chain("downloads")
}

/// The same, picked in the system's "Open" window; empty when closed.
#[tauri::command]
pub async fn downloads_pick<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> CmdResult<Vec<String>> {
    ensure_main(&webview)?;
    let owner = crate::platform::desktop::owner(&webview.window()).or_fail("window")?;
    let title = crate::common::i18n::t(state.lang(), "Scegli il file scaricato", &[]);
    let kit = state.kit.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let files = crate::platform::desktop::pick_files(&owner, &title, true)?;
        if files.is_empty() {
            return Ok(Vec::new());
        }
        crate::setup::downloads::add(&kit, &files)
    })
    .await
    .or_fail("downloads")?
    .or_fail_chain("downloads")
}

/// Opens in the PC's browser the address of the download that failed in
/// the step `name` of `id`: only one the preparation reported.
#[tauri::command]
pub fn setup_by_hand_open<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    id: String,
    name: String,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let url = state
        .setup
        .snapshot()
        .steps
        .into_iter()
        .rfind(|s| s.id == id && s.name == name)
        .and_then(|s| match s.outcome {
            Some(crate::setup::runner::Outcome::Failed {
                by_hand: Some(b), ..
            }) => Some(b.url),
            _ => None,
        })
        .filter(|u| u.starts_with("https://") || u.starts_with("http://"))
        .ok_or_else(|| format!("{id}: no download to do by hand"))?;
    crate::platform::jobs::open_url(&url).or_fail_chain("browser")
}

/// The packages kept to reinstall the conda environments without the
/// network after a move (D-118), for the Settings page. Off the window's
/// thread: their size is the walk of the whole package cache.
#[tauri::command]
pub async fn conda_kept<R: Runtime>(
    webview: Webview<R>,
) -> CmdResult<crate::install::conda::Status> {
    ensure_main(&webview)?;
    off_main(&webview, "conda", |_, state| {
        Ok(crate::install::conda::status(&state.kit))
    })
    .await
}

/// «Libera»: removes the kept packages of this target (D-118); not while a
/// preparation runs, which may be writing them.
#[tauri::command]
pub async fn conda_kept_free<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> CmdResult<crate::install::conda::Status> {
    ensure_main(&webview)?;
    let kit = state.kit.clone();
    let busy = tr(&state, "C'è già un'installazione in corso.", &[]);
    tauri::async_runtime::spawn_blocking(move || {
        let freed = crate::setup::runner::while_idle(&kit, || crate::install::conda::free(&kit))
            .map_err(|_| busy)?
            .or_fail_chain("conda")?;
        crate::common::log::info(&format!(
            "conda: kept packages freed, {} MB",
            freed.div_ceil(1 << 20)
        ));
        Ok(crate::install::conda::status(&kit))
    })
    .await
    .or_fail("conda")?
}

#[cfg(test)]
#[path = "../../../test/rust/ipc/setup.rs"]
mod tests;
