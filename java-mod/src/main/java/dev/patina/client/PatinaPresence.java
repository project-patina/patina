package dev.patina.client;

import net.minecraft.client.Minecraft;
import net.minecraft.client.multiplayer.ServerData;

/**
 * What the player is doing, for the launcher's Discord activity. One log line each time it
 * changes, the same lines the Rust client writes (game/presence.rs): {@code [patina-presence] menu},
 * {@code singleplayer} or {@code server <address>}.
 */
public final class PatinaPresence {
	private static final String PREFIX = "[patina-presence] ";
	private static String reported;

	private PatinaPresence() {
	}

	/** The line for where the game is now. */
	static String line(Minecraft minecraft) {
		if (minecraft.level == null) {
			return "menu";
		}
		ServerData server = minecraft.getCurrentServer();
		if (minecraft.hasSingleplayerServer() || server == null) {
			return "singleplayer";
		}
		return "server " + server.ip;
	}

	public static void tick(Minecraft minecraft) {
		String now = line(minecraft);
		if (!now.equals(reported)) {
			reported = now;
			PatinaClient.LOGGER.info("{}{}", PREFIX, now);
		}
	}
}
