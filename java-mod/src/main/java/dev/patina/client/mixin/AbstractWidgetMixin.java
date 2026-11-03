package dev.patina.client.mixin;

import dev.patina.client.PatinaTitle;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.components.AbstractWidget;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/** Only the main menu buttons are affected; every other widget passes straight through. */
@Mixin(AbstractWidget.class)
abstract class AbstractWidgetMixin {
	@Inject(method = "extractRenderState", at = @At("HEAD"), require = 0)
	private void patina$beginWidget(GuiGraphicsExtractor graphics, int mouseX, int mouseY, float partialTick, CallbackInfo ci) {
		PatinaTitle.beginWidget((AbstractWidget) (Object) this, graphics);
	}

	@Inject(method = "extractRenderState", at = @At("TAIL"), require = 0)
	private void patina$endWidget(GuiGraphicsExtractor graphics, int mouseX, int mouseY, float partialTick, CallbackInfo ci) {
		PatinaTitle.endWidget((AbstractWidget) (Object) this, graphics);
	}

	@Inject(method = "areCoordinatesInRectangle", at = @At("HEAD"), cancellable = true, require = 0)
	private void patina$hitTest(double x, double y, CallbackInfoReturnable<Boolean> cir) {
		Boolean hit = PatinaTitle.hit((AbstractWidget) (Object) this, x, y);
		if (hit != null) {
			cir.setReturnValue(hit);
		}
	}
}
