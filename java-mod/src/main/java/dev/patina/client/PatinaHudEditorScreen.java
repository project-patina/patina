package dev.patina.client;

import com.mojang.blaze3d.platform.cursor.CursorTypes;
import java.util.List;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.components.Button;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.client.input.MouseButtonEvent;
import net.minecraft.client.renderer.RenderPipelines;
import net.minecraft.network.chat.CommonComponents;
import net.minecraft.network.chat.Component;
import net.minecraft.resources.Identifier;

/**
 * The HUD editor: the game at its real size, over the world when there is one and a stand-in
 * of it otherwise, with the mods drawn by the code that draws them in game, so where a mod is
 * let go is exactly where it shows. Drag a mod to move it; its corner or the wheel resizes it.
 */
public final class PatinaHudEditorScreen extends Screen {
	private static final int ACCENT = 0xFF000000 | PatinaClient.ACCENT;
	private static final int GUIDE = 0x80000000 | PatinaClient.ACCENT;
	private static final int HOVER_EDGE = 0x80FFFFFF;
	private static final int HINT = 0xFF9FB0AC;
	private static final int BUTTON_WIDTH = 100;
	/** Half the side of the resize handle. */
	private static final int HANDLE = 2;
	private static final float SCALE_STEP = 0.1f;

	private static final int[] FAR_HILLS = {62, 58, 58, 54, 54, 58, 62, 62, 66, 62, 58, 58, 62, 66, 66, 62};
	private static final int[] NEAR_HILLS = {74, 74, 70, 70, 74, 78, 78, 74, 74, 70, 70, 74, 74, 78, 74, 74};

	private final Screen parent;
	private final PatinaConfig config = PatinaConfig.get();
	private PatinaConfig.Placed selected;
	private PatinaConfig.Placed dragging;
	private boolean resizing;
	private float grabX;
	private float grabY;

	public PatinaHudEditorScreen(Screen parent) {
		super(Component.literal("Patina HUD"));
		this.parent = parent;
	}

	@Override
	protected void init() {
		int y = this.height / 2 - 10;
		this.addRenderableWidget(Button.builder(PatinaLang.text("patina.editor.reset"), press -> {
			PatinaConfig.Mods defaults = new PatinaConfig.Mods();
			List<PatinaConfig.Placed> mods = this.config.mods.all();
			for (int i = 0; i < mods.size(); i++) {
				mods.get(i).reset(defaults.all().get(i));
			}
			this.config.save();
		}).bounds(this.width / 2 - BUTTON_WIDTH - 2, y, BUTTON_WIDTH, 20).build());
		this.addRenderableWidget(Button.builder(CommonComponents.GUI_DONE, press -> this.onClose())
			.bounds(this.width / 2 + 2, y, BUTTON_WIDTH, 20).build());
	}

	@Override
	public void onClose() {
		this.config.save();
		this.minecraft.gui.setScreen(this.parent);
	}

	@Override
	public boolean isPauseScreen() {
		return false;
	}

	private PatinaHud.View view() {
		PatinaHud.View live = PatinaHud.liveView();
		return live != null ? live : PatinaHud.sampleView();
	}

	private PatinaHud.Box box(PatinaConfig.Placed mod) {
		return PatinaHud.place(this.font, this.width, this.height, this.view(), this.config.mods, mod);
	}

	/** The topmost shown mod under the pointer. */
	private PatinaConfig.Placed modAt(double mouseX, double mouseY) {
		List<PatinaConfig.Placed> mods = this.config.mods.all();
		for (int i = mods.size() - 1; i >= 0; i--) {
			PatinaConfig.Placed mod = mods.get(i);
			if (mod.enabled && this.box(mod).contains(mouseX, mouseY)) {
				return mod;
			}
		}
		return null;
	}

	private boolean onHandle(double mouseX, double mouseY) {
		if (this.selected == null || !this.selected.enabled) {
			return false;
		}
		PatinaHud.Box box = this.box(this.selected);
		return Math.abs(mouseX - (box.x() + box.width())) <= HANDLE + 1 && Math.abs(mouseY - (box.y() + box.height())) <= HANDLE + 1;
	}

	@Override
	public boolean mouseClicked(MouseButtonEvent event, boolean doubleClick) {
		if (event.button() == 0) {
			if (this.onHandle(event.x(), event.y())) {
				this.dragging = this.selected;
				this.resizing = true;
				return true;
			}
			PatinaConfig.Placed mod = this.modAt(event.x(), event.y());
			if (mod != null) {
				PatinaHud.Box box = this.box(mod);
				this.selected = mod;
				this.dragging = mod;
				this.resizing = false;
				this.grabX = (float) event.x() - box.x();
				this.grabY = (float) event.y() - box.y();
				return true;
			}
			this.selected = null;
		}
		return super.mouseClicked(event, doubleClick);
	}

	@Override
	public boolean mouseDragged(MouseButtonEvent event, double dx, double dy) {
		if (this.dragging == null) {
			return super.mouseDragged(event, dx, dy);
		}
		PatinaConfig.Placed mod = this.dragging;
		PatinaHud.Box box = this.box(mod);
		int[] size = PatinaHud.size(this.font, this.view(), this.config.mods, mod);
		float x = (float) event.x();
		float y = (float) event.y();
		if (this.resizing) {
			this.resize(mod, box, Math.max((x - box.x()) / size[0], (y - box.y()) / size[1]));
		} else {
			float w = size[0] * mod.scale;
			float h = size[1] * mod.scale;
			float left = Math.clamp(x - this.grabX, 0f, Math.max(0f, this.width - w));
			float top = Math.clamp(y - this.grabY, 0f, Math.max(0f, this.height - h));
			PatinaHud.moveTo(mod, left, top, w, h, this.width, this.height);
		}
		return true;
	}

	/** A new size with the top left corner where it was, moved in at a screen edge. */
	private void resize(PatinaConfig.Placed mod, PatinaHud.Box box, float scale) {
		int[] size = PatinaHud.size(this.font, this.view(), this.config.mods, mod);
		mod.scale = Math.round(Math.clamp(scale, PatinaConfig.MIN_SCALE, PatinaConfig.MAX_SCALE) * 20f) / 20f;
		float w = size[0] * mod.scale;
		float h = size[1] * mod.scale;
		float x = Math.min(box.x(), Math.max(0f, this.width - w));
		float y = Math.min(box.y(), Math.max(0f, this.height - h));
		PatinaHud.moveTo(mod, x, y, w, h, this.width, this.height);
	}

	@Override
	public boolean mouseReleased(MouseButtonEvent event) {
		if (this.dragging != null) {
			this.dragging = null;
			this.config.save();
			return true;
		}
		return super.mouseReleased(event);
	}

	@Override
	public boolean mouseScrolled(double mouseX, double mouseY, double scrollX, double scrollY) {
		PatinaConfig.Placed mod = this.modAt(mouseX, mouseY);
		if (mod != null && scrollY != 0) {
			this.selected = mod;
			this.resize(mod, this.box(mod), mod.scale + (float) Math.signum(scrollY) * SCALE_STEP);
			this.config.save();
			return true;
		}
		return super.mouseScrolled(mouseX, mouseY, scrollX, scrollY);
	}

	/** Over a world the world itself shows; without one, a stand-in of the game. */
	@Override
	public void extractBackground(GuiGraphicsExtractor graphics, int mouseX, int mouseY, float a) {
		if (this.minecraft.level == null) {
			drawScene(graphics, this.width, this.height);
		}
	}

	@Override
	public void extractRenderState(GuiGraphicsExtractor graphics, int mouseX, int mouseY, float a) {
		super.extractRenderState(graphics, mouseX, mouseY, a);
		String hint = PatinaLang.get("patina.editor.hint");
		graphics.text(this.font, hint, (this.width - this.font.width(hint)) / 2, this.height / 2 + 14, HINT);
		PatinaHud.View view = this.view();
		PatinaConfig.Mods mods = this.config.mods;
		for (PatinaConfig.Placed mod : mods.all()) {
			if (mod.enabled) {
				PatinaHud.draw(graphics, this.font, view, mods, mod, this.box(mod), 1f);
			}
		}
		if (this.dragging != null && !this.resizing) {
			if (this.dragging.horizontal() == 0 && this.dragging.x == 0f) {
				graphics.fill(this.width / 2, 0, this.width / 2 + 1, this.height, GUIDE);
			}
			if (this.dragging.vertical() == 0 && this.dragging.y == 0f) {
				graphics.fill(0, this.height / 2, this.width, this.height / 2 + 1, GUIDE);
			}
		}
		PatinaConfig.Placed hovered = this.dragging != null ? this.dragging : this.modAt(mouseX, mouseY);
		if (hovered != null && hovered != this.selected) {
			this.frame(graphics, this.box(hovered), HOVER_EDGE, false);
		}
		if (this.selected != null && this.selected.enabled) {
			PatinaHud.Box box = this.box(this.selected);
			this.frame(graphics, box, ACCENT, true);
			String size = Math.round(this.selected.scale * 100f) + "%";
			int y = box.y() >= 11 ? (int) Math.floor(box.y()) - 11 : (int) Math.ceil(box.y() + box.height()) + 3;
			graphics.text(this.font, size, (int) Math.floor(box.x()), y, HINT);
		}
		if (this.resizing && this.dragging != null || this.onHandle(mouseX, mouseY)) {
			graphics.requestCursor(CursorTypes.CROSSHAIR);
		} else if (hovered != null) {
			graphics.requestCursor(CursorTypes.RESIZE_ALL);
		}
	}

	/** A mod's outline, with the resize handle on the selected one. */
	private void frame(GuiGraphicsExtractor graphics, PatinaHud.Box box, int color, boolean handle) {
		int left = (int) Math.floor(box.x()) - 1;
		int top = (int) Math.floor(box.y()) - 1;
		int right = (int) Math.ceil(box.x() + box.width()) + 1;
		int bottom = (int) Math.ceil(box.y() + box.height()) + 1;
		graphics.outline(left, top, right - left, bottom - top, color);
		if (handle) {
			graphics.fill(right - 1 - HANDLE, bottom - 1 - HANDLE, right - 1 + HANDLE, bottom - 1 + HANDLE, color);
		}
	}

	private static Identifier sprite(String path) {
		return Identifier.withDefaultNamespace(path);
	}

	/**
	 * A stand-in for the game at its real size: sky, hills and ground, and the HUD where the
	 * game puts it (crosshair, hotbar, hearts, food and the experience bar).
	 */
	private static void drawScene(GuiGraphicsExtractor graphics, int width, int height) {
		graphics.fillGradient(0, 0, width, height * 3 / 4, 0xFF7FB2DA, 0xFFC4E0EC);
		hills(graphics, width, height, FAR_HILLS, 0xFF7DA689);
		hills(graphics, width, height, NEAR_HILLS, 0xFF4F8A45);
		graphics.fill(0, height * 84 / 100, width, height, 0xFF6B4A32);
		graphics.fill(0, height * 84 / 100, width, height * 84 / 100 + 3, 0xFF5E9A50);

		graphics.blitSprite(RenderPipelines.GUI_TEXTURED, sprite("hud/crosshair"), (width - 15) / 2, (height - 15) / 2, 15, 15);
		int bar = width / 2 - 91;
		graphics.blitSprite(RenderPipelines.GUI_TEXTURED, sprite("hud/hotbar"), bar, height - 22, 182, 22);
		graphics.blitSprite(RenderPipelines.GUI_TEXTURED, sprite("hud/hotbar_selection"), bar - 1, height - 23, 24, 23);
		graphics.blitSprite(RenderPipelines.GUI_TEXTURED, sprite("hud/experience_bar_background"), bar, height - 29, 182, 5);
		for (int i = 0; i < 10; i++) {
			int heart = bar + i * 8;
			graphics.blitSprite(RenderPipelines.GUI_TEXTURED, sprite("hud/heart/container"), heart, height - 39, 9, 9);
			graphics.blitSprite(RenderPipelines.GUI_TEXTURED, sprite("hud/heart/full"), heart, height - 39, 9, 9);
			int food = width / 2 + 91 - i * 8 - 9;
			graphics.blitSprite(RenderPipelines.GUI_TEXTURED, sprite("hud/food_empty"), food, height - 39, 9, 9);
			graphics.blitSprite(RenderPipelines.GUI_TEXTURED, sprite("hud/food_full"), food, height - 39, 9, 9);
		}
	}

	private static void hills(GuiGraphicsExtractor graphics, int width, int height, int[] tops, int color) {
		int columns = tops.length;
		for (int i = 0; i < columns; i++) {
			int left = width * i / columns;
			int right = width * (i + 1) / columns;
			graphics.fill(left, height * tops[i] / 100, right, height, color);
		}
	}
}
