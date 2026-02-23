pub(crate) mod systems;
#[cfg(test)]
mod tests;
pub mod types;

pub use systems::check_achievements;
pub use types::{Achievement, AchievementNotification, AchievementReward, AchievementTracker};

use bevy::prelude::*;

pub struct AchievementsPlugin;

impl Plugin for AchievementsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AchievementTracker>()
            .init_resource::<AchievementNotification>()
            .add_systems(
                FixedUpdate,
                check_achievements
                    .after(crate::stats::update_stats)
                    .after(crate::specialization::compute_specializations)
                    .in_set(crate::SimulationSet::PostSim),
            );
    }
}
