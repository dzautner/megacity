use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::crime::CrimeGrid;
use crate::economy::CityBudget;
use crate::education::EducationGrid;
use crate::education_jobs::EmploymentStats;
use crate::fire::FireGrid;
use crate::grid::{CellType, WorldGrid, ZoneType};
use crate::health::HealthGrid;
use crate::homelessness::HomelessnessStats;
use crate::loans::LoanBook;
use crate::pollution::PollutionGrid;
use crate::road_maintenance::RoadMaintenanceStats;
use crate::stats::CityStats;
use crate::traffic::TrafficGrid;
use crate::zones::ZoneDemand;
use crate::TickCounter;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Categories of city advisors, each monitoring a different domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AdvisorType {
    Finance,
    Infrastructure,
    Health,
    Education,
    Safety,
    Environment,
    Housing,
}

impl AdvisorType {
    /// Display name for this advisor.
    pub fn name(self) -> &'static str {
        match self {
            AdvisorType::Finance => "Finance",
            AdvisorType::Infrastructure => "Infrastructure",
            AdvisorType::Health => "Health",
            AdvisorType::Education => "Education",
            AdvisorType::Safety => "Safety",
            AdvisorType::Environment => "Environment",
            AdvisorType::Housing => "Housing",
        }
    }

    /// Icon character for this advisor type (for the notification area).
    pub fn icon(self) -> &'static str {
        match self {
            AdvisorType::Finance => "$",
            AdvisorType::Infrastructure => "I",
            AdvisorType::Health => "+",
            AdvisorType::Education => "E",
            AdvisorType::Safety => "!",
            AdvisorType::Environment => "~",
            AdvisorType::Housing => "H",
        }
    }
}

/// Unique identifier for each distinct advisor tip type.
/// Used for permanent dismissal tracking -- when the player dismisses a tip,
/// its `TipId` is recorded so the same tip category never appears again.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    bitcode::Encode,
    bitcode::Decode,
)]
pub enum TipId {
    // Finance
    TreasuryCritical,
    TreasuryLow,
    BudgetDeficit,
    DebtCritical,
    DebtElevated,
    HighTaxUnhappy,
    LowTaxDeclining,

    // Infrastructure
    PowerCoverageCritical,
    PowerCoverageLow,
    WaterCoverageCritical,
    WaterCoverageLow,
    RoadsCritical,
    RoadsPoor,

    // Health
    HealthCoverageCritical,
    HealthCoverageLow,
    PollutionHealthRisk,

    // Education
    EducationVeryLow,
    EducationLow,
    UnemploymentEducation,

    // Safety
    CrimeCritical,
    CrimeRising,
    ActiveFiresCritical,
    ActiveFires,
    HighCrimeCells,

    // Environment
    PollutionHigh,
    PollutionRising,
    PollutionHotspots,

    // Housing
    HomelessCritical,
    HomelessModerate,
    HighUnemployment,
    EmptyResidential,

    // Traffic
    TrafficCongestion,
    RoadHierarchyViolation,

    // Zone demand
    ZoneDemandResidential,
    ZoneDemandCommercial,
    ZoneDemandIndustrial,

    // Fire coverage
    FireCoverageGap,
}

impl TipId {
    /// Human-readable label for this tip type (used in dismiss confirmation).
    pub fn label(self) -> &'static str {
        match self {
            TipId::TreasuryCritical => "Treasury Critical",
            TipId::TreasuryLow => "Treasury Low",
            TipId::BudgetDeficit => "Budget Deficit",
            TipId::DebtCritical => "Debt Critical",
            TipId::DebtElevated => "Debt Elevated",
            TipId::HighTaxUnhappy => "High Tax / Unhappy",
            TipId::LowTaxDeclining => "Low Tax / Declining",
            TipId::PowerCoverageCritical => "Power Coverage Critical",
            TipId::PowerCoverageLow => "Power Coverage Low",
            TipId::WaterCoverageCritical => "Water Coverage Critical",
            TipId::WaterCoverageLow => "Water Coverage Low",
            TipId::RoadsCritical => "Roads Critical",
            TipId::RoadsPoor => "Roads Poor",
            TipId::HealthCoverageCritical => "Health Coverage Critical",
            TipId::HealthCoverageLow => "Health Coverage Low",
            TipId::PollutionHealthRisk => "Pollution Health Risk",
            TipId::EducationVeryLow => "Education Very Low",
            TipId::EducationLow => "Education Low",
            TipId::UnemploymentEducation => "Unemployment + Education",
            TipId::CrimeCritical => "Crime Critical",
            TipId::CrimeRising => "Crime Rising",
            TipId::ActiveFiresCritical => "Active Fires Critical",
            TipId::ActiveFires => "Active Fires",
            TipId::HighCrimeCells => "High Crime Cells",
            TipId::PollutionHigh => "Pollution High",
            TipId::PollutionRising => "Pollution Rising",
            TipId::PollutionHotspots => "Pollution Hotspots",
            TipId::HomelessCritical => "Homeless Critical",
            TipId::HomelessModerate => "Homeless Moderate",
            TipId::HighUnemployment => "High Unemployment",
            TipId::EmptyResidential => "Empty Residential",
            TipId::TrafficCongestion => "Traffic Congestion",
            TipId::RoadHierarchyViolation => "Road Hierarchy Violation",
            TipId::ZoneDemandResidential => "Residential Demand",
            TipId::ZoneDemandCommercial => "Commercial Demand",
            TipId::ZoneDemandIndustrial => "Industrial Demand",
            TipId::FireCoverageGap => "Fire Coverage Gap",
        }
    }
}

/// A single advisor message displayed in the advisor panel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvisorMessage {
    pub advisor_type: AdvisorType,
    pub tip_id: TipId,
    pub message: String,
    pub priority: u8, // 1 (low) to 5 (critical)
    pub suggestion: String,
    pub tick_created: u64,
    /// Optional grid coordinates (col, row) for "Show Location" button.
    /// When present, the UI can jump the camera to this location.
    pub location: Option<(usize, usize)>,
}

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Maximum number of messages kept in the panel at any time.
const MAX_MESSAGES: usize = 10;

/// Messages expire after this many ticks.
const EXPIRY_TICKS: u64 = 500;

/// The advisor system runs every N ticks.
const ADVISOR_INTERVAL: u64 = 200;

/// Resource that holds the current set of advisor messages shown to the player.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct AdvisorPanel {
    pub messages: Vec<AdvisorMessage>,
}

impl AdvisorPanel {
    /// Remove expired messages and keep at most `MAX_MESSAGES`, sorted by priority descending.
    fn prune(&mut self, current_tick: u64) {
        self.messages
            .retain(|m| current_tick.saturating_sub(m.tick_created) < EXPIRY_TICKS);
        self.messages.sort_by(|a, b| b.priority.cmp(&a.priority));
        self.messages.truncate(MAX_MESSAGES);
    }

    /// Push a message, then prune.
    fn push(&mut self, msg: AdvisorMessage, current_tick: u64) {
        self.messages.push(msg);
        self.prune(current_tick);
    }
}

/// Tracks which tip types the player has permanently dismissed.
/// Dismissed tips will never appear again (until the player resets them).
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct DismissedAdvisorTips {
    pub dismissed: HashSet<TipId>,
}

impl DismissedAdvisorTips {
    /// Returns true if this tip type has been dismissed.
    pub fn is_dismissed(&self, tip_id: TipId) -> bool {
        self.dismissed.contains(&tip_id)
    }

    /// Dismiss a tip type permanently.
    pub fn dismiss(&mut self, tip_id: TipId) {
        self.dismissed.insert(tip_id);
    }

    /// Restore a previously dismissed tip type.
    pub fn restore(&mut self, tip_id: TipId) {
        self.dismissed.remove(&tip_id);
    }

    /// Restore all dismissed tips.
    pub fn restore_all(&mut self) {
        self.dismissed.clear();
    }
}

/// Event sent from the UI to request the camera jump to a world position.
/// The rendering crate listens for this event and moves the `OrbitCamera` focus.
#[derive(Event, Debug, Clone)]
pub struct AdvisorJumpToLocation {
    /// Grid coordinates (col, row) to jump to.
    pub grid_x: usize,
    pub grid_y: usize,
}

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Bundled secondary resources so the system stays within Bevy's parameter limits.
#[derive(bevy::ecs::system::SystemParam)]
pub struct AdvisorExtras<'w> {
    pub employment: Res<'w, EmploymentStats>,
    pub homeless: Res<'w, HomelessnessStats>,
    pub loan_book: Res<'w, LoanBook>,
    pub pollution: Res<'w, PollutionGrid>,
    pub education_grid: Res<'w, EducationGrid>,
    pub crime: Res<'w, CrimeGrid>,
    pub fire: Res<'w, FireGrid>,
    pub health: Res<'w, HealthGrid>,
    pub road_stats: Res<'w, RoadMaintenanceStats>,
    pub traffic: Res<'w, TrafficGrid>,
    pub zone_demand: Res<'w, ZoneDemand>,
}

/// Analyzes city state every 200 ticks and generates contextual advisor messages.
#[allow(clippy::too_many_arguments)]
pub fn update_advisors(
    tick: Res<TickCounter>,
    stats: Res<CityStats>,
    budget: Res<CityBudget>,
    grid: Res<WorldGrid>,
    mut panel: ResMut<AdvisorPanel>,
    extras: AdvisorExtras,
    dismissed: Res<DismissedAdvisorTips>,
) {
    let t = tick.0;
    if !t.is_multiple_of(ADVISOR_INTERVAL) {
        return;
    }

    // Clear stale messages first
    panel.prune(t);

    // Collect new messages into a local vec, then push all at once
    let mut new_msgs: Vec<AdvisorMessage> = Vec::new();

    // ------ Finance ------
    finance_advice(t, &stats, &budget, &extras, &mut new_msgs);

    // ------ Infrastructure ------
    infrastructure_advice(t, &grid, &extras, &mut new_msgs);

    // ------ Health ------
    health_advice(t, &extras, &mut new_msgs);

    // ------ Education ------
    education_advice(t, &extras, &mut new_msgs);

    // ------ Safety ------
    safety_advice(t, &grid, &extras, &mut new_msgs);

    // ------ Environment ------
    environment_advice(t, &extras, &mut new_msgs);

    // ------ Housing ------
    housing_advice(t, &stats, &extras, &mut new_msgs);

    // ------ Traffic ------
    traffic_advice(t, &extras, &mut new_msgs);

    // ------ Zone Demand ------
    zone_demand_advice(t, &extras, &mut new_msgs);

    // ------ Fire Coverage ------
    fire_coverage_advice(t, &grid, &extras, &mut new_msgs);

    // Filter out dismissed tips
    new_msgs.retain(|msg| !dismissed.is_dismissed(msg.tip_id));

    for msg in new_msgs {
        panel.push(msg, t);
    }
}

// ---------------------------------------------------------------------------
// Location-finding helpers
// ---------------------------------------------------------------------------

/// Find the grid cell with the worst value in a flat grid array (highest value = worst).
fn find_worst_cell(levels: &[u8], width: usize) -> Option<(usize, usize)> {
    if levels.is_empty() {
        return None;
    }
    let (max_idx, &max_val) = levels.iter().enumerate().max_by_key(|(_, &v)| v).unwrap();
    if max_val == 0 {
        return None;
    }
    let x = max_idx % width;
    let y = max_idx / width;
    Some((x, y))
}

/// Find the grid cell with the lowest value (worst coverage).
fn find_worst_coverage_cell(levels: &[u8], width: usize) -> Option<(usize, usize)> {
    if levels.is_empty() {
        return None;
    }
    let (min_idx, _) = levels.iter().enumerate().min_by_key(|(_, &v)| v).unwrap();
    let x = min_idx % width;
    let y = min_idx / width;
    Some((x, y))
}

/// Find the first zoned cell without power.
fn find_unpowered_zone(grid: &WorldGrid) -> Option<(usize, usize)> {
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
fn find_unwatered_zone(grid: &WorldGrid) -> Option<(usize, usize)> {
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
fn find_first_fire(fire: &FireGrid) -> Option<(usize, usize)> {
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
fn find_fire_coverage_gap(grid: &WorldGrid) -> Option<(usize, usize)> {
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
// Advisor logic helpers
// ---------------------------------------------------------------------------

fn finance_advice(
    tick: u64,
    stats: &CityStats,
    budget: &CityBudget,
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

fn infrastructure_advice(
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

fn health_advice(tick: u64, extras: &AdvisorExtras, msgs: &mut Vec<AdvisorMessage>) {
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

fn education_advice(tick: u64, extras: &AdvisorExtras, msgs: &mut Vec<AdvisorMessage>) {
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

fn safety_advice(
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

fn environment_advice(tick: u64, extras: &AdvisorExtras, msgs: &mut Vec<AdvisorMessage>) {
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

fn housing_advice(
    tick: u64,
    stats: &CityStats,
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

fn traffic_advice(tick: u64, extras: &AdvisorExtras, msgs: &mut Vec<AdvisorMessage>) {
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

fn zone_demand_advice(tick: u64, extras: &AdvisorExtras, msgs: &mut Vec<AdvisorMessage>) {
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

fn fire_coverage_advice(
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

// ---------------------------------------------------------------------------
// Saveable implementation for DismissedAdvisorTips
// ---------------------------------------------------------------------------

impl crate::Saveable for DismissedAdvisorTips {
    const SAVE_KEY: &'static str = "dismissed_advisor_tips";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.dismissed.is_empty() {
            return None;
        }
        Some(bitcode::encode(
            &self.dismissed.iter().copied().collect::<Vec<_>>(),
        ))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        let tips: Vec<TipId> = crate::decode_or_warn(Self::SAVE_KEY, bytes);
        DismissedAdvisorTips {
            dismissed: tips.into_iter().collect(),
        }
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct AdvisorsPlugin;

impl Plugin for AdvisorsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<crate::SaveableRegistry>()
            .init_resource::<AdvisorPanel>()
            .init_resource::<DismissedAdvisorTips>()
            .add_event::<AdvisorJumpToLocation>()
            .add_systems(
                FixedUpdate,
                update_advisors
                    .after(crate::stats::update_stats)
                    .in_set(crate::SimulationSet::PostSim),
            );

        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<DismissedAdvisorTips>();
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_advisor_panel_prune_removes_expired() {
        let mut panel = AdvisorPanel::default();
        panel.messages.push(AdvisorMessage {
            advisor_type: AdvisorType::Finance,
            tip_id: TipId::TreasuryCritical,
            message: "Old message".into(),
            priority: 3,
            suggestion: "Do something".into(),
            tick_created: 0,
            location: None,
        });
        panel.messages.push(AdvisorMessage {
            advisor_type: AdvisorType::Health,
            tip_id: TipId::HealthCoverageCritical,
            message: "New message".into(),
            priority: 4,
            suggestion: "Do something else".into(),
            tick_created: 600,
            location: None,
        });

        panel.prune(600);
        assert_eq!(panel.messages.len(), 1);
        assert_eq!(panel.messages[0].advisor_type, AdvisorType::Health);
    }

    #[test]
    fn test_advisor_panel_prune_sorts_by_priority() {
        let mut panel = AdvisorPanel::default();
        panel.messages.push(AdvisorMessage {
            advisor_type: AdvisorType::Education,
            tip_id: TipId::EducationLow,
            message: "Low priority".into(),
            priority: 1,
            suggestion: "".into(),
            tick_created: 100,
            location: None,
        });
        panel.messages.push(AdvisorMessage {
            advisor_type: AdvisorType::Finance,
            tip_id: TipId::TreasuryCritical,
            message: "High priority".into(),
            priority: 5,
            suggestion: "".into(),
            tick_created: 100,
            location: None,
        });
        panel.messages.push(AdvisorMessage {
            advisor_type: AdvisorType::Safety,
            tip_id: TipId::CrimeRising,
            message: "Medium priority".into(),
            priority: 3,
            suggestion: "".into(),
            tick_created: 100,
            location: None,
        });

        panel.prune(200);
        assert_eq!(panel.messages.len(), 3);
        assert_eq!(panel.messages[0].priority, 5);
        assert_eq!(panel.messages[1].priority, 3);
        assert_eq!(panel.messages[2].priority, 1);
    }

    #[test]
    fn test_advisor_panel_truncates_to_max() {
        let mut panel = AdvisorPanel::default();
        for i in 0..15 {
            panel.messages.push(AdvisorMessage {
                advisor_type: AdvisorType::Finance,
                tip_id: TipId::TreasuryCritical,
                message: format!("Message {}", i),
                priority: (i % 5 + 1) as u8,
                suggestion: "".into(),
                tick_created: 100,
                location: None,
            });
        }
        panel.prune(200);
        assert_eq!(panel.messages.len(), MAX_MESSAGES);
    }

    #[test]
    fn test_advisor_panel_push() {
        let mut panel = AdvisorPanel::default();
        let msg = AdvisorMessage {
            advisor_type: AdvisorType::Housing,
            tip_id: TipId::HomelessCritical,
            message: "Test".into(),
            priority: 4,
            suggestion: "Build more".into(),
            tick_created: 50,
            location: None,
        };
        panel.push(msg, 50);
        assert_eq!(panel.messages.len(), 1);
        assert_eq!(panel.messages[0].advisor_type, AdvisorType::Housing);
    }

    #[test]
    fn test_advisor_type_name() {
        assert_eq!(AdvisorType::Finance.name(), "Finance");
        assert_eq!(AdvisorType::Infrastructure.name(), "Infrastructure");
        assert_eq!(AdvisorType::Health.name(), "Health");
        assert_eq!(AdvisorType::Education.name(), "Education");
        assert_eq!(AdvisorType::Safety.name(), "Safety");
        assert_eq!(AdvisorType::Environment.name(), "Environment");
        assert_eq!(AdvisorType::Housing.name(), "Housing");
    }

    #[test]
    fn test_advisor_panel_default_is_empty() {
        let panel = AdvisorPanel::default();
        assert!(panel.messages.is_empty());
    }

    #[test]
    fn test_dismissed_tips_dismiss_and_restore() {
        let mut dismissed = DismissedAdvisorTips::default();
        assert!(!dismissed.is_dismissed(TipId::BudgetDeficit));

        dismissed.dismiss(TipId::BudgetDeficit);
        assert!(dismissed.is_dismissed(TipId::BudgetDeficit));

        dismissed.restore(TipId::BudgetDeficit);
        assert!(!dismissed.is_dismissed(TipId::BudgetDeficit));
    }

    #[test]
    fn test_dismissed_tips_restore_all() {
        let mut dismissed = DismissedAdvisorTips::default();
        dismissed.dismiss(TipId::BudgetDeficit);
        dismissed.dismiss(TipId::CrimeCritical);
        dismissed.dismiss(TipId::TrafficCongestion);
        assert_eq!(dismissed.dismissed.len(), 3);

        dismissed.restore_all();
        assert!(dismissed.dismissed.is_empty());
    }

    #[test]
    fn test_tip_id_labels() {
        let all_tips = [
            TipId::TreasuryCritical,
            TipId::BudgetDeficit,
            TipId::TrafficCongestion,
            TipId::FireCoverageGap,
            TipId::ZoneDemandResidential,
        ];
        for tip in all_tips {
            assert!(!tip.label().is_empty());
        }
    }

    #[test]
    fn test_advisor_message_with_location() {
        let msg = AdvisorMessage {
            advisor_type: AdvisorType::Safety,
            tip_id: TipId::ActiveFires,
            message: "Fire detected".into(),
            priority: 5,
            suggestion: "Build fire station".into(),
            tick_created: 100,
            location: Some((42, 87)),
        };
        assert_eq!(msg.location, Some((42, 87)));
    }

    #[test]
    fn test_find_worst_cell() {
        let mut levels = vec![0u8; 256 * 256];
        levels[100 * 256 + 50] = 200; // x=50, y=100
        let result = find_worst_cell(&levels, 256);
        assert_eq!(result, Some((50, 100)));
    }

    #[test]
    fn test_find_worst_cell_all_zero() {
        let levels = vec![0u8; 256 * 256];
        let result = find_worst_cell(&levels, 256);
        assert_eq!(result, None);
    }

    #[test]
    fn test_advisor_jump_event() {
        let event = AdvisorJumpToLocation {
            grid_x: 128,
            grid_y: 64,
        };
        assert_eq!(event.grid_x, 128);
        assert_eq!(event.grid_y, 64);
    }
}
