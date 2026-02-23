//! ECS systems for the NIMBY/YIMBY mechanic.

use bevy::prelude::*;

use crate::buildings::{Building, UnderConstruction};
use crate::citizen::{Citizen, CitizenDetails, HomeLocation, Personality};
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::events::{CityEvent, CityEventType, EventJournal};
use crate::grid::WorldGrid;
use crate::happiness::ServiceCoverageGrid;
use crate::land_value::LandValueGrid;
use crate::policies::{Policies, Policy};
use crate::time_of_day::GameClock;
use crate::zones::ZoneDemand;
use crate::SlowTickTimer;

use super::opinion::{calculate_opinion, construction_slowdown, nimby_happiness_penalty};
use super::types::{
    zone_type_name, ZoneChangeEvent, ZoneSnapshot, EMINENT_DOMAIN_HAPPINESS_PENALTY,
    MAX_ZONE_CHANGES, OPINION_DURATION_TICKS, PROTEST_COOLDOWN_TICKS, PROTEST_THRESHOLD,
    REACTION_RADIUS,
};
use super::NimbyState;

/// Detect zone changes by comparing the current grid against the previous snapshot.
/// Any cell whose zone type changed is recorded as a `ZoneChangeEvent`.
pub fn detect_zone_changes(
    grid: Res<WorldGrid>,
    tick: Res<crate::TickCounter>,
    mut nimby: ResMut<NimbyState>,
    mut snapshot: ResMut<ZoneSnapshot>,
) {
    if !grid.is_changed() {
        return;
    }

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let idx = y * GRID_WIDTH + x;
            let current_zone = grid.get(x, y).zone;
            let old_zone = snapshot.zones[idx];

            if current_zone != old_zone && current_zone != crate::grid::ZoneType::None {
                // Record the zone change
                let event = ZoneChangeEvent {
                    grid_x: x,
                    grid_y: y,
                    old_zone,
                    new_zone: current_zone,
                    created_tick: tick.0,
                    remaining_ticks: OPINION_DURATION_TICKS,
                    protest_triggered: false,
                    protest_cooldown: 0,
                };

                nimby.zone_changes.push(event);
                nimby.total_changes_processed += 1;

                // Trim to max tracked events
                if nimby.zone_changes.len() > MAX_ZONE_CHANGES {
                    nimby.zone_changes.remove(0);
                }
            }

            // Update snapshot
            snapshot.zones[idx] = current_zone;
        }
    }
}

/// Update the opposition grid based on active zone changes.
/// Decays old events and computes per-cell opposition scores.
///
/// Runs on the slow tick timer to avoid per-frame cost.
#[allow(clippy::too_many_arguments)]
pub fn update_nimby_opinions(
    timer: Res<SlowTickTimer>,
    mut nimby: ResMut<NimbyState>,
    land_value: Res<LandValueGrid>,
    coverage: Res<ServiceCoverageGrid>,
    demand: Res<ZoneDemand>,
    policies: Res<Policies>,
    clock: Res<GameClock>,
    mut journal: ResMut<EventJournal>,
) {
    if !timer.should_run() {
        return;
    }

    // If Eminent Domain is active, suppress all opposition
    let eminent_domain_active = policies.is_active(Policy::EminentDomain);

    // Decay and remove expired zone change events
    nimby.zone_changes.retain_mut(|event| {
        event.remaining_ticks = event.remaining_ticks.saturating_sub(1);
        if event.protest_cooldown > 0 {
            event.protest_cooldown = event.protest_cooldown.saturating_sub(1);
        }
        event.remaining_ticks > 0
    });

    // Clear opposition grid
    nimby.opposition_grid.fill(0.0);

    if nimby.zone_changes.is_empty() {
        nimby.active_protests = 0;
        return;
    }

    let residential_vacancy = demand.vacancy_residential;

    // For each active zone change, compute opposition at surrounding cells
    // Clone to avoid borrow conflict with opposition_grid mutation
    let zone_changes_snapshot = nimby.zone_changes.to_vec();
    for event in &zone_changes_snapshot {
        let ex = event.grid_x as i32;
        let ey = event.grid_y as i32;

        // Time decay factor: opposition fades as the event ages
        let time_factor = event.remaining_ticks as f32 / OPINION_DURATION_TICKS as f32;

        for dy in -REACTION_RADIUS..=REACTION_RADIUS {
            for dx in -REACTION_RADIUS..=REACTION_RADIUS {
                let nx = ex + dx;
                let ny = ey + dy;
                if nx < 0 || ny < 0 || nx >= GRID_WIDTH as i32 || ny >= GRID_HEIGHT as i32 {
                    continue;
                }
                let ux = nx as usize;
                let uy = ny as usize;

                let distance = ((dx * dx + dy * dy) as f32).sqrt();
                if distance > REACTION_RADIUS as f32 {
                    continue;
                }

                let lv = land_value.get(ux, uy);
                let cov_idx = ServiceCoverageGrid::idx(ux, uy);
                let has_park = coverage.flags[cov_idx] & crate::happiness::COVERAGE_PARK != 0;
                let has_transit =
                    coverage.flags[cov_idx] & crate::happiness::COVERAGE_TRANSPORT != 0;

                // Use a representative citizen profile for grid-level calculation
                // (education 2 = middle income, neutral personality)
                let neutral_personality = Personality {
                    ambition: 0.5,
                    sociability: 0.5,
                    materialism: 0.5,
                    resilience: 0.5,
                };

                let opinion = calculate_opinion(
                    event.old_zone,
                    event.new_zone,
                    distance,
                    lv,
                    2,
                    &neutral_personality,
                    has_park,
                    has_transit,
                    residential_vacancy,
                );

                // Apply time decay and eminent domain override
                let effective_opinion = if eminent_domain_active {
                    opinion.min(0.0) // Only keep support (negative scores), suppress opposition
                } else {
                    opinion * time_factor
                };

                let idx = uy * GRID_WIDTH + ux;
                nimby.opposition_grid[idx] += effective_opinion;
            }
        }
    }

    // Check for protest triggers
    // Pre-compute local opposition per event to avoid borrow conflict
    let local_oppositions: Vec<f32> = nimby
        .zone_changes
        .iter()
        .map(|event| {
            if event.protest_triggered && event.protest_cooldown > 0 {
                return 0.0;
            }
            let ex = event.grid_x;
            let ey = event.grid_y;
            let mut local_opposition = 0.0_f32;
            let check_radius: i32 = 3;
            for dy in -check_radius..=check_radius {
                for dx in -check_radius..=check_radius {
                    let nx = ex as i32 + dx;
                    let ny = ey as i32 + dy;
                    if nx >= 0
                        && ny >= 0
                        && (nx as usize) < GRID_WIDTH
                        && (ny as usize) < GRID_HEIGHT
                    {
                        let opp = nimby.opposition_grid[ny as usize * GRID_WIDTH + nx as usize];
                        if opp > 0.0 {
                            local_opposition += opp;
                        }
                    }
                }
            }
            local_opposition
        })
        .collect();

    let mut protest_count = 0u32;
    for (i, event) in nimby.zone_changes.iter_mut().enumerate() {
        if event.protest_triggered && event.protest_cooldown > 0 {
            continue;
        }
        let local_opposition = local_oppositions[i];

        if local_opposition >= PROTEST_THRESHOLD && !eminent_domain_active {
            protest_count += 1;
            event.protest_triggered = true;
            event.protest_cooldown = PROTEST_COOLDOWN_TICKS;

            let zone_name = zone_type_name(event.new_zone);
            journal.push(CityEvent {
                event_type: CityEventType::NewPolicy(format!(
                    "Protest at ({}, {})",
                    event.grid_x, event.grid_y
                )),
                day: clock.day,
                hour: clock.hour,
                description: format!(
                    "Citizens are protesting {} development near ({}, {}). Opposition: {:.0}",
                    zone_name, event.grid_x, event.grid_y, local_opposition
                ),
            });
        }
    }

    nimby.active_protests = protest_count;
}

/// Apply NIMBY happiness penalties to citizens near opposed development.
/// Also applies the Eminent Domain global happiness penalty.
pub fn apply_nimby_happiness(
    timer: Res<SlowTickTimer>,
    nimby: Res<NimbyState>,
    policies: Res<Policies>,
    mut citizens: Query<(&HomeLocation, &mut CitizenDetails, &Personality), With<Citizen>>,
) {
    if !timer.should_run() {
        return;
    }

    // Skip if no active opposition and no eminent domain
    let eminent_domain_active = policies.is_active(Policy::EminentDomain);
    if nimby.zone_changes.is_empty() && !eminent_domain_active {
        return;
    }

    citizens
        .par_iter_mut()
        .for_each(|(home, mut details, personality)| {
            let opposition = nimby.opposition_at(home.grid_x, home.grid_y);

            // Per-citizen personality adjustment to opposition
            let personal_opposition = opposition
                * (0.7 + personality.materialism * 0.6)
                * (1.3 - personality.resilience * 0.5);

            let penalty = nimby_happiness_penalty(personal_opposition);
            if penalty > 0.0 {
                details.happiness = (details.happiness - penalty).max(0.0);
            }

            // Eminent Domain policy: global happiness penalty
            if eminent_domain_active {
                details.happiness = (details.happiness - EMINENT_DOMAIN_HAPPINESS_PENALTY).max(0.0);
            }
        });
}

/// Slow down construction of buildings in high-opposition areas.
pub fn apply_construction_slowdown(
    timer: Res<SlowTickTimer>,
    nimby: Res<NimbyState>,
    policies: Res<Policies>,
    mut buildings: Query<(&Building, &mut UnderConstruction)>,
) {
    if !timer.should_run() {
        return;
    }

    // Eminent Domain bypasses construction slowdown
    if policies.is_active(Policy::EminentDomain) {
        return;
    }

    if nimby.zone_changes.is_empty() {
        return;
    }

    for (building, mut construction) in &mut buildings {
        let opposition = nimby.opposition_at(building.grid_x, building.grid_y);
        let extra_ticks = construction_slowdown(opposition);
        if extra_ticks > 0 {
            construction.ticks_remaining += extra_ticks;
            construction.total_ticks += extra_ticks;
        }
    }
}
