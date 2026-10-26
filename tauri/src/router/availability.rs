//! Why a component of the catalog cannot be installed on a target, in the
//! user's language: shared by the "Linguaggi e strumenti" page, the
//! component × target grid, the actions that install what they need and
//! `Atollo components`. A component not needed on a target (ConPTY on
//! macOS and Linux, D-177) is neither: it has its own reason,
//! [`not_needed_reason`].
//!
//! The texts are pure functions of the component and the target, so every
//! target is tested from any system.

use crate::common::i18n::t;
use crate::kit::Target;
use crate::setup::Component;

/// Why `c` cannot be installed on `target`, translated: the catalog's
/// reason for that target, then "not available for <target>" when the
/// catalog has no build for it. `None` when it is available, or not
/// needed there ([`not_needed_reason`]).
pub fn unavailable_reason(c: &Component, target: Target, lang: &str) -> Option<String> {
    if c.not_needed_reason(target).is_some() {
        return None;
    }
    if let Some(reason) = c.unavailable_reason(target) {
        return Some(t(lang, reason, &[]));
    }
    (!c.is_available_on(target)).then(|| t(lang, "non disponibile per {0}", &[target.as_str()]))
}

/// The sentence for an action that cannot install `c` on `target`: "<name>
/// is not available: <reason>." with the catalog's reason, otherwise "not
/// available for <target>". `None` when `c` can be installed there: for
/// `ipc::ensure` and `Atollo components` (app/cli.rs). `None` too when it
/// is not needed there: nothing to install, nothing missing.
pub fn unavailable_message(c: &Component, target: Target, lang: &str) -> Option<String> {
    if c.is_available_on(target) || c.not_needed_reason(target).is_some() {
        return None;
    }
    if let Some(reason) = c.unavailable_reason(target) {
        let reason = t(lang, reason, &[]);
        return Some(t(lang, "{0} non è disponibile: {1}.", &[&c.name, &reason]));
    }
    Some(unavailable_msg(lang, &c.name, target))
}

/// Why `c` is not needed on `target`, translated: the catalog's
/// `notNeeded` reason (the system does its job there, D-177). `None` where
/// it is needed.
pub fn not_needed_reason(c: &Component, target: Target, lang: &str) -> Option<String> {
    c.not_needed_reason(target)
        .map(|reason| t(lang, reason, &[]))
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

#[cfg(test)]
#[path = "../../../test/rust/router/availability.rs"]
mod tests;
