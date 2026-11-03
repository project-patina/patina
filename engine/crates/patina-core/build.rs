//! `PATINA_CLIENT_VERSION`: the launcher's version, the root workspace's `[workspace.package]`
//! one. The client is released with the launcher, so that is its version too. This crate's own
//! version stands in when the engine is built outside the repository.
use std::{env, fs, path::Path};

fn main() {
    let dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    let manifest = Path::new(&dir).join("../../../Cargo.toml");
    println!("cargo:rerun-if-changed={}", manifest.display());
    let version = fs::read_to_string(&manifest)
        .ok()
        .and_then(|text| workspace_version(&text))
        .unwrap_or_else(|| env::var("CARGO_PKG_VERSION").unwrap_or_default());
    println!("cargo:rustc-env=PATINA_CLIENT_VERSION={version}");
}

/// The `version` line of `[workspace.package]`.
fn workspace_version(manifest: &str) -> Option<String> {
    let section = manifest.split("[workspace.package]").nth(1)?;
    let section = section.split("\n[").next()?;
    let line = section
        .lines()
        .find(|line| line.trim_start().starts_with("version"))?;
    Some(line.split('"').nth(1)?.to_owned())
}
