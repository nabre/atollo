use super::*;
use crate::kit::tests::TempDir;
use crate::setup::catalog::Catalog;

/// Asks every source for real: the lines of each versioned component,
/// then the release of each line. Prints `id line version url` for the
/// links to be checked; `cargo test live_lines -- --ignored --nocapture`.
#[test]
#[ignore]
fn live_lines() {
    let tmp = TempDir::new("live-lines");
    let kit = Kit::at(tmp.0.clone(), crate::kit::Arch::X64);
    kit.prepare().unwrap();
    let token = std::env::var("GITHUB_TOKEN").ok();
    let net = crate::net::Net::new(&crate::settings::Settings::default(), token).unwrap();
    let env = KitEnv::default();
    let cancel = AtomicBool::new(false);
    let ctx = Ctx {
        kit: &kit,
        net: &net,
        env: &env,
        cancel: &cancel,
    };
    let only = std::env::var("LIVE_ONLY").ok();
    for c in crate::versions::versioned() {
        if only.as_deref().is_some_and(|o| o != c.id) {
            continue;
        }
        let inst = installer(&c.id).unwrap();
        match inst.latest(&ctx, c) {
            Ok(r) => r
                .parts
                .iter()
                .for_each(|p| println!("LIVE\t{}\tlatest\t{}\t{}", c.id, r.version, p.url)),
            Err(e) => println!("FAIL\t{}\tlatest\t{e:#}", c.id),
        }
        let lines = match inst.lines(&ctx, c) {
            Ok(l) => l,
            Err(e) => {
                println!("FAIL\t{}\tlines\t{e:#}", c.id);
                continue;
            }
        };
        println!("LINES\t{}\t{}", c.id, lines.join(" "));
        for line in &lines {
            match inst.release_line(&ctx, c, line) {
                Ok(r) => r
                    .parts
                    .iter()
                    .for_each(|p| println!("LIVE\t{}\t{line}\t{}\t{}", c.id, r.version, p.url)),
                Err(e) => println!("FAIL\t{}\t{line}\t{e:#}", c.id),
            }
        }
    }
}

#[test]
fn every_catalog_component_has_an_installer() {
    for c in Catalog::get().all() {
        assert!(installer(&c.id).is_some(), "{} has no installer", c.id);
    }
}

#[test]
fn find_file_is_case_insensitive_and_recursive() {
    let tmp = TempDir::new("findfile");
    let deep = tmp.0.join("a").join("b");
    fs::create_dir_all(&deep).unwrap();
    fs::write(deep.join("UV.EXE"), b"").unwrap();
    assert_eq!(find_file(&tmp.0, "uv.exe"), Some(deep.join("UV.EXE")));
    assert_eq!(find_file(&tmp.0, "gh.exe"), None);
}

#[test]
fn check_key_wants_every_part_of_the_folder() {
    let tmp = TempDir::new("checkkey");
    let server = Catalog::get().find("vscodium-server").unwrap();
    let tree = tmp.0.join("tree");
    fs::create_dir_all(tree.join("out")).unwrap();
    fs::write(tree.join(r"out\server-main.js"), b"").unwrap();
    let err = check_key(server, &[], &tree).unwrap_err();
    assert!(err.to_string().contains("node.exe"), "{err}");
    fs::write(tree.join("node.exe"), b"").unwrap();
    check_key(server, &[], &tree).unwrap();

    // Parts in other folders (Maven, Gradle) are not in this tree.
    let java = Catalog::get().find("java").unwrap();
    let tree = tmp.0.join("jdk");
    fs::create_dir_all(tree.join("bin")).unwrap();
    fs::write(tree.join(r"bin\java.exe"), b"").unwrap();
    check_key(java, &[], &tree).unwrap();

    // Composer goes inside PHP's folder after the zip: not in it.
    let php = Catalog::get().find("php").unwrap();
    let tree = tmp.0.join("php");
    fs::create_dir_all(&tree).unwrap();
    fs::write(tree.join("php.exe"), b"").unwrap();
    assert!(check_key(php, &[], &tree).is_err());
    check_key(php, installer("php").unwrap().inside(), &tree).unwrap();
}
