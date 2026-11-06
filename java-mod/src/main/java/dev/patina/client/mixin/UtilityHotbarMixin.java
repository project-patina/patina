package dev.patina.client.mixin;
import dev.patina.client.PatinaUtilities;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.screens.inventory.CreativeModeInventoryScreen;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
/** Zoom's default key is also the save toolbar key: while zoomed, a number key only selects its slot. */
@Mixin(CreativeModeInventoryScreen.class)
abstract class UtilityHotbarMixin {
	@Inject(method = "handleHotbarLoadOrSave", at = @At("HEAD"), cancellable = true)
	private static void patina$notWhileZoomed(Minecraft minecraft, int index, boolean load, boolean save, CallbackInfo ci) {
		if (!PatinaUtilities.zooming() || minecraft.player == null) return;
		minecraft.player.getInventory().setSelectedSlot(index);
		ci.cancel();
	}
}
