package dev.patina.client.mixin;
import dev.patina.client.PatinaConfig;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.world.entity.player.Player;
/** Keeps upstream's drawing and synchronization intact; the Patina switch gates its HUD. */
@Mixin(targets = "squeek.appleskin.client.HUDOverlayHandler", remap = false)
abstract class AppleSkinToggleMixin {
	@Inject(method = "onPreRenderFood", at = @At("HEAD"), cancellable = true)
	private void patina$appleExhaustion(GuiGraphicsExtractor g, Player player, int x, int y, CallbackInfo ci) {
		if (!PatinaConfig.get().mods.appleskin.enabled) ci.cancel();
	}
	@Inject(method = "shouldRenderAnyOverlays", at = @At("HEAD"), cancellable = true)
	private void patina$appleHud(CallbackInfoReturnable<Boolean> cir) { if (!PatinaConfig.get().mods.appleskin.enabled) cir.setReturnValue(false); }
}
