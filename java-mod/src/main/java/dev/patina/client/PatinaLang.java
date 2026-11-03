package dev.patina.client;

import com.google.gson.Gson;
import com.google.gson.JsonParseException;
import com.google.gson.reflect.TypeToken;
import java.io.IOException;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;
import java.util.HashMap;
import java.util.Map;
import net.minecraft.client.Minecraft;
import net.minecraft.network.chat.Component;

/**
 * Without Fabric API the game never sees the assets of a mod, so like the textures the strings
 * are read straight from the jar. Other languages fall back to English.
 */
public final class PatinaLang {
	private static final Map<String, String> FALLBACK = read("en_us");
	private static String loadedCode = "";
	private static Map<String, String> strings = FALLBACK;

	private PatinaLang() {
	}

	public static String get(String key) {
		String code = Minecraft.getInstance().getLanguageManager().getSelected();
		if (!code.equals(loadedCode)) {
			loadedCode = code;
			strings = code.equals("en_us") ? FALLBACK : read(code);
		}
		String value = strings.get(key);
		return value != null ? value : FALLBACK.getOrDefault(key, key);
	}

	public static Component text(String key) {
		return Component.literal(get(key));
	}

	private static Map<String, String> read(String code) {
		try (InputStream stream = PatinaLang.class.getResourceAsStream("/assets/patina/lang/" + code + ".json")) {
			if (stream != null) {
				return new Gson().fromJson(new InputStreamReader(stream, StandardCharsets.UTF_8), new TypeToken<Map<String, String>>() { });
			}
		} catch (IOException | JsonParseException e) {
			PatinaClient.LOGGER.warn("Could not read the {} strings", code, e);
		}
		return new HashMap<>();
	}
}
