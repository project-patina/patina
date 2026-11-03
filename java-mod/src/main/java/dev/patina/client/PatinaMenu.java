package dev.patina.client;

import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.resources.Identifier;
import net.minecraft.util.ARGB;

/**
 * The Patina look. Every screen, also those of other mods, draws its widgets through a few
 * vanilla sprites and textures, so replacing those here themes them all with no screen touched.
 */
public final class PatinaMenu {
	private static final int ACCENT = 0xFF000000 | PatinaClient.ACCENT;
	private static final int FILL = 0xA00C1012;
	private static final int FILL_HOVER = 0xC0141C1E;
	private static final int FILL_DISABLED = 0x70101214;
	private static final int BORDER = 0x60FFFFFF;
	private static final int BORDER_DISABLED = 0x28FFFFFF;
	private static final int SIGNATURE = 0xFF6FDCC3;
	private static final int SCREEN = 0xA80A0E10;
	private static final int SCREEN_IN_WORLD = 0x900A0E10;
	private static final int LIST = 0x50000000;
	private static final int LINE = 0x30FFFFFF;

	private PatinaMenu() {
	}

	/** Returns whether the sprite was one of ours to draw. */
	public static boolean drawSprite(GuiGraphicsExtractor graphics, Identifier sprite, int x, int y, int width, int height, int color) {
		if (!sprite.getNamespace().equals("minecraft")) {
			return false;
		}
		float alpha = ARGB.alphaFloat(color);
		switch (sprite.getPath()) {
			case "widget/button" -> button(graphics, x, y, width, height, alpha, FILL, BORDER, false, true);
			case "widget/button_highlighted" -> button(graphics, x, y, width, height, alpha, FILL_HOVER, ACCENT, true, true);
			case "widget/button_disabled" -> button(graphics, x, y, width, height, alpha, FILL_DISABLED, BORDER_DISABLED, false, false);
			case "widget/slider" -> box(graphics, x, y, width, height, alpha, FILL, BORDER);
			case "widget/slider_highlighted" -> box(graphics, x, y, width, height, alpha, FILL_HOVER, ACCENT);
			case "widget/slider_handle" -> graphics.fill(x, y, x + width, y + height, PatinaClient.fade(0xFFD8E2E0, alpha));
			case "widget/slider_handle_highlighted" -> graphics.fill(x, y, x + width, y + height, PatinaClient.fade(ACCENT, alpha));
			case "widget/text_field" -> box(graphics, x, y, width, height, alpha, 0xC0080A0C, BORDER);
			case "widget/text_field_highlighted" -> box(graphics, x, y, width, height, alpha, 0xC0080A0C, ACCENT);
			case "widget/checkbox" -> box(graphics, x, y, width, height, alpha, FILL, BORDER);
			case "widget/checkbox_highlighted" -> box(graphics, x, y, width, height, alpha, FILL_HOVER, ACCENT);
			case "widget/checkbox_selected" -> checked(graphics, x, y, width, height, alpha, BORDER);
			case "widget/checkbox_selected_highlighted" -> checked(graphics, x, y, width, height, alpha, ACCENT);
			case "widget/scroller" -> graphics.fill(x, y, x + width, y + height, PatinaClient.fade(0x90FFFFFF, alpha));
			case "widget/scroller_background" -> graphics.fill(x, y, x + width, y + height, PatinaClient.fade(0x40000000, alpha));
			case "widget/tab" -> graphics.fill(x, y, x + width, y + height, PatinaClient.fade(0x60000000, alpha));
			case "widget/tab_highlighted" -> graphics.fill(x, y, x + width, y + height, PatinaClient.fade(FILL_HOVER, alpha));
			case "widget/tab_selected", "widget/tab_selected_highlighted" -> {
				graphics.fill(x, y, x + width, y + height, PatinaClient.fade(FILL_HOVER, alpha));
				graphics.fill(x, y + height - 2, x + width, y + height, PatinaClient.fade(ACCENT, alpha));
			}
			default -> {
				return false;
			}
		}
		return true;
	}

	/** Returns whether the texture was one of ours to draw. */
	public static boolean drawTexture(GuiGraphicsExtractor graphics, Identifier texture, int x, int y, int width, int height) {
		if (!texture.getNamespace().equals("minecraft")) {
			return false;
		}
		switch (texture.getPath()) {
			case "textures/gui/menu_background.png" -> graphics.fill(x, y, x + width, y + height, SCREEN);
			case "textures/gui/inworld_menu_background.png" -> graphics.fill(x, y, x + width, y + height, SCREEN_IN_WORLD);
			case "textures/gui/menu_list_background.png", "textures/gui/inworld_menu_list_background.png", "textures/gui/tab_header_background.png" ->
				graphics.fill(x, y, x + width, y + height, LIST);
			case "textures/gui/header_separator.png", "textures/gui/inworld_header_separator.png" ->
				graphics.fill(x, y + height - 1, x + width, y + height, LINE);
			case "textures/gui/footer_separator.png", "textures/gui/inworld_footer_separator.png" ->
				graphics.fill(x, y, x + width, y + 1, LINE);
			default -> {
				return false;
			}
		}
		return true;
	}

	private static void button(GuiGraphicsExtractor graphics, int x, int y, int width, int height, float alpha, int fill, int border, boolean hover, boolean active) {
		if (!PatinaTitle.drawPanel(alpha, hover, active)) {
			box(graphics, x, y, width, height, alpha, fill, border);
		}
	}

	private static void box(GuiGraphicsExtractor graphics, int x, int y, int width, int height, float alpha, int fill, int border) {
		graphics.fill(x, y, x + width, y + height, PatinaClient.fade(fill, alpha));
		graphics.outline(x, y, width, height, PatinaClient.fade(border, alpha));
	}

	private static void checked(GuiGraphicsExtractor graphics, int x, int y, int width, int height, float alpha, int border) {
		box(graphics, x, y, width, height, alpha, FILL, border);
		int inset = Math.max(3, width / 4);
		graphics.fill(x + inset, y + inset, x + width - inset, y + height - inset, PatinaClient.fade(ACCENT, alpha));
	}

	/** Sits on the line above the version text, which stays where the game puts it. */
	public static void drawSignature(GuiGraphicsExtractor graphics, Font font, int screenHeight, float alpha) {
		graphics.text(font, PatinaClient.signature(), 2, screenHeight - 20, PatinaClient.fade(SIGNATURE, alpha));
	}
}
