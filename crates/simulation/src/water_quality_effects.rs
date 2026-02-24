//! POLL-008: Water Quality Effects on Citizens and Fisheries
//!
//! Classifies water quality into 6 tiers (Pristine through Toxic) based on
//! the `WaterQualityGrid` and `WaterPollutionGrid`, then applies:
//! - Health effects per tier (bonus for pristine, penalty for polluted+)
//! - Tourism/natural beauty bonus for pristine water areas
//! - Drinking water treatment cost scaling from $500/MG (clean) to $5000/MG

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::citizen::{Citizen, CitizenDetails, HomeLocation};
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::economy::CityBudget;
use crate::groundwater::WaterQualityGrid;
use crate::tourism::Tourism;
use crate::water_pollution::WaterPollutionGrid;
use crate::SlowTickTimer;

// ---------------------------------------------------------------------------
// Water Quality Tier Classification
// ---------------------------------------------------------------------------

/// 6-tier water quality classification mapped from the u8 (0-255)
/// `WaterQualityGrid` levels. Higher quality values mean cleaner water.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WaterQualityTier {
    /// Quality 230-255: crystal-clear, untouched water
    Pristine,
    /// Quality 180-229: safe for all uses
    Clean,
    /// Quality 120-179: acceptable with basic treatment
    Moderate,
    /// Quality 60-119: requires significant treatment
    Polluted,
    /// Quality 20-59: dangerous, heavy contamination
    Heavy,
    /// Quality 0-19: lethal levels of contamination
    Toxic,
}

impl WaterQualityTier {
    /// Classify a water quality level (0=contaminated, 255=pure) into a tier.
    pub fn from_quality(quality: u8) -> Self {
        match quality {
            230..=255 => Self::Pristine,
            180..=229 => Self::Clean,
            120..=179 => Self::Moderate,
            60..=119 => Self::Polluted,
            20..=59 => Self::Heavy,
            0..=19 => Self::Toxic,
        }
    }

    /// Per-slow-tick health modifier. Positive = healing, negative = damage.
    pub fn health_modifier(&self) -> f32 {
        match self {
            Self::Pristine => 0.02,
            Self::Clean => 0.005,
            Self::Moderate => 0.0,
            Self::Polluted => -0.03,
            Self::Heavy => -0.08,
            Self::Toxic => -0.15,
        }
    }

    /// Drinking water treatment cost per million gallons (MG).
    pub fn treatment_cost_per_mg(&self) -> f64 {
        match self {
            Self::Pristine => 200.0,
            Self::Clean => 500.0,
            Self::Moderate => 1_500.0,
            Self::Polluted => 3_000.0,
            Self::Heavy => 5_000.0,
            Self::Toxic => 8_000.0,
        }
    }

    /// Tourism/natural beauty bonus for water of this tier.
    pub fn tourism_beauty_bonus(&self) -> f32 {
        match self {
            Self::Pristine => 5.0,
            Self::Clean => 2.0,
            Self::Moderate => 0.0,
            Self::Polluted => -2.0,
            Self::Heavy => -5.0,
            Self::Toxic => -8.0,
        }
    }
}

// ---------------------------------------------------------------------------
// State resource
// ---------------------------------------------------------------------------

/// Tracks aggregate water quality effects across the city.
#[derive(Resource, Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
pub struct WaterQualityEffects {
    /// Count of cells per tier (index 0=Pristine .. 5=Toxic).
    pub tier_counts: [u32; 6],
    /// City-wide average water quality (0-255).
    pub avg_quality: f32,
    /// Dominant tier index (0=Pristine, 5=Toxic).
    pub dominant_tier_idx: u8,
    /// Treatment cost applied last slow tick.
    pub treatment_cost_modifier: f64,
    /// Tourism bonus applied last slow tick.
    pub tourism_bonus_applied: f32,
}

impl WaterQualityEffects {
    pub fn tier_index(tier: WaterQualityTier) -> usize {
        match tier {
            WaterQualityTier::Pristine => 0,
            WaterQualityTier::Clean => 1,
            WaterQualityTier::Moderate => 2,
            WaterQualityTier::Polluted => 3,
            WaterQualityTier::Heavy => 4,
            WaterQualityTier::Toxic => 5,
        }
    }
}

impl crate::Saveable for WaterQualityEffects {
    const SAVE_KEY: &'static str = "water_quality_effects";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.tier_counts.iter().all(|&c| c == 0) {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

/// Effective quality = base quality minus half of surface pollution.
pub fn effective_quality(
    x: usize,
    y: usize,
    wq: &WaterQualityGrid,
    wp: &WaterPollutionGrid,
) -> u8 {
    wq.get(x, y).saturating_sub(wp.get(x, y) / 2)
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Classify water cells into tiers and apply treatment cost scaling.
/// Must run before health/tourism systems so tier_counts are populated.
#[allow(clippy::too_many_arguments)]
pub fn classify_water_quality_tiers(
    slow_timer: Res<SlowTickTimer>,
    water_quality: Res<WaterQualityGrid>,
    water_pollution: Res<WaterPollutionGrid>,
    grid: Res<crate::grid::WorldGrid>,
    stats: Res<crate::stats::CityStats>,
    mut effects: ResMut<WaterQualityEffects>,
    mut budget: ResMut<CityBudget>,
) {
    if !slow_timer.should_run() {
        return;
    }

    let mut counts = [0u32; 6];
    let mut quality_sum: u64 = 0;
    let mut water_cell_count: u32 = 0;

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let cell = grid.get(x, y);
            if cell.cell_type != crate::grid::CellType::Water && !cell.has_water {
                continue;
            }
            let eff = effective_quality(x, y, &water_quality, &water_pollution);
            counts[WaterQualityEffects::tier_index(WaterQualityTier::from_quality(eff))] += 1;
            quality_sum += eff as u64;
            water_cell_count += 1;
        }
    }

    effects.tier_counts = counts;
    effects.avg_quality = if water_cell_count > 0 {
        quality_sum as f32 / water_cell_count as f32
    } else {
        200.0
    };
    effects.dominant_tier_idx = counts
        .iter()
        .enumerate()
        .max_by_key(|(_, &c)| c)
        .map(|(i, _)| i as u8)
        .unwrap_or(1);

    // Treatment cost scaling: deduct from treasury only (not monthly_expenses
    // to avoid breaking economy breakdown invariants).
    let pop = stats.population;
    if pop > 0 {
        let demand_mg = (pop as f64 * 150.0) / 1_000_000.0;
        let cost = WaterQualityTier::from_quality(effects.avg_quality as u8)
            .treatment_cost_per_mg();
        let tick_cost = demand_mg * cost * 0.33;
        effects.treatment_cost_modifier = tick_cost;
        budget.treasury -= tick_cost;
    } else {
        effects.treatment_cost_modifier = 0.0;
    }
}

/// Apply health effects to citizens near water cells (radius 3).
pub fn apply_water_quality_health_effects(
    slow_timer: Res<SlowTickTimer>,
    water_quality: Res<WaterQualityGrid>,
    water_pollution: Res<WaterPollutionGrid>,
    grid: Res<crate::grid::WorldGrid>,
    mut citizens: Query<(&mut CitizenDetails, &HomeLocation), With<Citizen>>,
) {
    if !slow_timer.should_run() {
        return;
    }
    let r = 3i32;
    for (mut details, home) in &mut citizens {
        let (hx, hy) = (home.grid_x, home.grid_y);
        let mut found = false;
        let mut best: u8 = 0;
        let mut worst: u8 = 255;

        for dy in -r..=r {
            for dx in -r..=r {
                let nx = hx as i32 + dx;
                let ny = hy as i32 + dy;
                if nx < 0 || ny < 0 || nx as usize >= GRID_WIDTH || ny as usize >= GRID_HEIGHT {
                    continue;
                }
                let (ux, uy) = (nx as usize, ny as usize);
                if grid.get(ux, uy).cell_type != crate::grid::CellType::Water {
                    continue;
                }
                found = true;
                let eff = effective_quality(ux, uy, &water_quality, &water_pollution);
                best = best.max(eff);
                worst = worst.min(eff);
            }
        }
        if !found {
            continue;
        }
        let modifier = if worst < 120 {
            WaterQualityTier::from_quality(worst).health_modifier()
        } else {
            WaterQualityTier::from_quality(best).health_modifier()
        };
        details.health = (details.health + modifier).clamp(0.0, 100.0);
    }
}

/// Apply tourism/natural beauty bonus based on water quality distribution.
pub fn apply_water_quality_tourism_bonus(
    slow_timer: Res<SlowTickTimer>,
    mut effects: ResMut<WaterQualityEffects>,
    mut tourism: ResMut<Tourism>,
) {
    if !slow_timer.should_run() {
        return;
    }
    let total: u32 = effects.tier_counts.iter().sum();
    if total == 0 {
        effects.tourism_bonus_applied = 0.0;
        return;
    }
    let tiers = [
        WaterQualityTier::Pristine,
        WaterQualityTier::Clean,
        WaterQualityTier::Moderate,
        WaterQualityTier::Polluted,
        WaterQualityTier::Heavy,
        WaterQualityTier::Toxic,
    ];
    let mut bonus: f32 = 0.0;
    for (i, t) in tiers.iter().enumerate() {
        bonus += t.tourism_beauty_bonus() * (effects.tier_counts[i] as f32 / total as f32);
    }
    let capped = bonus.clamp(-10.0, 10.0);
    effects.tourism_bonus_applied = capped;
    tourism.natural_beauty_score = (tourism.natural_beauty_score + capped).clamp(0.0, 100.0);
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct WaterQualityEffectsPlugin;

impl Plugin for WaterQualityEffectsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WaterQualityEffects>();
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<WaterQualityEffects>();
        app.add_systems(
            FixedUpdate,
            (
                classify_water_quality_tiers,
                (
                    apply_water_quality_health_effects,
                    apply_water_quality_tourism_bonus,
                )
                    .after(classify_water_quality_tiers),
            )
                .after(crate::water_pollution::update_water_pollution)
                .after(crate::groundwater::update_groundwater)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}

// ---------------------------------------------------------------------------
// Unit Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Saveable;

    #[test]
    fn test_tier_boundaries() {
        assert_eq!(WaterQualityTier::from_quality(255), WaterQualityTier::Pristine);
        assert_eq!(WaterQualityTier::from_quality(230), WaterQualityTier::Pristine);
        assert_eq!(WaterQualityTier::from_quality(229), WaterQualityTier::Clean);
        assert_eq!(WaterQualityTier::from_quality(180), WaterQualityTier::Clean);
        assert_eq!(WaterQualityTier::from_quality(179), WaterQualityTier::Moderate);
        assert_eq!(WaterQualityTier::from_quality(120), WaterQualityTier::Moderate);
        assert_eq!(WaterQualityTier::from_quality(119), WaterQualityTier::Polluted);
        assert_eq!(WaterQualityTier::from_quality(60), WaterQualityTier::Polluted);
        assert_eq!(WaterQualityTier::from_quality(59), WaterQualityTier::Heavy);
        assert_eq!(WaterQualityTier::from_quality(20), WaterQualityTier::Heavy);
        assert_eq!(WaterQualityTier::from_quality(19), WaterQualityTier::Toxic);
        assert_eq!(WaterQualityTier::from_quality(0), WaterQualityTier::Toxic);
    }

    #[test]
    fn test_health_modifier_pristine_is_002() {
        assert!((WaterQualityTier::Pristine.health_modifier() - 0.02).abs() < f32::EPSILON);
    }

    #[test]
    fn test_health_modifier_monotonic() {
        let vals: Vec<f32> = [255u8, 200, 150, 100, 40, 5]
            .iter()
            .map(|&q| WaterQualityTier::from_quality(q).health_modifier())
            .collect();
        for w in vals.windows(2) {
            assert!(w[0] >= w[1], "{} should be >= {}", w[0], w[1]);
        }
    }

    #[test]
    fn test_treatment_cost_monotonic() {
        let vals: Vec<f64> = [255u8, 200, 150, 100, 40, 5]
            .iter()
            .map(|&q| WaterQualityTier::from_quality(q).treatment_cost_per_mg())
            .collect();
        for w in vals.windows(2) {
            assert!(w[1] >= w[0], "{} should be >= {}", w[1], w[0]);
        }
    }

    #[test]
    fn test_treatment_cost_spec_values() {
        assert!((WaterQualityTier::Clean.treatment_cost_per_mg() - 500.0).abs() < f64::EPSILON);
        assert!((WaterQualityTier::Heavy.treatment_cost_per_mg() - 5000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_tourism_bonus_signs() {
        assert!(WaterQualityTier::Pristine.tourism_beauty_bonus() > 0.0);
        assert!(WaterQualityTier::Polluted.tourism_beauty_bonus() < 0.0);
    }

    #[test]
    fn test_effective_quality_calc() {
        let mut wq = WaterQualityGrid::default();
        let mut wp = WaterPollutionGrid::default();
        wq.set(5, 5, 200);
        wp.set(5, 5, 40);
        assert_eq!(effective_quality(5, 5, &wq, &wp), 180);
    }

    #[test]
    fn test_effective_quality_saturates() {
        let mut wq = WaterQualityGrid::default();
        let mut wp = WaterPollutionGrid::default();
        wq.set(5, 5, 10);
        wp.set(5, 5, 200);
        assert_eq!(effective_quality(5, 5, &wq, &wp), 0);
    }

    #[test]
    fn test_saveable_skip_default() {
        assert!(WaterQualityEffects::default().save_to_bytes().is_none());
    }

    #[test]
    fn test_saveable_roundtrip() {
        let mut e = WaterQualityEffects::default();
        e.tier_counts = [10, 20, 30, 40, 50, 60];
        e.avg_quality = 150.0;
        let bytes = e.save_to_bytes().unwrap();
        let r = WaterQualityEffects::load_from_bytes(&bytes);
        assert_eq!(r.tier_counts, e.tier_counts);
        assert!((r.avg_quality - 150.0).abs() < 0.01);
    }
}
