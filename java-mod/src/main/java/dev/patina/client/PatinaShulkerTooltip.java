package dev.patina.client;
import java.util.ArrayList;
import java.util.List;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.screens.inventory.tooltip.ClientTooltipComponent;
import net.minecraft.core.component.DataComponents;
import net.minecraft.network.chat.Component;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.level.block.ShulkerBoxBlock;
import net.minecraft.world.item.BlockItem;

/** A read-only 9 by 3 inventory inside the item's ordinary tooltip. */
public final class PatinaShulkerTooltip {
	private PatinaShulkerTooltip() { }
	/** The tooltip's lines and grid, or null when the vanilla tooltip should show. */
	public static List<ClientTooltipComponent> components(ItemStack stack) {
		var config = PatinaConfig.get().mods.shulker_tooltip;
		if (!config.enabled || (config.require_shift && !com.mojang.blaze3d.platform.InputConstants.isKeyDown(com.mojang.blaze3d.platform.InputConstants.KEY_LSHIFT) && !com.mojang.blaze3d.platform.InputConstants.isKeyDown(com.mojang.blaze3d.platform.InputConstants.KEY_RSHIFT))
			|| !(stack.getItem() instanceof BlockItem block) || !(block.getBlock() instanceof ShulkerBoxBlock)) return null;
		var contents = stack.get(DataComponents.CONTAINER);
		List<ItemStack> slots = contents == null ? List.of() : contents.itemCopies().limit(27).toList();
		List<ClientTooltipComponent> components = new ArrayList<>();
		for (Component line : net.minecraft.client.gui.screens.Screen.getTooltipFromItem(Minecraft.getInstance(), stack)) components.add(ClientTooltipComponent.create(line.getVisualOrderText()));
		components.add(new ClientTooltipComponent() {
			@Override public int getHeight(Font font) { return 58; }
			@Override public int getWidth(Font font) { return 164; }
			@Override public void extractImage(Font font, int x, int y, int w, int h, GuiGraphicsExtractor g) {
				g.fill(x, y, x + 164, y + 56, 0xFF29252E);
				for (int i = 0; i < 27; i++) {
					int sx = x + 1 + i % 9 * 18, sy = y + 1 + i / 9 * 18;
					g.fill(sx, sy, sx + 18, sy + 18, 0xFF665870); g.fill(sx + 1, sy + 1, sx + 17, sy + 17, 0xFF211C27);
					if (i < slots.size() && !slots.get(i).isEmpty()) { g.item(slots.get(i), sx + 1, sy + 1); g.itemDecorations(font, slots.get(i), sx + 1, sy + 1); }
				}
			}
		});
		return components;
	}
}
