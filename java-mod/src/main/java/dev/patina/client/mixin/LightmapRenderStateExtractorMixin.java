package dev.patina.client.mixin;

import dev.patina.client.PatinaConfig;
import net.minecraft.client.Minecraft;
import net.minecraft.client.renderer.LightmapRenderStateExtractor;
import net.minecraft.client.renderer.state.LightmapRenderState;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/** Full Bright changes only the client lightmap, leaving gamma and world light untouched. */
@Mixin(LightmapRenderStateExtractor.class)
abstract class LightmapRenderStateExtractorMixin {
	@Shadow private boolean needsUpdate;
	@Unique private boolean patina$wasFullBright;

	@Inject(method = "extract", at = @At("HEAD"))
	private void patina$refreshLightmap(LightmapRenderState state, float partialTick, CallbackInfo ci) {
		boolean enabled = PatinaConfig.get().mods.fullbright.enabled;
		if (enabled != this.patina$wasFullBright) this.needsUpdate = true;
		this.patina$wasFullBright = enabled;
	}

	@Inject(method = "extract", at = @At("TAIL"))
	private void patina$fullBright(LightmapRenderState state, float partialTick, CallbackInfo ci) {
		if (!this.patina$wasFullBright || Minecraft.getInstance().level == null) return;
		state.ambientColor = LightmapRenderStateExtractor.WHITE;
		state.blockFactor = 0f;
		state.skyFactor = 0f;
		state.darknessEffectScale = 0f;
		state.bossOverlayWorldDarkening = 0f;
		state.brightness = 1f;
		state.needsUpdate = true;
	}
}
