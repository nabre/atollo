//! pnpm and yarn: installed globally with the kit's npm and corepack in
//! `data/npm-global`.

use std::fs;

use anyhow::{anyhow, bail, Result};
use serde_json::Value;

use super::run;
use crate::common::fsx::join_rel;
use crate::install::{Ctx, Installer, Progress, Release};
use crate::kit::Target;
use crate::setup::Component;

/// pnpm and yarn, installed globally with the kit's npm in `data/npm-global`
/// (their install scripts only print messages: not run).
pub struct Pnpm;

impl Installer for Pnpm {
    fn latest(&self, ctx: &Ctx, _component: &Component) -> Result<Release> {
        let version = |p: &str| -> Result<String> {
            let meta: Value = ctx
                .net
                .json(&format!("https://registry.npmjs.org/{p}/latest"))?;
            meta.get("version")
                .and_then(Value::as_str)
                .map(str::to_owned)
                .ok_or_else(|| anyhow!("{p}: no version"))
        };
        Ok(Release {
            version: format!(
                "pnpm {}, corepack {}",
                version("pnpm")?,
                version("corepack")?
            ),
            parts: Vec::new(),
        })
    }

    fn custom(&self, _target: Target) -> bool {
        true
    }

    fn install_custom(
        &self,
        ctx: &Ctx,
        _component: &Component,
        _release: &Release,
        progress: &mut dyn Progress,
    ) -> Result<()> {
        // npm-cli.js with the kit's node, no cmd.exe: its quoting breaks on
        // kit paths with `&` or parentheses (found in a test kit).
        let node = crate::setup::kit_path(ctx.kit, "node/node.exe");
        let npm_cli = if ctx.kit.os().is_windows() {
            crate::setup::kit_path(ctx.kit, "node/node_modules/npm/bin/npm-cli.js")
        } else {
            crate::setup::kit_path(ctx.kit, "node/lib/node_modules/npm/bin/npm-cli.js")
        };
        if !node.is_file() || !npm_cli.is_file() {
            bail!("the kit's Node.js is needed for pnpm and yarn");
        }
        // Given, not only in the kit's environment: the tools sandbox
        // (D-180) gets none of it.
        let prefix = ctx.kit.data().join("npm-global");
        let envs = [("NPM_CONFIG_PREFIX", prefix.as_path())];
        let npm = |args: &[&str], progress: &mut dyn Progress| -> Result<()> {
            let cli = npm_cli.display().to_string();
            let mut all = vec![cli.as_str(), "--global", "--no-fund", "--no-audit"];
            all.extend_from_slice(args);
            run(ctx, &node, &all, &envs, progress)
        };
        let _ = npm(&["uninstall", "yarn", "corepack"], progress);
        npm(
            &["install", "--ignore-scripts", "corepack@latest"],
            progress,
        )?;
        let corepack = if ctx.kit.os().is_windows() {
            join_rel(&prefix, "node_modules/corepack/dist/corepack.js")
        } else {
            let lib = join_rel(&prefix, "lib/node_modules/corepack/dist/corepack.js");
            if lib.is_file() {
                lib
            } else {
                join_rel(&prefix, "node_modules/corepack/dist/corepack.js")
            }
        };
        let bin_dir = if ctx.kit.os().is_windows() {
            prefix.clone()
        } else {
            prefix.join("bin")
        };
        fs::create_dir_all(&bin_dir)?;
        run(
            ctx,
            &node,
            &[
                &corepack.display().to_string(),
                "enable",
                "yarn",
                "--install-directory",
                &bin_dir.display().to_string(),
            ],
            &[],
            progress,
        )?;
        npm(
            &["install", "--force", "--allow-scripts=pnpm", "pnpm@latest"],
            progress,
        )
    }
}
