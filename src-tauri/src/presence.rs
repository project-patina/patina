//! The Discord activity: the launcher's page while no game runs, else the game
//! and where the player is in it, as the Patina clients report it on their
//! output (`[patina-presence] menu`, `singleplayer` or `server <address>`).

use std::collections::{HashMap, HashSet};
use std::net::IpAddr;
use std::sync::Mutex;

use amethyst_core::discord::{self, Activity, Presence};
use amethyst_core::instance::Instance;
use amethyst_core::settings::Settings;
use amethyst_core::{lang::tr, tr};
use tauri::{AppHandle, Manager};

use crate::AppState;

const PREFIX: &str = "[patina-presence] ";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Place {
    Menu,
    Singleplayer,
    Server(String),
}

impl Place {
    /// The place a game's output line reports, if it is a presence line.
    pub fn parse(line: &str) -> Option<Self> {
        let rest = line[line.find(PREFIX)? + PREFIX.len()..].trim();
        match rest.split_once(' ').unwrap_or((rest, "")) {
            ("menu", _) => Some(Self::Menu),
            ("singleplayer", _) => Some(Self::Singleplayer),
            ("server", address) => Some(Self::Server(address.trim().to_owned())),
            _ => None,
        }
    }
}

/// The host of `address` worth showing: a name, never an IP address, which may
/// be someone's home.
fn public_host(address: &str) -> Option<String> {
    let host = match address.rsplit_once(':') {
        Some((host, port)) if !host.contains(':') && port.chars().all(|c| c.is_ascii_digit()) => host,
        _ => address,
    };
    let host = host.trim().trim_end_matches('.').to_lowercase();
    let ip = host.trim_matches(|c| c == '[' || c == ']').parse::<IpAddr>().is_ok();
    let name = host.contains('.') && host.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-');
    (!ip && name && host != "localhost").then_some(host)
}

struct Game {
    version: String,
    started: i64,
    place: Option<Place>,
}

struct Inner {
    page: String,
    idle: bool,
    opened: i64,
    preparing: HashSet<String>,
    games: HashMap<String, Game>,
}

pub struct PresenceState {
    discord: Presence,
    inner: Mutex<Inner>,
}

impl PresenceState {
    pub fn new() -> Self {
        let inner = Inner { page: "home".into(), idle: false, opened: chrono::Utc::now().timestamp(), preparing: HashSet::new(), games: HashMap::new() };
        Self { discord: Presence::start(discord::APPLICATION_ID), inner: Mutex::new(inner) }
    }
}

fn activity(settings: &Settings, inner: &Inner) -> Option<Activity> {
    if !settings.discord_presence {
        return None;
    }
    let button = tr("Pobierz Patinę", "Get Patina").to_owned();
    if let Some(game) = inner.games.values().max_by_key(|g| g.started) {
        let state = match &game.place {
            Some(Place::Menu) => tr("W menu głównym", "In the main menu").to_owned(),
            Some(Place::Singleplayer) => tr("Świat jednoosobowy", "Singleplayer").to_owned(),
            Some(Place::Server(address)) => match public_host(address).filter(|_| settings.discord_show_server) {
                Some(host) => tr!("Na serwerze {host}", "On {host}"),
                None => tr("Gra wieloosobowa", "Multiplayer").to_owned(),
            },
            None => tr("W grze", "In game").to_owned(),
        };
        return Some(Activity {
            details: tr!("Gra w Minecraft {}", "Playing Minecraft {}", game.version),
            state,
            image_text: "Patina Client".into(),
            started: Some(game.started),
            button,
        });
    }
    let state = if !inner.preparing.is_empty() {
        tr("Przygotowuje grę", "Getting a game ready")
    } else if inner.idle {
        tr("Bezczynny", "Idle")
    } else {
        match inner.page.as_str() {
            "instances" | "instance" => tr("Przegląda instancje", "Browsing instances"),
            "discover" => tr("Szuka modów", "Looking for mods"),
            "servers" => tr("Przegląda serwery", "Browsing servers"),
            "wardrobe" => tr("Wybiera skina", "Picking a skin"),
            "settings" => tr("W ustawieniach", "In the settings"),
            _ => tr("Na stronie głównej", "On the home page"),
        }
    };
    Some(Activity {
        details: tr("W launcherze", "In the launcher").to_owned(),
        state: state.to_owned(),
        image_text: "Patina Launcher".into(),
        started: Some(inner.opened),
        button,
    })
}

fn update(app: &AppHandle, change: impl FnOnce(&mut Inner)) {
    let presence = app.state::<PresenceState>();
    let settings = app.state::<AppState>().settings();
    let mut inner = presence.inner.lock().unwrap();
    change(&mut inner);
    presence.discord.set(activity(&settings, &inner));
}

/// After the settings change.
pub fn refresh(app: &AppHandle) {
    update(app, |_| {});
}

pub fn preparing(app: &AppHandle, id: &str) {
    update(app, |i| {
        i.preparing.insert(id.to_owned());
    });
}

pub fn started(app: &AppHandle, id: &str, instance: &Instance, started: i64) {
    update(app, |i| {
        i.preparing.remove(id);
        i.games.insert(id.to_owned(), Game { version: instance.mc_version.clone(), started, place: None });
    });
}

/// The game's output line `line`; true when it was a presence line.
pub fn reported(app: &AppHandle, id: &str, line: &str) -> bool {
    let Some(place) = Place::parse(line) else { return false };
    update(app, |i| {
        if let Some(game) = i.games.get_mut(id) {
            game.place = Some(place);
        }
    });
    true
}

pub fn ended(app: &AppHandle, id: &str) {
    update(app, |i| {
        i.preparing.remove(id);
        i.games.remove(id);
    });
}

#[tauri::command]
/// The page the launcher shows, and whether its player has been away for a while.
pub fn presence_page(app: AppHandle, page: String, idle: bool) {
    update(&app, |i| {
        i.page = page;
        i.idle = idle;
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presence_lines_from_both_clients_parse() {
        let fabric = "[18:02:11] [Render thread/INFO] (Patina) [patina-presence] server mc.example.net:25570";
        assert_eq!(Place::parse(fabric), Some(Place::Server("mc.example.net:25570".into())));
        assert_eq!(Place::parse("[patina-presence] menu"), Some(Place::Menu));
        assert_eq!(Place::parse("[patina-presence] singleplayer\r"), Some(Place::Singleplayer));
        assert_eq!(Place::parse("[patina-presence] dancing"), None);
        assert_eq!(Place::parse("[Render thread/INFO] Loading animation active"), None);
    }

    #[test]
    fn only_server_names_are_shown_never_addresses() {
        assert_eq!(public_host("Mc.Example.net:25565").as_deref(), Some("mc.example.net"));
        assert_eq!(public_host("play.example.org.").as_deref(), Some("play.example.org"));
        assert_eq!(public_host("192.168.1.20:25565"), None);
        assert_eq!(public_host("[2001:db8::1]:25565"), None);
        assert_eq!(public_host("2001:db8::1"), None);
        assert_eq!(public_host("localhost"), None);
        assert_eq!(public_host("my-pc"), None);
    }

    fn inner() -> Inner {
        Inner { page: "servers".into(), idle: false, opened: 10, preparing: HashSet::new(), games: HashMap::new() }
    }

    #[test]
    fn the_launcher_shows_its_page_until_a_game_runs() {
        amethyst_core::lang::set("en");
        let mut settings = Settings::default();
        let mut i = inner();
        let shown = activity(&settings, &i).unwrap();
        assert_eq!((shown.details.as_str(), shown.state.as_str(), shown.started), ("In the launcher", "Browsing servers", Some(10)));
        i.idle = true;
        assert_eq!(activity(&settings, &i).unwrap().state, "Idle");
        i.games.insert("a".into(), Game { version: "26.3".into(), started: 50, place: Some(Place::Server("mc.example.net".into())) });
        let shown = activity(&settings, &i).unwrap();
        assert_eq!((shown.details.as_str(), shown.state.as_str(), shown.started), ("Playing Minecraft 26.3", "On mc.example.net", Some(50)));
        settings.discord_show_server = false;
        assert_eq!(activity(&settings, &i).unwrap().state, "Multiplayer");
        settings.discord_presence = false;
        assert_eq!(activity(&settings, &i), None);
    }
}
