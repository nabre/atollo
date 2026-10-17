//! Databases: the kit's servers (`services`, D-73), their passwords and the
//! projects' databases (`auth`, D-74), the read-only content explorer
//! (`browse`, D-79) and phpMyAdmin (`phpmyadmin`, D-107); the engines
//! themselves in `engine` (PLAN-PIATTAFORME.md §4.3).

pub mod auth;
pub mod browse;
pub mod engine;
pub mod phpmyadmin;
pub mod services;

pub use engine::DbEngine;
