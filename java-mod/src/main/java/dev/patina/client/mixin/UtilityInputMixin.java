package dev.patina.client.mixin;
import dev.patina.client.PatinaUtilities;
import net.minecraft.client.Minecraft;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
@Mixin(Minecraft.class)
abstract class UtilityInputMixin {
	@Inject(method = "tick", at = @At("TAIL"))
	private void patina$utilityTick(CallbackInfo ci) { PatinaUtilities.tick(); }
}
