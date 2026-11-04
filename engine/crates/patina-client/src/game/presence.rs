//! What the player is doing, for the launcher's Discord activity. One line on
//! stderr each time it changes, the same lines the Fabric client writes
//! (`PatinaPresence.java`): `[patina-presence] menu`, `singleplayer` or
//! `server <address>`.

use super::Game;

pub(super) const PREFIX: &str = "[patina-presence] ";

/// The line for where the game is now.
pub(super) fn line(on_title: bool, address: Option<&str>) -> String {
    match address {
        _ if on_title => "menu".to_owned(),
        Some(address) if address != "local" => format!("server {address}"),
        _ => "singleplayer".to_owned(),
    }
}

impl Game<'_> {
    pub(super) fn report_presence(&mut self) {
        let now = line(
            self.title.is_some(),
            self.remote.as_ref().map(|r| r.session.address.as_str()),
        );
        if self.presence.as_deref() != Some(now.as_str()) {
            eprintln!("{PREFIX}{now}");
            self.presence = Some(now);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::line;

    #[test]
    fn the_title_screen_is_the_menu_even_while_connected() {
        assert_eq!(line(true, Some("mc.example.net")), "menu");
        assert_eq!(line(false, Some("mc.example.net:25570")), "server mc.example.net:25570");
        assert_eq!(line(false, Some("local")), "singleplayer");
        assert_eq!(line(false, None), "singleplayer");
    }
}
