//! Any model of Hugging Face (`owner/repo`): the repository's last commit,
//! its GGUF files with their SHA-256 (the `lfs.oid` of the API), grouped
//! into the models one can download (a quantization, all its parts). The
//! download is from that commit (`resolve/<commit>/<file>`), so the files
//! are the ones listed. The token, for the gated models, goes to
//! huggingface.co only.

use std::time::Duration;

use anyhow::{anyhow, bail, Result};
use serde::Serialize;
use serde_json::Value;

use super::catalog::ModelFile;
use super::local::{is_mmproj, model_stem, valid_repo};
use crate::net::Net;

const API: &str = "https://huggingface.co/api/models";
const TIMEOUT: Duration = Duration::from_secs(30);

/// One model of a repository: a quantization, with all its parts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Choice {
    /// The id it gets in the kit.
    pub id: String,
    /// The file's stem, without the part number.
    pub label: String,
    pub quant: Option<String>,
    /// The first part first.
    pub files: Vec<ModelFile>,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoInfo {
    pub repo: String,
    pub commit: String,
    /// Its license, as the model card says.
    pub license: Option<String>,
    /// Smallest first.
    pub choices: Vec<Choice>,
}

/// Why a repository cannot be read.
#[derive(Debug, thiserror::Error)]
pub enum HfError {
    #[error("{0}: not a repository of Hugging Face (owner/name)")]
    BadName(String),
    #[error("{0} not found on Hugging Face")]
    NotFound(String),
    /// Gated or private: a token with access is needed.
    #[error("{0} needs a Hugging Face token with access to it")]
    Gated(String),
    #[error("{0}: no GGUF file")]
    NoGguf(String),
}

/// `owner/repo[:quant]`.
pub fn parse_spec(spec: &str) -> Result<(String, Option<String>)> {
    let spec = spec.trim();
    let (repo, quant) = match spec.split_once(':') {
        Some((r, q)) => (r, Some(q.trim().to_owned()).filter(|q| !q.is_empty())),
        None => (spec, None),
    };
    if !valid_repo(repo) {
        bail!(HfError::BadName(repo.to_owned()));
    }
    Ok((repo.to_owned(), quant))
}

/// The quantization in a file's stem (`Q4_K_M`, `IQ3_XXS`, `MXFP4`, `F16`...).
pub fn quant_of(stem: &str) -> Option<String> {
    static RE: std::sync::OnceLock<Option<regex::Regex>> = std::sync::OnceLock::new();
    let re = RE.get_or_init(|| {
        regex::Regex::new(
            r"(?i)(?:^|[-_.])((?:UD-)?(?:IQ\d_[A-Z]+|Q\d_K_[A-Z]{1,2}|Q\d_K|Q\d_\d|TQ\d_\d|MXFP4|BF16|F16|F32))(?:$|[-_.])",
        )
        .ok()
    });
    re.as_ref()?
        .captures_iter(stem)
        .last()
        .map(|c| c[1].to_ascii_uppercase())
}

/// The GGUF files of a tree listing (`api/models/<repo>/tree/<commit>`),
/// grouped into models: one per stem (its parts together, a model in
/// several parts only with all of them), projectors aside. Smallest first.
pub fn choices(tree: &Value) -> Vec<Choice> {
    let mut files: Vec<(String, ModelFile)> = Vec::new();
    for item in tree.as_array().into_iter().flatten() {
        if item.get("type").and_then(Value::as_str) != Some("file") {
            continue;
        }
        let Some(path) = item.get("path").and_then(Value::as_str) else {
            continue;
        };
        let name = path.rsplit('/').next().unwrap_or(path);
        if !name.to_ascii_lowercase().ends_with(".gguf") || is_mmproj(name) {
            continue;
        }
        if !crate::install::is_plain_file_name(name) {
            continue;
        }
        let lfs = item.get("lfs");
        let Some(sha) = lfs
            .and_then(|l| l.get("oid"))
            .and_then(Value::as_str)
            .filter(|s| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()))
        else {
            continue;
        };
        let bytes = lfs
            .and_then(|l| l.get("size"))
            .or_else(|| item.get("size"))
            .and_then(Value::as_u64)
            .unwrap_or(0);
        files.push((
            path.to_owned(),
            ModelFile {
                name: path.to_owned(),
                sha256: sha.to_ascii_lowercase(),
                bytes,
            },
        ));
    }
    files.sort_by(|a, b| a.0.cmp(&b.0));
    let mut out: Vec<Choice> = Vec::new();
    for (path, file) in files {
        let (dir, name) = path.rsplit_once('/').unwrap_or(("", &path));
        let stem = model_stem(name).to_owned();
        let key = format!("{dir}/{stem}");
        match out.iter_mut().find(|c| c.label == key) {
            Some(c) => {
                c.bytes += file.bytes;
                c.files.push(file);
            }
            None => out.push(Choice {
                id: super::slug(&stem),
                quant: quant_of(&stem),
                label: key,
                bytes: file.bytes,
                files: vec![file],
            }),
        }
    }
    // A model in several parts only with all of them.
    out.retain(|c| {
        let name = |f: &ModelFile| f.name.rsplit('/').next().unwrap_or(&f.name).to_owned();
        match super::local::part_of(name(&c.files[0]).trim_end_matches(".gguf")) {
            None => c.files.len() == 1,
            Some((_, count)) => c.files.len() == count as usize,
        }
    });
    for c in &mut out {
        c.label = c.label.trim_start_matches('/').to_owned();
    }
    // On disk a file is its last part: two choices with a file of the same
    // name (or the same id) would share it; the first stays.
    let mut names = std::collections::BTreeSet::new();
    let mut ids = std::collections::BTreeSet::new();
    out.retain(|c| {
        let own: Vec<&str> = c
            .files
            .iter()
            .map(|f| f.name.rsplit('/').next().unwrap_or(&f.name))
            .collect();
        let free = ids.insert(c.id.clone()) && own.iter().all(|n| !names.contains(*n));
        if free {
            names.extend(own.iter().map(|n| (*n).to_owned()));
        }
        free
    });
    out.sort_by_key(|c| c.bytes);
    out
}

fn headers(token: Option<&str>) -> Vec<(&'static str, String)> {
    token
        .map(|t| vec![("Authorization", format!("Bearer {t}"))])
        .unwrap_or_default()
}

fn get(net: &Net, url: &str, repo: &str, token: Option<&str>) -> Result<Value> {
    let headers = headers(token);
    let refs: Vec<(&str, &str)> = headers.iter().map(|(k, v)| (*k, v.as_str())).collect();
    net.api_json(url, &refs, None, TIMEOUT)
        .map_err(|e| match e.status {
            Some(401 | 403) => HfError::Gated(repo.to_owned()).into(),
            Some(404) => HfError::NotFound(repo.to_owned()).into(),
            _ => anyhow!("{url}: {}", e.detail),
        })
}

/// The repository's last commit and its GGUF models.
pub fn inspect(net: &Net, repo: &str, token: Option<&str>) -> Result<RepoInfo> {
    if !valid_repo(repo) {
        bail!(HfError::BadName(repo.to_owned()));
    }
    let meta = get(net, &format!("{API}/{repo}"), repo, token)?;
    let commit = meta
        .get("sha")
        .and_then(Value::as_str)
        .filter(|s| s.len() == 40 && s.bytes().all(|b| b.is_ascii_hexdigit()))
        .ok_or_else(|| anyhow!("{repo}: no commit in the answer of Hugging Face"))?
        .to_owned();
    // Gated: reading the list works, the download does not without access.
    let gated = !matches!(meta.get("gated"), None | Some(Value::Bool(false)));
    if gated && token.is_none() {
        bail!(HfError::Gated(repo.to_owned()));
    }
    let license = meta
        .pointer("/cardData/license")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let tree = get(
        net,
        &format!("{API}/{repo}/tree/{commit}?recursive=true"),
        repo,
        token,
    )?;
    let choices = choices(&tree);
    if choices.is_empty() {
        bail!(HfError::NoGguf(repo.to_owned()));
    }
    Ok(RepoInfo {
        repo: repo.to_owned(),
        commit,
        license,
        choices,
    })
}

/// The choice of `info` that `quant` names (its quantization or its id),
/// or the only one.
pub fn pick<'a>(info: &'a RepoInfo, quant: Option<&str>) -> Result<&'a Choice> {
    match quant {
        Some(q) => info
            .choices
            .iter()
            .find(|c| {
                c.quant
                    .as_deref()
                    .is_some_and(|x| x.eq_ignore_ascii_case(q))
                    || c.id.eq_ignore_ascii_case(q)
            })
            .ok_or_else(|| anyhow!("{}: no {q}; there are {}", info.repo, names(info))),
        None if info.choices.len() == 1 => Ok(&info.choices[0]),
        None => bail!(
            "{}: say which one, {}:<quantization> ({})",
            info.repo,
            info.repo,
            names(info)
        ),
    }
}

fn names(info: &RepoInfo) -> String {
    info.choices
        .iter()
        .map(|c| c.quant.clone().unwrap_or_else(|| c.id.clone()))
        .collect::<Vec<_>>()
        .join(", ")
}

/// The address of `file` of `repo` at `commit`, each part of its path
/// encoded.
pub fn file_url(repo: &str, commit: &str, file: &str) -> String {
    let path: Vec<String> = file.split('/').map(encode).collect();
    format!(
        "https://huggingface.co/{repo}/resolve/{commit}/{}",
        path.join("/")
    )
}

fn encode(part: &str) -> String {
    part.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'.' | b'_' | b'-' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

#[cfg(test)]
#[path = "../../../test/rust/llm/hf.rs"]
mod tests;
