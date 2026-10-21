//! macOS and Linux: the store of this system, and the sealing key of the computer's own sign-in.

use std::fs;

use anyhow::{bail, Context, Result};
use base64::Engine;

use super::{
    known_target, open_sealed, seal, unix, Item, SealKeyMissing, Store, System, DPAPI, SEALED,
};

/// The service of the kit's own sealing key in the Keychain or the
/// Secret Service (one per kit, only while keyring-before.json exists).
const SEAL_SERVICE: &str = "Atollo";

impl System {
    fn key_entry(&self) -> Result<keyring::Entry> {
        Ok(keyring::Entry::new(SEAL_SERVICE, &self.seal_user)?)
    }

    /// The id in the key's name saved with the kit, if it is not yet: the
    /// name then survives a move of the kit (PORT-V6-06).
    fn keep_seal_id(&self) -> Result<()> {
        let (file, id) = &self.seal_id;
        if !file.exists() {
            if let Some(dir) = file.parent() {
                fs::create_dir_all(dir)?;
            }
            fs::write(file, id).with_context(|| file.display().to_string())?;
        }
        Ok(())
    }

    /// The sealing key, created on first use. Missing when it is only
    /// read: [`SealKeyMissing`].
    fn key(&self, create: bool) -> Result<[u8; 32]> {
        let entry = self.key_entry()?;
        let text = match entry.get_password() {
            Ok(t) => t,
            Err(keyring::Error::NoEntry) if create => {
                let key: [u8; 32] = random()?;
                let text = base64::engine::general_purpose::STANDARD.encode(key);
                entry.set_password(&text)?;
                text
            }
            Err(keyring::Error::NoEntry) => return Err(SealKeyMissing.into()),
            Err(e) => return Err(e.into()),
        };
        let bytes = base64::engine::general_purpose::STANDARD.decode(text.trim())?;
        bytes
            .try_into()
            .map_err(|_| anyhow::anyhow!("sealing key of the wrong size"))
    }
}

fn random<const N: usize>() -> Result<[u8; N]> {
    use std::io::Read;
    let mut out = [0u8; N];
    fs::File::open("/dev/urandom")
        .and_then(|mut f| f.read_exact(&mut out))
        .context("/dev/urandom")?;
    Ok(out)
}

impl Store for System {
    fn list(&self) -> Result<Vec<Item>> {
        unix::list()
    }
    fn write(&self, item: &Item, _persist: u32) -> Result<()> {
        if !known_target(&item.target) {
            bail!("{}: not a name agy uses here", item.target);
        }
        unix::write(item)
    }
    fn delete(&self, target: &str) {
        unix::delete(target);
    }
    /// Fails closed, like DPAPI: without the sealing key (store locked,
    /// access denied, any error) the computer's secret never reaches the
    /// kit.
    fn protect(&self, blob: &str) -> Result<String> {
        self.keep_seal_id()
            .context("sealing key: its name in the kit")?;
        let key = self
            .key(true)
            .context("sealing key in the Keychain or the Secret Service")?;
        seal(&key, &random()?, blob)
    }
    fn unprotect(&self, blob: &str) -> Result<String> {
        if blob.starts_with(DPAPI) {
            bail!("DPAPI is not available on this system");
        }
        if !blob.starts_with(SEALED) {
            return Ok(blob.to_owned());
        }
        open_sealed(&self.key(false)?, blob)
    }
    fn forget_key(&self) {
        if let Ok(entry) = self.key_entry() {
            let _ = entry.delete_credential();
        }
    }
    fn listable(&self, target: &str) -> bool {
        known_target(target)
    }
    fn undo_partial(&self) -> bool {
        true
    }
}

// The real Secret Service: Linux only.
#[cfg(all(test, not(target_os = "macos")))]
#[path = "../../../../test/rust/agents/keyring/unix_store.rs"]
mod tests;
