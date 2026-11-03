//! The control socket: open when the game was started for an agent (`--control`,
//! `--control-socket`: patina-ctl launch, the launcher's MCP `play`), never under No AI
//! (`patina_core::no_ai`).
use super::Game;
use patina_client::{Result, control};

impl Game<'_> {
    /// At startup: open the socket if the game was started for an agent.
    pub(super) fn start_agent_control(&mut self) -> Result<()> {
        let Some(path) = &self.options.control else {
            return Ok(());
        };
        if patina_core::no_ai::is_on(&patina_client::logging::game_home()) {
            log::warn!(
                "Patina: control socket not opened: {}",
                patina_core::no_ai::REFUSAL
            );
            return Ok(());
        }
        // Asked for on the command line: failing to listen is an error.
        let server = control::Server::start(path)
            .map_err(|e| format!("control socket {}: {e}", path.display()))?;
        log::info!("Patina: control socket {}", server.path().display());
        self.control = Some(server);
        Ok(())
    }
}
