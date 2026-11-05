package dev.patina.client;

import com.google.gson.Gson;
import com.mojang.blaze3d.platform.InputConstants;
import java.nio.file.Files;
import java.time.Instant;
import java.time.ZoneId;
import java.time.format.DateTimeFormatter;
import net.minecraft.client.CameraType;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.player.LocalPlayer;
import net.minecraft.network.chat.Component;
import net.minecraft.world.phys.Vec3;

/** Transient client state. None of these utilities changes server weather or player rotation. */
public final class PatinaUtilities {
	private static final Gson JSON = new Gson();
	private static Vec3 lastPosition;
	private static LocalPlayer lastPlayer;
	private static boolean lastZoomKey, lastPerspectiveKey;
	private static boolean zoomActive, perspectiveActive;
	private static boolean zoomPress, perspectivePress;
	private static CameraType previousCamera;
	private static float cameraYaw, cameraPitch, zoomFactor = 4f, currentZoom = 1f;
	private static long lastZoomFrame;
	private static double speed;
	private static Death death;
	public record Death(String coordinates, String dimension, String time) { }
	private PatinaUtilities() { }

	public static void tick() {
		Minecraft mc = Minecraft.getInstance();
		var mods = PatinaConfig.get().mods;
		boolean playing = mc.player != null && mc.gui.screen() == null;
		boolean zoomKey = playing && PatinaKeyBindings.nativeCode(mods.zoom.key) >= 0 && InputConstants.isKeyDown(PatinaKeyBindings.nativeCode(mods.zoom.key));
		boolean perspectiveKey = playing && PatinaKeyBindings.nativeCode(mods.perspective.key) >= 0 && InputConstants.isKeyDown(PatinaKeyBindings.nativeCode(mods.perspective.key));
		boolean nextZoom = activation(mods.zoom, playing, "toggle".equals(mods.zoom.activation) ? zoomPress : zoomKey, false, zoomActive);
		boolean nextPerspective = activation(mods.perspective, playing, "toggle".equals(mods.perspective.activation) ? perspectivePress : perspectiveKey, false, perspectiveActive);
		zoomPress = perspectivePress = false;
		if (nextZoom && !zoomActive) zoomFactor = mods.zoom.factor;
		zoomActive = nextZoom; lastZoomKey = zoomKey; lastPerspectiveKey = perspectiveKey;
		if (nextPerspective && !perspectiveActive) {
			previousCamera = mc.options.getCameraType(); cameraYaw = mc.player.getYRot(); cameraPitch = mc.player.getXRot();
			mc.options.setCameraType(CameraType.THIRD_PERSON_BACK);
		} else if (!nextPerspective && perspectiveActive && previousCamera != null) {
			mc.options.setCameraType(previousCamera); previousCamera = null;
		}
		perspectiveActive = nextPerspective;
		if (mc.player != null && mc.player == lastPlayer && lastPosition != null) {
			Vec3 delta = mc.player.position().subtract(lastPosition);
			speed = delta.lengthSqr() > 4096 ? 0 : Math.hypot(delta.x, delta.z) * 20;
		} else speed = 0;
		lastPlayer = mc.player; lastPosition = mc.player == null ? null : mc.player.position();
	}
	public static void keyPressed(int shared) {
		Minecraft mc = Minecraft.getInstance();
		if (shared < 0 || mc.player == null || mc.gui.screen() != null) return;
		var mods = PatinaConfig.get().mods;
		if (mods.zoom.enabled && "toggle".equals(mods.zoom.activation) && mods.zoom.key == shared) zoomPress = !zoomPress;
		if (mods.perspective.enabled && "toggle".equals(mods.perspective.activation) && mods.perspective.key == shared) perspectivePress = !perspectivePress;
	}
	public static boolean activation(PatinaConfig.UtilityMod mod, boolean playing, boolean down, boolean previous, boolean active) {
		if (!mod.enabled || !playing) return false;
		return "toggle".equals(mod.activation) ? (down && !previous ? !active : active) : down;
	}
	public static boolean perspective() { return perspectiveActive; }
	public static float yaw() { return cameraYaw; }
	public static float pitch() { return cameraPitch; }
	public static void turn(LocalPlayer player, double dx, double dy) {
		if (perspectiveActive) {
			cameraYaw += (float) (dx * 0.15); cameraPitch = Math.clamp(cameraPitch + (float) (dy * 0.15), -90f, 90f);
		} else {
			double sensitivity = zoomActive ? PatinaConfig.get().mods.zoom.sensitivity : 1;
			player.turn(dx * sensitivity, dy * sensitivity);
		}
	}
	public static boolean scrollZoom(double amount) {
		if (!zoomActive || amount == 0 || Minecraft.getInstance().gui.screen() != null) return false;
		zoomFactor = Math.clamp((float) (zoomFactor * Math.pow(1.15, amount)), 1.5f, 20f); return true;
	}
	public static float zoomFov(float fov) {
		long now = System.nanoTime(); float dt = lastZoomFrame == 0 ? 1f : Math.min(0.1f, (now - lastZoomFrame) / 1e9f); lastZoomFrame = now;
		float target = zoomActive ? zoomFactor : 1f;
		if (PatinaConfig.get().mods.zoom.smooth) currentZoom += (target - currentZoom) * (1f - (float)Math.exp(-dt * 16));
		else currentZoom = target;
		if (Math.abs(currentZoom - target) < 0.001f) currentZoom = target;
		return fov / currentZoom;
	}
	public static double speed() { return speed; }
	public static int ping() {
		Minecraft mc = Minecraft.getInstance();
		if (mc.player == null || mc.getConnection() == null) return -1;
		var info = mc.getConnection().getPlayerInfo(mc.player.getUUID()); return info == null ? -1 : info.getLatency();
	}
	public static String coordinates(double x, double y, double z) { return (int)Math.floor(x) + " " + (int)Math.floor(y) + " " + (int)Math.floor(z); }
	public static void recordDeath(LocalPlayer player) {
		if (!PatinaConfig.get().mods.death_info.enabled || player == null) return;
		String coordinates = coordinates(player.getX(), player.getY(), player.getZ());
		death = new Death(coordinates, player.level().dimension().identifier().toString(), Instant.now().toString());
		Minecraft.getInstance().keyboardHandler.setClipboard(coordinates);
		try { Files.createDirectories(PatinaConfig.folder()); Files.writeString(PatinaConfig.folder().resolve("last-death.json"), JSON.toJson(death)); }
		catch (java.io.IOException e) { PatinaClient.LOGGER.warn("Could not save last death", e); }
	}
	public static void drawDeath(GuiGraphicsExtractor g, int width, int height) {
		if (!PatinaConfig.get().mods.death_info.enabled || death == null) return;
		var font = Minecraft.getInstance().font;
		String time = DateTimeFormatter.ofPattern("yyyy-MM-dd HH:mm:ss").withZone(ZoneId.systemDefault()).format(Instant.parse(death.time));
		String[] lines = { death.dimension, time, death.coordinates + "  —  " + PatinaLang.get("patina.death.copied") };
		int y = height / 4 + 126;
		for (int i = 0; i < lines.length; i++) g.text(font, lines[i], (width - font.width(lines[i])) / 2, y + i * 11, i == 2 ? 0xFF6FDCC3 : 0xFFE8F0EE);
	}
}
