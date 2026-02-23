//! PROG-004: Tiered Population Needs (Anno 1800 inspired)
//!
//! Citizens progress through 5 tiers with escalating needs:
//! - Tier 1 (Basic): food, water, housing
//! - Tier 2 (Comfort): + electricity, heating
//! - Tier 3 (Community): + schools, healthcare, parks
//! - Tier 4 (Cultural): + entertainment
//! - Tier 5 (Aspirational): + university education, high land value, high happiness

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::citizen::{Citizen, CitizenDetails, HomeLocation, Needs};
use crate::grid::WorldGrid;
use crate::happiness::ServiceCoverageGrid;
use crate::heating::HeatingGrid;
use crate::land_value::LandValueGrid;
use crate::SlowTickTimer;

// ---------------------------------------------------------------------------
// Tier enum
// ---------------------------------------------------------------------------

/// The five population tiers, ordered by escalating needs.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    Encode,
    Decode,
    Default,
)]
pub enum PopulationTier {
    #[default]
    Basic,
    Comfort,
    Community,
    Cultural,
    Aspirational,
}

impl PopulationTier {
    pub fn name(self) -> &'static str {
        match self {
            Self::Basic => "Basic",
            Self::Comfort => "Comfort",
            Self::Community => "Community",
            Self::Cultural => "Cultural",
            Self::Aspirational => "Aspirational",
        }
    }

    pub fn index(self) -> usize {
        match self {
            Self::Basic => 0,
            Self::Comfort => 1,
            Self::Community => 2,
            Self::Cultural => 3,
            Self::Aspirational => 4,
        }
    }

    pub fn from_index(idx: usize) -> Self {
        match idx {
            0 => Self::Basic,
            1 => Self::Comfort,
            2 => Self::Community,
            3 => Self::Cultural,
            _ => Self::Aspirational,
        }
    }

    /// Economic contribution multiplier per citizen in this tier.
    pub fn economic_multiplier(self) -> f32 {
        match self {
            Self::Basic => 1.0,
            Self::Comfort => 1.5,
            Self::Community => 2.5,
            Self::Cultural => 4.0,
            Self::Aspirational => 7.0,
        }
    }

    pub fn next(self) -> Option<Self> {
        match self {
            Self::Basic => Some(Self::Comfort),
            Self::Comfort => Some(Self::Community),
            Self::Community => Some(Self::Cultural),
            Self::Cultural => Some(Self::Aspirational),
            Self::Aspirational => None,
        }
    }

    pub fn prev(self) -> Option<Self> {
        match self {
            Self::Basic => None,
            Self::Comfort => Some(Self::Basic),
            Self::Community => Some(Self::Comfort),
            Self::Cultural => Some(Self::Community),
            Self::Aspirational => Some(Self::Cultural),
        }
    }
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

/// Attached to each citizen entity to track their current population tier.
#[derive(Component, Debug, Clone, Serialize, Deserialize, Default)]
pub struct PopulationTierComp(pub PopulationTier);

// ---------------------------------------------------------------------------
// Tier fulfillment
// ---------------------------------------------------------------------------

const BASIC_HUNGER_THRESHOLD: f32 = 30.0;
const BASIC_HAPPINESS_THRESHOLD: f32 = 20.0;
const ASPIRATIONAL_LAND_VALUE: u8 = 150;
const ASPIRATIONAL_HAPPINESS: f32 = 70.0;
const ASPIRATIONAL_EDUCATION: u8 = 3;

/// Check whether a citizen qualifies for a given tier.
#[allow(clippy::too_many_arguments)]
pub fn qualifies_for_tier(
    tier: PopulationTier,
    details: &CitizenDetails,
    needs: &Needs,
    has_power: bool,
    has_water: bool,
    cov: u8,
    is_heated: bool,
    land_value: u8,
) -> bool {
    match tier {
        PopulationTier::Basic => {
            needs.hunger >= BASIC_HUNGER_THRESHOLD
                && has_water
                && details.happiness >= BASIC_HAPPINESS_THRESHOLD
        }
        PopulationTier::Comfort => {
            qualifies_basic(details, needs, has_water) && has_power && is_heated
        }
        PopulationTier::Community => {
            qualifies_basic(details, needs, has_water)
                && has_power
                && is_heated
                && cov & crate::happiness::COVERAGE_EDUCATION != 0
                && cov & crate::happiness::COVERAGE_HEALTH != 0
                && cov & crate::happiness::COVERAGE_PARK != 0
        }
        PopulationTier::Cultural => {
            qualifies_for_tier(
                PopulationTier::Community,
                details,
                needs,
                has_power,
                has_water,
                cov,
                is_heated,
                land_value,
            ) && cov & crate::happiness::COVERAGE_ENTERTAINMENT != 0
        }
        PopulationTier::Aspirational => {
            qualifies_for_tier(
                PopulationTier::Cultural,
                details,
                needs,
                has_power,
                has_water,
                cov,
                is_heated,
                land_value,
            ) && details.education >= ASPIRATIONAL_EDUCATION
                && land_value >= ASPIRATIONAL_LAND_VALUE
                && details.happiness >= ASPIRATIONAL_HAPPINESS
        }
    }
}

/// Inlined Basic check to avoid recursion overhead in Comfort/Community.
#[inline]
fn qualifies_basic(details: &CitizenDetails, needs: &Needs, has_water: bool) -> bool {
    needs.hunger >= BASIC_HUNGER_THRESHOLD
        && has_water
        && details.happiness >= BASIC_HAPPINESS_THRESHOLD
}

// ---------------------------------------------------------------------------
// Stats resource
// ---------------------------------------------------------------------------

/// Aggregate population tier statistics for the entire city.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize, Encode, Decode)]
pub struct PopulationTierStats {
    pub basic_count: u32,
    pub comfort_count: u32,
    pub community_count: u32,
    pub cultural_count: u32,
    pub aspirational_count: u32,
    pub total_economic_output: f32,
}

impl PopulationTierStats {
    pub fn total(&self) -> u32 {
        self.basic_count
            + self.comfort_count
            + self.community_count
            + self.cultural_count
            + self.aspirational_count
    }

    pub fn count_for_tier(&self, tier: PopulationTier) -> u32 {
        match tier {
            PopulationTier::Basic => self.basic_count,
            PopulationTier::Comfort => self.comfort_count,
            PopulationTier::Community => self.community_count,
            PopulationTier::Cultural => self.cultural_count,
            PopulationTier::Aspirational => self.aspirational_count,
        }
    }

    pub fn percentage(&self, tier: PopulationTier) -> f32 {
        let total = self.total() as f32;
        if total == 0.0 {
            return 0.0;
        }
        self.count_for_tier(tier) as f32 / total
    }
}

impl crate::Saveable for PopulationTierStats {
    const SAVE_KEY: &'static str = "population_tier_stats";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.total() == 0 {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Auto-insert `PopulationTierComp` on citizens that lack it.
pub fn init_citizen_tiers(
    mut commands: Commands,
    new_citizens: Query<Entity, (With<Citizen>, Without<PopulationTierComp>)>,
) {
    for entity in &new_citizens {
        commands
            .entity(entity)
            .insert(PopulationTierComp::default());
    }
}

/// Evaluate each citizen's tier eligibility and advance or demote them.
#[allow(clippy::too_many_arguments)]
pub fn evaluate_population_tiers(
    slow_tick: Res<SlowTickTimer>,
    grid: Res<WorldGrid>,
    coverage: Res<ServiceCoverageGrid>,
    heating_grid: Res<HeatingGrid>,
    land_value_grid: Res<LandValueGrid>,
    mut citizens: Query<
        (
            &CitizenDetails,
            &Needs,
            &HomeLocation,
            &mut PopulationTierComp,
        ),
        With<Citizen>,
    >,
) {
    if !slow_tick.should_run() {
        return;
    }

    citizens
        .par_iter_mut()
        .for_each(|(details, needs, home, mut tier_comp)| {
            let cell = grid.get(home.grid_x, home.grid_y);
            let idx = ServiceCoverageGrid::idx(home.grid_x, home.grid_y);
            let cov = coverage.flags[idx];
            let heated = heating_grid.is_heated(home.grid_x, home.grid_y);
            let lv = land_value_grid.get(home.grid_x, home.grid_y);
            let current = tier_comp.0;

            // Try to advance one step
            if let Some(next) = current.next() {
                if qualifies_for_tier(
                    next,
                    details,
                    needs,
                    cell.has_power,
                    cell.has_water,
                    cov,
                    heated,
                    lv,
                ) {
                    tier_comp.0 = next;
                    return;
                }
            }

            // Demote if current tier requirements no longer met
            if current != PopulationTier::Basic
                && !qualifies_for_tier(
                    current,
                    details,
                    needs,
                    cell.has_power,
                    cell.has_water,
                    cov,
                    heated,
                    lv,
                )
            {
                tier_comp.0 = current.prev().unwrap_or(PopulationTier::Basic);
            }
        });
}

/// Aggregate tier statistics across all citizens.
pub fn update_population_tier_stats(
    slow_tick: Res<SlowTickTimer>,
    mut stats: ResMut<PopulationTierStats>,
    citizens: Query<&PopulationTierComp, With<Citizen>>,
) {
    if !slow_tick.should_run() {
        return;
    }

    stats.basic_count = 0;
    stats.comfort_count = 0;
    stats.community_count = 0;
    stats.cultural_count = 0;
    stats.aspirational_count = 0;
    stats.total_economic_output = 0.0;

    for tier_comp in &citizens {
        match tier_comp.0 {
            PopulationTier::Basic => stats.basic_count += 1,
            PopulationTier::Comfort => stats.comfort_count += 1,
            PopulationTier::Community => stats.community_count += 1,
            PopulationTier::Cultural => stats.cultural_count += 1,
            PopulationTier::Aspirational => stats.aspirational_count += 1,
        }
        stats.total_economic_output += tier_comp.0.economic_multiplier();
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct PopulationTiersPlugin;

impl Plugin for PopulationTiersPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PopulationTierStats>();

        let mut registry = app
            .world_mut()
            .get_resource_or_insert_with(crate::SaveableRegistry::default);
        registry.register::<PopulationTierStats>();

        app.add_systems(
            FixedUpdate,
            (
                init_citizen_tiers,
                evaluate_population_tiers,
                update_population_tier_stats,
            )
                .chain()
                .after(crate::happiness::update_happiness)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_ordering_and_index() {
        assert!(PopulationTier::Basic < PopulationTier::Aspirational);
        for i in 0..5 {
            assert_eq!(PopulationTier::from_index(i).index(), i);
        }
    }

    #[test]
    fn tier_next_prev() {
        assert_eq!(PopulationTier::Basic.next(), Some(PopulationTier::Comfort));
        assert_eq!(PopulationTier::Aspirational.next(), None);
        assert_eq!(PopulationTier::Basic.prev(), None);
        assert_eq!(
            PopulationTier::Aspirational.prev(),
            Some(PopulationTier::Cultural)
        );
    }

    #[test]
    fn economic_multiplier_increases_with_tier() {
        let tiers = [
            PopulationTier::Basic,
            PopulationTier::Comfort,
            PopulationTier::Community,
            PopulationTier::Cultural,
            PopulationTier::Aspirational,
        ];
        for w in tiers.windows(2) {
            assert!(w[1].economic_multiplier() > w[0].economic_multiplier());
        }
    }

    #[test]
    fn stats_percentage_and_empty() {
        let s = PopulationTierStats {
            basic_count: 50,
            comfort_count: 30,
            community_count: 10,
            cultural_count: 5,
            aspirational_count: 5,
            total_economic_output: 0.0,
        };
        assert_eq!(s.total(), 100);
        assert!((s.percentage(PopulationTier::Basic) - 0.5).abs() < 0.01);
        assert_eq!(
            PopulationTierStats::default().percentage(PopulationTier::Basic),
            0.0
        );
    }
}
