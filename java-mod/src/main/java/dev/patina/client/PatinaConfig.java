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
	public static final String HOTBAR_LEFT = "hotbar_left";

	private static final Gson GSON = new GsonBuilder().setPrettyPrinting().create();
	/** What corners stacked by before mods had a place of their own. */
	private static final int OLD_STACK_GAP = 3;
	private static PatinaConfig current;

	public Mods mods = new Mods();

	public static final class Mods {
		public Fps fps = new Fps();
		public Keystrokes keystrokes = new Keystrokes();
		public Coordinates coordinates = new Coordinates();
		public Armor armor = new Armor();
		public ToggleMod fullbright = new ToggleMod();
		public ToggleMod appleskin = new ToggleMod();
		public UtilityMod shulker_tooltip = new UtilityMod();
		public InfoMod speedometer = new InfoMod("bottom_left");
		public UtilityMod zoom = UtilityMod.zoom();
		public InfoMod ping = new InfoMod("top_left");
		public UtilityMod perspective = UtilityMod.perspective();
		public ToggleMod horse_stats = new ToggleMod();
		public EffectsMod status_effects = new EffectsMod();
		public UtilityMod weather = new UtilityMod();
		public ToggleMod death_info = new ToggleMod();
		public ToggleMod no_fog = new ToggleMod();

		public List<Placed> all() {
			return List.of(this.fps, this.keystrokes, this.coordinates, this.armor, this.speedometer, this.ping, this.status_effects);
		}

		public List<ToggleMod> catalogue() {
			return List.of(this.fps, this.keystrokes, this.coordinates, this.armor, this.fullbright,
				this.appleskin, this.shulker_tooltip, this.speedometer, this.zoom, this.ping, this.perspective,
				this.horse_stats, this.status_effects, this.weather, this.death_info, this.no_fog);
		}

		public Placed hud(int index) {
			ToggleMod entry = this.catalogue().get(index);
			return entry instanceof Placed placed ? placed : null;
		}

		public void resetUtility(int index) {
			boolean enabled = this.catalogue().get(index).enabled;
			Placed previous = this.hud(index);
			switch (index) {
				case 5 -> this.appleskin = new ToggleMod();
				case 6 -> this.shulker_tooltip = new UtilityMod();
				case 7 -> this.speedometer = new InfoMod("bottom_left");
				case 8 -> this.zoom = UtilityMod.zoom();
				case 9 -> this.ping = new InfoMod("top_left");
				case 10 -> this.perspective = UtilityMod.perspective();
				case 11 -> this.horse_stats = new ToggleMod();
				case 12 -> this.status_effects = new EffectsMod();
				case 13 -> this.weather = new UtilityMod();
				case 14 -> this.death_info = new ToggleMod();
				case 15 -> this.no_fog = new ToggleMod();
				default -> throw new IllegalArgumentException("Unknown utility module");
			}
			this.catalogue().get(index).enabled = enabled;
			Placed current = this.hud(index);
			if (previous != null && current != null) {
				current.position = previous.position; current.x = previous.x; current.y = previous.y; current.scale = previous.scale;
			}
		}

		public void resetSettings(int mod) {
			Placed previous = this.all().get(mod);
			Placed defaults = switch (mod) {
				case 0 -> this.fps = new Fps();
				case 1 -> this.keystrokes = new Keystrokes();
				case 2 -> this.coordinates = new Coordinates();
				case 3 -> this.armor = new Armor();
				default -> throw new IllegalArgumentException("Unknown HUD module: " + mod);
			};
			defaults.enabled = previous.enabled;
			defaults.position = previous.position;
			defaults.x = previous.x;
			defaults.y = previous.y;
			defaults.scale = previous.scale;
		}
	}

	/** A rendering module has no HUD position. */
	public static class ToggleMod {
		public boolean enabled = false;
	}

	public static final class UtilityMod extends ToggleMod {
		public int key = 67;
		public String activation = "hold";
		public float factor = 4f;
		public boolean smooth = true;
		public float sensitivity = 0.25f;
		public String weather = "clear";
		public float strength = 1f;
		public boolean require_shift = false;
		static UtilityMod zoom() { return new UtilityMod(); }
		static UtilityMod perspective() { UtilityMod mod = new UtilityMod(); mod.key = 86; return mod; }
	}

	public static final class InfoMod extends Placed {
		public String unit = "m/s";
		public boolean background = false;
		InfoMod() { super(false, null); }
		InfoMod(String position) { super(false, position); }
	}

	public static final class EffectsMod extends Placed {
		public int columns = 2;
		public boolean show_duration = true;
		public boolean show_amplifier = true;
		EffectsMod() { super(false, "center_right"); }
	}

	/**
	 * A mod's place: what it keeps to (a corner, an edge's middle or the screen's) and its
	 * distance from there in GUI pixels, so it stays put whatever the window size. Its size is
	 * a scale of its own.
	 */
	public static class Placed extends ToggleMod {
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

	public static final class Keystrokes extends Placed {
		public boolean show_mouse = true;
		public boolean show_space = true;
		public boolean show_shift = true;
		public boolean show_labels = true;

		Keystrokes() { super(false, "bottom_right"); }
	}

	public static final class Coordinates extends Placed {
		public boolean show_direction = true;
		public boolean show_angle = false;
		public boolean show_biome = false;
		public boolean background = false;

		Coordinates() { super(false, "top_right"); }
	}

	public static final class Armor extends Placed {
		public boolean vertical = false;
		public String theme = "minecraft";
		public boolean hide_empty = false;
		public String display = "bar";
		public boolean dynamic_color = true;
		public boolean warning = true;
		public int warning_threshold = 20;
		private boolean hotbar_anchor_migrated;

		Armor() { super(false, HOTBAR_LEFT); }
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
		if (this.mods.armor == null) this.mods.armor = defaults.armor;
		if (this.mods.fullbright == null) this.mods.fullbright = defaults.fullbright;
		if (this.mods.appleskin == null) this.mods.appleskin = defaults.appleskin;
		if (this.mods.shulker_tooltip == null) this.mods.shulker_tooltip = defaults.shulker_tooltip;
		if (this.mods.speedometer == null) this.mods.speedometer = defaults.speedometer;
		if (this.mods.zoom == null) this.mods.zoom = defaults.zoom;
		if (this.mods.ping == null) this.mods.ping = defaults.ping;
		if (this.mods.perspective == null) this.mods.perspective = defaults.perspective;
		if (this.mods.horse_stats == null) this.mods.horse_stats = defaults.horse_stats;
		if (this.mods.status_effects == null) this.mods.status_effects = defaults.status_effects;
		if (this.mods.weather == null) this.mods.weather = defaults.weather;
		if (this.mods.death_info == null) this.mods.death_info = defaults.death_info;
		if (this.mods.no_fog == null) this.mods.no_fog = defaults.no_fog;
		for (UtilityMod utility : List.of(this.mods.zoom, this.mods.perspective, this.mods.weather, this.mods.shulker_tooltip)) {
			utility.factor = Float.isFinite(utility.factor) ? Math.clamp(utility.factor, 1.5f, 20f) : 4f;
			utility.sensitivity = Float.isFinite(utility.sensitivity) ? Math.clamp(utility.sensitivity, 0.05f, 1f) : 0.25f;
			utility.strength = Float.isFinite(utility.strength) ? Math.clamp(utility.strength, 0f, 1f) : 1f;
			if (!List.of("clear", "rain", "thunder").contains(utility.weather == null ? "" : utility.weather)) utility.weather = "clear";
			if (!"toggle".equals(utility.activation)) utility.activation = "hold";
			if (utility.key < -1 || utility.key > 348) utility.key = 67;
		}
		this.mods.status_effects.columns = Math.clamp(this.mods.status_effects.columns, 1, 4);
		if (!"km/h".equals(this.mods.speedometer.unit)) this.mods.speedometer.unit = "m/s";
		Armor armor = this.mods.armor;
		armor.theme = "patina".equals(armor.theme) ? "patina" : "minecraft";
		if (armor.display == null || !List.of("bar", "remaining").contains(armor.display)) armor.display = "bar";
		armor.warning_threshold = Math.max(5, Math.min(50, armor.warning_threshold));
		Fps fps = this.mods.fps;
		fps.accuracy = "precise".equals(fps.accuracy) ? fps.accuracy : "whole";
		fps.style = "panel".equals(fps.style) ? fps.style : "text";
		// A file from before mods could be dragged keeps the old corner stacks.
		int[] used = new int[POSITIONS.size()];
		int[] heights = {"panel".equals(fps.style) ? 14 : 8, 98, 18, 30, 12, 12, 60};
		List<Placed> mods = this.mods.all();
		List<Placed> fallbacks = defaults.all();
		for (int i = 0; i < mods.size(); i++) {
			Placed mod = mods.get(i);
			if (mod == armor && !armor.hotbar_anchor_migrated && "bottom_left".equals(mod.position)
				&& Float.valueOf(0f).equals(mod.x) && Float.valueOf(0f).equals(mod.y)
				&& Float.valueOf(1f).equals(mod.scale)) {
				mod.position = HOTBAR_LEFT;
			}
			if (mod == armor) armor.hotbar_anchor_migrated = true;
			if (mod == armor && (mod.position == null || !POSITIONS.contains(mod.position) || HOTBAR_LEFT.equals(mod.position))) {
				mod.position = HOTBAR_LEFT;
				mod.x = 0f;
				mod.y = 0f;
				if (mod.scale == null || !(mod.scale >= MIN_SCALE && mod.scale <= MAX_SCALE)) mod.scale = 1f;
				continue;
			}
			if (mod.position == null || !POSITIONS.contains(mod.position)) {
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
		this.mods.armor.hotbar_anchor_migrated = true;
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
