//! Secrets never shown: in `Atollo env`, in the logs, in a panic message.

/// A variable whose name looks secret (`*_TOKEN`, `*_KEY`, `*SECRET*`,
/// `*PASSWORD*`, `*PASSWD*`, `tkn`), ignoring case.
pub fn is_secret(name: &str) -> bool {
    const SECRET: [&str; 5] = ["KEY", "TOKEN", "SECRET", "PASSWORD", "PASSWD"];
    let upper = name.to_ascii_uppercase();
    SECRET.iter().any(|s| upper.contains(s)) || upper == "TKN"
}

/// `value` of the variable `name` as it may be shown (`Atollo env`,
/// logs): `***` for a secret name ([`is_secret`]), the password of a URL
/// (`http://user:pass@proxy:8080`) masked in any other.
pub fn masked(name: &str, value: &str) -> String {
    if is_secret(name) {
        return "***".into();
    }
    let Some(scheme) = value.find("://").map(|i| i + 3) else {
        return value.to_owned();
    };
    let rest = &value[scheme..];
    let host_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    match rest[..host_end].rfind('@') {
        Some(at) => {
            let user = rest[..at].split(':').next().unwrap_or_default();
            format!("{}{user}:***{}", &value[..scheme], &rest[at..])
        }
        None => value.to_owned(),
    }
}

/// Replaces the values of variables whose name looks secret, for logs.
pub fn mask_secrets(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(eq) = rest.find('=') {
        let (before, after) = rest.split_at(eq);
        let name_start = before
            .rfind(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .map(|i| i + 1)
            .unwrap_or(0);
        out.push_str(before);
        out.push('=');
        let value = &after[1..];
        if is_secret(&before[name_start..]) {
            let quote = value.chars().next().filter(|c| *c == '"' || *c == '\'');
            let end = match quote {
                Some(q) => value[1..].find(q).map(|i| i + 2).unwrap_or(value.len()),
                None => value
                    .find(|c: char| c.is_whitespace() || c == '&' || c == '"' || c == '\'')
                    .unwrap_or(value.len()),
            };
            match quote {
                Some(q) => out.extend([q, '*', '*', '*', q]),
                None => out.push_str("***"),
            }
            rest = &value[end..];
        } else {
            rest = value;
        }
    }
    out.push_str(rest);
    out
}
