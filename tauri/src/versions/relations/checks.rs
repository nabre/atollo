//! The checks of the matrix: each link between components, at the versions
//! the kit has, and the runtimes the PC must provide.

use std::collections::BTreeMap;

use super::known::known;
use super::{admits, name_of, split, version_of, Have, Link, LinkStatus, RuntimeState};
use crate::kit::{Kit, Os};
use crate::setup::catalog::Catalog;
use crate::versions::ranges::{Bare, Range};

pub(super) fn links(lang: &str, haves: &BTreeMap<String, Have>) -> Vec<Link> {
    let catalog = Catalog::get();
    known()
        .links
        .iter()
        .filter(|s| haves.contains_key(split(&s.from).0) && haves.contains_key(&s.to))
        .map(|s| {
            let from = &haves[split(&s.from).0];
            let to = &haves[&s.to];
            let range = s
                .range
                .as_deref()
                .and_then(|r| Range::parse(r, Bare::Exact));
            // Only the kit's version counts: no lines "outside".
            let per_line = s.kit;
            let uses = to
                .kit
                .clone()
                .or_else(|| to.lines.first().map(|(l, _)| l.clone()));
            let outside: Vec<String> = match (&range, per_line) {
                (Some(r), false) => to
                    .lines
                    .iter()
                    .map(|(l, _)| l.clone())
                    .filter(|l| !admits(r, to, l))
                    .collect(),
                _ => Vec::new(),
            };
            let status = if !from.installed {
                LinkStatus::Inactive
            } else if !to.installed {
                if s.optional {
                    LinkStatus::Optional
                } else {
                    LinkStatus::Missing
                }
            } else if uses.as_deref().is_some_and(|l| match &range {
                // Composer is chosen for each PHP folder: it always fits.
                _ if s.from == "php/composer" => false,
                Some(r) => !admits(r, to, l),
                None => false,
            }) {
                LinkStatus::Incompatible
            } else {
                LinkStatus::Ok
            };
            Link {
                from: s.from.clone(),
                from_name: name_of(catalog, &s.from),
                to: s.to.clone(),
                to_name: name_of(catalog, &s.to),
                kind: s.kind.clone(),
                optional: s.optional,
                range: s.range.clone(),
                note: s.note.as_deref().map(|n| crate::i18n::t(lang, n, &[])),
                status,
                uses: uses.map(|l| version_of(to, &l)),
                outside,
            }
        })
        .collect()
}

/// The system runtime `id` exists on `os`: the Visual C++ DLLs, WebView2
/// and the x64 emulation of Windows 11 are Windows pieces, never missing
/// on macOS or Linux.
pub(super) fn runtime_on(id: &str, os: Os) -> bool {
    match id {
        "vcruntime" | "webview2" | "x64-emulation" => os.is_windows(),
        _ => true,
    }
}

pub(super) fn runtimes(kit: &Kit, lang: &str, haves: &BTreeMap<String, Have>) -> Vec<RuntimeState> {
    let catalog = Catalog::get();
    let arm64 = kit.arch() == crate::kit::Arch::Arm64;
    let installed = |id: &str| haves.get(id).is_some_and(|h| h.installed);
    known()
        .runtimes
        .iter()
        .filter(|s| (!s.arm64 || arm64) && runtime_on(&s.id, kit.os()))
        .map(|s| {
            let mut needed_by: Vec<String> = Vec::new();
            for f in &s.for_ {
                match f.as_str() {
                    "@app" => needed_by.push("Atollo".into()),
                    "@x64Only" => needed_by.extend(
                        catalog
                            .all()
                            .iter()
                            .filter(|c| c.x64_only && installed(&c.id))
                            .map(|c| c.name.clone()),
                    ),
                    id if installed(id) => needed_by.push(name_of(catalog, id)),
                    _ => {}
                }
            }
            let (present, detail) = match s.id.as_str() {
                "vcruntime" => {
                    let dll = crate::fsx::join_rel(kit.apps(), "tools/vcruntime/vcruntime140.dll");
                    (dll.is_file(), None)
                }
                "webview2" => match crate::os::webview::found(kit) {
                    Some(crate::os::webview::Runtime::System(v)) => (true, Some(v)),
                    Some(crate::os::webview::Runtime::Fixed { version, .. }) => {
                        (true, Some(version))
                    }
                    None => (false, None),
                },
                "x64-emulation" => {
                    let build = crate::os::windows_build();
                    (
                        build.is_none_or(|b| b >= 22000),
                        build.map(|b| format!("Windows build {b}")),
                    )
                }
                _ => (false, None),
            };
            RuntimeState {
                id: s.id.clone(),
                name: s.name.clone(),
                note: s.note.as_deref().map(|n| crate::i18n::t(lang, n, &[])),
                needed_by,
                present,
                detail,
            }
        })
        .collect()
}
