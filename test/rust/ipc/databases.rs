use super::*;

/// The interface knows `mysql`, `mongodb` and `sqlite`: each once, unknown
/// ids left out.
#[test]
fn detected_servers_as_the_projects() {
    let found = |ids: &[&str]| ids.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
    assert_eq!(
        detected_as(found(&["mysql", "sqlite", "redis", "mysql"])),
        ["mysql", "sqlite"]
    );
    assert_eq!(
        detected_as(found(&["mysql", "mongodb"])),
        ["mysql", "mongodb"]
    );
    assert!(detected_as(found(&["apache"])).is_empty());
}
