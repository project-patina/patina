//! Patina core: everything the launcher does that does not need a window.
//!
//! The Tauri app is a thin shell over this crate, so the whole install/launch
//! pipeline can be exercised headless with `cargo run -p amethyst-core --example cli`.

pub mod accounts;
pub mod auth;
pub mod content;
pub mod download;
pub mod engine;
pub mod error;
pub mod game_textures;
pub mod http;
pub mod install;
pub mod instance;
pub mod java;
pub mod launch;
pub mod loaders;
pub mod modrinth;
pub mod mojang;
pub mod news;
pub mod paths;
pub mod profile;
pub mod progress;
pub mod settings;
pub mod wallpapers;

pub use error::{Error, Result};

pub const LAUNCHER_NAME: &str = "Patina";
pub const LAUNCHER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Shared handles every operation needs.
#[derive(Clone)]
pub struct Ctx {
    pub paths: paths::Paths,
    pub http: reqwest::Client,
}

impl Ctx {
    pub fn new(paths: paths::Paths) -> Result<Self> {
        paths.ensure()?;
        Ok(Self { paths, http: http::client()? })
    }
}
