//! Components and installation: the catalog (`catalog`) and what each
//! component needs (`needs`), a preparation run (`runner`) and the app's
//! task around it (`task`), the final check (`checkup`), the files
//! downloaded by hand (`downloads`, D-82), the files written in
//! `data\kit\` (`kit_files`) and the app's own update (`self_update`).
//! The installers of the components are in `install`.

pub mod catalog;
pub mod checkup;
pub mod downloads;
pub mod integrity;
pub mod kit_files;
pub mod needs;
pub mod runner;
pub mod self_update;
pub mod task;
pub mod updates;

// What the whole crate reads from the catalog, by a short path
// (`crate::setup::Component`).
pub use catalog::{kit_path, Catalog, Choice, Component, Group};
