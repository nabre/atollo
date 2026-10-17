use super::venv_python;
use crate::detect::tests::{ids, project};
use crate::detect::{detect, Kind};
use crate::kit::Os;

#[test]
fn venv_python_per_os() {
    assert_eq!(venv_python(Os::Win32, ".venv"), r".venv\Scripts\python.exe");
    assert_eq!(venv_python(Os::Linux, ".venv"), ".venv/bin/python");
    assert_eq!(venv_python(Os::Darwin, "venv"), "venv/bin/python");
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
