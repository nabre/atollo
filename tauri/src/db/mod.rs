//! Databases: the kit's servers (`services`, D-73), their passwords and the
//! projects' databases (`auth`, D-74), phpMyAdmin (`phpmyadmin`, D-107) and
//! mongo-express (`mongo_express`, D-147), with what they share in
//! `web_admin`; the engines themselves in `engine` (D-110).

pub mod auth;
pub mod engine;
pub mod mongo_express;
pub mod phpmyadmin;
pub mod services;
pub mod web_admin;

pub use engine::DbEngine;
