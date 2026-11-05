package dev.patina.client.mixin;
import dev.patina.client.PatinaConfig;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.TooltipFlag;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;
@Mixin(targets = "squeek.appleskin.client.TooltipOverlayHandler", remap = false)
abstract class AppleSkinTooltipToggleMixin {
	@Inject(method = "shouldShowTooltip", at = @At("HEAD"), cancellable = true)
	private void patina$appleTooltip(ItemStack stack, TooltipFlag flags, CallbackInfoReturnable<Boolean> cir) {
		if (!PatinaConfig.get().mods.appleskin.enabled) cir.setReturnValue(false);
	}
}
