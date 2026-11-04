package dev.patina.client.mixin;

import dev.patina.client.PatinaPresence;
import net.minecraft.client.Minecraft;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(Minecraft.class)
abstract class MinecraftMixin {
	/** Every client tick, in the menus too: tells the launcher when the activity changes. */
	@Inject(method = "tick", at = @At("TAIL"))
	private void patina$reportPresence(CallbackInfo ci) {
		PatinaPresence.tick((Minecraft) (Object) this);
	}
}
