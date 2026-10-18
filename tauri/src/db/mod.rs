//! Databases: the kit's servers (`services`, D-73), their passwords and the
//! projects' databases (`auth`, D-74), the read-only content explorer
//! (`browse`, D-79) and phpMyAdmin (`phpmyadmin`, D-107); the engines
//! themselves in `engine` (D-110); MariaDB's data moved to MySQL in
//! `migrate_engine` (D-117).

pub mod auth;
pub mod browse;
pub mod engine;
pub mod migrate_engine;
pub mod phpmyadmin;
pub mod services;

pub use engine::DbEngine;
