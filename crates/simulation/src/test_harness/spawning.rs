//! Citizen spawning builder methods for `TestCity`.

use bevy::prelude::*;

use crate::citizen::{
    Citizen, CitizenDetails, CitizenState, CitizenStateComp, Family, Gender, HomeLocation, Needs,
    PathCache, Personality, Position, Velocity, WorkLocation,
};
use crate::grid::WorldGrid;
use crate::mode_choice::ChosenTransportMode;
use crate::movement::ActivityTimer;

use super::TestCity;

impl TestCity {
    // -----------------------------------------------------------------------
    // Citizen spawning
    // -----------------------------------------------------------------------

    /// Spawn a citizen with a home and work location.
    /// The home and work buildings must already exist (use `with_building` first).
    pub fn with_citizen(mut self, home: (usize, usize), work: (usize, usize)) -> Self {
        let world = self.app.world_mut();
        let home_entity = {
            let grid = world.resource::<WorldGrid>();
            grid.get(home.0, home.1)
                .building_id
                .unwrap_or(Entity::PLACEHOLDER)
        };
        let work_entity = {
            let grid = world.resource::<WorldGrid>();
            grid.get(work.0, work.1)
                .building_id
                .unwrap_or(Entity::PLACEHOLDER)
        };

        let (hx, hy) = WorldGrid::grid_to_world(home.0, home.1);

        world.spawn((
            Citizen,
            Position { x: hx, y: hy },
            Velocity { x: 0.0, y: 0.0 },
            HomeLocation {
                grid_x: home.0,
                grid_y: home.1,
                building: home_entity,
            },
            WorkLocation {
                grid_x: work.0,
                grid_y: work.1,
                building: work_entity,
            },
            CitizenStateComp(CitizenState::AtHome),
            PathCache::new(Vec::new()),
            CitizenDetails {
                age: 30,
                gender: Gender::Male,
                education: 2,
                happiness: 60.0,
                health: 90.0,
                salary: 3500.0,
                savings: 7000.0,
            },
            Personality {
                ambition: 0.5,
                sociability: 0.5,
                materialism: 0.5,
                resilience: 0.5,
            },
            Needs::default(),
            Family::default(),
            ActivityTimer::default(),
            ChosenTransportMode::default(),
        ));
        self
    }

    /// Spawn an unemployed citizen (no `WorkLocation`) at the given home.
    /// The home building must already exist (use `with_building` first).
    pub fn with_unemployed_citizen(mut self, home: (usize, usize)) -> Self {
        let world = self.app.world_mut();
        let home_entity = {
            let grid = world.resource::<WorldGrid>();
            grid.get(home.0, home.1)
                .building_id
                .unwrap_or(Entity::PLACEHOLDER)
        };

        let (hx, hy) = WorldGrid::grid_to_world(home.0, home.1);

        world.spawn((
            Citizen,
            Position { x: hx, y: hy },
            Velocity { x: 0.0, y: 0.0 },
            HomeLocation {
                grid_x: home.0,
                grid_y: home.1,
                building: home_entity,
            },
            CitizenStateComp(CitizenState::AtHome),
            PathCache::new(Vec::new()),
            CitizenDetails {
                age: 30,
                gender: Gender::Male,
                education: 0,
                happiness: 60.0,
                health: 90.0,
                salary: 0.0,
                savings: 1000.0,
            },
            Personality {
                ambition: 0.5,
                sociability: 0.5,
                materialism: 0.5,
                resilience: 0.5,
            },
            Needs::default(),
            Family::default(),
            ActivityTimer::default(),
            ChosenTransportMode::default(),
        ));
        self
    }
}
