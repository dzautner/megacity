//! Input handling module for the city builder.
//!
//! Split into sub-modules by concern:
//! - `types`: Resource types and enums (ActiveTool, CursorGridPos, etc.)
//! - `cursor`: Cursor position tracking, intersection snapping, status tick
//! - `placement`: Helper functions for placing roads, zones, utilities, services
//! - `road_drawing`: Freeform Bezier road drawing (straight and curved segments)
//! - `terrain_tools`: Terrain modification helpers (raise, lower, level, water)
//! - `tool_handler`: Main tool input dispatch system
//! - `keyboard`: Keyboard shortcuts, escape key, tree tool, road upgrade, building delete

mod cursor;
mod keyboard;
mod placement;
mod road_drawing;
mod terrain_tools;
mod tool_handler;
mod types;

#[cfg(test)]
mod tests;

// Re-export all public items so callers don't need to change their imports.

// Types and resources
pub use types::{
    ActiveTool, CursorGridPos, DrawPhase, GridSnap, IntersectionSnap, RoadDrawState,
    SelectedBuilding, StatusMessage,
};

// Cursor systems
pub use cursor::{tick_status_message, update_cursor_grid_pos, update_intersection_snap};

// Tool handler system
pub use tool_handler::handle_tool_input;

// Keyboard shortcut systems
pub use keyboard::{
    delete_selected_building, handle_escape_key, handle_road_upgrade_tool, handle_tree_tool,
    keyboard_tool_switch, toggle_curve_draw_mode, toggle_grid_snap,
};
