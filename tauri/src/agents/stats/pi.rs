//! Pi (D-152): usage from its sessions (`sessions\<folder>\*.jsonl`): the
//! `usage` of every reply (`input`, `cacheRead`, `cacheWrite`, `output`),
//! and of the summaries and the cache warming Pi records apart. No plan
//! limits: the local models have none, the providers of the cloud are not
//! asked by the kit.

use std::path::Path;
use std::sync::LazyLock;

use serde_json::Value;

use super::util::{
    json_lines, recent_files, summarize, to_time, FileCache, Period, Record, Tokens, Usage,
};

struct Parsed {
    records: Vec<Record>,
    last: Option<i64>,
}

static FILES: LazyLock<FileCache<Parsed>> = LazyLock::new(FileCache::new);

/// The usage of an entry: a reply's (`message.usage`), or that of an entry
/// Pi records apart (`usage`, `compaction`, `branch_summary`).
fn record(o: &Value) -> Option<Record> {
    let kind = o.get("type").and_then(Value::as_str)?;
    let (u, from) = match kind {
        "message" => {
            let m = o.get("message")?;
            if m.get("role").and_then(Value::as_str) != Some("assistant") {
                return None;
            }
            (m.get("usage")?, m)
        }
        "usage" | "compaction" | "branch_summary" => (o.get("usage")?, o),
        _ => return None,
    };
    let n = |k: &str| u.get(k).and_then(Value::as_u64).unwrap_or(0);
    let model = from.get("model").and_then(Value::as_str).map(|m| {
        match from.get("provider").and_then(Value::as_str) {
            Some(p) => format!("{p}/{m}"),
            None => m.to_owned(),
        }
    });
    Some(Record {
        time: to_time(from.get("timestamp")).or_else(|| to_time(o.get("timestamp"))),
        model,
        tokens: Tokens::new(n("input") + n("cacheWrite"), n("cacheRead"), n("output")),
    })
}

fn parse(text: &str) -> Parsed {
    let mut out = Parsed {
        records: Vec::new(),
        last: None,
    };
    json_lines(text, |o, _| {
        if let Some(t) = to_time(o.get("timestamp")) {
            out.last = Some(out.last.map_or(t, |l| l.max(t)));
        }
        out.records.extend(record(o));
    });
    out
}

/// Usage of the sessions under `dir` (Pi's folder) in period `p`.
pub fn usage(dir: &Path, p: &Period) -> Usage {
    let files = recent_files(
        &dir.join("sessions"),
        &|n| n.ends_with(".jsonl"),
        p.since(),
        None,
    );
    FILES.keep_only(&files.iter().collect::<Vec<_>>());
    let mut records = Vec::new();
    let mut sessions = Vec::new();
    for f in &files {
        let parsed = FILES.get(f, parse);
        records.extend(parsed.records.iter().cloned());
        sessions.extend(parsed.last);
    }
    summarize(&records, &sessions, true, p)
}

#[cfg(test)]
#[path = "../../../../test/rust/agents/stats/pi.rs"]
mod tests;
