//! Data model for the `MediaType::Custom` template system.
//!
//! A [`LcdTemplate`] is a reusable layout of widgets (gauges, labels, bars,
//! images, videos, CPU core strips) that a Custom LCD media type can render.
//! Templates are referenced from `LcdConfig.template_id`, stored in
//! `lcd_templates.json`, and authored via the in-app layout editor.
//!
//! Commit 1 ships the types and stub built-ins only. Widget rendering and the
//! full `CustomAsset` renderer land in Commit 2.

use crate::media::{SensorRange, SensorSourceConfig};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A reusable LCD layout template.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LcdTemplate {
    /// Stable unique id. Built-ins use reserved ids (e.g. `cooler-default`);
    /// user templates use a generated id.
    pub id: String,
    /// Display name, user-editable.
    pub name: String,
    /// Authoring canvas width in pixels.
    pub base_width: u32,
    /// Authoring canvas height in pixels.
    pub base_height: u32,
    /// Background fill or image.
    pub background: TemplateBackground,
    /// Widgets drawn in vector order (first = bottom).
    #[serde(default)]
    pub widgets: Vec<Widget>,
    /// Physical mount orientation the designer laid out for.
    #[serde(default)]
    pub orientation: TemplateOrientation,
}

/// Design-time layout intent. Distinct from `LcdConfig.orientation`, which is
/// the per-device physical rotation applied after template composition.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TemplateOrientation {
    /// Widgets authored against `base_width × base_height` as-is.
    #[default]
    Portrait,
    /// Widgets authored against a 90° CW rotation of the base canvas.
    Landscape,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum TemplateBackground {
    Color { rgb: [u8; 3] },
    Image { path: PathBuf },
}

impl Default for TemplateBackground {
    fn default() -> Self {
        Self::Color { rgb: [0, 0, 0] }
    }
}

/// A positioned, sized, optionally rotated widget inside a template.
///
/// `x` / `y` are the widget center in template-space (pixels against
/// `base_width × base_height`). `width` / `height` are the axis-aligned
/// bounding box before rotation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Widget {
    pub id: String,
    pub kind: WidgetKind,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    #[serde(default)]
    pub rotation: f32,
    #[serde(default = "default_true")]
    pub visible: bool,
    /// Sensor-bound widgets re-read at this interval. `None` = template default (1000 ms).
    #[serde(default)]
    pub update_interval_ms: Option<u64>,
    /// Video widget playback rate. `None` = template default.
    #[serde(default)]
    pub fps: Option<f32>,
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WidgetKind {
    Label {
        text: String,
        #[serde(default)]
        font: FontRef,
        font_size: f32,
        color: [u8; 3],
        #[serde(default)]
        align: TextAlign,
    },
    ValueText {
        source: SensorSourceConfig,
        /// Printf-style format string applied to the sensor value, e.g. `"{:.1}"`.
        #[serde(default = "default_value_format")]
        format: String,
        #[serde(default)]
        unit: String,
        #[serde(default)]
        font: FontRef,
        font_size: f32,
        color: [u8; 3],
        #[serde(default)]
        align: TextAlign,
    },
    RadialGauge {
        source: SensorSourceConfig,
        value_min: f32,
        value_max: f32,
        start_angle: f32,
        sweep_angle: f32,
        /// Inner radius as a fraction of `min(width, height) / 2`.
        #[serde(default = "default_inner_radius_pct")]
        inner_radius_pct: f32,
        background_color: [u8; 3],
        #[serde(default)]
        ranges: Vec<SensorRange>,
    },
    VerticalBar {
        source: SensorSourceConfig,
        value_min: f32,
        value_max: f32,
        background_color: [u8; 3],
        #[serde(default)]
        corner_radius: f32,
        #[serde(default)]
        ranges: Vec<SensorRange>,
    },
    HorizontalBar {
        source: SensorSourceConfig,
        value_min: f32,
        value_max: f32,
        background_color: [u8; 3],
        #[serde(default)]
        corner_radius: f32,
        #[serde(default)]
        ranges: Vec<SensorRange>,
    },
    Speedometer {
        source: SensorSourceConfig,
        value_min: f32,
        value_max: f32,
        start_angle: f32,
        sweep_angle: f32,
        needle_color: [u8; 3],
        tick_color: [u8; 3],
        #[serde(default = "default_tick_count")]
        tick_count: u32,
        background_color: [u8; 3],
    },
    CoreBars {
        #[serde(default)]
        orientation: BarOrientation,
        color_cold: [u8; 3],
        color_hot: [u8; 3],
        background_color: [u8; 3],
        #[serde(default)]
        show_labels: bool,
    },
    Image {
        path: PathBuf,
        #[serde(default = "default_opacity")]
        opacity: f32,
        #[serde(default)]
        fit: ImageFit,
    },
    Video {
        path: PathBuf,
        #[serde(default = "default_true")]
        loop_playback: bool,
        #[serde(default = "default_opacity")]
        opacity: f32,
        #[serde(default)]
        fit: ImageFit,
    },
}

fn default_value_format() -> String {
    "{:.0}".to_string()
}

fn default_inner_radius_pct() -> f32 {
    0.78
}

fn default_tick_count() -> u32 {
    10
}

fn default_opacity() -> f32 {
    1.0
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TextAlign {
    Left,
    #[default]
    Center,
    Right,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BarOrientation {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageFit {
    #[default]
    Stretch,
    Contain,
    Cover,
}

/// Reference to a font — either one of the bundled built-ins or a user file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FontRef {
    Builtin { font: BuiltinFont },
    File { path: PathBuf },
}

impl Default for FontRef {
    fn default() -> Self {
        Self::Builtin {
            font: BuiltinFont::VictorMono,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuiltinFont {
    VictorMono,
    JetBrainsMono,
    Digital7,
}

impl LcdTemplate {
    /// Basic structural validation — deep widget-specific checks live in the
    /// renderer (Commit 2). Returns an error message suitable for surfacing
    /// via `tracing::warn`.
    pub fn validate(&self) -> Result<(), String> {
        if self.id.trim().is_empty() {
            return Err("template id must not be empty".into());
        }
        if self.name.trim().is_empty() {
            return Err(format!("template '{}' name must not be empty", self.id));
        }
        if self.base_width == 0 || self.base_height == 0 {
            return Err(format!(
                "template '{}' base dimensions must be positive",
                self.id
            ));
        }
        for (i, w) in self.widgets.iter().enumerate() {
            if w.width <= 0.0 || w.height <= 0.0 {
                return Err(format!(
                    "template '{}' widget[{i}] '{}' has non-positive size",
                    self.id, w.id
                ));
            }
            if let Some(ms) = w.update_interval_ms {
                if !(100..=10_000).contains(&ms) {
                    return Err(format!(
                        "template '{}' widget[{i}] update_interval_ms must be in [100, 10000]",
                        self.id
                    ));
                }
            }
        }
        Ok(())
    }
}
