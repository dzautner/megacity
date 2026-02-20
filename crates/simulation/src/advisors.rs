use bevy::prelude::*;
use serde::{Deserialize, Serialize};

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
}

/// A single advisor message displayed in the advisor panel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvisorMessage {
    pub advisor_type: AdvisorType,
    pub message: String,
    pub priority: u8, // 1 (low) to 5 (critical)
    pub suggestion: String,
    pub tick_created: u64,
}

// ---------------------------------------------------------------------------
// Resource
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
}

/// Analyzes city state every 200 ticks and generates contextual advisor messages.
pub fn update_advisors(
    tick: Res<TickCounter>,
    stats: Res<CityStats>,
    budget: Res<CityBudget>,
    grid: Res<WorldGrid>,
    mut panel: ResMut<AdvisorPanel>,
    extras: AdvisorExtras,
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
    safety_advice(t, &extras, &mut new_msgs);

    // ------ Environment ------
    environment_advice(t, &extras, &mut new_msgs);

    // ------ Housing ------
    housing_advice(t, &stats, &extras, &mut new_msgs);

    for msg in new_msgs {
        panel.push(msg, t);
    }
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
            message: format!("Treasury critically low: ${:.0}", budget.treasury),
            priority: 5,
            suggestion: "Consider raising taxes or taking a loan to cover expenses.".into(),
            tick_created: tick,
        });
    } else if budget.treasury < 10_000.0 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Finance,
            message: format!("Treasury running low: ${:.0}", budget.treasury),
            priority: 3,
            suggestion: "Watch spending and consider growing your tax base.".into(),
            tick_created: tick,
        });
    }

    // Negative cash flow
    if budget.monthly_income > 0.0 && budget.monthly_expenses > budget.monthly_income {
        let deficit = budget.monthly_expenses - budget.monthly_income;
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Finance,
            message: format!("Running a deficit of ${:.0}/month!", deficit),
            priority: 4,
            suggestion: "Reduce service budgets or increase revenue sources.".into(),
            tick_created: tick,
        });
    }

    // High debt-to-income
    let dti = extras.loan_book.debt_to_income(budget.monthly_income);
    if dti.is_finite() && dti > 5.0 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Finance,
            message: format!("Debt-to-income ratio is dangerously high: {:.1}x", dti),
            priority: 5,
            suggestion: "Avoid taking new loans and focus on paying down debt.".into(),
            tick_created: tick,
        });
    } else if dti.is_finite() && dti > 2.0 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Finance,
            message: format!("Debt-to-income ratio is elevated: {:.1}x", dti),
            priority: 3,
            suggestion: "Consider limiting new borrowing.".into(),
            tick_created: tick,
        });
    }

    // Tax rate suggestions
    if budget.tax_rate > 0.15 && stats.average_happiness < 50.0 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Finance,
            message: "High tax rates are hurting citizen happiness.".into(),
            priority: 3,
            suggestion: "Consider lowering the tax rate to improve morale.".into(),
            tick_created: tick,
        });
    }

    if stats.population > 0 && budget.tax_rate < 0.05 && budget.treasury < 20_000.0 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Finance,
            message: "Tax rate is very low and treasury is declining.".into(),
            priority: 2,
            suggestion: "A modest tax increase could stabilize the budget.".into(),
            tick_created: tick,
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
                message: format!("Only {:.0}% of zones have power!", power_cov * 100.0),
                priority: 5,
                suggestion: "Build more power plants to expand coverage.".into(),
                tick_created: tick,
            });
        } else if power_cov < 0.8 {
            msgs.push(AdvisorMessage {
                advisor_type: AdvisorType::Infrastructure,
                message: format!(
                    "Power coverage at {:.0}% -- some areas are dark.",
                    power_cov * 100.0
                ),
                priority: 3,
                suggestion: "Consider adding a power plant near underserved areas.".into(),
                tick_created: tick,
            });
        }

        if water_cov < 0.5 {
            msgs.push(AdvisorMessage {
                advisor_type: AdvisorType::Infrastructure,
                message: format!("Only {:.0}% of zones have water!", water_cov * 100.0),
                priority: 5,
                suggestion: "Build more water towers to serve the population.".into(),
                tick_created: tick,
            });
        } else if water_cov < 0.8 {
            msgs.push(AdvisorMessage {
                advisor_type: AdvisorType::Infrastructure,
                message: format!("Water coverage at {:.0}%.", water_cov * 100.0),
                priority: 3,
                suggestion: "Add water towers to improve coverage.".into(),
                tick_created: tick,
            });
        }
    }

    // Road maintenance warnings
    if extras.road_stats.critical_roads_count > 20 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Infrastructure,
            message: format!(
                "{} road segments in critical condition!",
                extras.road_stats.critical_roads_count
            ),
            priority: 4,
            suggestion: "Increase road maintenance budget to prevent further deterioration.".into(),
            tick_created: tick,
        });
    } else if extras.road_stats.poor_roads_count > 100 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Infrastructure,
            message: format!(
                "{} roads in poor condition.",
                extras.road_stats.poor_roads_count
            ),
            priority: 2,
            suggestion: "Boost road maintenance budget to keep traffic flowing smoothly.".into(),
            tick_created: tick,
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
            message: "Health coverage is critically low across the city.".into(),
            priority: 5,
            suggestion: "Build hospitals to improve healthcare access.".into(),
            tick_created: tick,
        });
    } else if avg_health < 80.0 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Health,
            message: "Some areas lack adequate health coverage.".into(),
            priority: 3,
            suggestion: "Consider building hospitals in underserved neighborhoods.".into(),
            tick_created: tick,
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
            message: "High pollution levels are creating health risks.".into(),
            priority: 4,
            suggestion: "Reduce industrial density and plant trees to lower pollution.".into(),
            tick_created: tick,
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
            message: "Education coverage is very low.".into(),
            priority: 4,
            suggestion: "Build elementary schools and high schools across the city.".into(),
            tick_created: tick,
        });
    } else if avg_edu < 1.5 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Education,
            message: "Many areas lack education facilities.".into(),
            priority: 3,
            suggestion: "Expand education coverage with schools and libraries.".into(),
            tick_created: tick,
        });
    }

    // High unemployment + low education
    if extras.employment.unemployment_rate > 0.10 && avg_edu < 1.0 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Education,
            message: "Unemployment is high and workforce education is lacking.".into(),
            priority: 4,
            suggestion: "Invest in education to improve worker qualifications.".into(),
            tick_created: tick,
        });
    }
}

fn safety_advice(tick: u64, extras: &AdvisorExtras, msgs: &mut Vec<AdvisorMessage>) {
    // Average crime level
    let total = extras.crime.levels.len() as f32;
    let crime_sum: f32 = extras.crime.levels.iter().map(|&v| v as f32).sum();
    let avg_crime = if total > 0.0 { crime_sum / total } else { 0.0 };

    if avg_crime > 50.0 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Safety,
            message: "Crime rates are dangerously high!".into(),
            priority: 5,
            suggestion: "Build police stations in high-crime areas and increase police budget."
                .into(),
            tick_created: tick,
        });
    } else if avg_crime > 20.0 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Safety,
            message: "Crime is on the rise in some neighborhoods.".into(),
            priority: 3,
            suggestion: "Consider adding police stations to affected areas.".into(),
            tick_created: tick,
        });
    }

    // Count active fires
    let active_fires = extras.fire.fire_levels.iter().filter(|&&v| v > 0).count();
    if active_fires > 10 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Safety,
            message: format!("{} active fires in the city!", active_fires),
            priority: 5,
            suggestion: "Build fire stations to improve response times.".into(),
            tick_created: tick,
        });
    } else if active_fires > 0 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Safety,
            message: format!("{} active fire(s) reported.", active_fires),
            priority: 3,
            suggestion: "Ensure fire stations cover all neighborhoods.".into(),
            tick_created: tick,
        });
    }

    // Low police coverage (high crime areas with no police nearby)
    let high_crime_cells = extras.crime.levels.iter().filter(|&&v| v > 40).count();
    if high_crime_cells > 200 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Safety,
            message: format!("{} cells have high crime levels.", high_crime_cells),
            priority: 4,
            suggestion: "Expand police coverage to reduce crime hotspots.".into(),
            tick_created: tick,
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
            message: format!(
                "Average pollution level is {:.0}/255 -- too high!",
                avg_pollution
            ),
            priority: 4,
            suggestion: "Plant trees, add parks, and reduce industrial density.".into(),
            tick_created: tick,
        });
    } else if avg_pollution > 20.0 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Environment,
            message: "Pollution is rising in parts of the city.".into(),
            priority: 2,
            suggestion: "Consider adding parks and green spaces to offset pollution.".into(),
            tick_created: tick,
        });
    }

    // High pollution hotspots
    let hotspots = extras.pollution.levels.iter().filter(|&&v| v > 100).count();
    if hotspots > 50 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Environment,
            message: format!("{} severe pollution hotspots detected.", hotspots),
            priority: 4,
            suggestion: "Consider relocating heavy industry away from residential areas.".into(),
            tick_created: tick,
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
            message: format!(
                "{} citizens are homeless! ({} in shelters)",
                extras.homeless.total_homeless, extras.homeless.sheltered
            ),
            priority: 5,
            suggestion: "Zone more residential areas and build shelters.".into(),
            tick_created: tick,
        });
    } else if extras.homeless.total_homeless > 10 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Housing,
            message: format!(
                "{} homeless citizens need housing.",
                extras.homeless.total_homeless
            ),
            priority: 3,
            suggestion: "Expand residential zones to meet housing demand.".into(),
            tick_created: tick,
        });
    }

    // High unemployment can indicate housing-jobs mismatch
    if extras.employment.unemployment_rate > 0.15 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Housing,
            message: format!(
                "Unemployment at {:.1}% -- citizens may struggle to afford housing.",
                extras.employment.unemployment_rate * 100.0
            ),
            priority: 3,
            suggestion: "Zone more commercial and industrial areas to create jobs.".into(),
            tick_created: tick,
        });
    }

    // Low population but lots of residential -- city may be unattractive
    if stats.population > 0 && stats.population < 500 && stats.residential_buildings > 200 {
        msgs.push(AdvisorMessage {
            advisor_type: AdvisorType::Housing,
            message: "Many residential buildings are empty.".into(),
            priority: 2,
            suggestion: "Improve city attractiveness with services and lower taxes.".into(),
            tick_created: tick,
        });
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
            message: "Old message".into(),
            priority: 3,
            suggestion: "Do something".into(),
            tick_created: 0,
        });
        panel.messages.push(AdvisorMessage {
            advisor_type: AdvisorType::Health,
            message: "New message".into(),
            priority: 4,
            suggestion: "Do something else".into(),
            tick_created: 600,
        });

        panel.prune(600);
        // The first message was created at tick 0, current tick 600 => 600 >= EXPIRY_TICKS(500)
        assert_eq!(panel.messages.len(), 1);
        assert_eq!(panel.messages[0].advisor_type, AdvisorType::Health);
    }

    #[test]
    fn test_advisor_panel_prune_sorts_by_priority() {
        let mut panel = AdvisorPanel::default();
        panel.messages.push(AdvisorMessage {
            advisor_type: AdvisorType::Education,
            message: "Low priority".into(),
            priority: 1,
            suggestion: "".into(),
            tick_created: 100,
        });
        panel.messages.push(AdvisorMessage {
            advisor_type: AdvisorType::Finance,
            message: "High priority".into(),
            priority: 5,
            suggestion: "".into(),
            tick_created: 100,
        });
        panel.messages.push(AdvisorMessage {
            advisor_type: AdvisorType::Safety,
            message: "Medium priority".into(),
            priority: 3,
            suggestion: "".into(),
            tick_created: 100,
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
                message: format!("Message {}", i),
                priority: (i % 5 + 1) as u8,
                suggestion: "".into(),
                tick_created: 100,
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
            message: "Test".into(),
            priority: 4,
            suggestion: "Build more".into(),
            tick_created: 50,
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
}

pub struct AdvisorsPlugin;

impl Plugin for AdvisorsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AdvisorPanel>()
            .add_systems(
                FixedUpdate,
                update_advisors.after(crate::stats::update_stats),
            );
    }
}
