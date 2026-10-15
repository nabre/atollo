//! Go (installed with the kit when chosen in Linguaggi e strumenti).

use std::path::Path;

use super::{exists, read, tr, Action, Found, Kind};

pub(super) fn detect(dir: &Path, lang: &str) -> Option<Found> {
    if !exists(dir, "go.mod") {
        return None;
    }
    let module = read(dir, "go.mod");
    let web = [
        "gin-gonic",
        "labstack/echo",
        "gofiber",
        "go-chi",
        "gorilla/mux",
    ]
    .iter()
    .any(|n| module.contains(n))
        || read(dir, "main.go").contains("net/http");
    let kind = if web { Kind::Web } else { Kind::Command };
    let mut warnings = Vec::new();
    if web {
        warnings.push(tr(
            lang,
            "Il programma deve leggere la porta dalla variabile PORT.",
            &[],
        ));
    }
    Some(Found {
        platform: "Go".into(),
        kind,
        reason: "go.mod".into(),
        actions: vec![Action::new("go:run", "go run .", "go run .", kind)],
        warnings,
    })
}
