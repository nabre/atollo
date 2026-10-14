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
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            lang: None,
            network: Network::default(),
            gemini_api_key: None,
            auto_update: true,
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

    #[allow(dead_code, reason = "first caller in phase 6 (Settings page)")]
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
