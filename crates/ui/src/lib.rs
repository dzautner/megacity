use bevy::prelude::*;
use bevy_egui::EguiPlugin;

pub mod advisor_tips;
pub mod cell_info_panel;
pub mod citizen_info;
pub mod day_night_panel;
pub mod district_inspect;
pub mod graphs;
pub mod info_panel;
pub mod localization;
pub mod milestones;
pub mod multi_select;
pub mod oneway_ui;
pub mod progressive_disclosure;
pub mod road_segment_info;
pub mod search;
pub mod service_coverage_panel;
pub mod settings_panel;
pub mod theme;
pub mod toolbar;
pub mod tutorial;
pub mod waste_dashboard;
pub mod water_dashboard;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .add_plugins(cell_info_panel::CellInfoPanelPlugin)
            .add_plugins(citizen_info::CitizenInfoPlugin)
            .add_plugins(district_inspect::DistrictInspectPlugin)
            .add_plugins(road_segment_info::RoadSegmentInfoPlugin)
            .add_plugins(waste_dashboard::WasteDashboardPlugin)
            .add_plugins(localization::LocalizationUiPlugin)
            .add_plugins(multi_select::MultiSelectUiPlugin)
            .add_plugins(progressive_disclosure::ProgressiveDisclosurePlugin)
            .add_plugins(service_coverage_panel::ServiceCoveragePanelPlugin)
            .add_plugins(oneway_ui::OneWayUiPlugin)
            .add_plugins(settings_panel::SettingsPanelPlugin)
            .add_plugins(advisor_tips::AdvisorTipsPlugin)
            .add_plugins(search::SearchPlugin)
            .init_resource::<day_night_panel::DayNightPanelVisible>()
            .init_resource::<milestones::Milestones>()
            .init_resource::<graphs::HistoryData>()
            .init_resource::<graphs::ChartsState>()
            .init_resource::<toolbar::OpenCategory>()
            .init_resource::<info_panel::JournalVisible>()
            .init_resource::<info_panel::ChartsVisible>()
            .init_resource::<info_panel::AdvisorVisible>()
            .init_resource::<info_panel::PoliciesVisible>()
            .init_resource::<info_panel::BudgetPanelVisible>()
            .init_resource::<water_dashboard::WaterDashboardVisible>()
            .add_systems(Startup, theme::apply_cute_theme)
            .add_systems(
                Update,
                (
                    milestones::check_milestones,
                    graphs::record_history,
                    toolbar::toolbar_ui,
                    info_panel::info_panel_ui,
                ),
            )
            .add_systems(
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
                ),
            );
    }
}
