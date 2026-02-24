// ---------------------------------------------------------------------------
// Save migration logic
// ---------------------------------------------------------------------------
//
// This module defines the concrete migration steps and exposes the
// `migrate_save()` function used by the load pipeline.  The migration chain
// is built via `build_migration_registry()` which validates that every
// version transition from v0 to CURRENT_SAVE_VERSION is covered.

use crate::save_error::SaveError;
pub use crate::save_migrate_registry::MigrationReport;
use crate::save_migrate_registry::{MigrationRegistry, MigrationStep};
use crate::save_types::{SaveData, CURRENT_SAVE_VERSION};

/// Build the full migration registry with all version transition steps.
///
/// Each step handles one version bump.  All new fields use `#[serde(default)]`
/// and `Option<T>`, so deserialization fills safe defaults -- migration mostly
/// bumps the version and occasionally fixes up data.
///
/// The registry constructor validates the chain is contiguous (no gaps).
pub(crate) fn build_migration_registry() -> MigrationRegistry {
    let steps = vec![
        // v0 -> v1: Legacy unversioned saves.
        MigrationStep {
            from_version: 0,
            description: "Legacy unversioned save -> v1 baseline",
            migrate_fn: |_save| {},
        },
        // v1 -> v2: Added policies, weather, unlock_state, extended_budget, loans.
        MigrationStep {
            from_version: 1,
            description: "Add policies, weather, unlock_state, extended_budget, loans",
            migrate_fn: |_save| {},
        },
        // v2 -> v3: Added lifecycle_timer, per-citizen path_cache / velocity / position.
        MigrationStep {
            from_version: 2,
            description: "Add lifecycle_timer, citizen path_cache/velocity/position",
            migrate_fn: |_save| {},
        },
        // v3 -> v4: Added life_sim_timer.
        MigrationStep {
            from_version: 3,
            description: "Add life_sim_timer (LifeSimTimer serialization)",
            migrate_fn: |_save| {},
        },
        // v4 -> v5: Added stormwater_grid.
        MigrationStep {
            from_version: 4,
            description: "Add stormwater_grid (StormwaterGrid serialization)",
            migrate_fn: |_save| {},
        },
        // v5 -> v6: Added water_sources and vacancy rate fields.
        MigrationStep {
            from_version: 5,
            description: "Add water_sources, vacancy rate fields in SaveDemand",
            migrate_fn: |_save| {},
        },
        // v6 -> v7: Added degree_days.
        MigrationStep {
            from_version: 6,
            description: "Add degree_days (HDD/CDD for HVAC energy demand)",
            migrate_fn: |_save| {},
        },
        // v7 -> v8: Added climate_zone to SaveWeather.
        MigrationStep {
            from_version: 7,
            description: "Add climate_zone to SaveWeather",
            migrate_fn: |_save| {},
        },
        // v8 -> v9: Added construction_modifiers.
        MigrationStep {
            from_version: 8,
            description: "Add construction_modifiers",
            migrate_fn: |_save| {},
        },
        // v9 -> v10: Added recycling_state.
        MigrationStep {
            from_version: 9,
            description: "Add recycling_state (RecyclingState + RecyclingEconomics)",
            migrate_fn: |_save| {},
        },
        // v10 -> v11: Added wind_damage_state.
        MigrationStep {
            from_version: 10,
            description: "Add wind_damage_state",
            migrate_fn: |_save| {},
        },
        // v11 -> v12: Added uhi_grid.
        MigrationStep {
            from_version: 11,
            description: "Add uhi_grid (urban heat island)",
            migrate_fn: |_save| {},
        },
        // v12 -> v13: Added drought_state.
        MigrationStep {
            from_version: 12,
            description: "Add drought_state",
            migrate_fn: |_save| {},
        },
        // v13 -> v14: Added heat_wave_state.
        MigrationStep {
            from_version: 13,
            description: "Add heat_wave_state",
            migrate_fn: |_save| {},
        },
        // v14 -> v15: Added composting_state.
        MigrationStep {
            from_version: 14,
            description: "Add composting_state",
            migrate_fn: |_save| {},
        },
        // v15 -> v16: Added cold_snap_state.
        MigrationStep {
            from_version: 15,
            description: "Add cold_snap_state",
            migrate_fn: |_save| {},
        },
        // v16 -> v17: Added water_treatment_state.
        MigrationStep {
            from_version: 16,
            description: "Add water_treatment_state",
            migrate_fn: |save| {
                save.water_treatment_state = None;
            },
        },
        // v17 -> v18: Added groundwater_depletion_state.
        MigrationStep {
            from_version: 17,
            description: "Add groundwater_depletion_state",
            migrate_fn: |save| {
                save.groundwater_depletion_state = None;
            },
        },
        // v18 -> v19: Added wastewater_state.
        MigrationStep {
            from_version: 18,
            description: "Add wastewater_state",
            migrate_fn: |save| {
                save.wastewater_state = None;
            },
        },
        // v19 -> v20: Added hazardous_waste_state.
        MigrationStep {
            from_version: 19,
            description: "Add hazardous_waste_state",
            migrate_fn: |save| {
                save.hazardous_waste_state = None;
            },
        },
        // v20 -> v21: Added storm_drainage_state.
        MigrationStep {
            from_version: 20,
            description: "Add storm_drainage_state",
            migrate_fn: |save| {
                save.storm_drainage_state = None;
            },
        },
        // v21 -> v22: Added landfill_capacity_state.
        MigrationStep {
            from_version: 21,
            description: "Add landfill_capacity_state",
            migrate_fn: |save| {
                save.landfill_capacity_state = None;
            },
        },
        // v22 -> v23: Added flood_state.
        MigrationStep {
            from_version: 22,
            description: "Add flood_state (urban flooding simulation)",
            migrate_fn: |save| {
                save.flood_state = None;
            },
        },
        // v23 -> v24: Added reservoir_state.
        MigrationStep {
            from_version: 23,
            description: "Add reservoir_state (reservoir water level tracking)",
            migrate_fn: |save| {
                save.reservoir_state = None;
            },
        },
        // v24 -> v25: Added landfill_gas_state.
        MigrationStep {
            from_version: 24,
            description: "Add landfill_gas_state (landfill gas collection/energy)",
            migrate_fn: |save| {
                save.landfill_gas_state = None;
            },
        },
        // v25 -> v26: Added cso_state.
        MigrationStep {
            from_version: 25,
            description: "Add cso_state (SewerSystemState for CSO events)",
            migrate_fn: |save| {
                save.cso_state = None;
            },
        },
        // v26 -> v27: Added water_conservation_state.
        MigrationStep {
            from_version: 26,
            description: "Add water_conservation_state",
            migrate_fn: |save| {
                save.water_conservation_state = None;
            },
        },
        // v27 -> v28: Added fog_state.
        MigrationStep {
            from_version: 27,
            description: "Add fog_state (fog and visibility)",
            migrate_fn: |save| {
                save.fog_state = None;
            },
        },
        // v28 -> v29: Added urban_growth_boundary.
        MigrationStep {
            from_version: 28,
            description: "Add urban_growth_boundary (UGB polygon)",
            migrate_fn: |save| {
                save.urban_growth_boundary = None;
            },
        },
        // v29 -> v30: Added snow_state.
        MigrationStep {
            from_version: 29,
            description: "Add snow_state (SnowGrid + SnowPlowingState)",
            migrate_fn: |save| {
                save.snow_state = None;
            },
        },
        // v30 -> v31: Added agriculture_state.
        MigrationStep {
            from_version: 30,
            description: "Add agriculture_state (growing season and crop yield)",
            migrate_fn: |save| {
                save.agriculture_state = None;
            },
        },
        // v31 -> v32: Added family graph.
        MigrationStep {
            from_version: 31,
            description: "Add family graph (partner/children/parent entity refs)",
            migrate_fn: |_save| {
                // Family fields on SaveCitizen default correctly via serde.
            },
        },
    ];

    MigrationRegistry::new(steps, CURRENT_SAVE_VERSION)
}

/// Migrate a `SaveData` from any older version up to `CURRENT_SAVE_VERSION`.
///
/// Returns the original version so callers can log the migration.
///
/// # Errors
///
/// Returns `SaveError::VersionMismatch` if the save file was created by a newer
/// version of the game (i.e. `save.version > CURRENT_SAVE_VERSION`).
pub fn migrate_save(save: &mut SaveData) -> Result<u32, SaveError> {
    let registry = build_migration_registry();
    let report = registry.migrate(save)?;
    Ok(report.original_version)
}

/// Migrate a `SaveData` and return a detailed migration report.
///
/// This is the richer API that provides step-by-step migration details.
///
/// # Errors
///
/// Returns `SaveError::VersionMismatch` if the save file was created by a
/// newer version of the game.
pub fn migrate_save_with_report(save: &mut SaveData) -> Result<MigrationReport, SaveError> {
    let registry = build_migration_registry();
    registry.migrate(save)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::save_types::*;
    use std::collections::BTreeMap;

    /// Helper to create a minimal `SaveData` for migration tests.
    pub(crate) fn minimal_save(version: u32) -> SaveData {
        SaveData {
            version,
            grid: SaveGrid {
                cells: vec![],
                width: 1,
                height: 1,
            },
            roads: SaveRoadNetwork {
                road_positions: vec![],
            },
            clock: SaveClock {
                day: 0,
                hour: 0.0,
                speed: 1.0,
            },
            budget: SaveBudget {
                treasury: 0.0,
                tax_rate: 0.0,
                last_collection_day: 0,
            },
            demand: SaveDemand {
                residential: 0.0,
                commercial: 0.0,
                industrial: 0.0,
                office: 0.0,
                vacancy_residential: 0.0,
                vacancy_commercial: 0.0,
                vacancy_industrial: 0.0,
                vacancy_office: 0.0,
            },
            buildings: vec![],
            citizens: vec![],
            utility_sources: vec![],
            service_buildings: vec![],
            road_segments: None,
            policies: None,
            weather: None,
            unlock_state: None,
            extended_budget: None,
            loan_book: None,
            lifecycle_timer: None,
            virtual_population: None,
            life_sim_timer: None,
            stormwater_grid: None,
            water_sources: None,
            degree_days: None,
            construction_modifiers: None,
            recycling_state: None,
            wind_damage_state: None,
            uhi_grid: None,
            drought_state: None,
            heat_wave_state: None,
            composting_state: None,
            cold_snap_state: None,
            water_treatment_state: None,
            groundwater_depletion_state: None,
            wastewater_state: None,
            hazardous_waste_state: None,
            storm_drainage_state: None,
            landfill_capacity_state: None,
            flood_state: None,
            reservoir_state: None,
            landfill_gas_state: None,
            cso_state: None,
            water_conservation_state: None,
            fog_state: None,
            urban_growth_boundary: None,
            snow_state: None,
            agriculture_state: None,
            extensions: BTreeMap::new(),
        }
    }

    #[test]
    fn test_migrate_save_rejects_future_version() {
        let mut save = minimal_save(CURRENT_SAVE_VERSION + 1);
        let result = migrate_save(&mut save);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SaveError::VersionMismatch { .. }
        ));
    }

    #[test]
    fn test_migrate_save_rejects_far_future_version() {
        let mut save = minimal_save(CURRENT_SAVE_VERSION + 100);
        let result = migrate_save(&mut save);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SaveError::VersionMismatch { .. }
        ));
    }

    #[test]
    fn test_migrate_save_accepts_current_version() {
        let mut save = minimal_save(CURRENT_SAVE_VERSION);
        let result = migrate_save(&mut save);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), CURRENT_SAVE_VERSION);
    }

    #[test]
    fn test_migrate_with_report_from_v0() {
        let mut save = minimal_save(0);
        let report = migrate_save_with_report(&mut save).unwrap();
        assert_eq!(report.original_version, 0);
        assert_eq!(report.final_version, CURRENT_SAVE_VERSION);
        assert_eq!(report.steps_applied, CURRENT_SAVE_VERSION);
        assert_eq!(
            report.step_descriptions.len(),
            CURRENT_SAVE_VERSION as usize
        );
        assert!(
            report.step_descriptions[0].contains("Legacy"),
            "First step should mention Legacy, got: {}",
            report.step_descriptions[0]
        );
    }

    #[test]
    fn test_migrate_with_report_noop() {
        let mut save = minimal_save(CURRENT_SAVE_VERSION);
        let report = migrate_save_with_report(&mut save).unwrap();
        assert_eq!(report.steps_applied, 0);
        assert!(report.step_descriptions.is_empty());
    }

    #[test]
    fn test_every_version_migrates_to_current() {
        for v in 0..=CURRENT_SAVE_VERSION {
            let mut save = minimal_save(v);
            let result = migrate_save(&mut save);
            assert!(
                result.is_ok(),
                "Migration from v{v} should succeed, got: {:?}",
                result.err()
            );
            assert_eq!(
                save.version, CURRENT_SAVE_VERSION,
                "After migration from v{v}, version should be {CURRENT_SAVE_VERSION}"
            );
        }
    }

    #[test]
    fn test_partial_migration_step_count() {
        for start_version in 0..=CURRENT_SAVE_VERSION {
            let mut save = minimal_save(start_version);
            let report = migrate_save_with_report(&mut save).unwrap();
            let expected_steps = CURRENT_SAVE_VERSION - start_version;
            assert_eq!(
                report.steps_applied, expected_steps,
                "From v{start_version}: expected {expected_steps} steps, got {}",
                report.steps_applied
            );
        }
    }
}
