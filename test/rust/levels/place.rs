use super::*;
use crate::kit::{Kit, Os, Target};
use crate::platform::safe_path::tests::{junction, remove_junction};
use crate::test_support::{kit_arch, TempDir};
use std::fs;
use std::path::PathBuf;

/// A kit of the system running the tests (`place_of` reads its disk) with
/// the project `shop` and its `web/src`.
fn kit_with_shop(name: &str) -> (TempDir, Kit) {
    let tmp = TempDir::new(name);
    let kit = Kit::at_target(tmp.0.clone(), Target::new(Os::current(), kit_arch()));
    fs::create_dir_all(kit.projects().join("shop").join("web").join("src")).unwrap();
    (tmp, kit)
}

fn shop(kit: &Kit, below: Option<&str>) -> Place {
    Place::Project {
        name: "shop".into(),
        dir: kit.projects().join("shop"),
        below: below.map(str::to_owned),
    }
}

/// A kit reached through an 8.3 short name (`C:\Users\RUNNER~1\…` on the
/// CI) and a folder given with long names, or the other way round, are the
/// same place. Skipped where the volume makes no short names.
#[cfg(windows)]
#[test]
fn short_and_long_names_are_the_same_place() {
    use std::os::windows::ffi::{OsStrExt, OsStringExt};
    use windows::Win32::Storage::FileSystem::GetShortPathNameW;
    let (tmp, _) = kit_with_shop("levels-place-short");
    let long_root = tmp.0.join("A Long Folder Name");
    fs::create_dir_all(long_root.join("projects").join("shop").join("web")).unwrap();
    let wide: Vec<u16> = long_root.as_os_str().encode_wide().chain([0]).collect();
    let mut buf = vec![0u16; 1024];
    // SAFETY: `wide` ends with a NUL and `buf` lives for the call.
    let got =
        unsafe { GetShortPathNameW(windows::core::PCWSTR(wide.as_ptr()), Some(&mut buf)) } as usize;
    if got == 0 || got >= buf.len() {
        return;
    }
    let short_root = PathBuf::from(std::ffi::OsString::from_wide(&buf[..got]));
    if short_root == long_root {
        return;
    }
    for (kit_root, given) in [(&short_root, &long_root), (&long_root, &short_root)] {
        let kit = Kit::at_target(kit_root.clone(), Target::new(Os::Win32, kit_arch()));
        let place = place_of(&kit, &given.join("projects").join("shop").join("web"));
        assert_eq!(
            place,
            Place::Project {
                name: "shop".into(),
                dir: kit.projects().join("shop"),
                below: Some("web".into()),
            },
            "kit at {kit_root:?}, folder under {given:?}"
        );
    }
}

#[test]
fn the_project_of_a_subfolder() {
    let (_tmp, kit) = kit_with_shop("levels-place-sub");
    let projects = kit.projects();
    assert_eq!(place_of(&kit, &projects.join("shop")), shop(&kit, None));
    assert_eq!(
        place_of(&kit, &projects.join("shop").join("web").join("src")),
        shop(&kit, Some("web/src"))
    );
    // `.` and `..` are read as text, never climbing out unseen.
    assert_eq!(
        place_of(
            &kit,
            &projects.join("shop").join("web").join("..").join("web")
        ),
        shop(&kit, Some("web"))
    );
    // A folder below that does not exist yet still belongs to the project.
    assert_eq!(
        place_of(&kit, &projects.join("shop").join("new")),
        shop(&kit, Some("new"))
    );
    // A folder out of `projects/`, the kit's own folders included.
    for dir in [kit.root().join("data"), kit.root().to_path_buf()] {
        assert_eq!(place_of(&kit, &dir), Place::Folder { dir: dir.clone() });
    }
    let outside = std::env::temp_dir();
    assert_eq!(
        place_of(&kit, &outside),
        Place::Folder {
            dir: outside.clone()
        }
    );
    // A project that does not exist is no project.
    let ghost = projects.join("ghost");
    assert_eq!(place_of(&kit, &ghost), Place::Folder { dir: ghost });
    // A relative folder says nothing of where it is.
    let relative = PathBuf::from("projects").join("shop");
    assert!(matches!(place_of(&kit, &relative), Place::Folder { .. }));
}

#[test]
fn a_link_is_never_a_project() {
    let (tmp, kit) = kit_with_shop("levels-place-link");
    let projects = kit.projects();
    let outside = tmp.0.join("outside");
    fs::create_dir_all(outside.join("src")).unwrap();
    // A project folder that is a link.
    let linked = projects.join("linked");
    junction(&linked, &outside);
    assert!(matches!(place_of(&kit, &linked), Place::Folder { .. }));
    assert!(matches!(
        place_of(&kit, &linked.join("src")),
        Place::Folder { .. }
    ));
    // A link below a project: what is behind it is not the project's.
    let inner = projects.join("shop").join("out");
    junction(&inner, &outside);
    assert!(matches!(
        place_of(&kit, &inner.join("src")),
        Place::Folder { .. }
    ));
    assert!(matches!(place_of(&kit, &inner), Place::Folder { .. }));
    // The project itself is still one.
    assert_eq!(
        place_of(&kit, &projects.join("shop").join("web")),
        shop(&kit, Some("web"))
    );
    remove_junction(&inner);
    remove_junction(&linked);
    // `projects/` itself a link: nothing in it is a project.
    let kit2_root = tmp.0.join("kit2");
    fs::create_dir_all(&kit2_root).unwrap();
    let kit2 = Kit::at_target(kit2_root, kit.target());
    junction(&kit2.projects(), &projects);
    assert!(matches!(
        place_of(&kit2, &kit2.projects().join("shop")),
        Place::Folder { .. }
    ));
    remove_junction(&kit2.projects());
}

#[test]
fn projects_itself_is_a_folder() {
    let (_tmp, kit) = kit_with_shop("levels-place-projects");
    let projects = kit.projects();
    assert_eq!(
        place_of(&kit, &projects),
        Place::Folder {
            dir: projects.clone()
        }
    );
}

/// Without regard to case where the file system has none by default
/// (Windows, macOS), the project's name as it is on disk; exact elsewhere.
#[test]
fn case_rules_follow_the_file_system() {
    let (_tmp, kit) = kit_with_shop("levels-place-case");
    let upper = kit.projects().join("SHOP").join("web");
    let place = place_of(&kit, &upper);
    if kit.os().is_windows() || kit.os().is_macos() {
        assert_eq!(place, shop(&kit, Some("web")));
    } else {
        assert_eq!(place, Place::Folder { dir: upper });
    }
    // A verbatim path (`\\?\D:\…`, as `canonicalize` gives on Windows) is
    // the same folder.
    if kit.os().is_windows() {
        let verbatim = PathBuf::from(format!(r"\\?\{}", kit.projects().join("shop").display()));
        assert_eq!(place_of(&kit, &verbatim), shop(&kit, None));
    }
}
