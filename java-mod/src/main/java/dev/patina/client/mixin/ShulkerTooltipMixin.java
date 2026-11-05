package dev.patina.client.mixin;
import dev.patina.client.PatinaShulkerTooltip;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.Font;
import net.minecraft.world.item.ItemStack;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
@Mixin(GuiGraphicsExtractor.class)
abstract class ShulkerTooltipMixin {
	@Inject(method = "setTooltipForNextFrame(Lnet/minecraft/client/gui/Font;Lnet/minecraft/world/item/ItemStack;II)V", at = @At("HEAD"), cancellable = true)
	private void patina$shulker(Font font, ItemStack stack, int x, int y, CallbackInfo ci) {
		if (PatinaShulkerTooltip.draw((GuiGraphicsExtractor)(Object)this, font, stack, x, y)) ci.cancel();
	}
}
