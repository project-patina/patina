package dev.patina.client;

import java.util.HashMap;
import java.util.Map;

/** client.json uses shared GLFW codes; Minecraft 26.3's keyboard uses SDL scancodes. */
public final class PatinaKeyBindings {
	private static final Map<Integer, Integer> SDL = new HashMap<>();
	private static final Map<Integer, Integer> SHARED = new HashMap<>();
	static {
		for (int i = 0; i < 26; i++) pair(65 + i, 4 + i);
		for (int i = 0; i < 9; i++) pair(49 + i, 30 + i);
		pair(48, 39);
		for (int i = 0; i < 12; i++) pair(290 + i, 58 + i);
		for (int i = 0; i < 12; i++) pair(302 + i, 104 + i);
		for (int i = 1; i <= 9; i++) pair(320 + i, 88 + i);
		int[][] special = {{32,44},{39,52},{44,54},{45,45},{46,55},{47,56},{59,51},{61,46},{91,47},{92,49},{93,48},{96,53},
			{256,41},{257,40},{258,43},{259,42},{260,73},{261,76},{262,79},{263,80},{264,81},{265,82},{266,75},{267,78},{268,74},{269,77},
			{280,57},{281,71},{282,83},{283,70},{284,72},{320,98},{330,99},{331,84},{332,85},{333,86},{334,87},{335,88},{336,103},
			{340,225},{341,224},{342,226},{343,227},{344,229},{345,228},{346,230},{347,231},{348,101}};
		for (int[] codes : special) pair(codes[0], codes[1]);
	}
	private PatinaKeyBindings() { }
	private static void pair(int shared, int nativeCode) { SDL.put(shared, nativeCode); SHARED.put(nativeCode, shared); }
	public static int nativeCode(int shared) { return SDL.getOrDefault(shared, -1); }
	public static int sharedCode(int nativeCode) { return SHARED.getOrDefault(nativeCode, -1); }
}
