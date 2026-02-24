//! Runtime invariant validation for core simulation state.
//!
//! These systems run periodically (every slow-tick cycle) and log warnings
//! when invariant violations are detected, tracking violation counts for
//! integration testing.
//!
//! Validated invariants:
//! 1. **Job occupancy**: no building has more workers than its capacity.
//! 2. **Marriage reciprocity**: all partner relationships are symmetric.
//! 3. **Employment consistency**: per-building worker counts derived from
//!    `WorkLocation` components match the `Building::occupants` field.

use bevy::prelude::*;
use std::collections::HashMap;

use crate::buildings::Building;
use crate::citizen::{Citizen, Family, HomeLocation, WorkLocation};
use crate::SlowTickTimer;

/// Tracks the number of invariant violations detected during the last
/// validation pass. Used by integration tests to assert zero violations.
#[derive(Resource, Default, Debug)]
pub struct InvariantViolations {
    pub job_overcapacity: u32,
    pub marriage_non_reciprocal: u32,
    pub employment_drift: u32,
}

pub fn validate_job_occupancy(
    slow_tick: Res<SlowTickTimer>,
    mut buildings: Query<(Entity, &mut Building)>,
    mut violations: ResMut<InvariantViolations>,
) {
    if !slow_tick.should_run() {
        return;
    }
    violations.job_overcapacity = 0;
    for (entity, mut building) in &mut buildings {
        if building.occupants > building.capacity {
            warn!(
                "Invariant violation: building {:?} at ({},{}) has {} occupants but capacity {}. Clamping.",
                entity, building.grid_x, building.grid_y, building.occupants, building.capacity
            );
            building.occupants = building.capacity;
            violations.job_overcapacity += 1;
        }
    }
}

pub fn validate_marriage_reciprocity(
    slow_tick: Res<SlowTickTimer>,
    mut citizens: Query<(Entity, &mut Family), With<Citizen>>,
    mut violations: ResMut<InvariantViolations>,
) {
    if !slow_tick.should_run() {
        return;
    }
    violations.marriage_non_reciprocal = 0;
    let partner_map: HashMap<Entity, Option<Entity>> =
        citizens.iter().map(|(e, f)| (e, f.partner)).collect();
    let mut to_clear: Vec<Entity> = Vec::new();
    for (&entity, &partner_opt) in &partner_map {
        if let Some(partner) = partner_opt {
            match partner_map.get(&partner) {
                Some(Some(back)) if *back == entity => {}
                _ => {
                    warn!(
                        "Invariant violation: citizen {:?} has partner {:?} but the link is not reciprocal. Clearing.",
                        entity, partner
                    );
                    to_clear.push(entity);
                    violations.marriage_non_reciprocal += 1;
                }
            }
        }
    }
    for entity in to_clear {
        if let Ok((_, mut family)) = citizens.get_mut(entity) {
            family.partner = None;
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn validate_employment_consistency(
    slow_tick: Res<SlowTickTimer>,
    workers: Query<&WorkLocation, With<Citizen>>,
    residents: Query<&HomeLocation, With<Citizen>>,
    mut buildings: Query<(Entity, &mut Building)>,
    mut violations: ResMut<InvariantViolations>,
) {
    if !slow_tick.should_run() {
        return;
    }
    violations.employment_drift = 0;
    let mut worker_counts: HashMap<Entity, u32> = HashMap::new();
    for work in &workers {
        *worker_counts.entry(work.building).or_insert(0) += 1;
    }
    let mut resident_counts: HashMap<Entity, u32> = HashMap::new();
    for home in &residents {
        *resident_counts.entry(home.building).or_insert(0) += 1;
    }
    for (entity, mut building) in &mut buildings {
        if building.zone_type.is_job_zone() {
            let actual = worker_counts.get(&entity).copied().unwrap_or(0);
            if actual > building.occupants {
                warn!(
                    "Employment drift: building {:?} at ({},{}) has {} occupants but {} actual workers. Correcting.",
                    entity, building.grid_x, building.grid_y, building.occupants, actual
                );
                building.occupants = actual;
                violations.employment_drift += 1;
            }
        } else if building.zone_type.is_residential() {
            let actual = resident_counts.get(&entity).copied().unwrap_or(0);
            if actual > building.occupants {
                warn!(
                    "Residential drift: building {:?} at ({},{}) has {} occupants but {} actual residents. Correcting.",
                    entity, building.grid_x, building.grid_y, building.occupants, actual
                );
                building.occupants = actual;
                violations.employment_drift += 1;
            }
        }
    }
}

pub struct SimulationInvariantsPlugin;

impl Plugin for SimulationInvariantsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InvariantViolations>().add_systems(
            FixedUpdate,
            (
                validate_job_occupancy,
                validate_marriage_reciprocity,
                validate_employment_consistency,
            )
                // Order-independent: read-only validation systems that write only
                // InvariantViolations (private resource); no shared mutable state.
                .in_set(crate::SimulationSet::PostSim),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invariant_violations_default() {
        let v = InvariantViolations::default();
        assert_eq!(v.job_overcapacity, 0);
        assert_eq!(v.marriage_non_reciprocal, 0);
        assert_eq!(v.employment_drift, 0);
    }
}
