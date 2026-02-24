use bevy::prelude::*;
use bevy_egui::EguiPlugin;

use crate::*;

/// Register all UI plugins and systems.
///
/// Each plugin is registered on its own line for conflict-free parallel additions.
/// When adding a new UI plugin, just append a new `app.add_plugins(...)` line
/// at the end of the appropriate section.
pub(crate) fn register_ui_systems(app: &mut App) {
    // Core egui
    app.add_plugins(EguiPlugin);

    // UI feature plugins
    app.add_plugins(cell_info_panel::CellInfoPanelPlugin);
    app.add_plugins(cell_tooltip::CellTooltipPlugin);
    app.add_plugins(citizen_info::CitizenInfoPlugin);
    app.add_plugins(context_menu::ContextMenuPlugin);
    app.add_plugins(district_inspect::DistrictInspectPlugin);
    app.add_plugins(road_segment_info::RoadSegmentInfoPlugin);
    app.add_plugins(waste_dashboard::WasteDashboardPlugin);
    app.add_plugins(localization::LocalizationUiPlugin);
    app.add_plugins(multi_select::MultiSelectUiPlugin);
    app.add_plugins(progressive_disclosure::ProgressiveDisclosurePlugin);
    app.add_plugins(service_coverage_panel::ServiceCoveragePanelPlugin);
    app.add_plugins(oneway_ui::OneWayUiPlugin);
    app.add_plugins(settings_panel::SettingsPanelPlugin);
    app.add_plugins(advisor_tips::AdvisorTipsPlugin);
    app.add_plugins(aqi_tooltip::AqiTooltipPlugin);
    app.add_plugins(auto_grid_ui::AutoGridUiPlugin);
    app.add_plugins(keybindings_panel::KeybindingsPanelPlugin);
    app.add_plugins(search::SearchPlugin);
    app.add_plugins(overlay_legend::OverlayLegendPlugin);
    app.add_plugins(dual_overlay::DualOverlayPlugin);
    app.add_plugins(two_key_shortcuts::TwoKeyShortcutPlugin);
    app.add_plugins(minimap::MinimapPlugin);
    app.add_plugins(notification_ticker::NotificationTickerPlugin);
    app.add_plugins(box_selection::BoxSelectionUiPlugin);
    app.add_plugins(zone_brush_ui::ZoneBrushUiPlugin);
    app.add_plugins(info_panel::budget::BudgetBreakdownPlugin);
    app.add_plugins(energy_dashboard::EnergyDashboardPlugin);
    // UI resources
    app.init_resource::<day_night_panel::DayNightPanelVisible>();
    app.init_resource::<milestones::Milestones>();
    app.init_resource::<graphs::HistoryData>();
    app.init_resource::<graphs::ChartsState>();
    app.init_resource::<toolbar::OpenCategory>();
    app.init_resource::<toolbar::ToolCatalog>();
    app.init_resource::<info_panel::JournalVisible>();
    app.init_resource::<info_panel::ChartsVisible>();
    app.init_resource::<info_panel::AdvisorVisible>();
    app.init_resource::<info_panel::PoliciesVisible>();
    app.init_resource::<info_panel::BudgetPanelVisible>();
    app.init_resource::<water_dashboard::WaterDashboardVisible>();

    // UI systems
    app.add_systems(Startup, theme::apply_cute_theme);
    app.add_systems(
        Update,
        (
            milestones::check_milestones,
            graphs::record_history,
            toolbar::toolbar_ui,
            info_panel::info_panel_ui,
        ),
    );
    app.add_systems(
        Update,
        (
            milestones::milestones_ui,
            graphs::graphs_ui,
            info_panel::policies_ui,
            info_panel::panel_keybinds,
            info_panel::quick_save_load_keybinds,
            info_panel::event_journal_ui,
            info_panel::advisor_window_ui,
            info_panel::budget_panel_ui,
            toolbar::speed_keybinds,
            info_panel::groundwater_tooltip_ui,
            water_dashboard::water_dashboard_ui,
            tutorial::tutorial_ui,
            day_night_panel::day_night_panel_ui,
            road_cost_display::road_cost_display_ui,
        ),
    );
}
