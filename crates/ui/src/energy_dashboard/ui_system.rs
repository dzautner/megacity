//! Main energy dashboard UI system.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use simulation::battery_storage::BatteryState;
use simulation::coal_power::{CoalPowerState, PowerPlant, PowerPlantType};
use simulation::energy_demand::EnergyGrid;
use simulation::energy_dispatch::EnergyDispatchState;
use simulation::energy_pricing::{EnergyEconomics, TimeOfUsePeriod};
use simulation::time_of_day::GameClock;
use simulation::wind_power::WindPowerState;

use super::panels;
use super::types::{EnergyDashboardVisible, EnergyHistory, GenerationMix};

/// Records a history sample once per game-hour.
pub fn record_energy_history(
    clock: Res<GameClock>,
    energy_grid: Res<EnergyGrid>,
    mut history: ResMut<EnergyHistory>,
) {
    let current_hour = clock.hour as u32;
    if current_hour == history.last_recorded_hour {
        return;
    }
    history.last_recorded_hour = current_hour;
    history.push(energy_grid.total_demand_mwh, energy_grid.total_supply_mwh);
}

/// Aggregates the generation mix from per-type state resources.
fn build_generation_mix(
    coal_state: &CoalPowerState,
    wind_state: &WindPowerState,
    battery_state: &BatteryState,
    plants: &Query<&PowerPlant>,
) -> GenerationMix {
    // Gas output: sum from PowerPlant entities of type NaturalGas.
    let gas_mw: f32 = plants
        .iter()
        .filter(|p| p.plant_type == PowerPlantType::NaturalGas)
        .map(|p| p.current_output_mw)
        .sum();

    GenerationMix {
        coal_mw: coal_state.total_output_mw,
        gas_mw,
        wind_mw: wind_state.total_output_mw,
        battery_mw: battery_state.last_discharge_mwh,
    }
}

/// Period display name.
fn period_name(period: TimeOfUsePeriod) -> &'static str {
    match period {
        TimeOfUsePeriod::OffPeak => "Off-Peak",
        TimeOfUsePeriod::MidPeak => "Mid-Peak",
        TimeOfUsePeriod::OnPeak => "On-Peak",
    }
}

/// Displays the energy dashboard window.
#[allow(clippy::too_many_arguments)]
pub fn energy_dashboard_ui(
    mut contexts: EguiContexts,
    visible: Res<EnergyDashboardVisible>,
    energy_grid: Res<EnergyGrid>,
    dispatch_state: Res<EnergyDispatchState>,
    economics: Res<EnergyEconomics>,
    coal_state: Res<CoalPowerState>,
    wind_state: Res<WindPowerState>,
    battery_state: Res<BatteryState>,
    history: Res<EnergyHistory>,
    plants: Query<&PowerPlant>,
) {
    if !visible.0 {
        return;
    }

    let mix = build_generation_mix(
        &coal_state,
        &wind_state,
        &battery_state,
        &plants,
    );

    egui::Window::new("Energy Dashboard")
        .default_open(true)
        .default_width(340.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.small("Power grid overview");
            ui.separator();

            panels::render_blackout_status(
                ui,
                dispatch_state.has_deficit,
                dispatch_state.load_shed_fraction,
                dispatch_state.blackout_cells,
            );

            ui.add_space(4.0);
            ui.separator();

            panels::render_supply_demand(
                ui,
                energy_grid.total_demand_mwh,
                energy_grid.total_supply_mwh,
                energy_grid.reserve_margin,
            );

            ui.add_space(4.0);
            ui.separator();

            panels::render_price(
                ui,
                economics.current_price_per_kwh,
                period_name(economics.current_period),
            );

            ui.add_space(4.0);
            ui.separator();

            panels::render_generation_mix(ui, &mix);

            ui.add_space(4.0);
            ui.separator();

            panels::render_history_graph(ui, &history);
        });
}
