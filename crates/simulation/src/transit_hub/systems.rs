//! Systems and plugin for transit hub detection, land value, and statistics.

use bevy::prelude::*;

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::land_value::LandValueGrid;
use crate::services::ServiceBuilding;
use crate::SlowTickTimer;

use super::{
    TransitHub, TransitHubEntry, TransitHubStats, TransitHubType, TransitHubs, TransitMode,
    DEFAULT_TRANSFER_PENALTY_MINUTES, HUB_DETECTION_RADIUS, HUB_LAND_VALUE_MULTIPLIER,
    HUB_LAND_VALUE_RADIUS, HUB_TRANSFER_PENALTY_MINUTES, TRANSIT_STATION_BASE_BOOST,
};

// =============================================================================
// Systems
// =============================================================================

/// Detect co-located transit stops and create/update transit hub entities.
///
/// Scans all `ServiceBuilding` entities that are transit-related, groups them
/// by proximity, and creates `TransitHub` components for locations with 2+
/// different transit modes within `HUB_DETECTION_RADIUS`.
#[allow(clippy::too_many_arguments)]
pub fn update_transit_hubs(
    slow_timer: Res<SlowTickTimer>,
    mut hubs_registry: ResMut<TransitHubs>,
    services: Query<&ServiceBuilding>,
    mut commands: Commands,
    existing_hubs: Query<(Entity, &TransitHub)>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // Collect all transit stops with their positions and modes.
    let mut transit_stops: Vec<(usize, usize, TransitMode)> = Vec::new();
    for service in &services {
        if let Some(mode) = TransitMode::from_service_type(service.service_type) {
            transit_stops.push((service.grid_x, service.grid_y, mode));
        }
    }

    // Remove existing hub entities (we rebuild each cycle).
    for (entity, _) in &existing_hubs {
        commands.entity(entity).despawn();
    }

    // Group transit stops into clusters. For each stop, find all other stops
    // within detection radius and collect the unique modes.
    let mut hub_entries: Vec<TransitHubEntry> = Vec::new();
    let mut used: Vec<bool> = vec![false; transit_stops.len()];

    for i in 0..transit_stops.len() {
        if used[i] {
            continue;
        }

        let (cx, cy, mode_i) = transit_stops[i];
        let mut cluster_modes: Vec<TransitMode> = vec![mode_i];
        let mut cluster_indices: Vec<usize> = vec![i];

        for j in (i + 1)..transit_stops.len() {
            if used[j] {
                continue;
            }
            let (sx, sy, mode_j) = transit_stops[j];
            let dx = (cx as i32 - sx as i32).abs();
            let dy = (cy as i32 - sy as i32).abs();
            if dx <= HUB_DETECTION_RADIUS && dy <= HUB_DETECTION_RADIUS {
                if !cluster_modes.contains(&mode_j) {
                    cluster_modes.push(mode_j);
                }
                cluster_indices.push(j);
            }
        }

        // Only form a hub if 2+ different modes are co-located.
        if cluster_modes.len() >= 2 {
            for &idx in &cluster_indices {
                used[idx] = true;
            }

            if let Some(hub_type) = TransitHubType::from_modes(&cluster_modes) {
                let entry = TransitHubEntry {
                    grid_x: cx,
                    grid_y: cy,
                    hub_type,
                    modes: cluster_modes.clone(),
                };
                hub_entries.push(entry);

                // Spawn an ECS entity with the TransitHub component.
                commands.spawn(TransitHub::new(hub_type, cx, cy));
            }
        }
    }

    hubs_registry.hubs = hub_entries;
}

/// Apply land value boost from transit hubs.
///
/// Hubs provide a 1.5x multiplier on the base transit station land value boost,
/// applied within `HUB_LAND_VALUE_RADIUS` cells of the hub center.
pub fn transit_hub_land_value(
    slow_timer: Res<SlowTickTimer>,
    hubs: Res<TransitHubs>,
    mut land_value: ResMut<LandValueGrid>,
) {
    if !slow_timer.should_run() {
        return;
    }

    let boosted_value = (TRANSIT_STATION_BASE_BOOST as f32 * HUB_LAND_VALUE_MULTIPLIER) as i32;

    for hub in &hubs.hubs {
        let cx = hub.grid_x as i32;
        let cy = hub.grid_y as i32;

        for dy in -HUB_LAND_VALUE_RADIUS..=HUB_LAND_VALUE_RADIUS {
            for dx in -HUB_LAND_VALUE_RADIUS..=HUB_LAND_VALUE_RADIUS {
                let nx = cx + dx;
                let ny = cy + dy;
                if nx < 0 || ny < 0 || nx >= GRID_WIDTH as i32 || ny >= GRID_HEIGHT as i32 {
                    continue;
                }

                let dist = dx.abs() + dy.abs();
                let effect = (boosted_value - dist * 2).max(0);
                if effect > 0 {
                    let cur = land_value.get(nx as usize, ny as usize);
                    land_value.set(
                        nx as usize,
                        ny as usize,
                        (cur as i32 + effect).min(255) as u8,
                    );
                }
            }
        }
    }
}

/// Update hub statistics resource.
pub fn update_hub_stats(
    slow_timer: Res<SlowTickTimer>,
    hubs: Res<TransitHubs>,
    mut stats: ResMut<TransitHubStats>,
) {
    if !slow_timer.should_run() {
        return;
    }

    let mut bus_metro: u32 = 0;
    let mut train_metro: u32 = 0;
    let mut multi_modal: u32 = 0;

    for hub in &hubs.hubs {
        match hub.hub_type {
            TransitHubType::BusMetroHub => bus_metro += 1,
            TransitHubType::TrainMetroHub => train_metro += 1,
            TransitHubType::MultiModalHub => multi_modal += 1,
        }
    }

    let total = bus_metro + train_metro + multi_modal;

    stats.total_hubs = total;
    stats.bus_metro_hubs = bus_metro;
    stats.train_metro_hubs = train_metro;
    stats.multi_modal_hubs = multi_modal;

    // Average transfer penalty: hub locations use reduced penalty.
    if total > 0 {
        stats.avg_transfer_penalty = HUB_TRANSFER_PENALTY_MINUTES;
    } else {
        stats.avg_transfer_penalty = DEFAULT_TRANSFER_PENALTY_MINUTES;
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct TransitHubPlugin;

impl Plugin for TransitHubPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TransitHubs>()
            .init_resource::<TransitHubStats>()
            .add_systems(
                FixedUpdate,
                (
                    update_transit_hubs,
                    transit_hub_land_value.after(crate::land_value::update_land_value),
                    update_hub_stats.after(update_transit_hubs),
                ),
            );

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<TransitHubs>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<TransitHubStats>();
    }
}
