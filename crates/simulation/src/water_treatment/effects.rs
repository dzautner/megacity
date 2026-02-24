//! Water treatment grid effects (SVC-017).
//!
//! Connects the `WaterTreatmentState` aggregate metrics to per-cell grids:
//! - Treatment plants reduce water pollution in their vicinity
//! - Untreated overflow increases water pollution near discharge points
//! - WellPump provides clean water in low-pollution areas (boosts quality)
//! - Treatment coverage feeds back to the per-area `WaterQualityGrid`

use bevy::prelude::*;

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::groundwater::WaterQualityGrid;
use crate::services::{ServiceBuilding, ServiceType};
use crate::water_pollution::WaterPollutionGrid;
use crate::SlowTickTimer;

use super::WaterTreatmentState;

// =============================================================================
// Constants
// =============================================================================

/// Radius (grid cells) around a treatment plant within which pollution is reduced.
const TREATMENT_EFFECT_RADIUS: i32 = 10;

/// Base pollution reduction per cell at distance 0 from a treatment plant.
/// Actual reduction is scaled by the plant's removal efficiency.
const BASE_POLLUTION_REDUCTION: f32 = 20.0;

/// Base quality boost per cell at distance 0 from a treatment plant.
/// Scaled by removal efficiency.
const BASE_QUALITY_BOOST: f32 = 15.0;

/// Radius (grid cells) around a WellPump that boosts water quality.
const WELL_PUMP_RADIUS: i32 = 8;

/// Quality boost from a WellPump in clean areas (low pollution).
const WELL_PUMP_QUALITY_BOOST: u8 = 12;

/// Pollution threshold below which a WellPump provides clean water.
const WELL_PUMP_CLEAN_THRESHOLD: u8 = 30;

/// Pollution added per cell when untreated overflow occurs.
/// Scaled by the overflow fraction (overflow / total demand).
const OVERFLOW_POLLUTION_BASE: f32 = 8.0;

/// Radius around treatment plants for overflow discharge effects.
const OVERFLOW_DISCHARGE_RADIUS: i32 = 12;

// =============================================================================
// Systems
// =============================================================================

/// Apply treatment plant effects to the water pollution and quality grids.
///
/// For each WaterTreatmentPlant service building:
/// - Look up its treatment level from `WaterTreatmentState`
/// - Reduce water pollution in a radius proportional to removal efficiency
/// - Boost water quality in a radius proportional to removal efficiency
///
/// Also handles overflow: when treatment_coverage < 1.0, untreated sewage
/// increases water pollution near treatment plants (or city center if none).
#[allow(clippy::too_many_arguments)]
pub fn apply_treatment_grid_effects(
    slow_timer: Res<SlowTickTimer>,
    state: Res<WaterTreatmentState>,
    services: Query<(Entity, &ServiceBuilding)>,
    mut water_pollution: ResMut<WaterPollutionGrid>,
    mut water_quality: ResMut<WaterQualityGrid>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // --- Phase 1: Treatment plant pollution reduction and quality boost ---
    for (entity, service) in &services {
        if service.service_type != ServiceType::WaterTreatmentPlant {
            continue;
        }

        // Look up this plant's treatment level
        let efficiency = state
            .plants
            .get(&entity)
            .map(|p| p.level.removal_efficiency())
            .unwrap_or(0.6); // Default to Primary if not yet registered

        let px = service.grid_x as i32;
        let py = service.grid_y as i32;

        for dy in -TREATMENT_EFFECT_RADIUS..=TREATMENT_EFFECT_RADIUS {
            for dx in -TREATMENT_EFFECT_RADIUS..=TREATMENT_EFFECT_RADIUS {
                let nx = px + dx;
                let ny = py + dy;
                if nx < 0
                    || ny < 0
                    || (nx as usize) >= GRID_WIDTH
                    || (ny as usize) >= GRID_HEIGHT
                {
                    continue;
                }
                let ux = nx as usize;
                let uy = ny as usize;
                let dist = dx.abs() + dy.abs();

                // Pollution reduction: scales with efficiency and falls off with distance
                let reduction =
                    (BASE_POLLUTION_REDUCTION * efficiency - dist as f32 * 1.5).max(0.0) as u8;
                if reduction > 0 {
                    let idx = uy * water_pollution.width + ux;
                    water_pollution.levels[idx] =
                        water_pollution.levels[idx].saturating_sub(reduction);
                }

                // Quality boost: scales with efficiency and falls off with distance
                let boost = (BASE_QUALITY_BOOST * efficiency - dist as f32).max(0.0) as u8;
                if boost > 0 {
                    let idx = uy * water_quality.width + ux;
                    water_quality.levels[idx] = water_quality.levels[idx].saturating_add(boost);
                }
            }
        }
    }

    // --- Phase 2: Overflow effects (untreated sewage increases pollution) ---
    if state.treatment_coverage < 1.0 && state.city_demand_mgd > 0.0 {
        let overflow_fraction = 1.0 - state.treatment_coverage;
        let pollution_amount =
            (OVERFLOW_POLLUTION_BASE * overflow_fraction).clamp(1.0, 25.0) as u8;

        // Discharge near treatment plants, or city center if none
        let discharge_centers: Vec<(i32, i32)> = services
            .iter()
            .filter(|(_, sb)| sb.service_type == ServiceType::WaterTreatmentPlant)
            .map(|(_, sb)| (sb.grid_x as i32, sb.grid_y as i32))
            .collect();

        let centers = if discharge_centers.is_empty() {
            vec![(GRID_WIDTH as i32 / 2, GRID_HEIGHT as i32 / 2)]
        } else {
            discharge_centers
        };

        for (cx, cy) in &centers {
            for dy in -OVERFLOW_DISCHARGE_RADIUS..=OVERFLOW_DISCHARGE_RADIUS {
                for dx in -OVERFLOW_DISCHARGE_RADIUS..=OVERFLOW_DISCHARGE_RADIUS {
                    let nx = cx + dx;
                    let ny = cy + dy;
                    if nx < 0
                        || ny < 0
                        || (nx as usize) >= GRID_WIDTH
                        || (ny as usize) >= GRID_HEIGHT
                    {
                        continue;
                    }
                    let ux = nx as usize;
                    let uy = ny as usize;
                    let dist = dx.abs() + dy.abs();

                    // Pollution falls off with distance
                    let amount = pollution_amount.saturating_sub(dist as u8);
                    if amount > 0 {
                        let idx = uy * water_pollution.width + ux;
                        water_pollution.levels[idx] =
                            water_pollution.levels[idx].saturating_add(amount);
                    }
                }
            }
        }
    }
}

/// WellPump system: in low-pollution areas, well pumps boost water quality.
///
/// For each WellPump service building, if the local water pollution is below
/// the clean threshold, the pump boosts water quality in its radius.
pub fn apply_well_pump_effects(
    slow_timer: Res<SlowTickTimer>,
    services: Query<&ServiceBuilding>,
    water_pollution: Res<WaterPollutionGrid>,
    mut water_quality: ResMut<WaterQualityGrid>,
) {
    if !slow_timer.should_run() {
        return;
    }

    for service in &services {
        if service.service_type != ServiceType::WellPump {
            continue;
        }

        let px = service.grid_x;
        let py = service.grid_y;

        // Check local pollution level at the pump site
        if px >= GRID_WIDTH || py >= GRID_HEIGHT {
            continue;
        }
        let local_pollution = water_pollution.get(px, py);

        // Only provide clean water if the area is relatively unpolluted
        if local_pollution > WELL_PUMP_CLEAN_THRESHOLD {
            continue;
        }

        // Effectiveness inversely proportional to remaining pollution
        let effectiveness =
            1.0 - (local_pollution as f32 / WELL_PUMP_CLEAN_THRESHOLD as f32).clamp(0.0, 1.0);
        let boost_base = (WELL_PUMP_QUALITY_BOOST as f32 * effectiveness) as u8;

        for dy in -WELL_PUMP_RADIUS..=WELL_PUMP_RADIUS {
            for dx in -WELL_PUMP_RADIUS..=WELL_PUMP_RADIUS {
                let nx = px as i32 + dx;
                let ny = py as i32 + dy;
                if nx < 0
                    || ny < 0
                    || (nx as usize) >= GRID_WIDTH
                    || (ny as usize) >= GRID_HEIGHT
                {
                    continue;
                }
                let ux = nx as usize;
                let uy = ny as usize;
                let dist = dx.abs() + dy.abs();
                let boost = boost_base.saturating_sub(dist as u8);
                if boost > 0 {
                    let idx = uy * water_quality.width + ux;
                    water_quality.levels[idx] = water_quality.levels[idx].saturating_add(boost);
                }
            }
        }
    }
}
