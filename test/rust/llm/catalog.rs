use super::*;

use crate::llm::local::{part_of, valid_repo};

/// Every model of the catalog can be downloaded and checked: a repository,
/// a commit, files with a SHA-256 and a size, the parts all there, an id
/// the server and the agents accept.
#[test]
fn every_model_is_complete() {
    let models = all();
    assert!(models.len() >= 5);
    let mut ids = std::collections::BTreeSet::new();
    for m in models {
        assert!(crate::llm::valid_id(&m.id), "{}", m.id);
        assert!(ids.insert(&m.id), "{} twice", m.id);
        assert!(valid_repo(&m.repo), "{}", m.repo);
        assert_eq!(m.commit.len(), 40, "{}", m.id);
        assert!(!m.files.is_empty(), "{}", m.id);
        for f in &m.files {
            assert_eq!(f.sha256.len(), 64, "{}", f.name);
            assert!(f.sha256.bytes().all(|b| b.is_ascii_hexdigit()));
            assert!(f.bytes > 0, "{}", f.name);
            assert!(crate::install::is_plain_file_name(&f.name), "{}", f.name);
            assert!(f.name.ends_with(".gguf"), "{}", f.name);
        }
        if let Some((_, count)) = part_of(m.files[0].name.trim_end_matches(".gguf")) {
            assert_eq!(m.files.len(), count as usize, "{}: parts", m.id);
        }
        assert!(m.ctx >= 4096 && m.ram_gb > 0, "{}", m.id);
        assert!(!m.license.is_empty() && !m.desc.is_empty(), "{}", m.id);
    }
    assert!(find("gpt-oss-20b").is_some());
}

/// One model of the catalog fits a FAT32 drive: all its files under 4 GiB.
#[test]
fn a_model_fits_fat32() {
    assert!(all()
        .iter()
        .any(|m| m.files.iter().all(|f| f.bytes <= u64::from(u32::MAX))));
}

/// The descriptions are translated (they are shown in four languages).
#[test]
fn descriptions_are_in_the_catalog() {
    let catalog: serde_json::Value =
        serde_json::from_str(include_str!("../../../tauri/i18n/catalog.json")).unwrap();
    for m in all() {
        assert!(catalog.get(&m.desc).is_some(), "{}: {}", m.id, m.desc);
    }
}
