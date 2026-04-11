//! Render a `preview.png` for a template folder.
//!
//! Usage:
//!   cargo run -p lianli-media --bin render-preview -- <template-dir> [--out <file>]
//!
//! Loads `<template-dir>/template.json`, substitutes a fixed mock value for
//! every sensor-bearing widget, renders one frame at the template's native
//! `base_width × base_height`, and writes the result to
//! `<template-dir>/preview.png` (or `--out` if given).

use anyhow::{anyhow, Context, Result};
use lianli_media::CustomAsset;
use lianli_shared::media::SensorSourceConfig;
use lianli_shared::screen::ScreenInfo;
use lianli_shared::template::{LcdTemplate, WidgetKind};
use std::path::PathBuf;

const MOCK_VALUE: f32 = 65.0;

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let mut template_dir: Option<PathBuf> = None;
    let mut out_path: Option<PathBuf> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--out" | "-o" => {
                out_path = Some(PathBuf::from(
                    args.next()
                        .ok_or_else(|| anyhow!("--out requires a value"))?,
                ));
            }
            "-h" | "--help" => {
                print_usage();
                return Ok(());
            }
            other if template_dir.is_none() => template_dir = Some(PathBuf::from(other)),
            other => return Err(anyhow!("unexpected argument '{other}'")),
        }
    }

    let template_dir = template_dir.ok_or_else(|| {
        print_usage();
        anyhow!("missing template directory")
    })?;

    let template_path = template_dir.join("template.json");
    let raw = std::fs::read_to_string(&template_path)
        .with_context(|| format!("reading {}", template_path.display()))?;
    let mut template: LcdTemplate = serde_json::from_str(&raw)
        .with_context(|| format!("parsing {}", template_path.display()))?;

    stub_sensor_sources(&mut template);

    // Render 1:1 in the template's own coordinate space. orientation=0 means
    // `render_dimensions` returns (base_width, base_height) and apply_orientation
    // is a no-op, giving a pixel-perfect preview at native resolution.
    let screen = ScreenInfo {
        width: template.base_width,
        height: template.base_height,
        max_fps: 30,
        jpeg_quality: 100,
        max_payload: usize::MAX,
        device_rotation: 0,
        h264: false,
    };

    let asset = CustomAsset::new(&template, 0.0, &screen, &[])
        .map_err(|e| anyhow!("building custom asset: {e}"))?;
    let frame = asset
        .render_frame(true)
        .map_err(|e| anyhow!("rendering frame: {e}"))?
        .ok_or_else(|| anyhow!("render_frame returned no frame"))?;

    let decoded =
        image::load_from_memory(&frame.data).context("decoding rendered JPEG for PNG re-encode")?;

    let out = out_path.unwrap_or_else(|| template_dir.join("preview.png"));
    decoded
        .save(&out)
        .with_context(|| format!("writing {}", out.display()))?;
    println!("wrote {}", out.display());
    Ok(())
}

fn print_usage() {
    eprintln!("usage: render-preview <template-dir> [--out <file>]");
}

fn stub_sensor_sources(template: &mut LcdTemplate) {
    for widget in template.widgets.iter_mut() {
        if let Some(source) = widget_source_mut(&mut widget.kind) {
            *source = SensorSourceConfig::Constant { value: MOCK_VALUE };
        }
    }
}

fn widget_source_mut(kind: &mut WidgetKind) -> Option<&mut SensorSourceConfig> {
    match kind {
        WidgetKind::ValueText { source, .. }
        | WidgetKind::RadialGauge { source, .. }
        | WidgetKind::VerticalBar { source, .. }
        | WidgetKind::HorizontalBar { source, .. }
        | WidgetKind::Speedometer { source, .. } => Some(source),
        _ => None,
    }
}
