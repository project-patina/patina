package dev.patina.client;

import net.fabricmc.loader.api.FabricLoader;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public final class PatinaClient {
	public static final String MOD_ID = "patina";
	public static final Logger LOGGER = LoggerFactory.getLogger("Patina");
	/** The brand teal of the launcher, shared with the Rust client. */
	public static final int ACCENT = 0x3FB8A0;

	private static final String VERSION = FabricLoader.getInstance().getModContainer(MOD_ID)
		.map(mod -> mod.getMetadata().getVersion().getFriendlyString())
		.orElse("");

	private PatinaClient() {
	}

	public static String signature() {
		return "Patina Client " + VERSION;
	}

	/** Replaces the alpha of an ARGB colour with its own alpha scaled by {@code alpha}. */
	public static int fade(int argb, float alpha) {
		int a = Math.round((argb >>> 24) * Math.clamp(alpha, 0f, 1f));
		return a << 24 | argb & 0xFFFFFF;
	}
}
