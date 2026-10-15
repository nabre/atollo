//! What is inside the kit's database servers (DECISIONS.md D-79), read only:
//! the databases with their tables or collections, then one page of rows.
//!
//! - through the kit's own clients, logged in as the administrator as the
//!   rest of `db/services.rs`: `mariadb` with the kit's `[client]` login and the
//!   SQL on stdin, mongosh with the password and the names in its
//!   environment (`DEVKIT_DB_*`); never a password or a name on the command
//!   line;
//! - MariaDB answers one JSON value per line (`JSON_OBJECT`, `JSON_ARRAY`),
//!   so no escaping of tabs or `NULL` to undo; binary columns come as
//!   `{"$bytes": n}` and long texts are cut;
//! - MongoDB answers Extended JSON (relaxed): `{"$oid": …}`, `{"$date": …}`;
//!   the columns are the top-level fields of the page, in order of appearance.

use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Result};
use serde::Serialize;
use serde_json::Value;

use super::auth;
use crate::db::services::{bin, Services};
use crate::jobs;

/// Rows of one page.
pub const PAGE: u64 = 100;
/// Characters of a text value shown at most.
const TEXT_MAX: u32 = 2000;
const WITHIN: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Table {
    pub name: String,
    /// Estimated (InnoDB, MongoDB); None for views.
    pub rows: Option<u64>,
    pub view: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Database {
    pub name: String,
    /// The server's own: `mysql`, `information_schema`, `admin`, `local`…
    pub system: bool,
    pub tables: Vec<Table>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Column {
    pub name: String,
    /// MariaDB's column type (`int(11)`, `varchar(255)`); empty for MongoDB.
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Rows {
    pub columns: Vec<Column>,
    pub rows: Vec<Vec<Value>>,
    pub total: u64,
    pub offset: u64,
}

fn is_system(engine: &str, name: &str) -> bool {
    match engine {
        "mongodb" => matches!(name, "admin" | "config" | "local"),
        _ => matches!(
            name.to_ascii_lowercase().as_str(),
            "information_schema" | "mysql" | "performance_schema" | "sys"
        ),
    }
}

/// Runs a client with `input` on stdin and returns its output; stdout and
/// stderr are read while it runs, so a big answer cannot block it.
fn output(mut cmd: Command, input: &str, limit: Duration) -> Result<String> {
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = jobs::spawn(&mut cmd)?;
    let read = |pipe: Option<Box<dyn Read + Send>>| {
        std::thread::spawn(move || {
            let mut buf = Vec::new();
            if let Some(mut p) = pipe {
                let _ = p.read_to_end(&mut buf);
            }
            buf
        })
    };
    let out = read(
        child
            .stdout
            .take()
            .map(|p| Box::new(p) as Box<dyn Read + Send>),
    );
    let err = read(
        child
            .stderr
            .take()
            .map(|p| Box::new(p) as Box<dyn Read + Send>),
    );
    if let Some(mut stdin) = child.stdin.take() {
        // A client that already failed closes it: its error output says why.
        let _ = stdin.write_all(input.as_bytes());
    }
    let start = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if start.elapsed() > limit {
            let _ = child.kill();
            bail!("{:?}: no answer", cmd.get_program());
        }
        std::thread::sleep(Duration::from_millis(50));
    };
    let out = out.join().map_err(|_| anyhow!("reading the answer"))?;
    let err = err.join().map_err(|_| anyhow!("reading the errors"))?;
    if !status.success() {
        bail!("{}", String::from_utf8_lossy(&err).trim());
    }
    Ok(String::from_utf8_lossy(&out).into_owned())
}

/// `name` as a MariaDB identifier.
fn ident(name: &str) -> String {
    format!("`{}`", name.replace('`', "``"))
}

/// `text` as a MariaDB string literal.
fn literal(text: &str) -> String {
    format!("'{}'", text.replace('\\', "\\\\").replace('\'', "\\'"))
}

/// The JSON values of a client's answer, one per line.
fn json_lines(text: &str) -> Result<Vec<Value>> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(|l| serde_json::from_str(l).map_err(|e| anyhow!("unexpected answer: {e}")))
        .collect()
}

fn mariadb(services: &Services, sql: &str) -> Result<String> {
    let (_, env) = services.running_server("mariadb")?;
    let mut cmd = services.mariadb_tool("mariadb.exe", &env);
    cmd.args([
        "--batch",
        "--raw",
        "--skip-column-names",
        "--default-character-set=utf8mb4",
    ]);
    output(cmd, sql, WITHIN)
}

fn mongosh(services: &Services, script: &str, vars: &[(&str, &str)]) -> Result<Value> {
    let (port, env) = services.running_server("mongodb")?;
    let admin = auth::load(services.kit())
        .mongodb_admin
        .ok_or_else(|| anyhow!("MongoDB has no administrator yet"))?;
    let mut cmd = jobs::kit_command(bin(services.kit(), "mongosh", "mongosh.exe"), &env);
    cmd.args(["--quiet", "--host", "127.0.0.1", "--port"])
        .arg(port.to_string())
        .args(["--eval", script])
        .env("DEVKIT_DB_ADMIN", admin);
    for (k, v) in vars {
        cmd.env(k, v);
    }
    let text = output(cmd, "", WITHIN)?;
    // The answer is the last line: mongosh may warn before it.
    let last = text
        .lines()
        .rev()
        .find(|l| !l.trim().is_empty())
        .ok_or_else(|| anyhow!("mongosh: no answer"))?;
    serde_json::from_str(last).map_err(|e| anyhow!("mongosh: unexpected answer: {e}"))
}

const MARIADB_TREE: &str = "SELECT JSON_OBJECT('db', s.SCHEMA_NAME, 't', t.TABLE_NAME, \
    'n', t.TABLE_ROWS, 'view', t.TABLE_TYPE = 'VIEW') \
    FROM information_schema.SCHEMATA s \
    LEFT JOIN information_schema.TABLES t ON t.TABLE_SCHEMA = s.SCHEMA_NAME \
    ORDER BY s.SCHEMA_NAME, t.TABLE_NAME;\n";

// Plain loops: mongosh makes shell calls async and a callback of `map`
// would not wait for them. Counting can be refused even to root
// (`config.system.sessions`): null.
const MONGO_TREE_JS: &str = "const a = db.getSiblingDB('admin');     a.auth('admin', process.env.DEVKIT_DB_ADMIN);     const out = [];     for (const d of a.adminCommand({ listDatabases: 1, nameOnly: true }).databases) {       const s = db.getSiblingDB(d.name);       let infos = [];       try { infos = s.getCollectionInfos(); } catch (e) {}       const tables = [];       for (const c of infos) {         let rows = null;         if (c.type !== 'view') { try { rows = s.getCollection(c.name).estimatedDocumentCount(); } catch (e) {} }         tables.push({ name: c.name, view: c.type === 'view', rows: rows });       }       out.push({ name: d.name, tables: tables });     }     print(JSON.stringify(out));";

const MONGO_ROWS_JS: &str = "const a = db.getSiblingDB('admin'); \
    a.auth('admin', process.env.DEVKIT_DB_ADMIN); \
    const c = db.getSiblingDB(process.env.DEVKIT_DB_NAME).getCollection(process.env.DEVKIT_DB_TABLE); \
    const docs = c.find().skip(Number(process.env.DEVKIT_DB_OFFSET)).limit(Number(process.env.DEVKIT_DB_LIMIT)).toArray(); \
    print(EJSON.stringify({ total: c.countDocuments(), docs }, undefined, undefined, { relaxed: true }));";

/// The databases of a running server, with their tables or collections,
/// sorted by name.
pub fn tree(services: &Services, engine: &str) -> Result<Vec<Database>> {
    let mut out: Vec<Database> = Vec::new();
    match engine {
        "mariadb" => {
            for row in json_lines(&mariadb(services, MARIADB_TREE)?)? {
                let db = row["db"].as_str().unwrap_or_default().to_owned();
                if out.last().is_none_or(|d| d.name != db) {
                    out.push(Database {
                        system: is_system(engine, &db),
                        name: db,
                        tables: Vec::new(),
                    });
                }
                if let (Some(d), Some(t)) = (out.last_mut(), row["t"].as_str()) {
                    let view =
                        row["view"].as_i64() == Some(1) || row["view"].as_bool() == Some(true);
                    d.tables.push(Table {
                        name: t.to_owned(),
                        rows: if view { None } else { row["n"].as_u64() },
                        view,
                    });
                }
            }
        }
        "mongodb" => {
            let answer = mongosh(services, MONGO_TREE_JS, &[])?;
            for d in answer.as_array().into_iter().flatten() {
                let name = d["name"].as_str().unwrap_or_default().to_owned();
                let mut tables: Vec<Table> = d["tables"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(|t| Table {
                        name: t["name"].as_str().unwrap_or_default().to_owned(),
                        rows: t["rows"].as_u64(),
                        view: t["view"].as_bool() == Some(true),
                    })
                    .collect();
                tables.sort_by(|a, b| a.name.cmp(&b.name));
                out.push(Database {
                    system: is_system(engine, &name),
                    name,
                    tables,
                });
            }
            out.sort_by(|a, b| a.name.cmp(&b.name));
        }
        _ => bail!("unknown server {engine}"),
    }
    Ok(out)
}

/// A MariaDB column as an expression of `JSON_ARRAY`: binary data as its
/// size, long texts cut.
fn mariadb_value(name: &str, data_type: &str) -> String {
    let c = ident(name);
    match data_type.to_ascii_lowercase().as_str() {
        "binary" | "varbinary" | "tinyblob" | "blob" | "mediumblob" | "longblob" | "bit"
        | "geometry" | "point" | "linestring" | "polygon" | "multipoint" | "multilinestring"
        | "multipolygon" | "geometrycollection" => {
            format!("IF({c} IS NULL, NULL, JSON_OBJECT('$bytes', OCTET_LENGTH({c})))")
        }
        "char" | "varchar" | "tinytext" | "text" | "mediumtext" | "longtext" | "json" => {
            format!("LEFT({c}, {TEXT_MAX})")
        }
        _ => c,
    }
}

/// One page of a table or collection, from `offset`.
pub fn rows(
    services: &Services,
    engine: &str,
    database: &str,
    table: &str,
    offset: u64,
) -> Result<Rows> {
    match engine {
        "mariadb" => {
            let sql = format!(
                "SELECT JSON_OBJECT('c', COLUMN_NAME, 'k', DATA_TYPE, 'ct', COLUMN_TYPE) \
                 FROM information_schema.COLUMNS WHERE TABLE_SCHEMA = {} AND TABLE_NAME = {} \
                 ORDER BY ORDINAL_POSITION;\n",
                literal(database),
                literal(table)
            );
            let cols = json_lines(&mariadb(services, &sql)?)?;
            if cols.is_empty() {
                bail!("table not found: {database}.{table}");
            }
            let columns: Vec<Column> = cols
                .iter()
                .map(|c| Column {
                    name: c["c"].as_str().unwrap_or_default().to_owned(),
                    kind: c["ct"].as_str().unwrap_or_default().to_owned(),
                })
                .collect();
            let values: Vec<String> = cols
                .iter()
                .map(|c| {
                    mariadb_value(
                        c["c"].as_str().unwrap_or_default(),
                        c["k"].as_str().unwrap_or_default(),
                    )
                })
                .collect();
            let from = format!("{}.{}", ident(database), ident(table));
            let sql = format!(
                "SELECT COUNT(*) FROM {from};\nSELECT JSON_ARRAY({}) FROM {from} LIMIT {PAGE} OFFSET {offset};\n",
                values.join(", ")
            );
            let mut lines = json_lines(&mariadb(services, &sql)?)?.into_iter();
            let total = lines.next().and_then(|v| v.as_u64()).unwrap_or(0);
            let rows = lines
                .map(|v| match v {
                    Value::Array(a) => a,
                    other => vec![other],
                })
                .collect();
            Ok(Rows {
                columns,
                rows,
                total,
                offset,
            })
        }
        "mongodb" => {
            let (offset_s, limit_s) = (offset.to_string(), PAGE.to_string());
            let answer = mongosh(
                services,
                MONGO_ROWS_JS,
                &[
                    ("DEVKIT_DB_NAME", database),
                    ("DEVKIT_DB_TABLE", table),
                    ("DEVKIT_DB_OFFSET", &offset_s),
                    ("DEVKIT_DB_LIMIT", &limit_s),
                ],
            )?;
            Ok(documents(&answer, offset))
        }
        _ => bail!("unknown server {engine}"),
    }
}

/// MongoDB's documents as a table: the top-level fields in order of
/// appearance, a missing field as null.
fn documents(answer: &Value, offset: u64) -> Rows {
    let docs: Vec<&serde_json::Map<String, Value>> = answer["docs"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_object)
        .collect();
    let mut names: Vec<String> = Vec::new();
    for d in &docs {
        for k in d.keys() {
            if !names.contains(k) {
                names.push(k.clone());
            }
        }
    }
    let rows = docs
        .iter()
        .map(|d| {
            names
                .iter()
                .map(|n| d.get(n).cloned().unwrap_or(Value::Null))
                .collect()
        })
        .collect();
    Rows {
        columns: names
            .into_iter()
            .map(|name| Column {
                name,
                kind: String::new(),
            })
            .collect(),
        rows,
        total: answer["total"].as_u64().unwrap_or(0),
        offset,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_quoted() {
        assert_eq!(ident("a`b"), "`a``b`");
        assert_eq!(literal(r"it's\x"), r"'it\'s\\x'");
        assert!(is_system("mariadb", "MySQL"));
        assert!(is_system("mongodb", "local"));
        assert!(!is_system("mongodb", "shop"));
    }

    #[test]
    fn mariadb_values_hide_binary_and_cut_text() {
        assert_eq!(mariadb_value("id", "int"), "`id`");
        assert_eq!(mariadb_value("t", "longtext"), "LEFT(`t`, 2000)");
        assert!(mariadb_value("b", "BLOB").contains("'$bytes', OCTET_LENGTH(`b`)"));
    }

    #[test]
    fn documents_become_columns_in_order() {
        let answer: Value = serde_json::from_str(
            r#"{"total": 7, "docs": [{"_id": {"$oid": "1"}, "a": 1}, {"_id": {"$oid": "2"}, "b": [2]}]}"#,
        )
        .unwrap();
        let r = documents(&answer, 0);
        let names: Vec<&str> = r.columns.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, ["_id", "a", "b"]);
        assert_eq!(r.rows[1][1], Value::Null);
        assert_eq!(r.rows[1][2], serde_json::json!([2]));
        assert_eq!(r.total, 7);
    }

    #[test]
    fn json_lines_skip_blanks_and_fail_on_garbage() {
        assert_eq!(json_lines("1\n\n[2]\n").unwrap().len(), 2);
        assert!(json_lines("ERROR 1146").is_err());
    }
}
