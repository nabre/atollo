//! What the statistics of the three agents share (port of nabre/atollo
//! `core/stats/util.js`): thresholds, times, token counts, session files
//! and the summary of a period.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use serde_json::Value;

/// % used: green below WARN, red from CRITICAL.
pub const WARN: f64 = 70.0;
pub const CRITICAL: f64 = 90.0;
pub const DAY_MS: i64 = 24 * 3600 * 1000;
/// Larger session files are skipped.
const MAX_FILE: u64 = 64 * 1024 * 1024;

pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as i64)
}

fn ms_of(time: SystemTime) -> i64 {
    time.duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as i64)
}

/// A percentage of a limit: 0-100 with one decimal, `None` if not a number.
pub fn clamp_percent(v: f64) -> Option<f64> {
    v.is_finite()
        .then(|| (v.clamp(0.0, 100.0) * 10.0).round() / 10.0)
}

pub fn level(p: Option<f64>) -> &'static str {
    match p {
        None => "none",
        Some(p) if p >= CRITICAL => "critical",
        Some(p) if p >= WARN => "warning",
        Some(_) => "ok",
    }
}

/// A time in a JSON answer: seconds or milliseconds, or ISO-8601 text.
pub fn to_time(v: Option<&Value>) -> Option<i64> {
    match v? {
        Value::Number(n) => {
            let n = n.as_f64()?;
            let ms = if n < 1e12 { n * 1000.0 } else { n };
            (n.is_finite() && n > 0.0).then_some(ms as i64)
        }
        Value::String(s) => parse_iso(s),
        _ => None,
    }
}

/// Days from 1970-01-01 to a civil date (Howard Hinnant's algorithm).
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// `2026-10-14T08:12:33.123Z`, with `+02:00`, `+0200` or no zone (UTC);
/// a date alone is midnight UTC.
pub fn parse_iso(s: &str) -> Option<i64> {
    let s = s.trim();
    let num = |a: usize, b: usize| -> Option<i64> {
        let part = s.get(a..b)?;
        part.bytes()
            .all(|c| c.is_ascii_digit())
            .then(|| part.parse().ok())?
    };
    if s.get(4..5)? != "-" || s.get(7..8)? != "-" {
        return None;
    }
    let (y, mo, d) = (num(0, 4)?, num(5, 7)?, num(8, 10)?);
    if !(1..=12).contains(&mo) || !(1..=31).contains(&d) {
        return None;
    }
    let mut ms = days_from_civil(y, mo, d) * DAY_MS;
    if s.len() == 10 {
        return Some(ms);
    }
    if !matches!(s.get(10..11)?, "T" | "t" | " ") || s.get(13..14)? != ":" {
        return None;
    }
    let (h, mi) = (num(11, 13)?, num(14, 16)?);
    let mut rest = &s[16..];
    let mut sec = 0;
    if let Some(r) = rest.strip_prefix(':') {
        sec = r.get(..2)?.parse().ok()?;
        rest = &r[2..];
    }
    if let Some(r) = rest.strip_prefix('.').or_else(|| rest.strip_prefix(',')) {
        let digits = r.bytes().take_while(u8::is_ascii_digit).count();
        let frac = &r[..digits];
        let mut millis = 0;
        for (i, c) in frac.bytes().take(3).enumerate() {
            millis += i64::from(c - b'0') * [100, 10, 1][i];
        }
        ms += millis;
        rest = &r[digits..];
    }
    ms += ((h * 60 + mi) * 60 + sec) * 1000;
    match rest {
        "" | "Z" | "z" => Some(ms),
        zone => {
            let sign = match zone.get(..1)? {
                "+" => 1,
                "-" => -1,
                _ => return None,
            };
            let z = zone[1..].replace(':', "");
            if z.len() != 4 || !z.bytes().all(|c| c.is_ascii_digit()) {
                return None;
            }
            let minutes: i64 = z[..2].parse::<i64>().ok()? * 60 + z[2..].parse::<i64>().ok()?;
            Some(ms - sign * minutes * 60_000)
        }
    }
}

/// Start (UTC ms) of the local day `back` days before the day of `now`,
/// right across daylight-saving changes (Windows' time zone rules).
pub fn local_day_start(now: i64, back: i64) -> i64 {
    use windows::Win32::Foundation::{FILETIME, SYSTEMTIME};
    use windows::Win32::System::Time::{
        FileTimeToSystemTime, SystemTimeToFileTime, SystemTimeToTzSpecificLocalTime,
        TzSpecificLocalTimeToSystemTime,
    };
    const EPOCH_DIFF_MS: i64 = 11_644_473_600_000;
    let to_ft = |ms: i64| {
        let ticks = ((ms + EPOCH_DIFF_MS) * 10_000) as u64;
        FILETIME {
            dwLowDateTime: ticks as u32,
            dwHighDateTime: (ticks >> 32) as u32,
        }
    };
    let of_ft = |ft: FILETIME| {
        let ticks = (u64::from(ft.dwHighDateTime) << 32) | u64::from(ft.dwLowDateTime);
        (ticks / 10_000) as i64 - EPOCH_DIFF_MS
    };
    let utc_days = now.div_euclid(DAY_MS) * DAY_MS - back * DAY_MS;
    // SAFETY: every call reads and writes only the local structures passed.
    unsafe {
        let (mut utc, mut local) = (SYSTEMTIME::default(), SYSTEMTIME::default());
        if FileTimeToSystemTime(&to_ft(now), &mut utc).is_err()
            || SystemTimeToTzSpecificLocalTime(None, &utc, &mut local).is_err()
        {
            return utc_days;
        }
        local.wHour = 0;
        local.wMinute = 0;
        local.wSecond = 0;
        local.wMilliseconds = 0;
        let mut naive = FILETIME::default();
        if SystemTimeToFileTime(&local, &mut naive).is_err() {
            return utc_days;
        }
        let mut day = SYSTEMTIME::default();
        let mut start = SYSTEMTIME::default();
        let mut out = FILETIME::default();
        if FileTimeToSystemTime(&to_ft(of_ft(naive) - back * DAY_MS), &mut day).is_err()
            || TzSpecificLocalTimeToSystemTime(None, &day, &mut start).is_err()
            || SystemTimeToFileTime(&start, &mut out).is_err()
        {
            return utc_days;
        }
        of_ft(out)
    }
}

/// The period of the statistics: the 7 local days up to today, today
/// included; `days[6]` is today.
#[derive(Debug, Clone, Copy)]
pub struct Period {
    pub days: [i64; 7],
}

impl Period {
    pub fn now() -> Period {
        let now = now_ms();
        Period {
            days: std::array::from_fn(|i| local_day_start(now, 6 - i as i64)),
        }
    }
    pub fn since(&self) -> i64 {
        self.days[0]
    }
    pub fn today(&self) -> i64 {
        self.days[6]
    }
    /// The day of `time`; times after today (a clock ahead) go to today.
    fn day_of(&self, time: i64) -> usize {
        (1..7).rev().find(|i| time >= self.days[*i]).unwrap_or(0)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct Tokens {
    /// New input (Claude: cache writes included).
    pub input: u64,
    /// Input read from the cache.
    pub cached: u64,
    pub output: u64,
    pub total: u64,
}

impl Tokens {
    pub fn new(input: u64, cached: u64, output: u64) -> Tokens {
        Tokens {
            input,
            cached,
            output,
            total: input + cached + output,
        }
    }
    fn add(&mut self, o: &Tokens) {
        self.input += o.input;
        self.cached += o.cached;
        self.output += o.output;
        self.total += o.total;
    }
}

/// One reply of an agent with its tokens.
#[derive(Debug, Clone)]
pub struct Record {
    pub time: Option<i64>,
    pub model: Option<String>,
    pub tokens: Tokens,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Day {
    pub start: i64,
    #[serde(flatten)]
    pub tokens: Tokens,
    pub sessions: u32,
}

/// Tokens of today and of the 7 days, per day, sessions, last activity and
/// models, all over the same period.
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Usage {
    /// False when the agent saves no token counts (only sessions).
    pub tokens: bool,
    pub since: i64,
    pub today: Tokens,
    pub week: Tokens,
    pub sessions_today: u32,
    pub sessions_week: u32,
    pub last: Option<i64>,
    pub models: BTreeMap<String, u64>,
    pub days: Vec<Day>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Records and sessions (the time of a session's last activity) → the
/// summary of the period. A session counts in the day of its last activity.
pub fn summarize(records: &[Record], sessions: &[i64], has_tokens: bool, p: &Period) -> Usage {
    let mut u = Usage {
        tokens: has_tokens,
        since: p.since(),
        days: p
            .days
            .iter()
            .map(|start| Day {
                start: *start,
                tokens: Tokens::default(),
                sessions: 0,
            })
            .collect(),
        ..Usage::default()
    };
    for r in records {
        let Some(time) = r.time.filter(|t| *t >= p.since()) else {
            continue;
        };
        u.week.add(&r.tokens);
        if time >= p.today() {
            u.today.add(&r.tokens);
        }
        u.days[p.day_of(time)].tokens.add(&r.tokens);
        u.last = Some(u.last.map_or(time, |l| l.max(time)));
        // <synthetic>: messages written by Claude Code itself, without tokens.
        if let Some(model) = r.model.as_deref().filter(|m| *m != "<synthetic>") {
            if r.tokens.total > 0 {
                *u.models.entry(model.to_owned()).or_default() += r.tokens.total;
            }
        }
    }
    for s in sessions.iter().filter(|s| **s >= p.since()) {
        u.sessions_week += 1;
        if *s >= p.today() {
            u.sessions_today += 1;
        }
        u.days[p.day_of(*s)].sessions += 1;
        u.last = Some(u.last.map_or(*s, |l| l.max(*s)));
    }
    u
}

/// One file of sessions: path, size and last change.
pub struct Found {
    pub file: PathBuf,
    pub size: u64,
    pub modified: i64,
}

/// Files under `dir` whose name matches, changed since `since`.
pub fn recent_files(
    dir: &Path,
    matches: &dyn Fn(&str) -> bool,
    since: i64,
    max_size: Option<u64>,
) -> Vec<Found> {
    let max = max_size.unwrap_or(MAX_FILE);
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(entries) = fs::read_dir(&d) else {
            continue;
        };
        for e in entries.flatten() {
            let Ok(kind) = e.file_type() else { continue };
            if kind.is_dir() {
                stack.push(e.path());
            } else if kind.is_file() && matches(&e.file_name().to_string_lossy()) {
                let Ok(meta) = e.metadata() else { continue };
                let modified = meta.modified().map_or(0, ms_of);
                if modified >= since && meta.len() <= max {
                    out.push(Found {
                        file: e.path(),
                        size: meta.len(),
                        modified,
                    });
                }
            }
        }
    }
    out
}

/// Size, last change and what a file gave.
type Entries<T> = HashMap<PathBuf, (u64, i64, Arc<T>)>;

/// What a session file gave, parsed again only when it changes.
pub struct FileCache<T> {
    files: Mutex<Entries<T>>,
}

impl<T> FileCache<T> {
    pub fn new() -> FileCache<T> {
        FileCache {
            files: Mutex::new(HashMap::new()),
        }
    }

    pub fn get(&self, f: &Found, parse: impl FnOnce(&str) -> T) -> Arc<T> {
        if let Some((size, modified, value)) = self.lock().get(&f.file) {
            if *size == f.size && *modified == f.modified {
                return Arc::clone(value);
            }
        }
        let bytes = fs::read(&f.file).unwrap_or_default();
        let value = Arc::new(parse(&String::from_utf8_lossy(&bytes)));
        self.lock()
            .insert(f.file.clone(), (f.size, f.modified, Arc::clone(&value)));
        value
    }

    /// Files deleted or older than the period leave the cache too.
    pub fn keep_only(&self, found: &[&Found]) {
        let kept: HashSet<&PathBuf> = found.iter().map(|f| &f.file).collect();
        self.lock().retain(|file, _| kept.contains(file));
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Entries<T>> {
        self.files.lock().unwrap_or_else(|e| e.into_inner())
    }
}

/// One JSON object per line (CRLF too): `each(object, line number)`.
pub fn json_lines(text: &str, mut each: impl FnMut(&Value, usize)) {
    for (i, line) in text.lines().enumerate() {
        let line = line.trim();
        if !line.starts_with('{') {
            continue;
        }
        if let Ok(v) = serde_json::from_str::<Value>(line) {
            each(&v, i);
        }
    }
}

/// `Retry-After`: seconds or an HTTP date, in ms, at most 6 h (0: not given).
pub fn retry_after(value: Option<&str>, now: i64) -> i64 {
    let Some(v) = value.map(str::trim).filter(|v| !v.is_empty()) else {
        return 0;
    };
    let ms = if v.bytes().all(|c| c.is_ascii_digit()) {
        v.parse::<i64>().map_or(0, |s| s.saturating_mul(1000))
    } else {
        http_date(v).map_or(0, |t| t - now)
    };
    ms.clamp(0, 6 * 3600 * 1000)
}

/// `Wed, 21 Oct 2026 07:28:00 GMT` (the only form servers send today).
fn http_date(s: &str) -> Option<i64> {
    let parts: Vec<&str> = s.split_whitespace().collect();
    let [_, day, month, year, time, _] = parts.as_slice() else {
        return None;
    };
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let m = months.iter().position(|m| m.eq_ignore_ascii_case(month))? + 1;
    let iso = format!("{year}-{m:02}-{:0>2}T{time}Z", day);
    parse_iso(&iso)
}

#[cfg(test)]
#[path = "../../../../test/rust/agents/stats/util.rs"]
mod tests;
