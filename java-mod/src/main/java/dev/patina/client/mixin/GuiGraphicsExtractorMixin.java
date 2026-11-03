package dev.patina.client.mixin;

import com.mojang.renderpearl.api.pipeline.RenderPipeline;
import dev.patina.client.PatinaMenu;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.resources.Identifier;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/** The two overloads every sprite and texture blit of a widget or screen background ends in. */
@Mixin(GuiGraphicsExtractor.class)
abstract class GuiGraphicsExtractorMixin {
	@Inject(
		method = "blitSprite(Lcom/mojang/renderpearl/api/pipeline/RenderPipeline;Lnet/minecraft/resources/Identifier;IIIII)V",
		at = @At("HEAD"), cancellable = true, require = 0
	)
	private void patina$drawSprite(RenderPipeline pipeline, Identifier sprite, int x, int y, int width, int height, int color, CallbackInfo ci) {
		if (PatinaMenu.drawSprite((GuiGraphicsExtractor) (Object) this, sprite, x, y, width, height, color)) {
			ci.cancel();
		}
	}

	@Inject(
		method = "blit(Lcom/mojang/renderpearl/api/pipeline/RenderPipeline;Lnet/minecraft/resources/Identifier;IIFFIIIIIII)V",
		at = @At("HEAD"), cancellable = true, require = 0
	)
	private void patina$drawTexture(RenderPipeline pipeline, Identifier texture, int x, int y, float u, float v, int width, int height, int srcWidth, int srcHeight, int textureWidth, int textureHeight, int color, CallbackInfo ci) {
		if (PatinaMenu.drawTexture((GuiGraphicsExtractor) (Object) this, texture, x, y, width, height)) {
			ci.cancel();
		}
	}
}
