//! The checks of the instructions, official or changed in Settings, before
//! they are saved or used.

use std::collections::BTreeSet;

use super::{Action, EnvVar, Kind, Piece};

/// A problem: the Italian source and its argument (translated by the caller).
pub type Problem = (&'static str, Vec<String>);

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 40
        && id.starts_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit())
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

fn check_commands(lines: &[String]) -> Result<(), Problem> {
    match lines.iter().find(|l| l.chars().any(char::is_control)) {
        Some(l) => Err(("Comando non valido: {0}", vec![l.clone()])),
        None => Ok(()),
    }
}

fn check_actions(actions: &[Action]) -> Result<(), Problem> {
    for a in actions {
        if a.name.is_empty() {
            return Err(("Azione senza nome: {0}", vec![a.command.clone()]));
        }
        if a.command.is_empty() {
            return Err(("Azione senza comando: {0}", vec![a.name.clone()]));
        }
        if a.name.len() > 60 || a.name.chars().any(char::is_control) {
            return Err(("Valore non valido: {0}", vec![a.name.clone()]));
        }
        check_commands(std::slice::from_ref(&a.command))?;
    }
    Ok(())
}

fn valid_env_key(key: &str) -> bool {
    key.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
        && key.len() <= 80
        && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Variables of `.env`: a name of letters, digits and `_`, a value on one line.
pub fn check_env(env: &[EnvVar]) -> Result<(), Problem> {
    for v in env {
        if !valid_env_key(&v.key) {
            return Err(("Variabile non valida: {0}", vec![v.key.clone()]));
        }
        if v.value.chars().any(char::is_control) {
            return Err(("Valore non valido: {0}", vec![v.key.clone()]));
        }
    }
    Ok(())
}

fn check_database(database: Option<&String>) -> Result<(), Problem> {
    match database {
        Some(db) if !crate::db::services::SERVERS.contains(&db.as_str()) => {
            Err(("Server di database sconosciuto: {0}", vec![db.clone()]))
        }
        _ => Ok(()),
    }
}

fn check_needs(needs: &[String]) -> Result<(), Problem> {
    let catalog = crate::setup::catalog::Catalog::get();
    match needs.iter().find(|n| catalog.find(n).is_none()) {
        Some(n) => Err(("Strumento del kit sconosciuto: {0}", vec![n.clone()])),
        None => Ok(()),
    }
}

/// Checks a whole list (cleaned) as it would be saved.
pub fn validate(pieces: &[Piece]) -> Result<(), Problem> {
    let mut ids = BTreeSet::new();
    for p in pieces {
        if !valid_id(&p.id) {
            return Err(("Identificativo non valido: {0}", vec![p.id.clone()]));
        }
        if !ids.insert(p.id.as_str()) {
            return Err(("Identificativo ripetuto: {0}", vec![p.id.clone()]));
        }
        if p.name.is_empty() || p.name.len() > 60 {
            return Err(("Nome non valido: {0}", vec![p.id.clone()]));
        }
        if !p.home.is_empty() && !p.home.starts_with("https://") {
            return Err(("Indirizzo non valido: {0}", vec![p.home.clone()]));
        }
    }
    let bases: BTreeSet<&str> = pieces
        .iter()
        .filter(|p| p.kind == Kind::Base)
        .map(|p| p.id.as_str())
        .collect();
    for p in pieces {
        check_needs(&p.needs)?;
        check_commands(&p.create)?;
        check_commands(&p.steps)?;
        check_actions(&p.actions)?;
        check_env(&p.env)?;
        check_database(p.database.as_ref())?;
        match p.kind {
            Kind::Base => {
                if p.create.is_empty() {
                    return Err((
                        "{0}: manca il comando che crea il progetto.",
                        vec![p.name.clone()],
                    ));
                }
            }
            Kind::Addon => {
                if p.on.is_empty() {
                    return Err(("{0}: scegli almeno una base.", vec![p.name.clone()]));
                }
                for (base, v) in &p.on {
                    if !bases.contains(base.as_str()) {
                        return Err(("Base sconosciuta: {0}", vec![base.clone()]));
                    }
                    check_needs(&v.needs)?;
                    check_database(v.database.as_ref())?;
                    check_commands(&v.create)?;
                    check_commands(&v.steps)?;
                    check_actions(&v.actions)?;
                    check_env(&v.env)?;
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../../test/rust/projects/frameworks/validate.rs"]
mod tests;
