package dev.patina.client.mixin;
import dev.patina.client.PatinaConfig;
import net.minecraft.client.Camera;
import net.minecraft.client.DeltaTracker;
import net.minecraft.client.multiplayer.ClientLevel;
import net.minecraft.client.renderer.fog.FogData;
import net.minecraft.client.renderer.fog.FogRenderer;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;
@Mixin(FogRenderer.class)
abstract class UtilityFogMixin {
	@Inject(method = "setupFog", at = @At("RETURN"))
	private void patina$noFog(Camera camera, int distance, DeltaTracker delta, float darken, ClientLevel level, CallbackInfoReturnable<FogData> cir) {
		if (!PatinaConfig.get().mods.no_fog.enabled) return;
		FogData data = cir.getReturnValue();
		data.environmentalStart = data.renderDistanceStart = 1000000f;
		data.environmentalEnd = data.renderDistanceEnd = data.skyEnd = data.cloudEnd = 1000001f;
	}
}
