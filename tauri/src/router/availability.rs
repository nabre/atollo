//! Why a component of the catalog cannot be installed on a target, in the
//! user's language: shared by the "Linguaggi e strumenti" page, the
//! component × target grid, the actions that install what they need and
//! `Atollo components`.
//!
//! The texts are pure functions of the component, the target and whether
//! Rosetta 2 is missing (`no_rosetta`), so every target is tested from any
//! system; the `_on_host` variant asks the machine through
//! [`super::target::rosetta_missing_on_host`].

use crate::common::i18n::t;
use crate::kit::Target;
use crate::setup::Component;

/// Why `c` cannot be installed on `target`, translated: the catalog's
/// reason for that target, then Rosetta 2 missing on this Mac
/// (`no_rosetta`), then "not available for <target>" when the catalog has
/// no build for it. `None` when it is available.
pub fn unavailable_reason(
    c: &Component,
    target: Target,
    no_rosetta: bool,
    lang: &str,
) -> Option<String> {
    if let Some(reason) = c.unavailable_reason(target) {
        return Some(t(lang, reason, &[]));
    }
    if no_rosetta {
        return Some(t(
            lang,
            "richiede Rosetta 2: installala con softwareupdate --install-rosetta (serve un amministratore)",
            &[],
        ));
    }
    (!c.is_available_on(target)).then(|| t(lang, "non disponibile per {0}", &[target.as_str()]))
}

/// The sentence for an action that cannot install `c` on `target`: "<name>
/// is not available: <reason>." with the catalog's reason or Rosetta 2,
/// otherwise "not available for <target>". `None` when `c` can be
/// installed there.
pub fn unavailable_message(
    c: &Component,
    target: Target,
    no_rosetta: bool,
    lang: &str,
) -> Option<String> {
    if c.is_available_on(target) && !no_rosetta {
        return None;
    }
    if c.unavailable_reason(target).is_some() || no_rosetta {
        let reason = unavailable_reason(c, target, no_rosetta, lang)?;
        return Some(t(lang, "{0} non è disponibile: {1}.", &[&c.name, &reason]));
    }
    Some(unavailable_msg(lang, &c.name, target))
}

/// The sentence without a reason of its own: not made for that system
/// (PHP on macOS or Linux).
pub fn unavailable_msg(lang: &str, name: &str, target: Target) -> String {
    t(
        lang,
        "{0} non è disponibile per {1}.",
        &[name, target.as_str()],
    )
}

/// [`unavailable_message`] with Rosetta 2 probed on this host: for
/// `ipc::ensure` and `Atollo components` (app/cli.rs).
pub fn unavailable_message_on_host(c: &Component, target: Target, lang: &str) -> Option<String> {
    let no_rosetta = c.is_available_on(target) && super::target::rosetta_missing_on_host(target, c);
    unavailable_message(c, target, no_rosetta, lang)
}

#[cfg(test)]
#[path = "../../../test/rust/router/availability.rs"]
mod tests;
