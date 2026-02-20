use bevy::prelude::*;
use bevy_egui::EguiPlugin;

pub mod graphs;
pub mod info_panel;
pub mod milestones;
pub mod oneway_ui;
pub mod theme;
pub mod toolbar;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .init_resource::<milestones::Milestones>()
            .init_resource::<graphs::HistoryData>()
            .init_resource::<toolbar::OpenCategory>()
            .init_resource::<info_panel::JournalVisible>()
            .init_resource::<info_panel::ChartsVisible>()
            .init_resource::<info_panel::AdvisorVisible>()
            .init_resource::<info_panel::PoliciesVisible>()
            .init_resource::<info_panel::BudgetPanelVisible>()
            .add_systems(Startup, theme::apply_cute_theme)
            .add_systems(
                Update,
                (
                    milestones::check_milestones,
                    graphs::record_history,
                    toolbar::toolbar_ui,
                    info_panel::info_panel_ui,
                    info_panel::building_inspection_ui,
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
                ),
            )
            .add_plugins(oneway_ui::OneWayUiPlugin);
    }
}
