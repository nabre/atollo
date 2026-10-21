//! Database servers and tools of the kit (DECISIONS.md D-73): MySQL
//! (D-110), MongoDB with mongosh, SQLite. Sources and checks in
//! docs/app/CATALOG.md §2.
//!
//! Files: `mod` (what the archives of MySQL and MongoDB lose before the
//! swap), `mysql` (Oracle's builds pinned in the catalog), `mongodb` (the
//! server, mongosh, conda-forge on Linux), `mongodb_tools` (the Database
//! Tools of each Linux distribution), `sqlite` (sqlite.org, conda-forge on
//! Linux ARM64), `phpmyadmin`.

mod mongodb;
mod mongodb_tools;
mod mysql;
mod phpmyadmin;
mod sqlite;

use std::fs;
use std::path::Path;

use anyhow::Result;

pub use mongodb::MongoDb;
pub use mysql::{mysql_lines_for, MySql, MysqlPin};
pub use phpmyadmin::PhpMyAdmin;
pub use sqlite::Sqlite;

/// Debug symbols (`.pdb`) are most of the archives (2.2 GB of MongoDB's
/// 2.5): removed from the new tree before the swap.
fn remove_debug_symbols(tree: &Path) -> Result<()> {
    for entry in fs::read_dir(tree)?.flatten() {
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            remove_debug_symbols(&path)?;
        } else if path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("pdb"))
        {
            fs::remove_file(&path)?;
        }
    }
    Ok(())
}
