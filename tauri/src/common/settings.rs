//! The user's settings, in `data\app\settings.json`.
//!
//! Language, proxy and certificates, Gemini API key, local network, automatic
//! updates. A file that cannot be read is never overwritten: the defaults are
//! used and the problem is reported, so a hand edit with a typo does not lose
//! the other values.
//! Keys this version does not know (written by a newer one) are kept.
//! The Settings page saves each change as it is made (D-137): every
//! read-change-write goes through [`Settings::update`], one at a time.
//!
//! Two values are not kept as they appear (D-137):
//! - the Gemini key lives in `data/app/secrets/gemini-api-key`, readable by
//!   the owner only; one left in `settings.json` by an older version is read
//!   and moved there at the next save;
//! - a PEM file inside the kit is saved relative to the kit's folder, so it
//!   follows the kit to another drive or folder; [`Settings::load`] gives it
//!   back as a full path.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::kit::Kit;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Network {
    pub https_proxy: Option<String>,
    pub http_proxy: Option<String>,
    pub no_proxy: Option<String>,
    /// PEM file with extra CA certificates (SSL inspection), for Node.
    pub extra_ca_certs: Option<String>,
    /// Lets the projects' servers listen on the network (`ATOLLO_ALLOW_LAN`).
    pub allow_lan: bool,
    /// Keys of a newer version, kept as they are.
    #[serde(flatten)]
    pub other: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    /// `it`, `en`, `fr`, `de`; `None` = the language of Windows.
    pub lang: Option<String>,
    pub network: Network,
    /// Read from `data/app/secrets/gemini-api-key` (or, from an older
    /// version, this file); never written here.
    #[serde(skip_serializing)]
    pub gemini_api_key: Option<String>,
    /// The key's file is there but cannot be read (PORT-V6-01): `save`
    /// leaves it alone unless the key is removed on purpose.
    #[serde(skip)]
    pub gemini_unreadable: bool,
    pub auto_update: bool,
    /// Rust toolchain host on Windows: `gnu` (default, own linker) or
    /// `msvc` (needs Visual Studio Build Tools on the PC). Chosen on Rust's
    /// page in Tools (`rust_toolchain_set`, D-137). The lines of Python
    /// and PHP are no setting any more: a new install takes the newest, the
    /// Tools page downloads others and each project chooses its own (D-80);
    /// `pythonVersion`/`phpVersion` of an older file stay in `other`.
    pub rust_toolchain: Option<String>,
    /// `TEMP` of the programs: `auto`, `kit` or `system`.
    pub program_temp: ProgramTemp,
    /// Light or dark: the system's, or one always (D-137).
    pub theme: Theme,
    /// The archives of the conda environments that cannot follow the kit
    /// stay in `data/app/conda`, so a move reinstalls them without the
    /// network (D-118).
    pub keep_conda_packages: bool,
    /// The system the Tools page shows (`linux-x64`, `darwin-arm64`…),
    /// chosen last there with «Piattaforma» (D-121); `None` = this PC's.
    /// Only `ipc::setup::tools_target_set` writes it: the Settings
    /// page keeps it as it is.
    pub tools_target: Option<String>,
    /// Keys of a newer version, kept as they are.
    #[serde(flatten)]
    pub other: serde_json::Map<String, serde_json::Value>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            lang: None,
            network: Network::default(),
            gemini_api_key: None,
            gemini_unreadable: false,
            auto_update: true,
            rust_toolchain: None,
            program_temp: ProgramTemp::Auto,
            theme: Theme::System,
            keep_conda_packages: true,
            tools_target: None,
            other: serde_json::Map::new(),
        }
    }
}

/// Where the kit's programs keep `TEMP` (D-90): `Auto` decides by
/// the kit's disk, `Kit` is `data\tmp`, `System` the host's temporary folder.
/// An unknown value in the file reads as `Auto`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ProgramTemp {
    #[default]
    Auto,
    Kit,
    System,
}

impl<'de> Deserialize<'de> for ProgramTemp {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Ok(match s.to_ascii_lowercase().as_str() {
            "kit" => ProgramTemp::Kit,
            "system" => ProgramTemp::System,
            _ => ProgramTemp::Auto,
        })
    }
}

/// The theme of the app's windows, the editor's and the terminals' (D-137):
/// `System` follows Windows, macOS or the desktop of Linux. The windows'
/// webviews take it as their `prefers-color-scheme`, so the interface's CSS,
/// the in-app terminal and VSCodium (`window.autoDetectColorScheme`) follow
/// it with no theme of their own. An unknown value reads as `System`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

impl<'de> Deserialize<'de> for Theme {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Ok(match s.to_ascii_lowercase().as_str() {
            "light" => Theme::Light,
            "dark" => Theme::Dark,
            _ => Theme::System,
        })
    }
}

impl Theme {
    /// The windows' theme: `None` follows the system.
    pub fn window(self) -> Option<tauri::Theme> {
        match self {
            Theme::System => None,
            Theme::Light => Some(tauri::Theme::Light),
            Theme::Dark => Some(tauri::Theme::Dark),
        }
    }
}

/// Settings plus the reason they could not be read, if any.
pub struct Loaded {
    pub settings: Settings,
    pub problem: Option<String>,
}

impl Settings {
    pub fn load(kit: &Kit) -> Loaded {
        let mut loaded = Self::load_from(&kit.settings_file());
        let s = &mut loaded.settings;
        match read_secret(&gemini_file(kit)) {
            Ok(Some(key)) => s.gemini_api_key = Some(key),
            Ok(None) => {}
            Err(e) => {
                s.gemini_unreadable = true;
                crate::common::log::warn(&format!("Gemini key: {e:#}"));
            }
        }
        if let Some(ca) = &mut s.network.extra_ca_certs {
            if Path::new(ca.as_str()).is_relative() {
                *ca = crate::common::fsx::join_rel(kit.root(), ca)
                    .display()
                    .to_string();
            }
        }
        loaded
    }

    fn load_from(path: &Path) -> Loaded {
        let (settings, problem) = crate::common::fsx::load_json(path);
        Loaded { settings, problem }
    }

    /// What the Settings page sends, checked before it is saved. Returns the
    /// Italian source of the problem and its argument (translated by the caller).
    pub fn validate(&self) -> Result<(), (&'static str, String)> {
        if let Some(l) = &self.lang {
            if !crate::common::i18n::LANGS.contains(&l.as_str()) {
                return Err(("Lingua non valida: {0}", l.clone()));
            }
        }
        let n = &self.network;
        for p in [&n.https_proxy, &n.http_proxy].into_iter().flatten() {
            let ok = (p.starts_with("http://") || p.starts_with("https://"))
                && p.len() > 8
                && !p.chars().any(char::is_whitespace);
            if !ok {
                return Err(("Proxy non valido (http://host:porta): {0}", p.clone()));
            }
        }
        if let Some(np) = &n.no_proxy {
            if np.chars().any(|c| c.is_control()) {
                return Err(("Valore non valido: {0}", np.clone()));
            }
        }
        if let Some(ca) = &n.extra_ca_certs {
            if !Path::new(ca).is_file() {
                return Err(("File dei certificati non trovato: {0}", ca.clone()));
            }
            if crate::net::ca::extra_certs(Path::new(ca)).is_err() {
                return Err(("Nessun certificato PEM nel file: {0}", ca.clone()));
            }
        }
        if let Some(key) = &self.gemini_api_key {
            if key.chars().any(|c| c.is_whitespace() || c.is_control()) {
                return Err(("Valore non valido: {0}", "GEMINI_API_KEY".into()));
            }
        }
        if let Some(t) = &self.rust_toolchain {
            if t != "gnu" && t != "msvc" {
                return Err(("Valore non valido: {0}", t.clone()));
            }
        }
        if let Some(t) = &self.tools_target {
            if crate::kit::Target::parse(t).is_none() {
                return Err(("Valore non valido: {0}", t.clone()));
            }
        }
        Ok(())
    }

    /// Writes `settings.json` and the Gemini key's own file (removed when
    /// there is no key, kept when it could not be read). A PEM file inside
    /// the kit is written relative to it.
    pub fn save(&self, kit: &Kit) -> anyhow::Result<()> {
        let key = gemini_file(kit);
        match self.gemini_api_key.as_deref().filter(|k| !k.is_empty()) {
            Some(k) => crate::router::write_private(&key, k)?,
            None if self.gemini_unreadable => {}
            None => match std::fs::remove_file(&key) {
                Err(e) if e.kind() != std::io::ErrorKind::NotFound => return Err(e.into()),
                _ => {}
            },
        }
        let mut stored = self.clone();
        if let Some(ca) = &mut stored.network.extra_ca_certs {
            if let Ok(rel) = Path::new(ca.as_str()).strip_prefix(kit.root()) {
                *ca = rel.to_string_lossy().replace('\\', "/");
            }
        }
        crate::common::fsx::write_json(&kit.settings_file(), &stored)
    }

    /// Reads the file, lets `change` modify it and saves it, with no other
    /// change in between (the Settings page saves as the user types, the
    /// Tools page its «Piattaforma», installations read them). `change`
    /// gets the settings as they are on disk and whether the file was
    /// readable; an error from it leaves the file as it is.
    pub fn update<E: From<anyhow::Error>>(
        kit: &Kit,
        change: impl FnOnce(&mut Settings, Option<&str>) -> Result<(), E>,
    ) -> Result<Settings, E> {
        let _one = LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let loaded = Self::load(kit);
        let mut settings = loaded.settings;
        change(&mut settings, loaded.problem.as_deref())?;
        settings.save(kit)?;
        Ok(settings)
    }
}

/// The Gemini API key, owner-only (D-137).
pub fn gemini_file(kit: &Kit) -> PathBuf {
    kit.app_dir().join("secrets").join("gemini-api-key")
}

/// The secret in `path`: `Ok(None)` when there is none (no file, or empty),
/// an error naming the file when it is there but cannot be read (made
/// private by another account, the kit used on another PC: PORT-V6-01),
/// never mistaken for "not set".
pub(crate) fn read_secret(path: &Path) -> anyhow::Result<Option<String>> {
    match std::fs::read_to_string(path) {
        Ok(t) => Ok(Some(t.trim().to_owned()).filter(|t| !t.is_empty())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(crate::router::fs::access_error(path, e)),
    }
}

/// A Gemini key left in `settings.json` by an older version goes to its own
/// file at start; a file that cannot be read stays as it is.
pub fn move_secrets(kit: &Kit) {
    match read_secret(&gemini_file(kit)) {
        Ok(None) => {}
        Ok(Some(_)) => return,
        Err(e) => {
            // Never replaced by an older key: the user decides (the error
            // says how).
            crate::common::log::warn(&format!("Gemini key: {e:#}"));
            return;
        }
    }
    let loaded = Settings::load_from(&kit.settings_file());
    if loaded.problem.is_some() || loaded.settings.gemini_api_key.is_none() {
        return;
    }
    if let Err(e) = Settings::update(kit, |_, _| Ok::<(), anyhow::Error>(())) {
        crate::common::log::warn(&format!("settings: moving the Gemini key: {e:#}"));
    }
}

/// One read-change-write of `settings.json` at a time.
static LOCK: Mutex<()> = Mutex::new(());

#[cfg(test)]
#[path = "../../../test/rust/common/settings.rs"]
mod tests;
