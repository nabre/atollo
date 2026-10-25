//! Antigravity's sign-in in the Secret Service (`agents::keyring`, D-115),
//! through the `keyring` crate; written into its `login` collection, the
//! only one go-keyring reads (`agents::keyring::LOGIN_COLLECTION`).

use anyhow::{Context, Result};
use keyring::secret_service::{delete_item, get_item_attributes, get_item_secret, SsCredential};

use crate::agents::keyring::{
    login_collection, no_secret_service, split_target, unix_item, Item, NoStore, KNOWN,
};

/// go-keyring's items carry only `service` and `username`; the ones the
/// kit writes through the crate also `application` = `rust-keyring`.
/// agy's own copy wins when both are there.
pub fn list() -> Result<Vec<Item>> {
    let mut out = Vec::new();
    for (service, user) in KNOWN {
        let cred = SsCredential::new_with_no_target(service, user)?;
        let found = match cred.map_matching_items(
            |i| Ok((get_item_attributes(i)?, get_item_secret(i)?)),
            false,
        ) {
            Ok(found) => found,
            Err(keyring::Error::NoEntry) => Vec::new(),
            Err(e) => {
                let text = format!("{e:#}");
                if no_secret_service(&text) {
                    return Err(anyhow::Error::new(NoStore(text)));
                }
                return Err(e).context("Secret Service");
            }
        };
        let ours = |a: &std::collections::HashMap<String, String>| {
            a.get("application").is_some_and(|v| v == "rust-keyring")
        };
        let pick = found
            .iter()
            .find(|(a, _)| !ours(a))
            .or_else(|| found.first());
        if let Some((_, raw)) = pick {
            out.push(unix_item(service, user, raw));
        }
    }
    Ok(out)
}

/// Into the `login` collection only, whatever `default` is: go-keyring
/// reads nothing else. Without it (KeePassXC, some KWallet setups)
/// nothing is written and the restore fails closed with
/// `agents::keyring::NO_LOGIN_COLLECTION`.
pub fn write(item: &Item) -> Result<()> {
    use dbus_secret_service::{EncryptionType, SecretService};

    let Some((service, user)) = split_target(&item.target) else {
        anyhow::bail!("{}: not a go-keyring name", item.target);
    };
    let raw = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        item.blob.as_deref().unwrap_or_default(),
    )
    .context("credential blob")?;
    // The same session the `keyring` crate opens (no crypto feature).
    let ss = SecretService::connect(EncryptionType::Plain).context("Secret Service")?;
    let all = ss.get_all_collections().context("Secret Service")?;
    let login = &all[login_collection(all.iter().map(|c| &*c.path))?];
    login
        .ensure_unlocked()
        .context("Secret Service: the login collection is locked")?;
    delete(&item.target);
    // The attributes and label the crate gives its items (`application`
    // = `rust-keyring`: `list` prefers agy's own copy to these).
    let cred = SsCredential::new_with_target(Some("login"), service, user)?;
    let attributes = cred
        .attributes
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    login
        .create_item(&cred.label, attributes, &raw, true, "text/plain")
        .with_context(|| format!("Secret Service {}", item.target))?;
    Ok(())
}

pub fn delete(target: &str) {
    let Some((service, user)) = split_target(target) else {
        return;
    };
    if let Ok(cred) = SsCredential::new_with_no_target(service, user) {
        let _ = cred.map_matching_items(delete_item, false);
    }
}
