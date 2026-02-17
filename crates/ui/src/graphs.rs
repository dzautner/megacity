use std::collections::VecDeque;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use simulation::stats::CityStats;
use simulation::time_of_day::GameClock;

const MAX_HISTORY: usize = 200;

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

pub fn graphs_ui(
    mut contexts: EguiContexts,
    history: Res<HistoryData>,
) {
    egui::Window::new("Trends")
        .default_open(false)
        .show(contexts.ctx_mut(), |ui| {
            if history.population.is_empty() {
                ui.label("No data yet...");
                return;
            }

            let pop: Vec<f32> = history.population.iter().copied().collect();
            let hap: Vec<f32> = history.happiness.iter().copied().collect();
            let tre: Vec<f32> = history.treasury.iter().copied().collect();

            ui.heading("Population");
            draw_sparkline(ui, &pop, egui::Color32::GREEN);

            ui.heading("Happiness");
            draw_sparkline(ui, &hap, egui::Color32::YELLOW);

            ui.heading("Treasury");
            draw_sparkline(ui, &tre, egui::Color32::GOLD);
        });
}

fn draw_sparkline(ui: &mut egui::Ui, data: &[f32], color: egui::Color32) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(180.0, 40.0),
        egui::Sense::hover(),
    );

    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 2.0, egui::Color32::from_gray(30));

    if data.len() < 2 {
        return;
    }

    let min_val = data.iter().cloned().fold(f32::INFINITY, f32::min);
    let max_val = data.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let range = (max_val - min_val).max(1.0);

    let points: Vec<egui::Pos2> = data
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            let x = rect.min.x + (i as f32 / (data.len() - 1) as f32) * rect.width();
            let y = rect.max.y - ((v - min_val) / range) * rect.height();
            egui::pos2(x, y)
        })
        .collect();

    for window in points.windows(2) {
        painter.line_segment([window[0], window[1]], egui::Stroke::new(1.5, color));
    }

    // Show latest value
    if let Some(&last) = data.last() {
        ui.label(format!("  Latest: {:.0}", last));
    }
}
