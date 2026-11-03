//! The rows Video Settings shows beyond vanilla's: far terrain, resolution scale, anti-aliasing,
//! HDR output, battery mode, the graphics API and the FPS overlay (`Opt::Graphics`). They sit in
//! vanilla's list and draw with vanilla's widgets; this file says what each row reads and writes
//! in `graphics::Graphics`.
use crate::{
    graphics::{FarDetail, GraphicsApi, Overlay, Upscaling},
    settings::Settings,
    text::Text,
};

/// A row of `patina-graphics.txt`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gfx {
    LodDistance,
    FarDetail,
    ResolutionScale,
    Antialiasing,
    Hdr,
    HdrBlackLevel,
    HdrPeak,
    HdrPaperWhite,
    HdrUiBrightness,
    BatteryMode,
    GraphicsApi,
    Overlay,
}

fn tr(key: &str) -> Text {
    Text::translate(key, &[])
}
fn on_off(on: bool) -> Text {
    tr(if on { "options.on" } else { "options.off" })
}
fn literal(value: impl std::fmt::Display) -> Text {
    Text::literal(value.to_string())
}
fn nits(value: u32, auto: bool) -> Text {
    let key = if auto {
        "patina.options.nits_auto"
    } else {
        "patina.options.nits"
    };
    Text::translate(key, &[literal(value)])
}

impl Gfx {
    /// The translation key of the row's name; its tooltip is this plus `.tooltip`.
    pub fn caption(self) -> &'static str {
        match self {
            Gfx::LodDistance => "patina.options.far_terrain",
            Gfx::FarDetail => "patina.options.far_detail",
            Gfx::ResolutionScale => "patina.options.resolution_scale",
            Gfx::Antialiasing => "patina.options.antialiasing",
            Gfx::Hdr => "patina.options.hdr",
            Gfx::HdrBlackLevel => "patina.options.hdr_black_level",
            Gfx::HdrPeak => "patina.options.hdr_peak",
            Gfx::HdrPaperWhite => "patina.options.hdr_paper_white",
            Gfx::HdrUiBrightness => "patina.options.hdr_ui_brightness",
            Gfx::BatteryMode => "patina.options.battery_mode",
            Gfx::GraphicsApi => "patina.options.graphics_api",
            Gfx::Overlay => "patina.options.fps_overlay",
        }
    }
    /// A slider's position (0..1), or `None` for a cycle button.
    pub fn slider(self, s: &Settings) -> Option<f32> {
        let g = &s.patina;
        match self {
            Gfx::ResolutionScale => Some((g.resolution_scale as f32 - 50.0) / 150.0),
            // Auto, then 80..480 by 10.
            Gfx::HdrPaperWhite => Some(match g.hdr_paper_white {
                0 => 0.0,
                n => (n as f32 - 70.0) / 410.0,
            }),
            // Auto, then 200..2000 by 50.
            Gfx::HdrPeak => Some(match g.hdr_peak {
                0 => 0.0,
                n => (n as f32 - 150.0) / 1850.0,
            }),
            Gfx::HdrBlackLevel => Some(g.hdr_black_level as f32 / 100.0),
            Gfx::HdrUiBrightness => Some((g.hdr_ui_brightness as f32 - 25.0) / 75.0),
            _ => None,
        }
    }
    /// The row's value, for `options.generic_value`.
    pub fn value(self, s: &Settings) -> Text {
        let g = &s.patina;
        match self {
            Gfx::LodDistance => match g.lod_distance {
                0 => tr("options.off"),
                n => Text::translate("options.chunks", &[literal(n)]),
            },
            Gfx::FarDetail => tr(match g.far_detail {
                FarDetail::Full => "patina.options.far_detail.full",
                FarDetail::Reduced => "patina.options.far_detail.reduced",
            }),
            Gfx::ResolutionScale => literal(format!("{}%", g.resolution_scale)),
            Gfx::Antialiasing => match (g.fxaa, g.taa) {
                (_, true) => literal("TAA"),
                (true, false) => literal("FXAA"),
                (false, false) => tr("options.off"),
            },
            Gfx::Hdr => on_off(g.hdr),
            Gfx::HdrPaperWhite => match g.hdr_paper_white {
                0 => nits(g.paper_white_nits(), true),
                n => nits(n, false),
            },
            Gfx::HdrPeak => match g.hdr_peak {
                0 => nits(g.peak_nits(), true),
                n => nits(n, false),
            },
            Gfx::HdrBlackLevel => Text::translate(
                "patina.options.nits",
                &[literal(format!("{:.2}", g.hdr_black_level as f32 / 100.0))],
            ),
            Gfx::HdrUiBrightness => literal(format!("{}%", g.hdr_ui_brightness)),
            Gfx::BatteryMode => on_off(g.battery),
            Gfx::GraphicsApi => match g.api {
                GraphicsApi::Auto => tr("patina.options.glyphs.auto"),
                api => literal(api.label()),
            },
            Gfx::Overlay => match g.overlay {
                Overlay::Off => tr("options.off"),
                Overlay::Fps => literal("FPS"),
                Overlay::Graph => tr("patina.options.fps_overlay.graph"),
            },
        }
    }
    pub fn set_slider(self, s: &mut Settings, v: f32) {
        let g = &mut s.patina;
        match self {
            Gfx::ResolutionScale => g.resolution_scale = (50.0 + (v * 30.0).round() * 5.0) as u32,
            Gfx::HdrPaperWhite => {
                g.hdr_paper_white = match (v * 41.0).round() as u32 {
                    0 => 0,
                    step => 70 + step * 10,
                }
            }
            Gfx::HdrPeak => {
                g.hdr_peak = match (v * 37.0).round() as u32 {
                    0 => 0,
                    step => 150 + step * 50,
                }
            }
            Gfx::HdrBlackLevel => g.hdr_black_level = (v * 100.0).round() as u32,
            Gfx::HdrUiBrightness => g.hdr_ui_brightness = (25.0 + (v * 15.0).round() * 5.0) as u32,
            _ => {}
        }
    }
    pub fn cycle(self, s: &mut Settings, back: bool) {
        fn step<T: Copy + PartialEq>(all: &[T], now: T, back: bool) -> T {
            let i = all.iter().position(|x| *x == now).unwrap_or(0);
            let n = all.len();
            all[if back { (i + n - 1) % n } else { (i + 1) % n }]
        }
        let g = &mut s.patina;
        match self {
            Gfx::LodDistance => {
                g.lod_distance = step(&[0, 64, 128, 256, 512, 1024], g.lod_distance, back)
            }
            Gfx::FarDetail => {
                g.far_detail = step(&[FarDetail::Full, FarDetail::Reduced], g.far_detail, back)
            }
            // Off, FXAA, TAA.
            Gfx::Antialiasing => {
                let i = if g.taa { 2 } else { usize::from(g.fxaa) };
                let i = (i + if back { 2 } else { 1 }) % 3;
                (g.fxaa, g.taa) = (i == 1, i == 2);
                g.upscaling = Upscaling::Native;
            }
            Gfx::Hdr => g.hdr = !g.hdr,
            Gfx::BatteryMode => g.battery = !g.battery,
            Gfx::GraphicsApi => g.api = step(GraphicsApi::choices(), g.api, back),
            Gfx::Overlay => {
                g.overlay = step(
                    &[Overlay::Off, Overlay::Fps, Overlay::Graph],
                    g.overlay,
                    back,
                )
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn far_terrain_cycles_without_changing_the_render_distance() {
        let mut s = Settings::parse("renderDistance:12\n");
        for expected in [64, 128, 256, 512, 1024, 0, 64] {
            Gfx::LodDistance.cycle(&mut s, false);
            assert_eq!(s.patina.lod_distance, expected);
            assert_eq!(s.render_distance, 12);
        }
        Gfx::LodDistance.cycle(&mut s, true);
        assert_eq!(s.patina.lod_distance, 0);
    }

    #[test]
    fn antialiasing_cycles_off_fxaa_taa() {
        let mut s = Settings::default();
        let state = |s: &Settings| (s.patina.fxaa, s.patina.taa);
        assert_eq!(state(&s), (false, false));
        Gfx::Antialiasing.cycle(&mut s, false);
        assert_eq!(state(&s), (true, false));
        Gfx::Antialiasing.cycle(&mut s, false);
        assert_eq!(state(&s), (false, true));
        Gfx::Antialiasing.cycle(&mut s, false);
        assert_eq!(state(&s), (false, false));
        Gfx::Antialiasing.cycle(&mut s, true);
        assert_eq!(state(&s), (false, true));
    }

    #[test]
    fn sliders_land_on_their_steps() {
        let mut s = Settings::default();
        Gfx::ResolutionScale.set_slider(&mut s, 1.0);
        assert_eq!(s.patina.resolution_scale, 200);
        assert_eq!(Gfx::ResolutionScale.slider(&s), Some(1.0));
        Gfx::ResolutionScale.set_slider(&mut s, 0.0);
        assert_eq!(s.patina.resolution_scale, 50);
        Gfx::HdrPeak.set_slider(&mut s, 0.0);
        assert_eq!(s.patina.hdr_peak, 0);
        Gfx::HdrPeak.set_slider(&mut s, 1.0);
        assert_eq!(s.patina.hdr_peak, 2000);
    }
}
