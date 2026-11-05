package dev.patina.client;

import com.mojang.blaze3d.platform.NativeImage;
import java.io.IOException;
import java.io.InputStream;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.renderer.texture.DynamicTexture;
import net.minecraft.resources.Identifier;

/** The title landscape and the Minecraft scene used to lay out HUD modules. */
public final class PatinaBackground {
	private static final Backdrop MENU = new Backdrop("landscape-7.png", "mono");
	private static final Backdrop HUD = new Backdrop("hud-scene.png", "hud_scene");
	private static final int HUD_SCENE_HEIGHT = 1080;

	private PatinaBackground() { }

	public static void drawMenu(GuiGraphicsExtractor graphics, int width, int height) {
		drawLauncherScene(graphics, 0, 0, width, height);
	}

	static void drawLauncherScene(GuiGraphicsExtractor graphics, int x, int y, int width, int height) {
		drawCover(graphics, MENU, x, y, width, height, Integer.MAX_VALUE, 1f);
	}

	static void drawHudScene(GuiGraphicsExtractor graphics, int x, int y, int width, int height) {
		drawCover(graphics, HUD, x, y, width, height, HUD_SCENE_HEIGHT, 0.5f);
	}

	private static void drawCover(GuiGraphicsExtractor graphics, Backdrop backdrop, int x, int y, int width, int height, int maxHeight, float alignY) {
		if (width <= 0 || height <= 0) return;
		if (!backdrop.load()) {
			graphics.fill(x, y, x + width, y + height, 0xFF1F2122);
			return;
		}
		int sourceHeight = Math.min(backdrop.height, maxHeight);
		float scale = Math.max((float) width / backdrop.width, (float) height / sourceHeight);
		float shownWidth = width / scale;
		float shownHeight = height / scale;
		float u = (backdrop.width - shownWidth) / (2f * backdrop.width);
		float v = (sourceHeight - shownHeight) * alignY / backdrop.height;
		graphics.blit(backdrop.id, x, y, x + width, y + height, u, u + shownWidth / backdrop.width, v, v + shownHeight / backdrop.height);
	}

	private static final class Backdrop {
		private final String path;
		private final Identifier id;
		private boolean attempted;
		private int width;
		private int height;

		Backdrop(String file, String name) {
			this.path = "/assets/patina/textures/gui/background/" + file;
			this.id = Identifier.fromNamespaceAndPath(PatinaClient.MOD_ID, "background/" + name);
		}

		boolean load() {
			if (!this.attempted) {
				this.attempted = true;
				try (InputStream stream = PatinaBackground.class.getResourceAsStream(this.path)) {
					if (stream != null) {
						NativeImage image = NativeImage.read(stream);
						this.width = image.getWidth();
						this.height = image.getHeight();
						Minecraft.getInstance().getTextureManager().register(this.id, new DynamicTexture(() -> this.path, image));
					}
				} catch (IOException e) {
					PatinaClient.LOGGER.warn("Could not read {}", this.path, e);
				}
			}
			return this.width > 0;
		}
	}
}
