//! Launcher updates as seen by the UI.

use std::sync::atomic::{AtomicBool, Ordering};

use amethyst_core::update::{self, Source, Targets, UpdateInfo};
use amethyst_core::{bail, Result};
use tauri::{AppHandle, Emitter, State};

use crate::AppState;

#[tauri::command]
pub async fn update_check(state: State<'_, AppState>) -> Result<Option<UpdateInfo>> {
    if amethyst_core::DEV {
        return Ok(None);
    }
    let Some(release) = update::find(&Source::official(), update::current_version()).await? else { return Ok(None) };
    Ok(Some(release.info(update::installable(&Targets::current(&state.ctx)?))))
}

/// Holds `updating`, which also blocks launches, unless the install went through.
struct Reservation<'a>(&'a AtomicBool);

impl Drop for Reservation<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

fn reserve(state: &AppState) -> Result<Reservation<'_>> {
    // Same lock order as `launch`, so a game cannot start between the check and the flag.
    let installs = state.installs.lock().unwrap();
    let running = state.running.lock().unwrap();
    if !installs.is_empty() || !running.is_empty() {
        bail!("Najpierw zamknij grę: aktualizacja podmienia pliki silnika", "Close the game first: the update replaces engine files");
    }
    if state.updating.swap(true, Ordering::SeqCst) {
        bail!("Aktualizacja już trwa albo czeka na restart", "An update is already running or waiting for a restart");
    }
    Ok(Reservation(&state.updating))
}

#[tauri::command]
pub async fn update_install(app: AppHandle, state: State<'_, AppState>) -> Result<()> {
    if amethyst_core::DEV {
        bail!("Patina Dev nie aktualizuje się sama", "Patina Dev does not update itself");
    }
    let reservation = reserve(&state)?;
    let source = Source::official();
    let Some(release) = update::find(&source, update::current_version()).await? else {
        bail!("Nie ma nowszej wersji", "There is no newer version");
    };
    let targets = Targets::current(&state.ctx)?;
    if !update::installable(&targets) {
        bail!("Ta kopia launchera nie może się sama zaktualizować", "This copy of the launcher cannot update itself");
    }
    update::install(&state.ctx, &source, &release, targets, move |progress| {
        let _ = app.emit("update-progress", progress);
    })
    .await?;
    // The old launcher must not start the new engine: only a restart clears this.
    std::mem::forget(reservation);
    Ok(())
}

#[tauri::command]
pub fn update_restart(app: AppHandle) {
    app.restart();
}
