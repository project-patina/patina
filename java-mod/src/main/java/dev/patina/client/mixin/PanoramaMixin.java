package dev.patina.client.mixin;

import dev.patina.client.PatinaBackground;
import dev.patina.client.PatinaTitle;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.screens.TitleScreen;
import net.minecraft.client.renderer.Panorama;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/** Every menu without a world behind it draws the panorama, so this covers them all. */
@Mixin(Panorama.class)
abstract class PanoramaMixin {
	@Inject(method = "extractRenderState", at = @At("HEAD"), cancellable = true, require = 0)
	private void patina$drawBackground(GuiGraphicsExtractor graphics, int width, int height, CallbackInfo ci) {
		PatinaBackground.drawMenu(graphics, width, height);
		if (Minecraft.getInstance().gui.screen() instanceof TitleScreen) {
			PatinaTitle.drawShade(width, height);
		}
		ci.cancel();
	}
}
