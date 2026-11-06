package dev.patina.client;

import com.google.gson.Gson;

/** Run before any world/registry bootstrap: previews must not construct a nonempty ItemStack. */
public final class ArmorPreviewSmokeTest {
	public static void main(String[] args) throws Exception {
		PatinaHud.ArmorPiece sample = PatinaHud.ArmorPiece.sample("diamond_helmet", 18, 363);
		if (!sample.present() || sample.stack() != null || sample.remaining() != 18 || sample.maximum() != 363) throw new AssertionError("Preview needs detached example data");
		if (!sample.icon().toString().equals("minecraft:textures/item/diamond_helmet.png")) throw new AssertionError("Incorrect preview texture");
		if (PatinaHud.ArmorPiece.sample(null, 0, 0).present()) throw new AssertionError("Empty preview slot must remain empty");
		PatinaConfig config = new Gson().fromJson("{\"mods\":{\"fps\":{\"enabled\":false,\"position\":\"center\",\"x\":12,\"y\":-8},\"armor\":{\"display\":null,\"warning_threshold\":999},\"keystrokes\":{\"show_space\":false}}}", PatinaConfig.class);
		var valid = PatinaConfig.class.getDeclaredMethod("valid");
		valid.setAccessible(true);
		valid.invoke(config);
		if (config.mods.fps.enabled || config.mods.fps.x != 12f || config.mods.fps.y != -8f) throw new AssertionError("Existing HUD placement changed");
		if (config.mods.keystrokes.show_space || !config.mods.keystrokes.show_shift) throw new AssertionError("Optional keys did not migrate");
		if (!config.mods.armor.display.equals("bar") || config.mods.armor.warning_threshold != 50) throw new AssertionError("Malformed armor options were not repaired");
		if (config.mods.all().size() != 7) throw new AssertionError("Armor module missing");
		if (config.mods.fullbright.enabled) throw new AssertionError("Full Bright must default to off for old configs");
		config.mods.fullbright.enabled = true;
		PatinaConfig reloaded = new Gson().fromJson(new Gson().toJson(config), PatinaConfig.class);
		valid.invoke(reloaded);
		if (!reloaded.mods.fullbright.enabled || reloaded.mods.fps.x != 12f) throw new AssertionError("Full Bright persistence changed HUD placement");
		if (reloaded.mods.all().size() != 7) throw new AssertionError("Full Bright must not appear in the HUD editor");
        PatinaConfig partial = new Gson().fromJson("{\"mods\":{\"speedometer\":{\"enabled\":true},\"ping\":{\"enabled\":true},\"fps\":{\"position\":null}}}", PatinaConfig.class);
        valid.invoke(partial);
        if (!partial.mods.speedometer.position.equals("bottom_left") || !partial.mods.ping.position.equals("top_left")) throw new AssertionError("Partial HUD settings must keep each module's anchor");
        if (reloaded.mods.catalogue().size() != 16) throw new AssertionError("Every client module must have a catalogue entry");
        if (reloaded.mods.zoom.key != 67 || reloaded.mods.perspective.key != 86) throw new AssertionError("Independent default key binds");
        if (PatinaKeyBindings.nativeCode(67) != com.mojang.blaze3d.platform.InputConstants.KEY_C || PatinaKeyBindings.sharedCode(com.mojang.blaze3d.platform.InputConstants.KEY_B) != 66 || PatinaKeyBindings.nativeCode(86) != com.mojang.blaze3d.platform.InputConstants.KEY_V) throw new AssertionError("SDL key translation");
        var keyMod = reloaded.mods.perspective; keyMod.enabled = true; keyMod.activation = "toggle";
        if (!PatinaUtilities.activation(keyMod, true, true, false) || !PatinaUtilities.activation(keyMod, true, false, true)
            || PatinaUtilities.activation(keyMod, true, true, true) || PatinaUtilities.activation(keyMod, false, true, true)) throw new AssertionError("Perspective activation modes");
        if (!PatinaUtilities.coordinates(123.9,45.8,-1422.2).equals("123 45 -1423")) throw new AssertionError("Death coordinates must floor negative positions");
        if (!PatinaHud.duration(-1).equals("∞") || !PatinaHud.roman(8).equals("VIII")) throw new AssertionError("Effect levels and infinite duration");
        System.out.println("PASS: preview before registry bootstrap; empty slot; old configuration migration; malformed armor options");
	}
}
