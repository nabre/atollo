//! The names a project may have, valid on every system.

use thiserror::Error;

/// Why a project name is not accepted.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum NameError {
    #[error("empty name")]
    Empty,
    #[error("forbidden character")]
    ForbiddenChar,
    #[error("starts with a dot or ends with a dot or a space")]
    BadEdge,
    #[error("reserved Windows name")]
    Reserved,
    #[error("longer than 100 characters")]
    TooLong,
}

pub fn validate_project_name(name: &str) -> Result<(), NameError> {
    if name.is_empty() {
        return Err(NameError::Empty);
    }
    if name.chars().count() > 100 {
        return Err(NameError::TooLong);
    }
    if name.chars().any(|c| {
        c.is_control() || matches!(c, '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|')
    }) {
        return Err(NameError::ForbiddenChar);
    }
    if name.starts_with('.') || name.ends_with('.') || name.ends_with(' ') {
        return Err(NameError::BadEdge);
    }
    let stem = name.split('.').next().unwrap_or(name).to_ascii_lowercase();
    let reserved = matches!(stem.as_str(), "con" | "prn" | "aux" | "nul")
        || ((stem.starts_with("com") || stem.starts_with("lpt"))
            && stem.len() == 4
            && stem.as_bytes()[3].is_ascii_digit());
    if reserved {
        return Err(NameError::Reserved);
    }
    Ok(())
}
