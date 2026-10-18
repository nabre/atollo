//! A MariaDB dump made readable by MySQL (D-117), line by line and byte for
//! byte: only what MySQL refuses changes, and only outside strings, quoted
//! names and plain comments, so the data of the rows never does.
//!
//! - MariaDB's executable comments `/*M!…*/` go (`NOTE_VERBOSITY`, the
//!   sandbox line `/*M!999999\- enable the sandbox mode */`, also in its
//!   older form `/*!999999\-…*/`, which MySQL's client takes for a command);
//!   a statement left empty goes with them;
//! - `CREATE SEQUENCE`: MySQL has no sequences, the filter fails naming
//!   each one (`database.sequence`);
//! - collations `uca1400_*` become MySQL's `utf8mb4_0900_ai_ci`, or
//!   `utf8mb4_0900_as_cs` / `_as_ci` for the sensitive ones (a unique key
//!   of `a` and `A` stays valid); `utf8mb3` becomes `utf8mb4` (collations
//!   and introducers too), `_nopad` collations their padded twin;
//! - the table options of Aria `PAGE_CHECKSUM=n`, `TRANSACTIONAL=n` and
//!   `ROW_FORMAT=PAGE` go, `ENGINE=Aria` is `ENGINE=InnoDB`;
//! - the `sql_mode` of routines and triggers keeps only the modes MySQL
//!   knows (MariaDB's default has `NO_AUTO_CREATE_USER`, gone in 8.0).
//!
//! What is left that MySQL still refuses stops the restore, and the
//! migration with it: MariaDB stays as it was (`migrate_engine`).

use std::io::{BufRead, Write};

use anyhow::Result;

/// What the filter changed, for the log.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Report {
    pub mariadb_comments: usize,
    pub collations: usize,
    pub charsets: usize,
    pub table_options: usize,
    pub engines: usize,
    pub sql_modes: usize,
}

/// The dump has sequences, which MySQL does not have (`database.name`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sequences(pub Vec<String>);

impl std::fmt::Display for Sequences {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MySQL has no sequences, and the MariaDB data has {} (CREATE SEQUENCE): \
             {}. Replace them with AUTO_INCREMENT columns in MariaDB (phpMyAdmin), \
             then the move to MySQL is tried again",
            self.0.len(),
            self.0.join(", ")
        )
    }
}

impl std::error::Error for Sequences {}

/// The modes of `sql_mode` MySQL 8.4 and 9.x know.
const MYSQL_MODES: [&str; 21] = [
    "ALLOW_INVALID_DATES",
    "ANSI",
    "ANSI_QUOTES",
    "ERROR_FOR_DIVISION_BY_ZERO",
    "HIGH_NOT_PRECEDENCE",
    "IGNORE_SPACE",
    "NO_AUTO_VALUE_ON_ZERO",
    "NO_BACKSLASH_ESCAPES",
    "NO_DIR_IN_CREATE",
    "NO_ENGINE_SUBSTITUTION",
    "NO_UNSIGNED_SUBTRACTION",
    "NO_ZERO_DATE",
    "NO_ZERO_IN_DATE",
    "ONLY_FULL_GROUP_BY",
    "PAD_CHAR_TO_FULL_LENGTH",
    "PIPES_AS_CONCAT",
    "REAL_AS_FLOAT",
    "STRICT_ALL_TABLES",
    "STRICT_TRANS_TABLES",
    "TIME_TRUNCATE_FRACTIONAL",
    "TRADITIONAL",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Quote {
    Single,
    Double,
    Back,
}

/// What a word or a quoted name is waited for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Expect {
    Nothing,
    /// After `USE`: the database of the statements that follow.
    Database,
    /// After `CREATE [OR REPLACE] SEQUENCE [IF NOT EXISTS]`.
    Sequence,
    /// After `sql_mode` (and `=`): its value.
    SqlMode,
}

/// The state of the filter between two lines: a string, a quoted name or
/// a comment can go on over several lines (a routine's body).
#[derive(Debug)]
pub struct Filter {
    quote: Option<Quote>,
    plain_comment: bool,
    mariadb_comment: bool,
    exec_comment: bool,
    expect: Expect,
    /// The last words outside quotes in the statement (upper case).
    words: Vec<String>,
    database: Option<String>,
    sequences: Vec<String>,
    /// A quoted name being read for `expect`.
    name: Vec<u8>,
    /// A `sql_mode` string being read.
    mode: Option<Vec<u8>>,
    pub report: Report,
}

impl Default for Filter {
    fn default() -> Filter {
        Filter {
            quote: None,
            plain_comment: false,
            mariadb_comment: false,
            exec_comment: false,
            expect: Expect::Nothing,
            words: Vec::new(),
            database: None,
            sequences: Vec::new(),
            name: Vec::new(),
            mode: None,
            report: Report::default(),
        }
    }
}

fn is_word(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'$'
}

fn starts(line: &[u8], i: usize, what: &[u8]) -> bool {
    line.len() >= i + what.len() && line[i..i + what.len()].eq_ignore_ascii_case(what)
}

/// MySQL's name of a MariaDB collation or character set (`word` outside
/// quotes), None when MySQL has it.
pub fn mysql_name(word: &str) -> Option<String> {
    let lower = word.to_ascii_lowercase();
    if lower.contains("uca1400") {
        let to = if lower.ends_with("_as_cs") || lower.ends_with("_ai_cs") {
            "utf8mb4_0900_as_cs"
        } else if lower.ends_with("_as_ci") {
            "utf8mb4_0900_as_ci"
        } else {
            "utf8mb4_0900_ai_ci"
        };
        return Some(to.to_owned());
    }
    let mut out = lower.clone();
    for (from, to) in [("_utf8mb3", "_utf8mb4"), ("utf8mb3", "utf8mb4")] {
        if let Some(rest) = out.strip_prefix(from) {
            out = format!("{to}{rest}");
            break;
        }
    }
    if out.contains("_nopad") && (out.ends_with("_ci") || out.ends_with("_bin")) {
        out = out.replace("_nopad", "");
    }
    (out != lower).then_some(out)
}

/// `value` of `sql_mode` with the modes MySQL knows only.
pub fn mysql_sql_mode(value: &str) -> String {
    value
        .split(',')
        .map(str::trim)
        .filter(|m| MYSQL_MODES.iter().any(|k| k.eq_ignore_ascii_case(m)))
        .collect::<Vec<_>>()
        .join(",")
}

impl Filter {
    fn qualified(&self, name: &str) -> String {
        match &self.database {
            Some(db) => format!("{db}.{name}"),
            None => name.to_owned(),
        }
    }

    /// A name (a word or a quoted one) read where `expect` waits for it.
    fn name_read(&mut self, name: String) {
        match self.expect {
            Expect::Database => {
                self.database = Some(name);
                self.expect = Expect::Nothing;
            }
            Expect::Sequence => {
                let q = self.qualified(&name);
                self.sequences.push(q);
                self.expect = Expect::Nothing;
            }
            Expect::Nothing | Expect::SqlMode => {}
        }
    }

    /// One line of the dump, without its end; None: the line goes (it held
    /// only what MariaDB alone runs).
    pub fn line(&mut self, line: &[u8]) -> Option<Vec<u8>> {
        let mut out: Vec<u8> = Vec::with_capacity(line.len());
        let mut dropped = false;
        let mut i = 0;
        while i < line.len() {
            let b = line[i];
            if self.mariadb_comment {
                if starts(line, i, b"*/") {
                    self.mariadb_comment = false;
                    i += 2;
                } else {
                    i += 1;
                }
                continue;
            }
            if self.plain_comment {
                out.push(b);
                if starts(line, i, b"*/") {
                    out.push(b'/');
                    self.plain_comment = false;
                    i += 2;
                } else {
                    i += 1;
                }
                continue;
            }
            if let Some(q) = self.quote {
                i = self.in_quote(q, line, i, &mut out);
                continue;
            }
            // Outside quotes and comments.
            if starts(line, i, b"/*M!") || starts(line, i, b"/*!999999\\-") {
                self.mariadb_comment = true;
                self.report.mariadb_comments += 1;
                dropped = true;
                i += 3;
                continue;
            }
            if starts(line, i, b"/*!") {
                out.extend_from_slice(b"/*!");
                self.exec_comment = true;
                i += 3;
                while i < line.len() && line[i].is_ascii_digit() {
                    out.push(line[i]);
                    i += 1;
                }
                continue;
            }
            if starts(line, i, b"/*") {
                out.extend_from_slice(b"/*");
                self.plain_comment = true;
                i += 2;
                continue;
            }
            if self.exec_comment && starts(line, i, b"*/") {
                out.extend_from_slice(b"*/");
                self.exec_comment = false;
                i += 2;
                continue;
            }
            let dash_comment =
                starts(line, i, b"--") && line.get(i + 2).is_none_or(|c| c.is_ascii_whitespace());
            if dash_comment || b == b'#' {
                out.extend_from_slice(&line[i..]);
                break;
            }
            match b {
                b'\'' | b'"' | b'`' => {
                    let q = match b {
                        b'\'' => Quote::Single,
                        b'"' => Quote::Double,
                        _ => Quote::Back,
                    };
                    self.quote = Some(q);
                    self.name.clear();
                    if self.expect == Expect::SqlMode && q != Quote::Back {
                        self.mode = Some(Vec::new());
                    }
                    out.push(b);
                    i += 1;
                }
                _ if is_word(b) => {
                    let start = i;
                    while i < line.len() && is_word(line[i]) {
                        i += 1;
                    }
                    let word = String::from_utf8_lossy(&line[start..i]).into_owned();
                    i = self.word(&word, line, i, &mut out);
                }
                b';' => {
                    self.words.clear();
                    self.expect = Expect::Nothing;
                    out.push(b);
                    i += 1;
                }
                _ => {
                    if !b.is_ascii_whitespace() && b != b'=' && b != b'@' {
                        if self.expect == Expect::SqlMode {
                            self.expect = Expect::Nothing;
                        }
                        self.words.clear();
                    }
                    out.push(b);
                    i += 1;
                }
            }
        }
        if dropped {
            let rest = String::from_utf8_lossy(&out);
            let rest = rest.trim();
            if rest.is_empty() || rest.chars().all(|c| c == ';') {
                return None;
            }
        }
        Some(out)
    }

    /// Inside a string or a quoted name from `i`; returns where it goes on.
    fn in_quote(&mut self, q: Quote, line: &[u8], mut i: usize, out: &mut Vec<u8>) -> usize {
        while i < line.len() {
            let b = line[i];
            let close = match q {
                Quote::Single => b'\'',
                Quote::Double => b'"',
                Quote::Back => b'`',
            };
            if b == b'\\' && q != Quote::Back {
                if let Some(m) = &mut self.mode {
                    m.push(b);
                    m.extend(line.get(i + 1));
                } else {
                    out.push(b);
                    out.extend(line.get(i + 1));
                }
                i += 2;
                continue;
            }
            if b == close {
                if line.get(i + 1) == Some(&close) {
                    // A doubled quote: still inside.
                    match &mut self.mode {
                        Some(m) => m.extend_from_slice(&[b, b]),
                        None => out.extend_from_slice(&[b, b]),
                    }
                    self.name.extend_from_slice(&[b]);
                    i += 2;
                    continue;
                }
                self.quote = None;
                if let Some(m) = self.mode.take() {
                    let value = String::from_utf8_lossy(&m).into_owned();
                    let kept = mysql_sql_mode(&value);
                    if kept != value {
                        self.report.sql_modes += 1;
                    }
                    out.extend_from_slice(kept.as_bytes());
                    self.expect = Expect::Nothing;
                }
                out.push(b);
                if q == Quote::Back {
                    let name = String::from_utf8_lossy(&self.name).into_owned();
                    self.name_read(name);
                }
                return i + 1;
            }
            match &mut self.mode {
                Some(m) => m.push(b),
                None => out.push(b),
            }
            self.name.push(b);
            i += 1;
        }
        // The string goes on in the next line.
        if let Some(m) = &mut self.mode {
            m.push(b'\n');
        }
        self.name.push(b'\n');
        i
    }

    /// A word outside quotes, at `line[..i]`; returns where the line goes on.
    fn word(&mut self, word: &str, line: &[u8], i: usize, out: &mut Vec<u8>) -> usize {
        let upper = word.to_ascii_uppercase();
        // `= n` or `=PAGE` after a table option, spaces allowed.
        let value_after = |from: usize| -> Option<(usize, String)> {
            let mut j = from;
            while j < line.len() && line[j] == b' ' {
                j += 1;
            }
            if line.get(j) != Some(&b'=') {
                return None;
            }
            j += 1;
            while j < line.len() && line[j] == b' ' {
                j += 1;
            }
            let start = j;
            while j < line.len() && is_word(line[j]) {
                j += 1;
            }
            (j > start).then(|| {
                (
                    j,
                    String::from_utf8_lossy(&line[start..j]).to_ascii_uppercase(),
                )
            })
        };
        match upper.as_str() {
            "PAGE_CHECKSUM" | "TRANSACTIONAL" => {
                if let Some((end, v)) = value_after(i) {
                    if v.chars().all(|c| c.is_ascii_digit()) {
                        while out.last() == Some(&b' ') {
                            out.pop();
                        }
                        self.report.table_options += 1;
                        return end;
                    }
                }
            }
            "ROW_FORMAT" => {
                if let Some((end, v)) = value_after(i) {
                    if v == "PAGE" {
                        while out.last() == Some(&b' ') {
                            out.pop();
                        }
                        self.report.table_options += 1;
                        return end;
                    }
                }
            }
            "ARIA" => {
                let before = String::from_utf8_lossy(out).to_ascii_uppercase();
                if before
                    .trim_end()
                    .trim_end_matches('=')
                    .trim_end()
                    .ends_with("ENGINE")
                {
                    out.extend_from_slice(b"InnoDB");
                    self.report.engines += 1;
                    self.words.push(upper);
                    return i;
                }
            }
            _ => {}
        }
        match self.expect {
            Expect::Database | Expect::Sequence => {
                let skip = self.expect == Expect::Sequence
                    && matches!(upper.as_str(), "IF" | "NOT" | "EXISTS");
                if !skip {
                    self.name_read(word.to_owned());
                }
            }
            Expect::SqlMode | Expect::Nothing => {}
        }
        if upper == "SEQUENCE"
            && self.words.first().map(String::as_str) == Some("CREATE")
            && self
                .words
                .iter()
                .skip(1)
                .all(|w| w == "OR" || w == "REPLACE")
        {
            self.expect = Expect::Sequence;
        }
        if upper == "USE" && self.words.is_empty() {
            self.expect = Expect::Database;
        }
        if upper == "SQL_MODE" {
            self.expect = Expect::SqlMode;
        }
        match mysql_name(word) {
            Some(name) => {
                if name.contains("0900") || name.ends_with("_ci") || name.ends_with("_bin") {
                    self.report.collations += 1;
                } else {
                    self.report.charsets += 1;
                }
                out.extend_from_slice(name.as_bytes());
            }
            None => out.extend_from_slice(word.as_bytes()),
        }
        self.words.push(upper);
        if self.words.len() > 4 {
            self.words.remove(0);
        }
        i
    }
}

/// Filters the whole dump from `input` into `output`, keeping each line's
/// end (`\n` or `\r\n`). Fails with [`Sequences`] when the dump has any, after
/// reading all of it (every sequence is named).
pub fn filter_dump(mut input: impl BufRead, mut output: impl Write) -> Result<Report> {
    let mut filter = Filter::default();
    let mut buf = Vec::new();
    loop {
        buf.clear();
        if input.read_until(b'\n', &mut buf)? == 0 {
            break;
        }
        let (body, end): (&[u8], &[u8]) = if buf.ends_with(b"\r\n") {
            (&buf[..buf.len() - 2], b"\r\n")
        } else if buf.ends_with(b"\n") {
            (&buf[..buf.len() - 1], b"\n")
        } else {
            (&buf[..], b"")
        };
        if let Some(line) = filter.line(body) {
            output.write_all(&line)?;
            output.write_all(end)?;
        }
    }
    output.flush()?;
    if !filter.sequences.is_empty() {
        return Err(Sequences(filter.sequences.clone()).into());
    }
    Ok(filter.report)
}

#[cfg(test)]
#[path = "../../../../test/rust/db/migrate_engine/filter.rs"]
mod tests;
