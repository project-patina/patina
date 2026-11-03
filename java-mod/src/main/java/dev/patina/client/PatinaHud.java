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
	public record View(String fps, String coordinates, String facing, boolean[] keysDown) {
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
		return new View(fps, coordinates(player), facing(player.getDirection()), keysDown(minecraft));
	}

	/** The made-up values the editor shows without a world. */
	public static View sampleView() {
		String fps = PatinaConfig.get().mods.fps.accuracy.equals("precise") ? format(143.6, 91) : "144 FPS";
		return new View(fps, "128 64 -256", facing(Direction.NORTH), new boolean[] {true, false, false, false, true, false});
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
			options.keyAttack.isDown(), options.keyUse.isDown()
		};
	}

	/** The mod's size at a scale of 1. */
	public static int[] size(Font font, View view, PatinaConfig.Mods mods, PatinaConfig.Placed mod) {
		if (mod == mods.fps) {
			int pad = mods.fps.style.equals("panel") ? PANEL_PADDING : 0;
			return new int[] {font.width(view.fps) + pad * 2, font.lineHeight - 1 + pad * 2};
		}
		if (mod == mods.keystrokes) {
			return new int[] {KEY * 3 + KEY_GAP * 2, KEY * 3 + KEY_GAP * 2};
		}
		return new int[] {Math.max(font.width(view.coordinates), font.width(view.facing)), font.lineHeight * 2};
	}

	/** Where the mod sits on a {@code width} x {@code height} screen, kept on it whatever its place says. */
	public static Box place(Font font, int width, int height, View view, PatinaConfig.Mods mods, PatinaConfig.Placed mod) {
		int[] size = size(font, view, mods, mod);
		float w = size[0] * mod.scale;
		float h = size[1] * mod.scale;
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
			drawKeys(graphics, font, view.keysDown, alpha);
		} else {
			graphics.text(font, view.coordinates, 0, 0, PatinaClient.fade(TEXT, alpha), true);
			graphics.text(font, view.facing, 0, font.lineHeight, PatinaClient.fade(DIM_TEXT, alpha), true);
		}
		graphics.pose().popMatrix();
	}

	private static void drawKeys(GuiGraphicsExtractor graphics, Font font, boolean[] down, float alpha) {
		Minecraft minecraft = Minecraft.getInstance();
		List<KeyMapping> moves = List.of(minecraft.options.keyUp, minecraft.options.keyLeft, minecraft.options.keyDown, minecraft.options.keyRight);
		int step = KEY + KEY_GAP;
		drawKey(graphics, font, step, 0, KEY, label(moves.get(0)), down[0], alpha);
		drawKey(graphics, font, 0, step, KEY, label(moves.get(1)), down[1], alpha);
		drawKey(graphics, font, step, step, KEY, label(moves.get(2)), down[2], alpha);
		drawKey(graphics, font, step * 2, step, KEY, label(moves.get(3)), down[3], alpha);
		int half = (KEY * 3 + KEY_GAP) / 2;
		drawKey(graphics, font, 0, step * 2, half, PatinaLang.get("patina.keystrokes.left"), down[4], alpha);
		drawKey(graphics, font, half + KEY_GAP, step * 2, half, PatinaLang.get("patina.keystrokes.right"), down[5], alpha);
	}

	/** A long key name such as a numpad key keeps only what fits. */
	private static String label(KeyMapping key) {
		String name = key.getTranslatedKeyMessage().getString().toUpperCase(Locale.ROOT);
		return name.length() > 3 ? name.substring(0, 3) : name;
	}

	private static void drawKey(GuiGraphicsExtractor graphics, Font font, int x, int y, int width, String label, boolean down, float alpha) {
		graphics.fill(x, y, x + width, y + KEY, PatinaClient.fade(down ? KEY_DOWN : KEY_IDLE, alpha));
		graphics.text(font, label, x + (width - font.width(label) + 1) / 2, y + (KEY - font.lineHeight + 2) / 2, PatinaClient.fade(TEXT, alpha), false);
	}
}
