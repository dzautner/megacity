// ---------------------------------------------------------------------------
// Save migration logic
// ---------------------------------------------------------------------------

use crate::save_error::SaveError;
use crate::save_types::{SaveData, CURRENT_SAVE_VERSION};

/// Migrate a `SaveData` from any older version up to `CURRENT_SAVE_VERSION`.
///
/// Each migration step handles one version bump. All new fields use `#[serde(default)]`
/// and `Option<T>`, so deserialization itself fills in safe defaults -- migration mostly
/// just bumps the version number so the save will be written at the current version on
/// the next save.
///
/// Returns the original version so callers can log the migration.
///
/// # Errors
///
/// Returns `SaveError::VersionMismatch` if the save file was created by a newer
/// version of the game (i.e. `save.version > CURRENT_SAVE_VERSION`).
pub fn migrate_save(save: &mut SaveData) -> Result<u32, SaveError> {
    let original_version = save.version;

    // Reject saves from a newer (future) version of the game.  The debug_assert
    // below only fires in debug builds; this explicit check protects release
    // builds from silently loading incompatible data.
    if save.version > CURRENT_SAVE_VERSION {
        return Err(SaveError::VersionMismatch {
            expected_max: CURRENT_SAVE_VERSION,
            found: save.version,
        });
    }

    // v0 -> v1: Legacy unversioned saves. All required fields (grid, roads,
    // clock, budget, demand, buildings, citizens, etc.) are already present
    // in the original format.  Option fields default to None.
    if save.version == 0 {
        save.version = 1;
    }

    // v1 -> v2: Added policies, weather, unlock_state, extended_budget, loan_book.
    // These are all `Option<T>` with `#[serde(default)]`, so they deserialize as None
    // from a v1 save -- no data fixup needed.
    if save.version == 1 {
        save.version = 2;
    }

    // v2 -> v3: Added lifecycle_timer and per-citizen path_cache / velocity / position.
    // All use `#[serde(default)]` so they already have safe zero/empty defaults.
    if save.version == 2 {
        save.version = 3;
    }

    // v3 -> v4: Added life_sim_timer (LifeSimTimer serialization).
    // Uses `#[serde(default)]` so it deserializes as None from a v3 save.
    if save.version == 3 {
        save.version = 4;
    }

    // v4 -> v5: Added stormwater_grid (StormwaterGrid serialization).
    // Uses `#[serde(default)]` so it deserializes as None from a v4 save.
    if save.version == 4 {
        save.version = 5;
    }

    // v5 -> v6: Added water_sources (WaterSource component serialization).
    // Uses `#[serde(default)]` so it deserializes as None from a v5 save.
    // Also added vacancy rate fields to SaveDemand for market-driven zone demand.
    // Uses `#[serde(default)]` so vacancy fields default to 0.0 from a v5 save.
    if save.version == 5 {
        save.version = 6;
    }

    // v6 -> v7: Added degree_days (HDD/CDD tracking for HVAC energy demand).
    // Uses `#[serde(default)]` so it deserializes as None from a v6 save.
    if save.version == 6 {
        save.version = 7;
    }

    // v7 -> v8: Added climate_zone to SaveWeather.
    // Uses `#[serde(default)]` so it deserializes as 0 (Temperate) from a v7 save.
    if save.version == 7 {
        save.version = 8;
    }

    // v8 -> v9: Added construction_modifiers (ConstructionModifiers serialization).
    // Uses `#[serde(default)]` so it deserializes as None from a v8 save.
    if save.version == 8 {
        save.version = 9;
    }

    // v9 -> v10: Added recycling_state (RecyclingState + RecyclingEconomics).
    // Uses `#[serde(default)]` so it deserializes as None from a v9 save.
    if save.version == 9 {
        save.version = 10;
    }

    // v10 -> v11: Added wind_damage_state (WindDamageState serialization).
    // Uses `#[serde(default)]` so it deserializes as None from a v10 save.
    if save.version == 10 {
        save.version = 11;
    }

    // v11 -> v12: Added uhi_grid (UhiGrid serialization for urban heat island).
    // Uses `#[serde(default)]` so it deserializes as None from a v11 save.
    if save.version == 11 {
        save.version = 12;
    }

    // v12 -> v13: Added drought_state (DroughtState serialization for drought index).
    // Uses `#[serde(default)]` so it deserializes as None from a v12 save.
    if save.version == 12 {
        save.version = 13;
    }

    // v13 -> v14: Added heat_wave_state (HeatWaveState serialization for heat wave effects).
    // Uses `#[serde(default)]` so it deserializes as None from a v13 save.
    if save.version == 13 {
        save.version = 14;
    }

    // v14 -> v15: Added composting_state (CompostingState serialization for composting facilities).
    // Uses `#[serde(default)]` so it deserializes as None from a v14 save.
    if save.version == 14 {
        save.version = 15;
    }

    // v15 -> v16: Added cold_snap_state (ColdSnapState serialization for cold snap effects).
    // Uses `#[serde(default)]` so it deserializes as None from a v15 save.
    if save.version == 15 {
        save.version = 16;
    }

    // v16 -> v17: Added water_treatment_state (WaterTreatmentState serialization for water treatment plants).
    // Uses `#[serde(default)]` so it deserializes as None from a v16 save.
    if save.version == 16 {
        save.water_treatment_state = None;
        save.version = 17;
    }

    // v17 -> v18: Added groundwater_depletion_state (GroundwaterDepletionState serialization).
    // Uses `#[serde(default)]` so it deserializes as None from a v17 save.
    if save.version == 17 {
        save.groundwater_depletion_state = None;
        save.version = 18;
    }
    // v18 -> v19: Added wastewater_state (WastewaterState serialization).
    // Uses `#[serde(default)]` so it deserializes as None from a v18 save.
    if save.version == 18 {
        save.wastewater_state = None;
        save.version = 19;
    }
    // v19 -> v20: Added hazardous_waste_state (HazardousWasteState serialization).
    // Uses `#[serde(default)]` so it deserializes as None from a v19 save.
    if save.version == 19 {
        save.hazardous_waste_state = None;
        save.version = 20;
    }
    // v20 -> v21: Added storm_drainage_state (StormDrainageState serialization for storm drainage infrastructure).
    // Uses `#[serde(default)]` so it deserializes as None from a v20 save.
    if save.version == 20 {
        save.storm_drainage_state = None;
        save.version = 21;
    }
    // v21 -> v22: Added landfill_capacity_state (LandfillCapacityState serialization for landfill warnings).
    // Uses `#[serde(default)]` so it deserializes as None from a v21 save.
    if save.version == 21 {
        save.landfill_capacity_state = None;
        save.version = 22;
    }
    // v22 -> v23: Added flood_state (FloodState serialization for urban flooding simulation).
    // Uses `#[serde(default)]` so it deserializes as None from a v22 save.
    if save.version == 22 {
        save.flood_state = None;
        save.version = 23;
    }
    // v23 -> v24: Added reservoir_state (ReservoirState serialization for reservoir water level tracking).
    // Uses `#[serde(default)]` so it deserializes as None from a v23 save.
    if save.version == 23 {
        save.reservoir_state = None;
        save.version = 24;
    }
    // v24 -> v25: Added landfill_gas_state (LandfillGasState serialization for landfill gas collection and energy).
    // Uses `#[serde(default)]` so it deserializes as None from a v24 save.
    if save.version == 24 {
        save.landfill_gas_state = None;
        save.version = 25;
    }
    // v25 -> v26: Added cso_state (SewerSystemState serialization for CSO events).
    // Uses `#[serde(default)]` so it deserializes as None from a v25 save.
    if save.version == 25 {
        save.cso_state = None;
        save.version = 26;
    }
    // v26 -> v27: Added water_conservation_state (WaterConservationState serialization for water conservation).
    // Uses `#[serde(default)]` so it deserializes as None from a v26 save.
    if save.version == 26 {
        save.water_conservation_state = None;
        save.version = 27;
    }
    // v27 -> v28: Added fog_state (FogState serialization for fog and visibility).
    // Uses `#[serde(default)]` so it deserializes as None from a v27 save.
    if save.version == 27 {
        save.fog_state = None;
        save.version = 28;
    }
    // v28 -> v29: Added urban_growth_boundary (UrbanGrowthBoundary serialization for UGB polygon).
    // Uses `#[serde(default)]` so it deserializes as None from a v28 save.
    if save.version == 28 {
        save.urban_growth_boundary = None;
        save.version = 29;
    }
    // v29 -> v30: Added snow_state (SnowGrid + SnowPlowingState serialization for snow accumulation and plowing).
    // Uses `#[serde(default)]` so it deserializes as None from a v29 save.
    if save.version == 29 {
        save.snow_state = None;
        save.version = 30;
    }
    // v30 -> v31: Added agriculture_state (AgricultureState serialization for growing season and crop yield).
    // Uses `#[serde(default)]` so it deserializes as None from a v30 save.
    if save.version == 30 {
        save.agriculture_state = None;
        save.version = 31;
    }
    // v31 -> v32: Added family graph (partner/children/parent indices on SaveCitizen).
    // Uses `#[serde(default)]` so old citizens deserialize with u32::MAX (no relationship).
    if save.version == 31 {
        // Family fields on SaveCitizen already default correctly via serde.
        save.version = 32;
    }
    debug_assert_eq!(save.version, CURRENT_SAVE_VERSION);

    Ok(original_version)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::save_types::*;
    use std::collections::BTreeMap;

    /// Helper to create a minimal `SaveData` for migration tests.
    fn minimal_save(version: u32) -> SaveData {
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

        let err = result.unwrap_err();
        let err_msg = format!("{err}");
        assert!(
            err_msg.contains("mismatch"),
            "Error message should mention version mismatch, got: {err_msg}"
        );
        assert!(
            matches!(err, SaveError::VersionMismatch { .. }),
            "Should be VersionMismatch variant"
        );
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
}
