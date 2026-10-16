use crate::detect::tests::{ids, project};
use crate::detect::{detect, Kind};

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
            (".venv/Scripts/python.exe", ""),
        ],
    );
    let p = detect(&fast.0, "it");
    assert_eq!(p.platform, "FastAPI");
    assert_eq!(
        p.actions[0].command,
        r".venv\Scripts\python.exe -m uvicorn app.main:api --host 127.0.0.1 --port {port} --reload"
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
