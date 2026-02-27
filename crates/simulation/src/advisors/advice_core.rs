//! Advisor logic for Finance, Infrastructure, Health, and Education domains,
//! plus grid-search helpers used by all advice functions.

use crate::config::GRID_WIDTH;
use crate::fire::FireGrid;
use crate::grid::{CellType, WorldGrid, ZoneType};

use super::types::{AdvisorExtras, AdvisorMessage, AdvisorType, TipId};

// ---------------------------------------------------------------------------
// Location-finding helpers
// ---------------------------------------------------------------------------

/// Find the grid cell with the worst value in a flat grid array (highest value = worst).
pub(crate) fn find_worst_cell(levels: &[u8], width: usize) -> Option<(usize, usize)> {
    if levels.is_empty() {
        return None;
    }
    let Some((max_idx, &max_val)) = levels.iter().enumerate().max_by_key(|(_, &v)| v) else { return None; };
    if max_val == 0 {
        return None;
    }
    let x = max_idx % width;
    let y = max_idx / width;
    Some((x, y))
}

/// Find the grid cell with the lowest value (worst coverage).
pub(crate) fn find_worst_coverage_cell(levels: &[u8], width: usize) -> Option<(usize, usize)> {
    if levels.is_empty() {
        return None;
    }
    let Some((min_idx, _)) = levels.iter().enumerate().min_by_key(|(_, &v)| v) else { return None; };
    let x = min_idx % width;
    let y = min_idx / width;
    Some((x, y))
}

/// Find the first zoned cell without power.
pub(crate) fn find_unpowered_zone(grid: &WorldGrid) -> Option<(usize, usize)> {
    for (i, cell) in grid.cells.iter().enumerate() {
        if cell.cell_type == CellType::Grass && cell.zone != ZoneType::None && !cell.has_power {
            let x = i % GRID_WIDTH;
            let y = i / GRID_WIDTH;
            return Some((x, y));
        }
    }
    None
}

/// Find the first zoned cell without water.
pub(crate) fn find_unwatered_zone(grid: &WorldGrid) -> Option<(usize, usize)> {
    for (i, cell) in grid.cells.iter().enumerate() {
        if cell.cell_type == CellType::Grass && cell.zone != ZoneType::None && !cell.has_water {
            let x = i % GRID_WIDTH;
            let y = i / GRID_WIDTH;
            return Some((x, y));
        }
    }
    None
}

/// Find the first active fire cell.
pub(crate) fn find_first_fire(fire: &FireGrid) -> Option<(usize, usize)> {
    for (i, &level) in fire.fire_levels.iter().enumerate() {
        if level > 0 {
            let x = i % GRID_WIDTH;
            let y = i / GRID_WIDTH;
            return Some((x, y));
        }
    }
    None
}

/// Find a zoned building cell (proxy for fire coverage gap location).
pub(crate) fn find_fire_coverage_gap(grid: &WorldGrid) -> Option<(usize, usize)> {
    for (i, cell) in grid.cells.iter().enumerate() {
        if cell.building_id.is_some() && cell.zone != ZoneType::None {
            let x = i % GRID_WIDTH;
            let y = i / GRID_WIDTH;
            return Some((x, y));
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Finance advice
// ---------------------------------------------------------------------------

pub(crate) fn finance_advice(
    tick: u64,
    stats: &crate::stats::CityStats,
    budget: &crate::economy::CityBudget,
    extras: &AdvisorExtras,
    msgs: &mut Vec<AdvisorMessage>,
) {
    // Low treasury warning
    if budget.treasury < 1000.0 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Finance,
            tip_id: TipId::TreasuryCritical,
            message: format!("Treasury critically low: ${:.0}", budget.treasury),
            priority: 5,
            suggestion: "Consider raising taxes or taking a loan to cover expenses.".into(),
            tick_created: tick,
            location: None,
        });
    } else if budget.treasury < 10_000.0 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Finance,
            tip_id: TipId::TreasuryLow,
            message: format!("Treasury running low: ${:.0}", budget.treasury),
            priority: 3,
            suggestion: "Watch spending and consider growing your tax base.".into(),
            tick_created: tick,
            location: None,
        });
    }

    // Negative cash flow
    if budget.monthly_income > 0.0 && budget.monthly_expenses > budget.monthly_income {
        let deficit = budget.monthly_expenses - budget.monthly_income;
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Finance,
            tip_id: TipId::BudgetDeficit,
            message: format!("Running a deficit of ${:.0}/month!", deficit),
            priority: 4,
            suggestion: "Reduce service budgets or increase revenue sources.".into(),
            tick_created: tick,
            location: None,
        });
    }

    // High debt-to-income
    let dti = extras.loan_book.debt_to_income(budget.monthly_income);
    if dti.is_finite() && dti > 5.0 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Finance,
            tip_id: TipId::DebtCritical,
            message: format!("Debt-to-income ratio is dangerously high: {:.1}x", dti),
            priority: 5,
            suggestion: "Avoid taking new loans and focus on paying down debt.".into(),
            tick_created: tick,
            location: None,
        });
    } else if dti.is_finite() && dti > 2.0 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Finance,
            tip_id: TipId::DebtElevated,
            message: format!("Debt-to-income ratio is elevated: {:.1}x", dti),
            priority: 3,
            suggestion: "Consider limiting new borrowing.".into(),
            tick_created: tick,
            location: None,
        });
    }

    // Tax rate suggestions
    if budget.tax_rate > 0.15 && stats.average_happiness < 50.0 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Finance,
            tip_id: TipId::HighTaxUnhappy,
            message: "High tax rates are hurting citizen happiness.".into(),
            priority: 3,
            suggestion: "Consider lowering the tax rate to improve morale.".into(),
            tick_created: tick,
            location: None,
        });
    }

    if stats.population > 0 && budget.tax_rate < 0.05 && budget.treasury < 20_000.0 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Finance,
            tip_id: TipId::LowTaxDeclining,
            message: "Tax rate is very low and treasury is declining.".into(),
            priority: 2,
            suggestion: "A modest tax increase could stabilize the budget.".into(),
            tick_created: tick,
            location: None,
        });
    }
}

// ---------------------------------------------------------------------------
// Infrastructure advice
// ---------------------------------------------------------------------------

pub(crate) fn infrastructure_advice(
    tick: u64,
    grid: &WorldGrid,
    extras: &AdvisorExtras,
    msgs: &mut Vec<AdvisorMessage>,
) {
    // Compute power and water coverage over zoned cells
    let mut total_zoned = 0u32;
    let mut powered = 0u32;
    let mut watered = 0u32;
    for cell in &grid.cells {
        if cell.cell_type == CellType::Grass && cell.zone != ZoneType::None {
            total_zoned += 1;
            if cell.has_power {
                powered += 1;
            }
            if cell.has_water {
                watered += 1;
            }
        }
    }

    if total_zoned > 0 {
        let power_cov = powered as f32 / total_zoned as f32;
        let water_cov = watered as f32 / total_zoned as f32;

        if power_cov < 0.5 {
            msgs.push(AdvisorMessage {
                advisor_type: AdvisorType::Infrastructure,
                tip_id: TipId::PowerCoverageCritical,
                message: format!("Only {:.0}% of zones have power!", power_cov * 100.0),
                priority: 5,
                suggestion: "Build more power plants to expand coverage.".into(),
                tick_created: tick,
                location: find_unpowered_zone(grid),
            });
        } else if power_cov < 0.8 {
            msgs.push(AdvisorMessage {
                advisor_type: AdvisorType::Infrastructure,
                tip_id: TipId::PowerCoverageLow,
                message: format!(
                    "Power coverage at {:.0}% -- some areas are dark.",
                    power_cov * 100.0
                ),
                priority: 3,
                suggestion: "Consider adding a power plant near underserved areas.".into(),
                tick_created: tick,
                location: find_unpowered_zone(grid),
            });
        }

        if water_cov < 0.5 {
            msgs.push(AdvisorMessage {
                advisor_type: AdvisorType::Infrastructure,
                tip_id: TipId::WaterCoverageCritical,
                message: format!("Only {:.0}% of zones have water!", water_cov * 100.0),
                priority: 5,
                suggestion: "Build more water towers to serve the population.".into(),
                tick_created: tick,
                location: find_unwatered_zone(grid),
            });
        } else if water_cov < 0.8 {
            msgs.push(AdvisorMessage {
                advisor_type: AdvisorType::Infrastructure,
                tip_id: TipId::WaterCoverageLow,
                message: format!("Water coverage at {:.0}%.", water_cov * 100.0),
                priority: 3,
                suggestion: "Add water towers to improve coverage.".into(),
                tick_created: tick,
                location: find_unwatered_zone(grid),
            });
        }
    }

    // Road maintenance warnings
    if extras.road_stats.critical_roads_count > 20 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Infrastructure,
            tip_id: TipId::RoadsCritical,
            message: format!(
                "{} road segments in critical condition!",
                extras.road_stats.critical_roads_count
            ),
            priority: 4,
            suggestion: "Increase road maintenance budget to prevent further deterioration.".into(),
            tick_created: tick,
            location: None,
        });
    } else if extras.road_stats.poor_roads_count > 100 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Infrastructure,
            tip_id: TipId::RoadsPoor,
            message: format!(
                "{} roads in poor condition.",
                extras.road_stats.poor_roads_count
            ),
            priority: 2,
            suggestion: "Boost road maintenance budget to keep traffic flowing smoothly.".into(),
            tick_created: tick,
            location: None,
        });
    }
}

// ---------------------------------------------------------------------------
// Health advice
// ---------------------------------------------------------------------------

pub(crate) fn health_advice(tick: u64, extras: &AdvisorExtras, msgs: &mut Vec<AdvisorMessage>) {
    // Compute average health coverage
    let total = extras.health.levels.len() as f32;
    let sum: f32 = extras.health.levels.iter().map(|&v| v as f32).sum();
    let avg_health = if total > 0.0 { sum / total } else { 0.0 };

    if avg_health < 30.0 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Health,
            tip_id: TipId::HealthCoverageCritical,
            message: "Health coverage is critically low across the city.".into(),
            priority: 5,
            suggestion: "Build hospitals to improve healthcare access.".into(),
            tick_created: tick,
            location: find_worst_coverage_cell(&extras.health.levels, GRID_WIDTH),
        });
    } else if avg_health < 80.0 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Health,
            tip_id: TipId::HealthCoverageLow,
            message: "Some areas lack adequate health coverage.".into(),
            priority: 3,
            suggestion: "Consider building hospitals in underserved neighborhoods.".into(),
            tick_created: tick,
            location: find_worst_coverage_cell(&extras.health.levels, GRID_WIDTH),
        });
    }

    // High pollution areas affect health
    let pollution_sum: f32 = extras.pollution.levels.iter().map(|&v| v as f32).sum();
    let avg_pollution = if total > 0.0 {
        pollution_sum / total
    } else {
        0.0
    };
    if avg_pollution > 60.0 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Health,
            tip_id: TipId::PollutionHealthRisk,
            message: "High pollution levels are creating health risks.".into(),
            priority: 4,
            suggestion: "Reduce industrial density and plant trees to lower pollution.".into(),
            tick_created: tick,
            location: find_worst_cell(&extras.pollution.levels, GRID_WIDTH),
        });
    }
}

// ---------------------------------------------------------------------------
// Education advice
// ---------------------------------------------------------------------------

pub(crate) fn education_advice(tick: u64, extras: &AdvisorExtras, msgs: &mut Vec<AdvisorMessage>) {
    // Compute average education level across the grid
    let total = extras.education_grid.levels.len() as f32;
    let sum: f32 = extras.education_grid.levels.iter().map(|&v| v as f32).sum();
    let avg_edu = if total > 0.0 { sum / total } else { 0.0 };

    if avg_edu < 0.5 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Education,
            tip_id: TipId::EducationVeryLow,
            message: "Education coverage is very low.".into(),
            priority: 4,
            suggestion: "Build elementary schools and high schools across the city.".into(),
            tick_created: tick,
            location: None,
        });
    } else if avg_edu < 1.5 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Education,
            tip_id: TipId::EducationLow,
            message: "Many areas lack education facilities.".into(),
            priority: 3,
            suggestion: "Expand education coverage with schools and libraries.".into(),
            tick_created: tick,
            location: None,
        });
    }

    // High unemployment + low education
    if extras.employment.unemployment_rate > 0.10 && avg_edu < 1.0 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Education,
            tip_id: TipId::UnemploymentEducation,
            message: "Unemployment is high and workforce education is lacking.".into(),
            priority: 4,
            suggestion: "Invest in education to improve worker qualifications.".into(),
            tick_created: tick,
            location: None,
        });
    }
}
