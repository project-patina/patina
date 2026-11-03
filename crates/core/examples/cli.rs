//! Headless test driver for the install/launch pipeline.
//!
//!   AMETHYST_ROOT=/tmp/am cargo run -p amethyst-core --example cli -- launch 1.21.1 fabric [seconds]
//!   cargo run -p amethyst-core --example cli -- loaders fabric 1.21.1
//!
//! `launch` uses an offline test profile, waits `seconds` (default 60) for the
//! game to reach the main menu, then kills it.

use std::sync::Arc;

use amethyst_core::instance::{Instance, NewInstance};
use amethyst_core::loaders::Loader;
use amethyst_core::paths::Paths;
use amethyst_core::progress::Reporter;
use amethyst_core::settings::Settings;
use amethyst_core::{engine, install, launch, loaders, Ctx};
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> amethyst_core::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let root = std::env::var("AMETHYST_ROOT").map(Into::into).unwrap_or_else(|_| Paths::default_root());
    let ctx = Ctx::new(Paths::new(root))?;
    let args: Vec<String> = std::env::args().skip(1).collect();
    let loader = |s: &str| match s {
        "vanilla" => Loader::Vanilla,
        "quilt" => Loader::Quilt,
        "rust" => Loader::Rust,
        _ => Loader::Fabric,
    };
    match args.first().map(String::as_str) {
        Some("loaders") => {
            let v = loaders::versions(&ctx, loader(&args[1]), &args[2]).await?;
            println!("{} versions, first: {:?}", v.len(), v.first());
        }
        Some("launch") => {
            let mc = args.get(1).cloned().unwrap_or_else(|| "1.21.1".into());
            let ld = loader(args.get(2).map(String::as_str).unwrap_or("fabric"));
            let secs: u64 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(60);
            let mut inst = Instance::create(&ctx, NewInstance { name: format!("cli {mc} {}", ld.label()), mc_version: mc, loader: ld, loader_version: None, icon: None, banner: None })?;
            let settings = Settings::default();
            let rep = Reporter::new(&inst.id, Arc::new(|p| eprintln!("[{}] {} {}/{}", p.stage, p.message, p.done, p.total)));
            let cancel = CancellationToken::new();
            let t = std::time::Instant::now();
            let prep = install::prepare(&ctx, &mut inst, &settings, &rep, &cancel).await?;
            if ld == Loader::Rust {
                engine::prepare(&ctx, &prep, &settings, &rep, &cancel).await?;
            } else if ld != Loader::Vanilla {
                let mods = amethyst_core::modrinth::install_performance_mods(&ctx, &inst, &rep, &cancel).await?;
                println!("mods: {mods:?}");
            }
            amethyst_core::patina_mod::sync(&ctx, &inst)?;
            println!("prepared in {:?}, java {}", t.elapsed(), prep.java.display());
            let player = launch::Player::offline("AmethystTest");
            let spec = if ld == Loader::Rust { engine::build(&ctx, &inst, &player, &settings)? } else { launch::build(&ctx, &inst, &prep, &player, &settings) };
            println!("$ {}", spec.display());
            let mut child = launch::spawn(&spec, true)?;
            launch::feed_stdin(&mut child, &spec).await?;
            let seen_menu = Arc::new(std::sync::atomic::AtomicBool::new(false));
            let sm = seen_menu.clone();
            launch::pipe_output(&mut child, spec.secrets.clone(), Arc::new(move |l, level| {
                if l.contains("Sound engine started") || l.contains("Created:") || l.contains("rest-ready") {
                    sm.store(true, std::sync::atomic::Ordering::Relaxed);
                }
                println!("{level:>5}| {l}");
            }));
            tokio::select! {
                s = child.wait() => println!("game exited early: {s:?}"),
                _ = tokio::time::sleep(std::time::Duration::from_secs(secs)) => {
                    let _ = child.kill().await;
                    println!("killed after {secs}s");
                }
            }
            println!("RESULT reached_menu={}", seen_menu.load(std::sync::atomic::Ordering::Relaxed));
            Instance::delete(&ctx, &inst.id)?;
        }
        Some("modrinth") => {
            use amethyst_core::content::{self, ContentKind};
            use amethyst_core::modrinth;
            let q = modrinth::SearchQuery { query: "shader".into(), kind: ContentKind::Mod, game_version: Some("1.21.1".into()), loader: Some(Loader::Fabric), ..Default::default() };
            let r = modrinth::search(&ctx, &q).await?;
            println!("search: {} hits, first {:?}", r.total_hits, r.hits.first().map(|h| &h.title));
            let inst = Instance::create(&ctx, NewInstance { name: "cli modrinth".into(), mc_version: "1.21.1".into(), loader: Loader::Fabric, loader_version: None, icon: None, banner: None })?;
            let rep = Reporter::silent(&inst.id);
            let c = CancellationToken::new();
            // Iris pulls in Sodium as a required dependency.
            let done = modrinth::install(&ctx, &inst, ContentKind::Mod, "iris", None, &rep, &c).await?;
            println!("installed: {:?}", done.iter().map(|d| (&d.title, d.dependency)).collect::<Vec<_>>());
            // Downgrade Sodium to an older 1.21.1 build, then check updates.
            let sodium = modrinth::versions(&ctx, "sodium", &["fabric"], Some("1.21.1")).await?;
            let old = sodium.iter().next_back().unwrap();
            modrinth::install(&ctx, &inst, ContentKind::Mod, "sodium", Some(&old.id), &rep, &c).await?;
            let ups = modrinth::check_updates(&ctx, &inst, ContentKind::Mod).await?;
            println!("updates after downgrade: {:?}", ups.iter().map(|u| (&u.file, &u.version_number)).collect::<Vec<_>>());
            // Forget metadata and identify by hash.
            content::save_meta(&ctx, &inst.id, &Default::default())?;
            modrinth::identify(&ctx, &inst.id, ContentKind::Mod).await?;
            for i in content::list(&ctx, &inst.id, ContentKind::Mod) {
                println!("  {} -> {:?}", i.file, i.meta.map(|m| (m.title, m.version_number)));
            }
            let shaders = modrinth::install(&ctx, &inst, ContentKind::Shader, "complementary-reimagined", None, &rep, &c).await?;
            println!("shader: {:?}", shaders.iter().map(|d| &d.file).collect::<Vec<_>>());
            Instance::delete(&ctx, &inst.id)?;
        }
        _ => eprintln!("usage: cli launch <mc> <vanilla|fabric|quilt> [secs] | cli loaders <loader> <mc>"),
    }
    Ok(())
}
