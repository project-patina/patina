package dev.patina.client;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.google.gson.JsonParseException;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.util.List;
import net.fabricmc.loader.api.FabricLoader;

/** The client settings, kept in {@code <game dir>/patina/client.json}. */
public final class PatinaConfig {
	/** Where a mod keeps to: a corner, the middle of an edge, or the middle of the screen. */
	public static final List<String> POSITIONS = List.of(
		"top_left", "top_center", "top_right",
		"center_left", "center", "center_right",
		"bottom_left", "bottom_center", "bottom_right"
	);
	public static final float MIN_SCALE = 0.5f;
	public static final float MAX_SCALE = 2.5f;

	private static final Gson GSON = new GsonBuilder().setPrettyPrinting().create();
	/** What corners stacked by before mods had a place of their own. */
	private static final int OLD_STACK_GAP = 3;
	private static PatinaConfig current;

	public Mods mods = new Mods();

	public static final class Mods {
		public Fps fps = new Fps();
		public Placed keystrokes = new Placed(false, "bottom_right");
		public Placed coordinates = new Placed(false, "top_right");

		public List<Placed> all() {
			return List.of(this.fps, this.keystrokes, this.coordinates);
		}
	}

	/**
	 * A mod's place: what it keeps to (a corner, an edge's middle or the screen's) and its
	 * distance from there in GUI pixels, so it stays put whatever the window size. Its size is
	 * a scale of its own.
	 */
	public static class Placed {
		public boolean enabled;
		public String position;
		public Float x;
		public Float y;
		public Float scale;

		Placed(boolean enabled, String position) {
			this.enabled = enabled;
			this.position = position;
			this.x = 0f;
			this.y = 0f;
			this.scale = 1f;
		}

		/** -1 keeps to the left edge, 1 to the right one, 0 to the middle. */
		public int horizontal() {
			return this.position.endsWith("left") ? -1 : this.position.endsWith("right") ? 1 : 0;
		}

		/** -1 keeps to the top edge, 1 to the bottom one, 0 to the middle. */
		public int vertical() {
			return this.position.startsWith("top") ? -1 : this.position.startsWith("bottom") ? 1 : 0;
		}

		public void reset(Placed defaults) {
			this.position = defaults.position;
			this.x = 0f;
			this.y = 0f;
			this.scale = 1f;
		}
	}

	public static final class Fps extends Placed {
		public String accuracy = "whole";
		public String style = "text";

		Fps() {
			super(true, "top_left");
		}
	}

	public static Path folder() {
		return FabricLoader.getInstance().getGameDir().resolve("patina");
	}

	public static PatinaConfig get() {
		if (current == null) {
			current = load();
		}
		return current;
	}

	private static PatinaConfig load() {
		Path file = folder().resolve("client.json");
		PatinaConfig config = null;
		if (Files.isRegularFile(file)) {
			try {
				config = GSON.fromJson(Files.readString(file), PatinaConfig.class);
			} catch (IOException | JsonParseException e) {
				PatinaClient.LOGGER.warn("Could not read {}, using the defaults", file, e);
			}
		}
		return config == null ? new PatinaConfig() : config.valid();
	}

	/** A hand-edited or older file loses the values it broke, never the rest. */
	private PatinaConfig valid() {
		Mods defaults = new Mods();
		if (this.mods == null) {
			this.mods = defaults;
		}
		if (this.mods.fps == null) {
			this.mods.fps = defaults.fps;
		}
		if (this.mods.keystrokes == null) {
			this.mods.keystrokes = defaults.keystrokes;
		}
		if (this.mods.coordinates == null) {
			this.mods.coordinates = defaults.coordinates;
		}
		Fps fps = this.mods.fps;
		fps.accuracy = "precise".equals(fps.accuracy) ? fps.accuracy : "whole";
		fps.style = "panel".equals(fps.style) ? fps.style : "text";
		// A file from before mods could be dragged keeps the old corner stacks.
		int[] used = new int[POSITIONS.size()];
		int[] heights = {"panel".equals(fps.style) ? 14 : 8, 58, 18};
		List<Placed> mods = this.mods.all();
		List<Placed> fallbacks = defaults.all();
		for (int i = 0; i < mods.size(); i++) {
			Placed mod = mods.get(i);
			if (!POSITIONS.contains(mod.position)) {
				mod.position = fallbacks.get(i).position;
			}
			int corner = POSITIONS.indexOf(mod.position);
			if (mod.x == null || mod.y == null || !Float.isFinite(mod.x) || !Float.isFinite(mod.y)) {
				mod.x = 0f;
				mod.y = mod.enabled ? (float) used[corner] : 0f;
			}
			if (mod.enabled) {
				used[corner] += heights[i] + OLD_STACK_GAP;
			}
			if (mod.scale == null || !(mod.scale >= MIN_SCALE && mod.scale <= MAX_SCALE)) {
				mod.scale = 1f;
			}
		}
		return this;
	}

	public void save() {
		Path file = folder().resolve("client.json");
		try {
			Files.createDirectories(file.getParent());
			Path temporary = file.resolveSibling("client.json.tmp");
			Files.writeString(temporary, GSON.toJson(this));
			Files.move(temporary, file, StandardCopyOption.REPLACE_EXISTING, StandardCopyOption.ATOMIC_MOVE);
		} catch (IOException e) {
			PatinaClient.LOGGER.warn("Could not save {}", file, e);
		}
	}
}
