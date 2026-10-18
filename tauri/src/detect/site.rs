//! Static site (HTML, CSS, JS only): served by the exe's file server.

use std::path::Path;

use super::{exists, tr, Action, Found, Kind};

pub(super) fn detect(dir: &Path, lang: &str) -> Option<Found> {
    let root = ["index.html", r"public\index.html", r"docs\index.html"]
        .into_iter()
        .find(|f| exists(dir, f))?;
    // Split on the `\` the candidates are written with: `Path::parent`
    // does not see it as a separator on macOS and Linux.
    let folder = root
        .rsplit_once('\\')
        .map_or_else(|| ".".to_owned(), |(d, _)| d.to_owned());
    let shown = if folder == "." {
        tr(lang, "cartella del progetto", &[])
    } else {
        folder.clone()
    };
    let actions = vec![Action::serve(
        "static",
        &tr(lang, "sito statico ({0})", &[&shown]),
        &folder,
    )];
    Some(Found {
        platform: tr(lang, "Sito statico", &[]),
        kind: Kind::Web,
        reason: root.replace('\\', "/"),
        actions,
        warnings: Vec::new(),
    })
}
