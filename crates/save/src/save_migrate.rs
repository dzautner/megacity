// ---------------------------------------------------------------------------
// Save migration logic
// ---------------------------------------------------------------------------

use crate::save_types::{SaveData, CURRENT_SAVE_VERSION};

/// Migrate a `SaveData` from any older version up to `CURRENT_SAVE_VERSION`.
///
/// Each migration step handles one version bump. All new fields use `#[serde(default)]`
/// and `Option<T>`, so deserialization itself fills in safe defaults -- migration mostly
/// just bumps the version number so the save will be written at the current version on
/// the next save.
///
/// Returns the original version so callers can log the migration.
pub fn migrate_save(save: &mut SaveData) -> u32 {
    let original_version = save.version;

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

    // Ensure version is at the current value (safety net for future additions).

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
    debug_assert_eq!(save.version, CURRENT_SAVE_VERSION);

    original_version
}
