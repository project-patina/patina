package dev.patina.client;

import java.util.Arrays;
import java.util.List;
import java.util.Locale;
import net.minecraft.client.KeyMapping;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.player.LocalPlayer;
import net.minecraft.core.Direction;
import net.minecraft.network.chat.Component;
import net.minecraft.world.entity.EquipmentSlot;
import net.minecraft.world.entity.HumanoidArm;
import net.minecraft.world.item.ItemStack;
import net.minecraft.resources.Identifier;
import net.minecraft.client.renderer.RenderPipelines;
import net.minecraft.util.Mth;

/** The three built-in mods. The settings preview draws them through the same code as the game. */
public final class PatinaHud {
	public static final int MARGIN = 4;
	private static final int PANEL_PADDING = 3;
	private static final int PANEL = 0x70000000;
	private static final int TEXT = 0xFFE8F0EE;
	private static final int DIM_TEXT = 0xFF9FB0AC;
	private static final int KEY = 18;
	private static final int KEY_GAP = 2;
	private static final int KEY_IDLE = 0x70000000;
	private static final int KEY_DOWN = 0xC0000000 | PatinaClient.ACCENT;
	private static final int SAMPLES = 512;
	private static final long REFRESH_NANOS = 250_000_000L;

	/** What a frame shows: the game, or the made-up values of the preview. */
	public record View(String fps, String coordinates, String facing, boolean[] keysDown, String biome, float yaw, float pitch, boolean leftOffhand, ArmorPiece[] armor, String speed, String ping, List<EffectPiece> effects) {
	}
	public record EffectPiece(Identifier sprite, int amplifier, int ticks) { }

	/** Preview pieces do not construct ItemStacks before a world's registries are bound. */
	public record ArmorPiece(ItemStack stack, Identifier icon, int remaining, int maximum, boolean present) {
		static ArmorPiece live(ItemStack stack) {
			int maximum = !stack.isEmpty() && stack.isDamageableItem() ? stack.getMaxDamage() : 0;
			return new ArmorPiece(stack, null, Math.max(0, maximum - stack.getDamageValue()), maximum, !stack.isEmpty());
		}
		static ArmorPiece sample(String item, int remaining, int maximum) {
			return new ArmorPiece(null, item == null ? null : Identifier.withDefaultNamespace("textures/item/" + item + ".png"), remaining, maximum, item != null);
		}
	}

	/** A mod's rectangle on screen, its scale already applied. */
	public record Box(float x, float y, float width, float height) {
		public boolean contains(double px, double py) {
			return px >= this.x && px < this.x + this.width && py >= this.y && py < this.y + this.height;
		}
	}

	private static final long[] FRAME_NANOS = new long[SAMPLES];
	private static long frameCount;
	private static long lastFrame;
	private static long lastRefresh;
	private static String preciseText = "";

	private PatinaHud() {
	}

	public static void drawGame(GuiGraphicsExtractor graphics) {
		Minecraft minecraft = Minecraft.getInstance();
		LocalPlayer player = minecraft.player;
		long now = System.nanoTime();
		if (lastFrame != 0) {
			FRAME_NANOS[(int) (frameCount++ % SAMPLES)] = now - lastFrame;
		}
		lastFrame = now;
		if (player == null || minecraft.gui.hud.isHidden() || minecraft.gui.hud.getDebugOverlay().showDebugScreen()) {
			return;
		}
		// The editor draws them itself, where they are being dragged.
		if (minecraft.gui.screen() instanceof PatinaHudEditorScreen) {
			return;
		}
		PatinaConfig.Mods mods = PatinaConfig.get().mods;
		View view = liveView();
		boolean showPosition = !minecraft.showOnlyReducedInfo();
		for (PatinaConfig.Placed mod : mods.all()) {
			if (mod.enabled && (mod != mods.coordinates || showPosition)) {
				draw(graphics, minecraft.font, view, mods, mod, place(minecraft.font, graphics.guiWidth(), graphics.guiHeight(), view, mods, mod), 1f);
			}
		}
	}

	/** What the mods show now, or {@code null} without a player. */
	public static View liveView() {
		Minecraft minecraft = Minecraft.getInstance();
		LocalPlayer player = minecraft.player;
		if (player == null) {
			return null;
		}
		PatinaConfig.Mods mods = PatinaConfig.get().mods;
		String fps = mods.fps.accuracy.equals("precise") ? precise(System.nanoTime()) : minecraft.getFps() + " FPS";
		String biome = minecraft.level.getBiome(player.blockPosition()).unwrapKey().map(key -> {
			var id = key.identifier();
			return Component.translatable("biome." + id.getNamespace() + "." + id.getPath()).getString();
		}).orElse("?");
		boolean leftOffhand = player.getMainArm() == HumanoidArm.RIGHT && !player.getOffhandItem().isEmpty();
		return new View(fps, coordinates(player), facing(player.getDirection()), keysDown(minecraft), biome, player.getYRot(), player.getXRot(), leftOffhand, new ArmorPiece[] {
			ArmorPiece.live(player.getItemBySlot(EquipmentSlot.HEAD)), ArmorPiece.live(player.getItemBySlot(EquipmentSlot.CHEST)),
			ArmorPiece.live(player.getItemBySlot(EquipmentSlot.LEGS)), ArmorPiece.live(player.getItemBySlot(EquipmentSlot.FEET))
		}, String.format(Locale.ROOT, "%.1f %s", PatinaUtilities.speed() * ("km/h".equals(mods.speedometer.unit) ? 3.6 : 1), mods.speedometer.unit),
			(PatinaUtilities.ping() < 0 ? "—" : "" + PatinaUtilities.ping()) + " ms", player.getActiveEffects().stream().filter(net.minecraft.world.effect.MobEffectInstance::showIcon)
				.sorted(java.util.Comparator.comparing(e -> e.getEffect().unwrapKey().map(key -> key.identifier().toString()).orElse("")))
				.map(e -> new EffectPiece(net.minecraft.client.gui.Hud.getMobEffectSprite(e.getEffect()), e.getAmplifier(), e.isInfiniteDuration() ? -1 : e.getDuration())).toList());
	}

	/** The made-up values the editor shows without a world. */
	public static View sampleView() {
		String fps = PatinaConfig.get().mods.fps.accuracy.equals("precise") ? format(143.6, 91) : "144 FPS";
		return new View(fps, "128 64 -256", facing(Direction.NORTH), new boolean[] {true, false, false, false, true, false, true, false}, PatinaLang.get("patina.preview.biome"), 180f, 12f, false, new ArmorPiece[] {
			ArmorPiece.sample("turtle_helmet", 8, 275), ArmorPiece.sample(null, 0, 0),
			ArmorPiece.sample("iron_leggings", 225, 225), ArmorPiece.sample("golden_boots", 65, 91)
		}, String.format(Locale.ROOT, "%.1f %s", "km/h".equals(PatinaConfig.get().mods.speedometer.unit) ? 15.48 : 4.3, PatinaConfig.get().mods.speedometer.unit), "42 ms", List.of(
			new EffectPiece(Identifier.withDefaultNamespace("mob_effect/regeneration"), 2, 5400),
			new EffectPiece(Identifier.withDefaultNamespace("mob_effect/speed"), 1, 12000),
			new EffectPiece(Identifier.withDefaultNamespace("mob_effect/strength"), 2, 5700),
			new EffectPiece(Identifier.withDefaultNamespace("mob_effect/jump_boost"), 1, 12000),
			new EffectPiece(Identifier.withDefaultNamespace("mob_effect/fire_resistance"), 0, 2400),
			new EffectPiece(Identifier.withDefaultNamespace("mob_effect/night_vision"), 0, 1160)));
	}

	/** Refreshed a few times a second: a number that changes every frame cannot be read. */
	private static String precise(long now) {
		if (now - lastRefresh >= REFRESH_NANOS) {
			lastRefresh = now;
			int count = (int) Math.min(frameCount, SAMPLES);
			if (count > 0) {
				long[] sorted = Arrays.copyOf(FRAME_NANOS, count);
				Arrays.sort(sorted);
				long total = 0;
				for (long nanos : sorted) {
					total += nanos;
				}
				int worst = Math.max(1, count / 100);
				long worstTotal = 0;
				for (int i = count - worst; i < count; i++) {
					worstTotal += sorted[i];
				}
				preciseText = format(1e9 * count / total, 1e9 * worst / worstTotal);
			}
		}
		return preciseText;
	}

	public static String format(double fps, double low) {
		return String.format(Locale.ROOT, "%.1f FPS  1%% %.0f", fps, low);
	}

	private static String coordinates(LocalPlayer player) {
		return Mth.floor(player.getX()) + " " + Mth.floor(player.getY()) + " " + Mth.floor(player.getZ());
	}

	public static String facing(Direction direction) {
		return switch (direction) {
			case SOUTH -> PatinaLang.get("patina.facing.south") + " +Z";
			case WEST -> PatinaLang.get("patina.facing.west") + " -X";
			case EAST -> PatinaLang.get("patina.facing.east") + " +X";
			default -> PatinaLang.get("patina.facing.north") + " -Z";
		};
	}

	private static boolean[] keysDown(Minecraft minecraft) {
		var options = minecraft.options;
		return new boolean[] {
			options.keyUp.isDown(), options.keyLeft.isDown(), options.keyDown.isDown(), options.keyRight.isDown(),
			options.keyAttack.isDown(), options.keyUse.isDown(), options.keyJump.isDown(), options.keyShift.isDown()
		};
	}

	/** The mod's size at a scale of 1. */
	public static int[] size(Font font, View view, PatinaConfig.Mods mods, PatinaConfig.Placed mod) {
		if (mod == mods.speedometer || mod == mods.ping) {
			boolean background = mod == mods.speedometer ? mods.speedometer.background : mods.ping.background;
			return new int[] {font.width(mod == mods.speedometer ? view.speed : view.ping) + (background ? 6 : 0), 8 + (background ? 6 : 0)};
		}
		if (mod == mods.status_effects) {
			int count = Math.max(1, view.effects.size()), columns = Math.min(mods.status_effects.columns, count);
			return new int[] {columns * 30 - 2, Math.ceilDiv(count, columns) * 30 - 2};
		}
		if (mod == mods.fps) {
			int pad = mods.fps.style.equals("panel") ? PANEL_PADDING : 0;
			return new int[] {font.width(view.fps) + pad * 2, font.lineHeight - 1 + pad * 2};
		}
		if (mod == mods.keystrokes) {
			PatinaConfig.Keystrokes keys = mods.keystrokes;
			return new int[] {KEY * 3 + KEY_GAP * 2, KEY * 2 + KEY_GAP + (KEY + KEY_GAP) * ((keys.show_mouse ? 1 : 0) + (keys.show_space || keys.show_shift ? 1 : 0))};
		}
		if (mod == mods.armor) {
			int count = 0;
			for (ArmorPiece stack : view.armor) if (!mods.armor.hide_empty || stack.present) count++;
			return armorSize(mods.armor, count);
		}
		List<String> lines = coordinateLines(view, mods.coordinates);
		int pad = mods.coordinates.background ? PANEL_PADDING : 0;
		return new int[] {lines.stream().mapToInt(font::width).max().orElse(0) + pad * 2, lines.size() * font.lineHeight + pad * 2};
	}

	/** Where the mod sits on a {@code width} x {@code height} screen, kept on it whatever its place says. */
	public static Box place(Font font, int width, int height, View view, PatinaConfig.Mods mods, PatinaConfig.Placed mod) {
		int[] size = size(font, view, mods, mod);
		float w = size[0] * mod.scale;
		float h = size[1] * mod.scale;
		if (mod == mods.armor && PatinaConfig.HOTBAR_LEFT.equals(mod.position)) {
			float right = width / 2 - 91 - (view.leftOffhand ? 29 : 0) - 4;
			float x = right - w;
			// A narrow window can run out of room beside the hotbar.
			float bottom = x < 0 ? height - 50 : height;
			return new Box(Math.max(0, x), Math.max(0, bottom - h), w, h);
		}
		float x = along(mod.horizontal(), mod.x, w, width);
		float y = along(mod.vertical(), mod.y, h, height);
		return new Box(Mth.clamp(x, 0f, Math.max(0f, width - w)), Mth.clamp(y, 0f, Math.max(0f, height - h)), w, h);
	}

	/** Where a box of {@code size} starts on a {@code room} long side, kept to {@code side}. */
	private static float along(int side, float offset, float size, int room) {
		return side < 0 ? MARGIN + offset : side > 0 ? room - MARGIN - size - offset : room / 2f + offset - size / 2f;
	}

	/** The distance from {@code side} of a box at {@code start}: what {@link #along} takes back. */
	private static float offset(int side, float start, float size, int room) {
		return side < 0 ? start - MARGIN : side > 0 ? room - MARGIN - size - start : start + size / 2f - room / 2f;
	}

	/** Which side a box keeps to: the third of the screen its middle is in. */
	private static int side(float start, float size, int room) {
		float middle = start + size / 2f;
		return middle < room / 3f ? -1 : middle > room * 2f / 3f ? 1 : 0;
	}

	/** Gives the mod what is nearest to it of the corners, edges' middles and the middle, and its distance from there. */
	public static void moveTo(PatinaConfig.Placed mod, float x, float y, float w, float h, int width, int height) {
		int horizontal = side(x, w, width);
		int vertical = side(y, h, height);
		String row = vertical < 0 ? "top" : vertical > 0 ? "bottom" : "center";
		String column = horizontal < 0 ? "left" : horizontal > 0 ? "right" : "center";
		mod.position = row.equals(column) ? "center" : row + "_" + column;
		mod.x = snap(offset(horizontal, x, w, width));
		mod.y = snap(offset(vertical, y, h, height));
	}

	/** A box let go next to its margin lines up with it. */
	private static float snap(float offset) {
		return Math.abs(offset) < 3f ? 0f : Math.round(offset * 2f) / 2f;
	}

	/** Draws one mod into its box; {@code alpha} fades it, for a mod that is off in the preview. */
	public static void draw(GuiGraphicsExtractor graphics, Font font, View view, PatinaConfig.Mods mods, PatinaConfig.Placed mod, Box box, float alpha) {
		graphics.pose().pushMatrix();
		graphics.pose().translate(box.x(), box.y());
		graphics.pose().scale(mod.scale);
		if (mod == mods.fps) {
			boolean panel = mods.fps.style.equals("panel");
			int pad = panel ? PANEL_PADDING : 0;
			if (panel) {
				graphics.fill(0, 0, font.width(view.fps) + pad * 2, font.lineHeight - 1 + pad * 2, PatinaClient.fade(PANEL, alpha));
			}
			graphics.text(font, view.fps, pad, pad, PatinaClient.fade(TEXT, alpha), !panel);
		} else if (mod == mods.keystrokes) {
			drawKeys(graphics, font, view.keysDown, mods.keystrokes, alpha);
		} else if (mod == mods.armor) {
			drawArmor(graphics, font, view.armor, mods.armor, alpha);
		} else if (mod == mods.speedometer || mod == mods.ping) {
			boolean background = mod == mods.speedometer ? mods.speedometer.background : mods.ping.background;
			int pad = background ? 3 : 0;
			int[] size = size(font, view, mods, mod);
			if (background) graphics.fill(0, 0, size[0], size[1], PatinaClient.fade(PANEL, alpha));
			graphics.text(font, mod == mods.speedometer ? view.speed : view.ping, pad, pad, PatinaClient.fade(TEXT, alpha));
		} else if (mod == mods.status_effects) {
			for (int i = 0; i < view.effects.size(); i++) {
				EffectPiece effect = view.effects.get(i); int columns = Math.min(mods.status_effects.columns, view.effects.size());
				int x = i % columns * 30, y = i / columns * 30;
				PatinaSettingsScreen.roundedPlate(graphics, x, y, 28, 28, PatinaClient.fade(0xB0101112, alpha), PatinaClient.fade(0xFF66686A, alpha), true, true);
				graphics.blitSprite(RenderPipelines.GUI_TEXTURED, effect.sprite, x + 5, y + 5, 18, 18, PatinaClient.fade(0xFFFFFFFF, alpha));
				if (mods.status_effects.show_amplifier) {
					String level = roman(effect.amplifier + 1); graphics.text(font, level, x + (28 - font.width(level)) / 2, y + 1, PatinaClient.fade(0xFFFFFFFF, alpha));
				}
				if (mods.status_effects.show_duration) {
					String duration = duration(effect.ticks); graphics.text(font, duration, x + (28 - font.width(duration)) / 2, y + 19, PatinaClient.fade(0xFFFFFFFF, alpha));
				}
			}
		} else {
			int pad = mods.coordinates.background ? PANEL_PADDING : 0;
			int[] size = size(font, view, mods, mod);
			if (pad > 0) graphics.fill(0, 0, size[0], size[1], PatinaClient.fade(PANEL, alpha));
			List<String> lines = coordinateLines(view, mods.coordinates);
			for (int i = 0; i < lines.size(); i++) graphics.text(font, lines.get(i), pad, pad + (pad > 0 ? 1 : 0) + i * font.lineHeight, PatinaClient.fade(i == 0 ? TEXT : DIM_TEXT, alpha), pad == 0);
		}
		graphics.pose().popMatrix();
	}

	public static String duration(int ticks) { return ticks < 0 ? "∞" : ticks >= 1200 ? (int)Math.ceil(ticks / 1200.0) + "M" : (int)Math.ceil(ticks / 20.0) + "S"; }
	public static String roman(int level) { return level >= 1 && level <= 10 ? new String[] {"I","II","III","IV","V","VI","VII","VIII","IX","X"}[level - 1] : "" + level; }

	public static List<String> coordinateLines(View view, PatinaConfig.Coordinates options) {
		java.util.ArrayList<String> lines = new java.util.ArrayList<>();
		lines.add(view.coordinates);
		String orientation = options.show_angle ? String.format(Locale.ROOT, "%.1f° / %.1f°", Mth.wrapDegrees(view.yaw), view.pitch) : "";
		if (options.show_direction) orientation += (orientation.isEmpty() ? "" : "  ") + view.facing;
		if (!orientation.isEmpty()) lines.add(orientation);
		if (options.show_biome) lines.add(PatinaLang.get("patina.option.biome") + ": " + view.biome);
		return lines;
	}

	private static void drawKeys(GuiGraphicsExtractor graphics, Font font, boolean[] down, PatinaConfig.Keystrokes options, float alpha) {
		Minecraft minecraft = Minecraft.getInstance();
		List<KeyMapping> moves = List.of(minecraft.options.keyUp, minecraft.options.keyLeft, minecraft.options.keyDown, minecraft.options.keyRight);
		int step = KEY + KEY_GAP;
		drawKey(graphics, font, step, 0, KEY, options.show_labels ? label(moves.get(0)) : "", down[0], alpha);
		for (int i = 0; i < 3; i++) drawKey(graphics, font, step * i, step, KEY, options.show_labels ? label(moves.get(i + 1)) : "", down[i + 1], alpha);
		int y = step * 2;
		if (options.show_mouse) {
			int half = (KEY * 3 + KEY_GAP) / 2;
			drawKey(graphics, font, 0, y, half, options.show_labels ? PatinaLang.get("patina.keystrokes.left") : "", down[4], alpha);
			drawKey(graphics, font, half + KEY_GAP, y, KEY * 3 + KEY_GAP - half, options.show_labels ? PatinaLang.get("patina.keystrokes.right") : "", down[5], alpha);
			y += step;
		}
		int width = KEY * 3 + KEY_GAP * 2;
		int half = (width - KEY_GAP) / 2;
		if (options.show_shift) {
			drawKey(graphics, font, 0, y, options.show_space ? half : width, options.show_labels ? "SHIFT" : "", down[7], alpha);
		}
		if (options.show_space) {
			drawKey(graphics, font, options.show_shift ? half + KEY_GAP : 0, y, options.show_shift ? width - half - KEY_GAP : width,
				options.show_labels ? PatinaLang.get("patina.key.space") : "", down[6], alpha);
		}
	}

	private static final int ARMOR_SLOT = 22;
	private static final int ARMOR_ALERT = 12;
	private static final Identifier HOTBAR_TEXTURE = Identifier.withDefaultNamespace("textures/gui/sprites/hud/hotbar.png");
	private static final Identifier[] EMPTY_ARMOR = {
		Identifier.withDefaultNamespace("container/slot/helmet"), Identifier.withDefaultNamespace("container/slot/chestplate"),
		Identifier.withDefaultNamespace("container/slot/leggings"), Identifier.withDefaultNamespace("container/slot/boots")
	};

	private static int[] armorSize(PatinaConfig.Armor options, int count) {
		// The editor still needs a selectable area when every empty slot is hidden.
		count = Math.max(1, count);
		int alert = options.warning ? ARMOR_ALERT : 0;
		boolean numbers = !"bar".equals(options.display);
		return options.vertical
			? new int[] {alert + ARMOR_SLOT + (numbers ? 28 : 0), count * ARMOR_SLOT}
			: new int[] {count * 20 + 2, alert + ARMOR_SLOT + (numbers ? 10 : 0)};
	}

	/** Only the slots are interactive in the HUD editor. */
	static Box armorSlots(Box box, PatinaConfig.Armor armor) {
		float slot = ARMOR_SLOT * armor.scale;
		return armor.vertical
			? new Box(box.x() + (armor.warning ? ARMOR_ALERT * armor.scale : 0), box.y(), slot, box.height())
			: new Box(box.x(), box.y() + box.height() - slot, box.width(), slot);
	}

	private static void drawArmor(GuiGraphicsExtractor graphics, Font font, ArmorPiece[] stacks, PatinaConfig.Armor options, float alpha) {
		int count = 0;
		for (ArmorPiece piece : stacks) if (piece.present || !options.hide_empty) count++;
		if (count == 0) return;
		int alert = options.warning ? ARMOR_ALERT : 0;
		int tint = PatinaClient.fade(0xFFE2EEE9, alpha);
		boolean bar = "bar".equals(options.display);
		int frameY = alert + (bar ? 0 : 10);
		if (!options.vertical) drawArmorFrame(graphics, 0, frameY, count, tint, alpha);
		int shown = 0;
		for (int slot = 0; slot < stacks.length; slot++) {
			ArmorPiece piece = stacks[slot];
			if (!piece.present && options.hide_empty) continue;
			int x = options.vertical ? alert : shown * 20;
			int y = options.vertical ? shown * ARMOR_SLOT : frameY;
			if (options.vertical) drawArmorFrame(graphics, x, y, 1, tint, alpha);
			shown++;
			if (!piece.present) {
				graphics.blitSprite(RenderPipelines.GUI_TEXTURED, EMPTY_ARMOR[slot], x + 3, y + 3, 16, 16, PatinaClient.fade(0x55FFFFFF, alpha));
				continue;
			}
			if (piece.stack != null) graphics.item(piece.stack, x + 3, y + 3);
			else graphics.blit(RenderPipelines.GUI_TEXTURED, piece.icon, x + 3, y + 3, 0f, 0f, 16, 16, 16, 16);
			if (piece.maximum <= 0) continue;
			int remaining = Mth.clamp(piece.remaining, 0, piece.maximum);
			float ratio = remaining / (float) piece.maximum;
			int color = options.dynamic_color ? 0xFF000000 | Mth.hsvToRgb(ratio / 3f, 1f, 1f) : 0xFFFFFFFF;
			if (bar) {
				if (remaining < piece.maximum) {
					int fill = Math.max(1, Math.round(ratio * 13));
					graphics.fill(x + 5, y + 16, x + 18, y + 18, PatinaClient.fade(0xFF000000, alpha));
					graphics.fill(x + 5, y + 16, x + 5 + fill, y + 17, PatinaClient.fade(color, alpha));
				}
			} else {
				String text = Integer.toString(remaining);
				float scale = Math.min(1f, (options.vertical ? 26f : 18f) / Math.max(1, font.width(text)));
				graphics.pose().pushMatrix();
				graphics.pose().translate(options.vertical ? x + 24 : x + 11 - font.width(text) * scale / 2,
					options.vertical ? y + 7 : y - 9);
				graphics.pose().scale(scale);
				graphics.text(font, text, 0, 0, PatinaClient.fade(color, alpha), true);
				graphics.pose().popMatrix();
			}
			if (options.warning && ratio * 100 <= options.warning_threshold) {
				drawArmorAlert(graphics, options.vertical ? x - 11 : x + 7, options.vertical ? y + 7 : alert - 10, alpha);
			}
		}
	}

	private static void drawArmorFrame(GuiGraphicsExtractor graphics, int x, int y, int slots, int color, float alpha) {
		// Keep the game's two end caps and use only the required number of slots.
		int body = slots * 20;
		graphics.blit(RenderPipelines.GUI_TEXTURED, HOTBAR_TEXTURE, x, y, 0f, 0f, body, 22, body, 22, 182, 22, color);
		graphics.blit(RenderPipelines.GUI_TEXTURED, HOTBAR_TEXTURE, x + body, y, 180f, 0f, 2, 22, 2, 22, 182, 22, color);
		int right = x + body;
		int copperLight = PatinaClient.fade(0xFFD99D74, alpha);
		int copper = PatinaClient.fade(0xFFA66C4C, alpha);
		int copperShade = PatinaClient.fade(0xFF79513B, alpha);
		graphics.fill(x + 1, y + 1, right + 1, y + 2, copperLight);
		graphics.fill(x + 1, y + 2, right + 1, y + 3, copper);
		graphics.fill(x + 1, y + 3, x + 2, y + 20, copper);
		graphics.fill(right, y + 3, right + 1, y + 20, copperShade);
		graphics.fill(x + 2, y + 3, x + 3, y + 19, copperShade);
		graphics.fill(right - 1, y + 3, right, y + 19, copper);
		for (int slot = 1; slot < slots; slot++) {
			int seam = x + slot * 20;
			graphics.fill(seam - 1, y + 3, seam + 3, y + 19, copper);
			graphics.fill(seam, y + 3, seam + 1, y + 19, copperShade);
			graphics.fill(seam + 1, y + 3, seam + 2, y + 19, copperLight);
		}
		graphics.fill(x + 2, y + 19, right, y + 20, copper);
		graphics.fill(x + 1, y + 20, right + 1, y + 21, copperShade);
	}

	private static void drawArmorAlert(GuiGraphicsExtractor graphics, int x, int y, float alpha) {
		int outline = PatinaClient.fade(0xFF651C19, alpha);
		int red = PatinaClient.fade(0xFFE34B3D, alpha);
		int white = PatinaClient.fade(0xFFFFF4E9, alpha);
		for (int row = 0; row < 9; row++) {
			int inset = (8 - row) / 2;
			graphics.fill(x + inset, y + row, x + 9 - inset, y + row + 1, outline);
			if (row >= 2 && row < 8) {
				graphics.fill(x + inset + 1, y + row, x + 8 - inset, y + row + 1, red);
			}
		}
		graphics.fill(x + 4, y + 3, x + 5, y + 6, white);
		graphics.fill(x + 4, y + 7, x + 5, y + 8, white);
	}

	/** A long key name such as a numpad key keeps only what fits. */
	private static String label(KeyMapping key) {
		String name = key.getTranslatedKeyMessage().getString().toUpperCase(Locale.ROOT);
		return name.length() > 3 ? name.substring(0, 3) : name;
	}

	private static void drawKey(GuiGraphicsExtractor graphics, Font font, int x, int y, int width, String label, boolean down, float alpha) {
		graphics.fill(x, y, x + width, y + KEY, PatinaClient.fade(down ? KEY_DOWN : KEY_IDLE, alpha));
		int textWidth = font.width(label);
		float scale = Math.min(1f, (width - 4f) / Math.max(1, textWidth));
		graphics.pose().pushMatrix();
		graphics.pose().translate(x + (float) Math.floor((width - textWidth * scale + 1) / 2),
			y + (float) Math.floor((KEY - font.lineHeight * scale + 2) / 2));
		graphics.pose().scale(scale);
		graphics.text(font, label, 0, 0, PatinaClient.fade(TEXT, alpha), false);
		graphics.pose().popMatrix();
	}
}
