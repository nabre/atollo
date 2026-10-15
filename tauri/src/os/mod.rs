//! Windows and the app's processes: the Job Object every program of the kit
//! starts in (`jobs`, D-07), the processes of the PC by folder
//! (`processes`), the kit's open programs, other copies and "Rimuovi
//! l'unità" (`sessions`), one app per kit (`instance`, D-08), WebView2
//! (`webview`, D-05) and reading inside a project without following links
//! (`safe_path`, D-77).

pub mod instance;
pub mod jobs;
pub mod processes;
pub mod safe_path;
pub mod sessions;
pub mod webview;
