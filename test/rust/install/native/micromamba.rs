use super::*;

#[test]
fn r_versions_of_windows_only() {
    let info = serde_json::json!({"files": [
        {"version": "4.6.1", "attrs": {"subdir": "linux-64"}},
        {"version": "4.6.0", "attrs": {"subdir": "win-64"}},
        {"version": "4.5.3", "attrs": {"subdir": "win-64"}},
        {"version": "4.6.0", "attrs": {"subdir": "win-64"}},
        {"version": "4.7.0rc1", "attrs": {"subdir": "win-64"}}
    ]});
    assert_eq!(conda_versions(&info, "win-64"), ["4.6.0", "4.5.3"]);
}

#[test]
fn conda_versions_per_subdir() {
    let info = serde_json::json!({"files": [
        {"version": "2.49.0", "attrs": {"subdir": "linux-64"}},
        {"version": "2.51.0", "attrs": {"subdir": "linux-64"}},
        {"version": "2.52.0.rc1", "attrs": {"subdir": "linux-64"}},
        {"version": "2.52.0rc1", "attrs": {"subdir": "linux-64"}},
        {"version": "2.51.0", "attrs": {"subdir": "linux-64"}},
        {"version": "2.50.1", "attrs": {"subdir": "osx-arm64"}},
        {"version": "2.51.0", "attrs": {"subdir": "linux-aarch64"}},
        {"version": "2.48.1", "attrs": {"subdir": "noarch"}},
        {"attrs": {"subdir": "linux-64"}},
        {"version": "2.53.0"}
    ]});
    assert_eq!(
        conda_versions(&info, Target::LINUX_X64.conda_subdir()),
        ["2.51.0", "2.49.0"]
    );
    assert_eq!(
        conda_versions(&info, Target::DARWIN_ARM64.conda_subdir()),
        ["2.50.1"]
    );
    assert_eq!(
        conda_versions(&info, Target::LINUX_ARM64.conda_subdir()),
        ["2.51.0"]
    );
    assert!(conda_versions(&serde_json::json!({}), "linux-64").is_empty());
}

#[test]
fn micromamba_create_args_git() {
    let prefix = Path::new("/kit/data/apps/linux-x64/git/2.51");
    let args = micromamba_create_args(prefix, &["git=2.51.0"], None);
    assert_eq!(
        args,
        [
            "create",
            "--no-rc",
            "-y",
            "-q",
            "-p",
            &prefix.display().to_string(),
            "-c",
            "conda-forge",
            "--override-channels",
            "git=2.51.0",
        ]
    );
    // -p is followed by the prefix, the specs come last.
    let p = args.iter().position(|a| a == "-p").unwrap();
    assert_eq!(args[p + 1], prefix.display().to_string());
    assert_eq!(args.last().map(String::as_str), Some("git=2.51.0"));
    // R: several specs, in order.
    let r = micromamba_create_args(prefix, &["r-base=4.5.1", "libblas=*=*openblas"], None);
    assert_eq!(&r[r.len() - 2..], ["r-base=4.5.1", "libblas=*=*openblas"]);
    assert!(!r.iter().any(|a| a == "--platform"));
}

/// A kit of Linux ARM64 prepared from Linux x64: this PC's micromamba
/// makes a `linux-aarch64` environment (`--platform` before the specs); its
/// own platform needs no flag.
#[test]
fn micromamba_create_args_for_another_platform() {
    let prefix = Path::new("/kit/data/apps/linux-arm64/r/4.6");
    let specs = ["r-base=4.6.1", "libblas=*=*openblas"];
    let platform = Target::LINUX_ARM64.conda_subdir();
    let args = micromamba_create_args(prefix, &specs, Some(platform));
    let p = args.iter().position(|a| a == "--platform").unwrap();
    assert_eq!(args[p + 1], "linux-aarch64");
    assert_eq!(&args[p + 2..], specs);
}

/// A kit this PC cannot run, of the same system (Linux ARM64 prepared from
/// Linux x64): the kit's micromamba does not start here, so this PC's one
/// creates the environment. Where the PC runs the kit, or the systems
/// differ (the component is staged), the kit's own.
#[test]
fn micromamba_runs_on_this_pc() {
    use crate::kit::Target;
    assert_eq!(
        micromamba_runner_on(Target::LINUX_ARM64, Target::LINUX_X64, false),
        Target::LINUX_X64
    );
    assert_eq!(
        micromamba_runner_on(Target::LINUX_ARM64, Target::LINUX_ARM64, true),
        Target::LINUX_ARM64
    );
    assert_eq!(
        micromamba_runner_on(Target::LINUX_X64, Target::WIN32_X64, false),
        Target::LINUX_X64
    );
}
