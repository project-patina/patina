package dev.patina.client;

import com.mojang.blaze3d.platform.InputConstants;
import java.util.ArrayList;
import java.util.List;
import java.util.HashSet;
import java.util.Set;
import java.util.function.BooleanSupplier;
import java.util.function.Consumer;
import java.util.function.Function;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.components.Button;
import net.minecraft.client.gui.components.AbstractSliderButton;
import net.minecraft.client.gui.components.AbstractWidget;
import net.minecraft.client.input.KeyEvent;
import net.minecraft.client.input.MouseButtonEvent;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.network.chat.Component;

/** Categorised settings for a single HUD module. */
public final class PatinaModuleSettingsScreen extends Screen {
	private static final String[] NAMES = {"patina.mod.fps", "patina.mod.keystrokes", "patina.mod.coordinates", "patina.mod.armor"};
	private static final int CONTROL_HEIGHT = 14;
	private static final int ROW_STEP = 22;
	private static final int GROUP_GAP = 6;
	private static final int THUMB_WIDTH = 96;
	private static final int THUMB_HEIGHT = 70;
	private static final float TEXT_SCALE = 0.82f;
	private record Row(AbstractWidget widget, String label) { }
	private record Category(String key, Button header, List<Row> rows) { }
	private record Choice(Button control, List<Button> values) { }
	private final PatinaSettingsScreen parent;
	private final int mod;
	private final PatinaConfig config = PatinaConfig.get();
	private final List<Category> categories = new ArrayList<>();
	private final List<Choice> choices = new ArrayList<>();
	private final Set<String> collapsed = new HashSet<>();
	private Choice openChoice;
	private int x, y, panelWidth, panelHeight, listWidth, viewTop, viewBottom, contentHeight, scroll;

	public PatinaModuleSettingsScreen(PatinaSettingsScreen parent, int mod) {
		super(PatinaLang.text(NAMES[mod]));
		this.parent = parent;
		this.mod = mod;
		if (mod == 3) this.collapsed.addAll(List.of("layout", "appearance", "warnings"));
	}

	@Override
	protected void init() {
		this.panelWidth = Math.min(this.width - 16, 386);
		this.panelHeight = PatinaSettingsScreen.panelHeight(this.height, 211);
		this.x = (this.width - this.panelWidth) / 2;
		this.y = (this.height - this.panelHeight) / 2;
		this.viewTop = this.y + 32;
		this.viewBottom = this.y + this.panelHeight - 33;
		this.listWidth = this.panelWidth - 28 - (this.hasThumbnail() ? THUMB_WIDTH + 14 : 0);
		this.closeChoice();
		this.choices.clear();
		this.categories.clear();
		this.contentHeight = 0;
		if (this.mod == 0) {
			category("appearance");
			choice("accuracy", "accuracy.", this.config.mods.fps.accuracy, List.of("whole", "precise"), value -> this.config.mods.fps.accuracy = value);
			choice("style", "style.", this.config.mods.fps.style, List.of("text", "panel"), value -> this.config.mods.fps.style = value);
		} else if (this.mod == 1) {
			PatinaConfig.Keystrokes keys = this.config.mods.keystrokes;
			category("keys");
			toggle("mouse", () -> keys.show_mouse, value -> keys.show_mouse = value);
			toggle("space", () -> keys.show_space, value -> keys.show_space = value);
			toggle("shift", () -> keys.show_shift, value -> keys.show_shift = value);
			category("appearance");
			toggle("labels", () -> keys.show_labels, value -> keys.show_labels = value);
		} else if (this.mod == 2) {
			PatinaConfig.Coordinates coordinates = this.config.mods.coordinates;
			category("information");
			toggle("direction", () -> coordinates.show_direction, value -> coordinates.show_direction = value);
			toggle("angle", () -> coordinates.show_angle, value -> coordinates.show_angle = value);
			toggle("biome", () -> coordinates.show_biome, value -> coordinates.show_biome = value);
			category("appearance");
			toggle("background", () -> coordinates.background, value -> coordinates.background = value);
		} else {
			PatinaConfig.Armor armor = this.config.mods.armor;
			category("layout");
			toggle("vertical", () -> armor.vertical, value -> armor.vertical = value);
			toggle("hide_empty", () -> armor.hide_empty, value -> armor.hide_empty = value);
			choice("durability", "durability.", armor.display, List.of("bar", "remaining"), value -> armor.display = value);
			toggle("dynamic_color", () -> armor.dynamic_color, value -> armor.dynamic_color = value);
			category("appearance");
			row("theme", new CompactButton(PatinaLang.text("patina.option.theme.patina"), 52, press -> {}, false)).active = false;
			category("warnings");
			toggle("warning", () -> armor.warning, value -> armor.warning = value);
			row("threshold", new WarningSlider(armor));
		}
		Button preview = new CompactButton(PatinaLang.text("patina.settings.preview"), 64, press ->
			this.minecraft.gui.setScreen(new PatinaHudEditorScreen(this, this.config.mods.all().get(this.mod))), false);
		preview.setX(this.x + this.panelWidth - 132);
		preview.setY(this.y + 6);
		this.addRenderableWidget(preview);
		Button back = new CompactButton(PatinaLang.text("patina.settings.back"), 52, press -> this.onClose(), false);
		back.setX(this.x + this.panelWidth - 62);
		back.setY(this.y + 6);
		this.addRenderableWidget(back);
		this.addRenderableWidget(new PatinaResetButton(this.x + 12, this.y + this.panelHeight - 23, 96,
			PatinaLang.text("patina.settings.reset"), TEXT_SCALE, press -> {
				this.config.mods.resetSettings(this.mod);
				this.config.save();
				this.rebuildWidgets();
			}));
		for (Choice choice : this.choices) {
			for (Button value : choice.values) this.addRenderableWidget(value);
		}
		this.positionRows();
	}

	private void category(String key) {
		Button header = new CompactButton(this.categoryLabel(key), this.listWidth, press -> {
			this.closeChoice();
			if (!this.collapsed.add(key)) this.collapsed.remove(key);
			this.positionRows();
		}, false) {
			@Override
			protected void extractContents(GuiGraphicsExtractor graphics, int mouseX, int mouseY, float a) {
				super.extractContents(graphics, mouseX, mouseY, a);
				drawChevron(graphics, this.getX() + 10, this.getY() + 7, collapsed.contains(key) ? 0 : 1);
			}
		};
		this.categories.add(new Category(key, this.addRenderableWidget(header), new ArrayList<>()));
	}

	private Component categoryLabel(String key) {
		return PatinaLang.text("patina.category." + key);
	}

	private AbstractWidget row(String key, AbstractWidget widget) {
		this.categories.getLast().rows.add(new Row(this.addRenderableWidget(widget), PatinaLang.get("patina.option." + key)));
		return widget;
	}

	private void toggle(String key, BooleanSupplier read, Consumer<Boolean> write) {
		row(key, new PatinaSettingsScreen.Switch(this.font, PatinaLang.text("patina.option." + key), 56, CONTROL_HEIGHT, read, () -> {
			write.accept(!read.getAsBoolean());
			this.config.save();
		}, TEXT_SCALE, false));
	}

	private void choice(String key, String prefix, String current, List<String> values, Consumer<String> write) {
		Function<String, Component> label = value -> PatinaLang.text("patina.option." + prefix + value);
		int width = Math.min(91, this.listWidth * 3 / 5);
		Function<String, Component> buttonLabel = value ->
			Component.literal(this.font.plainSubstrByWidth(label.apply(value).getString(), (int) ((width - 20) / TEXT_SCALE)));
		Button control = new CompactButton(buttonLabel.apply(current), width, press -> {
			Choice choice = this.choices.stream().filter(item -> item.control == press).findFirst().orElseThrow();
			boolean alreadyOpen = this.openChoice == choice;
			this.closeChoice();
			if (!alreadyOpen) {
				this.openChoice = choice;
				int top = Math.max(2, press.getY() - choice.values.size() * CONTROL_HEIGHT);
				for (int i = 0; i < choice.values.size(); i++) {
					Button option = choice.values.get(i);
					option.setX(press.getX() + press.getWidth() - option.getWidth());
					option.setY(top + i * CONTROL_HEIGHT);
					option.visible = true;
				}
			}
		}, false) {
			@Override
			protected void extractContents(GuiGraphicsExtractor graphics, int mouseX, int mouseY, float a) {
				super.extractContents(graphics, mouseX, mouseY, a);
				drawChevron(graphics, this.getX() + this.getWidth() - 8, this.getY() + 7,
					openChoice != null && openChoice.control == this ? 1 : -1);
			}
		};
		row(key, control);
		List<Button> options = new ArrayList<>();
		int optionWidth = Math.max(width, values.stream().mapToInt(value -> (int) Math.ceil(this.font.width(label.apply(value)) * TEXT_SCALE) + 10).max().orElse(width));
		for (String value : values) {
			Button option = new CompactButton(label.apply(value), optionWidth, press -> {
				write.accept(value);
				this.config.save();
				control.setMessage(buttonLabel.apply(value));
				this.closeChoice();
				this.setFocused(control);
			}, true);
			option.visible = false;
			options.add(option);
		}
		this.choices.add(new Choice(control, options));
	}

	private void closeChoice() {
		if (this.openChoice == null) return;
		for (Button option : this.openChoice.values) option.visible = false;
		this.openChoice = null;
	}

	private void positionRows() {
		this.contentHeight = Math.max(0, this.categories.size() - 1) * GROUP_GAP;
		for (Category category : this.categories) {
			this.contentHeight += ROW_STEP;
			if (!this.collapsed.contains(category.key)) this.contentHeight += category.rows.size() * ROW_STEP;
		}
		this.scroll = Math.clamp(this.scroll, 0, Math.max(0, this.contentHeight - (this.viewBottom - this.viewTop)));
		int rowY = this.viewTop - this.scroll;
		for (Category category : this.categories) {
			this.position(category.header, rowY, true);
			rowY += ROW_STEP;
			boolean expanded = !this.collapsed.contains(category.key);
			for (Row row : category.rows) {
				this.position(row.widget, rowY, expanded);
				if (expanded) rowY += ROW_STEP;
			}
			rowY += GROUP_GAP;
		}
	}

	private void position(AbstractWidget widget, int rowY, boolean shown) {
		widget.setX(this.x + 12 + this.listWidth - widget.getWidth());
		widget.setY(rowY);
		widget.visible = shown && rowY >= this.viewTop && rowY + CONTROL_HEIGHT <= this.viewBottom;
	}

	@Override
	public boolean mouseClicked(MouseButtonEvent event, boolean doubleClick) {
		if (this.openChoice != null) {
			for (Button option : this.openChoice.values) {
				if (option.mouseClicked(event, doubleClick)) return true;
			}
			if (!this.openChoice.control.isMouseOver(event.x(), event.y())) {
				this.closeChoice();
				return true;
			}
		}
		return super.mouseClicked(event, doubleClick);
	}

	@Override
	public boolean keyPressed(KeyEvent event) {
		if (this.openChoice != null && event.key() == InputConstants.KEY_ESCAPE) {
			this.closeChoice();
			return true;
		}
		return super.keyPressed(event);
	}

	private static void drawChevron(GuiGraphicsExtractor graphics, int x, int y, int direction) {
		for (int d = -2; d <= 2; d++) {
			int offset = 1 - Math.abs(d);
			int px = x + (direction == 0 ? offset : d);
			int py = y + (direction == 0 ? d : direction * offset);
			graphics.fill(px, py, px + 1, py + 1, PatinaSettingsScreen.HEADING);
		}
	}

	private class CompactButton extends Button {
		private final boolean popup;

		CompactButton(Component label, int width, OnPress press, boolean popup) {
			super(0, 0, width, CONTROL_HEIGHT, label, press, DEFAULT_NARRATION);
			this.popup = popup;
		}

		@Override
		protected void extractContents(GuiGraphicsExtractor graphics, int mouseX, int mouseY, float a) {
			int x = this.getX(), y = this.getY(), w = this.getWidth(), h = this.getHeight();
			if (this.popup) {
				boolean highlighted = this.isHoveredOrFocused();
				graphics.fill(x, y, x + w, y + h, highlighted ? 0xFF1F3935 : 0xFF121A1D);
				graphics.outline(x, y, w, h, highlighted ? PatinaSettingsScreen.ACCENT : PatinaSettingsScreen.LINE);
			} else {
				this.extractDefaultSprite(graphics);
			}
			Component label = this.getMessage();
			float textX = x + (w - font.width(label) * TEXT_SCALE) / 2;
			PatinaSettingsScreen.drawLabel(graphics, font, label, textX, y + (h - 8 * TEXT_SCALE) / 2,
				TEXT_SCALE, this.active ? 0xFFFFFFFF : 0xFFA0A0A0);
		}
	}

	private final class WarningSlider extends AbstractSliderButton {
		private final PatinaConfig.Armor armor;

		WarningSlider(PatinaConfig.Armor armor) {
			super(0, 0, 68, CONTROL_HEIGHT, PatinaLang.text("patina.option.threshold"), (armor.warning_threshold - 5) / 45.0);
			this.armor = armor;
			this.updateMessage();
		}

		@Override
		public void onClick(MouseButtonEvent event, boolean doubleClick) {
			this.setFromPointer(event.x());
		}

		@Override
		protected void onDrag(MouseButtonEvent event, double dx, double dy) {
			this.setFromPointer(event.x());
		}

		private void setFromPointer(double mouseX) {
			this.value = Math.clamp((mouseX - this.getX() - 4) / (this.getWidth() - 32), 0.0, 1.0);
			this.applyValue();
			this.updateMessage();
		}

		@Override
		protected void updateMessage() {
			this.setMessage(PatinaLang.text("patina.option.threshold").copy().append(" " + this.armor.warning_threshold + "%"));
		}

		@Override
		protected void applyValue() {
			int threshold = 5 + (int) Math.round(this.value * 45);
			if (threshold == this.armor.warning_threshold) return;
			this.armor.warning_threshold = threshold;
			config.save();
		}

		@Override
		public void extractWidgetRenderState(GuiGraphicsExtractor graphics, int mouseX, int mouseY, float a) {
			int x = this.getX() + 4, y = this.getY() + 7, end = this.getX() + this.getWidth() - 28;
			int knob = x + (int) Math.round((end - x) * this.value);
			graphics.fill(x, y - 1, end, y + 1, 0xFF545E5E);
			graphics.fill(x, y - 1, knob, y + 1, PatinaSettingsScreen.ACCENT);
			graphics.fill(knob - 2, y - 3, knob + 3, y + 3, this.isHoveredOrFocused() ? 0xFFFFFFFF : PatinaSettingsScreen.HEADING);
			PatinaSettingsScreen.drawLabel(graphics, font, Component.literal(this.armor.warning_threshold + "%"), end + 5, y - 4 * TEXT_SCALE, TEXT_SCALE, 0xFFFFFFFF);
		}
	}

	@Override
	public boolean mouseScrolled(double mouseX, double mouseY, double horizontal, double vertical) {
		if (vertical != 0 && mouseX >= this.x && mouseX < this.x + 24 + this.listWidth && mouseY >= this.viewTop && mouseY <= this.viewBottom) {
			this.closeChoice();
			this.scroll -= (int) Math.round(vertical * ROW_STEP);
			this.positionRows();
			return true;
		}
		return super.mouseScrolled(mouseX, mouseY, horizontal, vertical);
	}

	@Override
	public void onClose() { this.minecraft.gui.setScreen(this.parent); }

	@Override
	public void extractRenderState(GuiGraphicsExtractor graphics, int mouseX, int mouseY, float partialTick) {
		graphics.fill(this.x, this.y, this.x + this.panelWidth, this.y + this.panelHeight, 0xF50C1012);
		graphics.outline(this.x, this.y, this.panelWidth, this.panelHeight, PatinaSettingsScreen.LINE);
		graphics.fill(this.x, this.y, this.x + this.panelWidth, this.y + 2, PatinaSettingsScreen.ACCENT);
		PatinaSettingsScreen.drawLabel(graphics, this.font, this.title, this.x + 12, this.y + 10, TEXT_SCALE, PatinaSettingsScreen.HEADING);
		graphics.enableScissor(this.x + 12, this.viewTop, this.x + 12 + this.listWidth, this.viewBottom);
		for (Category category : this.categories) {
			for (Row row : category.rows) {
				if (!row.widget.visible) continue;
				int labelWidth = row.widget.getX() - (this.x + 12) - 5;
				String label = this.font.plainSubstrByWidth(row.label, Math.max(0, (int) (labelWidth / TEXT_SCALE)));
				PatinaSettingsScreen.drawLabel(graphics, this.font, Component.literal(label), this.x + 12, row.widget.getY() + 4, TEXT_SCALE, 0xFFFFFFFF);
			}
		}
		graphics.disableScissor();
		if (this.contentHeight > this.viewBottom - this.viewTop) {
			int track = this.viewBottom - this.viewTop;
			int thumb = Math.max(12, track * track / this.contentHeight);
			int top = this.viewTop + (track - thumb) * this.scroll / (this.contentHeight - track);
			graphics.fill(this.x + this.listWidth + 15, this.viewTop, this.x + this.listWidth + 17, this.viewBottom, 0x30FFFFFF);
			graphics.fill(this.x + this.listWidth + 15, top, this.x + this.listWidth + 17, top + thumb, PatinaSettingsScreen.ACCENT);
		}
		if (this.hasThumbnail()) this.drawThumbnail(graphics);
		super.extractRenderState(graphics, mouseX, mouseY, partialTick);
	}
	private boolean hasThumbnail() {
		return this.panelWidth >= 284 && this.viewBottom - this.viewTop >= THUMB_HEIGHT;
	}

	private void drawThumbnail(GuiGraphicsExtractor graphics) {
		int x = this.x + this.panelWidth - 12 - THUMB_WIDTH;
		int y = this.viewTop;
		PatinaBackground.drawLauncherScene(graphics, x + 1, y + 1, THUMB_WIDTH - 2, THUMB_HEIGHT - 2);
		graphics.outline(x, y, THUMB_WIDTH, THUMB_HEIGHT, PatinaSettingsScreen.LINE);
		PatinaHud.View view = PatinaHud.sampleView();
		PatinaConfig.Placed placed = this.config.mods.all().get(this.mod);
		int[] size = PatinaHud.size(this.font, view, this.config.mods, placed);
		float zoom = Math.min(0.95f, Math.min((THUMB_WIDTH - 12f) / size[0], (THUMB_HEIGHT - 12f) / size[1]));
		graphics.pose().pushMatrix();
		graphics.pose().translate(x + (THUMB_WIDTH - size[0] * zoom) / 2, y + (THUMB_HEIGHT - size[1] * zoom) / 2);
		graphics.pose().scale(zoom / placed.scale);
		PatinaHud.draw(graphics, this.font, view, this.config.mods, placed, new PatinaHud.Box(0, 0, 0, 0), 1f);
		graphics.pose().popMatrix();
	}
}
