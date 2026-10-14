//! The user's settings, in `data\app\settings.json`.
//!
//! They replace nabre/atollo's `data\proxy.cmd`: language, proxy and
//! certificates, Gemini API key, local network, automatic updates. A file that
//! cannot be read is never overwritten: the defaults are used and the problem
//! is reported, so a hand edit with a typo does not lose the other values.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::kit::{write_atomic, Kit};

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
        Self::load_from(&kit.settings_file())
    }

    fn load_from(path: &Path) -> Loaded {
        match fs::read(path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Loaded {
                settings: Settings::default(),
                problem: None,
            },
            Err(e) => Loaded {
                settings: Settings::default(),
                problem: Some(format!("{}: {e}", path.display())),
            },
            Ok(bytes) => {
                let text = String::from_utf8_lossy(&bytes);
                let text = text.trim_start_matches('\u{feff}');
                match serde_json::from_str(text) {
                    Ok(settings) => Loaded {
                        settings,
                        problem: None,
                    },
                    Err(e) => Loaded {
                        settings: Settings::default(),
                        problem: Some(format!("{}: {e}", path.display())),
                    },
                }
            }
        }
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
        Ok(())
    }

    pub fn save(&self, kit: &Kit) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        write_atomic(&kit.settings_file(), json.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kit::{tests::TempDir, Arch};

    #[test]
    fn missing_file_gives_defaults() {
        let tmp = TempDir::new("settings-missing");
        let loaded = Settings::load_from(&tmp.0.join("settings.json"));
        assert_eq!(loaded.settings, Settings::default());
        assert!(loaded.problem.is_none());
        assert!(loaded.settings.auto_update);
    }

    #[test]
    fn validation() {
        let ok = Settings {
            lang: Some("de".into()),
            python_version: Some("3.13".into()),
            php_version: Some("8.4".into()),
            rust_toolchain: Some("msvc".into()),
            ..Settings::default()
        };
        assert!(ok.validate().is_ok());
        let bad = |f: fn(&mut Settings)| {
            let mut s = Settings::default();
            f(&mut s);
            s.validate().is_err()
        };
        assert!(bad(|s| s.lang = Some("es".into())));
        assert!(bad(|s| s.network.https_proxy = Some("proxy:8080".into())));
        assert!(bad(
            |s| s.network.extra_ca_certs = Some(r"Z:\none.pem".into())
        ));
        assert!(bad(|s| s.python_version = Some("3.13 & x".into())));
        assert!(bad(|s| s.php_version = Some("7.4".into())));
        assert!(bad(|s| s.rust_toolchain = Some("x".into())));
        assert!(bad(|s| s.gemini_api_key = Some("a b".into())));
    }

    #[test]
    fn round_trip_and_partial_file() {
        let tmp = TempDir::new("settings-rt");
        let kit = Kit::at(tmp.0.clone(), Arch::X64);
        kit.prepare().unwrap();
        let s = Settings {
            lang: Some("de".into()),
            network: Network {
                allow_lan: true,
                ..Network::default()
            },
            ..Settings::default()
        };
        s.save(&kit).unwrap();
        assert_eq!(Settings::load(&kit).settings, s);

        fs::write(kit.settings_file(), "\u{feff}{\"lang\":\"fr\"}").unwrap();
        let loaded = Settings::load(&kit).settings;
        assert_eq!(loaded.lang.as_deref(), Some("fr"));
        assert!(loaded.auto_update, "missing keys take the default");
    }

    #[test]
    fn broken_file_is_reported_not_replaced() {
        let tmp = TempDir::new("settings-bad");
        let path = tmp.0.join("settings.json");
        fs::write(&path, "{ \"lang\": ").unwrap();
        let loaded = Settings::load_from(&path);
        assert!(loaded.problem.is_some());
        assert_eq!(fs::read_to_string(&path).unwrap(), "{ \"lang\": ");
    }
}
