//! Saveable implementations for city specialization state.
//!
//! Persists `CitySpecializations` and `SpecializationBonuses` across save/load
//! so that specialization type, level, progress, and derived bonuses are not
//! reset when the player loads a game.

use bevy::prelude::*;

use crate::specialization::{CitySpecializations, SpecializationBonuses};
use crate::Saveable;

// ---------------------------------------------------------------------------
// CitySpecializations
// ---------------------------------------------------------------------------

impl Saveable for CitySpecializations {
    const SAVE_KEY: &'static str = "city_specializations";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        // Skip saving if all scores are at default (0.0).
        let all_default = self
            .scores
            .values()
            .all(|s| s.score == 0.0 && s.level == 0);
        if all_default {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// ---------------------------------------------------------------------------
// SpecializationBonuses
// ---------------------------------------------------------------------------

impl Saveable for SpecializationBonuses {
    const SAVE_KEY: &'static str = "specialization_bonuses";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        // Skip if all bonuses are zero (default state).
        let all_zero = self.commercial_income_bonus == 0.0
            && self.park_happiness_bonus == 0.0
            && self.industrial_production_bonus == 0.0
            && self.industrial_land_value_penalty == 0.0
            && self.office_income_bonus == 0.0
            && self.tech_education_speed_bonus == 0.0
            && self.credit_rating_boost == 0.0
            && self.loan_interest_reduction == 0.0
            && self.education_advancement_bonus == 0.0
            && self.culture_happiness_bonus == 0.0
            && self.culture_land_value_bonus == 0.0;
        if all_zero {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct SpecializationSavePlugin;

impl Plugin for SpecializationSavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<CitySpecializations>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<SpecializationBonuses>();
    }
}
