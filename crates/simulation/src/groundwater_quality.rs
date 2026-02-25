//! POLL-028: Groundwater Quality System Enhancement
//!
//! Enhances groundwater quality tracking with:
//! - Landfill leachate contamination (unlined=radius 10, lined=radius 3)
//! - Industrial discharge contamination (radius 5)
//! - Treatment plant quality recovery
//! - Drinking water quality tier based on average quality at well locations
//!
//! This module creates new systems that read/write the existing
//! `WaterQualityGrid` resource without modifying `groundwater.rs`.

use bevy::prelude::*;

use crate::buildings::Building;
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::ZoneType;
use crate::groundwater::WaterQualityGrid;
use crate::landfill::LandfillState;
use crate::services::{ServiceBuilding, ServiceType};
use crate::SlowTickTimer;

// =============================================================================
// Constants
// =============================================================================

/// Contamination radius for unlined landfills (in grid cells).
pub const LANDFILL_UNLINED_RADIUS: i32 = 10;

/// Contamination radius for lined landfills (in grid cells).
pub const LANDFILL_LINED_RADIUS: i32 = 3;

/// Contamination radius for industrial buildings (in grid cells).
pub const INDUSTRIAL_DISCHARGE_RADIUS: i32 = 5;

/// Max quality degradation per tick at the center of an unlined landfill plume.
pub const LANDFILL_UNLINED_INTENSITY: u8 = 12;

/// Max quality degradation per tick at the center of a lined landfill plume.
pub const LANDFILL_LINED_INTENSITY: u8 = 4;

/// Max quality degradation per tick at the center of an industrial discharge.
pub const INDUSTRIAL_DISCHARGE_INTENSITY: u8 = 6;

/// Treatment plant quality recovery radius (in grid cells).
pub const TREATMENT_RECOVERY_RADIUS: i32 = 15;

/// Max quality recovery per tick at the center of a treatment plant.
pub const TREATMENT_RECOVERY_INTENSITY: u8 = 8;

// =============================================================================
// Drinking Water Quality Tier
// =============================================================================

/// Drinking water quality tier, derived from average groundwater quality
/// at well pump locations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DrinkingWaterTier {
    /// Quality >= 180: excellent, safe for consumption.
    Excellent,
    /// Quality 120..180: good, meets standards.
    #[default]
    Good,
    /// Quality 60..120: fair, some treatment recommended.
    Fair,
    /// Quality < 60: poor, health risk.
    Poor,
}

impl DrinkingWaterTier {
    /// Classify a quality value (0-255) into a tier.
    pub fn from_quality(quality: u8) -> Self {
        match quality {
            180..=255 => Self::Excellent,
            120..=179 => Self::Good,
            60..=119 => Self::Fair,
            _ => Self::Poor,
        }
    }

    /// Returns a human-readable label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Excellent => "Excellent",
            Self::Good => "Good",
            Self::Fair => "Fair",
            Self::Poor => "Poor",
        }
    }
}

// =============================================================================
// Drinking Water Quality Stats Resource
// =============================================================================

/// Aggregated drinking water quality information derived from well locations.
#[derive(Resource, Debug, Clone, Default)]
pub struct DrinkingWaterQuality {
    /// Average quality at well pump locations (0.0-255.0).
    pub avg_well_quality: f32,
    /// Current drinking water tier.
    pub tier: DrinkingWaterTier,
    /// Number of well pumps sampled.
    pub well_count: u32,
}

// =============================================================================
// Systems
// =============================================================================

/// Degrades groundwater quality near active landfill sites based on liner type.
///
/// - Unlined landfills: radius 10, intensity 12
/// - Lined landfills (with or without gas collection): radius 3, intensity 4
pub fn landfill_leachate_contamination(
    slow_timer: Res<SlowTickTimer>,
    landfill_state: Res<LandfillState>,
    mut quality: ResMut<WaterQualityGrid>,
) {
    if !slow_timer.should_run() {
        return;
    }

    for site in &landfill_state.sites {
        // Only active and recently-closed landfills leach.
        // Converted-to-park sites no longer contaminate.
        if matches!(
            site.status,
            crate::landfill::LandfillStatus::ConvertedToPark
        ) {
            continue;
        }

        let (radius, intensity) = match site.liner_type {
            crate::landfill::LandfillLinerType::Unlined => {
                (LANDFILL_UNLINED_RADIUS, LANDFILL_UNLINED_INTENSITY)
            }
            _ => (LANDFILL_LINED_RADIUS, LANDFILL_LINED_INTENSITY),
        };

        apply_contamination_plume(
            &mut quality,
            site.grid_x as i32,
            site.grid_y as i32,
            radius,
            intensity,
        );
    }
}

/// Degrades groundwater quality near industrial buildings (radius 5).
pub fn industrial_discharge_contamination(
    slow_timer: Res<SlowTickTimer>,
    buildings: Query<&Building>,
    mut quality: ResMut<WaterQualityGrid>,
) {
    if !slow_timer.should_run() {
        return;
    }

    for building in &buildings {
        if building.zone_type != ZoneType::Industrial {
            continue;
        }

        apply_contamination_plume(
            &mut quality,
            building.grid_x as i32,
            building.grid_y as i32,
            INDUSTRIAL_DISCHARGE_RADIUS,
            INDUSTRIAL_DISCHARGE_INTENSITY,
        );
    }
}

/// Treatment plants improve groundwater quality in their vicinity.
pub fn treatment_plant_quality_recovery(
    slow_timer: Res<SlowTickTimer>,
    services: Query<&ServiceBuilding>,
    mut quality: ResMut<WaterQualityGrid>,
) {
    if !slow_timer.should_run() {
        return;
    }

    for service in &services {
        if service.service_type != ServiceType::WaterTreatmentPlant {
            continue;
        }

        let cx = service.grid_x as i32;
        let cy = service.grid_y as i32;

        for dy in -TREATMENT_RECOVERY_RADIUS..=TREATMENT_RECOVERY_RADIUS {
            for dx in -TREATMENT_RECOVERY_RADIUS..=TREATMENT_RECOVERY_RADIUS {
                let nx = cx + dx;
                let ny = cy + dy;
                if nx < 0
                    || ny < 0
                    || (nx as usize) >= GRID_WIDTH
                    || (ny as usize) >= GRID_HEIGHT
                {
                    continue;
                }
                let dist = dx.abs() + dy.abs();
                let effect = TREATMENT_RECOVERY_INTENSITY.saturating_sub(dist as u8);
                if effect > 0 {
                    let ux = nx as usize;
                    let uy = ny as usize;
                    let idx = uy * GRID_WIDTH + ux;
                    quality.levels[idx] = quality.levels[idx].saturating_add(effect);
                }
            }
        }
    }
}

/// Computes average groundwater quality at well pump locations and derives
/// the drinking water quality tier.
pub fn update_drinking_water_quality(
    slow_timer: Res<SlowTickTimer>,
    services: Query<&ServiceBuilding>,
    quality: Res<WaterQualityGrid>,
    mut drinking: ResMut<DrinkingWaterQuality>,
) {
    if !slow_timer.should_run() {
        return;
    }

    let mut sum: u64 = 0;
    let mut count: u32 = 0;

    for service in &services {
        if service.service_type != ServiceType::WellPump {
            continue;
        }
        let x = service.grid_x;
        let y = service.grid_y;
        if x < GRID_WIDTH && y < GRID_HEIGHT {
            sum += quality.get(x, y) as u64;
            count += 1;
        }
    }

    if count > 0 {
        let avg = sum as f32 / count as f32;
        drinking.avg_well_quality = avg;
        drinking.tier = DrinkingWaterTier::from_quality(avg as u8);
    } else {
        // No wells: default to Good based on overall grid average
        drinking.avg_well_quality = 200.0;
        drinking.tier = DrinkingWaterTier::Good;
    }
    drinking.well_count = count;
}

// =============================================================================
// Helpers
// =============================================================================

/// Apply a circular contamination plume centered at (cx, cy) with the given
/// radius and peak intensity. Quality degrades more at the center and
/// falls off linearly with Manhattan distance.
fn apply_contamination_plume(
    quality: &mut WaterQualityGrid,
    cx: i32,
    cy: i32,
    radius: i32,
    intensity: u8,
) {
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let nx = cx + dx;
            let ny = cy + dy;
            if nx < 0 || ny < 0 || (nx as usize) >= GRID_WIDTH || (ny as usize) >= GRID_HEIGHT {
                continue;
            }
            let dist = dx.abs() + dy.abs();
            if dist > radius {
                continue;
            }
            let decay = intensity
                .saturating_sub((dist as u32 * intensity as u32 / radius.max(1) as u32) as u8);
            if decay > 0 {
                let ux = nx as usize;
                let uy = ny as usize;
                let idx = uy * GRID_WIDTH + ux;
                quality.levels[idx] = quality.levels[idx].saturating_sub(decay);
            }
        }
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct GroundwaterQualityPlugin;

impl Plugin for GroundwaterQualityPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DrinkingWaterQuality>().add_systems(
            FixedUpdate,
            (
                landfill_leachate_contamination,
                industrial_discharge_contamination,
                treatment_plant_quality_recovery,
                update_drinking_water_quality,
            )
                .after(crate::groundwater::update_groundwater)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}

// =============================================================================
// Unit tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drinking_water_tier_excellent() {
        assert_eq!(
            DrinkingWaterTier::from_quality(200),
            DrinkingWaterTier::Excellent
        );
        assert_eq!(
            DrinkingWaterTier::from_quality(180),
            DrinkingWaterTier::Excellent
        );
        assert_eq!(
            DrinkingWaterTier::from_quality(255),
            DrinkingWaterTier::Excellent
        );
    }

    #[test]
    fn test_drinking_water_tier_good() {
        assert_eq!(
            DrinkingWaterTier::from_quality(150),
            DrinkingWaterTier::Good
        );
        assert_eq!(
            DrinkingWaterTier::from_quality(120),
            DrinkingWaterTier::Good
        );
        assert_eq!(
            DrinkingWaterTier::from_quality(179),
            DrinkingWaterTier::Good
        );
    }

    #[test]
    fn test_drinking_water_tier_fair() {
        assert_eq!(
            DrinkingWaterTier::from_quality(100),
            DrinkingWaterTier::Fair
        );
        assert_eq!(
            DrinkingWaterTier::from_quality(60),
            DrinkingWaterTier::Fair
        );
        assert_eq!(
            DrinkingWaterTier::from_quality(119),
            DrinkingWaterTier::Fair
        );
    }

    #[test]
    fn test_drinking_water_tier_poor() {
        assert_eq!(
            DrinkingWaterTier::from_quality(0),
            DrinkingWaterTier::Poor
        );
        assert_eq!(
            DrinkingWaterTier::from_quality(30),
            DrinkingWaterTier::Poor
        );
        assert_eq!(
            DrinkingWaterTier::from_quality(59),
            DrinkingWaterTier::Poor
        );
    }

    #[test]
    fn test_contamination_plume_degrades_center() {
        let mut quality = WaterQualityGrid::default();
        let center_x = 50;
        let center_y = 50;
        let before = quality.get(center_x as usize, center_y as usize);

        apply_contamination_plume(&mut quality, center_x, center_y, 5, 10);

        let after = quality.get(center_x as usize, center_y as usize);
        assert!(
            after < before,
            "center should be degraded: before={before}, after={after}"
        );
    }

    #[test]
    fn test_contamination_plume_falloff() {
        let mut quality = WaterQualityGrid::default();
        apply_contamination_plume(&mut quality, 50, 50, 10, 12);

        let center_q = quality.get(50, 50);
        let mid_q = quality.get(55, 50); // dist=5
        let edge_q = quality.get(60, 50); // dist=10

        assert!(
            center_q < mid_q,
            "center({center_q}) should be worse than mid({mid_q})"
        );
        assert!(
            mid_q < edge_q,
            "mid({mid_q}) should be worse than edge({edge_q})"
        );
    }

    #[test]
    fn test_contamination_plume_beyond_radius_unaffected() {
        let mut quality = WaterQualityGrid::default();
        let original = quality.get(65, 50);

        apply_contamination_plume(&mut quality, 50, 50, 10, 12);

        // Manhattan distance 15 > radius 10, should be unaffected
        assert_eq!(quality.get(65, 50), original);
    }

    #[test]
    fn test_contamination_plume_saturating_sub() {
        let mut quality = WaterQualityGrid::default();
        quality.set(50, 50, 5); // nearly zero already

        apply_contamination_plume(&mut quality, 50, 50, 5, 20);

        // Should clamp at 0, not underflow
        assert_eq!(quality.get(50, 50), 0);
    }

    #[test]
    fn test_unlined_larger_radius_than_lined() {
        assert!(
            LANDFILL_UNLINED_RADIUS > LANDFILL_LINED_RADIUS,
            "Unlined radius ({}) should exceed lined radius ({})",
            LANDFILL_UNLINED_RADIUS,
            LANDFILL_LINED_RADIUS,
        );
    }

    #[test]
    fn test_unlined_more_intense_than_lined() {
        assert!(
            LANDFILL_UNLINED_INTENSITY > LANDFILL_LINED_INTENSITY,
            "Unlined intensity ({}) should exceed lined intensity ({})",
            LANDFILL_UNLINED_INTENSITY,
            LANDFILL_LINED_INTENSITY,
        );
    }

    #[test]
    fn test_drinking_water_tier_labels() {
        assert_eq!(DrinkingWaterTier::Excellent.label(), "Excellent");
        assert_eq!(DrinkingWaterTier::Good.label(), "Good");
        assert_eq!(DrinkingWaterTier::Fair.label(), "Fair");
        assert_eq!(DrinkingWaterTier::Poor.label(), "Poor");
    }
}
