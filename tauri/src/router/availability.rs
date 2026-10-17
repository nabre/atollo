//! Why a component of the catalog cannot be installed on a target, in the
//! user's language: shared by the "Linguaggi e strumenti" page, the
//! actions that install what they need and `Atollo components`.
//!
//! The texts are pure functions of the component, the target and whether
//! Rosetta 2 is missing (`no_rosetta`), so every target is tested from any
//! system; the `_on_host` variant asks the machine through
//! [`super::target::rosetta_missing_on_host`].

use crate::i18n::t;
use crate::kit::Target;
use crate::setup::catalog::Component;

/// Why `c` cannot be installed on `target`, translated: the catalog's
/// reason for that target, then Rosetta 2 missing on this Mac
/// (`no_rosetta`), then "not available for <target>" when the catalog has
/// no build for it. `None` when it is available, and for the components
/// without an ARM64 build on Windows ARM64 (the page says it its own way,
/// `noArm64`).
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
    (!c.is_available_on(target) && target != Target::WIN32_ARM64)
        .then(|| t(lang, "non disponibile per {0}", &[target.as_str()]))
}

/// The sentence for an action that cannot install `c` on `target`: "<name>
/// is not available: <reason>." with the catalog's reason or Rosetta 2,
/// otherwise "not available on ARM64 PCs" / "not available for <target>".
/// `None` when `c` can be installed there.
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
    Some(if target == Target::WIN32_ARM64 {
        t(lang, "{0} non è disponibile sui PC ARM64.", &[&c.name])
    } else {
        t(
            lang,
            "{0} non è disponibile per {1}.",
            &[&c.name, target.as_str()],
        )
    })
}

/// [`unavailable_message`] with Rosetta 2 probed on this host: for
/// `commands::ensure` and `Atollo components` (cli.rs).
// The callers are in files of other work packages: drop the allow once
// they use it.
#[allow(dead_code)]
pub fn unavailable_message_on_host(c: &Component, target: Target, lang: &str) -> Option<String> {
    let no_rosetta = c.is_available_on(target) && super::target::rosetta_missing_on_host(target, c);
    unavailable_message(c, target, no_rosetta, lang)
}

#[cfg(test)]
#[path = "../../../test/rust/router/availability.rs"]
mod tests;
