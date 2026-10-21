//! Downloads of models: a catalog model or a choice of a Hugging Face
//! repository becomes a [`Plan`] (its files at a fixed commit, each with
//! its SHA-256), checked against the drive before a byte is written: a file
//! larger than FAT32 holds is refused with what to do, and so is a model
//! that does not fit the free space. Each file goes through `<file>.part`
//! and is resumed from where it stopped (`net::resume`), also after the app
//! was closed: the entry in `atollo.json` is written first, `complete` once
//! every file is there and checked.

use std::collections::BTreeSet;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use anyhow::{bail, Result};

use super::catalog::{CatalogModel, ModelFile};
use super::hf::{Choice, RepoInfo};
use super::local::{self, Entry};
use crate::kit::Kit;
use crate::net::Net;

/// What to download.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub id: String,
    pub name: String,
    pub repo: String,
    pub commit: String,
    /// The files: `name` is the path in the repository; on disk each is
    /// its last part, in the model's folder.
    pub files: Vec<ModelFile>,
    pub catalog: Option<String>,
    pub ctx: Option<u32>,
}

impl Plan {
    pub fn from_catalog(m: &CatalogModel) -> Plan {
        Plan {
            id: m.id.clone(),
            name: m.name.clone(),
            repo: m.repo.clone(),
            commit: m.commit.clone(),
            files: m.files.clone(),
            catalog: Some(m.id.clone()),
            ctx: Some(m.ctx),
        }
    }

    pub fn from_choice(info: &RepoInfo, c: &Choice) -> Plan {
        let repo_name = info.repo.rsplit('/').next().unwrap_or(&info.repo);
        let name = match &c.quant {
            Some(q) => format!("{} {q}", repo_name.trim_end_matches("-GGUF")),
            None => c.label.rsplit('/').next().unwrap_or(&c.label).to_owned(),
        };
        Plan {
            id: c.id.clone(),
            name,
            repo: info.repo.clone(),
            commit: info.commit.clone(),
            files: c.files.clone(),
            catalog: None,
            ctx: None,
        }
    }

    pub fn bytes(&self) -> u64 {
        self.files.iter().map(|f| f.bytes).sum()
    }

    /// The folder of the model.
    pub fn dir(&self, kit: &Kit) -> std::path::PathBuf {
        super::models_dir(kit).join(local::folder_of(&self.repo))
    }

    /// The files as the entry of `atollo.json` has them: by their name on
    /// disk.
    fn local_files(&self) -> Vec<ModelFile> {
        self.files
            .iter()
            .map(|f| ModelFile {
                name: disk_name(&f.name).to_owned(),
                ..f.clone()
            })
            .collect()
    }
}

/// The name on disk of a file of the repository: its last part.
pub fn disk_name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

/// Why a model cannot go on the kit's drive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Room {
    /// A file larger than the file system holds (FAT32: 4 GiB - 1).
    TooLargeForFs {
        file: String,
        bytes: u64,
        fs: String,
    },
    /// Not enough free space: what is needed and what is free.
    NoSpace { need: u64, free: u64 },
}

/// Bytes kept free on the drive beyond the model.
const MARGIN: u64 = 512 << 20;

/// Whether `plan` fits a drive of file system `fs` (its largest file
/// `max_file`) with `free` bytes free, `done` of them already downloaded.
pub fn room_for(
    plan: &Plan,
    fs: &str,
    max_file: Option<u64>,
    free: u64,
    done: u64,
) -> Option<Room> {
    if let Some(max) = max_file {
        if let Some(f) = plan.files.iter().find(|f| f.bytes > max) {
            return Some(Room::TooLargeForFs {
                file: disk_name(&f.name).to_owned(),
                bytes: f.bytes,
                fs: fs.to_owned(),
            });
        }
    }
    let need = plan.bytes().saturating_sub(done) + MARGIN;
    (need > free).then_some(Room::NoSpace { need, free })
}

/// `bytes` as GB with one decimal, for messages.
pub fn gb(bytes: u64) -> String {
    format!("{:.1}", bytes as f64 / 1e9)
}

impl Room {
    /// The message for the user, in `lang`.
    pub fn message(&self, lang: &str) -> String {
        match self {
            Room::TooLargeForFs { file, bytes, fs } => crate::i18n::t(
                lang,
                "{0} pesa {1} GB: il disco del kit è {2} e non tiene file oltre i 4 GB. Su un disco exFAT o NTFS ci sta, oppure scegli un modello più piccolo.",
                &[file, &gb(*bytes), fs],
            ),
            Room::NoSpace { need, free } => crate::i18n::t(
                lang,
                "Spazio insufficiente sul disco del kit: servono {0} GB, liberi {1} GB.",
                &[&gb(*need), &gb(*free)],
            ),
        }
    }
}

/// Bytes of `plan` already on disk (whole files and parts).
fn done_of(kit: &Kit, plan: &Plan) -> u64 {
    let dir = plan.dir(kit);
    plan.files
        .iter()
        .map(|f| {
            let file = dir.join(disk_name(&f.name));
            match std::fs::metadata(&file) {
                Ok(m) if m.is_file() => m.len(),
                _ => crate::net::resume::downloaded(&file),
            }
        })
        .sum()
}

/// The drive check of `plan` on the kit's disk, as an error in `lang`.
pub fn check_room(kit: &Kit, plan: &Plan, lang: &str) -> Result<()> {
    let root = super::models_dir(kit);
    let at = existing_parent(&root);
    let info = crate::os::volume::info(&at)?;
    let room = room_for(
        plan,
        info.fs.as_str(),
        info.fs.max_file_size(),
        info.free,
        done_of(kit, plan),
    );
    match room {
        Some(r) => bail!("{}", r.message(lang)),
        None => Ok(()),
    }
}

fn existing_parent(path: &Path) -> std::path::PathBuf {
    let mut p = path.to_path_buf();
    while !p.exists() {
        match p.parent() {
            Some(parent) => p = parent.to_path_buf(),
            None => break,
        }
    }
    p
}

/// The models being downloaded now (`<folder>/<id>`): one download of a
/// model at a time.
static PULLING: Mutex<BTreeSet<String>> = Mutex::new(BTreeSet::new());

struct Pulling(String);

impl Pulling {
    fn take(key: String) -> Option<Pulling> {
        let mut set = PULLING
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        set.insert(key.clone()).then_some(Pulling(key))
    }
}

impl Drop for Pulling {
    fn drop(&mut self) {
        PULLING
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(&self.0);
    }
}

/// The entry of `plan` in its folder's manifest: an entry with the same
/// files already there (the same model under another id) is used instead;
/// an id taken by other files is refused.
fn entry_for(kit: &Kit, plan: &Plan) -> Result<Entry> {
    let dir = plan.dir(kit);
    let mut manifest = local::read_manifest(&dir)?;
    let files = plan.local_files();
    let names = |fs: &[ModelFile]| fs.iter().map(|f| f.name.clone()).collect::<Vec<_>>();
    if let Some(e) = manifest
        .models
        .iter()
        .find(|e| names(&e.files) == names(&files))
    {
        return Ok(e.clone());
    }
    if local::list(kit).iter().any(|m| m.id == plan.id) {
        bail!("a model called {} is already in the kit", plan.id);
    }
    let entry = Entry {
        id: plan.id.clone(),
        name: plan.name.clone(),
        repo: plan.repo.clone(),
        commit: plan.commit.clone(),
        files,
        catalog: plan.catalog.clone(),
        ctx: plan.ctx,
        complete: false,
    };
    manifest.models.push(entry.clone());
    local::write_manifest(&dir, &manifest)?;
    Ok(entry)
}

fn mark_complete(kit: &Kit, plan: &Plan, id: &str) -> Result<()> {
    let dir = plan.dir(kit);
    let mut manifest = local::read_manifest(&dir)?;
    for e in manifest.models.iter_mut().filter(|e| e.id == id) {
        e.complete = true;
    }
    local::write_manifest(&dir, &manifest)
}

/// Downloads `plan` (or what is missing of it) into its folder; `token`
/// goes to huggingface.co only. `progress(done, total)` over the whole
/// model. Returns the model's id (that of an entry with the same files).
pub fn pull(
    kit: &Kit,
    net: &Net,
    plan: &Plan,
    token: Option<&str>,
    lang: &str,
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(u64, u64),
) -> Result<String> {
    if !super::valid_id(&plan.id) || !local::valid_repo(&plan.repo) {
        bail!("{}: not a model the kit can download", plan.id);
    }
    if plan
        .files
        .iter()
        .any(|f| !crate::install::is_plain_file_name(disk_name(&f.name)))
    {
        bail!("{}: refused file name", plan.repo);
    }
    let Some(_pulling) = Pulling::take(format!("{}/{}", local::folder_of(&plan.repo), plan.id))
    else {
        bail!("{} is already being downloaded", plan.name);
    };
    check_room(kit, plan, lang)?;
    let entry = entry_for(kit, plan)?;
    let dir = plan.dir(kit);
    let total = plan.bytes();
    let auth = token.map(|t| format!("Bearer {t}"));
    let mut before = 0u64;
    for f in &plan.files {
        let dest = dir.join(disk_name(&f.name));
        if dest.is_file() && std::fs::metadata(&dest).is_ok_and(|m| m.len() == f.bytes) {
            before += f.bytes;
            progress(before, total);
            continue;
        }
        let url = super::hf::file_url(&plan.repo, &plan.commit, &f.name);
        let headers: Vec<(&str, &str)> = auth
            .as_deref()
            .map(|a| vec![("Authorization", a)])
            .unwrap_or_default();
        crate::log::info(&format!("model {}: downloading {url}", entry.id));
        net.download_resumable(
            &url,
            &dest,
            &f.sha256,
            Some(f.bytes),
            &headers,
            cancel,
            &mut |done, _| progress(before + done, total),
        )?;
        before += f.bytes;
    }
    mark_complete(kit, plan, &entry.id)?;
    crate::log::info(&format!("model {}: downloaded and checked", entry.id));
    Ok(entry.id)
}

#[cfg(test)]
#[path = "../../../test/rust/llm/pull.rs"]
mod tests;
