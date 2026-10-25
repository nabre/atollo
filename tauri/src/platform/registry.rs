//! The keys programs write in the current user's registry (rustup's
//! uninstall entry at every install), which the kit puts back as it found
//! them (`install/native/rust.rs`). Windows only (`os/windows/registry.rs`):
//! macOS and Linux have no registry, every key reads as missing.

use anyhow::Result;

use crate::os::current::registry as imp;

/// The text values `names` of `HKCU\<key>`; `None` when the key has none
/// of them (or does not exist).
pub fn user_values(key: &str, names: &[&str]) -> Option<Vec<(String, Option<String>)>> {
    let values: Vec<_> = names
        .iter()
        .map(|n| ((*n).to_owned(), user_value(key, n)))
        .collect();
    values.iter().any(|(_, v)| v.is_some()).then_some(values)
}

/// The text value `name` of `HKCU\<key>`.
pub fn user_value(key: &str, name: &str) -> Option<String> {
    imp::user_value(key, name)
}

/// Deletes `HKCU\<key>` and what is under it.
pub fn delete_user_key(key: &str) -> Result<()> {
    imp::delete_user_key(key)
}

/// Sets the text value `name` of `HKCU\<key>`.
pub fn set_user_value(key: &str, name: &str, value: &str) -> Result<()> {
    imp::set_user_value(key, name, value)
}
