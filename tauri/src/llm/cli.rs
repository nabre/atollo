//! `models` of the command line: the catalog's models and the others on
//! disk, a download (`models pull <id|owner/repo[:quant]>`), resumed when
//! started again after an interruption, and `models rm <id>`.

use std::io::{self, Write};
use std::sync::atomic::AtomicBool;

use anyhow::Result;

use super::pull::{gb, Plan};
use super::{catalog, hf, local};
use crate::i18n::t;
use crate::kit::Kit;
use crate::net::Net;

/// `[x]` downloaded, `[~]` begun (with how much), `[ ]` not there.
fn mark(m: Option<&local::LocalModel>) -> String {
    match m {
        Some(m) if m.complete => "[x]".into(),
        Some(m) if m.bytes > 0 => format!("[~ {}%]", m.downloaded * 100 / m.bytes),
        Some(_) => "[~]".into(),
        None => "[ ]".into(),
    }
}

pub fn list(kit: &Kit, lang: &str, out: &mut dyn Write) -> io::Result<()> {
    let here = local::list(kit);
    let find = |id: &str| {
        here.iter()
            .find(|m| m.catalog.as_deref() == Some(id) || m.id == id)
    };
    writeln!(out, "{}", t(lang, "Catalogo di Atollo:", &[]))?;
    for m in catalog::all() {
        writeln!(
            out,
            "  {:<8} {:<22} {:<22} {:>6} GB  {:<7} {}",
            mark(find(&m.id)),
            m.id,
            m.name,
            gb(m.bytes()),
            m.quant,
            t(lang, &m.desc, &[])
        )?;
    }
    let others: Vec<&local::LocalModel> = here
        .iter()
        .filter(|m| {
            m.catalog
                .as_ref()
                .is_none_or(|c| catalog::find(c).is_none())
        })
        .filter(|m| catalog::find(&m.id).is_none())
        .collect();
    if !others.is_empty() {
        writeln!(out, "\n{}", t(lang, "Altri modelli nel kit:", &[]))?;
        for m in others {
            let source = if m.repo.is_empty() {
                m.folder.clone()
            } else {
                m.repo.clone()
            };
            writeln!(
                out,
                "  {:<8} {:<40} {:>6} GB  {source}",
                mark(Some(m)),
                m.id,
                gb(m.bytes)
            )?;
        }
    }
    writeln!(
        out,
        "\n{}",
        t(
            lang,
            "Cartella dei modelli: {0}",
            &[&super::models_dir(kit).display().to_string()]
        )
    )?;
    out.flush()
}

/// The plan of `spec`: a catalog id, or `owner/repo[:quant]` read from
/// Hugging Face (the quantization, or the id the model has in the kit).
pub fn plan_of(kit: &Kit, net: &Net, spec: &str, token: Option<&str>) -> Result<Plan> {
    if let Some(m) = catalog::find(spec.trim()) {
        return Ok(Plan::from_catalog(m));
    }
    let (repo, quant) = hf::parse_spec(spec)?;
    let info = hf::inspect(net, &repo, token)?;
    match quant {
        Some(q) => Plan::of_choice(kit, &info, &q),
        None => Ok(Plan::from_choice(&info, hf::pick(&info, None)?)),
    }
}

pub fn pull(kit: &Kit, net: &Net, spec: &str, lang: &str, out: &mut dyn Write) -> Result<String> {
    // A catalog model is public: a token that cannot be read does not stop it.
    let token = if catalog::find(spec.trim()).is_some() {
        super::hf_token_if_readable(kit)
    } else {
        super::hf_token(kit)?
    };
    let plan = plan_of(kit, net, spec, token.as_deref())?;
    writeln!(
        out,
        "  · {}",
        t(
            lang,
            "{0}: {1} GB da {2}",
            &[&plan.name, &gb(plan.bytes()), &plan.repo]
        )
    )?;
    let mut shown = u64::MAX;
    let cancel = std::sync::Arc::new(AtomicBool::new(false));
    let id = super::pull::pull(
        kit,
        net,
        &plan,
        token.as_deref(),
        lang,
        cancel,
        &mut |done, total| {
            let pct = done * 100 / total.max(1);
            if pct / 5 != shown {
                shown = pct / 5;
                let _ = writeln!(out, "  · {pct}% ({} / {} GB)", gb(done), gb(total));
                let _ = out.flush();
            }
        },
    )?;
    writeln!(
        out,
        "  √ {}",
        t(lang, "{0}: scaricato e verificato", &[&id])
    )?;
    // Pi's models.json lists it (a server already running reads it at its
    // next start).
    crate::agents::sync::run(kit, None);
    Ok(id)
}

pub fn remove(kit: &Kit, id: &str, lang: &str, out: &mut dyn Write) -> Result<()> {
    local::remove(kit, id)?;
    crate::agents::sync::run(kit, None);
    writeln!(out, "  √ {}", t(lang, "{0}: cancellato", &[id]))?;
    Ok(())
}

/// `llm start`: the server in the foreground, until it stops or Ctrl+C.
pub fn start(kit: &Kit, env: &crate::env::KitEnv, lang: &str, out: &mut dyn Write) -> Result<()> {
    let llm = super::service::Llm::new(kit.clone());
    let port = llm.start(env, lang)?;
    writeln!(
        out,
        "  √ {}",
        t(
            lang,
            "Modelli locali su {0} (chiave in {1}). Ctrl+C per fermarli.",
            &[
                &super::service::url_of(port),
                &super::settings::key_file(kit).display().to_string()
            ]
        )
    )?;
    out.flush()?;
    llm.wait();
    Ok(())
}

/// `llm stop`: every server of this kit.
pub fn stop(kit: &Kit, lang: &str, out: &mut dyn Write) -> Result<()> {
    let n = super::service::stop_everywhere(kit);
    writeln!(
        out,
        "  √ {}",
        t(
            lang,
            "Server dei modelli locali fermati: {0}",
            &[&n.to_string()]
        )
    )?;
    Ok(())
}

/// `llm status`: whether a server of the kit answers.
pub fn status(kit: &Kit, lang: &str, out: &mut dyn Write) -> Result<()> {
    let (port, up) = super::service::probe(kit);
    let url = super::service::url_of(port);
    if up {
        writeln!(out, "  √ {}", t(lang, "In esecuzione su {0}", &[&url]))?;
    } else {
        writeln!(out, "  · {}", t(lang, "Fermo ({0})", &[&url]))?;
    }
    Ok(())
}
