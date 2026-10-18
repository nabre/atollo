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
    // A tree outside `data/apps/<target>`: the keys of the host's target
    // (`node.exe` on Windows, `node` elsewhere).
    let host = crate::kit::Target::detect();
    let exe = |stem: &str| crate::router::bins::exe_name(host.os, stem);
    let server = Catalog::get().find("vscodium-server").unwrap();
    let tree = tmp.0.join("tree");
    fs::create_dir_all(tree.join("out")).unwrap();
    fs::write(tree.join("out").join("server-main.js"), b"").unwrap();
    let err = check_key(server, &[], &tree).unwrap_err();
    let node = format!("vscodium-server/{}", exe("node"));
    assert!(err.to_string().contains(&node), "{err}");
    fs::write(tree.join(exe("node")), b"").unwrap();
    check_key(server, &[], &tree).unwrap();

    // Parts in other folders (Maven, Gradle) are not in this tree.
    let java = Catalog::get().find("java").unwrap();
    let tree = tmp.0.join("jdk");
    fs::create_dir_all(tree.join("bin")).unwrap();
    fs::write(tree.join("bin").join(exe("java")), b"").unwrap();
    check_key(java, &[], &tree).unwrap();

    // Composer goes inside PHP's folder after the zip: not in it.
    let php = Catalog::get().find("php").unwrap();
    let tree = tmp.0.join("php");
    let key = php.key_for(host);
    let file = crate::fsx::join_rel(&tree, key.strip_prefix("php/").unwrap());
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(file, b"").unwrap();
    assert!(check_key(php, &[], &tree).is_err());
    check_key(php, installer("php").unwrap().inside(), &tree).unwrap();
}

struct Quiet;

impl Progress for Quiet {
    fn note(&mut self, _text: &str) {}
    fn bytes(&mut self, _done: u64, _total: Option<u64>) {}
}

/// A `.tar.gz` with `files` (name, content), for a part added by hand.
fn tar_gz(path: &Path, files: &[(&str, &[u8])]) {
    let gz =
        flate2::write::GzEncoder::new(fs::File::create(path).unwrap(), flate2::Compression::fast());
    let mut tar = tar::Builder::new(gz);
    for (name, data) in files {
        let mut header = tar::Header::new_gnu();
        header.set_size(data.len() as u64);
        header.set_mode(0o755);
        header.set_cksum();
        tar.append_data(&mut header, name, *data).unwrap();
    }
    tar.into_inner().unwrap().finish().unwrap();
}

/// Issue #4: the CLI archive of PHP is there (added by hand), the FPM one
/// cannot be downloaded. Nothing is put in place: the installed `bin/php`,
/// `sbin/php-fpm`, `composer.phar` and version stay as they were, the new
/// CLI never replaces the folder without its FPM.
#[test]
fn a_failed_later_part_leaves_the_installed_php_as_it_was() {
    let tmp = TempDir::new("php-parts");
    let kit = Kit::at_target(tmp.0.clone(), crate::kit::Target::LINUX_X64);
    kit.prepare().unwrap();
    let php = Catalog::get().find("php").unwrap();

    // PHP 8.4.22 installed: its folder with every part, and its version.
    let old = crate::versions::install_dir(&kit, php, "8.4.22");
    fs::create_dir_all(old.join("bin")).unwrap();
    fs::create_dir_all(old.join("sbin")).unwrap();
    fs::write(old.join("bin").join("php"), b"old php").unwrap();
    fs::write(old.join("sbin").join("php-fpm"), b"old fpm").unwrap();
    fs::write(old.join("composer.phar"), b"old composer").unwrap();
    crate::versions::write_version(&old, "8.4.22").unwrap();
    php.save_version(&kit, "8.4.22").unwrap();

    // The same line again (8.4.23 goes in the folder of 8.4): the CLI
    // archive added by hand, the FPM one only on the network, offline.
    assert_eq!(crate::versions::install_dir(&kit, php, "8.4.23"), old);
    let downloads = crate::setup::downloads::dir(&kit);
    fs::create_dir_all(&downloads).unwrap();
    let cli = downloads.join("php-8.4.23-cli-linux-x86_64.tar.gz");
    tar_gz(&cli, &[("php", b"new php")]);
    let part = |file: &str, sha: String, dest: Dest| Part {
        url: format!("http://127.0.0.1:9/{file}"),
        file_name: file.to_owned(),
        expect: Expect::Sha256(sha),
        kind: Kind::TarGz,
        inner: Inner::Root,
        dest,
        sign: None,
    };
    let release = Release {
        version: "8.4.23".to_owned(),
        parts: vec![
            part(
                "php-8.4.23-cli-linux-x86_64.tar.gz",
                verify::sha256_hex(&cli).unwrap(),
                Dest::Own,
            ),
            part(
                "php-8.4.23-fpm-linux-x86_64.tar.gz",
                "0".repeat(64),
                Dest::Inside("php-fpm"),
            ),
        ],
    };
    let net = crate::net::Net::new(&crate::settings::Settings::default(), None).unwrap();
    let env = KitEnv::default();
    // Cancelled: the network part fails at once; the one added by hand is
    // taken without the network.
    let cancel = AtomicBool::new(true);
    let ctx = Ctx {
        kit: &kit,
        net: &net,
        env: &env,
        cancel: &cancel,
    };
    assert!(install(&ctx, php, &release, &mut Quiet).is_err());

    assert_eq!(fs::read(old.join("bin").join("php")).unwrap(), b"old php");
    assert_eq!(
        fs::read(old.join("sbin").join("php-fpm")).unwrap(),
        b"old fpm"
    );
    assert_eq!(
        fs::read(old.join("composer.phar")).unwrap(),
        b"old composer"
    );
    assert_eq!(php.installed_version(&kit).as_deref(), Some("8.4.22"));
    assert_eq!(
        fs::read_to_string(old.join(".devkit-version")).unwrap(),
        "8.4.22"
    );
}
