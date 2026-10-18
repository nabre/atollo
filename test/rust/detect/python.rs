use super::{has_venv, venv_python};
use crate::detect::tests::{ids, project};
use crate::detect::{detect, Kind};
use crate::kit::Os;

#[test]
fn venv_python_per_os() {
    assert_eq!(venv_python(Os::Win32, ".venv"), r".venv\Scripts\python.exe");
    assert_eq!(venv_python(Os::Linux, ".venv"), ".venv/bin/python");
    assert_eq!(venv_python(Os::Darwin, "venv"), "venv/bin/python");
}

/// The venv of each system is found by its own layout, and only that one.
#[test]
fn has_venv_per_os() {
    let win = project("venv-win", &[(".venv/Scripts/python.exe", "")]);
    let unix = project(
        "venv-unix",
        &[(".venv/bin/python", ""), (".venv/pyvenv.cfg", "")],
    );
    let empty_bin = project("venv-nobin", &[(".venv/bin/pip", "")]);
    assert!(has_venv(&win.0, Os::Win32, ".venv"));
    assert!(!has_venv(&win.0, Os::Linux, ".venv"));
    for os in [Os::Linux, Os::Darwin] {
        assert!(has_venv(&unix.0, os, ".venv"));
        assert!(!has_venv(&unix.0, os, "venv"));
        assert!(!has_venv(&empty_bin.0, os, ".venv"));
    }
    assert!(!has_venv(&unix.0, Os::Win32, ".venv"));
}

/// A real venv of macOS and Linux: `bin/python` is a link to a Python
/// outside the project. It is found, and the run command uses it.
#[cfg(unix)]
#[test]
fn venv_with_a_linked_python_is_found() {
    let p = project(
        "venv-link",
        &[
            (".venv/pyvenv.cfg", "home = /usr/bin\n"),
            ("requirements.txt", "flask\n"),
            ("app.py", "from flask import Flask\napp = Flask(__name__)\n"),
        ],
    );
    let bin = p.0.join(".venv").join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    std::os::unix::fs::symlink("/usr/bin/python3", bin.join("python")).unwrap();
    assert!(has_venv(&p.0, Os::current(), ".venv"));
    let found = detect(&p.0, "it");
    let run = found.actions.iter().find(|a| a.id == "py:flask").unwrap();
    assert!(
        run.command.starts_with(".venv/bin/python -m flask"),
        "{}",
        run.command
    );
    assert!(found.warnings.is_empty(), "{:?}", found.warnings);
    // `bin` itself a link (to a folder outside): never trusted.
    let q = project("venv-binlink", &[(".venv/pyvenv.cfg", "")]);
    std::os::unix::fs::symlink("/usr/bin", q.0.join(".venv").join("bin")).unwrap();
    assert!(!has_venv(&q.0, Os::current(), ".venv"));
}

#[test]
fn venv_action_uses_the_python_of_the_system() {
    let req = project(
        "py-req",
        &[("requirements.txt", "requests\n"), ("tool.py", "")],
    );
    let p = detect(&req.0, "it");
    let venv = p.actions.iter().find(|a| a.id == "py:venv").unwrap();
    assert_eq!(
        venv.command,
        format!(
            "python -m venv .venv && {} -m pip install -r requirements.txt",
            venv_python(Os::current(), ".venv")
        )
    );
    if cfg!(windows) {
        assert_eq!(
            venv.command,
            r"python -m venv .venv && .venv\Scripts\python.exe -m pip install -r requirements.txt",
            "unchanged on Windows"
        );
    }
}

#[test]
fn frameworks() {
    let fast = project(
        "py-fastapi",
        &[
            (
                "pyproject.toml",
                "[project]\ndependencies = [\"fastapi\"]\n",
            ),
            (
                "app/main.py",
                "from fastapi import FastAPI\napi: FastAPI = FastAPI()\n",
            ),
            (&venv_python(Os::current(), ".venv").replace('\\', "/"), ""),
        ],
    );
    let p = detect(&fast.0, "it");
    assert_eq!(p.platform, "FastAPI");
    let py = if cfg!(windows) {
        r".venv\Scripts\python.exe"
    } else {
        ".venv/bin/python"
    };
    assert_eq!(
        p.actions[0].command,
        format!("{py} -m uvicorn app.main:api --host 127.0.0.1 --port {{port}} --reload")
    );
    assert!(ids(&p).contains(&"uv:sync"));
    assert!(p.warnings.is_empty());

    let dj = project(
        "py-django",
        &[("manage.py", ""), ("requirements.txt", "django\n")],
    );
    let p = detect(&dj.0, "it");
    assert_eq!(p.platform, "Django");
    assert!(p.warnings.iter().any(|w| w.contains(".venv")));

    let st = project(
        "py-streamlit",
        &[
            ("requirements.txt", "streamlit\n"),
            ("streamlit_app.py", "import streamlit as st\n"),
        ],
    );
    assert_eq!(detect(&st.0, "it").platform, "Streamlit");

    let script = project(
        "py-script",
        &[("tool.py", "print(1)\n"), ("tests/test_x.py", "")],
    );
    let p = detect(&script.0, "it");
    assert_eq!(p.kind, Kind::Command);
    assert_eq!(ids(&p), ["py:run", "py:test"]);
}
