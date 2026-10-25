//! Why a component of the catalog cannot be installed on a target, in the
//! user's language: shared by the "Linguaggi e strumenti" page, the
//! component × target grid, the actions that install what they need and
//! `Atollo components`.
//!
//! The texts are pure functions of the component, the target, the host and
//! whether Rosetta 2 is missing (`no_rosetta`), so every target is tested
//! from any system; the `_on_host` variant asks the machine through
//! [`super::target::rosetta_missing_on_host`].

use crate::i18n::t;
use crate::kit::Target;
use crate::setup::Component;

/// "On ARM64 PCs" is said only on a Windows ARM64 PC looking at its own
/// system: from another one the target is named, as for every system.
pub fn on_arm64_pc(target: Target, host: Target) -> bool {
    target == Target::WIN32_ARM64 && host == target
}

/// Why `c` cannot be installed on `target` from `host`, translated: the
/// catalog's reason for that target, then Rosetta 2 missing on this Mac
/// (`no_rosetta`), then "not available for <target>" when the catalog has
/// no build for it. `None` when it is available, and on a Windows ARM64
/// PC for the components without an ARM64 build there (the page says it
/// its own way, `noArm64`).
pub fn unavailable_reason(
    c: &Component,
    target: Target,
    host: Target,
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
    (!c.is_available_on(target) && !on_arm64_pc(target, host))
        .then(|| t(lang, "non disponibile per {0}", &[target.as_str()]))
}

/// The sentence for an action that cannot install `c` on `target`: "<name>
/// is not available: <reason>." with the catalog's reason or Rosetta 2,
/// otherwise "not available on ARM64 PCs" / "not available for <target>".
/// `None` when `c` can be installed there.
pub fn unavailable_message(
    c: &Component,
    target: Target,
    host: Target,
    no_rosetta: bool,
    lang: &str,
) -> Option<String> {
    if c.is_available_on(target) && !no_rosetta {
        return None;
    }
    if c.unavailable_reason(target).is_some() || no_rosetta {
        let reason = unavailable_reason(c, target, host, no_rosetta, lang)?;
        return Some(t(lang, "{0} non è disponibile: {1}.", &[&c.name, &reason]));
    }
    Some(unavailable_msg(lang, &c.name, target, host))
}

/// The sentence without a reason of its own: no build for ARM64 PCs on a
/// Windows ARM64 PC, otherwise not made for that system (PHP on macOS or
/// Linux, R for Windows ARM64 seen from another PC).
pub fn unavailable_msg(lang: &str, name: &str, target: Target, host: Target) -> String {
    if on_arm64_pc(target, host) {
        t(lang, "{0} non è disponibile sui PC ARM64.", &[name])
    } else {
        t(
            lang,
            "{0} non è disponibile per {1}.",
            &[name, target.as_str()],
        )
    }
}

/// [`unavailable_message`] with Rosetta 2 probed on this host: for
/// `ipc::ensure` and `Atollo components` (cli.rs).
pub fn unavailable_message_on_host(c: &Component, target: Target, lang: &str) -> Option<String> {
    let no_rosetta = c.is_available_on(target) && super::target::rosetta_missing_on_host(target, c);
    unavailable_message(c, target, Target::detect(), no_rosetta, lang)
}

#[cfg(test)]
#[path = "../../../test/rust/router/availability.rs"]
mod tests;
