package dev.patina.client.mixin;
import dev.patina.client.PatinaConfig;
import net.minecraft.world.level.Level;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;
/** Overrides queried client weather; packets and the integrated/server world stay untouched. */
@Mixin(Level.class)
abstract class UtilityWeatherMixin {
	private boolean patina$clientOverride() { return ((Level)(Object)this).isClientSide() && PatinaConfig.get().mods.weather.enabled; }
	@Inject(method = "getRainLevel", at = @At("HEAD"), cancellable = true)
	private void patina$rain(float partialTick, CallbackInfoReturnable<Float> cir) {
		if (patina$clientOverride()) { var weather = PatinaConfig.get().mods.weather; cir.setReturnValue("clear".equals(weather.weather) ? 0f : weather.strength); }
	}
	@Inject(method = "getThunderLevel", at = @At("HEAD"), cancellable = true)
	private void patina$thunder(float partialTick, CallbackInfoReturnable<Float> cir) {
		if (patina$clientOverride()) { var weather = PatinaConfig.get().mods.weather; cir.setReturnValue("thunder".equals(weather.weather) ? weather.strength : 0f); }
	}
	@Inject(method = "isRaining", at = @At("HEAD"), cancellable = true)
	private void patina$raining(CallbackInfoReturnable<Boolean> cir) { if (patina$clientOverride()) cir.setReturnValue(!"clear".equals(PatinaConfig.get().mods.weather.weather)); }
}
