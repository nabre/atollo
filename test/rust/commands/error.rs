use super::*;
use anyhow::Context;

#[test]
fn serializes_as_the_plain_string() {
    let e = CmdError::from("Progetto non trovato: «x»\n\"y\"");
    assert_eq!(
        serde_json::to_string(&e).unwrap(),
        serde_json::to_string("Progetto non trovato: «x»\n\"y\"").unwrap()
    );
    // What Tauri sends for an `Err`: the same as for `Err(String)`.
    let sent = |e: tauri::ipc::InvokeError| e.0;
    assert_eq!(
        sent(CmdError::from("forbidden").into()),
        sent(String::from("forbidden").into())
    );
    assert_eq!(
        sent(CmdError::from("forbidden").into()),
        serde_json::json!("forbidden")
    );
}

#[test]
fn conversions_keep_the_text() {
    assert_eq!(CmdError::from("a").to_string(), "a");
    assert_eq!(CmdError::from(String::from("b")).to_string(), "b");
    let e = anyhow::anyhow!("root").context("outer");
    assert_eq!(CmdError::from(e).to_string(), "outer: root");
    let q = || -> CmdResult<()> {
        Err(anyhow::anyhow!("disk full")).context("writing")?;
        Ok(())
    };
    assert_eq!(q().unwrap_err().to_string(), "writing: disk full");
}

#[test]
fn or_fail_sends_the_short_or_whole_text() {
    let chain = || Err::<(), _>(anyhow::anyhow!("root").context("outer"));
    assert_eq!(chain().or_fail("x").unwrap_err().to_string(), "outer");
    assert_eq!(
        chain().or_fail_chain("x").unwrap_err().to_string(),
        "outer: root"
    );
    let io = Err::<(), _>(std::io::Error::other("denied"));
    assert_eq!(io.or_fail("x").unwrap_err().to_string(), "denied");
    assert_eq!(
        fail("ctx", "unknown server x").to_string(),
        "unknown server x"
    );
    assert_eq!(Ok::<u8, String>(1).or_fail("x"), Ok(1));
}
