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
        declared(dir, spec("python", "pyproject.toml")).as_deref(),
        Some(">=3.10,<3.13")
    );
    fs::write(dir.join("composer.json"), r#"{"require": {"php": "^8.2"}}"#).unwrap();
    assert_eq!(
        declared(dir, spec("php", "composer.json")).as_deref(),
        Some("^8.2")
    );
    fs::write(dir.join(".python-version"), "# pinned\n3.12.4\n").unwrap();
    assert_eq!(
        declared(dir, spec("python", ".python-version")).as_deref(),
        Some("3.12.4")
    );
    fs::write(
        dir.join("CMakeLists.txt"),
        "cmake_minimum_required(VERSION 3.20...3.31)\n",
    )
    .unwrap();
    assert_eq!(
        declared(dir, spec("cmake", "CMakeLists.txt")).as_deref(),
        Some("3.20")
    );
    fs::write(
        dir.join("pubspec.yaml"),
        "name: x\nenvironment:\n  sdk: ^3.5.0\n",
    )
    .unwrap();
    assert_eq!(
        declared(dir, spec("dart", "pubspec.yaml")).as_deref(),
        Some("^3.5.0")
    );
    fs::write(
        dir.join("Project.toml"),
        "name = \"X\"\n\n[compat]\njulia = \"1.10\"\n",
    )
    .unwrap();
    assert_eq!(
        declared(dir, spec("julia", "Project.toml")).as_deref(),
        Some("1.10")
    );
    fs::write(
        dir.join("renv.lock"),
        r#"{"R": {"Version": "4.4.1"}, "Packages": {}}"#,
    )
    .unwrap();
    assert_eq!(
        declared(dir, spec("r", "renv.lock")).as_deref(),
        Some("4.4.1")
    );
    fs::write(
        dir.join("DESCRIPTION"),
        "Package: x\nDepends: R (>= 4.1.0), utils\n",
    )
    .unwrap();
    assert_eq!(
        declared(dir, spec("r", "DESCRIPTION")).as_deref(),
        Some("4.1.0")
    );
    fs::write(
        dir.join("typst.toml"),
        "[package]\nname = \"x\"\ncompiler = \"0.13.0\"\n",
    )
    .unwrap();
    assert_eq!(
        declared(dir, spec("typst", "typst.toml")).as_deref(),
        Some("0.13.0")
    );
    assert_eq!(as_line("3.12.4"), "3.12");
    assert_eq!(as_line("v22"), "22");
    assert_eq!(as_line("8.0.302"), "8.0");
}
