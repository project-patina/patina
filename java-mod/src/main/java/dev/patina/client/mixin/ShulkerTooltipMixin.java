package dev.patina.client.mixin;
import dev.patina.client.PatinaShulkerTooltip;
import java.util.List;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.screens.inventory.tooltip.ClientTooltipComponent;
import net.minecraft.client.gui.screens.inventory.tooltip.ClientTooltipPositioner;
import net.minecraft.client.gui.screens.inventory.tooltip.DefaultTooltipPositioner;
import net.minecraft.resources.Identifier;
import net.minecraft.world.item.ItemStack;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
@Mixin(GuiGraphicsExtractor.class)
abstract class ShulkerTooltipMixin {
	@Shadow
	private void setTooltipForNextFrameInternal(Font font, List<ClientTooltipComponent> components, int x, int y, ClientTooltipPositioner positioner, Identifier background, boolean focused) { }

	/** Queued like vanilla's own tooltip, so it is drawn over the rest of the screen. */
	@Inject(method = "setTooltipForNextFrame(Lnet/minecraft/client/gui/Font;Lnet/minecraft/world/item/ItemStack;II)V", at = @At("HEAD"), cancellable = true)
	private void patina$shulker(Font font, ItemStack stack, int x, int y, CallbackInfo ci) {
		List<ClientTooltipComponent> components = PatinaShulkerTooltip.components(stack);
		if (components == null) return;
		this.setTooltipForNextFrameInternal(font, components, x, y, DefaultTooltipPositioner.INSTANCE, null, false);
		ci.cancel();
	}
}
