//! Rust: Tauri (without package.json), Trunk (WebAssembly), web servers,
//! programs.

use std::path::Path;

use regex::Regex;

use super::{exists, read, tr, Action, Found, Kind};

fn dep(cargo: &str, name: &str) -> bool {
    Regex::new(&format!(r"(?m)^\s*{}\s*=", regex::escape(name))).is_ok_and(|re| re.is_match(cargo))
}

pub(super) fn detect(dir: &Path, lang: &str) -> Option<Found> {
    if !exists(dir, "Cargo.toml") {
        return None;
    }
    let cargo = read(dir, "Cargo.toml").to_ascii_lowercase();
    let mut actions = Vec::new();
    let mut warnings = Vec::new();
    let (platform, kind, reason);

    if dep(&cargo, "tauri") {
        (platform, kind) = ("Tauri".to_owned(), Kind::Desktop);
        reason = tr(lang, "{0} con {1}", &["Cargo.toml", "tauri"]);
        let root = if dir.ends_with("src-tauri") {
            dir.parent().unwrap_or(dir)
        } else {
            dir
        };
        let (a, w) = super::node::tauri(root, false, lang);
        actions.extend(a);
        warnings.extend(w);
    } else if exists(dir, "Trunk.toml")
        || (exists(dir, "index.html")
            && ["yew", "leptos", "dioxus", "sycamore"]
                .iter()
                .any(|n| dep(&cargo, n)))
    {
        (platform, kind) = ("Rust + WebAssembly (Trunk)".to_owned(), Kind::Web);
        reason = "Trunk.toml / index.html".to_owned();
        actions.push(Action::new(
            "trunk:serve",
            "trunk serve",
            "trunk serve --port {port} --address 127.0.0.1",
            Kind::Web,
        ));
        warnings.push(tr(
            lang,
            "Serve Trunk: cargo install trunk  e  rustup target add wasm32-unknown-unknown",
            &[],
        ));
    } else if let Some(fw) = [
        "axum",
        "actix-web",
        "rocket",
        "warp",
        "poem",
        "tide",
        "salvo",
        "hyper",
    ]
    .into_iter()
    .find(|n| dep(&cargo, n))
    {
        (platform, kind) = (format!("Rust ({fw})"), Kind::Web);
        reason = tr(lang, "{0} con {1}", &["Cargo.toml", fw]);
        actions.push(Action::new(
            "cargo:run",
            "cargo run",
            "cargo run",
            Kind::Web,
        ));
        warnings.push(tr(
            lang,
            "Il programma deve leggere la porta dalla variabile PORT (std::env::var(\"PORT\")). Rocket usa ROCKET_PORT, già impostata.",
            &[],
        ));
    } else {
        (platform, kind, reason) = ("Rust".to_owned(), Kind::Command, "Cargo.toml".to_owned());
        actions.push(Action::new(
            "cargo:run",
            "cargo run",
            "cargo run",
            Kind::Command,
        ));
    }
    if kind != Kind::Desktop {
        actions.push(Action::new(
            "cargo:build",
            "cargo build --release",
            "cargo build --release",
            Kind::Command,
        ));
        actions.push(Action::new(
            "cargo:test",
            "cargo test",
            "cargo test",
            Kind::Command,
        ));
    }
    Some(Found {
        platform,
        kind,
        reason,
        actions,
        warnings,
    })
}

#[cfg(test)]
#[path = "../../../test/rust/detect/rust.rs"]
mod tests;
