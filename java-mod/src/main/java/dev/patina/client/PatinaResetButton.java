package dev.patina.client;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.components.Button;
import net.minecraft.network.chat.Component;

/** The HUD editor's reset action, with an undo arrow and a visible label. */
public final class PatinaResetButton extends Button {
	public static final int WIDTH = 51;
	private static final int HEIGHT = 15;
	private static final int[] ICON = {
		0b100011100, 0b100100010, 0b101000001,
		0b111100001, 0b000000001, 0b000000001,
		0b010000010, 0b001000100, 0b000111000
	};

	private final float textScale;

	public PatinaResetButton(int x, int y, OnPress onPress) {
		this(x, y, WIDTH, PatinaLang.text("patina.editor.reset"), 1f, onPress);
	}

	PatinaResetButton(int x, int y, int width, Component label, float textScale, OnPress onPress) {
		super(x, y, width, HEIGHT, label, onPress, DEFAULT_NARRATION);
		this.textScale = textScale;
	}

	@Override
	protected void extractContents(GuiGraphicsExtractor graphics, int mouseX, int mouseY, float partialTick) {
		int left = this.getX(), top = this.getY();
		boolean highlighted = this.isHoveredOrFocused();
		int fill = highlighted ? 0xF7243937 : 0xF011181B;
		int edge = highlighted ? 0xFF6FDCC3 : 0xFF475B59;
		PatinaSettingsScreen.roundedPlate(graphics, left, top, this.getWidth(), HEIGHT, fill, edge, true, true);
		var font = Minecraft.getInstance().font;
		float textWidth = font.width(this.getMessage()) * this.textScale;
		int x = left + (int) ((this.getWidth() - 13 - textWidth) / 2), y = top + 3;
		int color = PatinaClient.fade(highlighted ? 0xFF6FDCC3 : 0xFFD8EEEA, this.alpha);
		PatinaSettingsScreen.drawLabel(graphics, font, this.getMessage(), x + 13, top + (HEIGHT - 8 * this.textScale) / 2, this.textScale, 0xFFE9F3F0);
		for (int row = 0; row < ICON.length; row++) {
			for (int column = 0; column < 9; column++) {
				if ((ICON[row] & (1 << (8 - column))) != 0) {
					graphics.fill(x + column, y + row, x + column + 1, y + row + 1, color);
				}
			}
		}
	}
}
