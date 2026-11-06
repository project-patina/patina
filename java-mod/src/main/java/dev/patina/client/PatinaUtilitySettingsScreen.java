package dev.patina.client;

import java.util.ArrayList;
import java.util.List;
import java.util.function.Consumer;
import java.util.function.Supplier;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.components.AbstractSliderButton;
import net.minecraft.client.gui.components.Button;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.client.input.KeyEvent;
import net.minecraft.network.chat.Component;
import com.mojang.blaze3d.platform.InputConstants;

/** Settings shared by the client utilities; only relevant controls are shown. */
public final class PatinaUtilitySettingsScreen extends Screen {
	private final Screen parent;
	private final int mod;
	private final PatinaConfig config = PatinaConfig.get();
	private final List<Button> popup = new ArrayList<>();
	private final List<String> labels = new ArrayList<>();
	private int x, y, w, h, row;
	private boolean binding;
	private Button keyButton;
	private PatinaConfig.UtilityMod keys;

	PatinaUtilitySettingsScreen(Screen parent, int mod) {
		super(PatinaLang.text(PatinaSettingsScreen.NAMES[mod])); this.parent = parent; this.mod = mod;
	}

	@Override protected void init() {
		this.w = Math.min(this.width - 16, 386); this.h = Math.min(this.height - 16, 226);
		this.x = (this.width - this.w) / 2; this.y = (this.height - this.h) / 2;
		this.row = 0; this.labels.clear(); this.popup.clear(); this.binding = false;
		var mods = this.config.mods;
		var toggle = mods.catalogue().get(this.mod);
		Button back = button(PatinaLang.text("patina.settings.back"), this.x + this.w - 62, this.y + 6, 50, press -> this.onClose());
		this.addRenderableWidget(back);
		PatinaSettingsScreen.Switch enabled = new PatinaSettingsScreen.Switch(this.font, PatinaLang.text("patina.settings.enabled"), 56, 14,
			() -> toggle.enabled, () -> { toggle.enabled = !toggle.enabled; this.config.save(); }, 0.82f, false);
		enabled.setX(this.x + this.w - 126); enabled.setY(this.y + 6); this.addRenderableWidget(enabled);
		if (this.mod == 8 || this.mod == 10) {
			this.keys = this.mod == 8 ? mods.zoom : mods.perspective;
			choice("activation", () -> this.keys.activation, List.of("hold", "toggle"), value -> this.keys.activation = value);
			this.keyButton = control("key", button(this.keyLabel(), 0, 0, 100, press -> { this.binding = true; press.setMessage(PatinaLang.text("patina.option.press_key")); }));
			slider(this.mod == 8 ? "zoom_factor" : "camera_distance", this.keys.factor, this.mod == 8 ? 1.5f : 1.5f, this.mod == 8 ? 20f : 12f, value -> this.keys.factor = value);
			if (this.mod == 8) {
				check("smooth", () -> this.keys.smooth, value -> this.keys.smooth = value);
				slider("sensitivity", this.keys.sensitivity, 0.05f, 1f, value -> this.keys.sensitivity = value);
			}
		} else if (this.mod == 6) {
			check("require_shift", () -> mods.shulker_tooltip.require_shift, value -> mods.shulker_tooltip.require_shift = value);
		} else if (this.mod == 7) {
			choice("unit", () -> mods.speedometer.unit, List.of("m/s", "km/h"), value -> mods.speedometer.unit = value);
			check("background", () -> mods.speedometer.background, value -> mods.speedometer.background = value);
		} else if (this.mod == 9) {
			check("background", () -> mods.ping.background, value -> mods.ping.background = value);
		} else if (this.mod == 12) {
			choice("columns", () -> "" + mods.status_effects.columns, List.of("1", "2", "3", "4"), value -> mods.status_effects.columns = Integer.parseInt(value));
			check("duration", () -> mods.status_effects.show_duration, value -> mods.status_effects.show_duration = value);
			check("amplifier", () -> mods.status_effects.show_amplifier, value -> mods.status_effects.show_amplifier = value);
		} else if (this.mod == 13) {
			choice("weather", () -> mods.weather.weather, List.of("clear", "rain", "thunder"), value -> mods.weather.weather = value);
			slider("intensity", mods.weather.strength, 0.05f, 1f, value -> mods.weather.strength = value);
		}
		var hud = mods.hud(this.mod);
		if (hud != null) this.addRenderableWidget(button(PatinaLang.text("patina.settings.preview"), this.x + this.w - 116, this.y + this.h - 24, 104,
			press -> this.minecraft.gui.setScreen(new PatinaHudEditorScreen(this, hud))));
		this.addRenderableWidget(new PatinaResetButton(this.x + 12, this.y + this.h - 24, 96, PatinaLang.text("patina.settings.reset"), 0.82f,
			press -> { this.config.mods.resetUtility(this.mod); this.config.save(); this.rebuildWidgets(); }));
	}

	private Component keyLabel() {
		return PatinaKeyBindings.nativeCode(this.keys.key) < 0 ? PatinaLang.text("patina.value.unbound") : InputConstants.Type.KEYBOARD.getOrCreate(PatinaKeyBindings.nativeCode(this.keys.key)).getDisplayName();
	}
	private Button button(Component label, int x, int y, int width, Button.OnPress press) {
		return new Button(x, y, width, 14, label, press, message -> message.get()) {
			@Override protected void extractContents(GuiGraphicsExtractor g, int mx, int my, float a) {
				this.extractDefaultSprite(g);
				PatinaSettingsScreen.drawLabel(g, font, this.getMessage(), this.getX() + (this.width - font.width(this.getMessage()) * 0.82f) / 2,
					this.getY() + 4, 0.82f, 0xFFFFFFFF);
			}
		};
	}
	private <T extends net.minecraft.client.gui.components.AbstractWidget> T control(String label, T widget) {
		widget.setX(this.x + this.w - 112); widget.setY(this.y + 50 + this.row++ * 24);
		this.labels.add(label); return this.addRenderableWidget(widget);
	}
	private void check(String label, Supplier<Boolean> read, Consumer<Boolean> write) {
		control(label, new PatinaSettingsScreen.Switch(this.font, PatinaLang.text("patina.option." + label), 100, 14, () -> read.get(),
			() -> { write.accept(!read.get()); this.config.save(); }, 0.82f, false));
	}
	private void choice(String label, Supplier<String> read, List<String> values, Consumer<String> write) {
		Button control = control(label, button(valueLabel(read.get()), 0, 0, 100, press -> {
			boolean open = this.popup.stream().anyMatch(b -> b.visible); this.popup.forEach(b -> b.visible = false);
			if (open) return;
			for (int i = 0; i < values.size(); i++) {
				String value = values.get(i);
				Button option = button(valueLabel(value), press.getX(), Math.max(this.y + 24, press.getY() - values.size() * 14) + i * 14, 100, selected -> {
					write.accept(value); this.config.save(); this.rebuildWidgets();
				});
				this.popup.add(this.addRenderableWidget(option));
			}
		}));
	}
	private Component valueLabel(String value) {
		String key = "patina.value." + value;
		String text = PatinaLang.get(key);
		return Component.literal(text.equals(key) ? value : text);
	}
	private void slider(String label, float current, float min, float max, Consumer<Float> write) {
		control(label, new AbstractSliderButton(0, 0, 100, 14, Component.empty(), (current - min) / (max - min)) {
			{ this.updateMessage(); }
			@Override protected void updateMessage() { this.setMessage(Component.literal(String.format(java.util.Locale.ROOT, "%.2f", min + this.value * (max - min)))); }
			@Override protected void applyValue() { write.accept((float) (min + this.value * (max - min))); config.save(); }
		});
	}
	@Override public boolean keyPressed(KeyEvent event) {
		if (this.binding) {
			// Delete unbinds; a key the shared table cannot name keeps the old binding.
			int key = event.key() == InputConstants.KEY_DELETE ? -1 : PatinaKeyBindings.sharedCode(event.key());
			if (event.key() != InputConstants.KEY_ESCAPE && (key >= 0 || event.key() == InputConstants.KEY_DELETE)) { this.keys.key = key; this.config.save(); }
			this.binding = false; this.keyButton.setMessage(this.keyLabel()); return true;
		}
		if (event.key() == InputConstants.KEY_ESCAPE && this.popup.stream().anyMatch(b -> b.visible)) { this.popup.forEach(b -> b.visible = false); return true; }
		return super.keyPressed(event);
	}
	@Override public void onClose() { this.minecraft.gui.setScreen(this.parent); }
	@Override public void extractRenderState(GuiGraphicsExtractor g, int mx, int my, float a) {
		g.fill(this.x, this.y, this.x + this.w, this.y + this.h, PatinaSettingsScreen.PANEL);
		g.outline(this.x, this.y, this.w, this.h, PatinaSettingsScreen.LINE);
		g.fill(this.x, this.y, this.x + this.w, this.y + 2, PatinaSettingsScreen.ACCENT);
		PatinaSettingsScreen.drawLabel(g, this.font, this.title, this.x + 12, this.y + 10, 0.9f, PatinaSettingsScreen.HEADING);
		String description = PatinaLang.get(PatinaSettingsScreen.NAMES[this.mod] + ".description");
		var lines = this.font.split(Component.literal(description), (int)((this.w - 24) / 0.72f));
		for (int i = 0; i < lines.size() && i < 2; i++) {
			var line = lines.get(i);
			g.pose().pushMatrix(); g.pose().translate(this.x + 12, this.y + 30 + i * 8); g.pose().scale(0.72f); g.text(this.font, line, 0, 0, PatinaSettingsScreen.HINT); g.pose().popMatrix();
		}
		for (int i = 0; i < this.labels.size(); i++) PatinaSettingsScreen.drawLabel(g, this.font, PatinaLang.text("patina.option." + this.labels.get(i)), this.x + 12, this.y + 54 + i * 24, 0.82f, 0xFFE8F0EE);
		super.extractRenderState(g, mx, my, a);
	}
}
