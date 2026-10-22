//! The languages' variables of the kit environment, with the folders the
//! PATH needs after them.

use std::path::{Path, PathBuf};

use crate::fsx::join_rel;

use super::{node_options, Cx, KitEnv};

/// The folders of the languages that the PATH adds ([`super::path`]).
pub(super) struct LangDirs {
    pub(super) pyuser: PathBuf,
    pub(super) npm_prefix: PathBuf,
    pub(super) uv_bin: PathBuf,
    pub(super) composer: PathBuf,
    pub(super) rustup: PathBuf,
    pub(super) cargo: PathBuf,
    pub(super) gobin: PathBuf,
    pub(super) dotnet_home: PathBuf,
    pub(super) pub_cache: PathBuf,
    pub(super) bun: PathBuf,
    pub(super) deno_root: PathBuf,
    pub(super) pnpm: PathBuf,
}

/// The languages' variables (Python, Node, PHP, Rust, Go, Java and Gradle,
/// .NET, Dart, Zig, Julia, Bun, Deno, Typst, TeX, pnpm, R): the folders the
/// PATH adds after.
pub(super) fn language_vars(env: &mut KitEnv, cx: &Cx) -> LangDirs {
    let Cx { kit, .. } = *cx;
    let data = cx.data.clone();
    let home = cx.home.clone();
    let platform = cx.platform.clone();
    let fwd = |p: &Path| p.to_string_lossy().replace('\\', "/");
    let app = |rel: &str| cx.app(rel);
    let has = |rel: &str| cx.has(rel);
    // Python, pip, Node, npm, uv, gh, PHP, Composer (_env.cmd:101-139)
    let pyuser = data.join("python-user");
    env.set_path("PYTHONUSERBASE", &pyuser);
    env.set_path("PIP_CACHE_DIR", join_rel(&data, "cache/pip"));
    env.set("PIP_DISABLE_PIP_VERSION_CHECK", "1");
    env.set("PYTHONIOENCODING", "utf-8");
    let npm_prefix = data.join("npm-global");
    env.set_path("NPM_CONFIG_PREFIX", &npm_prefix);
    env.set_path("NPM_CONFIG_CACHE", join_rel(&data, "cache/npm"));
    env.set_path("NPM_CONFIG_USERCONFIG", home.join(".npmrc"));
    // yarn through corepack (the pnpm component): its downloads in the
    // kit, no question before one, never a change to package.json.
    env.set_path("COREPACK_HOME", join_rel(&data, "cache/corepack"));
    env.set("COREPACK_ENABLE_DOWNLOAD_PROMPT", "0");
    env.set("COREPACK_ENABLE_AUTO_PIN", "0");
    env.set("NODE_OPTIONS", node_options(kit));
    env.set_path("UV_CACHE_DIR", join_rel(&data, "cache/uv"));
    env.set_path("UV_PYTHON_INSTALL_DIR", join_rel(&data, "uv/python"));
    // uv's own Pythons only (downloaded into the kit): never one of the
    // PC's, found through the registry, and none written into it.
    env.set("UV_PYTHON_PREFERENCE", "only-managed");
    env.set("UV_PYTHON_INSTALL_REGISTRY", "0");
    env.set_path("UV_TOOL_DIR", join_rel(&data, "uv/tools"));
    let uv_bin = join_rel(&data, "uv/bin");
    env.set_path("UV_TOOL_BIN_DIR", &uv_bin);
    env.set_path("GH_CONFIG_DIR", data.join("gh"));
    // The PHP in use: php.ini finds its extensions through it, so a
    // saved version of PHP loads its own (versions/mod.rs); each line
    // has its php.ini (PHPRC). Apache's, with `/`, follow them when a
    // version is chosen for the kit or a project (versions::apply).
    env.set_php(&data, &app("php"));
    env.set_path("CP_APACHE", app("apache"));
    env.set("CP_APACHE_FWD", fwd(&app("apache")));
    // Caddy (D-111): its folder, as Apache's.
    env.set_path("CP_CADDY", app("caddy"));
    env.set("CP_CADDY_FWD", fwd(&app("caddy")));
    let composer = data.join("composer");
    env.set_path("COMPOSER_HOME", &composer);
    env.set_path("COMPOSER_CACHE_DIR", join_rel(&data, "cache/composer"));

    // Rust and the other languages (_env.cmd:153-184)
    let rustup = app("rust/rustup");
    let cargo = app("rust/cargo");
    env.set_path("RUSTUP_HOME", &rustup);
    env.set_path("CARGO_HOME", &cargo);
    let gobin = join_rel(&data, "go/bin").join(&platform);
    env.set_path("GOPATH", data.join("go"));
    env.set_path("GOBIN", &gobin);
    env.set_path("GOCACHE", join_rel(&data, "cache/go-build"));
    env.set_path("GOMODCACHE", join_rel(&data, "go/pkg/mod"));
    env.set_path("GOENV", join_rel(&data, "go/env"));
    env.set("GOTELEMETRY", "off");
    if has(r"java\bin\java.exe") {
        env.set_path("JAVA_HOME", app("java"));
    }
    env.set_path("GRADLE_USER_HOME", data.join("gradle"));
    // Gradle 9 runs on Java 17 to 27: the kit's gradle (data\kit\bin)
    // starts it on the newest JDK, whatever Java a project uses (its
    // toolchain compiles). With `/`: the versions' rewrites leave it.
    if let Some(java) = crate::setup::Catalog::get().find("java") {
        let newest = crate::versions::newest_line(kit, java);
        let fits = newest
            .as_deref()
            .and_then(|l| l.split('.').next())
            .and_then(|m| m.parse::<u32>().ok())
            .is_some_and(|m| m >= 17);
        if let (true, Some(dir)) = (fits, crate::versions::dir_for(kit, java, newest.as_deref())) {
            env.set("ATOLLO_GRADLE_JAVA", fwd(&dir));
        }
    }
    let dotnet_home = data.join("dotnet").join(&platform);
    env.set_path("DOTNET_ROOT", app("dotnet"));
    env.set_path("DOTNET_CLI_HOME", &dotnet_home);
    env.set_path("NUGET_PACKAGES", join_rel(&data, "cache/nuget"));
    env.set("DOTNET_CLI_TELEMETRY_OPTOUT", "1");
    env.set("DOTNET_NOLOGO", "1");
    env.set("DOTNET_SKIP_FIRST_TIME_EXPERIENCE", "1");
    env.set("DOTNET_MULTILEVEL_LOOKUP", "0");
    let pub_cache = join_rel(&data, "cache/pub");
    env.set_path("PUB_CACHE", &pub_cache);
    env.set_path("ZIG_GLOBAL_CACHE_DIR", join_rel(&data, "cache/zig"));
    env.set_path("JULIA_DEPOT_PATH", data.join("julia").join(&platform));
    let bun = data.join("bun").join(&platform);
    env.set_path("BUN_INSTALL", &bun);
    env.set_path("DENO_DIR", join_rel(&data, "cache/deno"));
    let deno_root = data.join("deno").join(&platform);
    env.set_path("DENO_INSTALL_ROOT", &deno_root);
    env.set_path("TYPST_PACKAGE_PATH", join_rel(&data, "typst/packages"));
    env.set_path("TYPST_PACKAGE_CACHE_PATH", join_rel(&data, "cache/typst"));
    env.set_path("TEXMFHOME", data.join("texmf"));
    let texlive = data.join("texlive").join(&platform);
    env.set_path("TEXMFVAR", texlive.join("texmf-var"));
    env.set_path("TEXMFCONFIG", texlive.join("texmf-config"));
    let pnpm = data.join("pnpm").join(&platform);
    env.set_path("PNPM_HOME", &pnpm);
    env.set_path("pnpm_config_store_dir", join_rel(&data, "cache/pnpm-store"));
    env.set_path("YARN_CACHE_FOLDER", join_rel(&data, "cache/yarn"));
    // One library per minor version of R (`%v` = 4.6): packages
    // compiled for one do not load in another (D-81).
    env.set(
        "R_LIBS_USER",
        format!("{}", data.join("r").join(&platform).join("%v").display()),
    );
    LangDirs {
        pyuser,
        npm_prefix,
        uv_bin,
        composer,
        rustup,
        cargo,
        gobin,
        dotnet_home,
        pub_cache,
        bun,
        deno_root,
        pnpm,
    }
}

#[cfg(test)]
#[path = "../../../test/rust/env/langs.rs"]
mod tests;
