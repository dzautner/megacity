use bevy::prelude::*;

pub mod advisor_tips;
pub mod aqi_tooltip;
pub mod auto_grid_ui;
pub mod box_selection;
pub mod cell_info_panel;
pub mod cell_tooltip;
pub mod citizen_info;
pub mod context_menu;
pub mod day_night_panel;
pub mod district_inspect;
pub mod dual_overlay;
pub mod graphs;
pub mod info_panel;
pub mod keybindings_panel;
pub mod localization;
pub mod milestones;
pub mod minimap;
pub mod multi_select;
pub mod notification_ticker;
pub mod oneway_ui;
pub mod overlay_legend;
pub mod progressive_disclosure;
pub mod road_cost_display;
pub mod road_segment_info;
pub mod search;
pub mod service_coverage_panel;
pub mod settings_panel;
pub mod theme;
pub mod toolbar;
pub mod tutorial;
pub mod two_key_shortcuts;
pub mod waste_dashboard;
pub mod water_dashboard;
pub mod zone_brush_ui;

mod plugin_registration;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        // Register all UI systems and plugins (extracted for conflict-free additions)
        plugin_registration::register_ui_systems(app);
    }
}
