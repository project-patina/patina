package dev.patina.client;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.renderer.RenderPipelines;
import net.minecraft.client.renderer.texture.TextureManager;
import net.minecraft.resources.Identifier;

/** Draws the startup overlay: the animation of the launcher and a thin progress bar. */
public final class PatinaSplash {
	static final Identifier TEXTURE = Identifier.fromNamespaceAndPath(PatinaClient.MOD_ID, "textures/gui/splash.png");

	private static final int COLUMNS = 16;
	private static final int FRAMES = 320;
	private static final int FRAME_SIZE = 160;
	private static final long LOOP_MS = 5120;
	/** Where in a loop the fragments have settled into the whole cube. */
	private static final long CUBE_FROM_MS = 2240;
	private static final long CUBE_UNTIL_MS = 2800;
	private static final float SIZE_PIXELS = 240f;
	private static final float MIN_HEIGHT_SHARE = 0.22f;
	private static final int BAR_GAP = 6;
	private static final int BAR_TRACK = 0x30FFFFFF;

	private static long startedAt = -1;
	private static int lastFrame;

	private PatinaSplash() {
	}

	public static void registerTexture(TextureManager textures) {
		textures.registerAndLoad(TEXTURE, new SplashTexture());
	}

	/** A new overlay plays the animation from its start. */
	public static void reset() {
		startedAt = -1;
	}

	/** The overlay may only leave on a whole cube; a load that runs long waits for the next one. */
	public static boolean mayLeave(long now) {
		if (startedAt < 0) {
			return true;
		}
		long into = (now - startedAt) % LOOP_MS;
		return now - startedAt >= CUBE_FROM_MS && into >= CUBE_FROM_MS && into < CUBE_UNTIL_MS;
	}

	/** Side of the animation in GUI units: the same size on screen whatever the GUI scale. */
	private static int size(GuiGraphicsExtractor graphics) {
		float scale = Minecraft.getInstance().getWindow().getGuiScale();
		return Math.round(Math.max(SIZE_PIXELS / scale, graphics.guiHeight() * MIN_HEIGHT_SHARE));
	}

	/** @param leaving keeps the frame still while the overlay fades, so the cube stays whole */
	public static void drawAnimation(GuiGraphicsExtractor graphics, long now, int color, boolean leaving) {
		if (startedAt < 0) {
			startedAt = now;
		}
		if (!leaving) {
			lastFrame = (int) ((now - startedAt) % LOOP_MS * FRAMES / LOOP_MS);
		}
		int frame = lastFrame;
		int size = size(graphics);
		graphics.blit(
			RenderPipelines.GUI_TEXTURED, TEXTURE,
			(graphics.guiWidth() - size) / 2, (graphics.guiHeight() - size) / 2,
			frame % COLUMNS * FRAME_SIZE, frame / COLUMNS * FRAME_SIZE,
			size, size, FRAME_SIZE, FRAME_SIZE,
			COLUMNS * FRAME_SIZE, ((FRAMES + COLUMNS - 1) / COLUMNS) * FRAME_SIZE,
			color
		);
	}

	public static void drawBar(GuiGraphicsExtractor graphics, float progress, float opacity) {
		int size = size(graphics);
		int width = size * 4 / 5;
		int x = (graphics.guiWidth() - width) / 2;
		int y = (graphics.guiHeight() + size) / 2 + BAR_GAP;
		int filled = Math.round(width * progress);
		graphics.fill(x, y, x + width, y + 1, PatinaClient.fade(BAR_TRACK, opacity));
		graphics.fill(x, y, x + filled, y + 1, PatinaClient.fade(0xFF000000 | PatinaClient.ACCENT, opacity));
	}
}
