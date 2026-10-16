//! The user's settings, in `data\app\settings.json`.
//!
//! They replace nabre/atollo's `data\proxy.cmd`: language, proxy and
//! certificates, Gemini API key, local network, automatic updates. A file that
//! cannot be read is never overwritten: the defaults are used and the problem
//! is reported, so a hand edit with a typo does not lose the other values.

use std::path::Path;

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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings {
    /// `it`, `en`, `fr`, `de`; `None` = the language of Windows.
    pub lang: Option<String>,
    pub network: Network,
    pub gemini_api_key: Option<String>,
    pub auto_update: bool,
    /// Python minor version (`3.13`); `None` = the kit default.
    pub python_version: Option<String>,
    /// PHP branch (`8.4`); `None` = the newest.
    pub php_version: Option<String>,
    /// Rust toolchain host on x64: `gnu` (default, own linker) or `msvc`
    /// (needs Visual Studio Build Tools on the PC).
    pub rust_toolchain: Option<String>,
    /// Where consoles open: `None` = the kit's Windows Terminal, `classic` =
    /// Windows' classic console (D-68).
    pub terminal: Option<String>,
    /// `TEMP` of the programs: `auto`, `kit` or `system`.
    pub program_temp: ProgramTemp,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            lang: None,
            network: Network::default(),
            gemini_api_key: None,
            auto_update: true,
            python_version: None,
            php_version: None,
            rust_toolchain: None,
            terminal: None,
            program_temp: ProgramTemp::Auto,
        }
    }
}

/// Where the kit's programs keep `TEMP` (PLAN-DATA 4.2.5): `Auto` decides by
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

/// Settings plus the reason they could not be read, if any.
pub struct Loaded {
    pub settings: Settings,
    pub problem: Option<String>,
}

impl Settings {
    pub fn load(kit: &Kit) -> Loaded {
        Self::load_from(&kit.settings_file())
    }

    fn load_from(path: &Path) -> Loaded {
        let (settings, problem) = crate::fsx::load_json(path);
        Loaded { settings, problem }
    }

    /// What the Settings page sends, checked before it is saved. Returns the
    /// Italian source of the problem and its argument (translated by the caller).
    pub fn validate(&self) -> Result<(), (&'static str, String)> {
        if let Some(l) = &self.lang {
            if !crate::i18n::LANGS.contains(&l.as_str()) {
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
        }
        if let Some(key) = &self.gemini_api_key {
            if key.chars().any(|c| c.is_whitespace() || c.is_control()) {
                return Err(("Valore non valido: {0}", "GEMINI_API_KEY".into()));
            }
        }
        let version = |v: &Option<String>, major: &str| {
            v.as_ref().is_none_or(|v| {
                v.strip_prefix(major)
                    .and_then(|m| m.strip_prefix('.'))
                    .is_some_and(|m| {
                        !m.is_empty() && m.len() <= 2 && m.chars().all(|c| c.is_ascii_digit())
                    })
            })
        };
        if !version(&self.python_version, "3") {
            return Err((
                "Versione non valida: {0}",
                self.python_version.clone().unwrap_or_default(),
            ));
        }
        if !version(&self.php_version, "8") {
            return Err((
                "Versione non valida: {0}",
                self.php_version.clone().unwrap_or_default(),
            ));
        }
        if let Some(t) = &self.rust_toolchain {
            if t != "gnu" && t != "msvc" {
                return Err(("Valore non valido: {0}", t.clone()));
            }
        }
        if let Some(t) = &self.terminal {
            if t != "classic" {
                return Err(("Valore non valido: {0}", t.clone()));
            }
        }
        Ok(())
    }

    /// Consoles in Windows' classic console instead of the kit's terminal.
    pub fn classic_console(&self) -> bool {
        self.terminal.as_deref() == Some("classic")
    }

    pub fn save(&self, kit: &Kit) -> anyhow::Result<()> {
        crate::fsx::write_json(&kit.settings_file(), self)
    }
}

#[cfg(test)]
#[path = "../../test/rust/settings.rs"]
mod tests;
