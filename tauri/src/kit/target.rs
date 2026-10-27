//! The systems and processors a kit is for: `Os`, `Arch` and `Target`,
//! with the names each source and tool gives them.

/// Operating system family of a kit target (`win32`, `darwin`, `linux`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Os {
    Win32,
    Darwin,
    Linux,
}

impl Os {
    /// Operating system of the running binary.
    pub const fn current() -> Os {
        crate::platform::OS
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Os::Win32 => "win32",
            Os::Darwin => "darwin",
            Os::Linux => "linux",
        }
    }

    pub const fn is_windows(self) -> bool {
        matches!(self, Os::Win32)
    }

    pub const fn is_macos(self) -> bool {
        matches!(self, Os::Darwin)
    }

    pub const fn is_linux(self) -> bool {
        matches!(self, Os::Linux)
    }

    pub const fn is_unix(self) -> bool {
        matches!(self, Os::Darwin | Os::Linux)
    }

    pub const fn exe_suffix(self) -> &'static str {
        match self {
            Os::Win32 => ".exe",
            Os::Darwin | Os::Linux => "",
        }
    }

    pub const fn path_sep(self) -> char {
        match self {
            Os::Win32 => ';',
            Os::Darwin | Os::Linux => ':',
        }
    }
}

/// Processor platform of the downloaded programs (`x64` or `arm64`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Arch {
    X64,
    Arm64,
}

impl Arch {
    /// The processor of this PC (`platform::host::arch`).
    pub fn detect() -> Arch {
        crate::platform::host::arch()
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Arch::X64 => "x64",
            Arch::Arm64 => "arm64",
        }
    }

    /// Folder name used under `data/apps` on the current OS (`win32-x64`, etc.).
    pub fn platform(self) -> String {
        Target::new(Os::current(), self).as_str().to_owned()
    }
}

/// Combined OS + CPU target. Atollo supports exactly the systems Docker
/// builds Docker Sandboxes for (D-176): `win32-x64`, `darwin-arm64`,
/// `linux-x64`, `linux-arm64` ([`Target::ALL`]). Any other pair is only a
/// detected host to refuse ([`unsupported_host`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Target {
    pub os: Os,
    pub arch: Arch,
}

impl Target {
    pub const WIN32_X64: Target = Target {
        os: Os::Win32,
        arch: Arch::X64,
    };
    pub const DARWIN_ARM64: Target = Target {
        os: Os::Darwin,
        arch: Arch::Arm64,
    };
    pub const LINUX_X64: Target = Target {
        os: Os::Linux,
        arch: Arch::X64,
    };
    pub const LINUX_ARM64: Target = Target {
        os: Os::Linux,
        arch: Arch::Arm64,
    };

    /// The supported targets, in the order of the grid of the Tools page.
    pub const ALL: [Target; 4] = [
        Self::WIN32_X64,
        Self::DARWIN_ARM64,
        Self::LINUX_X64,
        Self::LINUX_ARM64,
    ];

    /// One of [`Target::ALL`]: a system Docker Sandboxes runs on (D-176).
    pub const fn is_supported(self) -> bool {
        matches!(
            (self.os, self.arch),
            (Os::Win32, Arch::X64) | (Os::Darwin, Arch::Arm64) | (Os::Linux, _)
        )
    }

    pub const fn new(os: Os, arch: Arch) -> Target {
        Target { os, arch }
    }

    pub fn detect() -> Target {
        Target::new(Os::current(), Arch::detect())
    }

    /// Canonical folder and matrix identifier (`win32-x64`, `darwin-arm64`, ...);
    /// `unsupported` for a pair outside [`Target::ALL`], which
    /// [`Target::parse`] never gives.
    pub const fn as_str(self) -> &'static str {
        match (self.os, self.arch) {
            (Os::Win32, Arch::X64) => "win32-x64",
            (Os::Darwin, Arch::Arm64) => "darwin-arm64",
            (Os::Linux, Arch::X64) => "linux-x64",
            (Os::Linux, Arch::Arm64) => "linux-arm64",
            _ => "unsupported",
        }
    }

    /// A supported target by its name; `None` for anything else (an older
    /// kit's setting or folder: the caller falls back to the host).
    pub fn parse(s: &str) -> Option<Target> {
        let s = s.trim();
        Self::ALL.into_iter().find(|t| t.as_str() == s)
    }

    pub const fn is_windows(self) -> bool {
        self.os.is_windows()
    }

    pub const fn is_macos(self) -> bool {
        self.os.is_macos()
    }

    pub const fn is_linux(self) -> bool {
        self.os.is_linux()
    }

    pub const fn is_unix(self) -> bool {
        self.os.is_unix()
    }

    pub const fn exe_suffix(self) -> &'static str {
        self.os.exe_suffix()
    }

    /// Rust / LLVM target triple (`x86_64-pc-windows-msvc`, `aarch64-apple-darwin`, ...).
    /// Windows is x64 only, macOS ARM64 only.
    pub const fn rust_triple(self) -> &'static str {
        match (self.os, self.arch) {
            (Os::Win32, _) => "x86_64-pc-windows-msvc",
            (Os::Darwin, _) => "aarch64-apple-darwin",
            (Os::Linux, Arch::X64) => "x86_64-unknown-linux-gnu",
            (Os::Linux, Arch::Arm64) => "aarch64-unknown-linux-gnu",
        }
    }

    /// GNU / LLVM CPU name (`x86_64` or `aarch64`).
    pub const fn gnu_arch(self) -> &'static str {
        match self.arch {
            Arch::X64 => "x86_64",
            Arch::Arm64 => "aarch64",
        }
    }

    /// Go / GitHub CLI / Docker OS name (`windows`, `darwin`, `linux`).
    pub const fn go_os(self) -> &'static str {
        match self.os {
            Os::Win32 => "windows",
            Os::Darwin => "darwin",
            Os::Linux => "linux",
        }
    }

    /// Go / GitHub CLI / Docker CPU name (`amd64` or `arm64`).
    pub const fn go_arch(self) -> &'static str {
        match self.arch {
            Arch::X64 => "amd64",
            Arch::Arm64 => "arm64",
        }
    }

    /// Node.js / VSCodium / Dart OS name (`win`, `darwin`, `linux`).
    pub const fn node_os(self) -> &'static str {
        match self.os {
            Os::Win32 => "win",
            Os::Darwin => "darwin",
            Os::Linux => "linux",
        }
    }

    /// Eclipse Adoptium OS name (`windows`, `mac`, `linux`).
    pub const fn adoptium_os(self) -> &'static str {
        match self.os {
            Os::Win32 => "windows",
            Os::Darwin => "mac",
            Os::Linux => "linux",
        }
    }

    /// Eclipse Adoptium CPU name (`x64` or `aarch64`).
    pub const fn adoptium_arch(self) -> &'static str {
        match self.arch {
            Arch::X64 => "x64",
            Arch::Arm64 => "aarch64",
        }
    }

    /// Conda-forge subdirectory (`win-64`, `osx-arm64`, `linux-64`, ...).
    pub const fn conda_subdir(self) -> &'static str {
        match (self.os, self.arch) {
            (Os::Win32, _) => "win-64",
            (Os::Darwin, _) => "osx-arm64",
            (Os::Linux, Arch::X64) => "linux-64",
            (Os::Linux, Arch::Arm64) => "linux-aarch64",
        }
    }

    /// Micromamba standalone binary asset name on GitHub releases (native
    /// on every target).
    pub const fn micromamba_asset(self) -> &'static str {
        match (self.os, self.arch) {
            (Os::Win32, _) => "micromamba-win-64",
            (Os::Darwin, _) => "micromamba-osx-arm64",
            (Os::Linux, Arch::X64) => "micromamba-linux-64",
            (Os::Linux, Arch::Arm64) => "micromamba-linux-aarch64",
        }
    }

    /// TinyTeX / TeX Live binary subdirectory under `tinytex/bin/`.
    pub const fn texlive_bin(self) -> &'static str {
        match (self.os, self.arch) {
            (Os::Win32, _) => "windows",
            (Os::Darwin, _) => "universal-darwin",
            (Os::Linux, Arch::X64) => "x86_64-linux",
            (Os::Linux, Arch::Arm64) => "aarch64-linux",
        }
    }
}

/// Why Atollo stops at start on `host` (D-176): its core is Docker
/// Sandboxes, which Docker builds only for the four [`Target::ALL`].
/// `None` on those. `platform::host::arch` reads the PC's processor, also
/// when this exe runs translated.
pub fn unsupported_host(host: Target, lang: &str) -> Option<String> {
    (!host.is_supported()).then(|| {
        crate::common::i18n::t(
            lang,
            "Atollo funziona solo sui sistemi dove gira Docker Sandboxes: Windows x64, macOS con Apple silicon, Linux x64 e Linux ARM64. Questo sistema non è tra questi.",
            &[],
        )
    })
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
