mod advisor;
pub mod budget;
mod building_inspection;
mod city_overview;
mod economy_section;
mod event_journal;
mod finance_section;
mod groundwater_tooltip;
mod keybinds;
mod minimap;
mod panel;
mod policies;
mod services_section;
mod types;

// Re-export all public items so the rest of the crate sees the same API.
pub use advisor::advisor_window_ui;
pub use budget::budget_panel_ui;
pub use building_inspection::building_inspection_ui;
pub use event_journal::event_journal_ui;
pub use groundwater_tooltip::groundwater_tooltip_ui;
pub use keybinds::{panel_keybinds, quick_save_load_keybinds};
pub use panel::info_panel_ui;
pub use policies::policies_ui;
pub use types::{
    AdvisorVisible, BudgetPanelVisible, ChartsVisible, JournalVisible, MinimapCache,
    PoliciesVisible,
};
