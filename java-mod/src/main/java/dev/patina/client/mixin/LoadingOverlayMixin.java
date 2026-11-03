package dev.patina.client.mixin;

import dev.patina.client.PatinaClient;
import dev.patina.client.PatinaSplash;
import java.util.function.IntSupplier;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.screens.LoadingOverlay;
import net.minecraft.client.renderer.texture.TextureManager;
import net.minecraft.resources.Identifier;
import net.minecraft.util.Util;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.Redirect;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

import com.mojang.renderpearl.api.pipeline.RenderPipeline;

/**
 * Only what the overlay draws and when it may leave change. Its fades and the hand-over to the
 * title screen stay in vanilla code, so a failed injection leaves the plain overlay and never a
 * stuck game.
 */
@Mixin(LoadingOverlay.class)
abstract class LoadingOverlayMixin {
	@Unique
	private static final String BLIT = "Lnet/minecraft/client/gui/GuiGraphicsExtractor;blit(Lcom/mojang/renderpearl/api/pipeline/RenderPipeline;Lnet/minecraft/resources/Identifier;IIFFIIIIIII)V";
	@Unique
	private static boolean patina$announced;

	@Shadow
	private float currentProgress;
	@Shadow
	private long fadeOutStart;

	@Inject(method = "<init>", at = @At("TAIL"), require = 0)
	private void patina$restartAnimation(CallbackInfo ci) {
		PatinaSplash.reset();
	}

	/** Holds the hand-over back, never for good: a loop always reaches the next whole cube. */
	@Inject(method = "isReadyToFadeOut", at = @At("HEAD"), cancellable = true, require = 0)
	private void patina$waitForCube(CallbackInfoReturnable<Boolean> cir) {
		if (!PatinaSplash.mayLeave(Util.getMillis())) {
			cir.setReturnValue(false);
		}
	}

	@Inject(method = "registerTextures", at = @At("TAIL"), require = 0)
	private static void patina$registerSplash(TextureManager textures, CallbackInfo ci) {
		PatinaSplash.registerTexture(textures);
	}

	@Redirect(method = "extractRenderState", at = @At(value = "INVOKE", target = "Ljava/util/function/IntSupplier;getAsInt()I"), require = 0)
	private int patina$blackBackground(IntSupplier brandBackground) {
		return 0xFF000000;
	}

	/** The Mojang logo is drawn in two halves; the first becomes the animation at the same alpha. */
	@Redirect(method = "extractRenderState", at = @At(value = "INVOKE", target = BLIT, ordinal = 0), require = 0)
	private void patina$drawAnimation(GuiGraphicsExtractor graphics, RenderPipeline pipeline, Identifier texture, int x, int y, float u, float v, int width, int height, int regionWidth, int regionHeight, int textureWidth, int textureHeight, int color) {
		if (!patina$announced) {
			patina$announced = true;
			PatinaClient.LOGGER.info("Loading animation active");
		}
		PatinaSplash.drawAnimation(graphics, Util.getMillis(), color, this.fadeOutStart > -1L);
	}

	@Redirect(method = "extractRenderState", at = @At(value = "INVOKE", target = BLIT, ordinal = 1), require = 0)
	private void patina$dropSecondLogoHalf(GuiGraphicsExtractor graphics, RenderPipeline pipeline, Identifier texture, int x, int y, float u, float v, int width, int height, int regionWidth, int regionHeight, int textureWidth, int textureHeight, int color) {
	}

	@Inject(method = "extractProgressBar", at = @At("HEAD"), cancellable = true, require = 0)
	private void patina$drawBar(GuiGraphicsExtractor graphics, int x0, int y0, int x1, int y1, float opacity, CallbackInfo ci) {
		PatinaSplash.drawBar(graphics, this.currentProgress, opacity);
		ci.cancel();
	}
}
