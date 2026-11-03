package dev.patina.client;

import com.mojang.blaze3d.platform.cursor.CursorTypes;
import java.util.List;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.components.AbstractWidget;
import net.minecraft.client.gui.components.Button;
import net.minecraft.client.gui.components.CycleButton;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.client.input.MouseButtonEvent;
import net.minecraft.client.resources.sounds.SimpleSoundInstance;
import net.minecraft.network.chat.CommonComponents;
import net.minecraft.network.chat.Component;
import net.minecraft.sounds.SoundEvents;

/**
 * The Patina Client screen: a small panel with each mod and its switch, the FPS counter's
 * options, and the way into the HUD editor, where the mods are placed on the game itself.
 */
public final class PatinaSettingsScreen extends Screen {
	private static final int PANEL_WIDTH = 280;
	private static final int PADDING = 8;
	private static final int TITLE = 14;
	private static final int ROW = 20;
	private static final int ROW_GAP = 2;
	private static final int CAPTION = 11;
	private static final int SECTION_GAP = 6;
	private static final int PILL_WIDTH = 24;
	private static final int PILL_HEIGHT = 12;
	private static final int KNOB = 10;
	private static final int PANEL = 0xB80C1012;
	private static final int LINE = 0x30FFFFFF;
	private static final int HEADING = 0xFF6FDCC3;
	private static final int HINT = 0xFF9FB0AC;
	private static final int ACCENT = 0xFF000000 | PatinaClient.ACCENT;
	private static final int FILL = 0xA00C1012;
	private static final int FILL_HOVER = 0xC0141C1E;
	private static final int BORDER = 0x60FFFFFF;
	private static final int PILL_OFF = 0x40FFFFFF;
	private static final int KNOB_COLOR = 0xFFF0F3F3;
	private static final String[] NAMES = {"patina.mod.fps", "patina.mod.keystrokes", "patina.mod.coordinates"};

	private final Screen parent;
	private final PatinaConfig config = PatinaConfig.get();
	private int panelX;
	private int panelY;
	private int panelWidth;
	private int panelHeight;

	public PatinaSettingsScreen(Screen parent) {
		super(Component.literal("Patina Client"));
		this.parent = parent;
	}

	private int rowY(int row) {
		return this.panelY + PADDING + TITLE + row * (ROW + ROW_GAP);
	}

	private int captionY() {
		return this.rowY(NAMES.length) - ROW_GAP + SECTION_GAP;
	}

	@Override
	protected void init() {
		this.panelWidth = Math.min(this.width - 24, PANEL_WIDTH);
		this.panelHeight = PADDING + TITLE + NAMES.length * (ROW + ROW_GAP) - ROW_GAP + SECTION_GAP + CAPTION + ROW + SECTION_GAP + ROW + ROW_GAP + ROW + PADDING;
		this.panelX = (this.width - this.panelWidth) / 2;
		this.panelY = (this.height - this.panelHeight) / 2;

		int x = this.panelX + PADDING;
		int inner = this.panelWidth - PADDING * 2;
		int half = (inner - 4) / 2;
		int y = this.captionY() + CAPTION;
		PatinaConfig.Fps fps = this.config.mods.fps;
		AbstractWidget accuracy = this.choice("patina.option.accuracy", "patina.option.accuracy.", fps.accuracy, List.of("whole", "precise"), value -> fps.accuracy = value);
		accuracy.setRectangle(half, ROW, x, y);
		this.addRenderableWidget(accuracy);
		AbstractWidget style = this.choice("patina.option.style", "patina.option.style.", fps.style, List.of("text", "panel"), value -> fps.style = value);
		style.setRectangle(half, ROW, x + half + 4, y);
		this.addRenderableWidget(style);
		y += ROW + SECTION_GAP;
		this.addRenderableWidget(Button.builder(PatinaLang.text("patina.settings.edit"), press -> this.minecraft.gui.setScreen(new PatinaHudEditorScreen(this)))
			.bounds(x, y, inner, ROW).build());
		y += ROW + ROW_GAP;
		this.addRenderableWidget(Button.builder(CommonComponents.GUI_DONE, press -> this.onClose()).bounds(x, y, inner, ROW).build());
	}

	private AbstractWidget choice(String name, String prefix, String current, List<String> values, java.util.function.Consumer<String> set) {
		return CycleButton.<String>builder(value -> PatinaLang.text(prefix + value), current)
			.withValues(values)
			.create(0, 0, ROW, ROW, PatinaLang.text(name), (button, value) -> {
				set.accept(value);
				this.config.save();
			});
	}

	@Override
	public void onClose() {
		this.minecraft.gui.setScreen(this.parent);
	}

	/** The mod row under the pointer, or -1. */
	private int rowAt(double mouseX, double mouseY) {
		int x = this.panelX + PADDING;
		for (int i = 0; i < NAMES.length; i++) {
			int y = this.rowY(i);
			if (mouseX >= x && mouseX < x + this.panelWidth - PADDING * 2 && mouseY >= y && mouseY < y + ROW) {
				return i;
			}
		}
		return -1;
	}

	@Override
	public boolean mouseClicked(MouseButtonEvent event, boolean doubleClick) {
		int row = this.rowAt(event.x(), event.y());
		if (event.button() == 0 && row >= 0) {
			PatinaConfig.Placed mod = this.config.mods.all().get(row);
			mod.enabled = !mod.enabled;
			this.config.save();
			this.minecraft.getSoundManager().play(SimpleSoundInstance.forUI(SoundEvents.UI_BUTTON_CLICK, 1f));
			return true;
		}
		return super.mouseClicked(event, doubleClick);
	}

	@Override
	public void extractRenderState(GuiGraphicsExtractor graphics, int mouseX, int mouseY, float a) {
		graphics.fill(this.panelX, this.panelY, this.panelX + this.panelWidth, this.panelY + this.panelHeight, PANEL);
		graphics.outline(this.panelX, this.panelY, this.panelWidth, this.panelHeight, LINE);
		graphics.text(this.font, this.title, this.panelX + PADDING, this.panelY + PADDING + 1, HEADING);
		int x = this.panelX + PADDING;
		int width = this.panelWidth - PADDING * 2;
		int hovered = this.rowAt(mouseX, mouseY);
		for (int i = 0; i < NAMES.length; i++) {
			int y = this.rowY(i);
			boolean hover = i == hovered;
			boolean on = this.config.mods.all().get(i).enabled;
			graphics.fill(x, y, x + width, y + ROW, hover ? FILL_HOVER : FILL);
			graphics.outline(x, y, width, ROW, hover ? ACCENT : BORDER);
			graphics.text(this.font, PatinaLang.get(NAMES[i]), x + 6, y + (ROW - 8) / 2, 0xFFFFFFFF);
			int pillX = x + width - 6 - PILL_WIDTH;
			int pillY = y + (ROW - PILL_HEIGHT) / 2;
			graphics.fill(pillX, pillY, pillX + PILL_WIDTH, pillY + PILL_HEIGHT, on ? ACCENT : PILL_OFF);
			int knobX = on ? pillX + PILL_WIDTH - 1 - KNOB : pillX + 1;
			graphics.fill(knobX, pillY + 1, knobX + KNOB, pillY + 1 + KNOB, KNOB_COLOR);
		}
		if (hovered >= 0) {
			graphics.requestCursor(CursorTypes.POINTING_HAND);
		}
		graphics.text(this.font, PatinaLang.get(NAMES[0]), x, this.captionY() + 1, HINT);
		super.extractRenderState(graphics, mouseX, mouseY, a);
	}
}
