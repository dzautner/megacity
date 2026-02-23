use bevy::prelude::*;

use super::systems::{assign_workplace_details, job_matching};
use super::EmploymentStats;

pub struct EducationJobsPlugin;

impl Plugin for EducationJobsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EmploymentStats>()
            .add_systems(
                FixedUpdate,
                assign_workplace_details
                    .after(crate::buildings::progress_construction)
                    .in_set(crate::SimulationSet::PreSim),
            )
            .add_systems(
                FixedUpdate,
                job_matching
                    .after(crate::life_simulation::job_seeking)
                    .in_set(crate::SimulationSet::Simulation),
            );
    }
}
