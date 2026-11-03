package dev.patina.client.mixin;

import dev.patina.client.PatinaClient;
import dev.patina.client.PatinaMenu;
import dev.patina.client.PatinaSettingsScreen;
import dev.patina.client.PatinaTitle;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.components.AbstractButton;
import net.minecraft.client.gui.components.AbstractWidget;
import net.minecraft.client.gui.components.Button;
import net.minecraft.client.gui.components.PlainTextButton;
import net.minecraft.client.gui.components.events.GuiEventListener;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.client.gui.screens.TitleScreen;
import net.minecraft.network.chat.Component;
import net.minecraft.network.chat.contents.TranslatableContents;
import net.minecraft.util.Mth;
import net.minecraft.util.Util;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.Shadow;
import org.spongepowered.asm.mixin.Unique;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

@Mixin(TitleScreen.class)
abstract class TitleScreenMixin extends Screen {
	@Unique
	private static final String LOGO = "Lnet/minecraft/client/gui/components/LogoRenderer;extractRenderState(Lnet/minecraft/client/gui/GuiGraphicsExtractor;IF)V";
	@Unique
	private static final String SPLASH = "Lnet/minecraft/client/gui/components/SplashRenderer;extractRenderState(Lnet/minecraft/client/gui/GuiGraphicsExtractor;ILnet/minecraft/client/gui/Font;F)V";
	@Unique
	private static final Comparator<AbstractWidget> READING_ORDER = Comparator.comparingInt(AbstractWidget::getY).thenComparingInt(AbstractWidget::getX);
	@Unique
	private static boolean patina$announced;

	@Shadow
	private boolean fading;
	@Shadow
	private long fadeInStart;

	private TitleScreenMixin(Component title) {
		super(title);
	}

	/** The Realms button goes, and with it the invite icons that sit on it. */
	@Inject(method = "realmsNotificationsEnabled", at = @At("HEAD"), cancellable = true, require = 0)
	private void patina$noRealmsNotifications(CallbackInfoReturnable<Boolean> cir) {
		cir.setReturnValue(false);
	}

	@Unique
	private static boolean patina$isKey(AbstractWidget widget, String key) {
		return widget.getMessage().getContents() instanceof TranslatableContents contents && contents.getKey().equals(key);
	}

	/** Runs for whatever buttons the screen ended up with, so those of other mods come along. */
	@Inject(method = "init", at = @At("TAIL"), require = 0)
	private void patina$arrange(CallbackInfo ci) {
		List<AbstractWidget> buttons = new ArrayList<>();
		for (GuiEventListener child : this.children()) {
			if (child instanceof AbstractButton button && !(child instanceof PlainTextButton)) {
				buttons.add(button);
			}
		}
		buttons.sort(READING_ORDER);

		AbstractWidget patina = Button.builder(Component.literal("Patina Client"), press -> this.minecraft.gui.setScreen(new PatinaSettingsScreen(this))).build();
		int slot = buttons.size();
		for (int i = buttons.size() - 1; i >= 0; i--) {
			if (patina$isKey(buttons.get(i), "menu.online")) {
				slot = i;
				this.removeWidget(buttons.remove(i));
				break;
			}
			if (patina$isKey(buttons.get(i), "menu.options")) {
				slot = i;
			}
		}
		buttons.add(slot, patina);

		PatinaTitle.arrange(this.width, this.height, buttons);

		// Added again in reading order, which is also the order Tab walks them in.
		buttons.sort(READING_ORDER);
		for (AbstractWidget button : buttons) {
			this.removeWidget(button);
		}
		for (AbstractWidget button : buttons) {
			this.addRenderableWidget(button);
		}
	}

	@Inject(method = "extractRenderState", at = @At("HEAD"), require = 0)
	private void patina$beginFrame(GuiGraphicsExtractor graphics, int mouseX, int mouseY, float partialTick, CallbackInfo ci) {
		long now = Util.getMillis();
		long start = this.fading && this.fadeInStart == 0L ? now : this.fadeInStart;
		PatinaTitle.beginFrame(now, start);
	}

	@Inject(method = "extractRenderState", at = @At(value = "INVOKE", target = LOGO), require = 0)
	private void patina$pushLogo(GuiGraphicsExtractor graphics, int mouseX, int mouseY, float partialTick, CallbackInfo ci) {
		PatinaTitle.pushLogo(graphics, this.width);
	}

	@Inject(method = "extractRenderState", at = @At(value = "INVOKE", target = LOGO, shift = At.Shift.AFTER), require = 0)
	private void patina$popLogo(GuiGraphicsExtractor graphics, int mouseX, int mouseY, float partialTick, CallbackInfo ci) {
		PatinaTitle.popLogo(graphics);
	}

	@Inject(method = "extractRenderState", at = @At(value = "INVOKE", target = SPLASH), require = 0)
	private void patina$pushSplash(GuiGraphicsExtractor graphics, int mouseX, int mouseY, float partialTick, CallbackInfo ci) {
		PatinaTitle.pushLogo(graphics, this.width);
	}

	@Inject(method = "extractRenderState", at = @At(value = "INVOKE", target = SPLASH, shift = At.Shift.AFTER), require = 0)
	private void patina$popSplash(GuiGraphicsExtractor graphics, int mouseX, int mouseY, float partialTick, CallbackInfo ci) {
		PatinaTitle.popLogo(graphics);
	}

	@Inject(method = "extractRenderState", at = @At("TAIL"), require = 0)
	private void patina$drawOverlay(GuiGraphicsExtractor graphics, int mouseX, int mouseY, float partialTick, CallbackInfo ci) {
		if (!patina$announced) {
			patina$announced = true;
			PatinaClient.LOGGER.info("Title screen reached, {}", PatinaClient.signature());
		}
		PatinaTitle.drawPlayer(graphics, this.width, this.height, mouseX, mouseY);
		// The curve the screen itself uses for its widgets while it fades in.
		float alpha = 1f;
		if (this.fading) {
			alpha = Mth.clampedMap((Util.getMillis() - this.fadeInStart) / 2000f, 0.5f, 1f, 0f, 1f);
		}
		PatinaMenu.drawSignature(graphics, this.font, this.height, alpha);
	}
}
