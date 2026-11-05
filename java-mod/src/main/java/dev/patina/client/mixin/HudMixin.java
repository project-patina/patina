package dev.patina.client.mixin;

import dev.patina.client.PatinaHud;
import dev.patina.client.PatinaConfig;
import net.minecraft.client.DeltaTracker;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.Hud;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(Hud.class)
abstract class HudMixin {
	@Inject(method = "extractEffects", at = @At("HEAD"), cancellable = true)
	private void patina$replaceEffects(GuiGraphicsExtractor graphics, DeltaTracker deltaTracker, CallbackInfo ci) {
		if (PatinaConfig.get().mods.status_effects.enabled) ci.cancel();
	}
	@Inject(method = "extractRenderState(Lnet/minecraft/client/gui/GuiGraphicsExtractor;Lnet/minecraft/client/DeltaTracker;)V", at = @At("TAIL"), require = 0)
	private void patina$drawMods(GuiGraphicsExtractor graphics, DeltaTracker deltaTracker, CallbackInfo ci) {
		PatinaHud.drawGame(graphics);
	}
}
