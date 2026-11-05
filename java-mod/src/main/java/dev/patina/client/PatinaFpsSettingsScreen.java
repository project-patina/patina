package dev.patina.client;

import java.util.List;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.components.CycleButton;
import net.minecraft.client.gui.components.Button;
import net.minecraft.client.gui.screens.Screen;

/** The FPS options appear only after opening their module tile. Escape returns to the tiles. */
public final class PatinaFpsSettingsScreen extends Screen {
	private final PatinaSettingsScreen parent;
	private int x, y, panelWidth;

	public PatinaFpsSettingsScreen(PatinaSettingsScreen parent) {
		super(PatinaLang.text("patina.mod.fps"));
		this.parent = parent;
	}

	@Override
	protected void init() {
		this.parent.resize(this.width, this.height);
		this.panelWidth = Math.min(280, this.width - 48);
		this.x = (this.width - this.panelWidth) / 2;
		this.y = (this.height - 130) / 2;
		PatinaConfig config = PatinaConfig.get();
		this.addRenderableWidget(CycleButton.<String>builder(value -> PatinaLang.text("patina.option.accuracy." + value), config.mods.fps.accuracy)
			.withValues(List.of("whole", "precise"))
			.create(this.x + 12, this.y + 36, this.panelWidth - 24, 20, PatinaLang.text("patina.option.accuracy"), (button, value) -> {
				config.mods.fps.accuracy = value;
				config.save();
			}));
		this.addRenderableWidget(CycleButton.<String>builder(value -> PatinaLang.text("patina.option.style." + value), config.mods.fps.style)
			.withValues(List.of("text", "panel"))
			.create(this.x + 12, this.y + 62, this.panelWidth - 24, 20, PatinaLang.text("patina.option.style"), (button, value) -> {
				config.mods.fps.style = value;
				config.save();
			}));
		this.addRenderableWidget(Button.builder(PatinaLang.text("patina.settings.back"), press -> this.onClose())
			.bounds(this.x + 12, this.y + 98, this.panelWidth - 24, 20).build());
	}

	@Override
	public void onClose() {
		this.minecraft.gui.setScreen(this.parent);
	}

	@Override
	public void extractRenderState(GuiGraphicsExtractor graphics, int mouseX, int mouseY, float a) {
		this.parent.extractRenderState(graphics, -1, -1, a);
		graphics.fill(0, 0, this.width, this.height, 0xB0000000);
		graphics.fill(this.x, this.y, this.x + this.panelWidth, this.y + 130, 0xFF0C1012);
		graphics.outline(this.x, this.y, this.panelWidth, 130, PatinaSettingsScreen.LINE);
		graphics.fill(this.x, this.y, this.x + this.panelWidth, this.y + 2, PatinaSettingsScreen.ACCENT);
		graphics.text(this.font, this.title, this.x + 12, this.y + 14, PatinaSettingsScreen.HEADING);
		super.extractRenderState(graphics, mouseX, mouseY, a);
	}
}
