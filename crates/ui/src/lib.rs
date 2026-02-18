use bevy::prelude::*;
use bevy_egui::EguiPlugin;

pub mod graphs;
pub mod info_panel;
pub mod milestones;
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
                    info_panel::toggle_journal_visibility,
                    info_panel::event_journal_ui,
                ),
            );
    }
}
