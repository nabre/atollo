//! What every part of the engine uses and that belongs to none of them: the
//! kit's files on disk (`fsx`), the log, the clock, the texts in four
//! languages (`i18n`), the user's settings and who has each port.

pub mod fsx;
pub mod i18n;
pub mod log;
pub mod ports;
pub mod settings;
pub mod time;
