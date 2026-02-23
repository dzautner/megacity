//! Shortcut category and item type definitions.

use bevy::prelude::*;
use rendering::input::ActiveTool;
use rendering::overlay::OverlayMode;

mod placement;
mod views_and_tools;

/// A shortcut category: a trigger key and the list of sub-tools.
pub(crate) struct ShortcutCategory {
    pub key: KeyCode,
    pub label: &'static str,
    /// Short string shown in the popup header, e.g. "R" for roads.
    pub key_hint: &'static str,
    pub items: Vec<ShortcutItem>,
}

pub(crate) struct ShortcutItem {
    pub name: &'static str,
    pub tool: Option<ActiveTool>,
    pub overlay: Option<OverlayMode>,
}

/// Build the full list of shortcut categories (mirrors toolbar order).
pub(crate) fn build_shortcut_categories() -> Vec<ShortcutCategory> {
    let mut cats = placement::placement_categories();
    cats.extend(views_and_tools::views_and_tools_categories());
    cats
}
