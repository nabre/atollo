//! Python: Django, Flask, FastAPI, Streamlit, Gradio, scripts.

use std::path::Path;
use std::sync::LazyLock;

use regex::Regex;

use super::{exists, list_dir, read, tr, Action, Found, Kind};

static FASTAPI: LazyLock<Option<Regex>> = LazyLock::new(|| Regex::new(r"FastAPI\s*\(").ok());
static FASTAPI_VAR: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"(?m)^(\w+)\s*(?::[^=]+)?=\s*FastAPI\s*\(").ok());
static FLASK: LazyLock<Option<Regex>> = LazyLock::new(|| Regex::new(r"Flask\s*\(").ok());
static STREAMLIT: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"import\s+streamlit").ok());
static GRADIO: LazyLock<Option<Regex>> = LazyLock::new(|| Regex::new(r"import\s+gradio").ok());

const CANDIDATES: [&str; 9] = [
    "app.py",
    "main.py",
    "server.py",
    "wsgi.py",
    "application.py",
    "app/__init__.py",
    "app/main.py",
    "src/main.py",
    "src/app.py",
];

/// The project's Python: its virtual environment, uv, or the kit's.
fn interpreter(dir: &Path) -> (String, bool) {
    for v in [".venv", "venv", "env"] {
        if exists(dir, &format!(r"{v}\Scripts\python.exe")) {
            return (format!(r"{v}\Scripts\python.exe"), true);
        }
    }
    if exists(dir, "uv.lock") {
        return ("uv run python".into(), true);
    }
    ("python".into(), false)
}

/// First candidate file whose text matches.
fn find_file(dir: &Path, candidates: &[&str], re: &Option<Regex>) -> Option<(String, String)> {
    let re = re.as_ref()?;
    candidates.iter().find_map(|c| {
        let text = read(dir, &c.replace('/', "\\"));
        (!text.is_empty() && re.is_match(&text)).then(|| ((*c).to_owned(), text))
    })
}

fn module_name(file: &str) -> String {
    file.trim_end_matches(".py")
        .trim_end_matches("/__init__")
        .replace('/', ".")
}

pub(super) fn detect(dir: &Path, lang: &str) -> Option<Found> {
    let deps = format!(
        "{}\n{}\n{}",
        read(dir, "requirements.txt"),
        read(dir, "pyproject.toml"),
        read(dir, "Pipfile")
    )
    .to_ascii_lowercase();
    let py_files: Vec<String> = list_dir(dir)
        .into_iter()
        .filter(|f| f.ends_with(".py"))
        .collect();
    if deps.trim().is_empty() && py_files.is_empty() {
        return None;
    }
    let (py, has_env) = interpreter(dir);
    let uses = |n: &str| {
        Regex::new(&format!(
            r"(?m)(^|[^a-z0-9_-]){}([^a-z0-9_-]|$)",
            regex::escape(n)
        ))
        .is_ok_and(|re| re.is_match(&deps))
    };
    let mut actions = Vec::new();
    let mut warnings = Vec::new();
    let mut platform = "Python".to_owned();
    let mut kind = Kind::Command;
    let mut reason = if py_files.is_empty() {
        tr(lang, "dipendenze Python", &[])
    } else {
        tr(lang, "file {0}", &[".py"])
    };
    let mut with_streamlit = vec!["streamlit_app.py"];
    with_streamlit.extend(CANDIDATES);

    if exists(dir, "manage.py") {
        (platform, kind, reason) = ("Django".into(), Kind::Web, "manage.py".into());
        actions.push(Action::new(
            "py:django",
            "manage.py runserver",
            &format!("{py} manage.py runserver 127.0.0.1:{{port}}"),
            Kind::Web,
        ));
    } else if let Some((file, _)) = uses("streamlit")
        .then(|| find_file(dir, &with_streamlit, &STREAMLIT))
        .flatten()
    {
        (platform, kind) = ("Streamlit".into(), Kind::Web);
        reason = tr(lang, "{0} importa {1}", &[&file, "streamlit"]);
        actions.push(Action::new(
            "py:streamlit",
            &format!("streamlit run {file}"),
            &format!("{py} -m streamlit run {file} --server.port {{port}} --server.address 127.0.0.1 --server.headless true"),
            Kind::Web,
        ));
    } else if let Some((file, text)) = find_file(dir, &CANDIDATES, &FASTAPI) {
        let var = FASTAPI_VAR
            .as_ref()
            .and_then(|re| re.captures(&text))
            .and_then(|c| c.get(1))
            .map_or("app", |m| m.as_str());
        (platform, kind) = ("FastAPI".into(), Kind::Web);
        reason = tr(lang, "{0} crea {1}", &[&file, "FastAPI()"]);
        let module = module_name(&file);
        actions.push(Action::new(
            "py:uvicorn",
            &format!("uvicorn {module}:{var}"),
            &format!("{py} -m uvicorn {module}:{var} --host 127.0.0.1 --port {{port}} --reload"),
            Kind::Web,
        ));
    } else if let Some((file, _)) = find_file(dir, &CANDIDATES, &FLASK) {
        (platform, kind) = ("Flask".into(), Kind::Web);
        reason = tr(lang, "{0} crea {1}", &[&file, "Flask()"]);
        let module = module_name(&file);
        actions.push(Action::new(
            "py:flask",
            &format!("flask --app {module} run"),
            &format!("{py} -m flask --app {module} run --host 127.0.0.1 --port {{port}} --debug"),
            Kind::Web,
        ));
    } else if let Some((file, _)) = uses("gradio")
        .then(|| find_file(dir, &CANDIDATES, &GRADIO))
        .flatten()
    {
        (platform, kind) = ("Gradio".into(), Kind::Web);
        reason = tr(lang, "{0} importa {1}", &[&file, "gradio"]);
        let mut a = Action::new(
            "py:gradio",
            &format!("python {file}"),
            &format!("{py} {file}"),
            Kind::Web,
        );
        a.env
            .insert("GRADIO_SERVER_NAME".into(), "127.0.0.1".into());
        a.env.insert("GRADIO_SERVER_PORT".into(), "{port}".into());
        actions.push(a);
    } else if let Some(main) = ["main.py", "app.py", "__main__.py", "run.py"]
        .iter()
        .find(|f| py_files.iter().any(|p| p == *f))
        .map(|s| (*s).to_owned())
        .or_else(|| py_files.first().cloned())
    {
        actions.push(Action::new(
            "py:run",
            &format!("python {main}"),
            &format!("{py} {main}"),
            Kind::Command,
        ));
    }

    if exists(dir, "pyproject.toml") {
        actions.push(Action::new("uv:sync", "uv sync", "uv sync", Kind::Command));
    }
    if exists(dir, "requirements.txt") {
        actions.push(Action::new(
            "py:venv",
            &tr(lang, "crea .venv e installa requirements.txt", &[]),
            r"python -m venv .venv && .venv\Scripts\python.exe -m pip install -r requirements.txt",
            Kind::Command,
        ));
    }
    if exists(dir, "tests") || exists(dir, "pytest.ini") {
        actions.push(Action::new(
            "py:test",
            "pytest",
            &format!("{py} -m pytest"),
            Kind::Command,
        ));
    }
    if !has_env && kind == Kind::Web {
        warnings.push(tr(
            lang,
            "Nessun ambiente virtuale (.venv): le dipendenze devono essere installate nel Python del kit, oppure crea .venv.",
            &[],
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
#[path = "../../../test/rust/detect/python.rs"]
mod tests;
