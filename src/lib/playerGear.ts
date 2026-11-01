import { BoxGeometry, Color, Group, InstancedMesh, LinearSRGBColorSpace, Matrix4, MeshStandardMaterial } from "three";
import { api } from "./api";

/** The grip of the 16×16 sword texture, where the fist closes. */
const SWORD_GRIP = { x: 3.5, y: 12.5 };
/** The shield's front face in its 64×64 texture, as the game's model cuts it. */
const SHIELD_FACE = { x: 1, y: 1, width: 12, height: 22 };

interface Pixel {
  x: number;
  y: number;
  rgb: [number, number, number];
}

async function pixelsOf(url: string, area?: { x: number; y: number; width: number; height: number }): Promise<Pixel[]> {
  const image = new Image();
  image.src = url;
  await image.decode();
  const scale = area ? image.width / 64 : 1;
  const left = (area?.x ?? 0) * scale;
  const top = (area?.y ?? 0) * scale;
  const width = (area?.width ?? image.width / scale) * scale;
  const height = (area?.height ?? image.height / scale) * scale;
  const canvas = document.createElement("canvas");
  canvas.width = width;
  canvas.height = height;
  const context = canvas.getContext("2d", { willReadFrequently: true })!;
  context.drawImage(image, -left, -top);
  const { data } = context.getImageData(0, 0, width, height);
  const pixels: Pixel[] = [];
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const i = (x + y * width) * 4;
      if (data[i + 3] > 127) pixels.push({ x, y, rgb: [data[i], data[i + 1], data[i + 2]] });
    }
  }
  return pixels;
}

/** One lit cube per pixel, like the game's own items: a single draw call for each.
 * Colours stay as the texture has them, as skins do (see SkinViewer). */
function voxels(pixels: Pixel[], origin: { x: number; y: number }): InstancedMesh {
  const mesh = new InstancedMesh(new BoxGeometry(1, 1, 1), new MeshStandardMaterial({ roughness: 0.8 }), pixels.length);
  const at = new Matrix4();
  const colour = new Color();
  pixels.forEach(({ x, y, rgb }, i) => {
    mesh.setMatrixAt(i, at.makeTranslation(x - origin.x, origin.y - y, 0));
    mesh.setColorAt(i, colour.setRGB(rgb[0] / 255, rgb[1] / 255, rgb[2] / 255, LinearSRGBColorSpace));
  });
  return mesh;
}

export interface Gear {
  sword: Group;
  shield: Group;
  dispose(): void;
}

/** The diamond sword and shield of the installed game; `null` until one is installed. */
export async function loadGear(): Promise<Gear | null> {
  const textures = await api.gearTextures().catch(() => null);
  if (!textures) return null;
  const swordMesh = voxels(await pixelsOf(textures.sword), SWORD_GRIP);
  // The texture runs corner to corner: stand the blade along the hand's axis.
  swordMesh.rotation.z = Math.PI / 4;
  const shieldMesh = voxels(await pixelsOf(textures.shield, SHIELD_FACE), { x: (SHIELD_FACE.width - 1) / 2, y: (SHIELD_FACE.height - 1) / 2 });
  const sword = new Group();
  const shield = new Group();
  sword.add(swordMesh);
  shield.add(shieldMesh);
  return {
    sword,
    shield,
    dispose() {
      for (const mesh of [swordMesh, shieldMesh]) {
        mesh.parent?.removeFromParent();
        mesh.geometry.dispose();
        (mesh.material as MeshStandardMaterial).dispose();
        mesh.dispose();
      }
    },
  };
}
