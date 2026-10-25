use super::*;

#[test]
fn arch_follows_env_rule() {
    assert_eq!(arch_from_env(Some("AMD64"), None), Arch::X64);
    assert_eq!(arch_from_env(Some("ARM64"), None), Arch::Arm64);
    assert_eq!(arch_from_env(Some("x86"), Some("arm64")), Arch::Arm64);
    assert_eq!(arch_from_env(None, None), Arch::X64);
}
