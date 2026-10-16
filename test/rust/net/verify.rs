use super::*;
use crate::kit::tests::TempDir;

#[test]
fn known_hashes() {
    let tmp = TempDir::new("verify");
    let file = tmp.0.join("abc.txt");
    fs::write(&file, b"abc").unwrap();
    let sha256 = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
    let sha512 = "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f";
    assert_eq!(sha256_hex(&file).unwrap(), sha256);
    check(&file, &Expect::Sha256(sha256.to_uppercase())).unwrap();
    check(&file, &Expect::Sha512(sha512.into())).unwrap();
    let sha3 = "3a985da74fe225b2045c172d6bd390bd855f086e3e9d525b46bfe24511431532";
    check(&file, &Expect::Sha3_256(sha3.into())).unwrap();
    let integrity = format!(
        "sha512-{}",
        base64::engine::general_purpose::STANDARD.encode(hash_file::<Sha512>(&file).unwrap())
    );
    check(&file, &Expect::NpmIntegrity(integrity)).unwrap();
}

#[test]
fn mismatch_deletes_the_file() {
    let tmp = TempDir::new("verify-bad");
    let file = tmp.0.join("x.zip");
    fs::write(&file, b"tampered").unwrap();
    assert!(check(&file, &Expect::Sha256("00".repeat(32))).is_err());
    assert!(!file.exists());
}
