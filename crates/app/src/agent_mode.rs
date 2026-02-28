//! Headless `--agent` mode: a blocking synchronous loop that reads JSON
//! commands from stdin and writes JSON responses to stdout.
//!
//! When the `--agent` CLI flag is passed, the game skips all rendering and UI
//! plugins and enters this loop instead of the normal Bevy `app.run()`.
//!
//! ## Protocol
//!
//! Each line of stdin is a JSON object with a `"cmd"` discriminator.
//! Each line of stdout is a JSON response with `"protocol_version"` and
//! `"type"` fields. See [`simulation::agent_protocol`] for the full schema.

#[cfg(not(target_arch = "wasm32"))]
pub fn run_agent_mode(seed: Option<u64>) {
    use std::io::{BufRead, Write};

    use bevy::prelude::*;
    use bevy::state::app::StatesPlugin;

    use simulation::agent_protocol::{
        make_response, AgentCommand, ResponsePayload, PROTOCOL_VERSION,
    };
    use simulation::app_state::AppState;
    use simulation::replay::ReplayRecorder;
    use simulation::time_of_day::GameClock;
    use simulation::tutorial::TutorialState;
    use simulation::TickCounter;

    // -- Build a minimal Bevy App with simulation + save, no rendering/UI ---
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);

    // Skip the tutorial so it does not pause the GameClock.
    app.insert_resource(TutorialState {
        completed: true,
        active: false,
        ..Default::default()
    });

    // Start directly in Playing state so simulation systems run.
    app.insert_state(AppState::Playing);

    // Simulation plugin (all game logic, no rendering).
    app.add_plugins(simulation::SimulationPlugin);

    // Save plugin (for future save/load support via agent commands).
    app.add_plugins(save::SavePlugin);

    // Initial update so Startup systems execute and resources initialize.
    app.update();

    // Ensure tutorial is inactive and clock is unpaused after first update.
    if let Some(mut tutorial) = app.world_mut().get_resource_mut::<TutorialState>() {
        tutorial.completed = true;
        tutorial.active = false;
        tutorial.paused_by_tutorial = false;
    }
    if let Some(mut clock) = app.world_mut().get_resource_mut::<GameClock>() {
        clock.paused = false;
    }

    // Start the replay recorder so all agent actions are captured.
    {
        let agent_seed = seed.unwrap_or(0);
        let tick = app
            .world()
            .get_resource::<TickCounter>()
            .map(|t| t.0)
            .unwrap_or(0);
        if let Some(mut recorder) = app.world_mut().get_resource_mut::<ReplayRecorder>() {
            recorder.start(agent_seed, "agent_session".to_string(), tick);
        }
    }

    // -- I/O setup -----------------------------------------------------------
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();

    // Send the "ready" message so the external program knows we are live.
    let ready = make_response(ResponsePayload::Ready);
    let _ = writeln!(stdout, "{}", serde_json::to_string(&ready).unwrap());
    let _ = stdout.flush();

    // Log to stderr so it does not interfere with the JSON protocol on stdout.
    eprintln!(
        "megacity agent mode v{} ready — waiting for commands on stdin",
        PROTOCOL_VERSION
    );

    // -- Main command loop ---------------------------------------------------
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("stdin read error: {e}");
                break;
            }
        };

        if line.trim().is_empty() {
            continue;
        }

        let cmd: AgentCommand = match serde_json::from_str(&line) {
            Ok(c) => c,
            Err(e) => {
                let resp = make_response(ResponsePayload::Error {
                    message: format!("Parse error: {e}"),
                });
                let _ = writeln!(stdout, "{}", serde_json::to_string(&resp).unwrap());
                let _ = stdout.flush();
                continue;
            }
        };

        let response = process_command(cmd, &mut app);
        let is_goodbye = matches!(response.payload, ResponsePayload::Goodbye);

        let _ = writeln!(stdout, "{}", serde_json::to_string(&response).unwrap());
        let _ = stdout.flush();

        if is_goodbye {
            break;
        }
    }

    eprintln!("megacity agent mode shutting down");
}

// ---------------------------------------------------------------------------
// Command dispatch
// ---------------------------------------------------------------------------

#[cfg(not(target_arch = "wasm32"))]
fn process_command(
    cmd: simulation::agent_protocol::AgentCommand,
    app: &mut bevy::app::App,
) -> simulation::agent_protocol::AgentResponse {
    use simulation::agent_protocol::{make_response, AgentCommand, ResponsePayload};
    use simulation::game_actions::{ActionQueue, ActionSource, GameAction};
    use simulation::game_actions::{ActionResult, ActionResultLog};
    use simulation::observation_builder::CurrentObservation;
    use simulation::replay::{ReplayFile, ReplayPlayer, ReplayRecorder};
    use simulation::TickCounter;

    match cmd {
        AgentCommand::Observe => {
            let obs = app
                .world()
                .get_resource::<CurrentObservation>()
                .map(|co| co.observation.clone())
                .unwrap_or_default();
            make_response(ResponsePayload::Observation { observation: obs })
        }

        AgentCommand::Act { action } => {
            let tick = app
                .world()
                .get_resource::<TickCounter>()
                .map(|t| t.0)
                .unwrap_or(0);

            app.world_mut()
                .resource_mut::<ActionQueue>()
                .push(tick, ActionSource::Agent, action);

            // Run one tick so the executor processes the action.
            app.update();

            let result = app
                .world()
                .get_resource::<ActionResultLog>()
                .and_then(|log| log.last_n(1).first().map(|(_, r)| r.clone()))
                .unwrap_or(ActionResult::Success);

            make_response(ResponsePayload::ActionResult { result })
        }

        AgentCommand::BatchAct { actions } => {
            let mut results = Vec::with_capacity(actions.len());
            for action in actions {
                let tick = app
                    .world()
                    .get_resource::<TickCounter>()
                    .map(|t| t.0)
                    .unwrap_or(0);

                app.world_mut().resource_mut::<ActionQueue>().push(
                    tick,
                    ActionSource::Agent,
                    action,
                );

                app.update();

                let result = app
                    .world()
                    .get_resource::<ActionResultLog>()
                    .and_then(|log| log.last_n(1).first().map(|(_, r)| r.clone()))
                    .unwrap_or(ActionResult::Success);

                results.push(result);
            }
            make_response(ResponsePayload::BatchResult { results })
        }

        AgentCommand::Step { ticks } => {
            // Cap at 10 000 ticks to prevent accidental infinite loops.
            let n = ticks.min(10_000);
            for _ in 0..n {
                app.update();
            }
            let tick = app
                .world()
                .get_resource::<TickCounter>()
                .map(|t| t.0)
                .unwrap_or(0);
            make_response(ResponsePayload::StepComplete { tick })
        }

        AgentCommand::NewGame { seed } => {
            // Reset the WorldGrid: clear all cells to default (Grass, no zone/road/building)
            if let Some(mut grid) = app
                .world_mut()
                .get_resource_mut::<simulation::grid::WorldGrid>()
            {
                for cell in grid.cells.iter_mut() {
                    cell.cell_type = simulation::grid::CellType::Grass;
                    cell.zone = simulation::grid::ZoneType::None;
                    cell.road_type = simulation::grid::RoadType::Local;
                    cell.building_id = None;
                    cell.elevation = 0.0;
                    cell.has_power = false;
                    cell.has_water = false;
                }

                // Apply procedural terrain (coastline water bodies from seed)
                simulation::procedural_terrain::generate_terrain(&mut grid, seed);
            }

            // Reset TickCounter to 0
            if let Some(mut tick) = app.world_mut().get_resource_mut::<TickCounter>() {
                tick.0 = 0;
            }

            // Reset CityBudget to default (treasury = 50_000.0)
            app.world_mut()
                .insert_resource(simulation::economy::CityBudget::default());

            // Restart the ReplayRecorder with the new seed
            if let Some(mut recorder) = app.world_mut().get_resource_mut::<ReplayRecorder>() {
                recorder.start(seed, "agent_session".to_string(), 0);
            }

            // Run one update so systems settle after the reset
            app.update();

            make_response(ResponsePayload::Ok)
        }

        AgentCommand::SaveReplay { path } => {
            let tick = app
                .world()
                .get_resource::<TickCounter>()
                .map(|t| t.0)
                .unwrap_or(0);

            let replay_file = app
                .world_mut()
                .get_resource_mut::<ReplayRecorder>()
                .map(|mut recorder| recorder.stop(tick, 0));

            match replay_file {
                Some(replay) => {
                    let json = replay.to_json();
                    if let Err(e) = std::fs::write(&path, json) {
                        make_response(ResponsePayload::Error {
                            message: format!("Failed to write replay to {path}: {e}"),
                        })
                    } else {
                        make_response(ResponsePayload::Ok)
                    }
                }
                None => make_response(ResponsePayload::Error {
                    message: "ReplayRecorder resource not found".to_string(),
                }),
            }
        }

        AgentCommand::LoadReplay { path } => {
            let contents = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => {
                    return make_response(ResponsePayload::Error {
                        message: format!("Failed to read replay from {path}: {e}"),
                    });
                }
            };

            let replay = match ReplayFile::from_json(&contents) {
                Ok(r) => r,
                Err(e) => {
                    return make_response(ResponsePayload::Error {
                        message: format!("Failed to parse replay: {e}"),
                    });
                }
            };

            match app.world_mut().get_resource_mut::<ReplayPlayer>() {
                Some(mut player) => {
                    player.load(replay);
                    make_response(ResponsePayload::Ok)
                }
                None => make_response(ResponsePayload::Error {
                    message: "ReplayPlayer resource not found".to_string(),
                }),
            }
        }

        AgentCommand::Query { layers } => handle_query(layers, app),

        AgentCommand::Quit => make_response(ResponsePayload::Goodbye),
    }
}

// ---------------------------------------------------------------------------
// Query command handler
// ---------------------------------------------------------------------------

#[cfg(not(target_arch = "wasm32"))]
fn handle_query(
    layers: Vec<String>,
    app: &mut bevy::app::App,
) -> simulation::agent_protocol::AgentResponse {
    use simulation::agent_protocol::{make_response, ResponsePayload};

    let mut result = serde_json::Map::new();

    // Build the world snapshot once if any snapshot-based layers are requested
    let needs_snapshot = layers.iter().any(|l| {
        matches!(
            l.as_str(),
            "buildings" | "services" | "utilities" | "roads" | "zones" | "terrain"
        )
    });

    let snapshot = if needs_snapshot {
        Some(simulation::world_snapshot::build_world_snapshot(
            app.world_mut(),
        ))
    } else {
        None
    };

    for layer in &layers {
        match layer.as_str() {
            "map" => {
                let grid = app.world().resource::<simulation::grid::WorldGrid>();
                let detail = simulation::ascii_map::build_detail_map(grid, 10);
                result.insert("map".to_string(), serde_json::Value::String(detail));
            }
            "overview" => {
                let grid = app.world().resource::<simulation::grid::WorldGrid>();
                let overview = simulation::ascii_map::build_overview_map(grid);
                result.insert("overview".to_string(), serde_json::Value::String(overview));
            }
            "buildings" => {
                let text = simulation::world_snapshot_format::format_buildings(
                    &snapshot.as_ref().unwrap().buildings,
                );
                result.insert("buildings".to_string(), serde_json::Value::String(text));
            }
            "services" => {
                let text = simulation::world_snapshot_format::format_services(
                    &snapshot.as_ref().unwrap().services,
                );
                result.insert("services".to_string(), serde_json::Value::String(text));
            }
            "utilities" => {
                let text = simulation::world_snapshot_format::format_utilities(
                    &snapshot.as_ref().unwrap().utilities,
                );
                result.insert("utilities".to_string(), serde_json::Value::String(text));
            }
            "roads" => {
                let text = simulation::world_snapshot_format::format_roads_summary(
                    &snapshot.as_ref().unwrap().road_cells,
                );
                result.insert("roads".to_string(), serde_json::Value::String(text));
            }
            "zones" => {
                let text = simulation::world_snapshot_format::format_zones_summary(
                    &snapshot.as_ref().unwrap().zone_regions,
                );
                result.insert("zones".to_string(), serde_json::Value::String(text));
            }
            "terrain" => {
                let text = simulation::world_snapshot_format::format_terrain(
                    &snapshot.as_ref().unwrap().water_regions,
                );
                result.insert("terrain".to_string(), serde_json::Value::String(text));
            }
            unknown => {
                result.insert(
                    unknown.to_string(),
                    serde_json::Value::String(format!("Unknown layer: {}", unknown)),
                );
            }
        }
    }

    make_response(ResponsePayload::QueryResult {
        layers: serde_json::Value::Object(result),
    })
}
