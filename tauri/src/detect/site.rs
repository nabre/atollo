//! Static site (HTML, CSS, JS only): served by the exe's file server.

use std::path::Path;

use super::{exists, tr, Action, Found, Kind};

pub(super) fn detect(dir: &Path, lang: &str) -> Option<Found> {
    let root = ["index.html", r"public\index.html", r"docs\index.html"]
        .into_iter()
        .find(|f| exists(dir, f))?;
    let folder = Path::new(root)
        .parent()
        .map(|p| p.to_string_lossy().into_owned())
        .filter(|p| !p.is_empty())
        .unwrap_or_else(|| ".".into());
    let shown = if folder == "." {
        tr(lang, "cartella del progetto", &[])
    } else {
        folder.clone()
    };
    Some(Found {
        platform: tr(lang, "Sito statico", &[]),
        kind: Kind::Web,
        reason: root.replace('\\', "/"),
        actions: vec![Action::serve(
            "static",
            &tr(lang, "sito statico ({0})", &[&shown]),
            &folder,
        )],
        warnings: Vec::new(),
    })
}
