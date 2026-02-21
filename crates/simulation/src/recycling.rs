//! Recycling program tiers and economics (WASTE-004).
//!
//! Implements tiered recycling programs from "No program" (5% baseline diversion)
//! to "Zero waste goal" (60% diversion). Each tier specifies diversion rates,
//! participation rates, per-household costs, and contamination rates.
//!
//! Recycling economics tracks commodity prices per material type with market
//! cycles (~5 game-year period, 0.3x bust to 1.5x boom) and computes net
//! value per ton after subtracting collection and processing costs.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::time_of_day::GameClock;
use crate::SlowTickTimer;

// =============================================================================
// Recycling tiers
// =============================================================================

/// Recycling program tiers available to the player.
///
/// Each tier represents a progressively more ambitious (and costly) recycling
/// program. The player selects a tier as a city-wide policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum RecyclingTier {
    /// No formal program; only scavenging / informal recycling (~5% diversion).
    #[default]
    None,
    /// Voluntary drop-off centres; moderate participation (~15% diversion).
    VoluntaryDropoff,
    /// Curbside collection with basic sorting (~30% diversion).
    CurbsideBasic,
    /// Curbside collection with multi-stream sorting (~45% diversion).
    CurbsideSort,
    /// Single-stream (commingled) curbside collection (~40% diversion).
    SingleStream,
    /// Variable-rate pricing; pay by weight/volume (~50% diversion).
    PayAsYouThrow,
    /// Ambitious zero-waste goal with composting + reuse (~60% diversion).
    ZeroWaste,
}

impl RecyclingTier {
    /// Fraction of waste stream diverted from landfill (0.0..=1.0).
    pub fn diversion_rate(self) -> f32 {
        match self {
            Self::None => 0.05,
            Self::VoluntaryDropoff => 0.15,
            Self::CurbsideBasic => 0.30,
            Self::CurbsideSort => 0.45,
            Self::SingleStream => 0.40,
            Self::PayAsYouThrow => 0.50,
            Self::ZeroWaste => 0.60,
        }
    }

    /// Fraction of households participating in the program (0.0..=1.0).
    pub fn participation_rate(self) -> f32 {
        match self {
            Self::None => 0.05,
            Self::VoluntaryDropoff => 0.25,
            Self::CurbsideBasic => 0.60,
            Self::CurbsideSort => 0.70,
            Self::SingleStream => 0.80,
            Self::PayAsYouThrow => 0.85,
            Self::ZeroWaste => 0.95,
        }
    }

    /// Annual cost per household in dollars.
    pub fn cost_per_household_year(self) -> f64 {
        match self {
            Self::None => 0.0,
            Self::VoluntaryDropoff => 15.0,
            Self::CurbsideBasic => 60.0,
            Self::CurbsideSort => 90.0,
            Self::SingleStream => 75.0,
            Self::PayAsYouThrow => 50.0, // lower because variable rate shifts cost to users
            Self::ZeroWaste => 120.0,
        }
    }

    /// Contamination rate: fraction of the recycling stream that is actually
    /// non-recyclable waste and must go to landfill (0.0..=1.0).
    ///
    /// Higher convenience programs (single-stream) have worse contamination.
    pub fn contamination_rate(self) -> f32 {
        match self {
            Self::None => 0.30,
            Self::VoluntaryDropoff => 0.20,
            Self::CurbsideBasic => 0.20,
            Self::CurbsideSort => 0.15,
            Self::SingleStream => 0.25,
            Self::PayAsYouThrow => 0.18,
            Self::ZeroWaste => 0.15,
        }
    }

    /// Revenue potential multiplier relative to baseline commodity prices.
    /// Higher tiers with better sorting yield cleaner material and higher prices.
    pub fn revenue_potential(self) -> f32 {
        match self {
            Self::None => 0.3,
            Self::VoluntaryDropoff => 0.5,
            Self::CurbsideBasic => 0.7,
            Self::CurbsideSort => 0.9,
            Self::SingleStream => 0.65,
            Self::PayAsYouThrow => 0.8,
            Self::ZeroWaste => 1.0,
        }
    }

    /// Human-readable name for the tier.
    pub fn name(self) -> &'static str {
        match self {
            Self::None => "No Program",
            Self::VoluntaryDropoff => "Voluntary Drop-off",
            Self::CurbsideBasic => "Curbside Basic",
            Self::CurbsideSort => "Curbside Multi-Sort",
            Self::SingleStream => "Single Stream",
            Self::PayAsYouThrow => "Pay-As-You-Throw",
            Self::ZeroWaste => "Zero Waste Goal",
        }
    }

    /// All tiers in order of increasing ambition.
    pub fn all() -> &'static [RecyclingTier] {
        &[
            Self::None,
            Self::VoluntaryDropoff,
            Self::CurbsideBasic,
            Self::CurbsideSort,
            Self::SingleStream,
            Self::PayAsYouThrow,
            Self::ZeroWaste,
        ]
    }
}

// =============================================================================
// Recycling economics
// =============================================================================

/// Base commodity prices per ton for each recyclable material category.
/// These are the "neutral" prices before market cycle adjustment.
const BASE_PRICE_PAPER: f64 = 80.0; // $/ton
const BASE_PRICE_PLASTIC: f64 = 200.0;
const BASE_PRICE_GLASS: f64 = 25.0;
const BASE_PRICE_METAL: f64 = 300.0;
const BASE_PRICE_ORGANIC: f64 = 15.0; // compost value

/// Processing cost per ton (sorting, baling, transport to buyers).
const PROCESSING_COST_PER_TON: f64 = 50.0;

/// Collection cost per ton (trucks, fuel, labor).
const COLLECTION_COST_PER_TON: f64 = 40.0;

/// Commodity prices and market cycle for recyclable materials.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct RecyclingEconomics {
    /// Current price per ton for paper/cardboard.
    pub price_paper: f64,
    /// Current price per ton for plastics.
    pub price_plastic: f64,
    /// Current price per ton for glass.
    pub price_glass: f64,
    /// Current price per ton for metals.
    pub price_metal: f64,
    /// Current price per ton for organics (compost).
    pub price_organic: f64,

    /// Position in the market cycle (0.0..1.0).
    /// 0.0 = start of cycle, 0.5 = peak, wraps around.
    pub market_cycle_position: f64,

    /// Day when we last updated the market cycle.
    pub last_update_day: u32,
}

impl Default for RecyclingEconomics {
    fn default() -> Self {
        Self {
            price_paper: BASE_PRICE_PAPER,
            price_plastic: BASE_PRICE_PLASTIC,
            price_glass: BASE_PRICE_GLASS,
            price_metal: BASE_PRICE_METAL,
            price_organic: BASE_PRICE_ORGANIC,
            market_cycle_position: 0.0,
            last_update_day: 0,
        }
    }
}

impl RecyclingEconomics {
    /// Market cycle period in game days (~5 game years = 5 * 365 = 1825 days).
    const CYCLE_PERIOD_DAYS: f64 = 1825.0;

    /// Market price multiplier based on cycle position.
    ///
    /// Uses a sine wave: ranges from 0.3 (bust) to 1.5 (boom).
    /// The midpoint (1.0x) occurs at positions 0.0 and 0.5 on the way up/down.
    pub fn price_multiplier(&self) -> f64 {
        // sine wave: amplitude 0.6 around mean 0.9 => range [0.3, 1.5]
        let angle = self.market_cycle_position * std::f64::consts::TAU;
        0.9 + 0.6 * angle.sin()
    }

    /// Weighted-average revenue per ton of diverted material at current prices.
    ///
    /// Uses default MSW composition weights: paper 25%, plastics 13%, metals 9%,
    /// glass 4%, organics 34% (food+yard), other 15% (no value).
    pub fn revenue_per_ton(&self) -> f64 {
        let mult = self.price_multiplier();
        (self.price_paper * 0.25
            + self.price_plastic * 0.13
            + self.price_metal * 0.09
            + self.price_glass * 0.04
            + self.price_organic * 0.34)
            * mult
    }

    /// Net value per ton of recycled material (revenue minus costs).
    /// Can be negative when market prices are low.
    pub fn net_value_per_ton(&self) -> f64 {
        self.revenue_per_ton() - PROCESSING_COST_PER_TON - COLLECTION_COST_PER_TON
    }

    /// Advance the market cycle based on elapsed game days.
    pub fn update_market_cycle(&mut self, current_day: u32) {
        if current_day <= self.last_update_day {
            return;
        }
        let elapsed = (current_day - self.last_update_day) as f64;
        self.market_cycle_position += elapsed / Self::CYCLE_PERIOD_DAYS;
        // Keep position in [0, 1)
        self.market_cycle_position -= self.market_cycle_position.floor();
        self.last_update_day = current_day;
    }
}

// =============================================================================
// Recycling state
// =============================================================================

/// City-wide recycling program state, updated each slow tick.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct RecyclingState {
    /// Current recycling tier selected by the player.
    pub tier: RecyclingTier,
    /// Tons diverted from landfill this period.
    pub daily_tons_diverted: f64,
    /// Tons contaminated (waste in recycling stream sent to landfill) this period.
    pub daily_tons_contaminated: f64,
    /// Revenue from selling recyclable materials this period.
    pub daily_revenue: f64,
    /// Program operating costs this period (collection + processing).
    pub daily_cost: f64,
    /// Cumulative revenue since game start.
    pub total_revenue: f64,
    /// Cumulative costs since game start.
    pub total_cost: f64,
    /// Number of households participating.
    pub participating_households: u32,
}

impl Default for RecyclingState {
    fn default() -> Self {
        Self {
            tier: RecyclingTier::None,
            daily_tons_diverted: 0.0,
            daily_tons_contaminated: 0.0,
            daily_revenue: 0.0,
            daily_cost: 0.0,
            total_revenue: 0.0,
            total_cost: 0.0,
            participating_households: 0,
        }
    }
}

// =============================================================================
// Systems
// =============================================================================

/// Advances the recycling market cycle and recalculates daily economics.
///
/// Runs on the slow tick (~every 10 game seconds, treated as ~1 game day).
/// Reads the current waste generation from `WasteSystem` and applies the
/// selected recycling tier's diversion rate, contamination, and economics.
#[allow(clippy::too_many_arguments)]
pub fn update_recycling_economics(
    slow_timer: Res<SlowTickTimer>,
    clock: Res<GameClock>,
    mut economics: ResMut<RecyclingEconomics>,
    mut state: ResMut<RecyclingState>,
    waste_system: Res<crate::garbage::WasteSystem>,
    stats: Res<crate::stats::CityStats>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // Advance market cycle
    economics.update_market_cycle(clock.day);

    let tier = state.tier;
    let generated_tons = waste_system.period_generated_tons;

    // Compute households (approximate: population / 3 avg household size)
    let households = (stats.population as f64 / 3.0).max(0.0);
    let participating = (households * tier.participation_rate() as f64) as u32;
    state.participating_households = participating;

    // Diversion: fraction of total waste diverted based on tier
    let gross_diverted = generated_tons * tier.diversion_rate() as f64;

    // Contamination: fraction of diverted material that is actually waste
    let contaminated = gross_diverted * tier.contamination_rate() as f64;
    let net_diverted = gross_diverted - contaminated;

    state.daily_tons_diverted = net_diverted;
    state.daily_tons_contaminated = contaminated;

    // Revenue from selling clean recyclables
    let revenue_per_ton = economics.revenue_per_ton() * tier.revenue_potential() as f64;
    let revenue = net_diverted * revenue_per_ton;

    // Costs: per-household annual cost prorated to daily + per-ton processing
    let daily_household_cost = participating as f64 * tier.cost_per_household_year() / 365.0;
    let per_ton_cost = gross_diverted * (PROCESSING_COST_PER_TON + COLLECTION_COST_PER_TON);
    let total_cost = daily_household_cost + per_ton_cost;

    state.daily_revenue = revenue;
    state.daily_cost = total_cost;
    state.total_revenue += revenue;
    state.total_cost += total_cost;
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // RecyclingTier tests
    // =========================================================================

    #[test]
    fn tier_none_has_lowest_diversion() {
        assert_eq!(RecyclingTier::None.diversion_rate(), 0.05);
    }

    #[test]
    fn tier_zero_waste_has_highest_diversion() {
        assert_eq!(RecyclingTier::ZeroWaste.diversion_rate(), 0.60);
    }

    #[test]
    fn diversion_rates_ordered() {
        // All tiers except SingleStream should have strictly increasing diversion
        // when traversing None -> VoluntaryDropoff -> CurbsideBasic -> CurbsideSort -> ZeroWaste.
        let ordered = [
            RecyclingTier::None,
            RecyclingTier::VoluntaryDropoff,
            RecyclingTier::CurbsideBasic,
            RecyclingTier::CurbsideSort,
            RecyclingTier::ZeroWaste,
        ];
        for pair in ordered.windows(2) {
            assert!(
                pair[0].diversion_rate() < pair[1].diversion_rate(),
                "{:?} diversion ({}) should be less than {:?} ({})",
                pair[0],
                pair[0].diversion_rate(),
                pair[1],
                pair[1].diversion_rate(),
            );
        }
    }

    #[test]
    fn all_tiers_count() {
        assert_eq!(RecyclingTier::all().len(), 7);
    }

    #[test]
    fn contamination_rates_in_valid_range() {
        for tier in RecyclingTier::all() {
            let rate = tier.contamination_rate();
            assert!(
                (0.15..=0.30).contains(&rate),
                "{:?} contamination rate {rate} outside 15%-30%",
                tier,
            );
        }
    }

    #[test]
    fn single_stream_has_higher_contamination_than_curbside_sort() {
        // Single stream mixes materials, so contamination is worse
        assert!(
            RecyclingTier::SingleStream.contamination_rate()
                > RecyclingTier::CurbsideSort.contamination_rate(),
        );
    }

    #[test]
    fn no_program_has_zero_cost() {
        assert_eq!(RecyclingTier::None.cost_per_household_year(), 0.0);
    }

    #[test]
    fn zero_waste_most_expensive() {
        for tier in RecyclingTier::all() {
            assert!(
                tier.cost_per_household_year()
                    <= RecyclingTier::ZeroWaste.cost_per_household_year(),
                "{:?} cost ({}) exceeds ZeroWaste ({})",
                tier,
                tier.cost_per_household_year(),
                RecyclingTier::ZeroWaste.cost_per_household_year(),
            );
        }
    }

    // =========================================================================
    // RecyclingEconomics tests
    // =========================================================================

    #[test]
    fn default_economics_neutral_multiplier() {
        let econ = RecyclingEconomics::default();
        // At position 0.0, sin(0) = 0, so multiplier = 0.9
        let mult = econ.price_multiplier();
        assert!(
            (mult - 0.9).abs() < 0.01,
            "expected ~0.9 at cycle start, got {mult}"
        );
    }

    #[test]
    fn market_cycle_boom() {
        let mut econ = RecyclingEconomics::default();
        // Position 0.25 => sin(TAU*0.25) = sin(PI/2) = 1.0 => mult = 0.9 + 0.6 = 1.5
        econ.market_cycle_position = 0.25;
        let mult = econ.price_multiplier();
        assert!(
            (mult - 1.5).abs() < 0.01,
            "expected ~1.5 at boom, got {mult}"
        );
    }

    #[test]
    fn market_cycle_bust() {
        let mut econ = RecyclingEconomics::default();
        // Position 0.75 => sin(TAU*0.75) = sin(3PI/2) = -1.0 => mult = 0.9 - 0.6 = 0.3
        econ.market_cycle_position = 0.75;
        let mult = econ.price_multiplier();
        assert!(
            (mult - 0.3).abs() < 0.01,
            "expected ~0.3 at bust, got {mult}"
        );
    }

    #[test]
    fn market_cycle_advance() {
        let mut econ = RecyclingEconomics::default();
        econ.last_update_day = 0;
        // Advance half a cycle (912 days)
        econ.update_market_cycle(912);
        assert!(
            (econ.market_cycle_position - 912.0 / 1825.0).abs() < 0.001,
            "cycle position should be ~0.5, got {}",
            econ.market_cycle_position,
        );
        assert_eq!(econ.last_update_day, 912);
    }

    #[test]
    fn market_cycle_wraps_around() {
        let mut econ = RecyclingEconomics::default();
        econ.last_update_day = 0;
        // Advance more than one full cycle
        econ.update_market_cycle(2000);
        assert!(
            econ.market_cycle_position < 1.0,
            "cycle position should wrap, got {}",
            econ.market_cycle_position,
        );
    }

    #[test]
    fn net_value_per_ton_can_be_negative() {
        let mut econ = RecyclingEconomics::default();
        // At bust (0.3x), revenue should be low enough that net is negative
        econ.market_cycle_position = 0.75;
        let net = econ.net_value_per_ton();
        assert!(
            net < 0.0,
            "net value per ton should be negative during bust, got {net}"
        );
    }

    #[test]
    fn net_value_per_ton_positive_at_boom() {
        let mut econ = RecyclingEconomics::default();
        econ.market_cycle_position = 0.25;
        let net = econ.net_value_per_ton();
        assert!(
            net > 0.0,
            "net value per ton should be positive during boom, got {net}"
        );
    }

    #[test]
    fn revenue_per_ton_scales_with_multiplier() {
        let mut econ = RecyclingEconomics::default();
        econ.market_cycle_position = 0.0;
        let rev_start = econ.revenue_per_ton();

        econ.market_cycle_position = 0.25;
        let rev_boom = econ.revenue_per_ton();

        assert!(
            rev_boom > rev_start,
            "boom revenue ({rev_boom}) should exceed start ({rev_start})"
        );
    }

    // =========================================================================
    // RecyclingState tests
    // =========================================================================

    #[test]
    fn default_state_is_no_program() {
        let state = RecyclingState::default();
        assert_eq!(state.tier, RecyclingTier::None);
        assert_eq!(state.daily_tons_diverted, 0.0);
        assert_eq!(state.daily_revenue, 0.0);
        assert_eq!(state.total_revenue, 0.0);
    }

    #[test]
    fn tier_names_are_unique() {
        let names: Vec<&str> = RecyclingTier::all().iter().map(|t| t.name()).collect();
        for (i, name) in names.iter().enumerate() {
            for (j, other) in names.iter().enumerate() {
                if i != j {
                    assert_ne!(name, other, "duplicate tier name at indices {i} and {j}");
                }
            }
        }
    }

    #[test]
    fn participation_rate_increases_with_better_programs() {
        // None should have lowest participation
        assert!(
            RecyclingTier::None.participation_rate()
                < RecyclingTier::ZeroWaste.participation_rate(),
        );
    }

    #[test]
    fn revenue_potential_none_lowest() {
        for tier in RecyclingTier::all() {
            assert!(
                tier.revenue_potential() >= RecyclingTier::None.revenue_potential(),
                "{:?} revenue potential should be >= None",
                tier,
            );
        }
    }
}

pub struct RecyclingPlugin;

impl Plugin for RecyclingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RecyclingEconomics>()
            .init_resource::<RecyclingState>()
            .add_systems(
                FixedUpdate,
                update_recycling_economics
                    .after(crate::garbage::update_waste_generation)
                    .in_set(crate::SimulationSet::Simulation),
            );
    }
}
