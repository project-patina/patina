package dev.patina.client;

import java.util.Locale;
import java.util.function.BooleanSupplier;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.components.Button;
import net.minecraft.client.gui.components.EditBox;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.network.chat.Component;
import net.minecraft.util.FormattedCharSequence;

/** A compact catalogue of the installed HUD modules. */
public final class PatinaSettingsScreen extends Screen {
	static final int PANEL = 0xED0C1012;
	static final int LINE = 0x30FFFFFF;
	static final int HEADING = 0xFF6FDCC3;
	static final int HINT = 0xFF9FB0AC;
	static final int ACCENT = 0xFF000000 | PatinaClient.ACCENT;
	private static final int PADDING = 12;
	private static final int GAP = 10;
	private int cardHeight;
	private int panelHeight;
	private int frameX, frameY, frameWidth, frameHeight;
	private static final int SWITCH_HEIGHT = 14;
	private final ModTile[] tiles = new ModTile[16];
	private final Switch[] switches = new Switch[16];
	private String query = "";
	private int scrollRow;
	private int visibleRows;
	private int columns, tileWidth, tileTop;
	private int results;
	static final String[] NAMES = {"patina.mod.fps", "patina.mod.keystrokes", "patina.mod.coordinates", "patina.mod.armor", "patina.mod.fullbright", "patina.mod.appleskin", "patina.mod.shulker_tooltip", "patina.mod.speedometer", "patina.mod.zoom", "patina.mod.ping", "patina.mod.perspective", "patina.mod.horse_stats", "patina.mod.status_effects", "patina.mod.weather", "patina.mod.death_info", "patina.mod.no_fog"};
	private final Screen parent;
	private final PatinaConfig config = PatinaConfig.get();
	private int panelX, panelY, panelWidth;

	public PatinaSettingsScreen(Screen parent) {
		super(Component.literal("Patina Client"));
		this.parent = parent;
	}

	static int panelWidth(int width) {
		return Math.min(width - 16, 312);
	}

	static int panelHeight(int height, int minimum) {
		return Math.min(height - 16, minimum);
	}

	@Override
	protected void init() {
		this.panelWidth = panelWidth(this.width);
		this.panelX = (this.width - this.panelWidth) / 2;
		int columns = this.panelWidth >= 264 ? 4 : 2;
		this.panelHeight = panelHeight(this.height, 288);
		this.frameWidth = Math.min(this.width - 16, 396);
		this.frameHeight = panelHeight(this.height, this.panelHeight + 44);
		this.frameX = (this.width - this.frameWidth) / 2;
		this.frameY = (this.height - this.frameHeight) / 2;
		this.panelY = this.frameY + Math.min(8, (this.frameHeight - this.panelHeight) / 2);
		boolean compactHeader = this.panelWidth >= 252;
		int tileTop = compactHeader ? 36 : 50;
		this.cardHeight = 53;
		this.visibleRows = Math.max(1, (this.frameY + this.frameHeight - 8 - this.panelY - tileTop + GAP) / (this.cardHeight + GAP));
		int tileWidth = (this.panelWidth - PADDING * 2 - GAP * (columns - 1)) / columns;
		this.columns = columns; this.tileWidth = tileWidth; this.tileTop = tileTop;
		Component searchLabel = PatinaLang.text("patina.settings.search");
		int closeX = this.frameX + this.frameWidth - 26;
		int searchWidth = this.panelWidth - (compactHeader ? 198 : 2 * PADDING);
		int searchX = compactHeader ? closeX - 6 - searchWidth : this.panelX + PADDING;
		int layoutX = compactHeader ? searchX - 82 : this.panelX + 84;
		EditBox search = this.addRenderableWidget(new EditBox(this.font, searchX,
			this.panelY + (compactHeader ? 6 : 26), searchWidth, 14, searchLabel));
		search.setHint(searchLabel);
		search.setMaxLength(64);
		search.setValue(this.query);
		search.setResponder(value -> { this.scrollRow = 0; this.filter(value, columns, tileWidth, tileTop); });
		for (int i = 0; i < NAMES.length; i++) {
			int mod = i;
			this.tiles[i] = this.addRenderableWidget(new ModTile(0, 0, tileWidth, this.cardHeight - SWITCH_HEIGHT, i));
			this.switches[i] = this.addRenderableWidget(new Switch(this.font, PatinaLang.text(NAMES[i]), tileWidth, SWITCH_HEIGHT,
				() -> this.enabled(mod), () -> this.toggle(mod), 1f, true));
		}
		this.filter(this.query, columns, tileWidth, tileTop);
		this.addRenderableWidget(Button.builder(PatinaLang.text("patina.settings.edit"), press -> this.minecraft.gui.setScreen(new PatinaHudEditorScreen(this)))
			.bounds(layoutX, this.panelY + 6, 71, 14).build());
		this.addRenderableWidget(Button.builder(Component.literal("×"), press -> this.onClose())
			.bounds(closeX, this.panelY + 6, 14, 14).build());
	}

	private boolean enabled(int mod) {
		return this.config.mods.catalogue().get(mod).enabled;
	}

	private void toggle(int mod) {
		PatinaConfig.ToggleMod entry = this.config.mods.catalogue().get(mod);
		entry.enabled = !entry.enabled;
		this.config.save();
	}

	@Override
	public boolean mouseScrolled(double x, double y, double horizontal, double vertical) {
		if (vertical == 0) return super.mouseScrolled(x, y, horizontal, vertical);
		this.scrollRow = Math.clamp(this.scrollRow - (int) Math.signum(vertical), 0,
			Math.max(0, Math.ceilDiv(this.results, this.columns) - this.visibleRows));
		this.filter(this.query, this.columns, this.tileWidth, this.tileTop);
		return true;
	}

	private void filter(String value, int columns, int tileWidth, int tileTop) {
		this.query = value;
		this.results = 0;
		String needle = value.strip().toLowerCase(Locale.ROOT);
		for (int i = 0; i < NAMES.length; i++) {
			boolean visible = PatinaLang.get(NAMES[i]).toLowerCase(Locale.ROOT).contains(needle)
				|| NAMES[i].substring("patina.mod.".length()).contains(needle);
			if (i == 4) visible |= "full bright brightness jasnosc jasność".contains(needle);
			this.tiles[i].visible = this.switches[i].visible = false;
			if (visible) {
				int x = this.panelX + PADDING + (this.results % columns) * (tileWidth + GAP);
				int row = this.results / columns - this.scrollRow;
				int y = this.panelY + tileTop + row * (this.cardHeight + GAP);
				this.tiles[i].visible = this.switches[i].visible = row >= 0 && row < this.visibleRows;
				this.tiles[i].setX(x);
				this.tiles[i].setY(y);
				this.switches[i].setX(x);
				this.switches[i].setY(y + this.cardHeight - SWITCH_HEIGHT);
				this.results++;
			}
		}
	}

	@Override
	public void onClose() {
		this.minecraft.gui.setScreen(this.parent);
	}

	@Override
	public void extractRenderState(GuiGraphicsExtractor graphics, int mouseX, int mouseY, float a) {
		graphics.fill(this.frameX, this.frameY, this.frameX + this.frameWidth, this.frameY + this.frameHeight, PANEL);
		graphics.outline(this.frameX, this.frameY, this.frameWidth, this.frameHeight, LINE);
		graphics.fill(this.frameX, this.frameY, this.frameX + this.frameWidth, this.frameY + 2, ACCENT);
		float titleScale = this.frameWidth >= 330 ? 1.15f : 1f;
		drawLabel(graphics, this.font, this.title, this.frameX + PADDING, this.panelY + (14 - 8 * titleScale) / 2 + 6, titleScale, HEADING);
		if (this.results == 0) {
			graphics.text(this.font, PatinaLang.text("patina.settings.no_results"), this.panelX + PADDING, this.panelY + (this.panelWidth >= 252 ? 42 : 56), HINT);
		}
		super.extractRenderState(graphics, mouseX, mouseY, a);
	}

	private final class ModTile extends Button {
		private final int mod;

		ModTile(int x, int y, int width, int height, int mod) {
			super(x, y, width, height, PatinaLang.text(NAMES[mod]), press -> {
				if (mod == 4) PatinaSettingsScreen.this.toggle(mod);
				else if (mod >= 5) PatinaSettingsScreen.this.minecraft.gui.setScreen(new PatinaUtilitySettingsScreen(PatinaSettingsScreen.this, mod));
				else PatinaSettingsScreen.this.minecraft.gui.setScreen(new PatinaModuleSettingsScreen(PatinaSettingsScreen.this, mod));
			}, DEFAULT_NARRATION);
			this.mod = mod;
		}

		@Override
		protected void extractContents(GuiGraphicsExtractor graphics, int mouseX, int mouseY, float a) {
			boolean on = enabled(this.mod);
			int x = this.getX(), y = this.getY(), width = this.getWidth();
			roundedPlate(graphics, x, y, width, this.getHeight(), this.isHoveredOrFocused() ? 0xF01A2527 : 0xC010181B,
				this.isHoveredOrFocused() ? ACCENT : LINE, true, false);
			int stripe = on ? ACCENT : 0x405C716C;
			graphics.fill(x + 2, y + 1, x + width - 2, y + 2, stripe);
			graphics.fill(x + 1, y + 2, x + width - 1, y + 3, stripe);
			graphics.pose().pushMatrix();
			graphics.pose().translate(x + width / 2 - 7, y + 6);
			graphics.pose().scale(0.58f);
			drawIcon(graphics, this.mod, 0, 0, on ? HEADING : HINT);
			graphics.pose().popMatrix();
			float textScale = 0.86f;
			var lines = font.split(this.getMessage(), (int) ((width - 4) / textScale));
			int count = Math.min(2, lines.size());
			for (int i = 0; i < count; i++) {
				FormattedCharSequence line = lines.get(i);
				graphics.pose().pushMatrix();
				graphics.pose().translate(x + (width - font.width(line) * textScale) / 2f,
					y + this.getHeight() - 9 - (count - 1 - i) * 8);
				graphics.pose().scale(textScale);
				graphics.text(font, line, 0, 0, 0xFFF0F3F3);
				graphics.pose().popMatrix();
			}
		}
	}

	/** Two one-pixel steps soften the outside corners while the tile and switch share a straight seam. */
	static void roundedPlate(GuiGraphicsExtractor graphics, int x, int y, int w, int h, int fill, int edge, boolean top, boolean bottom) {
		roundedFill(graphics, x, y, w, h, edge, top ? 2 : 0, bottom ? 2 : 0);
		roundedFill(graphics, x + 1, y + 1, w - 2, h - 2, fill, top ? 1 : 0, bottom ? 1 : 0);
	}

	private static void roundedFill(GuiGraphicsExtractor graphics, int x, int y, int w, int h, int color, int top, int bottom) {
		graphics.fill(x, y + top, x + w, y + h - bottom, color);
		for (int inset = 1; inset <= top; inset++) {
			int row = y + top - inset;
			graphics.fill(x + inset, row, x + w - inset, row + 1, color);
		}
		for (int inset = 1; inset <= bottom; inset++) {
			int row = y + h - bottom + inset - 1;
			graphics.fill(x + inset, row, x + w - inset, row + 1, color);
		}
	}

	static void drawLabel(GuiGraphicsExtractor graphics, Font font, Component label, float x, float y, float scale, int color) {
		graphics.pose().pushMatrix();
		graphics.pose().translate(x, y);
		graphics.pose().scale(scale);
		graphics.text(font, label, 0, 0, color);
		graphics.pose().popMatrix();
	}

	static final class Switch extends Button {
		private final Font font;
		private final Component label;
		private final BooleanSupplier read;
		private final float textScale;
		private final boolean rounded;

		Switch(Font font, Component label, int width, int height, BooleanSupplier read, Runnable toggle, float textScale, boolean rounded) {
			super(0, 0, width, height, label, press -> toggle.run(), DEFAULT_NARRATION);
			this.font = font;
			this.label = label;
			this.read = read;
			this.textScale = textScale;
			this.rounded = rounded;
			updateMessage();
		}

		private void updateMessage() {
			this.setMessage(this.label.copy().append(" ").append(PatinaLang.text(this.read.getAsBoolean() ? "patina.value.on" : "patina.value.off")));
		}

		@Override
		public void onPress(net.minecraft.client.input.InputWithModifiers input) {
			super.onPress(input);
			updateMessage();
		}

		@Override
		protected void extractContents(GuiGraphicsExtractor graphics, int mouseX, int mouseY, float a) {
			boolean on = this.read.getAsBoolean();
			int x = this.getX(), y = this.getY(), w = this.getWidth(), h = this.getHeight();
			int half = w / 2;
			int start = on ? x + half : x;
			int end = on ? x + w : x + half;
			int edge = this.isHoveredOrFocused() ? HEADING : LINE;
			if (this.rounded) {
				roundedPlate(graphics, x, y, w, h, 0xFF191E21, edge, false, true);
			} else {
				graphics.fill(x, y, x + w, y + h, 0xFF191E21);
				graphics.outline(x, y, w, h, edge);
			}
			int activeColor = on ? 0xFF267563 : 0xFF4A5054;
			graphics.fill(start + 1, y + 1, end - 1, y + h - (this.rounded ? 2 : 1), activeColor);
			if (this.rounded) {
				graphics.fill(start + (on ? 1 : 2), y + h - 2, end - (on ? 2 : 1), y + h - 1, activeColor);
			}
			Component off = PatinaLang.text("patina.value.off"), enabled = PatinaLang.text("patina.value.on");
			float textY = y + (h - 8 * this.textScale) / 2;
			drawLabel(graphics, this.font, off, x + (half - this.font.width(off) * this.textScale) / 2, textY, this.textScale, on ? HINT : 0xFFFFFFFF);
			drawLabel(graphics, this.font, enabled, x + half + (w - half - this.font.width(enabled) * this.textScale) / 2, textY, this.textScale, on ? 0xFFFFFFFF : HINT);
		}
	}

	private static void drawIcon(GuiGraphicsExtractor graphics, int mod, int x, int y, int color) {
		if (mod == 0) {
			for (int i = 0; i < 3; i++) graphics.fill(x + 3 + i * 7, y + 16 - i * 6, x + 7 + i * 7, y + 23, color);
		} else if (mod == 1) {
			graphics.outline(x + 8, y, 8, 8, color);
			for (int i = 0; i < 3; i++) graphics.outline(x + i * 8, y + 9, 8, 8, color);
			graphics.fill(x + 1, y + 20, x + 23, y + 22, color);
		} else if (mod == 2) {
			graphics.outline(x + 7, y + 7, 10, 10, color);
			graphics.fill(x + 11, y, x + 13, y + 9, color);
			graphics.fill(x + 11, y + 15, x + 13, y + 24, color);
			graphics.fill(x, y + 11, x + 9, y + 13, color);
			graphics.fill(x + 15, y + 11, x + 24, y + 13, color);
		} else if (mod == 3) {
			graphics.fill(x + 5, y + 3, x + 19, y + 7, color);
			graphics.fill(x + 1, y + 5, x + 23, y + 11, color);
			graphics.fill(x + 6, y + 11, x + 18, y + 22, color);
			graphics.fill(x + 10, y + 2, x + 14, y + 6, PANEL);
		} else if (mod == 4) {
			graphics.fill(x + 7, y + 7, x + 17, y + 17, color);
			for (int i = 0; i < 3; i++) {
				int offset = 2 + i * 9;
				graphics.fill(x + offset, y + 2, x + offset + 2, y + 5, color);
				graphics.fill(x + offset, y + 19, x + offset + 2, y + 22, color);
			}
			graphics.fill(x + 1, y + 11, x + 5, y + 13, color);
			graphics.fill(x + 19, y + 11, x + 23, y + 13, color);
		} else {
			graphics.outline(x + 3, y + 3, 18, 18, color);
			String[] glyphs = {"A", "S", "S", "Z", "P", "P", "H", "E", "W", "D", "F"};
			var font = net.minecraft.client.Minecraft.getInstance().font;
			String glyph = glyphs[Math.clamp(mod - 5, 0, glyphs.length - 1)];
			graphics.text(font, glyph, x + (24 - font.width(glyph)) / 2, y + 8, color);

		}
	}
}
