use super::*;
use crate::kit::tests::TempDir;

/// The refusal of exFAT and FAT, injected so it runs on every system.
fn no_links(_: &str, _: &Path) -> io::Result<()> {
    Err(io::Error::new(io::ErrorKind::Unsupported, "no links here"))
}

/// Without links the targets are copied: a file, a folder, a link to a
/// link (whose copy comes first); a link outside, or a folder that would
/// hold itself, is skipped.
#[test]
fn links_become_copies_where_refused() {
    let tmp = TempDir::new("linkcopy");
    let dest = tmp.0.join("out");
    let pkg = dest.join("pkg");
    fs::create_dir_all(pkg.join("lib")).unwrap();
    fs::create_dir_all(pkg.join("bin")).unwrap();
    fs::write(pkg.join("lib").join("tool.sh"), "run").unwrap();
    let doc = pkg.join("share").join("doc");
    fs::create_dir_all(&doc).unwrap();
    fs::write(doc.join("a.txt"), "doc").unwrap();
    fs::write(tmp.0.join("outside.txt"), "secret").unwrap();
    let links = vec![
        (pkg.join("chain"), "bin/tool".to_owned()),
        (pkg.join("bin").join("tool"), "../lib/tool.sh".to_owned()),
        (pkg.join("doc"), "share/doc".to_owned()),
        (doc.join("up"), "..".to_owned()),
        (pkg.join("out"), "../../outside.txt".to_owned()),
    ];
    let skipped = finish_links(&dest, links, no_links).unwrap();
    assert_eq!(
        fs::read_to_string(pkg.join("bin").join("tool")).unwrap(),
        "run"
    );
    assert_eq!(fs::read_to_string(pkg.join("chain")).unwrap(), "run");
    assert_eq!(
        fs::read_to_string(pkg.join("doc").join("a.txt")).unwrap(),
        "doc"
    );
    for copy in [pkg.join("chain"), pkg.join("doc")] {
        assert!(!fs::symlink_metadata(&copy)
            .unwrap()
            .file_type()
            .is_symlink());
    }
    assert!(!pkg.join("out").exists() && !doc.join("up").exists());
    assert_eq!(skipped.len(), 2, "{skipped:?}");
}

/// Without links, a link to a script becomes a script that runs it where
/// it is (npm's `bin/npm -> ../lib/node_modules/npm/bin/npm-cli.js`
/// requires `../lib/cli.js` from there); a program stays a copy.
#[test]
fn links_to_scripts_run_them_where_they_are() {
    let tmp = TempDir::new("linkshim");
    let dest = tmp.0.join("out");
    let node = dest.join("node");
    let cli = node
        .join("lib")
        .join("node_modules")
        .join("npm")
        .join("bin");
    fs::create_dir_all(&cli).unwrap();
    fs::create_dir_all(node.join("bin")).unwrap();
    fs::write(
        cli.join("npm-cli.js"),
        "#!/usr/bin/env node\nrequire('../lib/cli.js')\n",
    )
    .unwrap();
    fs::write(node.join("bin").join("node.real"), b"\x7fELF").unwrap();
    let links = vec![
        (
            node.join("bin").join("npm"),
            "../lib/node_modules/npm/bin/npm-cli.js".to_owned(),
        ),
        (node.join("bin").join("node"), "node.real".to_owned()),
    ];
    finish_links(&dest, links, no_links).unwrap();
    assert_eq!(
        fs::read_to_string(node.join("bin").join("npm")).unwrap(),
        "#!/bin/sh\n# In place of a link to ../lib/node_modules/npm/bin/npm-cli.js (a disk without links).\n\
         exec \"$(dirname -- \"$0\")\"/'../lib/node_modules/npm/bin/npm-cli.js' \"$@\"\n"
    );
    assert_eq!(fs::read(node.join("bin").join("node")).unwrap(), b"\x7fELF");
    assert_eq!(
        relative_to(Path::new("/a/b"), Path::new("/a/c/d")).as_deref(),
        Some("../c/d")
    );
}

/// Without links, a link whose way crosses another link (`x -> lib64/foo`,
/// `lib64 -> lib`) is copied once that one is a copy; a way that still
/// leaves the folder after it stays skipped.
#[test]
fn links_through_a_copied_link_where_refused() {
    let tmp = TempDir::new("linkchain");
    let dest = tmp.0.join("out");
    fs::create_dir_all(dest.join("lib")).unwrap();
    fs::write(dest.join("lib").join("foo"), "so").unwrap();
    let links = vec![
        (dest.join("x"), "lib64/foo".to_owned()),
        (dest.join("y"), "lib64/../../secret".to_owned()),
        (dest.join("lib64"), "lib".to_owned()),
    ];
    let skipped = finish_links(&dest, links, no_links).unwrap();
    assert_eq!(fs::read_to_string(dest.join("x")).unwrap(), "so");
    assert_eq!(
        fs::read_to_string(dest.join("lib64").join("foo")).unwrap(),
        "so"
    );
    assert!(!dest.join("y").exists());
    assert_eq!(skipped.len(), 1, "{skipped:?}");

    // With links, a way through a link is never followed.
    let dest2 = tmp.0.join("out2");
    fs::create_dir_all(dest2.join("lib")).unwrap();
    let made = std::cell::RefCell::new(Vec::new());
    let links = vec![
        (dest2.join("x"), "lib64/foo".to_owned()),
        (dest2.join("lib64"), "lib".to_owned()),
    ];
    let skipped = finish_links(&dest2, links, |t, p| {
        made.borrow_mut().push((t.to_owned(), p.to_path_buf()));
        Ok(())
    })
    .unwrap();
    assert_eq!(made.borrow().len(), 1);
    assert_eq!(skipped.len(), 1, "{skipped:?}");
}

/// Without links, a folder copied in place of a link (`d -> a`) waits for a
/// link inside it that resolves only later (`a/x -> lib64/foo`, through
/// the copy of `lib64 -> lib`), so the copy holds it too.
#[test]
fn a_copied_folder_gets_links_resolved_later() {
    let tmp = TempDir::new("linklater");
    let dest = tmp.0.join("out");
    fs::create_dir_all(dest.join("a")).unwrap();
    fs::create_dir_all(dest.join("lib")).unwrap();
    fs::write(dest.join("lib").join("foo"), "so").unwrap();
    let links = vec![
        (dest.join("d"), "a".to_owned()),
        (dest.join("a").join("x"), "../lib64/foo".to_owned()),
        (dest.join("lib64"), "lib".to_owned()),
    ];
    let skipped = finish_links(&dest, links, no_links).unwrap();
    assert!(skipped.is_empty(), "{skipped:?}");
    assert_eq!(fs::read_to_string(dest.join("a").join("x")).unwrap(), "so");
    assert_eq!(fs::read_to_string(dest.join("d").join("x")).unwrap(), "so");
}

/// A link whose target can never stay inside (absolute, climbing above the
/// folder) is skipped before anything is made for it: no folder for its
/// parent, no error.
#[test]
fn hopeless_links_make_no_folder() {
    let tmp = TempDir::new("linkhopeless");
    let dest = tmp.0.join("out");
    fs::create_dir_all(&dest).unwrap();
    let links = vec![
        (dest.join("p").join("abs"), "/etc/passwd".to_owned()),
        (dest.join("q").join("up"), "../../x".to_owned()),
        (dest.join("r").join("win"), r"..\x".to_owned()),
    ];
    let skipped = finish_links(&dest, links, no_links).unwrap();
    assert_eq!(skipped.len(), 3, "{skipped:?}");
    for d in ["p", "q", "r"] {
        assert!(!dest.join(d).exists(), "{d}");
    }
}

#[test]
fn lexically_inside_by_text() {
    let top = Path::new("");
    let pkg = Path::new("pkg");
    assert!(lexically_inside(top, "a/b"));
    assert!(lexically_inside(pkg, "../x"));
    assert!(lexically_inside(pkg, "./b/../c"));
    assert!(!lexically_inside(top, "../x"));
    assert!(!lexically_inside(pkg, "../../x"));
    assert!(!lexically_inside(pkg, "a/../../../x"));
    assert!(!lexically_inside(pkg, "/abs"));
    assert!(!lexically_inside(pkg, ""));
    assert!(!lexically_inside(pkg, r"a\b"));
    assert!(!lexically_inside(pkg, "C:x"));
}

/// Links to folders nested in each other double the tree at each step once
/// each is a copy: past the budget the extraction stops with a clear error.
#[test]
fn copies_of_nested_links_have_a_budget() {
    let tmp = TempDir::new("linkbomb");
    let dest = tmp.0.join("out");
    fs::create_dir_all(dest.join("d0")).unwrap();
    fs::write(dest.join("d0").join("f"), vec![b'x'; 1024]).unwrap();
    let mut links = Vec::new();
    for i in 1..12 {
        fs::create_dir_all(dest.join(format!("d{i}"))).unwrap();
        for n in ["a", "b"] {
            links.push((dest.join(format!("d{i}")).join(n), format!("../d{}", i - 1)));
        }
    }
    let err = finish_links_within(&dest, links.clone(), no_links, 0).unwrap_err();
    assert!(err.to_string().contains("too much space"), "{err:#}");
    // The same links within the default budget: 2^11 KiB, well inside.
    let dest2 = tmp.0.join("out2");
    fs::create_dir_all(dest2.join("d0")).unwrap();
    fs::write(dest2.join("d0").join("f"), vec![b'x'; 1024]).unwrap();
    let links2: Vec<_> = links
        .into_iter()
        .map(|(p, t)| (dest2.join(p.strip_prefix(&dest).unwrap()), t))
        .collect();
    for i in 1..12 {
        fs::create_dir_all(dest2.join(format!("d{i}"))).unwrap();
    }
    assert!(finish_links(&dest2, links2, no_links).unwrap().is_empty());
    assert_eq!(tree_size(&dest2.join("d11")), 2048 * 1024);
}

/// A tar hard link where the file system has none (exFAT, FAT) is a copy;
/// another refusal is an error, not a missing file.
#[test]
fn hard_links_become_copies_where_refused() {
    let tmp = TempDir::new("hardcopy");
    let target = tmp.0.join("a");
    fs::write(&target, "data").unwrap();
    hard_link_or_copy(&target, &tmp.0.join("b"), |_, _| {
        Err(io::Error::new(io::ErrorKind::Unsupported, "no links here"))
    })
    .unwrap();
    assert_eq!(fs::read_to_string(tmp.0.join("b")).unwrap(), "data");
    let denied = hard_link_or_copy(&target, &tmp.0.join("c"), |_, _| {
        Err(io::Error::new(io::ErrorKind::PermissionDenied, "denied"))
    });
    assert!(denied.is_err());
    assert!(!tmp.0.join("c").exists());
    hard_link_or_copy(&target, &tmp.0.join("d"), |t, p| fs::hard_link(t, p)).unwrap();
    assert_eq!(fs::read_to_string(tmp.0.join("d")).unwrap(), "data");
}

/// A refusal for another reason (permissions) is an error, not a copy.
#[test]
fn other_link_errors_stop() {
    let tmp = TempDir::new("linkerr");
    let dest = tmp.0.join("out");
    fs::create_dir_all(&dest).unwrap();
    fs::write(dest.join("a"), "x").unwrap();
    let denied = |_: &str, _: &Path| -> io::Result<()> {
        Err(io::Error::new(io::ErrorKind::PermissionDenied, "denied"))
    };
    let links = vec![(dest.join("b"), "a".to_owned())];
    assert!(finish_links(&dest, links, denied).is_err());
    assert!(!dest.join("b").exists());
}

#[cfg(unix)]
#[test]
fn links_unsupported_codes() {
    for code in [libc::EPERM, libc::ENOTSUP, libc::EOPNOTSUPP] {
        assert!(
            links_unsupported(&io::Error::from_raw_os_error(code)),
            "{code}"
        );
    }
    assert!(!links_unsupported(&io::Error::from_raw_os_error(
        libc::EACCES
    )));
    assert!(!links_unsupported(&io::Error::from_raw_os_error(
        libc::ENOENT
    )));
}

#[test]
fn links_unsupported_kind() {
    assert!(links_unsupported(&io::Error::new(
        io::ErrorKind::Unsupported,
        "x"
    )));
    assert!(!links_unsupported(&io::Error::new(
        io::ErrorKind::PermissionDenied,
        "x"
    )));
}
