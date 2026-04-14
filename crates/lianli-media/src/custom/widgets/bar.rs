//! Vertical/horizontal fill bar.
//!
//! `corner_radius` is currently reserved (rounded corners not yet implemented).

use super::super::helpers::{range_color, unit_interval};
use image::{Rgba, RgbaImage};
use imageproc::drawing::draw_filled_rect_mut;
use imageproc::rect::Rect;
use lianli_shared::media::SensorRange;

#[allow(clippy::too_many_arguments)]
pub(in super::super) fn draw(
    sub: &mut RgbaImage,
    value: f32,
    value_min: f32,
    value_max: f32,
    background_color: [u8; 4],
    _corner_radius: f32,
    ranges: &[SensorRange],
    is_vertical: bool,
) {
    let (w, h) = (sub.width(), sub.height());
    let bg = Rgba(background_color);
    draw_filled_rect_mut(sub, Rect::at(0, 0).of_size(w, h), bg);

    let u = unit_interval(value, value_min, value_max);
    let color = range_color(ranges, u);
    if u <= 0.0 {
        return;
    }
    if is_vertical {
        let fill_h = ((h as f32) * u).round() as u32;
        if fill_h == 0 {
            return;
        }
        draw_filled_rect_mut(
            sub,
            Rect::at(0, (h - fill_h) as i32).of_size(w, fill_h),
            color,
        );
    } else {
        let fill_w = ((w as f32) * u).round() as u32;
        if fill_w == 0 {
            return;
        }
        draw_filled_rect_mut(sub, Rect::at(0, 0).of_size(fill_w, h), color);
    }
}
