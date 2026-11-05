package dev.patina.client.mixin;
import dev.patina.client.PatinaKeyBindings;
import dev.patina.client.PatinaUtilities;
import net.minecraft.client.KeyboardHandler;
import net.minecraft.client.Minecraft;
import net.minecraft.client.input.KeyEvent;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
@Mixin(KeyboardHandler.class)
abstract class UtilityKeyboardMixin {
	@Inject(method = "keyPress", at = @At("HEAD"))
	private void patina$utilityKey(long window, int action, KeyEvent event, CallbackInfo ci) {
		if (action == 1 && window == Minecraft.getInstance().getWindow().handle()) PatinaUtilities.keyPressed(PatinaKeyBindings.sharedCode(event.key()));
	}
}
