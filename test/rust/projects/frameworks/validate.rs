use super::*;
use crate::projects::frameworks::{official, Variant};

#[test]
fn validation() {
    let base = official().iter().find(|p| p.id == "vite").unwrap().clone();
    let bad = |f: fn(&mut Piece)| {
        let mut list = official().to_vec();
        let mut p = base.clone();
        p.id = "mine".into();
        f(&mut p);
        list.push(p);
        validate(&list).is_err()
    };
    assert!(!bad(|_| ()));
    assert!(bad(|p| p.id = "Mine".into()));
    assert!(bad(|p| p.id = "vite".into()));
    assert!(bad(|p| p.name.clear()));
    assert!(bad(|p| p.create.clear()));
    assert!(bad(|p| p.needs.push("nothing".into())));
    assert!(bad(|p| p.steps.push("a\u{7}b".into())));
    assert!(bad(|p| p.actions[0].name.clear()));
    assert!(bad(|p| p.home = "http://x".into()));
    assert!(bad(|p| {
        p.kind = Kind::Addon;
        p.on.insert("nothing".into(), Variant::default());
    }));
    assert!(bad(|p| p.kind = Kind::Addon));
    assert!(bad(|p| p.database = Some("oracle".into())));
    assert!(!bad(|p| p.database = Some("mysql".into())));
    assert!(bad(|p| {
        p.kind = Kind::Addon;
        p.on.insert(
            "vite".into(),
            Variant {
                database: Some("oracle".into()),
                ..Variant::default()
            },
        );
    }));
}
