//! Microsoft sign-in. The primary flow opens Microsoft's own login page in a
//! separate private webview window and catches the redirect carrying the code;
//! the device-code flow is a fallback for when that window cannot be used.

use std::sync::{Arc, Mutex};

use amethyst_core::accounts::{self, AccountView};
use amethyst_core::auth::{self, DeviceCode, MsTokens};
use amethyst_core::{Error, Result};
use tauri::webview::PageLoadEvent;
use tauri::{AppHandle, Manager, State, WebviewUrl, WebviewWindowBuilder, WindowEvent};
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

use crate::AppState;

const LOGIN_LABEL: &str = "ms-login";

type CodeTx = Arc<Mutex<Option<oneshot::Sender<Result<String>>>>>;

fn send(tx: &CodeTx, r: Result<String>) {
    if let Some(tx) = tx.lock().unwrap().take() {
        let _ = tx.send(r);
    }
}

async fn finish(state: &AppState, tokens: MsTokens) -> Result<AccountView> {
    let acc = accounts::complete_login(&state.ctx, tokens).await?;
    let uuid = acc.uuid.clone();
    let mut accs = state.accounts.lock().await;
    accs.upsert(acc);
    accs.save(&state.ctx)?;
    log::info!("signed in as {}", accs.active().map(|a| a.name.as_str()).unwrap_or("?"));
    Ok(accs.views().into_iter().find(|a| a.uuid == uuid).expect("just inserted"))
}

#[tauri::command]
pub async fn login_microsoft(app: AppHandle, state: State<'_, AppState>) -> Result<AccountView> {
    if let Some(w) = app.get_webview_window(LOGIN_LABEL) {
        let _ = w.destroy();
    }
    let (tx, rx) = oneshot::channel();
    let tx: CodeTx = Arc::new(Mutex::new(Some(tx)));
    let url = auth::authorize_url().parse().map_err(|e| Error::msg(format!("{e}")))?;

    let nav_tx = tx.clone();
    let load_tx = tx.clone();
    let window = WebviewWindowBuilder::new(&app, LOGIN_LABEL, WebviewUrl::External(url))
        .title("Zaloguj się — Microsoft")
        .inner_size(500.0, 660.0)
        .resizable(true)
        .center()
        .incognito(true)
        .on_navigation(move |u| match auth::code_from_redirect(u.as_str()) {
            Some(r) => {
                send(&nav_tx, r);
                false
            }
            None => true,
        })
        // Some redirects skip the navigation hook; the page-load event still sees them.
        .on_page_load(move |_, p| {
            if p.event() == PageLoadEvent::Started {
                if let Some(r) = auth::code_from_redirect(p.url().as_str()) {
                    send(&load_tx, r);
                }
            }
        })
        .build()
        .map_err(|e| Error::msg(format!("Nie udało się otworzyć okna logowania: {e}")))?;

    let close_tx = tx.clone();
    window.on_window_event(move |e| {
        if matches!(e, WindowEvent::CloseRequested { .. } | WindowEvent::Destroyed) {
            send(&close_tx, Err(Error::Cancelled));
        }
    });

    let code = rx.await.map_err(|_| Error::Cancelled)?;
    if let Some(w) = app.get_webview_window(LOGIN_LABEL) {
        let _ = w.destroy();
    }
    if let Some(main) = app.get_webview_window("main") {
        let _ = main.set_focus();
    }
    let tokens = auth::exchange_code(&state.ctx.http, &code?).await?;
    finish(&state, tokens).await
}

#[tauri::command]
pub async fn login_device_start(state: State<'_, AppState>) -> Result<DeviceCode> {
    auth::device_code_start(&state.ctx.http).await
}

#[tauri::command]
pub async fn login_device_finish(state: State<'_, AppState>, code: DeviceCode) -> Result<AccountView> {
    let cancel = CancellationToken::new();
    *state.login_cancel.lock().unwrap() = Some(cancel.clone());
    let tokens = auth::device_code_wait(&state.ctx.http, &code, &cancel).await;
    *state.login_cancel.lock().unwrap() = None;
    finish(&state, tokens?).await
}

#[tauri::command]
pub fn cancel_login(app: AppHandle, state: State<'_, AppState>) {
    if let Some(c) = state.login_cancel.lock().unwrap().take() {
        c.cancel();
    }
    if let Some(w) = app.get_webview_window(LOGIN_LABEL) {
        let _ = w.close();
    }
}
