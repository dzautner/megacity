//! Charts panel (UX-046): Enhanced graphs with population lines, budget area,
//! traffic bars, service radar, and happiness breakdown.

mod drawing;
mod population_budget;
mod traffic_services_happiness;

use std::collections::VecDeque;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use simulation::chart_data::ChartHistory;
use simulation::stats::CityStats;
use simulation::time_of_day::GameClock;

pub(crate) use drawing::{congestion_color, draw_multi_line_chart, draw_sparkline, legend_item};
pub(crate) use drawing::{draw_stacked_area, tail_slice};

use population_budget::{draw_budget_chart, draw_population_chart};
use traffic_services_happiness::{
    draw_happiness_breakdown, draw_service_radar, draw_traffic_chart,
};

const MAX_HISTORY: usize = 200;

// -----------------------------------------------------------------------
// Legacy history (kept for backward compat with existing record_history)
// -----------------------------------------------------------------------

#[derive(Resource)]
pub struct HistoryData {
    pub population: VecDeque<f32>,
    pub happiness: VecDeque<f32>,
    pub treasury: VecDeque<f32>,
    pub last_record_day: u32,
}

impl Default for HistoryData {
    fn default() -> Self {
        Self {
            population: VecDeque::with_capacity(MAX_HISTORY),
            happiness: VecDeque::with_capacity(MAX_HISTORY),
            treasury: VecDeque::with_capacity(MAX_HISTORY),
            last_record_day: 0,
        }
    }
}

pub fn record_history(
    clock: Res<GameClock>,
    stats: Res<CityStats>,
    budget: Res<simulation::economy::CityBudget>,
    mut history: ResMut<HistoryData>,
) {
    // Record every 10 game days
    if clock.day <= history.last_record_day + 10 {
        return;
    }
    history.last_record_day = clock.day;

    history.population.push_back(stats.population as f32);
    history.happiness.push_back(stats.average_happiness);
    history.treasury.push_back(budget.treasury as f32);

    // Trim old data (O(1) front removal with VecDeque)
    if history.population.len() > MAX_HISTORY {
        history.population.pop_front();
        history.happiness.pop_front();
        history.treasury.pop_front();
    }
}

// -----------------------------------------------------------------------
// Time range selector
// -----------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TimeRange {
    Month, // ~3 snapshots (30 days / 10 days per snapshot)
    Year,  // ~36 snapshots (360 days)
    AllTime,
}

impl TimeRange {
    pub(crate) fn label(self) -> &'static str {
        match self {
            TimeRange::Month => "1 Month",
            TimeRange::Year => "1 Year",
            TimeRange::AllTime => "All Time",
        }
    }

    pub(crate) fn max_points(self) -> usize {
        match self {
            TimeRange::Month => 3,
            TimeRange::Year => 36,
            TimeRange::AllTime => usize::MAX,
        }
    }
}

// -----------------------------------------------------------------------
// Chart tab
// -----------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChartTab {
    Population,
    Budget,
    Traffic,
    Services,
    Happiness,
}

impl ChartTab {
    fn label(self) -> &'static str {
        match self {
            ChartTab::Population => "Population",
            ChartTab::Budget => "Budget",
            ChartTab::Traffic => "Traffic",
            ChartTab::Services => "Services",
            ChartTab::Happiness => "Happiness",
        }
    }

    const ALL: [ChartTab; 5] = [
        ChartTab::Population,
        ChartTab::Budget,
        ChartTab::Traffic,
        ChartTab::Services,
        ChartTab::Happiness,
    ];
}

// -----------------------------------------------------------------------
// Main UI system
// -----------------------------------------------------------------------

/// Persistent state for the charts panel (tab + range selection).
#[derive(Resource)]
pub struct ChartsState {
    tab: ChartTab,
    range: TimeRange,
}

impl Default for ChartsState {
    fn default() -> Self {
        Self {
            tab: ChartTab::Population,
            range: TimeRange::AllTime,
        }
    }
}

pub fn graphs_ui(
    mut contexts: EguiContexts,
    history: Res<HistoryData>,
    chart_history: Res<ChartHistory>,
    visible: Res<crate::info_panel::ChartsVisible>,
    mut state: ResMut<ChartsState>,
) {
    if !visible.0 {
        return;
    }

    egui::Window::new("Charts")
        .default_size([420.0, 380.0])
        .show(contexts.ctx_mut(), |ui| {
            ui.small("Press [C] to toggle");

            // Tab bar
            ui.horizontal(|ui| {
                for tab in ChartTab::ALL {
                    if ui.selectable_label(state.tab == tab, tab.label()).clicked() {
                        state.tab = tab;
                    }
                }
            });

            // Time range selector
            ui.horizontal(|ui| {
                ui.label("Range:");
                for range in [TimeRange::Month, TimeRange::Year, TimeRange::AllTime] {
                    if ui
                        .selectable_label(state.range == range, range.label())
                        .clicked()
                    {
                        state.range = range;
                    }
                }
            });

            ui.separator();

            match state.tab {
                ChartTab::Population => {
                    draw_population_chart(ui, &chart_history, &history, state.range)
                }
                ChartTab::Budget => draw_budget_chart(ui, &chart_history, state.range),
                ChartTab::Traffic => draw_traffic_chart(ui, &chart_history),
                ChartTab::Services => draw_service_radar(ui, &chart_history),
                ChartTab::Happiness => draw_happiness_breakdown(ui, &chart_history),
            }
        });
}
