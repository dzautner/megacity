//! Toolbar UI module — split into sub-modules for maintainability.
//!
//! - `catalog`: tool/category definitions, tooltips, and the `ToolCatalog` resource
//! - `speed`: simulation speed keyboard shortcuts
//! - `ui_system`: the main `toolbar_ui` egui system
//! - `widgets`: reusable UI helpers (demand bars, speed buttons, formatting)

mod catalog;
mod speed;
mod ui_system;
mod widgets;

pub use catalog::{OpenCategory, ToolCatalog};
pub use speed::speed_keybinds;
pub use ui_system::toolbar_ui;
