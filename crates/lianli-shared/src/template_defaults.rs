//! Built-in `LcdTemplate`s shipped with the software.
//!
//! Built-ins live here (not in `lcd_templates.json`) so they're always fresh,
//! upgradable with software updates, and safe from accidental corruption.
//! Editing a built-in in the GUI triggers a "Duplicate to edit" flow that
//! clones the template into the user file under a new id.
//!
//! Commit 1 ships minimal stubs (just id + name + solid background) so the
//! rest of the pipeline can resolve them. Commit 2 will flesh them out with
//! widgets that match the existing Doublegauge / Cooler renderers.

use crate::template::{LcdTemplate, TemplateBackground, TemplateOrientation};

pub const BUILTIN_COOLER_ID: &str = "cooler-default";
pub const BUILTIN_DOUBLEGAUGE_ID: &str = "doublegauge-default";

/// Reserved ids that cannot be deleted or overwritten by user templates.
pub fn is_builtin_id(id: &str) -> bool {
    id == BUILTIN_COOLER_ID || id == BUILTIN_DOUBLEGAUGE_ID
}

/// Return all built-in templates. Called by the template store to merge with
/// user templates during resolution.
pub fn builtin_templates() -> Vec<LcdTemplate> {
    vec![builtin_cooler(), builtin_doublegauge()]
}

/// Look up a built-in template by id. Returns `None` for non-reserved ids.
pub fn builtin_template(id: &str) -> Option<LcdTemplate> {
    match id {
        BUILTIN_COOLER_ID => Some(builtin_cooler()),
        BUILTIN_DOUBLEGAUGE_ID => Some(builtin_doublegauge()),
        _ => None,
    }
}

fn builtin_cooler() -> LcdTemplate {
    LcdTemplate {
        id: BUILTIN_COOLER_ID.to_string(),
        name: "Cooler (default)".to_string(),
        base_width: 480,
        base_height: 480,
        background: TemplateBackground::Color { rgb: [0, 0, 0] },
        widgets: Vec::new(),
        orientation: TemplateOrientation::Portrait,
    }
}

fn builtin_doublegauge() -> LcdTemplate {
    LcdTemplate {
        id: BUILTIN_DOUBLEGAUGE_ID.to_string(),
        name: "Doublegauge (default)".to_string(),
        base_width: 400,
        base_height: 400,
        background: TemplateBackground::Color { rgb: [0, 0, 0] },
        widgets: Vec::new(),
        orientation: TemplateOrientation::Portrait,
    }
}
