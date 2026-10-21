use super::*;
use crate::kit::tests::TempDir;

/// BUG-V3-06: conda writes the environment's folder into R's Makeconf
/// (`LOCAL_SOFT`); only that line changes, line endings kept.
#[test]
fn makeconf_follows_the_folder_of_r() {
    let text = "SHELL = /bin/sh\r\nLOCAL_SOFT ?= D:/Kit/data/apps/win32-x64/r/4.6/Library\r\nLIBR = -L\"$(R_HOME)/bin/x64\"\r\n";
    let new = relocated_makeconf(text, Path::new(r"E:\Kit\data\apps\win32-x64\r\4.6")).unwrap();
    assert_eq!(
        new,
        "SHELL = /bin/sh\r\nLOCAL_SOFT ?= E:/Kit/data/apps/win32-x64/r/4.6/Library\r\nLIBR = -L\"$(R_HOME)/bin/x64\"\r\n"
    );
    assert_eq!(
        relocated_makeconf(&new, Path::new(r"E:\Kit\data\apps\win32-x64\r\4.6")),
        None,
        "already right: nothing to write"
    );
    let plain = "LOCAL_SOFT = C:/old/Library\n";
    assert_eq!(
        relocated_makeconf(plain, Path::new("D:/new/r")).as_deref(),
        Some("LOCAL_SOFT = D:/new/r/Library\n")
    );
    assert_eq!(relocated_makeconf("CC = gcc\n", Path::new("D:/r")), None);
}

/// conda-forge's `bin/R` names its own sed, unquoted: the system's, so R
/// starts from a folder with a space; the rest of the script unchanged.
#[test]
fn r_script_uses_the_system_sed() {
    let text = "R_HOME_DIR=\"/k it/r/lib/R\"\n## sed\nSED=/k it/r/bin/sed\nexport SED\n  SED=x\n";
    let new = plain_r_sed(text).unwrap();
    assert_eq!(
        new,
        "R_HOME_DIR=\"/k it/r/lib/R\"\n## sed\nSED=sed\nexport SED\n  SED=x\n"
    );
    assert_eq!(plain_r_sed(&new), None, "already plain: nothing to write");
    assert_eq!(
        plain_r_sed("SED=\"/k it/sed\"\r\n").as_deref(),
        Some("SED=sed\r\n")
    );
    assert_eq!(plain_r_sed("echo $SED\n"), None);
}

/// The kit moved to another drive letter or folder: at the next start every
/// line of R on Windows (the default and a saved one) names its own folder.
#[test]
fn moving_the_kit_rewrites_makeconf_on_windows() {
    let tmp = TempDir::new("makeconf-moved");
    let write = |root: &Path, line: &str, prefix: &str| {
        let file = join_rel(
            root,
            &format!("data/apps/win32-x64/r/{line}/lib/R/etc/x64/Makeconf"),
        );
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(&file, format!("CC = gcc\nLOCAL_SOFT ?= {prefix}/Library\n")).unwrap();
        file
    };
    let old = tmp.0.join("one");
    let first = Kit::at_target(old.clone(), Target::WIN32_X64);
    first.prepare().unwrap();
    let now = tmp.0.join("two");
    fs::rename(&old, &now).unwrap();
    let old_r = old.join("data/apps/win32-x64/r/4.6");
    let files = [
        write(&now, "4.6", &old_r.to_string_lossy().replace('\\', "/")),
        write(&now, "4.5", "D:/bld/r-base_1/_h_env"),
    ];
    let kit = Kit::at_target(now.clone(), Target::WIN32_X64);
    assert_eq!(kit.prepare().unwrap(), Some(old.clone()));
    for (file, line) in files.iter().zip(["4.6", "4.5"]) {
        let prefix = now
            .join("data/apps/win32-x64/r")
            .join(line)
            .to_string_lossy()
            .replace('\\', "/");
        assert_eq!(
            fs::read_to_string(file).unwrap(),
            format!("CC = gcc\nLOCAL_SOFT ?= {prefix}/Library\n"),
            "{line}"
        );
    }

    // Off Windows R is installed again instead (D-92): nothing rewritten.
    let linux = Kit::at_target(tmp.0.join("linux"), Target::LINUX_X64);
    let file = join_rel(
        linux.root(),
        "data/apps/linux-x64/r/4.6/lib/R/etc/x64/Makeconf",
    );
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(&file, "LOCAL_SOFT ?= /old/Library\n").unwrap();
    linux.prepare().unwrap();
    assert_eq!(
        fs::read_to_string(&file).unwrap(),
        "LOCAL_SOFT ?= /old/Library\n"
    );
}
