//! Persistence + resolution for LCD templates.
//!
//! The daemon owns `lcd_templates.json` next to `config.json`. Built-in
//! templates live in `lianli_shared::template_defaults` (as `const` in code)
//! and are *not* written to the file — they're merged in at read time.
//! Editing a built-in in the GUI triggers a "Duplicate to edit" flow that
//! writes a user-owned copy with a new id into the file.

use anyhow::{Context, Result};
use lianli_shared::template::LcdTemplate;
use lianli_shared::template_defaults::{builtin_template, builtin_templates, is_builtin_id};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::warn;

/// File format for `lcd_templates.json`.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
struct TemplateFile {
    #[serde(default)]
    templates: Vec<LcdTemplate>,
}

/// Derive the templates file path from the config file path.
pub fn templates_path_for(config_path: &Path) -> PathBuf {
    config_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
        .join("lcd_templates.json")
}

/// Load user templates from disk. Missing file → empty list (not an error).
/// Malformed file → logs warning and returns empty list so the daemon still
/// boots with built-ins available.
pub fn load_user_templates(path: &Path) -> Vec<LcdTemplate> {
    if !path.exists() {
        return Vec::new();
    }
    match fs::read_to_string(path) {
        Ok(json) => match serde_json::from_str::<TemplateFile>(&json) {
            Ok(file) => {
                // Silently drop any user templates that shadow built-in ids —
                // built-ins are reserved and resolved first.
                file.templates
                    .into_iter()
                    .filter(|t| {
                        if is_builtin_id(&t.id) {
                            warn!(
                                "Ignoring user template with reserved built-in id '{}'",
                                t.id
                            );
                            false
                        } else {
                            true
                        }
                    })
                    .collect()
            }
            Err(e) => {
                warn!("Failed to parse {}: {e}", path.display());
                Vec::new()
            }
        },
        Err(e) => {
            warn!("Failed to read {}: {e}", path.display());
            Vec::new()
        }
    }
}

/// Write user templates to disk. Built-in templates are filtered out on save
/// so the file only ever contains user content.
pub fn save_user_templates(path: &Path, templates: &[LcdTemplate]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("creating parent dir for {}", path.display()))?;
    }
    let file = TemplateFile {
        templates: templates
            .iter()
            .filter(|t| !is_builtin_id(&t.id))
            .cloned()
            .collect(),
    };
    let json = serde_json::to_string_pretty(&file)?;
    fs::write(path, json).with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}

/// Full template list visible to the GUI: built-ins first, then user entries.
pub fn all_templates(user: &[LcdTemplate]) -> Vec<LcdTemplate> {
    let mut out = builtin_templates();
    out.extend(user.iter().cloned());
    out
}

/// Resolve a template id against the combined built-in + user set.
#[allow(dead_code)] // used by Commit 2+ in the real CustomAsset renderer path
pub fn resolve_template(id: &str, user: &[LcdTemplate]) -> Option<LcdTemplate> {
    if let Some(t) = builtin_template(id) {
        return Some(t);
    }
    user.iter().find(|t| t.id == id).cloned()
}
