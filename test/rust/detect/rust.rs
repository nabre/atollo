use crate::detect::tests::{ids, project};
use crate::detect::{detect, Kind};

#[test]
fn kinds() {
    let axum = project(
        "rs-axum",
        &[("Cargo.toml", "[dependencies]\naxum = \"0.8\"\n")],
    );
    let p = detect(&axum.0, "it");
    assert_eq!(p.platform, "Rust (axum)");
    assert_eq!(p.kind, Kind::Web);
    assert_eq!(ids(&p), ["cargo:run", "cargo:build", "cargo:test"]);

    let tauri = project(
        "rs-tauri",
        &[(
            "Cargo.toml",
            "[dependencies]\ntauri = { version = \"2\" }\n",
        )],
    );
    let p = detect(&tauri.0, "it");
    assert_eq!(p.kind, Kind::Desktop);
    assert_eq!(p.actions[0].command, "cargo tauri dev");
}
