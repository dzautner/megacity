//! POL-004: Housing Affordability Crisis Mechanic
//!
//! Tracks city-wide housing affordability as a ratio of estimated rent to
//! citizen income. When the ratio exceeds a threshold the city enters a
//! housing affordability crisis that worsens over time, driving increased
//! homelessness and low-income emigration.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::buildings::{Building, UnderConstruction};
use crate::citizen::{Citizen, CitizenDetails};
use crate::homelessness::HomelessnessStats;
use crate::immigration::CityAttractiveness;
use crate::land_value::LandValueGrid;
use crate::SlowTickTimer;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Healthy affordability ratio threshold (rent/income < 0.3).
const HEALTHY_THRESHOLD: f32 = 0.3;

/// Stressed affordability ratio threshold (rent/income 0.3-0.5).
const STRESSED_THRESHOLD: f32 = 0.5;

/// City-wide ratio above which a crisis is declared.
const CRISIS_TRIGGER: f32 = 0.4;

/// City-wide ratio below which a crisis is lifted.
/// Slightly below the trigger to avoid oscillation.
const CRISIS_RELIEF: f32 = 0.35;

/// Maximum crisis duration (in slow ticks) for severity scaling.
const MAX_CRISIS_DURATION: u32 = 50;

/// Base fraction of homelessness increase per slow tick during crisis.
const BASE_HOMELESSNESS_BOOST: f32 = 0.05;

/// Maximum attractiveness penalty applied during a severe crisis.
const MAX_ATTRACTIVENESS_PENALTY: f32 = 25.0;

/// Base rent factor: land_value * this factor = estimated monthly rent.
const RENT_PER_LAND_VALUE: f32 = 8.0;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Affordability tier classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AffordabilityTier {
    /// rent/income < 0.3
    #[default]
    Healthy,
    /// rent/income 0.3-0.5
    Stressed,
    /// rent/income > 0.5
    Crisis,
}

impl AffordabilityTier {
    pub fn from_ratio(ratio: f32) -> Self {
        if ratio < HEALTHY_THRESHOLD {
            Self::Healthy
        } else if ratio < STRESSED_THRESHOLD {
            Self::Stressed
        } else {
            Self::Crisis
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Healthy => "Healthy",
            Self::Stressed => "Stressed",
            Self::Crisis => "Crisis",
        }
    }
}

/// Tracks city-wide housing affordability metrics and crisis state.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct HousingAffordability {
    /// City-wide average affordability ratio (rent / income).
    pub affordability_ratio: f32,
    /// Current affordability tier based on city-wide ratio.
    pub tier: AffordabilityTier,
    /// Whether the city is currently in an affordability crisis.
    pub crisis_active: bool,
    /// How many slow ticks the crisis has been active.
    pub crisis_duration: u32,
    /// Current severity (0.0-1.0), scales with duration.
    pub severity: f32,
    /// Average estimated rent across occupied residential buildings.
    pub average_rent: f32,
    /// Average citizen income.
    pub average_income: f32,
    /// Number of citizens in each affordability tier.
    pub citizens_healthy: u32,
    pub citizens_stressed: u32,
    pub citizens_crisis: u32,
}

impl Default for HousingAffordability {
    fn default() -> Self {
        Self {
            affordability_ratio: 0.0,
            tier: AffordabilityTier::Healthy,
            crisis_active: false,
            crisis_duration: 0,
            severity: 0.0,
            average_rent: 0.0,
            average_income: 0.0,
            citizens_healthy: 0,
            citizens_stressed: 0,
            citizens_crisis: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// Saveable
// ---------------------------------------------------------------------------

/// Subset of `HousingAffordability` that is persisted.
#[derive(bitcode::Encode, bitcode::Decode, Default)]
struct HousingAffordabilitySaveData {
    affordability_ratio: f32,
    crisis_active: bool,
    crisis_duration: u32,
    severity: f32,
    average_rent: f32,
    average_income: f32,
}

impl crate::Saveable for HousingAffordability {
    const SAVE_KEY: &'static str = "housing_affordability";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if !self.crisis_active && self.crisis_duration == 0 {
            return None; // default state, skip saving
        }
        let data = HousingAffordabilitySaveData {
            affordability_ratio: self.affordability_ratio,
            crisis_active: self.crisis_active,
            crisis_duration: self.crisis_duration,
            severity: self.severity,
            average_rent: self.average_rent,
            average_income: self.average_income,
        };
        Some(bitcode::encode(&data))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        let data: HousingAffordabilitySaveData =
            crate::decode_or_warn(Self::SAVE_KEY, bytes);
        Self {
            affordability_ratio: data.affordability_ratio,
            tier: AffordabilityTier::from_ratio(data.affordability_ratio),
            crisis_active: data.crisis_active,
            crisis_duration: data.crisis_duration,
            severity: data.severity,
            average_rent: data.average_rent,
            average_income: data.average_income,
            citizens_healthy: 0,
            citizens_stressed: 0,
            citizens_crisis: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// System: update_housing_affordability
// ---------------------------------------------------------------------------

/// Computes the city-wide affordability ratio from land values and citizen
/// incomes, determines crisis state, and applies crisis effects.
#[allow(clippy::too_many_arguments)]
pub fn update_housing_affordability(
    slow_tick: Res<SlowTickTimer>,
    land_value: Res<LandValueGrid>,
    citizens: Query<&CitizenDetails, With<Citizen>>,
    buildings: Query<&Building, Without<UnderConstruction>>,
    mut affordability: ResMut<HousingAffordability>,
    mut attractiveness: ResMut<CityAttractiveness>,
    mut homelessness_stats: ResMut<HomelessnessStats>,
) {
    if !slow_tick.should_run() {
        return;
    }

    // --- Compute average rent from occupied residential buildings ---
    let mut total_rent = 0.0_f32;
    let mut rent_count = 0u32;

    for b in &buildings {
        if (b.zone_type.is_residential() || b.zone_type.is_mixed_use()) && b.occupants > 0 {
            let lv = land_value.get(b.grid_x, b.grid_y) as f32;
            let estimated_rent = lv * RENT_PER_LAND_VALUE;
            total_rent += estimated_rent * b.occupants as f32;
            rent_count += b.occupants;
        }
    }

    let avg_rent = if rent_count > 0 {
        total_rent / rent_count as f32
    } else {
        0.0
    };

    // --- Compute average income from citizen salaries ---
    let mut total_income = 0.0_f32;
    let mut income_count = 0u32;
    let mut healthy = 0u32;
    let mut stressed = 0u32;
    let mut crisis = 0u32;

    for details in &citizens {
        let salary = details.salary;
        if salary > 0.0 {
            total_income += salary;
            income_count += 1;

            // Per-citizen affordability tier
            let personal_ratio = avg_rent / salary;
            match AffordabilityTier::from_ratio(personal_ratio) {
                AffordabilityTier::Healthy => healthy += 1,
                AffordabilityTier::Stressed => stressed += 1,
                AffordabilityTier::Crisis => crisis += 1,
            }
        }
    }

    let avg_income = if income_count > 0 {
        total_income / income_count as f32
    } else {
        0.0
    };

    // --- City-wide affordability ratio ---
    let ratio = if avg_income > 0.0 {
        (avg_rent / avg_income).clamp(0.0, 2.0)
    } else if avg_rent > 0.0 {
        // Income is 0 but rent exists: worst case
        2.0
    } else {
        0.0
    };

    affordability.affordability_ratio = ratio;
    affordability.tier = AffordabilityTier::from_ratio(ratio);
    affordability.average_rent = avg_rent;
    affordability.average_income = avg_income;
    affordability.citizens_healthy = healthy;
    affordability.citizens_stressed = stressed;
    affordability.citizens_crisis = crisis;

    // --- Crisis state machine ---
    if affordability.crisis_active {
        if ratio < CRISIS_RELIEF {
            // Crisis resolved
            affordability.crisis_active = false;
            affordability.crisis_duration = 0;
            affordability.severity = 0.0;
        } else {
            // Crisis continues, duration and severity increase
            affordability.crisis_duration += 1;
            affordability.severity = (affordability.crisis_duration as f32
                / MAX_CRISIS_DURATION as f32)
                .clamp(0.0, 1.0);
        }
    } else if ratio > CRISIS_TRIGGER {
        // Crisis begins
        affordability.crisis_active = true;
        affordability.crisis_duration = 1;
        affordability.severity = (1.0 / MAX_CRISIS_DURATION as f32).clamp(0.0, 1.0);
    }

    // --- Apply crisis effects ---
    if affordability.crisis_active {
        let severity = affordability.severity;

        // Effect 1: Boost homelessness (more low-income citizens lose homes)
        let homelessness_boost =
            (BASE_HOMELESSNESS_BOOST * severity * income_count as f32) as u32;
        homelessness_stats.total_homeless = homelessness_stats
            .total_homeless
            .saturating_add(homelessness_boost);

        // Effect 2: Reduce city attractiveness (drives emigration)
        let attractiveness_penalty = MAX_ATTRACTIVENESS_PENALTY * severity;
        attractiveness.overall_score =
            (attractiveness.overall_score - attractiveness_penalty).max(0.0);

        // Effect 3: Specifically reduce housing attractiveness factor
        attractiveness.housing_factor =
            (attractiveness.housing_factor - severity * 0.5).max(0.0);
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct HousingAffordabilityPlugin;

impl Plugin for HousingAffordabilityPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HousingAffordability>().add_systems(
            FixedUpdate,
            update_housing_affordability
                .after(crate::land_value::update_land_value)
                .after(crate::wealth::update_wealth_stats)
                .in_set(crate::SimulationSet::Simulation),
        );

        // Register for save/load
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<HousingAffordability>();
    }
}
