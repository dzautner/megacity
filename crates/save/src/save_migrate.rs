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

    // Ensure version is at the current value (safety net for future additions).
    debug_assert_eq!(save.version, CURRENT_SAVE_VERSION);

    original_version
}
