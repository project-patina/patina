package dev.patina.client.mixin;
import dev.patina.client.PatinaUtilities;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.screens.DeathScreen;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.network.chat.Component;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
@Mixin(DeathScreen.class)
abstract class UtilityDeathMixin extends Screen {
	@Unique private boolean patina$recorded;
	protected UtilityDeathMixin(Component title) { super(title); }
	@Inject(method = "init", at = @At("TAIL"))
	private void patina$recordDeath(CallbackInfo ci) { if (!this.patina$recorded) { PatinaUtilities.recordDeath(Minecraft.getInstance().player); this.patina$recorded = true; } }
	@Inject(method = "extractRenderState", at = @At("TAIL"))
	private void patina$deathInfo(GuiGraphicsExtractor g, int mx, int my, float a, CallbackInfo ci) { PatinaUtilities.drawDeath(g, this.width, this.height); }
}
