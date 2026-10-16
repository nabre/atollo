//! Static site (HTML, CSS, JS only): served by the exe's file server, and by
//! a Nginx of its own when the project has a `nginx.conf` or `.nginx`.

use std::path::Path;

use super::{exists, has_nginx, tr, Action, Found, Kind};

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
    let mut actions = vec![Action::serve(
        "static",
        &tr(lang, "sito statico ({0})", &[&shown]),
        &folder,
    )];
    if has_nginx(dir, &folder) {
        actions.push(Action::nginx(
            "nginx",
            &tr(lang, "sito web Nginx ({0})", &[&shown]),
            &folder,
        ));
    }
    Some(Found {
        platform: tr(lang, "Sito statico", &[]),
        kind: Kind::Web,
        reason: root.replace('\\', "/"),
        actions,
        warnings: Vec::new(),
    })
}
