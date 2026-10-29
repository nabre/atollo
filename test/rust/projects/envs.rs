use super::*;
use crate::kit::tests::TempDir;
use crate::kit::Target;
use std::cell::Cell;
use std::fs;

fn project(tmp: &TempDir, files: &[&str]) -> PathBuf {
    let dir = tmp.0.join("projects").join("shop");
    fs::create_dir_all(&dir).unwrap();
    for f in files {
        fs::write(dir.join(f), "").unwrap();
    }
    dir
}

fn venv(dir: &Path, name: &str) {
    fs::create_dir_all(dir.join(name)).unwrap();
    fs::write(dir.join(name).join("pyvenv.cfg"), "home = x\n").unwrap();
}

/// The PC's environment of the project in `dir`: in the kit's data.
fn pcv(kit: &Kit, dir: &Path) -> PathBuf {
    let v = pc_venv(kit, dir).unwrap();
    assert!(v.starts_with(kit.project_data()), "{}", v.display());
    v
}

fn env_with_path(dirs: &[PathBuf]) -> KitEnv {
    let mut env = KitEnv::default();
    let sep = env.path_sep().to_string();
    let path: Vec<String> = dirs.iter().map(|d| d.display().to_string()).collect();
    env.set("PATH", path.join(&sep));
    env
}

/// The tools come from the project's own files; Node and PHP have none.
#[test]
fn the_tools_come_from_the_projects_files() {
    let tmp = TempDir::new("envs-tools");
    let dir = project(
        &tmp,
        &[
            "requirements.txt",
            "Gemfile",
            "Cargo.toml",
            "Project.toml",
            "renv.lock",
        ],
    );
    assert_eq!(
        tools(&dir),
        [Tool::Python, Tool::Ruby, Tool::Rust, Tool::Julia, Tool::R]
    );
    let node = TempDir::new("envs-node");
    assert!(tools(&project(&node, &["package.json", "composer.json"])).is_empty());
}

/// On the PC the project's environment in the kit's data is uv's, and
/// first on the `PATH` with `VIRTUAL_ENV` only once it exists, `Scripts` on
/// Windows and `bin` elsewhere; applied again it changes nothing. A
/// `.venv` in the project's folder, which the agents' sandbox can write,
/// never reaches the `PATH`.
#[test]
fn the_venv_is_used_on_the_pc_only_when_it_exists() {
    let tmp = TempDir::new("envs-pc");
    let dir = project(&tmp, &["pyproject.toml"]);
    let tools_dir = tmp.0.join("tools");
    for target in [Target::WIN32_X64, Target::LINUX_X64] {
        let kit = Kit::at_target(tmp.0.clone(), target);
        let mut env = env_with_path(std::slice::from_ref(&tools_dir));
        apply_pc(&kit, &dir, &mut env);
        assert_eq!(env.get("VIRTUAL_ENV"), None, "{target:?}");
        assert_eq!(env.path_entries(), [tools_dir.display().to_string()]);
        let v = pcv(&kit, &dir);
        assert_eq!(
            env.get("UV_PROJECT_ENVIRONMENT"),
            Some(v.display().to_string()).as_deref()
        );
    }
    // Written by an agent in the sandbox: not the PC's.
    venv(&dir, ".venv");
    for target in [Target::WIN32_X64, Target::LINUX_X64] {
        let kit = Kit::at_target(tmp.0.clone(), target);
        let mut env = env_with_path(std::slice::from_ref(&tools_dir));
        apply_pc(&kit, &dir, &mut env);
        assert_eq!(env.path_entries(), [tools_dir.display().to_string()]);
        assert_eq!(env.get("VIRTUAL_ENV"), None);
    }
    for (target, bin) in [(Target::WIN32_X64, "Scripts"), (Target::LINUX_X64, "bin")] {
        let kit = Kit::at_target(tmp.0.clone(), target);
        let v = pcv(&kit, &dir);
        venv(v.parent().unwrap(), VENV);
        let mut env = env_with_path(std::slice::from_ref(&tools_dir));
        apply_pc(&kit, &dir, &mut env);
        let first = v.join(bin).display().to_string();
        assert_eq!(env.path_entries()[0], first, "{target:?}");
        assert_eq!(
            env.get("VIRTUAL_ENV"),
            Some(v.display().to_string()).as_deref()
        );
        let once = env.get("PATH").unwrap().to_owned();
        apply_pc(&kit, &dir, &mut env);
        assert_eq!(env.get("PATH"), Some(once.as_str()), "idempotent");
    }
}

/// Ruby's gems and Julia's project on the PC; R's cache in the kit.
#[test]
fn ruby_julia_and_r_get_their_variables_on_the_pc() {
    let tmp = TempDir::new("envs-vars");
    let dir = project(&tmp, &["Gemfile", "Project.toml", "renv.lock"]);
    let kit = Kit::at_target(tmp.0.clone(), Target::LINUX_X64);
    let mut env = KitEnv::default();
    apply_pc(&kit, &dir, &mut env);
    assert_eq!(
        env.get("BUNDLE_PATH"),
        Some(dir.join("vendor").join("bundle").display().to_string()).as_deref()
    );
    assert_eq!(env.get("JULIA_PROJECT"), Some("@."));
    assert!(env
        .get("RENV_PATHS_CACHE")
        .unwrap()
        .starts_with(&kit.data().display().to_string()));
}

/// In the VM every path is under the project's own folder, the Linux
/// names apart from the PC's; nothing for a project without them.
#[test]
fn the_vm_gets_linux_names_under_the_project_only() {
    let tmp = TempDir::new("envs-vm");
    let dir = project(&tmp, &["uv.lock", "Gemfile", "Cargo.toml", "renv.lock"]);
    let guest = "/d/kit/projects/shop";
    let vars = vm_vars(&dir, guest);
    let get = |n: &str| vars.iter().find(|(k, _)| k == n).map(|(_, v)| v.as_str());
    assert_eq!(
        get("UV_PROJECT_ENVIRONMENT"),
        Some("/d/kit/projects/shop/.venv-linux")
    );
    assert_eq!(get(VENV_VAR), Some("/d/kit/projects/shop/.venv-linux"));
    assert_eq!(
        get("BUNDLE_PATH"),
        Some("/d/kit/projects/shop/vendor/bundle-linux")
    );
    assert_eq!(
        get("CARGO_TARGET_DIR"),
        Some("/d/kit/projects/shop/target/linux")
    );
    assert_eq!(
        get("RENV_PATHS_CACHE"),
        None,
        "renv's cache stays in the VM's home"
    );
    for (name, value) in &vars {
        assert!(value.starts_with(guest) || value == "@.", "{name}={value}");
    }
    let other = TempDir::new("envs-vm-none");
    assert!(vm_vars(&project(&other, &["package.json"]), guest).is_empty());
    // The session makes and uses the Linux venv by itself.
    assert!(crate::isolation::vm::session_start().contains(VENV_VAR));
}

/// `uv venv` first, Python's own `venv` without uv, nothing without
/// either; never `uv sync` (it would run the packages' code).
#[test]
fn the_venv_is_made_by_uv_or_python() {
    let tmp = TempDir::new("envs-cmd");
    let (uv_dir, py_dir) = (tmp.0.join("uv"), tmp.0.join("python"));
    fs::create_dir_all(&uv_dir).unwrap();
    fs::create_dir_all(&py_dir).unwrap();
    for windows in [true, false] {
        let exe = |n: &str| {
            if windows {
                format!("{n}.exe")
            } else {
                n.to_owned()
            }
        };
        fs::write(uv_dir.join(exe("uv")), "").unwrap();
        fs::write(py_dir.join(exe("python")), "").unwrap();
        let at = tmp.0.join("data").join("venv");
        let shown = at.display().to_string();
        let both = env_with_path(&[py_dir.clone(), uv_dir.clone()]);
        let (program, args) = create_command(&both, windows, &at).unwrap();
        assert_eq!(program, uv_dir.join(exe("uv")));
        assert_eq!(args, ["venv", shown.as_str()]);
        let python = env_with_path(std::slice::from_ref(&py_dir));
        let (program, args) = create_command(&python, windows, &at).unwrap();
        assert_eq!(program, py_dir.join(exe("python")));
        assert_eq!(args, ["-m", "venv", shown.as_str()]);
        assert!(create_command(&KitEnv::default(), windows, &at).is_none());
        fs::remove_file(uv_dir.join(exe("uv"))).unwrap();
        fs::remove_file(py_dir.join(exe("python"))).unwrap();
    }
}

/// Made once, in `.gitignore` of a Git repository; a failure is kept for
/// the project's page; a folder that is not Python's is left alone.
#[test]
fn ensure_makes_the_venv_once() {
    let tmp = TempDir::new("envs-ensure");
    let dir = project(&tmp, &["requirements.txt"]);
    fs::create_dir_all(dir.join(".git")).unwrap();
    let uv_dir = tmp.0.join("uv");
    fs::create_dir_all(&uv_dir).unwrap();
    let kit = Kit::at_target(tmp.0.clone(), Target::LINUX_X64);
    fs::write(uv_dir.join("uv"), "").unwrap();
    let env = env_with_path(std::slice::from_ref(&uv_dir));
    let runs = Cell::new(0);
    let v = pcv(&kit, &dir);
    let make = |_: &Path, args: &[String], _: &Path, _: &KitEnv| {
        runs.set(runs.get() + 1);
        assert_eq!(args, ["venv".to_owned(), v.display().to_string()]);
        venv(v.parent().unwrap(), VENV);
        Ok(())
    };
    assert_eq!(status(&kit, &dir), Some(LocalEnv::Missing));
    assert_eq!(ensure_pc(&kit, &dir, &env, &make), Some(LocalEnv::Ready));
    assert_eq!(ensure_pc(&kit, &dir, &env, &make), Some(LocalEnv::Ready));
    assert_eq!(runs.get(), 1, "made once");
    let ignore = fs::read_to_string(dir.join(".gitignore")).unwrap();
    assert_eq!(ignore, "/.venv-linux/\n");
    assert!(!dir.join(".venv").exists(), "never in the project's folder");

    let broken = TempDir::new("envs-ensure-fail");
    let dir = project(&broken, &["pyproject.toml"]);
    let kit = Kit::at_target(broken.0.clone(), Target::LINUX_X64);
    let fail = |_: &Path, _: &[String], _: &Path, _: &KitEnv| Err("no Python 3.13".to_owned());
    let failed = LocalEnv::Failed {
        reason: "no Python 3.13".into(),
    };
    assert_eq!(ensure_pc(&kit, &dir, &env, &fail), Some(failed.clone()));
    assert_eq!(status(&kit, &dir), Some(failed));
    assert!(!dir.join(".gitignore").exists(), "not a Git repository");

    let node = TempDir::new("envs-ensure-node");
    let dir = project(&node, &["package.json"]);
    let never = |_: &Path, _: &[String], _: &Path, _: &KitEnv| -> Result<(), String> {
        panic!("not a Python project")
    };
    assert_eq!(ensure_pc(&kit, &dir, &env, &never), None);
}

/// `.gitignore` keeps its lines and its line ends; a name already there,
/// with or without slashes, is not added again.
#[test]
fn gitignore_gets_only_what_it_lacks() {
    let tmp = TempDir::new("envs-ignore");
    let dir = project(&tmp, &[]);
    fs::create_dir_all(dir.join(".git")).unwrap();
    fs::write(dir.join(".gitignore"), "node_modules\r\n.venv").unwrap();
    ignore(&dir, &[".venv", VENV_LINUX]).unwrap();
    assert_eq!(
        fs::read_to_string(dir.join(".gitignore")).unwrap(),
        "node_modules\r\n.venv\r\n/.venv-linux/\r\n"
    );
    ignore(&dir, &[".venv", VENV_LINUX]).unwrap();
    assert_eq!(
        fs::read_to_string(dir.join(".gitignore")).unwrap(),
        "node_modules\r\n.venv\r\n/.venv-linux/\r\n"
    );
}
