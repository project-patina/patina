//! The player on the title screen: the skin on a plain standing model, drawn
//! as HUD quads so it needs nothing of the world's model tables, which load
//! after the title appears. It stands as vanilla's `GuiGraphicsExtractor.skin`
//! puts a player in a box (`GuiSkinRenderer`), the way the Patina Fabric mod
//! draws it, with the same light.
use crate::hud::{HudBatch, Rect, rgba};
use glam::{Mat3, Vec3};

/// The atlas slot of a downloaded skin.
pub const SKIN_SPRITE: &str = "patina/title-skin";

/// One box: its corners in model pixels (y up, +z toward the viewer), the
/// corner of its unfolded texture, its size on the skin and how far the outer
/// layers stand off.
struct Part {
    min: [f32; 3],
    max: [f32; 3],
    uv: [f32; 2],
    size: [f32; 3],
    inflate: f32,
    head: bool,
}
const fn part(
    min: [f32; 3],
    max: [f32; 3],
    uv: [f32; 2],
    size: [f32; 3],
    inflate: f32,
    head: bool,
) -> Part {
    Part {
        min,
        max,
        uv,
        size,
        inflate,
        head,
    }
}
/// `PlayerModel.createMesh`'s boxes, in skin pixels with y up from the feet
/// and +z toward the viewer.
fn parts(slim: bool) -> [Part; 12] {
    let arm = if slim { 3.0 } else { 4.0 };
    let side = [arm, 12.0, 4.0];
    let drop = if slim { 0.5 } else { 0.0 };
    let (rl, rr) = ([-4.0 - arm, 12.0 - drop, -2.0], [-4.0, 24.0 - drop, 2.0]);
    let (ll, lr) = ([4.0, 12.0 - drop, -2.0], [4.0 + arm, 24.0 - drop, 2.0]);
    let leg = [4.0, 12.0, 4.0];
    let body = [8.0, 12.0, 4.0];
    let head = [8.0, 8.0, 8.0];
    let (hl, hh) = ([-4.0, 24.0, -4.0], [4.0, 32.0, 4.0]);
    [
        part(
            [-3.9, 0.0, -2.0],
            [0.1, 12.0, 2.0],
            [0.0, 16.0],
            leg,
            0.0,
            false,
        ),
        part(
            [-0.1, 0.0, -2.0],
            [3.9, 12.0, 2.0],
            [16.0, 48.0],
            leg,
            0.0,
            false,
        ),
        part(
            [-4.0, 12.0, -2.0],
            [4.0, 24.0, 2.0],
            [16.0, 16.0],
            body,
            0.0,
            false,
        ),
        part(rl, rr, [40.0, 16.0], side, 0.0, false),
        part(ll, lr, [32.0, 48.0], side, 0.0, false),
        part(hl, hh, [0.0, 0.0], head, 0.0, true),
        part(
            [-3.9, 0.0, -2.0],
            [0.1, 12.0, 2.0],
            [0.0, 32.0],
            leg,
            0.25,
            false,
        ),
        part(
            [-0.1, 0.0, -2.0],
            [3.9, 12.0, 2.0],
            [0.0, 48.0],
            leg,
            0.25,
            false,
        ),
        part(
            [-4.0, 12.0, -2.0],
            [4.0, 24.0, 2.0],
            [16.0, 32.0],
            body,
            0.25,
            false,
        ),
        part(rl, rr, [40.0, 32.0], side, 0.25, false),
        part(ll, lr, [48.0, 48.0], side, 0.25, false),
        part(hl, hh, [32.0, 0.0], head, 0.5, true),
    ]
}

/// How the model stands in its box: `GuiGraphicsExtractor.skin`'s scale (GUI
/// pixels per block), turn about x and y in degrees and pivot, and the head's
/// own turn (`ModelPart.yRot`, `xRot`) in radians.
#[derive(Clone, Copy, Debug)]
pub struct Pose {
    pub scale: f32,
    pub tilt: f32,
    pub turn: f32,
    pub pivot: f32,
    pub head_yaw: f32,
    pub head_pitch: f32,
}

/// `Lighting.Entry.PLAYER_SKIN`'s two lights, in the box's space (y down).
fn lights() -> [Vec3; 2] {
    [
        Vec3::new(0.2, -1.0, 1.0).normalize(),
        Vec3::new(-0.2, -1.0, 0.0).normalize(),
    ]
}
/// `minecraft_mix_light`: the light on a face with this normal.
fn light(normal: Vec3) -> f32 {
    let [a, b] = lights().map(|l| normal.dot(l).max(0.0));
    ((a + b) * 0.6 + 0.4).min(1.0)
}

/// Draw the model in the box `[x, y, w, h]` with the texture `skin`.
pub fn draw(batch: &mut HudBatch, skin: Rect, slim: bool, [bx, by, bw, bh]: [f32; 4], pose: Pose) {
    let s = pose.scale;
    // PictureInPictureRenderer: the origin at the box's bottom middle, z flipped.
    // GuiSkinRenderer: a turn about y, the model 1.601 blocks up, and the
    // whole tilted about x through the pivot.
    let yaw = Mat3::from_rotation_y((-pose.turn).to_radians());
    let tilt = Mat3::from_rotation_x(pose.tilt.to_radians());
    let head = Mat3::from_rotation_y(pose.head_yaw) * Mat3::from_rotation_x(pose.head_pitch);
    let pivot = Vec3::new(0.0, s * -pose.pivot, 0.0);
    let flip = Vec3::new(1.0, 1.0, -1.0);
    // Skin pixels, y up and +z to the viewer, to the model's own space.
    let model = |c: Vec3| Vec3::new(c.x, 24.0 - c.y, -c.z);
    let (du, dv) = ((skin.u1 - skin.u0) / 64.0, (skin.v1 - skin.v0) / 64.0);
    let uv = |x: f32, y: f32| [skin.u0 + x * du, skin.v0 + y * dv];
    let start = batch.vertices.len();
    for p in parts(slim) {
        let [x0, y0, z0] = p.min.map(|v| v - p.inflate);
        let [x1, y1, z1] = p.max.map(|v| v + p.inflate);
        let [u, v] = p.uv;
        let [w, h, d] = p.size;
        // Corners top left, top right, bottom right, bottom left as they
        // lie on the texture, the outward normal, and the texture rectangle.
        let faces: [([[f32; 3]; 4], Vec3, [f32; 4]); 6] = [
            (
                [[x0, y1, z1], [x1, y1, z1], [x1, y0, z1], [x0, y0, z1]],
                Vec3::Z,
                [u + d, v + d, w, h],
            ),
            (
                [[x1, y1, z0], [x0, y1, z0], [x0, y0, z0], [x1, y0, z0]],
                Vec3::NEG_Z,
                [u + d + w + d, v + d, w, h],
            ),
            (
                [[x0, y1, z0], [x0, y1, z1], [x0, y0, z1], [x0, y0, z0]],
                Vec3::NEG_X,
                [u, v + d, d, h],
            ),
            (
                [[x1, y1, z1], [x1, y1, z0], [x1, y0, z0], [x1, y0, z1]],
                Vec3::X,
                [u + d + w, v + d, d, h],
            ),
            (
                [[x0, y1, z0], [x1, y1, z0], [x1, y1, z1], [x0, y1, z1]],
                Vec3::Y,
                [u + d, v, w, d],
            ),
            (
                [[x0, y0, z1], [x1, y0, z1], [x1, y0, z0], [x0, y0, z0]],
                Vec3::NEG_Y,
                [u + d + w, v, w, d],
            ),
        ];
        let turned = |m: Vec3| if p.head { head * m } else { m };
        for (corners, normal, [tx, ty, tw, th]) in faces {
            // The normal as the shader sees it (the pose, not the tilt), and
            // as the screen does, for the faces turned away.
            let posed = (yaw * turned(model(normal) - model(Vec3::ZERO))) * flip;
            if (tilt * posed).z <= 0.0 {
                continue;
            }
            let tone = (light(posed) * 255.0) as u8;
            let point = |c: [f32; 3]| {
                let m = turned(model(Vec3::from(c))) / 16.0 + Vec3::new(0.0, -1.601, 0.0);
                let t = (yaw * m) * flip * s + Vec3::new(bw / 2.0, bh, 0.0);
                let t = tilt * (t - pivot) + pivot;
                [bx + t.x, by + t.y, t.z]
            };
            // A hair inside the texels, so the atlas's neighbours never bleed in.
            const IN: f32 = 0.02;
            batch.quad_3d(
                corners.map(point),
                [
                    uv(tx + IN, ty + IN),
                    uv(tx + tw - IN, ty + IN),
                    uv(tx + tw - IN, ty + th - IN),
                    uv(tx + IN, ty + th - IN),
                ],
                rgba(tone, tone, tone, 255),
            );
        }
    }
    batch.depth_since(start, [bx, by, bx + bw, by + bh]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slim_arms_are_a_pixel_narrower() {
        assert_eq!(parts(true)[3].size[0], 3.0);
        assert_eq!(parts(false)[3].size[0], 4.0);
    }
    #[test]
    fn the_front_is_lit_and_the_top_brightest() {
        let front = light(Vec3::Z);
        assert!((front - (0.4 + 0.6 / 2.04f32.sqrt())).abs() < 0.001);
        assert_eq!(light(Vec3::NEG_Y), 1.0);
        assert!(light(Vec3::X) < front);
    }
}
