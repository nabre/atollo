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

#[allow(dead_code)]
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

    pub const fn cmd_suffix(self) -> &'static str {
        match self {
            Os::Win32 => ".cmd",
            Os::Darwin | Os::Linux => "",
        }
    }

    pub const fn bat_suffix(self) -> &'static str {
        match self {
            Os::Win32 => ".bat",
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

/// Combined OS + CPU target (`win32-x64`, `win32-arm64`, `darwin-arm64`,
/// `darwin-x64`, `linux-x64`, `linux-arm64`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Target {
    pub os: Os,
    pub arch: Arch,
}

#[allow(dead_code)]
impl Target {
    pub const WIN32_X64: Target = Target {
        os: Os::Win32,
        arch: Arch::X64,
    };
    pub const WIN32_ARM64: Target = Target {
        os: Os::Win32,
        arch: Arch::Arm64,
    };
    pub const DARWIN_ARM64: Target = Target {
        os: Os::Darwin,
        arch: Arch::Arm64,
    };
    pub const DARWIN_X64: Target = Target {
        os: Os::Darwin,
        arch: Arch::X64,
    };
    pub const LINUX_X64: Target = Target {
        os: Os::Linux,
        arch: Arch::X64,
    };
    pub const LINUX_ARM64: Target = Target {
        os: Os::Linux,
        arch: Arch::Arm64,
    };

    pub const ALL: [Target; 6] = [
        Self::WIN32_X64,
        Self::WIN32_ARM64,
        Self::DARWIN_ARM64,
        Self::DARWIN_X64,
        Self::LINUX_X64,
        Self::LINUX_ARM64,
    ];

    pub const fn new(os: Os, arch: Arch) -> Target {
        Target { os, arch }
    }

    pub fn detect() -> Target {
        Target::new(Os::current(), Arch::detect())
    }

    pub const fn with_arch(self, arch: Arch) -> Target {
        Target { os: self.os, arch }
    }

    /// Canonical folder and matrix identifier (`win32-x64`, `darwin-arm64`, ...).
    pub const fn as_str(self) -> &'static str {
        match (self.os, self.arch) {
            (Os::Win32, Arch::X64) => "win32-x64",
            (Os::Win32, Arch::Arm64) => "win32-arm64",
            (Os::Darwin, Arch::Arm64) => "darwin-arm64",
            (Os::Darwin, Arch::X64) => "darwin-x64",
            (Os::Linux, Arch::X64) => "linux-x64",
            (Os::Linux, Arch::Arm64) => "linux-arm64",
        }
    }

    pub fn parse(s: &str) -> Option<Target> {
        match s.trim() {
            "win32-x64" => Some(Self::WIN32_X64),
            "win32-arm64" => Some(Self::WIN32_ARM64),
            "darwin-arm64" => Some(Self::DARWIN_ARM64),
            "darwin-x64" => Some(Self::DARWIN_X64),
            "linux-x64" => Some(Self::LINUX_X64),
            "linux-arm64" => Some(Self::LINUX_ARM64),
            _ => None,
        }
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
    pub const fn rust_triple(self) -> &'static str {
        match (self.os, self.arch) {
            (Os::Win32, Arch::X64) => "x86_64-pc-windows-msvc",
            (Os::Win32, Arch::Arm64) => "aarch64-pc-windows-msvc",
            (Os::Darwin, Arch::Arm64) => "aarch64-apple-darwin",
            (Os::Darwin, Arch::X64) => "x86_64-apple-darwin",
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
            (Os::Win32, Arch::X64) => "win-64",
            (Os::Win32, Arch::Arm64) => "win-arm64",
            (Os::Darwin, Arch::Arm64) => "osx-arm64",
            (Os::Darwin, Arch::X64) => "osx-64",
            (Os::Linux, Arch::X64) => "linux-64",
            (Os::Linux, Arch::Arm64) => "linux-aarch64",
        }
    }

    /// Micromamba standalone binary asset name on GitHub releases (native
    /// on every target, `win-arm64` too).
    pub const fn micromamba_asset(self) -> &'static str {
        match (self.os, self.arch) {
            (Os::Win32, Arch::X64) => "micromamba-win-64",
            (Os::Win32, Arch::Arm64) => "micromamba-win-arm64",
            (Os::Darwin, Arch::Arm64) => "micromamba-osx-arm64",
            (Os::Darwin, Arch::X64) => "micromamba-osx-64",
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

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
