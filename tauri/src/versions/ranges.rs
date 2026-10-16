//! Version ranges as projects and tools declare them, for the relations
//! matrix (`versions/relations/`): npm and Composer (`^20.1`, `~8.2`, `>=18 <23`,
//! `8.1.x`, `1.2 - 1.4`, `||` and `|`), PEP 440 (`>=3.10,<3.13`, `~=3.11`,
//! `==3.12.*`, `!=3.12.1`) and plain minimums (`go 1.21`, CMake's
//! `cmake_minimum_required`). Pre-release tags are ignored: `3.15.0a1` is
//! `3.15.0`.

use std::cmp::Ordering;

/// `major.minor.patch`, missing parts as 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version(pub u64, pub u64, pub u64);

impl Version {
    /// The first `a.b.c` in `text` (`v22.11.0`, `go1.25.3`,
    /// `8.4.13 (Composer 2.8.12)` → 8.4.13); `None` without a number.
    pub fn parse(text: &str) -> Option<Version> {
        let start = text.find(|c: char| c.is_ascii_digit())?;
        let mut parts = text[start..]
            .split(|c: char| !c.is_ascii_digit() && c != '.')
            .next()?
            .split('.')
            .map(|p| p.parse::<u64>().ok());
        let major = parts.next()??;
        let minor = parts.next().flatten().unwrap_or(0);
        let patch = parts.next().flatten().unwrap_or(0);
        Some(Version(major, minor, patch))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    Ge,
    Gt,
    Le,
    Lt,
    Eq,
    Ne,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Bound(Op, Version);

impl Bound {
    fn admits(&self, v: Version) -> bool {
        let c = v.cmp(&self.1);
        match self.0 {
            Op::Ge => c != Ordering::Less,
            Op::Gt => c == Ordering::Greater,
            Op::Le => c != Ordering::Greater,
            Op::Lt => c == Ordering::Less,
            Op::Eq => c == Ordering::Equal,
            Op::Ne => c != Ordering::Equal,
        }
    }
}

/// How a bare version (`1.21`, no operator) reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bare {
    /// npm, Composer, PEP 440 `==3.12.*`: the versions it names (`1.2` =
    /// any 1.2.x).
    Exact,
    /// Go's `go` line, CMake, Zig: at least that version.
    Minimum,
}

/// One or more alternatives (`||`), each the bounds that must all hold.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Range {
    any: Vec<Vec<Bound>>,
}

/// The parts written in `v` (`8` = 1, `8.2` = 2, `8.2.1` = 3), with `x`,
/// `X` and `*` as missing.
fn partial(v: &str) -> Option<(Version, usize)> {
    let v = v.trim().trim_start_matches(['v', 'V', '=']);
    let mut n = Vec::new();
    for p in v.split('.') {
        let digits: String = p.chars().take_while(char::is_ascii_digit).collect();
        if digits.is_empty() {
            break; // x, *, or a pre-release tag
        }
        n.push(digits.parse::<u64>().ok()?);
        if digits.len() != p.len() {
            break;
        }
    }
    if n.is_empty() {
        return None;
    }
    let len = n.len().min(3);
    Some((
        Version(
            n[0],
            n.get(1).copied().unwrap_or(0),
            n.get(2).copied().unwrap_or(0),
        ),
        len,
    ))
}

/// The version after the last part written: `1.2` → 1.3.0, `1` → 2.0.0.
fn next(v: Version, len: usize) -> Version {
    match len {
        1 => Version(v.0 + 1, 0, 0),
        2 => Version(v.0, v.1 + 1, 0),
        _ => Version(v.0, v.1, v.2 + 1),
    }
}

/// The bounds of one comparator (`^8.2`, `>=18`, `1.2.x`).
fn comparator(c: &str, bare: Bare) -> Option<Vec<Bound>> {
    let c = c.trim();
    let (op, rest) = ["~=", ">=", "<=", "==", "!=", ">", "<", "=", "^", "~"]
        .iter()
        .find_map(|op| c.strip_prefix(op).map(|r| (*op, r)))
        .unwrap_or(("", c));
    if matches!(rest.trim(), "*" | "x" | "X") || (op.is_empty() && rest.trim().is_empty()) {
        return Some(Vec::new()); // any version
    }
    let (v, len) = partial(rest)?;
    let range = |lo: Version, hi: Version| vec![Bound(Op::Ge, lo), Bound(Op::Lt, hi)];
    Some(match op {
        ">=" => vec![Bound(Op::Ge, v)],
        ">" if len < 3 => vec![Bound(Op::Ge, next(v, len))],
        ">" => vec![Bound(Op::Gt, v)],
        "<=" if len < 3 => vec![Bound(Op::Lt, next(v, len))],
        "<=" => vec![Bound(Op::Le, v)],
        "<" => vec![Bound(Op::Lt, v)],
        "!=" if len < 3 => vec![], // `!=3.12.*`: too rare to model, ignored
        "!=" => vec![Bound(Op::Ne, v)],
        // `^1.2.3` <2, `^0.2.3` <0.3, `^0.0.3` <0.0.4
        "^" => {
            let hi = if v.0 > 0 || len == 1 {
                Version(v.0 + 1, 0, 0)
            } else if v.1 > 0 || len == 2 {
                Version(0, v.1 + 1, 0)
            } else {
                Version(0, 0, v.2 + 1)
            };
            range(v, hi)
        }
        // `~1.2.3` <1.3, `~1.2` <1.3, `~1` <2
        "~" => range(v, if len == 1 { next(v, 1) } else { next(v, 2) }),
        // PEP 440 `~=3.11` <4, `~=3.11.2` <3.12
        "~=" => range(v, next(v, (len - 1).max(1))),
        "=" | "==" if len == 3 => vec![Bound(Op::Eq, v)],
        "=" | "==" => range(v, next(v, len)),
        _ if bare == Bare::Minimum => vec![Bound(Op::Ge, v)],
        _ if len == 3 => vec![Bound(Op::Eq, v)],
        _ => range(v, next(v, len)),
    })
}

impl Range {
    /// `None` when nothing in `text` is a version range.
    pub fn parse(text: &str, bare: Bare) -> Option<Range> {
        let mut any = Vec::new();
        // Composer also writes a single `|` for "or".
        for alt in text.split("||").flat_map(|a| a.split('|')) {
            let alt = alt.trim();
            if alt.is_empty() {
                continue;
            }
            let mut all = Vec::new();
            // `1.2 - 1.4`: from 1.2 to any 1.4.x.
            if let Some((lo, hi)) = alt.split_once(" - ") {
                let (lo, _) = partial(lo)?;
                let (hi, len) = partial(hi)?;
                all.push(Bound(Op::Ge, lo));
                all.push(if len == 3 {
                    Bound(Op::Le, hi)
                } else {
                    Bound(Op::Lt, next(hi, len))
                });
            } else {
                // Operators may stand apart from their version: `>= 18`.
                let joined = alt
                    .replace(", ", ",")
                    .replace(">= ", ">=")
                    .replace("<= ", "<=")
                    .replace("> ", ">")
                    .replace("< ", "<")
                    .replace("^ ", "^")
                    .replace("~ ", "~")
                    .replace("== ", "==")
                    .replace("!= ", "!=");
                for c in joined.split([',', ' ']).filter(|c| !c.is_empty()) {
                    all.extend(comparator(c, bare)?);
                }
            }
            any.push(all);
        }
        (!any.is_empty()).then_some(Range { any })
    }

    pub fn admits(&self, v: Version) -> bool {
        self.any.iter().any(|all| all.iter().all(|b| b.admits(v)))
    }

    /// `version` (any text with a version in it) is in the range.
    pub fn admits_text(&self, version: &str) -> bool {
        Version::parse(version).is_some_and(|v| self.admits(v))
    }

    /// The minor line `22.11` has a version in the range (any patch).
    pub fn admits_line(&self, line: &str) -> bool {
        let Some(Version(major, minor, _)) = Version::parse(line) else {
            return false;
        };
        // Its first and a late patch: ranges are about lines, not patches.
        [0, 999]
            .iter()
            .any(|p| self.admits(Version(major, minor, *p)))
    }
}

#[cfg(test)]
#[path = "../../../test/rust/versions/ranges.rs"]
mod tests;
