//! The boot splash: a black screen with Patina's animation in the middle, then a short fade to
//! the menu. The animation runs from the first frame the window shows, however fast the
//! resources load: it reaches the whole cube, keeps looping while they load, and leaves only
//! while the cube is whole. The sheet is Patina's own (`patina_theme::SPLASH_PNG`); it replaces
//! the 1x1 atlas of the loading renderer, so the window shows it before the game's resources
//! are read.
use crate::hud::{self, HudBatch, rgba};
use std::time::{Duration, Instant};

const COLUMNS: u32 = 8;
const FRAMES: u32 = 64;
const FRAME_SIZE: f32 = 160.0;
const SHEET_SIZE: f32 = FRAME_SIZE * COLUMNS as f32;
/// One loop of the sheet.
const LOOP: Duration = Duration::from_millis(5120);
/// The animation's size on screen, in window pixels.
const SIDE: f32 = 240.0;
/// The part of each loop where the logo may start to fade: the blocks have settled into the
/// whole cube (frames 24 to 37 hold it) and it has been seen for a moment.
const CUBE_FROM: Duration = Duration::from_millis(2240);
const CUBE_UNTIL: Duration = Duration::from_millis(3040);
/// Even when the resources are quick, so the logo doesn't flash by.
const SHOWN_AT_LEAST: Duration = Duration::from_millis(2300);
const LOGO_FADE: Duration = Duration::from_millis(220);
const MENU_FADE: Duration = Duration::from_millis(320);

/// The sheet's pixels: RGBA, its width and its height.
pub fn sheet() -> crate::Result<(Vec<u8>, u32, u32)> {
    let (width, height, pixels) = hud::decode_bytes(patina_theme::SPLASH_PNG)?;
    Ok((pixels, width, height))
}

/// What the frame shows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Stage {
    /// The animation, at this opacity.
    Logo(f32),
    /// The logo is gone: time to bring in the menu's resources behind the black.
    Swap,
    /// The menu, under black of this opacity.
    Reveal(f32),
    Done,
}

/// Where the splash is. Its clock is the wall clock: a slow frame skips animation, not time.
#[derive(Clone, Copy, Debug, Default)]
pub struct Splash {
    /// The first frame drawn. Opening the window and the GPU comes first, and can take seconds.
    started: Option<Instant>,
    /// When the logo fades, frozen on the frame due then. Set once the title screen can be drawn.
    leaving: Option<Instant>,
    /// The menu's resources are in: the black fades from here.
    revealing: Option<Instant>,
}

impl Splash {
    pub fn new() -> Self {
        Self::default()
    }
    /// A frame is about to be drawn. The first call starts the animation.
    pub fn begin(&mut self, now: Instant) {
        self.started.get_or_insert(now);
    }
    /// The title screen can be drawn. The first call after `begin` picks the moment to leave.
    pub fn menu_ready(&mut self, now: Instant) {
        if let Some(started) = self.started {
            self.leaving
                .get_or_insert_with(|| started + leaving_after(now - started));
        }
    }
    /// `Stage::Swap` is over: the menu is behind the black.
    pub fn swapped(&mut self, now: Instant) {
        self.revealing.get_or_insert(now);
    }
    pub fn stage(&self, now: Instant) -> Stage {
        if let Some(revealing) = self.revealing {
            let left = now.saturating_duration_since(revealing);
            return if left < MENU_FADE {
                Stage::Reveal(1.0 - left.as_secs_f32() / MENU_FADE.as_secs_f32())
            } else {
                Stage::Done
            };
        }
        let Some(leaving) = self.leaving else {
            return Stage::Logo(1.0);
        };
        let gone = now.saturating_duration_since(leaving);
        if gone < LOGO_FADE {
            Stage::Logo(1.0 - gone.as_secs_f32() / LOGO_FADE.as_secs_f32())
        } else {
            Stage::Swap
        }
    }
    /// Draws the frame due at `now` at `opacity`, over the black clear, using only the sheet.
    pub fn draw(&self, batch: &mut HudBatch, now: Instant, opacity: f32) {
        let frame = self.started.map_or(0, |started| {
            let until = self.leaving.map_or(now, |leaving| now.min(leaving));
            frame_at(until.saturating_duration_since(started))
        });
        let (u, v) = (
            (frame % COLUMNS) as f32 * FRAME_SIZE,
            (frame / COLUMNS) as f32 * FRAME_SIZE,
        );
        // Half a texel in: linear filtering must not reach the frame next to this one.
        let (u0, v0) = ((u + 0.5) / SHEET_SIZE, (v + 0.5) / SHEET_SIZE);
        let (u1, v1) = (
            (u + FRAME_SIZE - 0.5) / SHEET_SIZE,
            (v + FRAME_SIZE - 0.5) / SHEET_SIZE,
        );
        let side = SIDE.min(batch.width.min(batch.height) * batch.scale * 0.6) / batch.scale;
        let (x, y) = (
            ((batch.width - side) * batch.scale / 2.0).floor() / batch.scale,
            ((batch.height - side) * batch.scale / 2.0).floor() / batch.scale,
        );
        let alpha = (opacity.clamp(0.0, 1.0) * 255.0).round() as u8;
        batch.quad(
            [[x, y], [x, y + side], [x + side, y + side], [x + side, y]],
            [[u0, v0], [u0, v1], [u1, v1], [u1, v0]],
            rgba(255, 255, 255, alpha),
        );
    }
}

/// How long after the start the logo fades, the menu being ready `ready` after it: the first
/// moment, past the minimum, inside a loop's stretch of whole cube.
fn leaving_after(ready: Duration) -> Duration {
    let at = ready.max(SHOWN_AT_LEAST);
    let into = Duration::from_millis((at.as_millis() % LOOP.as_millis()) as u64);
    if into < CUBE_FROM {
        at + (CUBE_FROM - into)
    } else if into < CUBE_UNTIL {
        at
    } else {
        at + (LOOP - into) + CUBE_FROM
    }
}

/// The sheet's frame after `elapsed`, looping.
fn frame_at(elapsed: Duration) -> u32 {
    (elapsed.as_millis() * u128::from(FRAMES) / LOOP.as_millis() % u128::from(FRAMES)) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    const MS: fn(u64) -> Duration = Duration::from_millis;

    #[test]
    fn the_sheet_is_the_size_the_splash_expects() {
        let (pixels, width, height) = sheet().unwrap();
        assert_eq!((width, height), (1280, 1280));
        assert_eq!(pixels.len(), 1280 * 1280 * 4);
    }

    #[test]
    fn frames_follow_the_wall_clock_and_loop() {
        assert_eq!(frame_at(MS(0)), 0);
        assert_eq!(frame_at(MS(80)), 1);
        assert_eq!(frame_at(MS(5119)), 63);
        assert_eq!(frame_at(MS(5120)), 0);
        assert_eq!(frame_at(MS(5200 + 5120 * 100)), 1);
    }

    /// The frame drawn at `at`, read back from the quad's texture coordinates.
    fn drawn_frame(splash: &Splash, at: Instant) -> u32 {
        let mut batch = HudBatch::new([1280, 720]);
        splash.draw(&mut batch, at, 1.0);
        let [u, v] = batch.vertices[0].uv;
        let (column, row) = (u * SHEET_SIZE / FRAME_SIZE, v * SHEET_SIZE / FRAME_SIZE);
        row.floor() as u32 * COLUMNS + column.floor() as u32
    }

    #[test]
    fn the_sheet_holds_the_whole_cube_where_the_splash_leaves() {
        let (pixels, ..) = sheet().unwrap();
        let drawn = |frame: u32| {
            let (fx, fy) = (
                (frame % COLUMNS) as usize * 160,
                (frame / COLUMNS) as usize * 160,
            );
            let mut cells = Vec::with_capacity(160 * 160);
            for y in fy..fy + 160 {
                for x in fx..fx + 160 {
                    cells.push(pixels[(y * 1280 + x) * 4 + 3] > 128);
                }
            }
            cells
        };
        let whole = drawn(frame_at(CUBE_FROM));
        for ms in (2240..3040).step_by(40) {
            assert!(drawn(frame_at(MS(ms))) == whole, "frame at {ms} ms");
        }
    }

    #[test]
    fn it_leaves_only_on_the_whole_cube() {
        let into = |t: Duration| (t.as_millis() % LOOP.as_millis()) as u64;
        for ready in (0..30_000).step_by(37) {
            let at = leaving_after(MS(ready));
            assert!(at >= MS(ready).max(SHOWN_AT_LEAST), "ready at {ready}");
            assert!((2240..3040).contains(&into(at)), "ready at {ready}: {at:?}");
            assert!(at < MS(ready).max(SHOWN_AT_LEAST) + LOOP);
        }
        // Quick resources: the first hold, not before the minimum.
        assert_eq!(leaving_after(MS(100)), SHOWN_AT_LEAST);
        // Ready while the cube comes apart: the next loop's hold.
        assert_eq!(leaving_after(MS(4000)), LOOP + CUBE_FROM);
        // Ready inside a hold: at once.
        assert_eq!(leaving_after(LOOP + MS(2500)), LOOP + MS(2500));
    }

    #[test]
    fn its_clock_starts_at_the_first_frame() {
        let boot = Instant::now();
        let mut splash = Splash::new();
        // The window and the GPU took two seconds and the resources were ready first.
        splash.menu_ready(boot + MS(2000));
        assert_eq!(splash.stage(boot + MS(2000)), Stage::Logo(1.0));
        splash.begin(boot + MS(2000));
        splash.menu_ready(boot + MS(2010));
        assert_eq!(drawn_frame(&splash, boot + MS(2000)), 0);
        assert_eq!(splash.stage(boot + MS(4299)), Stage::Logo(1.0));
    }

    #[test]
    fn it_waits_for_the_menu_and_then_fades_both_ways() {
        let start = Instant::now();
        let mut splash = Splash::new();
        splash.begin(start);
        assert_eq!(splash.stage(start + MS(60_000)), Stage::Logo(1.0));
        // Ready early: the logo stays its minimum time first.
        splash.menu_ready(start + MS(100));
        assert_eq!(splash.stage(start + MS(2299)), Stage::Logo(1.0));
        let Stage::Logo(half) = splash.stage(start + MS(2410)) else {
            panic!("the logo should be fading");
        };
        assert!((half - 0.5).abs() < 0.01);
        // The fade holds the frame it began on: the cube, however long it takes.
        assert_eq!(drawn_frame(&splash, start + MS(2500)), frame_at(MS(2300)));
        let swap = start + MS(2520);
        assert_eq!(splash.stage(swap), Stage::Swap);
        splash.swapped(swap);
        assert_eq!(splash.stage(swap), Stage::Reveal(1.0));
        assert_eq!(splash.stage(swap + MS(320)), Stage::Done);
    }

    #[test]
    fn the_logo_is_centred_at_240_window_pixels_and_fits_small_windows() {
        let splash = Splash::new();
        for (size, side) in [
            ([1280, 720], 240.0),
            ([1920, 1080], 240.0),
            ([320, 240], 144.0),
        ] {
            let mut batch = HudBatch::new(size);
            splash.draw(&mut batch, Instant::now(), 1.0);
            let (x0, y0) = (batch.vertices[0].pos[0], batch.vertices[0].pos[1]);
            let (x1, y1) = (batch.vertices[2].pos[0], batch.vertices[2].pos[1]);
            assert!(((x1 - x0) * batch.scale - side).abs() < 0.01);
            assert!(((x0 + x1) * batch.scale / 2.0 - size[0] as f32 / 2.0).abs() <= 1.0);
            assert!(((y0 + y1) * batch.scale / 2.0 - size[1] as f32 / 2.0).abs() <= 1.0);
        }
    }
}
