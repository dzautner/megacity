//! Advisor logic for Safety, Environment, Housing, Traffic, Zone Demand,
//! and Fire Coverage domains.

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::WorldGrid;

use super::advice_core::{find_fire_coverage_gap, find_first_fire, find_worst_cell};
use super::types::{AdvisorExtras, AdvisorMessage, AdvisorType, TipId};

// ---------------------------------------------------------------------------
// Safety advice
// ---------------------------------------------------------------------------

pub(crate) fn safety_advice(
    tick: u64,
    _grid: &WorldGrid,
    extras: &AdvisorExtras,
    msgs: &mut Vec<AdvisorMessage>,
) {
    // Average crime level
    let total = extras.crime.levels.len() as f32;
    let crime_sum: f32 = extras.crime.levels.iter().map(|&v| v as f32).sum();
    let avg_crime = if total > 0.0 { crime_sum / total } else { 0.0 };

    if avg_crime > 50.0 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Safety,
            tip_id: TipId::CrimeCritical,
            message: "Crime rates are dangerously high!".into(),
            priority: 5,
            suggestion: "Build police stations in high-crime areas and increase police budget."
                .into(),
            tick_created: tick,
            location: find_worst_cell(&extras.crime.levels, GRID_WIDTH),
        });
    } else if avg_crime > 20.0 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Safety,
            tip_id: TipId::CrimeRising,
            message: "Crime is on the rise in some neighborhoods.".into(),
            priority: 3,
            suggestion: "Consider adding police stations to affected areas.".into(),
            tick_created: tick,
            location: find_worst_cell(&extras.crime.levels, GRID_WIDTH),
        });
    }

    // Count active fires
    let active_fires = extras.fire.fire_levels.iter().filter(|&&v| v > 0).count();
    if active_fires > 10 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Safety,
            tip_id: TipId::ActiveFiresCritical,
            message: format!("{} active fires in the city!", active_fires),
            priority: 5,
            suggestion: "Build fire stations to improve response times.".into(),
            tick_created: tick,
            location: find_first_fire(&extras.fire),
        });
    } else if active_fires > 0 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Safety,
            tip_id: TipId::ActiveFires,
            message: format!("{} active fire(s) reported.", active_fires),
            priority: 3,
            suggestion: "Ensure fire stations cover all neighborhoods.".into(),
            tick_created: tick,
            location: find_first_fire(&extras.fire),
        });
    }

    // Low police coverage (high crime areas with no police nearby)
    let high_crime_cells = extras.crime.levels.iter().filter(|&&v| v > 40).count();
    if high_crime_cells > 200 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Safety,
            tip_id: TipId::HighCrimeCells,
            message: format!("{} cells have high crime levels.", high_crime_cells),
            priority: 4,
            suggestion: "Expand police coverage to reduce crime hotspots.".into(),
            tick_created: tick,
            location: find_worst_cell(&extras.crime.levels, GRID_WIDTH),
        });
    }
}

// ---------------------------------------------------------------------------
// Environment advice
// ---------------------------------------------------------------------------

pub(crate) fn environment_advice(
    tick: u64,
    extras: &AdvisorExtras,
    msgs: &mut Vec<AdvisorMessage>,
) {
    let total = extras.pollution.levels.len() as f32;
    let pollution_sum: f32 = extras.pollution.levels.iter().map(|&v| v as f32).sum();
    let avg_pollution = if total > 0.0 {
        pollution_sum / total
    } else {
        0.0
    };

    if avg_pollution > 40.0 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Environment,
            tip_id: TipId::PollutionHigh,
            message: format!(
                "Average pollution level is {:.0}/255 -- too high!",
                avg_pollution
            ),
            priority: 4,
            suggestion: "Plant trees, add parks, and reduce industrial density.".into(),
            tick_created: tick,
            location: find_worst_cell(&extras.pollution.levels, GRID_WIDTH),
        });
    } else if avg_pollution > 20.0 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Environment,
            tip_id: TipId::PollutionRising,
            message: "Pollution is rising in parts of the city.".into(),
            priority: 2,
            suggestion: "Consider adding parks and green spaces to offset pollution.".into(),
            tick_created: tick,
            location: find_worst_cell(&extras.pollution.levels, GRID_WIDTH),
        });
    }

    // High pollution hotspots
    let hotspots = extras.pollution.levels.iter().filter(|&&v| v > 100).count();
    if hotspots > 50 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Environment,
            tip_id: TipId::PollutionHotspots,
            message: format!("{} severe pollution hotspots detected.", hotspots),
            priority: 4,
            suggestion: "Consider relocating heavy industry away from residential areas.".into(),
            tick_created: tick,
            location: find_worst_cell(&extras.pollution.levels, GRID_WIDTH),
        });
    }
}

// ---------------------------------------------------------------------------
// Housing advice
// ---------------------------------------------------------------------------

pub(crate) fn housing_advice(
    tick: u64,
    stats: &crate::stats::CityStats,
    extras: &AdvisorExtras,
    msgs: &mut Vec<AdvisorMessage>,
) {
    // Homelessness
    if extras.homeless.total_homeless > 50 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Housing,
            tip_id: TipId::HomelessCritical,
            message: format!(
                "{} citizens are homeless! ({} in shelters)",
                extras.homeless.total_homeless, extras.homeless.sheltered
            ),
            priority: 5,
            suggestion: "Zone more residential areas and build shelters.".into(),
            tick_created: tick,
            location: None,
        });
    } else if extras.homeless.total_homeless > 10 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Housing,
            tip_id: TipId::HomelessModerate,
            message: format!(
                "{} homeless citizens need housing.",
                extras.homeless.total_homeless
            ),
            priority: 3,
            suggestion: "Expand residential zones to meet housing demand.".into(),
            tick_created: tick,
            location: None,
        });
    }

    // High unemployment can indicate housing-jobs mismatch
    if extras.employment.unemployment_rate > 0.15 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Housing,
            tip_id: TipId::HighUnemployment,
            message: format!(
                "Unemployment at {:.1}% -- citizens may struggle to afford housing.",
                extras.employment.unemployment_rate * 100.0
            ),
            priority: 3,
            suggestion: "Zone more commercial and industrial areas to create jobs.".into(),
            tick_created: tick,
            location: None,
        });
    }

    // Low population but lots of residential -- city may be unattractive
    if stats.population > 0 && stats.population < 500 && stats.residential_buildings > 200 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Housing,
            tip_id: TipId::EmptyResidential,
            message: "Many residential buildings are empty.".into(),
            priority: 2,
            suggestion: "Improve city attractiveness with services and lower taxes.".into(),
            tick_created: tick,
            location: None,
        });
    }
}

// ---------------------------------------------------------------------------
// Traffic advice
// ---------------------------------------------------------------------------

pub(crate) fn traffic_advice(tick: u64, extras: &AdvisorExtras, msgs: &mut Vec<AdvisorMessage>) {
    // Find cells with high traffic congestion
    let mut congested_cells = 0u32;
    let mut worst_x = 0;
    let mut worst_y = 0;
    let mut worst_val: u16 = 0;

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let val = extras.traffic.get(x, y);
            if extras.traffic.congestion_level(x, y) > 0.7 {
                congested_cells += 1;
            }
            if val > worst_val {
                worst_val = val;
                worst_x = x;
                worst_y = y;
            }
        }
    }

    if congested_cells > 50 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Infrastructure,
            tip_id: TipId::TrafficCongestion,
            message: format!(
                "{} road cells experiencing heavy congestion!",
                congested_cells
            ),
            priority: 4,
            suggestion: "Upgrade roads, add alternative routes, or improve public transit.".into(),
            tick_created: tick,
            location: if worst_val > 0 {
                Some((worst_x, worst_y))
            } else {
                None
            },
        });
    }
}

// ---------------------------------------------------------------------------
// Zone demand advice
// ---------------------------------------------------------------------------

pub(crate) fn zone_demand_advice(
    tick: u64,
    extras: &AdvisorExtras,
    msgs: &mut Vec<AdvisorMessage>,
) {
    let demand = &extras.zone_demand;

    if demand.residential > 0.7 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Housing,
            tip_id: TipId::ZoneDemandResidential,
            message: format!(
                "High residential demand ({:.0}%)! Citizens want more housing.",
                demand.residential * 100.0
            ),
            priority: 3,
            suggestion: "Zone more residential areas to attract new citizens.".into(),
            tick_created: tick,
            location: None,
        });
    }

    if demand.commercial > 0.7 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Finance,
            tip_id: TipId::ZoneDemandCommercial,
            message: format!(
                "High commercial demand ({:.0}%)! Businesses want to expand.",
                demand.commercial * 100.0
            ),
            priority: 3,
            suggestion: "Zone more commercial areas to grow your tax base.".into(),
            tick_created: tick,
            location: None,
        });
    }

    if demand.industrial > 0.7 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Finance,
            tip_id: TipId::ZoneDemandIndustrial,
            message: format!(
                "High industrial demand ({:.0}%)! Factories need more space.",
                demand.industrial * 100.0
            ),
            priority: 3,
            suggestion: "Zone more industrial areas to create jobs and grow production.".into(),
            tick_created: tick,
            location: None,
        });
    }
}

// ---------------------------------------------------------------------------
// Fire coverage advice
// ---------------------------------------------------------------------------

pub(crate) fn fire_coverage_advice(
    tick: u64,
    grid: &WorldGrid,
    extras: &AdvisorExtras,
    msgs: &mut Vec<AdvisorMessage>,
) {
    // Count buildings and active fires as proxy for coverage gaps
    let active_fires = extras.fire.fire_levels.iter().filter(|&&v| v > 0).count();
    let building_count = grid
        .cells
        .iter()
        .filter(|c| c.building_id.is_some())
        .count();

    // If there are buildings and repeated fires, suggest fire coverage
    if building_count > 50 && active_fires > 3 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Safety,
            tip_id: TipId::FireCoverageGap,
            message: "Some areas lack fire station coverage.".into(),
            priority: 3,
            suggestion: "Build fire stations near residential and commercial areas.".into(),
            tick_created: tick,
            location: find_fire_coverage_gap(grid),
        });
    }
}
