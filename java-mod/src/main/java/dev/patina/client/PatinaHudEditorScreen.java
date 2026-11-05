package dev.patina.client;

import com.mojang.blaze3d.platform.cursor.CursorTypes;
import com.mojang.blaze3d.platform.InputConstants;
import java.util.List;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.components.Button;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.client.input.MouseButtonEvent;
import net.minecraft.client.renderer.RenderPipelines;
import net.minecraft.network.chat.Component;
import net.minecraft.resources.Identifier;

/**
 * Edits mods at their in-game size and position over a reference view of Minecraft.
 * Drag to move a mod, or resize it with the wheel or its corner handle.
 */
public final class PatinaHudEditorScreen extends Screen {
	private static final int ACCENT = 0xFF000000 | PatinaClient.ACCENT;
	private static final int GUIDE = 0x80000000 | PatinaClient.ACCENT;
	private static final int HOVER_EDGE = 0x80FFFFFF;
	private static final int HINT = 0xFF9FB0AC;
	/** Half the side of the resize handle. */
	private static final int HANDLE = 2;
	private static final float SCALE_STEP = 0.1f;


	private static final int BACK_WIDTH = 52;
	private static final int BUTTON_GAP = 6;
	private final Screen parent;
	private final PatinaConfig.Placed preview;
	private final PatinaConfig config = PatinaConfig.get();
	private PatinaConfig.Placed selected;
	private PatinaConfig.Placed dragging;
	private boolean resizing;
	private float grabX;
	private float grabY;

	public PatinaHudEditorScreen(Screen parent) {
		this(parent, null);
	}

	public PatinaHudEditorScreen(Screen parent, PatinaConfig.Placed preview) {
		super(Component.literal("Patina HUD"));
		this.parent = parent;
		this.preview = preview;
		this.selected = preview;
	}

	@Override
	protected void init() {
		int buttonX = (this.width - PatinaResetButton.WIDTH - BUTTON_GAP - BACK_WIDTH) / 2;
		this.addRenderableWidget(new PatinaResetButton(buttonX, 20, press -> {
			PatinaConfig.Mods defaults = new PatinaConfig.Mods();
			List<PatinaConfig.Placed> mods = this.config.mods.all();
			for (int i = 0; i < mods.size(); i++) {
				mods.get(i).reset(defaults.all().get(i));
			}
			this.selected = null;
			this.dragging = null;
			this.setDragging(false);
			this.config.save();
		}));
		this.addRenderableWidget(Button.builder(PatinaLang.text("patina.editor.back"), press -> this.onClose())
			.bounds(buttonX + PatinaResetButton.WIDTH + BUTTON_GAP, 20, BACK_WIDTH, 15).build());
	}

	@Override
	public void onClose() {
		this.dragging = null;
		this.setDragging(false);
		this.config.save();
		this.minecraft.gui.setScreen(this.parent);
	}

	@Override
	public boolean isPauseScreen() {
		return false;
	}

	private PatinaHud.View view() {
		PatinaHud.View live = this.preview == null ? PatinaHud.liveView() : null;
		return live != null ? live : PatinaHud.sampleView();
	}

	private boolean shown(PatinaConfig.Placed mod) {
		return mod != null && (mod.enabled || mod == this.preview);
	}

	private PatinaHud.Box box(PatinaConfig.Placed mod) {
		return PatinaHud.place(this.font, this.width, this.height, this.view(), this.config.mods, mod);
	}

	private PatinaHud.Box hitBox(PatinaConfig.Placed mod) {
		PatinaHud.Box box = this.box(mod);
		return mod == this.config.mods.armor ? PatinaHud.armorSlots(box, this.config.mods.armor) : box;
	}

	/** The topmost shown mod under the pointer. */
	private PatinaConfig.Placed modAt(double mouseX, double mouseY) {
		List<PatinaConfig.Placed> mods = this.config.mods.all();
		for (int i = mods.size() - 1; i >= 0; i--) {
			PatinaConfig.Placed mod = mods.get(i);
			if (this.shown(mod) && this.hitBox(mod).contains(mouseX, mouseY)) {
				return mod;
			}
		}
		return null;
	}

	private boolean onHandle(double mouseX, double mouseY) {
		if (!this.shown(this.selected)) {
			return false;
		}
		PatinaHud.Box box = this.hitBox(this.selected);
		return Math.abs(mouseX - (box.x() + box.width())) <= HANDLE + 1 && Math.abs(mouseY - (box.y() + box.height())) <= HANDLE + 1;
	}

	@Override
	public boolean mouseClicked(MouseButtonEvent event, boolean doubleClick) {
		// Controls remain clickable even when a HUD element has been placed over them.
		if (super.mouseClicked(event, doubleClick)) {
			return true;
		}
		if (event.button() == InputConstants.MOUSE_BUTTON_LEFT) {
			if (this.onHandle(event.x(), event.y())) {
				this.dragging = this.selected;
				this.resizing = true;
				this.setDragging(true);
				return true;
			}
			PatinaConfig.Placed mod = this.modAt(event.x(), event.y());
			if (mod != null) {
				PatinaHud.Box box = this.box(mod);
				this.selected = mod;
				this.dragging = mod;
				this.resizing = false;
				this.setFocused(null);
				this.setDragging(true);
				this.grabX = (float) event.x() - box.x();
				this.grabY = (float) event.y() - box.y();
				return true;
			}
			this.selected = null;
		}
		return false;
	}

	@Override
	public boolean mouseDragged(MouseButtonEvent event, double dx, double dy) {
		if (this.dragging == null || event.button() != InputConstants.MOUSE_BUTTON_LEFT) {
			return super.mouseDragged(event, dx, dy);
		}
		PatinaConfig.Placed mod = this.dragging;
		PatinaHud.Box box = this.box(mod);
		int[] size = PatinaHud.size(this.font, this.view(), this.config.mods, mod);
		float x = (float) event.x();
		float y = (float) event.y();
		if (this.resizing) {
			PatinaHud.Box slots = this.hitBox(mod);
			float right = (slots.x() + slots.width() - box.x()) / mod.scale;
			float bottom = (slots.y() + slots.height() - box.y()) / mod.scale;
			this.resize(mod, box, Math.max((x - box.x()) / right, (y - box.y()) / bottom));
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
		if (this.dragging != null && event.button() == InputConstants.MOUSE_BUTTON_LEFT) {
			this.dragging = null;
			this.setDragging(false);
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

	@Override
	public void extractBackground(GuiGraphicsExtractor graphics, int mouseX, int mouseY, float a) {
		drawScene(graphics, this.width, this.height);
	}

	@Override
	public void extractRenderState(GuiGraphicsExtractor graphics, int mouseX, int mouseY, float a) {
		String hint = PatinaLang.get("patina.editor.hint");
		int hintX = (this.width - this.font.width(hint)) / 2;
		graphics.fill(hintX - 5, 4, this.width - hintX + 5, 17, 0xD011181B);
		graphics.text(this.font, hint, hintX, 7, 0xFFE9F3F0);
		PatinaHud.View view = this.view();
		PatinaConfig.Mods mods = this.config.mods;
		for (PatinaConfig.Placed mod : mods.all()) {
			if (this.shown(mod)) {
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
			this.frame(graphics, this.hitBox(hovered), HOVER_EDGE, false);
		}
		if (this.shown(this.selected)) {
			PatinaHud.Box box = this.box(this.selected);
			this.frame(graphics, this.hitBox(this.selected), ACCENT, true);
			String size = Math.round(this.selected.scale * 100f) + "%";
			int y = box.y() >= 11 ? (int) Math.floor(box.y()) - 11 : (int) Math.ceil(box.y() + box.height()) + 3;
			graphics.text(this.font, size, (int) Math.floor(box.x()), y, HINT);
		}
		if (this.resizing && this.dragging != null || this.onHandle(mouseX, mouseY)) {
			graphics.requestCursor(CursorTypes.CROSSHAIR);
		} else if (hovered != null) {
			graphics.requestCursor(CursorTypes.RESIZE_ALL);
		}
		// Draw controls last so the buttons stay visible above movable HUD elements.
		super.extractRenderState(graphics, mouseX, mouseY, a);
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

	/** The photo and a vanilla-size reference HUD, used by the HUD editor. */
	private static void drawScene(GuiGraphicsExtractor graphics, int width, int height) {
		PatinaBackground.drawHudScene(graphics, 0, 0, width, height);

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

}
