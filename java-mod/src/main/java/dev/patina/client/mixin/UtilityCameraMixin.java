package dev.patina.client.mixin;
import dev.patina.client.PatinaConfig;
import dev.patina.client.PatinaUtilities;
import net.minecraft.client.Camera;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.ModifyArg;
import org.spongepowered.asm.mixin.injection.ModifyVariable;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;
@Mixin(Camera.class)
abstract class UtilityCameraMixin {
	@Inject(method = "calculateFov", at = @At("RETURN"), cancellable = true)
	private void patina$zoom(float partialTick, CallbackInfoReturnable<Float> cir) { cir.setReturnValue(PatinaUtilities.zoomFov(cir.getReturnValue())); }
	@ModifyVariable(method = "setRotation", at = @At("HEAD"), argsOnly = true, ordinal = 0)
	private float patina$cameraYaw(float value) { return PatinaUtilities.perspective() ? PatinaUtilities.yaw() : value; }
	@ModifyVariable(method = "setRotation", at = @At("HEAD"), argsOnly = true, ordinal = 1)
	private float patina$cameraPitch(float value) { return PatinaUtilities.perspective() ? PatinaUtilities.pitch() : value; }
	@ModifyArg(method = "alignWithEntity", at = @At(value = "INVOKE", target = "Lnet/minecraft/client/Camera;getMaxZoom(F)F"), index = 0)
	private float patina$cameraDistance(float value) { return PatinaUtilities.perspective() ? PatinaConfig.get().mods.perspective.factor : value; }
}
