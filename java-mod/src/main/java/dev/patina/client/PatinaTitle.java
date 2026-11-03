package dev.patina.client;

import java.util.ArrayList;
import java.util.IdentityHashMap;
import java.util.List;
import java.util.Map;
import java.util.function.Supplier;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.components.AbstractWidget;
import net.minecraft.client.model.Model;
import net.minecraft.client.model.geom.ModelLayers;
import net.minecraft.client.model.geom.ModelPart;
import net.minecraft.client.renderer.rendertype.RenderTypes;
import net.minecraft.util.Mth;
import net.minecraft.world.entity.player.PlayerModelType;
import net.minecraft.world.entity.player.PlayerSkin;
import org.joml.Matrix3x2f;

/**
 * The layout and motion of the main menu. Its buttons stay plain rectangles of the screen; they
 * are only drawn on a plane turned away to the right, and pointer tests go through the inverse of
 * that projection, so what looks hit is what is hit.
 */
public final class PatinaTitle {
	private static final float TURN = (float) Math.toRadians(17);
	private static final float TURN_COS = Mth.cos(TURN);
	private static final float TURN_SIN = Mth.sin(TURN);
	/** Distance to the plane in GUI units: smaller is a stronger perspective. */
	private static final float FOCAL = 440f;
	private static final float HOVER_PUSH = 5f;
	/** What the hover glow closes of its distance in a sixtieth of a second, at any frame rate. */
	private static final float HOVER_EASE = 0.3f;

	private static final int COLUMN_WIDTH = 170;
	private static final int ICON_SIZE = 20;
	private static final int PITCH = 24;
	private static final int BUTTON_HEIGHT = 20;
	private static final int ICON_GAP = 4;
	private static final float LOGO_SCALE = 0.72f;
	private static final int LOGO_WIDTH = 256;
	private static final int LOGO_HEIGHT = 51;
	private static final int VANILLA_LOGO_Y = 30;
	/** What the bottom left corner keeps free for the version lines. */
	private static final int BOTTOM_RESERVED = 26;

	private static final long LOGO_DELAY = 450;
	private static final long LOGO_TIME = 1000;
	private static final long BUTTON_DELAY = 750;
	private static final long BUTTON_STAGGER = 55;
	private static final long BUTTON_TIME = 650;
	private static final long PLAYER_DELAY = 900;
	private static final long PLAYER_TIME = 800;

	private static final float MODEL_HEIGHT = 2.125f;
	private static final float MODEL_FIT = 0.97f;
	private static final float MODEL_PIVOT = -1.0625f;
	private static final float MODEL_TILT = -3f;
	private static final float MODEL_TURN = -12f;
	private static final float HEAD_YAW_LIMIT = 0.8f;
	private static final float HEAD_PITCH_LIMIT = 0.42f;
	private static final float HEAD_REACH = 90f;

	private static final class Slot {
		final int order;
		float hover;
		float offset;

		Slot(int order) {
			this.order = order;
		}
	}

	private static final Map<AbstractWidget, Slot> SLOTS = new IdentityHashMap<>();
	private static AbstractWidget drawing;
	private static float originX;
	private static float centerY;
	private static int logoLeft;
	private static int logoTop;
	/** Milliseconds since the motion began; large once it has played. */
	private static long elapsed = Long.MAX_VALUE / 2;
	private static long lastFrame;
	private static float hoverStep = HOVER_EASE;

	private static Model.Simple wideModel;
	private static Model.Simple slimModel;
	private static Supplier<PlayerSkin> skin;

	private PatinaTitle() {
	}

	/** Takes the buttons in the order they read and puts them in a column on the left. */
	public static void arrange(int width, int height, List<AbstractWidget> buttons) {
		List<AbstractWidget> column = new ArrayList<>();
		List<AbstractWidget> icons = new ArrayList<>();
		for (AbstractWidget button : buttons) {
			(button.getWidth() <= ICON_SIZE + 4 ? icons : column).add(button);
		}
		logoLeft = Math.max(20, width / 18);
		logoTop = Math.max(8, height / 14);
		int top = logoTop + Math.round(LOGO_HEIGHT * LOGO_SCALE) + 12;
		int bottom = height - BOTTOM_RESERVED;
		int rows = column.size() + (icons.isEmpty() ? 0 : 1);
		int pitch = Math.min(PITCH, (bottom - top) / Math.max(1, rows));
		int buttonHeight = Math.min(BUTTON_HEIGHT, pitch - 2);
		int columnWidth = Math.min(COLUMN_WIDTH, width * 2 / 5);
		originX = logoLeft;
		centerY = top + pitch * rows / 2f;

		SLOTS.clear();
		int y = top;
		for (AbstractWidget button : column) {
			button.setRectangle(columnWidth, buttonHeight, logoLeft, y);
			SLOTS.put(button, new Slot(SLOTS.size()));
			y += pitch;
		}
		int x = logoLeft;
		for (AbstractWidget icon : icons) {
			icon.setPosition(x, y + 1);
			SLOTS.put(icon, new Slot(SLOTS.size()));
			x += icon.getWidth() + ICON_GAP;
		}
	}

	/** @param fadeStart when the title faded in, or 0 when the screen was reopened and shows at rest */
	public static void beginFrame(long now, long fadeStart) {
		elapsed = fadeStart == 0 ? Long.MAX_VALUE / 2 : now - fadeStart;
		float frames = lastFrame == 0 ? 1f : Math.clamp((now - lastFrame) * 60f / 1000f, 0f, 60f);
		hoverStep = 1f - (float) Math.pow(1f - HOVER_EASE, frames);
		lastFrame = now;
		for (Slot slot : SLOTS.values()) {
			float arrived = eased(BUTTON_DELAY + slot.order * BUTTON_STAGGER, BUTTON_TIME);
			slot.offset = -(1f - arrived) * (originX + COLUMN_WIDTH + 24f);
		}
	}

	private static float eased(long delay, long time) {
		float t = Math.clamp((float) (elapsed - delay) / time, 0f, 1f);
		float rest = 1f - t;
		return 1f - rest * rest * rest * rest;
	}

	/** Moves what the game draws at the logo's place to where the logo is heading. */
	public static void pushLogo(GuiGraphicsExtractor graphics, int width) {
		float t = eased(LOGO_DELAY, LOGO_TIME);
		float vanillaLeft = width / 2f - LOGO_WIDTH / 2f;
		float scale = Mth.lerp(t, 1f, LOGO_SCALE);
		float left = Mth.lerp(t, vanillaLeft, logoLeft);
		float top = Mth.lerp(t, VANILLA_LOGO_Y, logoTop);
		graphics.pose().pushMatrix();
		graphics.pose().translate(left, top);
		graphics.pose().scale(scale);
		graphics.pose().translate(-vanillaLeft, -VANILLA_LOGO_Y);
	}

	public static void popLogo(GuiGraphicsExtractor graphics) {
		graphics.pose().popMatrix();
	}

	private static float scaleAt(float u) {
		return FOCAL / (FOCAL + u * TURN_SIN);
	}

	private static float screenX(float u) {
		return originX + u * TURN_COS * scaleAt(u);
	}

	private static float screenY(float u, float y) {
		return centerY + (y - centerY) * scaleAt(u);
	}

	/** Called for every widget hit test: whether a menu button lies under the pointer. */
	public static Boolean hit(AbstractWidget widget, double mouseX, double mouseY) {
		if (!SLOTS.containsKey(widget)) {
			return null;
		}
		double x = mouseX - originX;
		double u = x * FOCAL / (TURN_COS * FOCAL - x * TURN_SIN);
		double y = centerY + (mouseY - centerY) / scaleAt((float) u);
		return u + originX >= widget.getX() && u + originX < widget.getRight() && y >= widget.getY() && y < widget.getBottom();
	}

	/** Lays the label on the same plane: at this size an affine fit of the projection is enough. */
	public static void beginWidget(AbstractWidget widget, GuiGraphicsExtractor graphics) {
		Slot slot = SLOTS.get(widget);
		if (slot == null) {
			return;
		}
		drawing = widget;
		float target = widget.isHoveredOrFocused() ? 1f : 0f;
		slot.hover += (target - slot.hover) * hoverStep;

		float u = widget.getX() + widget.getWidth() / 2f - originX;
		float y = widget.getY() + widget.getHeight() / 2f;
		float shift = slot.offset + slot.hover * HOVER_PUSH;
		float across = screenX(u + 1f) - screenX(u);
		float slope = screenY(u + 1f, y) - screenY(u, y);
		float down = scaleAt(u);
		float centerX = u + originX;
		Matrix3x2f plane = new Matrix3x2f(
			across, slope,
			0f, down,
			screenX(u) + shift - across * centerX, screenY(u, y) - slope * centerX - down * y
		);
		graphics.pose().pushMatrix();
		graphics.pose().mul(plane);
	}

	public static void endWidget(AbstractWidget widget, GuiGraphicsExtractor graphics) {
		if (drawing == widget) {
			drawing = null;
			graphics.pose().popMatrix();
		}
	}

	/** Draws the panel of the button being drawn, if a menu button is. */
	public static boolean drawPanel(float alpha, boolean hover, boolean active) {
		if (drawing == null) {
			return false;
		}
		AbstractWidget widget = drawing;
		Slot slot = SLOTS.get(widget);
		float left = widget.getX() - originX;
		float right = left + widget.getWidth();
		float top = widget.getY();
		float bottom = widget.getBottom();
		float shift = slot.offset + slot.hover * HOVER_PUSH;
		float glow = slot.hover;
		int accent = 0xFF000000 | PatinaClient.ACCENT;

		int nearFill = active ? mix(0xD0121A1C, 0xE0174A44, glow) : 0x80101416;
		int farFill = active ? mix(0xA80A0F11, 0xC00F2B28, glow) : 0x60101214;
		strip(left, top, right, bottom, nearFill, farFill, shift, alpha);
		// The underside gives the panel its thickness.
		strip(left, bottom, right, bottom + 2f, 0xD0040607, 0xB0040607, shift, alpha);
		int edge = active ? mix(0x38FFFFFF, accent, glow) : 0x18FFFFFF;
		strip(left, top, right, top + 1f, edge, edge, shift, alpha);
		strip(left, bottom - 1f, right, bottom, mix(0x14FFFFFF, accent, glow), mix(0x14FFFFFF, accent, glow), shift, alpha);
		strip(left, top, left + 1f + glow, bottom, edge, edge, shift, alpha);
		strip(right - 1f, top, right, bottom, edge, edge, shift, alpha);
		return true;
	}

	private static void strip(float left, float top, float right, float bottom, int nearColor, int farColor, float shift, float alpha) {
		int near = PatinaClient.fade(nearColor, alpha);
		int far = PatinaClient.fade(farColor, alpha);
		float nearX = screenX(left) + shift;
		float farX = screenX(right) + shift;
		GuiQuad.add(
			new float[] {nearX, screenY(left, top), nearX, screenY(left, bottom), farX, screenY(right, bottom), farX, screenY(right, top)},
			new int[] {near, near, far, far}
		);
	}

	private static int mix(int from, int to, float t) {
		int a = Math.round(Mth.lerp(t, from >>> 24, to >>> 24));
		int r = Math.round(Mth.lerp(t, from >> 16 & 255, to >> 16 & 255));
		int g = Math.round(Mth.lerp(t, from >> 8 & 255, to >> 8 & 255));
		int b = Math.round(Mth.lerp(t, from & 255, to & 255));
		return a << 24 | r << 16 | g << 8 | b;
	}

	/** Darkens the side the buttons are on, so they read over any picture. */
	public static void drawShade(int width, int height) {
		float right = width * 0.62f;
		int dark = 0x88000000;
		int clear = 0;
		GuiQuad.add(new float[] {0, 0, 0, height, right, height, right, 0}, new int[] {dark, dark, clear, clear});
	}

	/** The player's own skin, standing still; only the head moves, towards the pointer. */
	public static void drawPlayer(GuiGraphicsExtractor graphics, int width, int height, int mouseX, int mouseY) {
		Minecraft minecraft = Minecraft.getInstance();
		if (skin == null) {
			skin = minecraft.getSkinManager().createLookup(minecraft.getGameProfile(), false);
			wideModel = new Model.Simple(minecraft.getEntityModels().bakeLayer(ModelLayers.PLAYER), RenderTypes::entityTranslucent);
			slimModel = new Model.Simple(minecraft.getEntityModels().bakeLayer(ModelLayers.PLAYER_SLIM), RenderTypes::entityTranslucent);
		}
		int boxHeight = height * 3 / 4;
		int boxWidth = boxHeight * 3 / 5;
		int restX = width * 3 / 4 - boxWidth / 2;
		int x = restX + Math.round((1f - eased(PLAYER_DELAY, PLAYER_TIME)) * (width - restX + 8));
		int y = (height - boxHeight) * 3 / 5;
		if (x >= width) {
			return;
		}
		PlayerSkin current = skin.get();
		Model.Simple model = current.model() == PlayerModelType.SLIM ? slimModel : wideModel;
		ModelPart head = model.root().getChild("head");
		float headY = y + boxHeight * 0.13f;
		float headX = x + boxWidth / 2f;
		// The model's own turn is taken out, so the head faces the pointer rather than a point beside it.
		head.yRot = Mth.clamp((float) -Math.atan((mouseX - headX) / HEAD_REACH), -HEAD_YAW_LIMIT, HEAD_YAW_LIMIT) + (float) Math.toRadians(MODEL_TURN);
		head.xRot = Mth.clamp((float) Math.atan((mouseY - headY) / HEAD_REACH), -HEAD_PITCH_LIMIT, HEAD_PITCH_LIMIT);
		float scale = MODEL_FIT * boxHeight / MODEL_HEIGHT;
		graphics.skin(model, current.body().texturePath(), scale, MODEL_TILT, MODEL_TURN, MODEL_PIVOT, x, y, x + boxWidth, y + boxHeight);
	}
}
