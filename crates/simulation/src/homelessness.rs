use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::buildings::{Building, UnderConstruction};
use crate::citizen::{Citizen, CitizenDetails, HomeLocation};
use crate::TickCounter;

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Marker component for citizens who have lost their home.
/// `ticks_homeless` tracks how long the citizen has been without housing.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Homeless {
    pub ticks_homeless: u32,
    /// Whether this citizen is currently in a shelter.
    pub sheltered: bool,
}

/// A homeless shelter that can temporarily house displaced citizens.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct HomelessShelter {
    pub grid_x: usize,
    pub grid_y: usize,
    pub capacity: u32,
    pub current_occupants: u32,
}

// ---------------------------------------------------------------------------
// Resource
// ---------------------------------------------------------------------------

/// Tracks city-wide homelessness statistics.
#[derive(Resource, Default, Debug, Clone, Serialize, Deserialize)]
pub struct HomelessnessStats {
    pub total_homeless: u32,
    pub sheltered: u32,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// How often (in ticks) the homelessness check runs.
const CHECK_INTERVAL: u64 = 50;

/// Happiness penalty applied to homeless citizens (unsheltered).
pub const HOMELESS_PENALTY: f32 = 30.0;

/// Reduced happiness penalty for homeless citizens in a shelter.
pub const SHELTERED_PENALTY: f32 = 10.0;

/// Minimum salary threshold below which a citizen with negative savings
/// is considered unable to afford rent and becomes homeless.
const RENT_AFFORDABILITY_THRESHOLD: f32 = 1000.0;

// ---------------------------------------------------------------------------
// System: check_homelessness
// ---------------------------------------------------------------------------

/// Detects citizens who should become homeless and marks them.
///
/// A citizen becomes homeless when:
/// - Their home building entity no longer exists (despawned/demolished), OR
/// - Their home building entity is `Entity::PLACEHOLDER`, OR
/// - They cannot afford rent (savings < 0 and salary < threshold).
///
/// Runs every `CHECK_INTERVAL` ticks.
#[allow(clippy::type_complexity)]
pub fn check_homelessness(
    mut commands: Commands,
    tick: Res<TickCounter>,
    buildings: Query<Entity, With<Building>>,
    mut citizens: Query<
        (Entity, &HomeLocation, &mut CitizenDetails),
        (With<Citizen>, Without<Homeless>),
    >,
    mut stats: ResMut<HomelessnessStats>,
) {
    if !tick.0.is_multiple_of(CHECK_INTERVAL) {
        return;
    }

    let mut new_homeless = 0u32;

    for (entity, home, mut details) in &mut citizens {
        let mut should_be_homeless = false;

        // Check 1: home building is a placeholder (never assigned or cleared)
        // Check 2: home building entity has been despawned
        if home.building == Entity::PLACEHOLDER || buildings.get(home.building).is_err() {
            should_be_homeless = true;
        }

        // Check 3: cannot afford rent (negative savings + low salary)
        if !should_be_homeless
            && details.savings < 0.0
            && details.salary < RENT_AFFORDABILITY_THRESHOLD
        {
            should_be_homeless = true;
        }

        if should_be_homeless {
            commands.entity(entity).insert(Homeless {
                ticks_homeless: 0,
                sheltered: false,
            });
            // Immediate happiness penalty
            details.happiness = (details.happiness - HOMELESS_PENALTY).max(0.0);
            new_homeless += 1;
        }
    }

    // Recount total homeless (existing + new)
    // The full recount happens in seek_shelter, but we add new ones here
    // for immediate feedback.
    stats.total_homeless += new_homeless;
}

// ---------------------------------------------------------------------------
// System: seek_shelter
// ---------------------------------------------------------------------------

/// Homeless citizens attempt to find a shelter with available capacity.
/// If sheltered, the ongoing happiness penalty is reduced from -30 to -10.
pub fn seek_shelter(
    tick: Res<TickCounter>,
    mut homeless_citizens: Query<(Entity, &mut Homeless, &mut CitizenDetails), With<Citizen>>,
    mut shelters: Query<&mut HomelessShelter>,
    mut stats: ResMut<HomelessnessStats>,
) {
    if !tick.0.is_multiple_of(CHECK_INTERVAL) {
        return;
    }

    // Increment ticks_homeless for all homeless citizens
    for (_entity, mut homeless, _details) in &mut homeless_citizens {
        homeless.ticks_homeless += 1;
    }

    // Try to place unsheltered homeless citizens into shelters
    let mut sheltered_count = 0u32;
    let mut total_homeless = 0u32;

    for (_entity, mut homeless, mut details) in &mut homeless_citizens {
        total_homeless += 1;

        if homeless.sheltered {
            sheltered_count += 1;
            continue;
        }

        // Try to find a shelter with space
        for mut shelter in &mut shelters {
            if shelter.current_occupants < shelter.capacity {
                shelter.current_occupants += 1;
                homeless.sheltered = true;

                // Restore some happiness (difference between unsheltered and sheltered penalty)
                let restored = HOMELESS_PENALTY - SHELTERED_PENALTY;
                details.happiness = (details.happiness + restored).min(100.0);

                sheltered_count += 1;
                break;
            }
        }
    }

    stats.total_homeless = total_homeless;
    stats.sheltered = sheltered_count;
}

// ---------------------------------------------------------------------------
// System: recover_from_homelessness
// ---------------------------------------------------------------------------

/// Homeless citizens attempt to move into residential buildings with available capacity.
/// When successful, they get a new HomeLocation and lose the Homeless component.
pub fn recover_from_homelessness(
    mut commands: Commands,
    tick: Res<TickCounter>,
    mut homeless_citizens: Query<(Entity, &mut CitizenDetails, &Homeless), With<Citizen>>,
    mut buildings: Query<(Entity, &mut Building), Without<UnderConstruction>>,
    mut shelters: Query<&mut HomelessShelter>,
    mut stats: ResMut<HomelessnessStats>,
) {
    if !tick.0.is_multiple_of(CHECK_INTERVAL) {
        return;
    }

    // Collect residential buildings with capacity
    let available_homes: Vec<(Entity, usize, usize)> = buildings
        .iter()
        .filter(|(_, b)| b.zone_type.is_residential() && b.occupants < b.capacity)
        .map(|(e, b)| (e, b.grid_x, b.grid_y))
        .collect();

    if available_homes.is_empty() {
        return;
    }

    let mut home_idx = 0usize;
    let mut recovered = 0u32;

    for (citizen_entity, mut details, homeless) in &mut homeless_citizens {
        if home_idx >= available_homes.len() {
            break;
        }

        // Citizens need positive savings or a decent salary to recover
        // (otherwise they'd just become homeless again next check)
        if details.savings < 0.0 && details.salary < RENT_AFFORDABILITY_THRESHOLD {
            continue;
        }

        let (building_entity, gx, gy) = available_homes[home_idx];

        // Verify the building still has capacity (mutable borrow)
        if let Ok((_, mut building)) = buildings.get_mut(building_entity) {
            if building.occupants < building.capacity {
                building.occupants += 1;

                // If the citizen was sheltered, free up the shelter spot
                if homeless.sheltered {
                    for mut shelter in &mut shelters {
                        if shelter.current_occupants > 0 {
                            shelter.current_occupants -= 1;
                            break;
                        }
                    }
                }

                // Assign new home and remove Homeless component
                commands.entity(citizen_entity).insert(HomeLocation {
                    grid_x: gx,
                    grid_y: gy,
                    building: building_entity,
                });
                commands.entity(citizen_entity).remove::<Homeless>();

                // Restore happiness
                let penalty_to_restore = if homeless.sheltered {
                    SHELTERED_PENALTY
                } else {
                    HOMELESS_PENALTY
                };
                details.happiness = (details.happiness + penalty_to_restore).min(100.0);

                recovered += 1;
                home_idx += 1;
            } else {
                // Building filled up, move to next
                home_idx += 1;
            }
        } else {
            home_idx += 1;
        }
    }

    // Update stats
    if recovered > 0 {
        stats.total_homeless = stats.total_homeless.saturating_sub(recovered);
    }
}
