use crate::buildings::Building;
use crate::config::{CELL_SIZE, GRID_HEIGHT, GRID_WIDTH};
use crate::services::{ServiceBuilding, ServiceType};
use bevy::prelude::*;

use super::{
    facility_capacity_tons, GarbageGrid, WasteCollectionGrid, WasteProducer, WasteSystem,
    TRANSPORT_COST_PER_TON_MILE, WASTE_SERVICE_RADIUS_CELLS,
};

/// Attaches `WasteProducer` components to buildings that don't have one yet.
/// Runs on the slow tick to avoid overhead every frame.
pub fn attach_waste_producers(
    slow_timer: Res<crate::SlowTickTimer>,
    mut commands: Commands,
    buildings_without: Query<(Entity, &Building), Without<WasteProducer>>,
    services_without: Query<(Entity, &ServiceBuilding), Without<WasteProducer>>,
    policies: Res<crate::policies::Policies>,
) {
    if !slow_timer.should_run() {
        return;
    }
    let recycling_active = policies.is_active(crate::policies::Policy::RecyclingProgram);

    for (entity, building) in &buildings_without {
        let mut producer = WasteProducer::for_building(building.zone_type, building.level);
        producer.recycling_participation = recycling_active;
        commands.entity(entity).insert(producer);
    }

    for (entity, service) in &services_without {
        let mut producer = WasteProducer::for_service(service.service_type);
        producer.recycling_participation = recycling_active;
        commands.entity(entity).insert(producer);
    }
}

/// Updates recycling participation on all WasteProducers when the recycling
/// policy changes. Runs on the slow tick.
pub fn sync_recycling_policy(
    slow_timer: Res<crate::SlowTickTimer>,
    policies: Res<crate::policies::Policies>,
    mut producers: Query<&mut WasteProducer>,
) {
    if !slow_timer.should_run() {
        return;
    }
    let recycling_active = policies.is_active(crate::policies::Policy::RecyclingProgram);
    for mut producer in &mut producers {
        producer.recycling_participation = recycling_active;
    }
}

/// Aggregates waste generation across all buildings and updates the `WasteSystem`
/// resource with totals and per-capita metrics.
///
/// Runs on the slow tick (every ~10 seconds of game time).
/// The slow tick interval is 100 ticks at 10Hz = 10 game-seconds.
/// We treat each slow tick as representing roughly 1 game-day for waste calculations.
pub fn update_waste_generation(
    slow_timer: Res<crate::SlowTickTimer>,
    mut waste_system: ResMut<WasteSystem>,
    building_producers: Query<(&Building, &WasteProducer)>,
    service_producers: Query<(&ServiceBuilding, &WasteProducer)>,
    stats: Res<crate::stats::CityStats>,
) {
    if !slow_timer.should_run() {
        return;
    }

    let mut total_waste_lbs: f64 = 0.0;
    let mut recycling_count = 0u32;
    let mut producer_count = 0u32;

    // Zoned buildings (residential, commercial, industrial, office)
    for (building, producer) in &building_producers {
        let is_residential = building.zone_type.is_residential();
        let daily_waste = producer.effective_daily_waste(building.occupants, is_residential);
        total_waste_lbs += daily_waste as f64;
        producer_count += 1;
        if producer.recycling_participation {
            recycling_count += 1;
        }
    }

    // Service buildings (hospitals, schools, etc.)
    for (_service, producer) in &service_producers {
        let daily_waste = producer.effective_daily_waste(0, false);
        total_waste_lbs += daily_waste as f64;
        producer_count += 1;
        if producer.recycling_participation {
            recycling_count += 1;
        }
    }

    // Convert lbs to tons (1 ton = 2000 lbs)
    let period_tons = total_waste_lbs / 2000.0;

    let population = stats.population;
    let per_capita = if population > 0 {
        total_waste_lbs as f32 / population as f32
    } else {
        0.0
    };

    waste_system.period_generated_tons = period_tons;
    waste_system.total_generated_tons += period_tons;
    waste_system.per_capita_lbs_per_day = per_capita;
    waste_system.tracked_population = population;
    waste_system.recycling_buildings = recycling_count;
    waste_system.total_producers = producer_count;
}

/// Updates waste collection coverage and statistics (WASTE-003).
///
/// For each waste collection facility (transfer station, landfill, recycling center,
/// incinerator), marks cells within service radius as covered. Then computes the
/// collection rate as `min(1.0, total_capacity / total_generated)`, and accumulates
/// uncollected waste at uncovered buildings.
///
/// Closer buildings are implicitly served first (capacity-based, not per-truck).
/// Overlapping service areas do not double-count capacity.
#[allow(clippy::too_many_arguments)]
pub fn update_waste_collection(
    slow_timer: Res<crate::SlowTickTimer>,
    mut waste_system: ResMut<WasteSystem>,
    mut collection_grid: ResMut<WasteCollectionGrid>,
    waste_services: Query<&ServiceBuilding>,
    building_producers: Query<(&Building, &WasteProducer)>,
    service_producers: Query<(&ServiceBuilding, &WasteProducer)>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // Phase 1: Rebuild coverage grid from waste service buildings.
    collection_grid.clear_coverage();
    let mut total_capacity: f64 = 0.0;
    let mut facility_count = 0u32;

    for service in &waste_services {
        if !ServiceBuilding::is_garbage(service.service_type) {
            continue;
        }
        total_capacity += facility_capacity_tons(service.service_type);
        facility_count += 1;

        let radius = WASTE_SERVICE_RADIUS_CELLS;
        let sx = service.grid_x as i32;
        let sy = service.grid_y as i32;
        let r2 = (radius as f32 * CELL_SIZE) * (radius as f32 * CELL_SIZE);

        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let cx = sx + dx;
                let cy = sy + dy;
                if cx < 0 || cy < 0 || cx >= GRID_WIDTH as i32 || cy >= GRID_HEIGHT as i32 {
                    continue;
                }
                let wx_diff = dx as f32 * CELL_SIZE;
                let wy_diff = dy as f32 * CELL_SIZE;
                if wx_diff * wx_diff + wy_diff * wy_diff > r2 {
                    continue;
                }
                let idx = cy as usize * collection_grid.width + cx as usize;
                collection_grid.coverage[idx] = collection_grid.coverage[idx].saturating_add(1);
            }
        }
    }

    // Phase 2: Compute total waste generated by all buildings this period (lbs).
    let mut total_generated_lbs: f64 = 0.0;
    let mut uncovered_buildings = 0u32;

    // Collect per-building waste and coverage status for zoned buildings.
    for (building, producer) in &building_producers {
        let is_residential = building.zone_type.is_residential();
        let daily_lbs = producer.effective_daily_waste(building.occupants, is_residential) as f64;
        total_generated_lbs += daily_lbs;

        let covered = collection_grid.is_covered(building.grid_x, building.grid_y);
        if !covered {
            uncovered_buildings += 1;
        }
    }

    // Service buildings that produce waste.
    for (service, producer) in &service_producers {
        let daily_lbs = producer.effective_daily_waste(0, false) as f64;
        total_generated_lbs += daily_lbs;

        let covered = collection_grid.is_covered(service.grid_x, service.grid_y);
        if !covered {
            uncovered_buildings += 1;
        }
    }

    let total_generated_tons = total_generated_lbs / 2000.0;

    // Phase 3: Compute collection rate.
    let collection_rate = if total_generated_tons > 0.0 {
        (total_capacity / total_generated_tons).min(1.0)
    } else {
        1.0 // nothing to collect
    };

    let total_collected_tons = total_generated_tons * collection_rate;

    // Phase 4: Accumulate uncollected waste at uncovered building locations.
    // For covered buildings, reduce uncollected waste proportional to collection rate.
    // For uncovered buildings, all waste accumulates.
    for (building, producer) in &building_producers {
        let is_residential = building.zone_type.is_residential();
        let daily_lbs = producer.effective_daily_waste(building.occupants, is_residential);
        let idx = building.grid_y * collection_grid.width + building.grid_x;
        let covered = collection_grid.coverage[idx] > 0;

        if covered {
            // Covered: only uncollected fraction accumulates, and collected fraction decays.
            let uncollected_fraction = 1.0 - collection_rate as f32;
            collection_grid.uncollected_lbs[idx] += daily_lbs * uncollected_fraction;
            // Decay: collection picks up some accumulated waste too.
            collection_grid.uncollected_lbs[idx] *= 1.0 - collection_rate as f32 * 0.5;
        } else {
            // Not covered: all waste accumulates.
            collection_grid.uncollected_lbs[idx] += daily_lbs;
        }
        // Cap uncollected waste to prevent unbounded accumulation.
        collection_grid.uncollected_lbs[idx] = collection_grid.uncollected_lbs[idx].min(10_000.0);
    }

    for (service, producer) in &service_producers {
        let daily_lbs = producer.effective_daily_waste(0, false);
        let idx = service.grid_y * collection_grid.width + service.grid_x;
        let covered = collection_grid.coverage[idx] > 0;

        if covered {
            let uncollected_fraction = 1.0 - collection_rate as f32;
            collection_grid.uncollected_lbs[idx] += daily_lbs * uncollected_fraction;
            collection_grid.uncollected_lbs[idx] *= 1.0 - collection_rate as f32 * 0.5;
        } else {
            collection_grid.uncollected_lbs[idx] += daily_lbs;
        }
        collection_grid.uncollected_lbs[idx] = collection_grid.uncollected_lbs[idx].min(10_000.0);
    }

    // Phase 5: Compute transport cost (simplified: total_collected * cost_per_ton_mile * avg_distance).
    // Average distance approximated as half the service radius in cells, converted to miles.
    // 1 cell = CELL_SIZE world units. Assume 1 world unit ~ 1 meter, so CELL_SIZE meters per cell.
    let avg_distance_cells = WASTE_SERVICE_RADIUS_CELLS as f64 / 2.0;
    let avg_distance_miles = avg_distance_cells * CELL_SIZE as f64 / 1609.0; // meters to miles
    let transport_cost = total_collected_tons * TRANSPORT_COST_PER_TON_MILE * avg_distance_miles;

    // Phase 6: Update WasteSystem resource.
    waste_system.total_collected_tons = total_collected_tons;
    waste_system.total_capacity_tons = total_capacity;
    waste_system.collection_rate = collection_rate;
    waste_system.uncovered_buildings = uncovered_buildings;
    waste_system.transport_cost = transport_cost;
    waste_system.active_facilities = facility_count;
}

pub fn update_garbage(
    slow_timer: Res<crate::SlowTickTimer>,
    mut garbage: ResMut<GarbageGrid>,
    buildings: Query<(&Building, Option<&WasteProducer>)>,
    services: Query<&ServiceBuilding>,
    policies: Res<crate::policies::Policies>,
) {
    if !slow_timer.should_run() {
        return;
    }
    // Buildings produce garbage proportional to waste generation rate or occupants
    let garbage_mult = policies.garbage_multiplier();
    for (building, maybe_producer) in &buildings {
        let production = if let Some(producer) = maybe_producer {
            // Use the detailed waste rate: convert lbs/day to grid units
            // Scale down so the grid u8 stays in a reasonable range
            let is_residential = building.zone_type.is_residential();
            let daily_lbs = producer.effective_daily_waste(building.occupants, is_residential);
            // Map ~0-2000 lbs/day range down to 0-10 grid units
            ((daily_lbs / 200.0).min(10.0) * garbage_mult) as u8
        } else {
            // Fallback: original formula for buildings without WasteProducer yet
            ((building.occupants / 5).min(10) as f32 * garbage_mult) as u8
        };
        let cur = garbage.get(building.grid_x, building.grid_y);
        garbage.set(
            building.grid_x,
            building.grid_y,
            cur.saturating_add(production),
        );
    }

    // Garbage service buildings collect in radius
    for service in &services {
        if !ServiceBuilding::is_garbage(service.service_type) {
            continue;
        }
        let radius = (service.radius / 16.0) as i32;
        let collection = match service.service_type {
            ServiceType::Landfill => 3u8,
            ServiceType::RecyclingCenter => 5u8,
            ServiceType::Incinerator => 8u8,
            ServiceType::TransferStation => 4u8,
            _ => 0,
        };
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let nx = service.grid_x as i32 + dx;
                let ny = service.grid_y as i32 + dy;
                if nx >= 0 && ny >= 0 && (nx as usize) < GRID_WIDTH && (ny as usize) < GRID_HEIGHT {
                    let cur = garbage.get(nx as usize, ny as usize);
                    garbage.set(nx as usize, ny as usize, cur.saturating_sub(collection));
                }
            }
        }
    }
}

pub struct GarbagePlugin;

impl Plugin for GarbagePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GarbageGrid>()
            .init_resource::<WasteSystem>()
            .init_resource::<WasteCollectionGrid>()
            .add_systems(
                FixedUpdate,
                (
                    attach_waste_producers,
                    bevy::ecs::schedule::apply_deferred,
                    sync_recycling_policy,
                    update_garbage,
                    update_waste_generation,
                    update_waste_collection,
                )
                    .chain()
                    .after(crate::land_value::update_land_value)
                    .in_set(crate::SimulationSet::Simulation),
            );
    }
}
