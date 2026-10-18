use super::*;

/// D-112: the asset names of oven-sh/bun v1.4.3 (checked with `gh api`
/// on 17/10/2026); the baseline build only on x64, never the musl,
/// Android or profile builds.
#[test]
fn bun_asset_per_target_and_cpu() {
    let names = [
        "bun-darwin-aarch64.zip",
        "bun-darwin-x64-baseline.zip",
        "bun-darwin-x64.zip",
        "bun-darwin-x64-baseline-profile.zip",
        "bun-linux-aarch64.zip",
        "bun-linux-aarch64-musl.zip",
        "bun-linux-x64-android-baseline.zip",
        "bun-linux-x64-baseline.zip",
        "bun-linux-x64-musl-baseline.zip",
        "bun-linux-x64.zip",
        "bun-windows-aarch64.zip",
        "bun-windows-x64-baseline.zip",
        "bun-windows-x64.zip",
        "SHASUMS256.txt",
    ];
    let pick = |target, baseline| -> Vec<&str> {
        let re = regex::Regex::new(&bun_asset_for(target, baseline)).unwrap();
        names.iter().copied().filter(|n| re.is_match(n)).collect()
    };
    for (target, usual, baseline) in [
        (
            Target::WIN32_X64,
            "bun-windows-x64.zip",
            "bun-windows-x64-baseline.zip",
        ),
        (
            Target::WIN32_ARM64,
            "bun-windows-aarch64.zip",
            "bun-windows-aarch64.zip",
        ),
        (
            Target::DARWIN_ARM64,
            "bun-darwin-aarch64.zip",
            "bun-darwin-aarch64.zip",
        ),
        (
            Target::DARWIN_X64,
            "bun-darwin-x64.zip",
            "bun-darwin-x64-baseline.zip",
        ),
        (
            Target::LINUX_X64,
            "bun-linux-x64.zip",
            "bun-linux-x64-baseline.zip",
        ),
        (
            Target::LINUX_ARM64,
            "bun-linux-aarch64.zip",
            "bun-linux-aarch64.zip",
        ),
    ] {
        assert_eq!(pick(target, false), [usual], "{target}");
        assert_eq!(pick(target, true), [baseline], "{target}");
    }
}

/// The CPU decides only for a kit of this PC's own x64 system.
#[test]
fn bun_baseline_only_for_this_pc() {
    for t in Target::ALL {
        if t.arch == Arch::Arm64 || t != Target::detect() {
            assert!(!bun_baseline(t), "{t}");
        }
    }
    let here = Target::detect();
    if here.arch == Arch::X64 {
        assert_eq!(bun_baseline(here), !cpu_has_avx2());
    }
}
