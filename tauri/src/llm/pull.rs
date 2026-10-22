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
use super::hf::{Choice, HfError, RepoInfo};
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

    /// The plan of the choice `name` of `info`: its quantization or its id
    /// ([`super::hf::pick`]), or the id a model of the repository has in the
    /// kit. An id that got the publisher's name after it ([`entry_for`]) is
    /// no choice's, so a download of it left halfway is resumed by its
    /// files, under its own id.
    pub fn of_choice(kit: &Kit, info: &RepoInfo, name: &str) -> Result<Plan> {
        let dir = super::models_dir(kit).join(local::folder_of(&info.repo));
        let manifest = local::read_manifest(&dir).unwrap_or_default();
        let names = |fs: &[ModelFile]| -> Vec<String> {
            fs.iter().map(|f| disk_name(&f.name).to_owned()).collect()
        };
        let known = manifest.models.iter().find_map(|e| {
            let c = info
                .choices
                .iter()
                .find(|c| e.id.eq_ignore_ascii_case(name) && names(&c.files) == names(&e.files))?;
            Some((e, c))
        });
        if let Some((e, c)) = known {
            return Ok(Plan {
                id: e.id.clone(),
                ..Plan::from_choice(info, c)
            });
        }
        let c = super::hf::pick(info, Some(name))?;
        Ok(Plan::from_choice(info, c))
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

/// A download under way in this process: its model folder as on disk, the
/// model's id and its Cancel.
struct Running {
    folder: String,
    id: String,
    cancel: Arc<AtomicBool>,
}

/// The downloads under way in this process, one per model folder (a
/// repository: its manifest and its files are written by one download at
/// a time), by [`folder_key`].
static PULLING: Mutex<BTreeMap<String, Running>> = Mutex::new(BTreeMap::new());

fn pulling_map() -> std::sync::MutexGuard<'static, BTreeMap<String, Running>> {
    PULLING
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// A model folder as the kit's drive compares it: `Qwen--Qwen3-GGUF` and
/// `qwen--qwen3-gguf` are one folder on NTFS, exFAT and APFS, and must not
/// be downloaded into twice at once.
fn folder_key(folder: &str) -> String {
    crate::os::processes::path_key(folder)
}

/// The file that tells another process (the command line, the app) that a
/// folder is being downloaded into: the pid of the one downloading.
const LOCK: &str = ".pulling";

/// A download's hold on its folder, until dropped.
struct Pulling {
    key: String,
    lock: PathBuf,
}

impl Pulling {
    fn take(dir: &Path, folder: &str, id: &str, cancel: Arc<AtomicBool>) -> Result<Pulling> {
        let mut map = pulling_map();
        let key = folder_key(folder);
        if map.contains_key(&key) {
            bail!("a model of this repository is already being downloaded");
        }
        std::fs::create_dir_all(dir)?;
        let lock = dir.join(LOCK);
        if !claim(dir)? {
            bail!("a model of this repository is already being downloaded");
        }
        map.insert(
            key.clone(),
            Running {
                folder: folder.to_owned(),
                id: id.to_owned(),
                cancel,
            },
        );
        Ok(Pulling { key, lock })
    }
}

impl Drop for Pulling {
    fn drop(&mut self) {
        pulling_map().remove(&self.key);
        // Only our own lock: one that another process took over after ours
        // was judged stale stays its own.
        let mine = std::process::id().to_string();
        if std::fs::read_to_string(&self.lock).is_ok_and(|s| s.trim() == mine) {
            let _ = std::fs::remove_file(&self.lock);
        }
    }
}

/// Creates the lock of `dir` with this process's pid, or false when another
/// process holds it. The file is created only if it is not there
/// (`create_new`), so the app and the command line starting a download into
/// one folder at the same moment cannot both get it; a lock left by a
/// process that is gone, or by this one, is replaced.
fn claim(dir: &Path) -> Result<bool> {
    use std::io::Write;
    let lock = dir.join(LOCK);
    for _ in 0..3 {
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock)
        {
            Ok(mut file) => {
                file.write_all(std::process::id().to_string().as_bytes())?;
                return Ok(true);
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                if locked_elsewhere(dir) {
                    return Ok(false);
                }
                remove_if_there(&lock)?;
            }
            Err(e) => return Err(anyhow::Error::new(e).context(lock.display().to_string())),
        }
    }
    Ok(false)
}

/// Another process downloads into `dir` now: its lock with a live pid, or
/// one created a moment ago whose pid is not written yet.
fn locked_elsewhere(dir: &Path) -> bool {
    let me = std::process::id();
    let lock = dir.join(LOCK);
    let Ok(text) = std::fs::read_to_string(&lock) else {
        return false;
    };
    match text.trim().parse::<u32>() {
        Ok(pid) => pid != me && crate::agents::sync::alive(pid),
        Err(_) => std::fs::metadata(&lock)
            .and_then(|m| m.modified())
            .is_ok_and(|t| t.elapsed().is_ok_and(|age| age < LOCK_FRESH)),
    }
}

/// How long a lock without a pid counts as being written.
const LOCK_FRESH: std::time::Duration = std::time::Duration::from_secs(5);

/// A download goes on in `folder` (of `data/models/gguf`), here or in
/// another process: nothing there may be removed meanwhile.
pub fn busy(kit: &Kit, folder: &str) -> bool {
    pulling_map().contains_key(&folder_key(folder))
        || locked_elsewhere(&super::models_dir(kit).join(folder))
}

/// The downloads under way in this process, as `<folder>/<id>`.
pub fn pulling() -> Vec<String> {
    pulling_map()
        .values()
        .map(|r| format!("{}/{}", r.folder, r.id))
        .collect()
}

/// Stops the download `<folder>/<id>`; what arrived is kept.
pub fn cancel(key: &str) {
    let folder = key.split_once('/').map_or(key, |(f, _)| f);
    if let Some(r) = pulling_map().get(&folder_key(folder)) {
        r.cancel.store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

/// Stops every download (the drive is being removed, the app closes) and
/// waits for them to let go of their files, at most `limit`; how many
/// were running.
pub fn cancel_all(limit: std::time::Duration) -> usize {
    let n = {
        let map = pulling_map();
        for r in map.values() {
            r.cancel.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        map.len()
    };
    let start = std::time::Instant::now();
    while !pulling_map().is_empty() && start.elapsed() < limit {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    n
}

/// Every file of `e` is on disk in `dir` with its size, and the entry says
/// they were downloaded and checked.
fn whole(dir: &Path, e: &Entry) -> bool {
    e.complete
        && e.files.iter().all(|f| {
            std::fs::metadata(dir.join(&f.name)).is_ok_and(|m| m.is_file() && m.len() == f.bytes)
        })
}

/// Deletes `path`; nothing when it is not there.
fn remove_if_there(path: &Path) -> Result<()> {
    match std::fs::remove_file(path) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
            Err(anyhow::Error::new(e).context(path.display().to_string()))
        }
        _ => Ok(()),
    }
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
            .iter_mut()
            .find(|e| names(&e.files) == names(&files))
        {
            // A download left halfway may be of an older commit: publishers
            // upload new GGUFs under the same names, and the plan is read
            // again from the repository's last commit. Its bytes joined to
            // those of the new file would fail the checksum only at the end
            // of the whole download, so the files that changed start again
            // and the entry takes the plan's commit and checksums. A model
            // already downloaded and checked is left as it is.
            if !whole(&dir, e) && (e.commit != plan.commit || e.files != files) {
                for (old, new) in e.files.iter().zip(&files) {
                    if old.sha256 != new.sha256 || old.bytes != new.bytes {
                        let file = dir.join(&new.name);
                        remove_if_there(&crate::net::resume::part_of(&file))?;
                        remove_if_there(&file)?;
                    }
                }
                e.commit = plan.commit.clone();
                e.files = files.clone();
                e.complete = false;
            }
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

/// `dest` is file `f` of the plan, whole: of its size, and checked when it
/// arrived (the entry, complete, has it with the same checksum) or read now.
/// One of the same size but another content (put there by hand, left by
/// another commit) would otherwise be marked as downloaded and checked: it
/// is deleted, and downloaded again.
fn already_there(dest: &Path, entry: &Entry, f: &ModelFile) -> Result<bool> {
    if !std::fs::metadata(dest).is_ok_and(|m| m.is_file() && m.len() == f.bytes) {
        return Ok(false);
    }
    let name = disk_name(&f.name);
    let checked = entry.complete
        && entry
            .files
            .iter()
            .any(|e| e.name == name && e.sha256.eq_ignore_ascii_case(&f.sha256));
    if checked || crate::net::verify::sha256_hex(dest)?.eq_ignore_ascii_case(f.sha256.trim()) {
        return Ok(true);
    }
    crate::log::warn(&format!(
        "model {}: {name} does not match its checksum, downloaded again",
        entry.id
    ));
    remove_if_there(dest)?;
    Ok(false)
}

/// A file of `repo` refused (401, 403) as [`HfError::Gated`]: `inspect`
/// lets a gated repository through with any token, since its list can be
/// read, and only the download finds that the token has no access to it
/// (its licence not accepted on Hugging Face). The user then reads what to
/// do instead of a bare HTTP status.
fn refused_to_gated(e: anyhow::Error, repo: &str) -> anyhow::Error {
    let refused = e.chain().any(|cause| {
        matches!(
            cause.downcast_ref::<ureq::Error>(),
            Some(ureq::Error::StatusCode(401 | 403))
        )
    });
    if refused {
        crate::log::warn(&format!("{repo}: {e:#}"));
        HfError::Gated(repo.to_owned()).into()
    } else {
        e
    }
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
    // Downloaded and checked already: the entry keeps its own commit
    // (entry_for), so its files are not measured against the plan's.
    if whole(&dir, &entry) {
        let bytes = entry.files.iter().map(|f| f.bytes).sum();
        progress(bytes, bytes);
        return Ok(entry.id);
    }
    let total = plan.bytes();
    let auth = token.map(|t| format!("Bearer {t}"));
    let mut before = 0u64;
    for f in &plan.files {
        let dest = dir.join(disk_name(&f.name));
        if already_there(&dest, &entry, f)? {
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
        )
        .map_err(|e| refused_to_gated(e, &plan.repo))?;
        before += f.bytes;
    }
    mark_complete(kit, plan, &entry.id)?;
    crate::log::info(&format!("model {}: downloaded and checked", entry.id));
    Ok(entry.id)
}

#[cfg(test)]
#[path = "../../../test/rust/llm/pull.rs"]
mod tests;
