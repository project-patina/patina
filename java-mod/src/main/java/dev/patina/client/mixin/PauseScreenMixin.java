package dev.patina.client.mixin;

import com.llamalad7.mixinextras.sugar.Local;
import dev.patina.client.PatinaSettingsScreen;
import net.minecraft.client.gui.components.Button;
import net.minecraft.client.gui.layouts.GridLayout;
import net.minecraft.client.gui.screens.PauseScreen;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.network.chat.Component;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

@Mixin(PauseScreen.class)
abstract class PauseScreenMixin extends Screen {
	@Unique
	private static final int FULL_WIDTH = 204;

	private PauseScreenMixin(Component title) {
		super(title);
	}

	/** Takes a row of its own right above the disconnect button. */
	@Inject(
		method = "createPauseMenu",
		at = @At(value = "INVOKE", target = "Lnet/minecraft/network/chat/CommonComponents;disconnectButtonLabel(Z)Lnet/minecraft/network/chat/Component;"),
		require = 0
	)
	private void patina$addButton(CallbackInfo ci, @Local GridLayout.RowHelper helper) {
		Button button = Button.builder(Component.literal("Patina Client"), press -> this.minecraft.gui.setScreen(new PatinaSettingsScreen(this))).width(FULL_WIDTH).build();
		helper.addChild(button, 2);
	}
}
