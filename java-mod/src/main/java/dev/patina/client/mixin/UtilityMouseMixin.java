package dev.patina.client.mixin;
import dev.patina.client.PatinaUtilities;
import net.minecraft.client.MouseHandler;
import net.minecraft.client.player.LocalPlayer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.Redirect;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
@Mixin(MouseHandler.class)
abstract class UtilityMouseMixin {
	@Redirect(method = "turnPlayer", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/player/LocalPlayer;turn(DD)V"))
	private void patina$turn(LocalPlayer player, double x, double y) { PatinaUtilities.turn(player, x, y); }
	@Inject(method = "onScroll", at = @At("HEAD"), cancellable = true)
	private void patina$zoomScroll(long window, double x, double y, CallbackInfo ci) { if (PatinaUtilities.scrollZoom(y)) ci.cancel(); }
}
