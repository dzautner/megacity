//! Data-driven game parameters (UI-003).
//!
//! Extracts hardcoded simulation constants into a single [`GameParams`] resource
//! so they can be tuned at runtime without recompilation. The resource is
//! registered via the `Saveable` trait so parameter overrides persist across
//! save/load cycles.
//!
//! Systems that previously used module-level constants now read from
//! `Res<GameParams>` instead.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Road type parameters
// ---------------------------------------------------------------------------

/// Per-road-type tunables. Each road type (Local, Avenue, etc.) has one entry.
#[derive(Debug, Clone, Serialize, Deserialize, bitcode::Encode, bitcode::Decode)]
pub struct RoadTypeParams {
    /// Movement speed in pixels/second.
    pub speed: f32,
    /// Construction cost per cell.
    pub cost: f64,
    /// Vehicle capacity for BPR congestion model.
    pub capacity: u32,
    /// Monthly maintenance cost per cell.
    pub maintenance_cost: f64,
    /// Noise pollution radius in cells.
    pub noise_radius: u8,
}

/// Parameters for all six road types, keyed by variant name.
#[derive(Debug, Clone, Serialize, Deserialize, bitcode::Encode, bitcode::Decode)]
pub struct RoadParams {
    pub local: RoadTypeParams,
    pub avenue: RoadTypeParams,
    pub boulevard: RoadTypeParams,
    pub highway: RoadTypeParams,
    pub one_way: RoadTypeParams,
    pub path: RoadTypeParams,
}

impl Default for RoadParams {
    fn default() -> Self {
        Self {
            local: RoadTypeParams {
                speed: 30.0,
                cost: 10.0,
                capacity: 20,
                maintenance_cost: 0.3,
                noise_radius: 2,
            },
            avenue: RoadTypeParams {
                speed: 50.0,
                cost: 20.0,
                capacity: 40,
                maintenance_cost: 0.5,
                noise_radius: 3,
            },
            boulevard: RoadTypeParams {
                speed: 60.0,
                cost: 30.0,
                capacity: 60,
                maintenance_cost: 1.5,
                noise_radius: 4,
            },
            highway: RoadTypeParams {
                speed: 100.0,
                cost: 40.0,
                capacity: 80,
                maintenance_cost: 2.0,
                noise_radius: 8,
            },
            one_way: RoadTypeParams {
                speed: 40.0,
                cost: 15.0,
                capacity: 25,
                maintenance_cost: 0.4,
                noise_radius: 2,
            },
            path: RoadTypeParams {
                speed: 5.0,
                cost: 5.0,
                capacity: 5,
                maintenance_cost: 0.1,
                noise_radius: 0,
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Economy parameters
// ---------------------------------------------------------------------------

/// Tunables for the city economy.
#[derive(Debug, Clone, Serialize, Deserialize, bitcode::Encode, bitcode::Decode)]
pub struct EconomyParams {
    /// Starting treasury for a new city.
    pub starting_treasury: f64,
    /// Default tax rate (0.0..1.0).
    pub default_tax_rate: f32,
    /// Tax collection interval in game-days.
    pub tax_collection_interval_days: u32,
}

impl Default for EconomyParams {
    fn default() -> Self {
        Self {
            starting_treasury: 10_000.0,
            default_tax_rate: 0.10,
            tax_collection_interval_days: 30,
        }
    }
}

// ---------------------------------------------------------------------------
// Citizen movement parameters
// ---------------------------------------------------------------------------

/// Tunables for citizen movement and activity durations.
#[derive(Debug, Clone, Serialize, Deserialize, bitcode::Encode, bitcode::Decode)]
pub struct CitizenParams {
    /// Base movement speed in pixels per second.
    pub speed: f32,
    /// How many ticks a citizen spends shopping.
    pub shopping_duration_ticks: u32,
    /// How many ticks a citizen spends at leisure.
    pub leisure_duration_ticks: u32,
    /// School start hour (0-23).
    pub school_hours_start: u32,
    /// School end hour (0-23).
    pub school_hours_end: u32,
}

impl Default for CitizenParams {
    fn default() -> Self {
        Self {
            speed: 48.0,
            shopping_duration_ticks: 30,
            leisure_duration_ticks: 60,
            school_hours_start: 8,
            school_hours_end: 15,
        }
    }
}

// ---------------------------------------------------------------------------
// Building parameters
// ---------------------------------------------------------------------------

/// Tunables for building construction and spawning.
#[derive(Debug, Clone, Serialize, Deserialize, bitcode::Encode, bitcode::Decode)]
pub struct BuildingParams {
    /// Ticks between building spawn attempts.
    pub spawn_interval_ticks: u32,
    /// Base construction time in ticks (~100 = 10 seconds at 10Hz).
    pub construction_ticks: u32,
    /// Maximum buildings a spawner can place per zone per tick.
    pub max_buildings_per_zone_per_tick: u32,
}

impl Default for BuildingParams {
    fn default() -> Self {
        Self {
            spawn_interval_ticks: 2,
            construction_ticks: 100,
            max_buildings_per_zone_per_tick: 50,
        }
    }
}

// ---------------------------------------------------------------------------
// Citizen spawner parameters
// ---------------------------------------------------------------------------

/// Tunables for the citizen spawner system.
#[derive(Debug, Clone, Serialize, Deserialize, bitcode::Encode, bitcode::Decode)]
pub struct CitizenSpawnerParams {
    /// Ticks between spawn attempts.
    pub spawn_interval_ticks: u32,
    /// Normal max citizens to spawn per tick.
    pub max_spawn_per_tick: u32,
    /// Burst-mode max citizens per tick (when pop << capacity).
    pub burst_spawn_per_tick: u32,
}

impl Default for CitizenSpawnerParams {
    fn default() -> Self {
        Self {
            spawn_interval_ticks: 5,
            max_spawn_per_tick: 200,
            burst_spawn_per_tick: 5000,
        }
    }
}

// ---------------------------------------------------------------------------
// Zone demand parameters
// ---------------------------------------------------------------------------

/// Tunables for zone demand computation.
#[derive(Debug, Clone, Serialize, Deserialize, bitcode::Encode, bitcode::Decode)]
pub struct ZoneDemandParams {
    /// Natural vacancy rate range for residential zones (low, high).
    pub natural_vacancy_residential: (f32, f32),
    /// Natural vacancy rate range for commercial zones (low, high).
    pub natural_vacancy_commercial: (f32, f32),
    /// Natural vacancy rate range for industrial zones (low, high).
    pub natural_vacancy_industrial: (f32, f32),
    /// Natural vacancy rate range for office zones (low, high).
    pub natural_vacancy_office: (f32, f32),
    /// Damping factor applied to demand changes each tick (0.0..1.0).
    pub damping: f32,
    /// Bootstrap demand when roads exist but no buildings have been built.
    pub bootstrap_demand: f32,
}

impl Default for ZoneDemandParams {
    fn default() -> Self {
        Self {
            natural_vacancy_residential: (0.05, 0.07),
            natural_vacancy_commercial: (0.05, 0.08),
            natural_vacancy_industrial: (0.05, 0.08),
            natural_vacancy_office: (0.08, 0.12),
            damping: 0.15,
            bootstrap_demand: 0.5,
        }
    }
}

// ---------------------------------------------------------------------------
// Top-level GameParams resource
// ---------------------------------------------------------------------------

/// Central resource holding all data-driven game parameters.
///
/// Systems read from `Res<GameParams>` instead of hardcoded constants, allowing
/// runtime tuning and modding without recompilation.
#[derive(
    Resource, Debug, Clone, Serialize, Deserialize, bitcode::Encode, bitcode::Decode, Default,
)]
pub struct GameParams {
    pub road: RoadParams,
    pub economy: EconomyParams,
    pub citizen: CitizenParams,
    pub building: BuildingParams,
    pub citizen_spawner: CitizenSpawnerParams,
    pub zone_demand: ZoneDemandParams,
}

impl GameParams {
    /// Look up road parameters by `RoadType`.
    pub fn road_params(&self, road_type: crate::grid::RoadType) -> &RoadTypeParams {
        use crate::grid::RoadType;
        match road_type {
            RoadType::Local => &self.road.local,
            RoadType::Avenue => &self.road.avenue,
            RoadType::Boulevard => &self.road.boulevard,
            RoadType::Highway => &self.road.highway,
            RoadType::OneWay => &self.road.one_way,
            RoadType::Path => &self.road.path,
        }
    }
}

// ---------------------------------------------------------------------------
// Saveable implementation
// ---------------------------------------------------------------------------

impl crate::Saveable for GameParams {
    const SAVE_KEY: &'static str = "game_params";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        // Always save — even default params should be persisted so that saves
        // created with custom params are correctly restored.
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct GameParamsPlugin;

impl Plugin for GameParamsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameParams>();

        // Register for save/load via the SaveableRegistry
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<GameParams>();
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_params_match_original_constants() {
        let params = GameParams::default();

        // Economy
        assert_eq!(params.economy.starting_treasury, 10_000.0);
        assert!((params.economy.default_tax_rate - 0.10).abs() < f32::EPSILON);
        assert_eq!(params.economy.tax_collection_interval_days, 30);

        // Citizen movement
        assert!((params.citizen.speed - 48.0).abs() < f32::EPSILON);
        assert_eq!(params.citizen.shopping_duration_ticks, 30);
        assert_eq!(params.citizen.leisure_duration_ticks, 60);
        assert_eq!(params.citizen.school_hours_start, 8);
        assert_eq!(params.citizen.school_hours_end, 15);

        // Building
        assert_eq!(params.building.spawn_interval_ticks, 2);
        assert_eq!(params.building.construction_ticks, 100);
        assert_eq!(params.building.max_buildings_per_zone_per_tick, 50);

        // Citizen spawner
        assert_eq!(params.citizen_spawner.spawn_interval_ticks, 5);
        assert_eq!(params.citizen_spawner.max_spawn_per_tick, 200);
        assert_eq!(params.citizen_spawner.burst_spawn_per_tick, 5000);

        // Zone demand
        assert_eq!(params.zone_demand.natural_vacancy_residential, (0.05, 0.07));
        assert_eq!(params.zone_demand.natural_vacancy_commercial, (0.05, 0.08));
        assert_eq!(params.zone_demand.natural_vacancy_industrial, (0.05, 0.08));
        assert_eq!(params.zone_demand.natural_vacancy_office, (0.08, 0.12));
        assert!((params.zone_demand.damping - 0.15).abs() < f32::EPSILON);
        assert!((params.zone_demand.bootstrap_demand - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_road_params_lookup() {
        use crate::grid::RoadType;
        let params = GameParams::default();

        let local = params.road_params(RoadType::Local);
        assert!((local.speed - 30.0).abs() < f32::EPSILON);
        assert!((local.cost - 10.0).abs() < f64::EPSILON);
        assert_eq!(local.capacity, 20);
        assert!((local.maintenance_cost - 0.3).abs() < f64::EPSILON);
        assert_eq!(local.noise_radius, 2);

        let highway = params.road_params(RoadType::Highway);
        assert!((highway.speed - 100.0).abs() < f32::EPSILON);
        assert!((highway.cost - 40.0).abs() < f64::EPSILON);
        assert_eq!(highway.capacity, 80);
        assert!((highway.maintenance_cost - 2.0).abs() < f64::EPSILON);
        assert_eq!(highway.noise_radius, 8);
    }

    #[test]
    fn test_saveable_roundtrip() {
        let mut params = GameParams::default();
        params.economy.starting_treasury = 50_000.0;
        params.citizen.speed = 100.0;
        params.road.highway.speed = 200.0;

        let bytes = params.save_to_bytes().expect("should produce bytes");
        let restored = GameParams::load_from_bytes(&bytes);

        assert!((restored.economy.starting_treasury - 50_000.0).abs() < f64::EPSILON);
        assert!((restored.citizen.speed - 100.0).abs() < f32::EPSILON);
        assert!((restored.road.highway.speed - 200.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_default_road_params_match_grid_constants() {
        use crate::grid::RoadType;
        let params = GameParams::default();

        // Verify each road type's default params match the original hardcoded values
        let types = [
            (RoadType::Local, 30.0, 10.0, 20, 0.3, 2),
            (RoadType::Avenue, 50.0, 20.0, 40, 0.5, 3),
            (RoadType::Boulevard, 60.0, 30.0, 60, 1.5, 4),
            (RoadType::Highway, 100.0, 40.0, 80, 2.0, 8),
            (RoadType::OneWay, 40.0, 15.0, 25, 0.4, 2),
            (RoadType::Path, 5.0, 5.0, 5, 0.1, 0),
        ];

        for (rt, speed, cost, cap, maint, noise) in types {
            let rp = params.road_params(rt);
            assert!(
                (rp.speed - speed).abs() < f32::EPSILON,
                "{:?} speed mismatch",
                rt
            );
            assert!(
                (rp.cost - cost).abs() < f64::EPSILON,
                "{:?} cost mismatch",
                rt
            );
            assert_eq!(rp.capacity, cap, "{:?} capacity mismatch", rt);
            assert!(
                (rp.maintenance_cost - maint).abs() < f64::EPSILON,
                "{:?} maintenance mismatch",
                rt
            );
            assert_eq!(rp.noise_radius, noise, "{:?} noise_radius mismatch", rt);
        }
    }
}
