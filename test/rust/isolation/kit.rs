use super::*;

/// The kit of a project's sandbox: a v2 mixin with the sandbox's
/// variables, its name within the 64 characters of the spec, every value
/// quoted (a project's name is never YAML).
#[test]
fn a_projects_kit_is_a_mixin_with_its_variables() {
    let owner = Owner::Project {
        project: "shop: \"new\"\n- x".to_owned(),
    };
    let spec = spec_yaml("atollo-p-shop-0123", &owner);
    assert_eq!(
        spec,
        format!(
            "schemaVersion: \"2\"\nkind: mixin\nname: {}\n\
             displayName: \"Atollo: shop: \\\"new\\\"\\n- x\"\n\
             description: \"Written by Atollo (D-169): what makes this sandbox.\"\n\
             environment:\n  variables:\n    ATOLLO_SANDBOX: \"atollo-p-shop-0123\"\n\
             \x20   ATOLLO_OWNER: \"project\"\n    ATOLLO_PROJECT: \"shop: \\\"new\\\"\\n- x\"\n",
            kit_name("atollo-p-shop-0123")
        )
    );
    let name = kit_name("atollo-p-shop-0123");
    assert!(name.starts_with("atollo-") && name.len() == 31, "{name}");
    assert!(name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'));
    assert_ne!(name, kit_name("atollo-p-shop-0124"));
}

/// Written once, read back, its fingerprint the same until it changes;
/// listed with its owner, removed with its sandbox.
#[test]
fn a_kit_is_written_listed_and_removed_in_data() {
    let (_tmp, kit) = crate::test_support::prepared_kit("isolation-kit");
    let terminal = Owner::Terminal {
        terminal: "legacy".to_owned(),
        dir: "/sites/shop".to_owned(),
    };
    let (dir_t, print) = write(&kit, "atollo-t-legacy-1", &terminal).unwrap();
    assert_eq!(
        dir_t,
        kit.data()
            .join("app")
            .join("sandboxes")
            .join("atollo-t-legacy-1")
    );
    assert!(dir_t.join("spec.yaml").is_file());
    assert_eq!(applied(&kit, "atollo-t-legacy-1"), None);
    mark_applied(&kit, "atollo-t-legacy-1", &print).unwrap();
    assert_eq!(applied(&kit, "atollo-t-legacy-1"), Some(print.clone()));
    let (_, again) = write(&kit, "atollo-t-legacy-1", &terminal).unwrap();
    assert_eq!(again, print, "the same kit, the same fingerprint");

    let project = Owner::Project {
        project: "shop".to_owned(),
    };
    write(&kit, "atollo-p-shop-2", &project).unwrap();
    assert_eq!(
        all(&kit),
        [
            ("atollo-p-shop-2".to_owned(), project),
            ("atollo-t-legacy-1".to_owned(), terminal.clone()),
        ]
    );
    assert_eq!(owner(&kit, "atollo-t-legacy-1"), Some(terminal));
    assert_eq!(
        Owner::Terminal {
            terminal: "legacy".into(),
            dir: "/sites/shop".into()
        }
        .label(),
        "legacy · /sites/shop"
    );
    remove(&kit, "atollo-t-legacy-1").unwrap();
    remove(&kit, "atollo-t-legacy-1").unwrap();
    assert_eq!(all(&kit).len(), 1);
}
