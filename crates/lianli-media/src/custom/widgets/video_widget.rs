//! Looping video frame overlay.

use super::super::helpers::{blit_with_opacity, ElapsedMs};
use super::WidgetState;
use image::RgbaImage;

pub(in super::super) fn draw(
    sub: &mut RgbaImage,
    state: &WidgetState,
    opacity: f32,
    elapsed_ms: ElapsedMs,
) {
    if let Some(frames) = &state.video_frames {
        if !frames.is_empty() {
            let dur_ms = state.video_frame_duration.as_millis().max(1) as u64;
            let idx = ((elapsed_ms.0 / dur_ms) as usize) % frames.len();
            blit_with_opacity(sub, &frames[idx], opacity);
        }
    }
}
