//! Historical data collection for the charts panel (UX-046).
//!
//! Collects snapshots of population (with R/C/I worker breakdown), budget
//! income/expense breakdown, hourly traffic congestion, service coverage
//! percentages, and happiness factor breakdown.
//!
//! Data is recorded every 10 game-days and persisted via the Saveable trait.

use bevy::prelude::*;
use bitcode::{Decode, Encode};

use crate::budget::ExtendedBudget;
use crate::citizen::{Citizen, CitizenDetails, WorkLocation};
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::economy::CityBudget;
use crate::grid::{CellType, WorldGrid, ZoneType};
use crate::happiness::ServiceCoverageGrid;
use crate::stats::CityStats;
use crate::time_of_day::GameClock;
use crate::traffic::TrafficGrid;

/// Maximum number of history snapshots stored.
const MAX_SNAPSHOTS: usize = 360;

// -----------------------------------------------------------------------
// Data structures
// -----------------------------------------------------------------------

/// A single snapshot of population data.
#[derive(Debug, Clone, Default, Encode, Decode)]
pub struct PopulationSnapshot {
    pub total: u32,
    pub residential_workers: u32,
    pub commercial_workers: u32,
    pub industrial_workers: u32,
}

/// A single snapshot of budget data.
#[derive(Debug, Clone, Default, Encode, Decode)]
pub struct BudgetSnapshot {
    // Income sources
    pub residential_tax: f64,
    pub commercial_tax: f64,
    pub industrial_tax: f64,
    pub office_tax: f64,
    pub trade_income: f64,
    // Expense categories
    pub road_maintenance: f64,
    pub service_costs: f64,
    pub policy_costs: f64,
    pub loan_payments: f64,
}

/// Hourly traffic congestion (24 hours).
#[derive(Debug, Clone, Encode, Decode)]
pub struct TrafficByHour {
    /// Average congestion level per hour (0.0-1.0), indexed 0..24.
    pub congestion: [f32; 24],
}

impl Default for TrafficByHour {
    fn default() -> Self {
        Self {
            congestion: [0.0; 24],
        }
    }
}

/// Service coverage percentages (fraction of occupied cells covered).
#[derive(Debug, Clone, Default, Encode, Decode)]
pub struct ServiceCoverageSnapshot {
    pub health: f32,
    pub education: f32,
    pub police: f32,
    pub fire: f32,
    pub parks: f32,
    pub entertainment: f32,
    pub telecom: f32,
    pub transport: f32,
}

/// Happiness factor breakdown (average contribution per citizen).
#[derive(Debug, Clone, Default, Encode, Decode)]
pub struct HappinessBreakdown {
    pub base: f32,
    pub employment: f32,
    pub services: f32,
    pub environment: f32,
    pub economy: f32,
}

/// Main resource holding all chart history data.
#[derive(Resource, Default, Encode, Decode)]
pub struct ChartHistory {
    pub population: Vec<PopulationSnapshot>,
    pub budget: Vec<BudgetSnapshot>,
    pub traffic_hourly: TrafficByHour,
    pub service_coverage: ServiceCoverageSnapshot,
    pub happiness: HappinessBreakdown,
    pub last_record_day: u32,
}

/// Transient accumulator for traffic hourly averaging (not saved).
#[derive(Resource)]
pub struct TrafficHourlyAccum {
    pub samples: [u32; 24],
    pub accum: [f32; 24],
}

impl Default for TrafficHourlyAccum {
    fn default() -> Self {
        Self {
            samples: [0; 24],
            accum: [0.0; 24],
        }
    }
}

impl crate::Saveable for ChartHistory {
    const SAVE_KEY: &'static str = "chart_history";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.population.is_empty() {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        bitcode::decode(bytes).unwrap_or_default()
    }
}

// -----------------------------------------------------------------------
// Systems
// -----------------------------------------------------------------------

/// Records traffic congestion by hour. Runs frequently to capture intra-day patterns.
pub fn record_traffic_hourly(
    tick: Res<crate::TickCounter>,
    clock: Res<GameClock>,
    traffic: Res<TrafficGrid>,
    grid: Res<WorldGrid>,
    mut history: ResMut<ChartHistory>,
    mut accum: ResMut<TrafficHourlyAccum>,
) {
    // Sample every 20 ticks (~2 seconds real time)
    if !tick.0.is_multiple_of(20) {
        return;
    }

    let hour = clock.hour_of_day() as usize;
    if hour >= 24 {
        return;
    }

    // Compute average congestion across all road cells
    let mut total_congestion = 0.0_f32;
    let mut road_count = 0u32;
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            if grid.get(x, y).cell_type == CellType::Road {
                total_congestion += traffic.congestion_level(x, y);
                road_count += 1;
            }
        }
    }

    let avg = if road_count > 0 {
        total_congestion / road_count as f32
    } else {
        0.0
    };

    accum.accum[hour] += avg;
    accum.samples[hour] += 1;

    // Update the displayed average
    if accum.samples[hour] > 0 {
        history.traffic_hourly.congestion[hour] = accum.accum[hour] / accum.samples[hour] as f32;
    }
}

/// Records population, budget, service coverage, and happiness snapshots every 10 days.
#[allow(clippy::too_many_arguments)]
pub fn record_chart_snapshots(
    clock: Res<GameClock>,
    stats: Res<CityStats>,
    budget: Res<CityBudget>,
    extended: Res<ExtendedBudget>,
    grid: Res<WorldGrid>,
    coverage: Res<ServiceCoverageGrid>,
    citizens: Query<(&CitizenDetails, Option<&WorkLocation>), With<Citizen>>,
    mut history: ResMut<ChartHistory>,
) {
    if clock.day <= history.last_record_day + 10 {
        return;
    }
    history.last_record_day = clock.day;

    // --- Population with R/C/I worker breakdown ---
    let mut res_workers = 0u32;
    let mut com_workers = 0u32;
    let mut ind_workers = 0u32;

    for (_details, work) in &citizens {
        if let Some(work_loc) = work {
            let cell = grid.get(work_loc.grid_x, work_loc.grid_y);
            if cell.zone.is_residential() {
                res_workers += 1;
            } else if cell.zone.is_commercial() || cell.zone.is_mixed_use() {
                com_workers += 1;
            } else if cell.zone == ZoneType::Industrial {
                ind_workers += 1;
            }
        }
    }

    history.population.push(PopulationSnapshot {
        total: stats.population,
        residential_workers: res_workers,
        commercial_workers: com_workers,
        industrial_workers: ind_workers,
    });

    // --- Budget breakdown ---
    history.budget.push(BudgetSnapshot {
        residential_tax: extended.income_breakdown.residential_tax,
        commercial_tax: extended.income_breakdown.commercial_tax,
        industrial_tax: extended.income_breakdown.industrial_tax,
        office_tax: extended.income_breakdown.office_tax,
        trade_income: extended.income_breakdown.trade_income,
        road_maintenance: extended.expense_breakdown.road_maintenance,
        service_costs: extended.expense_breakdown.service_costs,
        policy_costs: extended.expense_breakdown.policy_costs,
        loan_payments: extended.expense_breakdown.loan_payments,
    });

    // --- Service coverage ---
    let mut occupied_count = 0u32;
    let mut health_count = 0u32;
    let mut edu_count = 0u32;
    let mut police_count = 0u32;
    let mut fire_count = 0u32;
    let mut park_count = 0u32;
    let mut ent_count = 0u32;
    let mut telecom_count = 0u32;
    let mut transport_count = 0u32;

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let cell = grid.get(x, y);
            if cell.building_id.is_some() || cell.cell_type == CellType::Road {
                occupied_count += 1;
                let idx = ServiceCoverageGrid::idx(x, y);
                if coverage.has_health(idx) {
                    health_count += 1;
                }
                if coverage.has_education(idx) {
                    edu_count += 1;
                }
                if coverage.has_police(idx) {
                    police_count += 1;
                }
                if coverage.has_fire(idx) {
                    fire_count += 1;
                }
                if coverage.has_park(idx) {
                    park_count += 1;
                }
                if coverage.has_entertainment(idx) {
                    ent_count += 1;
                }
                if coverage.has_telecom(idx) {
                    telecom_count += 1;
                }
                if coverage.has_transport(idx) {
                    transport_count += 1;
                }
            }
        }
    }

    let to_pct = |count: u32| -> f32 {
        if occupied_count > 0 {
            count as f32 / occupied_count as f32
        } else {
            0.0
        }
    };

    history.service_coverage = ServiceCoverageSnapshot {
        health: to_pct(health_count),
        education: to_pct(edu_count),
        police: to_pct(police_count),
        fire: to_pct(fire_count),
        parks: to_pct(park_count),
        entertainment: to_pct(ent_count),
        telecom: to_pct(telecom_count),
        transport: to_pct(transport_count),
    };

    // --- Happiness breakdown ---
    let mut total_happiness = 0.0_f32;
    let mut employed_count = 0u32;
    let citizen_count = citizens.iter().count() as f32;

    for (details, work) in &citizens {
        total_happiness += details.happiness;
        if work.is_some() {
            employed_count += 1;
        }
    }

    let avg_happiness = if citizen_count > 0.0 {
        total_happiness / citizen_count
    } else {
        0.0
    };
    let employment_rate = if citizen_count > 0.0 {
        employed_count as f32 / citizen_count
    } else {
        0.0
    };

    // Approximate happiness factor contributions
    history.happiness = HappinessBreakdown {
        base: 50.0_f32.min(avg_happiness),
        employment: (employment_rate * 15.0).min(avg_happiness.max(0.0)),
        services: (to_pct(health_count) * 5.0
            + to_pct(edu_count) * 3.0
            + to_pct(police_count) * 5.0
            + to_pct(park_count) * 8.0)
            .min(21.0),
        environment: (avg_happiness - 50.0 - employment_rate * 15.0).clamp(-10.0, 10.0),
        economy: if budget.tax_rate > 0.15 {
            -(8.0 * ((budget.tax_rate - 0.15) / 0.10))
        } else {
            2.0
        },
    };

    // Trim old data
    if history.population.len() > MAX_SNAPSHOTS {
        let excess = history.population.len() - MAX_SNAPSHOTS;
        history.population.drain(0..excess);
    }
    if history.budget.len() > MAX_SNAPSHOTS {
        let excess = history.budget.len() - MAX_SNAPSHOTS;
        history.budget.drain(0..excess);
    }
}

// -----------------------------------------------------------------------
// Plugin
// -----------------------------------------------------------------------

pub struct ChartDataPlugin;

impl Plugin for ChartDataPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChartHistory>()
            .init_resource::<TrafficHourlyAccum>()
            .add_systems(
                FixedUpdate,
                (
                    record_traffic_hourly.after(crate::traffic::update_traffic_density),
                    record_chart_snapshots.after(crate::economy::collect_taxes),
                ),
            );

        // Register for save/load
        let mut registry = app
            .world_mut()
            .get_resource_or_insert_with(crate::SaveableRegistry::default);
        registry.register::<ChartHistory>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chart_history_default() {
        let history = ChartHistory::default();
        assert!(history.population.is_empty());
        assert!(history.budget.is_empty());
        assert_eq!(history.last_record_day, 0);
    }

    #[test]
    fn test_traffic_by_hour_default() {
        let t = TrafficByHour::default();
        for h in 0..24 {
            assert_eq!(t.congestion[h], 0.0);
        }
    }

    #[test]
    fn test_saveable_skip_default() {
        let history = ChartHistory::default();
        assert!(
            history.save_to_bytes().is_none(),
            "Default state should skip saving"
        );
    }

    #[test]
    fn test_saveable_roundtrip() {
        let mut history = ChartHistory::default();
        history.population.push(PopulationSnapshot {
            total: 1000,
            residential_workers: 300,
            commercial_workers: 400,
            industrial_workers: 200,
        });
        history.last_record_day = 10;

        let bytes = history.save_to_bytes().expect("Non-default should save");
        let restored = ChartHistory::load_from_bytes(&bytes);

        assert_eq!(restored.population.len(), 1);
        assert_eq!(restored.population[0].total, 1000);
        assert_eq!(restored.last_record_day, 10);
    }

    #[test]
    fn test_snapshot_trimming() {
        let mut history = ChartHistory::default();
        for i in 0..400 {
            history.population.push(PopulationSnapshot {
                total: i,
                ..Default::default()
            });
        }
        // Simulate trim
        if history.population.len() > MAX_SNAPSHOTS {
            let excess = history.population.len() - MAX_SNAPSHOTS;
            history.population.drain(0..excess);
        }
        assert_eq!(history.population.len(), MAX_SNAPSHOTS);
        // First item should be the 40th (index 40, total=40)
        assert_eq!(history.population[0].total, 40);
    }
}
