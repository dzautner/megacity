//! Transit Hub / Multi-Modal Stations (TRAF-015).
//!
//! Multi-modal transit stations combine multiple transit types at a single
//! location, serving as transfer points with reduced transfer penalties.
//!
//! ## Hub Types
//! - **BusMetroHub**: Combined bus stop and metro station
//! - **TrainMetroHub**: Combined train and metro station
//! - **MultiModalHub**: All transit types at one location
//!
//! ## Transfer Penalties
//! - Default transfer between modes: 3 minutes
//! - Hub reduces to 1 minute between co-located modes
//!
//! ## Land Value
//! Hubs provide a 1.5x land value boost compared to individual stations.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::land_value::LandValueGrid;
use crate::services::{ServiceBuilding, ServiceType};
use crate::SlowTickTimer;

// =============================================================================
// Constants
// =============================================================================

/// Default transfer penalty between transit modes (in minutes).
pub const DEFAULT_TRANSFER_PENALTY_MINUTES: f32 = 3.0;

/// Reduced transfer penalty at hub locations (in minutes).
pub const HUB_TRANSFER_PENALTY_MINUTES: f32 = 1.0;

/// Land value boost multiplier for hubs relative to individual stations.
/// Individual transit stations give a base boost; hubs multiply it by this factor.
pub const HUB_LAND_VALUE_MULTIPLIER: f32 = 1.5;

/// Base land value boost for an individual transit station (in raw land value units).
pub const TRANSIT_STATION_BASE_BOOST: i32 = 10;

/// Radius (in cells) within which a hub boosts land value.
pub const HUB_LAND_VALUE_RADIUS: i32 = 8;

/// Radius (in cells) within which co-located transit stops form a hub.
pub const HUB_DETECTION_RADIUS: i32 = 2;

// =============================================================================
// Transit Mode
// =============================================================================

/// Individual transit modes that can be combined at a hub.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub enum TransitMode {
    Bus,
    Metro,
    Train,
    Tram,
    Ferry,
}

impl TransitMode {
    /// Convert a service type to a transit mode, if applicable.
    pub fn from_service_type(st: ServiceType) -> Option<Self> {
        match st {
            ServiceType::BusDepot => Some(TransitMode::Bus),
            ServiceType::SubwayStation => Some(TransitMode::Metro),
            ServiceType::TrainStation => Some(TransitMode::Train),
            ServiceType::TramDepot => Some(TransitMode::Tram),
            ServiceType::FerryPier => Some(TransitMode::Ferry),
            _ => None,
        }
    }
}

// =============================================================================
// Hub Type
// =============================================================================

/// Type of multi-modal transit hub.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub enum TransitHubType {
    /// Combined bus stop and metro station.
    BusMetroHub,
    /// Combined train and metro station.
    TrainMetroHub,
    /// All transit types at one location.
    MultiModalHub,
}

impl TransitHubType {
    /// Returns the set of transit modes supported by this hub type.
    pub fn supported_modes(&self) -> Vec<TransitMode> {
        match self {
            TransitHubType::BusMetroHub => vec![TransitMode::Bus, TransitMode::Metro],
            TransitHubType::TrainMetroHub => vec![TransitMode::Train, TransitMode::Metro],
            TransitHubType::MultiModalHub => vec![
                TransitMode::Bus,
                TransitMode::Metro,
                TransitMode::Train,
                TransitMode::Tram,
                TransitMode::Ferry,
            ],
        }
    }

    /// Determine the hub type from a set of co-located transit modes.
    /// Returns `None` if fewer than 2 modes are present.
    pub fn from_modes(modes: &[TransitMode]) -> Option<Self> {
        if modes.len() < 2 {
            return None;
        }

        let has_bus = modes.contains(&TransitMode::Bus);
        let has_metro = modes.contains(&TransitMode::Metro);
        let has_train = modes.contains(&TransitMode::Train);

        // If 3+ modes, it's a multi-modal hub
        if modes.len() >= 3 {
            return Some(TransitHubType::MultiModalHub);
        }

        // 2 modes: check specific combinations
        if has_bus && has_metro {
            Some(TransitHubType::BusMetroHub)
        } else if has_train && has_metro {
            Some(TransitHubType::TrainMetroHub)
        } else {
            // Any other combination of 2 modes is still a valid hub
            // (e.g., Bus+Train). Classify as MultiModalHub for simplicity.
            Some(TransitHubType::MultiModalHub)
        }
    }

    /// The transfer penalty reduction factor for this hub type.
    /// Penalty = DEFAULT * (1 - reduction). For standard hubs the penalty
    /// drops from 3min to 1min, so the reduction is ~0.667.
    pub fn transfer_penalty_reduction(&self) -> f32 {
        1.0 - (HUB_TRANSFER_PENALTY_MINUTES / DEFAULT_TRANSFER_PENALTY_MINUTES)
    }
}

// =============================================================================
// Component: TransitHub
// =============================================================================

/// ECS component marking an entity as a transit hub.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct TransitHub {
    pub hub_type: TransitHubType,
    pub supported_modes: Vec<TransitMode>,
    /// Reduction applied to transfer penalty (0.0 = no reduction, 1.0 = free transfer).
    pub transfer_penalty_reduction: f32,
    /// Grid coordinates of the hub center.
    pub grid_x: usize,
    pub grid_y: usize,
}

impl TransitHub {
    /// Create a new transit hub at the given location.
    pub fn new(hub_type: TransitHubType, grid_x: usize, grid_y: usize) -> Self {
        Self {
            supported_modes: hub_type.supported_modes(),
            transfer_penalty_reduction: hub_type.transfer_penalty_reduction(),
            hub_type,
            grid_x,
            grid_y,
        }
    }

    /// Get the effective transfer penalty in minutes when transferring
    /// between two modes at this hub. Returns the default penalty if one
    /// of the modes isn't supported by this hub.
    pub fn effective_transfer_penalty(&self, from: TransitMode, to: TransitMode) -> f32 {
        if self.supported_modes.contains(&from) && self.supported_modes.contains(&to) {
            HUB_TRANSFER_PENALTY_MINUTES
        } else {
            DEFAULT_TRANSFER_PENALTY_MINUTES
        }
    }
}

// =============================================================================
// Resource: TransitHubs (registry)
// =============================================================================

/// Registry tracking all transit hub locations and their detected modes.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize, Encode, Decode)]
pub struct TransitHubs {
    /// Hub entries keyed by (grid_x, grid_y).
    pub hubs: Vec<TransitHubEntry>,
}

/// A single hub entry in the registry.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct TransitHubEntry {
    pub grid_x: usize,
    pub grid_y: usize,
    pub hub_type: TransitHubType,
    pub modes: Vec<TransitMode>,
}

impl TransitHubs {
    /// Find a hub at or near the given coordinates (within detection radius).
    pub fn find_hub_near(&self, x: usize, y: usize) -> Option<&TransitHubEntry> {
        self.hubs.iter().find(|h| {
            let dx = (h.grid_x as i32 - x as i32).unsigned_abs() as usize;
            let dy = (h.grid_y as i32 - y as i32).unsigned_abs() as usize;
            dx <= HUB_DETECTION_RADIUS as usize && dy <= HUB_DETECTION_RADIUS as usize
        })
    }

    /// Get the transfer penalty between two modes at a location.
    /// Returns the hub-reduced penalty if a hub exists, otherwise the default.
    pub fn transfer_penalty_at(
        &self,
        x: usize,
        y: usize,
        from: TransitMode,
        to: TransitMode,
    ) -> f32 {
        if let Some(hub) = self.find_hub_near(x, y) {
            if hub.modes.contains(&from) && hub.modes.contains(&to) {
                return HUB_TRANSFER_PENALTY_MINUTES;
            }
        }
        DEFAULT_TRANSFER_PENALTY_MINUTES
    }
}

// =============================================================================
// Resource: TransitHubStats
// =============================================================================

/// Aggregated statistics about transit hub usage.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize, Encode, Decode)]
pub struct TransitHubStats {
    /// Total number of detected hubs.
    pub total_hubs: u32,
    /// Number of BusMetroHub hubs.
    pub bus_metro_hubs: u32,
    /// Number of TrainMetroHub hubs.
    pub train_metro_hubs: u32,
    /// Number of MultiModalHub hubs.
    pub multi_modal_hubs: u32,
    /// Total transfers tracked at hubs this cycle.
    pub total_transfers: u64,
    /// Average transfer penalty across all hubs (in minutes).
    pub avg_transfer_penalty: f32,
}

// =============================================================================
// Saveable implementations
// =============================================================================

impl crate::Saveable for TransitHubs {
    const SAVE_KEY: &'static str = "transit_hubs";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.hubs.is_empty() {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

impl crate::Saveable for TransitHubStats {
    const SAVE_KEY: &'static str = "transit_hub_stats";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.total_hubs == 0 {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

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

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // TransitMode tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_transit_mode_from_service_type() {
        assert_eq!(
            TransitMode::from_service_type(ServiceType::BusDepot),
            Some(TransitMode::Bus)
        );
        assert_eq!(
            TransitMode::from_service_type(ServiceType::SubwayStation),
            Some(TransitMode::Metro)
        );
        assert_eq!(
            TransitMode::from_service_type(ServiceType::TrainStation),
            Some(TransitMode::Train)
        );
        assert_eq!(
            TransitMode::from_service_type(ServiceType::TramDepot),
            Some(TransitMode::Tram)
        );
        assert_eq!(
            TransitMode::from_service_type(ServiceType::FerryPier),
            Some(TransitMode::Ferry)
        );
        assert_eq!(
            TransitMode::from_service_type(ServiceType::FireStation),
            None
        );
    }

    // -------------------------------------------------------------------------
    // TransitHubType tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_hub_type_from_modes_bus_metro() {
        let modes = vec![TransitMode::Bus, TransitMode::Metro];
        assert_eq!(
            TransitHubType::from_modes(&modes),
            Some(TransitHubType::BusMetroHub)
        );
    }

    #[test]
    fn test_hub_type_from_modes_train_metro() {
        let modes = vec![TransitMode::Train, TransitMode::Metro];
        assert_eq!(
            TransitHubType::from_modes(&modes),
            Some(TransitHubType::TrainMetroHub)
        );
    }

    #[test]
    fn test_hub_type_from_modes_multi_modal() {
        let modes = vec![TransitMode::Bus, TransitMode::Metro, TransitMode::Train];
        assert_eq!(
            TransitHubType::from_modes(&modes),
            Some(TransitHubType::MultiModalHub)
        );
    }

    #[test]
    fn test_hub_type_from_modes_single_returns_none() {
        let modes = vec![TransitMode::Bus];
        assert_eq!(TransitHubType::from_modes(&modes), None);
    }

    #[test]
    fn test_hub_type_from_modes_empty_returns_none() {
        let modes: Vec<TransitMode> = vec![];
        assert_eq!(TransitHubType::from_modes(&modes), None);
    }

    #[test]
    fn test_hub_type_supported_modes() {
        let bm = TransitHubType::BusMetroHub.supported_modes();
        assert!(bm.contains(&TransitMode::Bus));
        assert!(bm.contains(&TransitMode::Metro));
        assert_eq!(bm.len(), 2);

        let tm = TransitHubType::TrainMetroHub.supported_modes();
        assert!(tm.contains(&TransitMode::Train));
        assert!(tm.contains(&TransitMode::Metro));
        assert_eq!(tm.len(), 2);

        let mm = TransitHubType::MultiModalHub.supported_modes();
        assert!(mm.len() >= 3);
    }

    // -------------------------------------------------------------------------
    // TransitHub component tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_transit_hub_effective_penalty_supported_modes() {
        let hub = TransitHub::new(TransitHubType::BusMetroHub, 10, 10);
        let penalty = hub.effective_transfer_penalty(TransitMode::Bus, TransitMode::Metro);
        assert!((penalty - HUB_TRANSFER_PENALTY_MINUTES).abs() < f32::EPSILON);
    }

    #[test]
    fn test_transit_hub_effective_penalty_unsupported_mode() {
        let hub = TransitHub::new(TransitHubType::BusMetroHub, 10, 10);
        let penalty = hub.effective_transfer_penalty(TransitMode::Bus, TransitMode::Train);
        assert!((penalty - DEFAULT_TRANSFER_PENALTY_MINUTES).abs() < f32::EPSILON);
    }

    #[test]
    fn test_transit_hub_penalty_reduction() {
        let hub = TransitHub::new(TransitHubType::BusMetroHub, 10, 10);
        // Reduction should be ~0.667 (from 3min to 1min)
        let expected = 1.0 - (HUB_TRANSFER_PENALTY_MINUTES / DEFAULT_TRANSFER_PENALTY_MINUTES);
        assert!((hub.transfer_penalty_reduction - expected).abs() < 0.01);
    }

    // -------------------------------------------------------------------------
    // TransitHubs registry tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_transit_hubs_find_hub_near() {
        let mut registry = TransitHubs::default();
        registry.hubs.push(TransitHubEntry {
            grid_x: 50,
            grid_y: 50,
            hub_type: TransitHubType::BusMetroHub,
            modes: vec![TransitMode::Bus, TransitMode::Metro],
        });

        // Exact location
        assert!(registry.find_hub_near(50, 50).is_some());
        // Within detection radius
        assert!(registry.find_hub_near(51, 51).is_some());
        // Outside detection radius
        assert!(registry.find_hub_near(60, 60).is_none());
    }

    #[test]
    fn test_transfer_penalty_at_hub() {
        let mut registry = TransitHubs::default();
        registry.hubs.push(TransitHubEntry {
            grid_x: 50,
            grid_y: 50,
            hub_type: TransitHubType::BusMetroHub,
            modes: vec![TransitMode::Bus, TransitMode::Metro],
        });

        let penalty = registry.transfer_penalty_at(50, 50, TransitMode::Bus, TransitMode::Metro);
        assert!((penalty - HUB_TRANSFER_PENALTY_MINUTES).abs() < f32::EPSILON);

        // Unsupported mode pair at hub
        let penalty = registry.transfer_penalty_at(50, 50, TransitMode::Bus, TransitMode::Train);
        assert!((penalty - DEFAULT_TRANSFER_PENALTY_MINUTES).abs() < f32::EPSILON);

        // No hub at location
        let penalty = registry.transfer_penalty_at(100, 100, TransitMode::Bus, TransitMode::Metro);
        assert!((penalty - DEFAULT_TRANSFER_PENALTY_MINUTES).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------------------------
    // Saveable trait tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_saveable_transit_hubs_skips_default() {
        use crate::Saveable;
        let hubs = TransitHubs::default();
        assert!(hubs.save_to_bytes().is_none());
    }

    #[test]
    fn test_saveable_transit_hubs_saves_when_populated() {
        use crate::Saveable;
        let mut hubs = TransitHubs::default();
        hubs.hubs.push(TransitHubEntry {
            grid_x: 10,
            grid_y: 20,
            hub_type: TransitHubType::BusMetroHub,
            modes: vec![TransitMode::Bus, TransitMode::Metro],
        });
        assert!(hubs.save_to_bytes().is_some());
    }

    #[test]
    fn test_saveable_transit_hubs_roundtrip() {
        use crate::Saveable;
        let mut hubs = TransitHubs::default();
        hubs.hubs.push(TransitHubEntry {
            grid_x: 10,
            grid_y: 20,
            hub_type: TransitHubType::BusMetroHub,
            modes: vec![TransitMode::Bus, TransitMode::Metro],
        });
        let bytes = hubs.save_to_bytes().expect("should serialize");
        let restored = TransitHubs::load_from_bytes(&bytes);
        assert_eq!(restored.hubs.len(), 1);
        assert_eq!(restored.hubs[0].grid_x, 10);
        assert_eq!(restored.hubs[0].grid_y, 20);
        assert_eq!(restored.hubs[0].hub_type, TransitHubType::BusMetroHub);
    }

    #[test]
    fn test_saveable_transit_hub_stats_skips_default() {
        use crate::Saveable;
        let stats = TransitHubStats::default();
        assert!(stats.save_to_bytes().is_none());
    }

    #[test]
    fn test_saveable_transit_hub_stats_saves_when_nonzero() {
        use crate::Saveable;
        let stats = TransitHubStats {
            total_hubs: 3,
            ..Default::default()
        };
        assert!(stats.save_to_bytes().is_some());
    }

    #[test]
    fn test_saveable_keys() {
        use crate::Saveable;
        assert_eq!(TransitHubs::SAVE_KEY, "transit_hubs");
        assert_eq!(TransitHubStats::SAVE_KEY, "transit_hub_stats");
    }

    // -------------------------------------------------------------------------
    // Constant verification tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_constants() {
        assert!((DEFAULT_TRANSFER_PENALTY_MINUTES - 3.0).abs() < f32::EPSILON);
        assert!((HUB_TRANSFER_PENALTY_MINUTES - 1.0).abs() < f32::EPSILON);
        assert!((HUB_LAND_VALUE_MULTIPLIER - 1.5).abs() < f32::EPSILON);
        assert!(HUB_DETECTION_RADIUS > 0);
        assert!(HUB_LAND_VALUE_RADIUS > 0);
    }

    // -------------------------------------------------------------------------
    // Hub detection edge cases
    // -------------------------------------------------------------------------

    #[test]
    fn test_hub_type_two_non_standard_modes() {
        // Bus + Tram: not a standard named pair, classified as MultiModalHub
        let modes = vec![TransitMode::Bus, TransitMode::Tram];
        assert_eq!(
            TransitHubType::from_modes(&modes),
            Some(TransitHubType::MultiModalHub)
        );
    }

    #[test]
    fn test_hub_land_value_boost_exceeds_individual() {
        let hub_boost = (TRANSIT_STATION_BASE_BOOST as f32 * HUB_LAND_VALUE_MULTIPLIER) as i32;
        assert!(
            hub_boost > TRANSIT_STATION_BASE_BOOST,
            "Hub land value boost ({hub_boost}) should exceed individual station boost ({TRANSIT_STATION_BASE_BOOST})"
        );
    }
}
