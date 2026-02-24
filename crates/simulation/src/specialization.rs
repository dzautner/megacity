use std::collections::HashMap;

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::citizen::{Citizen, CitizenDetails};
use crate::economy::CityBudget;
use crate::production::CityGoods;
use crate::services::{ServiceBuilding, ServiceType};
use crate::stats::CityStats;
use crate::tourism::Tourism;
use crate::TickCounter;

// =============================================================================
// Types
// =============================================================================

/// The six possible city specializations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode, Serialize, Deserialize)]
pub enum CitySpecialization {
    Tourism,
    Industry,
    Technology,
    Finance,
    Education,
    Culture,
}

impl CitySpecialization {
    pub const ALL: &'static [CitySpecialization] = &[
        CitySpecialization::Tourism,
        CitySpecialization::Industry,
        CitySpecialization::Technology,
        CitySpecialization::Finance,
        CitySpecialization::Education,
        CitySpecialization::Culture,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Tourism => "Tourism",
            Self::Industry => "Industry",
            Self::Technology => "Technology",
            Self::Finance => "Finance",
            Self::Education => "Education",
            Self::Culture => "Culture",
        }
    }
}

/// Score and derived level for a single specialization.
#[derive(Debug, Clone, Copy, Encode, Decode, Serialize, Deserialize)]
pub struct SpecializationScore {
    /// Raw score in the range 0.0 to 100.0.
    pub score: f32,
    /// Derived level: 0=None, 1=Emerging, 2=Established, 3=Dominant.
    pub level: u8,
}

impl Default for SpecializationScore {
    fn default() -> Self {
        Self {
            score: 0.0,
            level: 0,
        }
    }
}

impl SpecializationScore {
    /// Compute level from score using fixed thresholds.
    pub fn level_from_score(score: f32) -> u8 {
        if score >= 75.0 {
            3
        } else if score >= 50.0 {
            2
        } else if score >= 25.0 {
            1
        } else {
            0
        }
    }

    pub fn level_name(level: u8) -> &'static str {
        match level {
            0 => "None",
            1 => "Emerging",
            2 => "Established",
            3 => "Dominant",
            _ => "Unknown",
        }
    }

    /// Bonus multiplier for a given level: 1x at level 1, 1.5x at level 2, 2x at level 3.
    /// Returns 0.0 for level 0 (no bonus).
    pub fn bonus_multiplier(level: u8) -> f32 {
        match level {
            0 => 0.0,
            1 => 1.0,
            2 => 1.5,
            3 => 2.0,
            _ => 0.0,
        }
    }
}

// =============================================================================
// Resources
// =============================================================================

/// Tracks the score and level of each city specialization.
#[derive(Resource, Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct CitySpecializations {
    pub scores: HashMap<CitySpecialization, SpecializationScore>,
}

impl Default for CitySpecializations {
    fn default() -> Self {
        let mut scores = HashMap::new();
        for &spec in CitySpecialization::ALL {
            scores.insert(spec, SpecializationScore::default());
        }
        Self { scores }
    }
}

impl CitySpecializations {
    pub fn get(&self, spec: CitySpecialization) -> SpecializationScore {
        self.scores.get(&spec).copied().unwrap_or_default()
    }
}

/// Active bonus multipliers derived from specialization levels.
/// Each field represents the effective bonus multiplier (0.0 = inactive).
/// Systems that consume these bonuses multiply their base values accordingly.
#[derive(Resource, Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
pub struct SpecializationBonuses {
    // Tourism bonuses
    /// Multiplier for extra commercial income (base +5%)
    pub commercial_income_bonus: f32,
    /// Extra happiness from parks (base +2)
    pub park_happiness_bonus: f32,

    // Industry bonuses
    /// Multiplier for extra industrial production (base +10%)
    pub industrial_production_bonus: f32,
    /// Land value penalty near industrial zones (base -5)
    pub industrial_land_value_penalty: f32,

    // Technology bonuses
    /// Multiplier for extra office building income (base +10%)
    pub office_income_bonus: f32,
    /// Multiplier for extra education speed (base +5%)
    pub tech_education_speed_bonus: f32,

    // Finance bonuses
    /// Credit rating boost (base +0.1)
    pub credit_rating_boost: f32,
    /// Loan interest reduction (base -2%)
    pub loan_interest_reduction: f32,

    // Education bonuses
    /// Education advancement speed bonus (base +5%)
    pub education_advancement_bonus: f32,

    // Culture bonuses
    /// Happiness bonus (base +3)
    pub culture_happiness_bonus: f32,
    /// Land value boost near cultural buildings (base +5)
    pub culture_land_value_bonus: f32,
}

// =============================================================================
// System
// =============================================================================

/// Interval in ticks between specialization recalculations.
const SPECIALIZATION_INTERVAL: u64 = 100;

/// Computes city specialization scores based on buildings, services, citizens,
/// and economic data. Runs every 100 ticks.
#[allow(clippy::too_many_arguments)]
pub fn compute_specializations(
    tick: Res<TickCounter>,
    mut specializations: ResMut<CitySpecializations>,
    mut bonuses: ResMut<SpecializationBonuses>,
    services: Query<&ServiceBuilding>,
    citizens: Query<&CitizenDetails, With<Citizen>>,
    stats: Res<CityStats>,
    tourism: Res<Tourism>,
    budget: Res<CityBudget>,
    city_goods: Res<CityGoods>,
) {
    if !tick.0.is_multiple_of(SPECIALIZATION_INTERVAL) {
        return;
    }

    // -------------------------------------------------------------------------
    // Count service types
    // -------------------------------------------------------------------------
    let mut parks = 0u32;
    let mut plazas = 0u32;
    let mut museums = 0u32;
    let mut stadiums = 0u32;
    let mut libraries = 0u32;
    let mut schools = 0u32;
    let mut universities = 0u32;
    let mut entertainment = 0u32; // sports fields, stadiums, etc.

    for service in &services {
        match service.service_type {
            ServiceType::SmallPark | ServiceType::LargePark | ServiceType::Playground => {
                parks += 1;
            }
            ServiceType::Plaza => {
                plazas += 1;
            }
            ServiceType::Museum => {
                museums += 1;
            }
            ServiceType::Stadium => {
                stadiums += 1;
                entertainment += 1;
            }
            ServiceType::SportsField => {
                entertainment += 1;
            }
            ServiceType::Library => {
                libraries += 1;
            }
            ServiceType::ElementarySchool | ServiceType::HighSchool | ServiceType::Kindergarten => {
                schools += 1;
            }
            ServiceType::University => {
                universities += 1;
            }
            ServiceType::Cathedral | ServiceType::TVStation => {
                entertainment += 1;
            }
            _ => {}
        }
    }

    // -------------------------------------------------------------------------
    // Count building types
    // -------------------------------------------------------------------------
    let industrial_count = stats.industrial_buildings;
    let office_count = stats.office_buildings;

    // -------------------------------------------------------------------------
    // Citizen education stats
    // -------------------------------------------------------------------------
    let total_citizens = citizens.iter().count() as f32;
    let high_edu_count = citizens.iter().filter(|c| c.education >= 3).count() as f32;
    let avg_education = if total_citizens > 0.0 {
        citizens.iter().map(|c| c.education as f32).sum::<f32>() / total_citizens
    } else {
        0.0
    };
    let high_edu_pct = if total_citizens > 0.0 {
        high_edu_count / total_citizens
    } else {
        0.0
    };

    // -------------------------------------------------------------------------
    // Production chain output (total production rate across all goods)
    // -------------------------------------------------------------------------
    let total_production: f32 = city_goods.production_rate.values().sum();

    // -------------------------------------------------------------------------
    // Compute each specialization score (0-100)
    // -------------------------------------------------------------------------

    // TOURISM: parks + plazas + museums + stadiums + entertainment + tourism resource score
    let tourism_score = {
        let venue_score = (parks + plazas + museums + stadiums + entertainment) as f32 * 3.0;
        let tourism_attr = tourism.attractiveness; // 0-100
        ((venue_score + tourism_attr) / 2.0).clamp(0.0, 100.0)
    };

    // INDUSTRY: industrial buildings + production chain output
    let industry_score = {
        let building_score = (industrial_count as f32 * 0.15).min(50.0);
        let production_score = (total_production * 2.0).min(50.0);
        (building_score + production_score).clamp(0.0, 100.0)
    };

    // TECHNOLOGY: office buildings + universities + high-education workforce %
    let technology_score = {
        let office_score = (office_count as f32 * 0.2).min(40.0);
        let uni_score = (universities as f32 * 10.0).min(20.0);
        let edu_workforce_score = (high_edu_pct * 100.0).min(40.0);
        (office_score + uni_score + edu_workforce_score).clamp(0.0, 100.0)
    };

    // FINANCE: office buildings + treasury size + trade balance
    let finance_score = {
        let office_score = (office_count as f32 * 0.15).min(30.0);
        let treasury_score = ((budget.treasury as f32 / 10_000.0) * 5.0).min(35.0);
        let trade_score = ((city_goods.trade_balance as f32).max(0.0) * 0.5).min(35.0);
        (office_score + treasury_score + trade_score).clamp(0.0, 100.0)
    };

    // EDUCATION: schools + universities + average education level
    let education_score = {
        let school_score = (schools as f32 * 5.0 + universities as f32 * 15.0).min(50.0);
        let avg_edu_score = (avg_education / 3.0 * 50.0).min(50.0);
        (school_score + avg_edu_score).clamp(0.0, 100.0)
    };

    // CULTURE: museums + libraries + plazas + parks + entertainment venues
    let culture_score = {
        let cultural_venues = museums + libraries + plazas + parks + entertainment;
        let venue_score = (cultural_venues as f32 * 5.0).min(70.0);
        let happiness_factor = (stats.average_happiness / 100.0 * 30.0).min(30.0);
        (venue_score + happiness_factor).clamp(0.0, 100.0)
    };

    // -------------------------------------------------------------------------
    // Update scores and levels
    // -------------------------------------------------------------------------
    let scores_data = [
        (CitySpecialization::Tourism, tourism_score),
        (CitySpecialization::Industry, industry_score),
        (CitySpecialization::Technology, technology_score),
        (CitySpecialization::Finance, finance_score),
        (CitySpecialization::Education, education_score),
        (CitySpecialization::Culture, culture_score),
    ];

    for (spec, raw_score) in scores_data {
        let entry = specializations.scores.entry(spec).or_default();
        entry.score = raw_score;
        entry.level = SpecializationScore::level_from_score(raw_score);
    }

    // -------------------------------------------------------------------------
    // Compute bonuses based on levels
    // -------------------------------------------------------------------------
    let tourism_mult = SpecializationScore::bonus_multiplier(
        specializations.get(CitySpecialization::Tourism).level,
    );
    let industry_mult = SpecializationScore::bonus_multiplier(
        specializations.get(CitySpecialization::Industry).level,
    );
    let tech_mult = SpecializationScore::bonus_multiplier(
        specializations.get(CitySpecialization::Technology).level,
    );
    let finance_mult = SpecializationScore::bonus_multiplier(
        specializations.get(CitySpecialization::Finance).level,
    );
    let education_mult = SpecializationScore::bonus_multiplier(
        specializations.get(CitySpecialization::Education).level,
    );
    let culture_mult = SpecializationScore::bonus_multiplier(
        specializations.get(CitySpecialization::Culture).level,
    );

    // Tourism: +5% commercial income, +2 happiness from parks
    bonuses.commercial_income_bonus = 0.05 * tourism_mult;
    bonuses.park_happiness_bonus = 2.0 * tourism_mult;

    // Industry: +10% industrial production, -5 land value near industrial
    bonuses.industrial_production_bonus = 0.10 * industry_mult;
    bonuses.industrial_land_value_penalty = 5.0 * industry_mult;

    // Technology: +10% office income, +5% education speed
    bonuses.office_income_bonus = 0.10 * tech_mult;
    bonuses.tech_education_speed_bonus = 0.05 * tech_mult;

    // Finance: +0.1 credit rating, -2% loan interest
    bonuses.credit_rating_boost = 0.1 * finance_mult;
    bonuses.loan_interest_reduction = 0.02 * finance_mult;

    // Education: +5% education advancement speed
    bonuses.education_advancement_bonus = 0.05 * education_mult;

    // Culture: +3 happiness, +5 land value near cultural
    bonuses.culture_happiness_bonus = 3.0 * culture_mult;
    bonuses.culture_land_value_bonus = 5.0 * culture_mult;
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_specializations_are_zero() {
        let specs = CitySpecializations::default();
        for &spec in CitySpecialization::ALL {
            let s = specs.get(spec);
            assert_eq!(s.score, 0.0, "{:?} should default to score 0.0", spec);
            assert_eq!(s.level, 0, "{:?} should default to level 0", spec);
        }
    }

    #[test]
    fn test_level_thresholds() {
        assert_eq!(SpecializationScore::level_from_score(0.0), 0);
        assert_eq!(SpecializationScore::level_from_score(10.0), 0);
        assert_eq!(SpecializationScore::level_from_score(24.9), 0);
        assert_eq!(SpecializationScore::level_from_score(25.0), 1);
        assert_eq!(SpecializationScore::level_from_score(40.0), 1);
        assert_eq!(SpecializationScore::level_from_score(49.9), 1);
        assert_eq!(SpecializationScore::level_from_score(50.0), 2);
        assert_eq!(SpecializationScore::level_from_score(60.0), 2);
        assert_eq!(SpecializationScore::level_from_score(74.9), 2);
        assert_eq!(SpecializationScore::level_from_score(75.0), 3);
        assert_eq!(SpecializationScore::level_from_score(100.0), 3);
    }

    #[test]
    fn test_bonus_multipliers() {
        assert_eq!(SpecializationScore::bonus_multiplier(0), 0.0);
        assert_eq!(SpecializationScore::bonus_multiplier(1), 1.0);
        assert_eq!(SpecializationScore::bonus_multiplier(2), 1.5);
        assert_eq!(SpecializationScore::bonus_multiplier(3), 2.0);
    }

    #[test]
    fn test_level_names() {
        assert_eq!(SpecializationScore::level_name(0), "None");
        assert_eq!(SpecializationScore::level_name(1), "Emerging");
        assert_eq!(SpecializationScore::level_name(2), "Established");
        assert_eq!(SpecializationScore::level_name(3), "Dominant");
    }

    #[test]
    fn test_bonuses_default_zero() {
        let bonuses = SpecializationBonuses::default();
        assert_eq!(bonuses.commercial_income_bonus, 0.0);
        assert_eq!(bonuses.park_happiness_bonus, 0.0);
        assert_eq!(bonuses.industrial_production_bonus, 0.0);
        assert_eq!(bonuses.office_income_bonus, 0.0);
        assert_eq!(bonuses.credit_rating_boost, 0.0);
        assert_eq!(bonuses.education_advancement_bonus, 0.0);
        assert_eq!(bonuses.culture_happiness_bonus, 0.0);
        assert_eq!(bonuses.culture_land_value_bonus, 0.0);
    }

    #[test]
    fn test_bonus_multipliers_apply_correctly() {
        // Tourism at level 2 (Established) => 1.5x multiplier
        // Base commercial income bonus is 5%, so at level 2 = 0.05 * 1.5 = 0.075
        let mult = SpecializationScore::bonus_multiplier(2);
        let commercial_bonus = 0.05 * mult;
        assert!((commercial_bonus - 0.075).abs() < f32::EPSILON);

        // Industry at level 3 (Dominant) => 2.0x multiplier
        // Base production bonus is 10%, so at level 3 = 0.10 * 2.0 = 0.20
        let mult = SpecializationScore::bonus_multiplier(3);
        let production_bonus = 0.10 * mult;
        assert!((production_bonus - 0.20).abs() < f32::EPSILON);

        // At level 0 (None) => 0.0 multiplier => no bonus
        let mult = SpecializationScore::bonus_multiplier(0);
        let no_bonus = 0.05 * mult;
        assert_eq!(no_bonus, 0.0);
    }
}

pub struct SpecializationPlugin;

impl Plugin for SpecializationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CitySpecializations>()
            .init_resource::<SpecializationBonuses>()
            .add_systems(
                FixedUpdate,
                compute_specializations
                    .after(crate::stats::update_stats)
                    .in_set(crate::SimulationSet::PostSim),
            );
    }
}
