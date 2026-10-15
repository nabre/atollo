//! Databases: the kit's servers (`services`, D-73), their passwords and the
//! projects' databases (`auth`, D-74), the read-only content explorer
//! (`browse`, D-79) and Adminer (`adminer`, D-85).

pub mod adminer;
pub mod auth;
pub mod browse;
pub mod services;
