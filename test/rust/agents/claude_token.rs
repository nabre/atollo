use super::*;
use crate::kit::{tests::TempDir, Arch};

/// A made-up token of the length `claude setup-token` prints.
fn sample() -> String {
    let body: String = (0..93)
        .map(|i| {
            let set = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
            set[(i * 7 + 3) % set.len()] as char
        })
        .collect();
    format!("sk-ant-oat01-{body}AA")
}

#[test]
fn offered_on_macos_only() {
    assert!(offered(Os::Darwin));
    assert!(!offered(Os::Linux));
    assert!(!offered(Os::Win32));
}

#[test]
fn tokens() {
    assert!(is_token(&sample()));
    assert!(!is_token("sk-ant-oat01-short"));
    assert!(!is_token(
        "sk-ant-api03-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    ));
    assert!(!is_token(&format!("{} ", sample())));
    assert!(!is_token(&format!("{}\u{1b}", sample())));
    assert_eq!(
        path_in(Path::new("data")),
        Path::new("data")
            .join("agents")
            .join("claude")
            .join("oauth-token")
    );
}

/// The kit's environment (`KitEnv::build`) of `kit`.
fn kit_env(kit: &Kit) -> KitEnv {
    KitEnv::build(
        kit,
        &crate::settings::Settings::default(),
        "en",
        &crate::env::Host::from_pairs(std::iter::empty()),
    )
}

/// With the file, `CLAUDE_CODE_OAUTH_TOKEN` in the kit's environment, so
/// in every program of the kit (terminals, editor, the Agents page);
/// without it (or after "Esci") nothing. Never in `Atollo env` nor in
/// `KitEnv`'s `Debug`.
#[test]
fn environment_with_and_without_token() {
    let tmp = TempDir::new("claude-token");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    assert_eq!(kit_env(&kit).get(VAR), None);
    assert_eq!(read(&kit), None);

    assert!(save(&kit, "not a token").is_err());
    save(&kit, &format!("  {}\n", sample())).unwrap();
    assert_eq!(read(&kit), Some(sample()));
    let env = kit_env(&kit);
    assert_eq!(env.get(VAR), Some(sample().as_str()));
    let mut printed = Vec::new();
    crate::cli::print_env(&env, &mut printed).unwrap();
    let printed = String::from_utf8(printed).unwrap();
    assert!(printed.contains(&format!("{VAR}=***\n")));
    assert!(!printed.contains(PREFIX));
    assert!(!format!("{env:?}").contains(PREFIX));
    assert!(!format!("{env:#?}").contains(PREFIX));

    // A file edited by hand into something else is ignored.
    std::fs::write(path(&kit), "garbage").unwrap();
    assert_eq!(kit_env(&kit).get(VAR), None);
    let mut other = KitEnv::default();
    apply_in(&kit.data(), &mut other);
    assert_eq!(other.get(VAR), None);

    save(&kit, &sample()).unwrap();
    remove(&kit).unwrap();
    assert!(!path(&kit).exists());
    remove(&kit).unwrap();
    assert_eq!(kit_env(&kit).get(VAR), None);
}

#[cfg(unix)]
#[test]
fn token_file_is_private() {
    use std::os::unix::fs::PermissionsExt;
    let tmp = TempDir::new("claude-token-mode");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    save(&kit, &sample()).unwrap();
    let mode = |p: &Path| std::fs::metadata(p).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode(&path(&kit)), 0o600);
    assert_eq!(mode(path(&kit).parent().unwrap()), 0o700);
    // A mode that did not stick (exFAT, FAT): not owner-only.
    std::fs::set_permissions(path(&kit), std::fs::Permissions::from_mode(0o644)).unwrap();
    assert!(!owner_only(&path(&kit)));
}

#[test]
fn escape_sequences_dropped() {
    assert_eq!(
        plain_text("\u{1b}[1m\u{1b}[38;2;1;2;3mbold\u{1b}[0m\r\nnext\rover"),
        "bold\nnext\nover"
    );
    assert_eq!(
        plain_text("\u{1b}]0;title\u{7}a\u{1b}]8;;https://x\u{1b}\\link\u{1b}]8;;\u{1b}\\b"),
        "alinkb"
    );
    assert_eq!(plain_text("\u{1b}[?25l\u{1b}[2K\u{1b}[1Ax\u{7}"), "x");
}

#[test]
fn token_found_in_the_output() {
    let t = sample();
    let out = format!(
        "\u{1b}[32m✓ Long-lived authentication token created successfully!\u{1b}[39m\r\n\r\nYour OAuth token (valid for 1 year):\r\n\r\n\u{1b}[33m{t}\u{1b}[39m\r\n\r\nStore this token securely. You won't be able to see it again.\r\n"
    );
    assert_eq!(find_token(&plain_text(&out)), Some(t.clone()));
    // None without one.
    assert_eq!(
        find_token("Opening browser to sign in…\nsk-ant-oat01-"),
        None
    );
}

/// The terminal (80 columns) wrapped the token, with or without a box.
#[test]
fn wrapped_token_joined_back() {
    let t = sample();
    let (a, rest) = t.split_at(80);
    let (b, c) = rest.split_at(20);
    let plain = format!("Your OAuth token:\n\n{a}\n{b}\n{c}\n\nStore this token securely.\n");
    assert_eq!(find_token(&plain), Some(t.clone()));
    let boxed = format!(
        "╭──────╮\n│ {a} │\n│ {b}{} │\n╰──────╯\nUse this token by setting: export CLAUDE_CODE_OAUTH_TOKEN=<token>\n",
        c
    );
    assert_eq!(find_token(&boxed), Some(t.clone()));
    // The next line of text is not glued to it.
    let glued = format!("{a}\n{b}{c}\nStore this token securely.\n");
    assert_eq!(find_token(&glued), Some(t.clone()));
}

#[test]
fn last_token_wins() {
    let first = sample();
    let second = sample().replace("AA", "BB");
    let text = format!("{first}\n\n{second}\n");
    assert_eq!(find_token(&text), Some(second));
}

/// Chunks split inside an escape sequence and inside the token.
#[test]
fn scan_across_chunks() {
    let t = sample();
    let out = format!("Token:\r\n\u{1b}[33m{t}\u{1b}[39m\r\n\r\nStore it.\r\n");
    let mut scan = TokenScan::default();
    let bytes: Vec<char> = out.chars().collect();
    for piece in bytes.chunks(7) {
        scan.push(&piece.iter().collect::<String>());
    }
    assert_eq!(scan.token(), Some(t.clone()));
    assert!(!format!("{scan:?}").contains(&t), "no token in Debug");
}

#[test]
fn scan_keeps_the_end_only() {
    let mut scan = TokenScan::default();
    scan.push(&"é".repeat(TokenScan::CAP));
    scan.push(&format!("\n{}\n", sample()));
    assert!(scan.raw.len() <= TokenScan::CAP);
    assert_eq!(scan.token(), Some(sample()));
}
