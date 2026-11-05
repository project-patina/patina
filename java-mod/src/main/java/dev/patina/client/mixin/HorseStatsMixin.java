package dev.patina.client.mixin;
import dev.patina.client.PatinaConfig;
import dev.patina.client.PatinaLang;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.screens.inventory.AbstractMountInventoryScreen;
import net.minecraft.client.gui.screens.inventory.AbstractContainerScreen;
import net.minecraft.world.entity.LivingEntity;
import net.minecraft.world.entity.animal.equine.AbstractHorse;
import net.minecraft.world.entity.ai.attributes.Attributes;
import net.minecraft.world.inventory.AbstractMountInventoryMenu;
import net.minecraft.world.entity.player.Inventory;
import net.minecraft.network.chat.Component;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
@Mixin(AbstractMountInventoryScreen.class)
abstract class HorseStatsMixin<T extends AbstractMountInventoryMenu> extends AbstractContainerScreen<T> {
	@Shadow protected LivingEntity mount;
	protected HorseStatsMixin(T menu, Inventory inventory, Component title) { super(menu, inventory, title); }
	@Inject(method = "extractRenderState", at = @At("TAIL"))
	private void patina$horseStats(GuiGraphicsExtractor g, int mx, int my, float a, CallbackInfo ci) {
		if (!PatinaConfig.get().mods.horse_stats.enabled || !(this.mount instanceof AbstractHorse horse)) return;
		double velocity = horse.getAttributeValue(Attributes.JUMP_STRENGTH), jump = 0;
		for (int i = 0; i < 100 && velocity > 0; i++) { jump += velocity; velocity = (velocity - 0.08) * 0.98; }
		String[] lines = {
			PatinaLang.get("patina.horse.health") + String.format(java.util.Locale.ROOT, " %.0f / %.0f", horse.getHealth(), horse.getMaxHealth()),
			PatinaLang.get("patina.horse.speed") + String.format(java.util.Locale.ROOT, " %.2f m/s", horse.getAttributeValue(Attributes.MOVEMENT_SPEED) * 42.1578),
			PatinaLang.get("patina.horse.jump") + String.format(java.util.Locale.ROOT, " %.2f", jump)
		};
		int w = java.util.Arrays.stream(lines).mapToInt(this.font::width).max().orElse(100) + 8;
		int x = this.leftPos >= w + 6 ? this.leftPos - w - 6 : this.leftPos;
		int y = this.leftPos >= w + 6 ? this.topPos : Math.max(4, this.topPos - 39);
		g.fill(x, y, x + w, y + 36, 0xE810181B); g.outline(x, y, w, 36, 0xFF3FB8A0);
		for (int i = 0; i < lines.length; i++) g.text(this.font, lines[i], x + 4, y + 4 + i * 10, 0xFFE8F0EE);
	}
}
