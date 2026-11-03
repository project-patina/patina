package dev.patina.client;

import com.mojang.blaze3d.platform.NativeImage;
import java.io.FileNotFoundException;
import java.io.IOException;
import java.io.InputStream;
import net.minecraft.client.renderer.texture.MipmapStrategy;
import net.minecraft.client.renderer.texture.ReloadableTexture;
import net.minecraft.client.renderer.texture.TextureContents;
import net.minecraft.client.resources.metadata.texture.TextureMetadataSection;
import net.minecraft.server.packs.resources.ResourceManager;

/**
 * The overlay is on screen before the first resource reload, so like the texture it replaces
 * this one cannot go through the resource manager: it reads straight from the mod jar.
 */
final class SplashTexture extends ReloadableTexture {
	private static final String PATH = "/assets/patina/textures/gui/splash.png";

	SplashTexture() {
		super(PatinaSplash.TEXTURE);
	}

	@Override
	public TextureContents loadContents(ResourceManager resources) throws IOException {
		try (InputStream stream = SplashTexture.class.getResourceAsStream(PATH)) {
			if (stream == null) {
				throw new FileNotFoundException(PATH);
			}
			return new TextureContents(NativeImage.read(stream), new TextureMetadataSection(true, true, MipmapStrategy.MEAN, 0f));
		}
	}
}
