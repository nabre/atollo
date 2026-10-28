use super::*;
use crate::kit::tests::TempDir;

#[test]
fn declared_versions() {
    let tmp = TempDir::new("relations-declared");
    let dir = &tmp.0;
    let spec = |component: &str, file: &str| {
        known()
            .projects
            .iter()
            .find(|p| p.component == component && p.file == file)
            .unwrap()
    };
    fs::write(
        dir.join("pyproject.toml"),
        "[project]\nname='x'\nrequires-python = \">=3.10,<3.13\"\n",
    )
    .unwrap();
    assert_eq!(
        declared(dir, "pyproject.toml", spec("python", "pyproject.toml")).as_deref(),
        Some(">=3.10,<3.13")
    );
    fs::write(dir.join("composer.json"), r#"{"require": {"php": "^8.2"}}"#).unwrap();
    assert_eq!(
        declared(dir, "composer.json", spec("php", "composer.json")).as_deref(),
        Some("^8.2")
    );
    fs::write(dir.join(".python-version"), "# pinned\n3.12.4\n").unwrap();
    assert_eq!(
        declared(dir, ".python-version", spec("python", ".python-version")).as_deref(),
        Some("3.12.4")
    );
    fs::write(
        dir.join("CMakeLists.txt"),
        "cmake_minimum_required(VERSION 3.20...3.31)\n",
    )
    .unwrap();
    assert_eq!(
        declared(dir, "CMakeLists.txt", spec("cmake", "CMakeLists.txt")).as_deref(),
        Some("3.20")
    );
    fs::write(
        dir.join("pubspec.yaml"),
        "name: x\nenvironment:\n  sdk: ^3.5.0\n",
    )
    .unwrap();
    assert_eq!(
        declared(dir, "pubspec.yaml", spec("dart", "pubspec.yaml")).as_deref(),
        Some("^3.5.0")
    );
    fs::write(
        dir.join("Project.toml"),
        "name = \"X\"\n\n[compat]\njulia = \"1.10\"\n",
    )
    .unwrap();
    assert_eq!(
        declared(dir, "Project.toml", spec("julia", "Project.toml")).as_deref(),
        Some("1.10")
    );
    fs::write(
        dir.join("renv.lock"),
        r#"{"R": {"Version": "4.4.1"}, "Packages": {}}"#,
    )
    .unwrap();
    assert_eq!(
        declared(dir, "renv.lock", spec("r", "renv.lock")).as_deref(),
        Some("4.4.1")
    );
    fs::write(
        dir.join("DESCRIPTION"),
        "Package: x\nDepends: R (>= 4.1.0), utils\n",
    )
    .unwrap();
    assert_eq!(
        declared(dir, "DESCRIPTION", spec("r", "DESCRIPTION")).as_deref(),
        Some("4.1.0")
    );
    fs::write(
        dir.join("typst.toml"),
        "[package]\nname = \"x\"\ncompiler = \"0.13.0\"\n",
    )
    .unwrap();
    assert_eq!(
        declared(dir, "typst.toml", spec("typst", "typst.toml")).as_deref(),
        Some("0.13.0")
    );
    assert_eq!(as_line("3.12.4"), "3.12");
    assert_eq!(as_line("v22"), "22");
    assert_eq!(as_line("8.0.302"), "8.0");
}

/// D-186: pin files at any depth; the shallowest says the version, and the
/// new ones (`rust-toolchain.toml`, `.tool-versions`) are read.
#[test]
fn the_shallowest_pin_wins() {
    let tmp = TempDir::new("relations-depth");
    let dir = &tmp.0;
    for (rel, text) in [
        ("web/app/.nvmrc", "18\n"),
        ("web/.nvmrc", "20.11.0\n"),
        (
            "tools/.tool-versions",
            "nodejs 22.1.0\npython 3.11.9\njava temurin-21.0.2\n",
        ),
        (
            "x/rust-toolchain.toml",
            "[toolchain]\nchannel = \"1.80.1\"\n",
        ),
    ] {
        let p = dir.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, text).unwrap();
    }
    let found: Vec<(String, String, String)> = declarations(dir, "shop")
        .into_iter()
        .map(|(s, rel, text)| (s.component.clone(), rel, text))
        .collect();
    let of = |component: &str, file: &str| {
        found
            .iter()
            .find(|(c, rel, _)| c == component && rel.ends_with(file))
            .map(|(_, rel, text)| (rel.as_str(), text.as_str()))
    };
    assert_eq!(of("node", ".nvmrc"), Some(("web/.nvmrc", "20.11.0")));
    assert_eq!(
        of("node", ".tool-versions"),
        Some(("tools/.tool-versions", "22.1.0"))
    );
    assert_eq!(
        of("python", ".tool-versions"),
        Some(("tools/.tool-versions", "3.11.9"))
    );
    assert_eq!(
        of("java", ".tool-versions"),
        Some(("tools/.tool-versions", "21"))
    );
    assert_eq!(
        of("rust", "rust-toolchain.toml"),
        Some(("x/rust-toolchain.toml", "1.80.1"))
    );
    // One depth: the knowledge's order; deeper files after.
    let depths: Vec<usize> = found
        .iter()
        .map(|(_, rel, _)| crate::detect::walk::depth(rel))
        .collect();
    assert!(depths.windows(2).all(|w| w[0] <= w[1]), "{depths:?}");
}
