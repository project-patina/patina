package dev.patina.client;

import com.mojang.blaze3d.platform.NativeImage;
import java.io.IOException;
import java.io.InputStream;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.renderer.texture.DynamicTexture;
import net.minecraft.resources.Identifier;

/** The picture behind the menus: the launcher's black Mono landscape, the same in the Rust client. */
public final class PatinaBackground {
	private static final String PATH = "/assets/patina/textures/gui/background/landscape-7.png";
	private static final Identifier ID = Identifier.fromNamespaceAndPath(PatinaClient.MOD_ID, "background/mono");
	private static final int FALLBACK = 0xFF1F2122;

	private static boolean loaded;
	private static int imageWidth;
	private static int imageHeight;

	private PatinaBackground() {
	}

	public static void drawMenu(GuiGraphicsExtractor graphics, int width, int height) {
		draw(graphics, 0, 0, width, height);
	}

	/** Covers the rectangle like CSS {@code cover}, keeping the ground as the launcher banners do. */
	public static void draw(GuiGraphicsExtractor graphics, int x, int y, int width, int height) {
		if (!load()) {
			graphics.fill(x, y, x + width, y + height, FALLBACK);
			return;
		}
		float scale = Math.max((float) width / imageWidth, (float) height / imageHeight);
		float shownWidth = width / scale / imageWidth;
		float shownHeight = height / scale / imageHeight;
		float u = (1f - shownWidth) / 2f;
		float v = 1f - shownHeight;
		graphics.blit(ID, x, y, x + width, y + height, u, u + shownWidth, v, v + shownHeight);
	}

	private static boolean load() {
		if (!loaded) {
			loaded = true;
			try (InputStream stream = PatinaBackground.class.getResourceAsStream(PATH)) {
				if (stream != null) {
					NativeImage image = NativeImage.read(stream);
					imageWidth = image.getWidth();
					imageHeight = image.getHeight();
					Minecraft.getInstance().getTextureManager().register(ID, new DynamicTexture(() -> "Patina background", image));
				}
			} catch (IOException e) {
				PatinaClient.LOGGER.warn("Could not read {}", PATH, e);
			}
		}
		return imageWidth > 0;
	}
}
