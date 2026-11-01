//! Install + launch pipeline as seen by the UI: progress, logs and state events.

use std::sync::Arc;

use amethyst_core::accounts;
use amethyst_core::instance::Instance;
use amethyst_core::launch::{self, Player};
use amethyst_core::loaders::Loader;
use amethyst_core::progress::Reporter;
use amethyst_core::{bail, engine, install, modrinth, Error, Result};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio_util::sync::CancellationToken;

use crate::{AppState, LogLine, Running};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameState {
    pub id: String,
    /// `preparing`, `running`, `exited`, `crashed`, `error`, `cancelled`.
    pub state: &'static str,
    pub message: Option<String>,
    pub code: Option<i32>,
}

pub fn reporter(app: &AppHandle, id: &str) -> Reporter {
    let app = app.clone();
    Reporter::new(id, Arc::new(move |p| {
        let _ = app.emit("progress", p);
    }))
}

fn emit_state(app: &AppHandle, id: &str, state: &'static str, message: Option<String>, code: Option<i32>) {
    let _ = app.emit("game-state", GameState { id: id.into(), state, message, code });
}

fn log(app: &AppHandle, id: &str, line: impl Into<String>, error: bool) {
    let l = LogLine { id: id.into(), line: line.into(), error };
    app.state::<AppState>().push_log(l.clone());
    let _ = app.emit("game-log", l);
}

fn main_window(app: &AppHandle) -> Option<tauri::WebviewWindow> {
    app.get_webview_window("main")
}

#[tauri::command]
pub fn launch(app: AppHandle, state: State<'_, AppState>, id: String) -> Result<()> {
    let cancel = CancellationToken::new();
    {
        // Keep one lock order everywhere: installs, then running. Reserving the
        // instance must be atomic with respect to edits, deletion and launch.
        let mut installs = state.installs.lock().unwrap();
        let running = state.running.lock().unwrap();
        if running.contains_key(&id) {
            bail!("Ta instancja już działa");
        }
        if installs.contains_key(&id) {
            bail!("Ta instancja jest właśnie przygotowywana");
        }
        Instance::load(&state.ctx, &id)?;
        installs.insert(id.clone(), cancel.clone());
    }
    state.logs.lock().unwrap().remove(&id);
    {
        let mut s = state.settings.write().unwrap();
        if s.last_instance.as_deref() != Some(&id) {
            s.last_instance = Some(id.clone());
            let _ = s.save(&state.ctx);
        }
    }
    emit_state(&app, &id, "preparing", None, None);
    tauri::async_runtime::spawn(async move {
        let res = run_game(&app, &id, cancel).await;
        let st = app.state::<AppState>();
        st.installs.lock().unwrap().remove(&id);
        st.running.lock().unwrap().remove(&id);
        match res {
            Ok(Some(exit)) => {
                emit_state(&app, &id, if exit.crashed { "crashed" } else { "exited" }, None, exit.code);
                if let Err(e) = launch::run_hook(&exit.post_exit_command, &exit.game_dir).await {
                    log(&app, &id, format!("[Patina] {e}"), true);
                }
            }
            Ok(None) => {}
            Err(Error::Cancelled) => emit_state(&app, &id, "cancelled", None, None),
            Err(e) => {
                log::error!("launch {id} failed: {e}");
                log(&app, &id, format!("[Patina] Błąd: {e}"), true);
                emit_state(&app, &id, "error", Some(e.to_string()), None);
            }
        }
    });
    Ok(())
}

struct GameExit {
    crashed: bool,
    code: Option<i32>,
    post_exit_command: String,
    game_dir: std::path::PathBuf,
}

async fn run_game(app: &AppHandle, id: &str, cancel: CancellationToken) -> Result<Option<GameExit>> {
    let st = app.state::<AppState>();
    let ctx = st.ctx.clone();
    let settings = st.settings();
    let mut inst = Instance::load(&ctx, id)?;

    // Account first: no point downloading 500 MB to then fail on login.
    let account = {
        let mut accs = st.accounts.lock().await;
        let Some(acc) = accs.active().cloned() else { bail!("Zaloguj się kontem Microsoft, aby grać") };
        match accounts::ensure_fresh(&ctx, &acc).await? {
            Some(fresh) => {
                accs.upsert(fresh.clone());
                accs.save(&ctx)?;
                fresh
            }
            None => acc,
        }
    };

    let rust = inst.loader == Loader::Rust;
    if rust {
        engine::ensure_owner(&ctx, &account.mc_access_token).await?;
    }

    let rep = reporter(app, id);
    let prep = install::prepare(&ctx, &mut inst, &settings, &rep, &cancel).await?;
    if rust {
        engine::prepare(&ctx, &prep, &settings, &rep, &cancel).await?;
    }
    if !inst.installed {
        if settings.install_performance_mods && !inst.loader.is_plain() {
            match modrinth::install_performance_mods(&ctx, &inst, &rep, &cancel).await {
                Ok(names) if !names.is_empty() => log(app, id, format!("[Patina] Zainstalowano mody: {}", names.join(", ")), false),
                Ok(_) => {}
                Err(Error::Cancelled) => return Err(Error::Cancelled),
                Err(e) => log(app, id, format!("[Patina] Nie udało się pobrać modów optymalizacyjnych: {e}"), true),
            }
        }
        // Reload: the player may have edited the instance while we downloaded.
        let mut fresh = Instance::load(&ctx, id)?;
        if fresh.mc_version == inst.mc_version && fresh.loader == inst.loader && fresh.loader_version == inst.loader_version {
            fresh.installed = true;
            fresh.save(&ctx)?;
        }
    }
    if cancel.is_cancelled() {
        return Err(Error::Cancelled);
    }

    rep.report("launch", 1, 1, "Uruchamianie gry…");
    let game_dir = ctx.paths.game_dir(id);
    tokio::select! {
        biased;
        _ = cancel.cancelled() => return Err(Error::Cancelled),
        result = launch::run_hook(&settings.pre_launch_command, &game_dir) => result?,
    }
    let player = Player {
        name: account.name.clone(),
        uuid: account.uuid.clone(),
        access_token: account.mc_access_token.clone(),
        xuid: account.xuid.clone().unwrap_or_else(|| "0".into()),
        user_type: "msa".into(),
    };
    let spec = if rust { engine::build(&ctx, &inst, &player, &settings)? } else { launch::build(&ctx, &inst, &prep, &player, &settings) };
    if !rust {
        log(app, id, format!("[Patina] Java: {}", prep.java.display()), false);
    }
    log(app, id, format!("[Patina] $ {}", spec.display()), false);

    let closing = settings.on_launch == "close";
    if cancel.is_cancelled() { return Err(Error::Cancelled); }
    let mut child = launch::spawn(&spec, !closing)?;
    if let Err(error) = launch::feed_stdin(&mut child, &spec).await {
        let _ = child.kill().await;
        return Err(error);
    }
    let kill = CancellationToken::new();
    let started = chrono::Utc::now().timestamp();
    {
        let mut installs = st.installs.lock().unwrap();
        let mut running = st.running.lock().unwrap();
        running.insert(id.to_string(), Running { kill: kill.clone(), started });
        installs.remove(id);
    }
    emit_state(app, id, "running", None, None);
    if let Ok(mut fresh) = Instance::load(&ctx, id) {
        fresh.last_played = Some(started);
        let _ = fresh.save(&ctx);
    }

    if closing {
        app.exit(0);
        return Ok(None);
    }
    if let Some(w) = main_window(app) {
        match settings.on_launch.as_str() {
            "hide" => {
                let _ = w.hide();
            }
            "minimize" => {
                let _ = w.minimize();
            }
            _ => {}
        }
    }

    let (a, gid) = (app.clone(), id.to_string());
    // The engine logs everything to stderr; only its crash report line is an error.
    launch::pipe_output(&mut child, spec.secrets.clone(), Arc::new(move |line, level| {
        let error = if rust { line.contains("crash report saved") } else { level == "ERROR" || level == "FATAL" };
        log(&a, &gid, line, error)
    }));

    let mut killed = false;
    let status = tokio::select! {
        s = child.wait() => s?,
        _ = kill.cancelled() => {
            killed = true;
            let _ = child.kill().await;
            child.wait().await?
        }
    };

    // Bookkeeping survives even if the instance was edited meanwhile.
    if let Ok(mut fresh) = Instance::load(&ctx, id) {
        fresh.play_time_secs += (chrono::Utc::now().timestamp() - started).max(0) as u64;
        fresh.last_played = Some(started);
        let _ = fresh.save(&ctx);
    }
    if settings.reopen_on_exit || settings.on_launch == "hide" {
        if let Some(w) = main_window(app) {
            let _ = w.show();
            let _ = w.unminimize();
            let _ = w.set_focus();
        }
    }
    let code = status.code();
    log(app, id, format!("[Patina] Gra zakończona (kod {})", code.map(|c| c.to_string()).unwrap_or_else(|| "—".into())), false);
    let crashed = !killed && !status.success();
    // Publish the terminal event only after the caller releases the reservation.
    Ok(Some(GameExit { crashed, code, post_exit_command: settings.post_exit_command, game_dir }))
}

#[tauri::command]
pub fn stop(state: State<'_, AppState>, id: String) {
    if let Some(r) = state.running.lock().unwrap().get(&id) {
        r.kill.cancel();
    }
}

#[tauri::command]
pub fn cancel_install(state: State<'_, AppState>, id: String) {
    if let Some(c) = state.installs.lock().unwrap().get(&id) {
        c.cancel();
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunningInfo {
    id: String,
    /// `preparing` or `running`.
    state: &'static str,
    started: Option<i64>,
}

#[tauri::command]
pub fn running(state: State<'_, AppState>) -> Vec<RunningInfo> {
    let installs = state.installs.lock().unwrap();
    let running = state.running.lock().unwrap();
    let mut out: Vec<RunningInfo> = installs.keys().map(|id| RunningInfo { id: id.clone(), state: "preparing", started: None }).collect();
    out.extend(running.iter().map(|(id, r)| RunningInfo { id: id.clone(), state: "running", started: Some(r.started) }));
    out
}

#[tauri::command]
pub fn game_log(state: State<'_, AppState>, id: String) -> Vec<LogLine> {
    state.logs.lock().unwrap().get(&id).map(|q| q.iter().cloned().collect()).unwrap_or_default()
}
