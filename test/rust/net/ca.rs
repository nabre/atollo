use super::*;
use crate::kit::tests::TempDir;
use crate::kit::Arch;

/// A certificate of this PC's store, as DER (none on a machine without
/// roots: the tests that need one stop there).
fn a_root() -> Option<Vec<u8>> {
    rustls_native_certs::load_native_certs()
        .certs
        .first()
        .map(|c| c.as_ref().to_vec())
}

#[test]
fn pem_written_reads_back() {
    let Some(der) = a_root() else { return };
    let tmp = TempDir::new("ca-pem");
    let file = tmp.0.join("extra.pem");
    fs::write(&file, pem(&[Certificate::from_der(&der)])).unwrap();
    let certs = extra_certs(&file).unwrap();
    assert_eq!(certs.len(), 1);
    assert_eq!(certs[0].der(), der.as_slice());
}

#[test]
fn a_file_without_certificates_is_refused() {
    let tmp = TempDir::new("ca-none");
    let file = tmp.0.join("empty.pem");
    fs::write(&file, "not a certificate\n").unwrap();
    assert!(extra_certs(&file).is_err());
    assert!(extra_certs(&tmp.0.join("missing.pem")).is_err());
}

#[test]
fn bundle_follows_the_setting() {
    let Some(der) = a_root() else { return };
    let tmp = TempDir::new("ca-bundle");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let extra = tmp.0.join("company.pem");
    fs::write(&extra, pem(&[Certificate::from_der(&der)])).unwrap();
    let mut settings = Settings::default();
    settings.network.extra_ca_certs = Some(extra.display().to_string());
    refresh(&kit, &settings);
    let text = fs::read_to_string(bundle_file(&kit)).unwrap();
    assert!(
        text.matches("BEGIN CERTIFICATE").count() >= 2,
        "system roots and the extra one"
    );

    settings.network.extra_ca_certs = None;
    refresh(&kit, &settings);
    assert!(!bundle_file(&kit).exists());
}
