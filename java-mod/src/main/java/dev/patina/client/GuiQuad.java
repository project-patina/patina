package dev.patina.client;

import com.mojang.blaze3d.vertex.VertexConsumer;
import com.mojang.renderpearl.api.pipeline.RenderPipeline;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.navigation.ScreenRectangle;
import net.minecraft.client.gui.render.TextureSetup;
import net.minecraft.client.renderer.RenderPipelines;
import net.minecraft.client.renderer.state.gui.GuiElementRenderState;
import org.joml.Matrix3x2f;
import org.joml.Matrix3x2fc;
import org.jspecify.annotations.Nullable;

/** A flat quad with four free corners and colours, which the graphics methods have no call for. */
public record GuiQuad(float[] xy, int[] colors, ScreenRectangle bounds) implements GuiElementRenderState {
	private static final Matrix3x2fc IDENTITY = new Matrix3x2f();

	/** Corners run top left, bottom left, bottom right, top right. */
	public static void add(float[] xy, int[] colors) {
		float left = Math.min(Math.min(xy[0], xy[2]), Math.min(xy[4], xy[6]));
		float right = Math.max(Math.max(xy[0], xy[2]), Math.max(xy[4], xy[6]));
		float top = Math.min(Math.min(xy[1], xy[3]), Math.min(xy[5], xy[7]));
		float bottom = Math.max(Math.max(xy[1], xy[3]), Math.max(xy[5], xy[7]));
		ScreenRectangle bounds = new ScreenRectangle((int) Math.floor(left), (int) Math.floor(top), (int) Math.ceil(right - left) + 1, (int) Math.ceil(bottom - top) + 1);
		Minecraft.getInstance().gameRenderer.gameRenderState().guiRenderState.addGuiElement(new GuiQuad(xy, colors, bounds));
	}

	@Override
	public void buildVertices(VertexConsumer vertices) {
		for (int i = 0; i < 4; i++) {
			vertices.addVertexWith2DPose(IDENTITY, this.xy[i * 2], this.xy[i * 2 + 1]).setColor(this.colors[i]);
		}
	}

	@Override
	public RenderPipeline pipeline() {
		return RenderPipelines.GUI;
	}

	@Override
	public TextureSetup textureSetup() {
		return TextureSetup.noTexture();
	}

	@Override
	public @Nullable ScreenRectangle scissorArea() {
		return null;
	}
}
