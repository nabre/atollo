//! Downloads of models: a catalog model or a choice of a Hugging Face
//! repository becomes a [`Plan`] (its files at a fixed commit, each with
//! its SHA-256), checked against the drive before a byte is written: a file
//! larger than FAT32 holds is refused with what to do, and so is a model
//! that does not fit the free space. Each file goes through `<file>.part`
//! and is resumed from where it stopped (`net::resume`), also after the app
//! was closed: the entry in `atollo.json` is written first, `complete` once
//! every file is there and checked.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

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

/// The downloads under way in this process, one per model folder (a
/// repository: its manifest and its files are written by one download at
/// a time): the model's id and its Cancel.
static PULLING: Mutex<BTreeMap<String, (String, Arc<AtomicBool>)>> = Mutex::new(BTreeMap::new());

fn pulling_map() -> std::sync::MutexGuard<'static, BTreeMap<String, (String, Arc<AtomicBool>)>> {
    PULLING
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// The file that tells another process (the command line, the app) that a
/// folder is being downloaded into: the pid of the one downloading.
const LOCK: &str = ".pulling";

/// A download's hold on its folder, until dropped.
struct Pulling {
    folder: String,
    lock: PathBuf,
}

impl Pulling {
    fn take(dir: &Path, folder: &str, id: &str, cancel: Arc<AtomicBool>) -> Result<Pulling> {
        let mut map = pulling_map();
        if map.contains_key(folder) || locked_elsewhere(dir) {
            bail!("a model of this repository is already being downloaded");
        }
        std::fs::create_dir_all(dir)?;
        let lock = dir.join(LOCK);
        std::fs::write(&lock, std::process::id().to_string())?;
        map.insert(folder.to_owned(), (id.to_owned(), cancel));
        Ok(Pulling {
            folder: folder.to_owned(),
            lock,
        })
    }
}

impl Drop for Pulling {
    fn drop(&mut self) {
        pulling_map().remove(&self.folder);
        let _ = std::fs::remove_file(&self.lock);
    }
}

/// Another process downloads into `dir` now (its lock, with a live pid).
fn locked_elsewhere(dir: &Path) -> bool {
    let me = std::process::id();
    std::fs::read_to_string(dir.join(LOCK))
        .ok()
        .and_then(|s| s.trim().parse::<u32>().ok())
        .is_some_and(|pid| pid != me && crate::agents::sync::alive(pid))
}

/// A download goes on in `folder` (of `data/models/gguf`), here or in
/// another process: nothing there may be removed meanwhile.
pub fn busy(kit: &Kit, folder: &str) -> bool {
    pulling_map().contains_key(folder) || locked_elsewhere(&super::models_dir(kit).join(folder))
}

/// The downloads under way in this process, as `<folder>/<id>`.
pub fn pulling() -> Vec<String> {
    pulling_map()
        .iter()
        .map(|(folder, (id, _))| format!("{folder}/{id}"))
        .collect()
}

/// Stops the download `<folder>/<id>`; what arrived is kept.
pub fn cancel(key: &str) {
    let folder = key.split_once('/').map_or(key, |(f, _)| f);
    if let Some((_, flag)) = pulling_map().get(folder) {
        flag.store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

/// Stops every download (the drive is being removed, the app closes) and
/// waits for them to let go of their files, at most `limit`; how many
/// were running.
pub fn cancel_all(limit: std::time::Duration) -> usize {
    let n = {
        let map = pulling_map();
        for (_, flag) in map.values() {
            flag.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        map.len()
    };
    let start = std::time::Instant::now();
    while !pulling_map().is_empty() && start.elapsed() < limit {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    n
}

/// The entry of `plan` in its folder's manifest: an entry with the same
/// files already there (the same model under another id) is used instead.
/// An id another repository has already (the same file of another
/// publisher) gets the publisher's name after it; one taken by other files
/// of the same repository is refused.
fn entry_for(kit: &Kit, plan: &Plan) -> Result<Entry> {
    let dir = plan.dir(kit);
    let folder = local::folder_of(&plan.repo);
    let files = plan.local_files();
    let names = |fs: &[ModelFile]| fs.iter().map(|f| f.name.clone()).collect::<Vec<_>>();
    let others = local::list(kit);
    let taken = |id: &str| others.iter().find(|m| m.id == id).map(|m| m.folder.clone());
    local::update_manifest(&dir, |manifest| {
        if let Some(e) = manifest
            .models
            .iter()
            .find(|e| names(&e.files) == names(&files))
        {
            return Ok(e.clone());
        }
        let mut id = plan.id.clone();
        match taken(&id) {
            Some(f) if f == folder => bail!("a model called {id} is already in the kit"),
            Some(_) => {
                let owner = plan.repo.split('/').next().unwrap_or_default();
                id = super::slug(&format!("{id}-{owner}"));
                if taken(&id).is_some() {
                    bail!("a model called {id} is already in the kit");
                }
            }
            None => {}
        }
        let entry = Entry {
            id,
            name: plan.name.clone(),
            repo: plan.repo.clone(),
            commit: plan.commit.clone(),
            files: files.clone(),
            catalog: plan.catalog.clone(),
            ctx: plan.ctx,
            complete: false,
        };
        manifest.models.push(entry.clone());
        Ok(entry)
    })
}

fn mark_complete(kit: &Kit, plan: &Plan, id: &str) -> Result<()> {
    local::update_manifest(&plan.dir(kit), |manifest| {
        let mut found = false;
        for e in manifest.models.iter_mut().filter(|e| e.id == id) {
            e.complete = true;
            found = true;
        }
        if !found {
            bail!("{id}: its entry is gone from {}", local::MANIFEST);
        }
        Ok(())
    })
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
    cancel: Arc<AtomicBool>,
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
    let dir = plan.dir(kit);
    let _pulling = Pulling::take(
        &dir,
        &local::folder_of(&plan.repo),
        &plan.id,
        Arc::clone(&cancel),
    )?;
    check_room(kit, plan, lang)?;
    let entry = entry_for(kit, plan)?;
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
            &cancel,
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
