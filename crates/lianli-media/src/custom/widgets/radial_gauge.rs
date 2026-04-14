//! Annular (donut) progress gauge.
//!
//! Reservations (not yet implemented in the template struct):
//! - Separate `bg_corner_radius` / `value_corner_radius` for rounded arc ends
//!   on the background ring vs. the filled-value ring. Today both ends are
//!   hard-cut at `start_angle` / `start_angle + sweep`.
//! - Explicit `show_background` flag, mirroring `Speedometer::show_gauge`.
//!   Until that lands, setting `background_color`'s alpha to 0 effectively
//!   hides the bg ring (the sub-canvas is transparent underneath, so the
//!   alpha-0 writes are visual no-ops).

use super::super::helpers::{draw_annulus, range_color, unit_interval};
use image::{Rgba, RgbaImage};
use lianli_shared::media::SensorRange;

#[allow(clippy::too_many_arguments)]
pub(in super::super) fn draw(
    sub: &mut RgbaImage,
    value: f32,
    value_min: f32,
    value_max: f32,
    start_angle: f32,
    sweep_angle: f32,
    inner_radius_pct: f32,
    background_color: [u8; 4],
    ranges: &[SensorRange],
) {
    let (w, h) = (sub.width() as f32, sub.height() as f32);
    let center = (w / 2.0, h / 2.0);
    let r_outer = (w.min(h) / 2.0).max(1.0);
    let r_inner = (r_outer * inner_radius_pct.clamp(0.0, 0.99)).max(1.0);

    let bg = Rgba(background_color);
    draw_annulus(sub, center, r_inner, r_outer, start_angle, sweep_angle, bg);

    let u = unit_interval(value, value_min, value_max);
    let fill_sweep = sweep_angle * u;
    let color = range_color(ranges, u);
    if fill_sweep.abs() > 0.01 {
        draw_annulus(
            sub,
            center,
            r_inner,
            r_outer,
            start_angle,
            fill_sweep,
            color,
        );
    }
}
