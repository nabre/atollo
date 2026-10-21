//! PHP (with Composer and the Visual C++ runtime) and Apache (port of
//! nabre/atollo `_prepare-web.ps1`).
//!
//! The Visual C++ runtime is not installed: its DLLs are extracted from
//! Microsoft's `vc_redist.x64.exe` (signature required: Microsoft publishes
//! no checksum) and copied next to `php.exe` and `httpd.exe`, the app-local
//! deployment Microsoft allows. PHP and Apache exist only for x64: on ARM64
//! they run emulated with the x64 DLLs.
//!
//! On macOS and Linux PHP is static-php-cli's ([`php_unix::STATIC_PHP`],
//! D-112): the CLI and FPM archives of the lines with a SHA-256 fixed in the
//! catalog.
//!
//! Files: `mod` (this description), `vcruntime` (the Visual C++ runtime),
//! `php` (windows.php.net, the installer, Composer), `php_unix` (the kit's
//! builds and static-php-cli's for macOS and Linux), `php_ini` (one
//! `php.ini` per line), `apache` (Apache Lounge, the kit's httpd, its site).

mod apache;
mod php;
mod php_ini;
mod php_unix;
mod vcruntime;

pub use apache::Apache;
pub use php::{adopt_composer, Php};
pub use php_ini::{migrate_php_ini, php_ini_dir};
pub(super) use vcruntime::copy_vc_runtime;
