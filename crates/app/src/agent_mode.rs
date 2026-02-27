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
pub fn run_agent_mode() {
    use std::io::{BufRead, Write};

    use bevy::prelude::*;
    use bevy::state::app::StatesPlugin;

    use simulation::agent_protocol::{
        make_response, AgentCommand, ResponsePayload, PROTOCOL_VERSION,
    };
    use simulation::app_state::AppState;
    use simulation::time_of_day::GameClock;
    use simulation::tutorial::TutorialState;

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
            // Queue a NewGame action so the executor handles world reset.
            let tick = app
                .world()
                .get_resource::<TickCounter>()
                .map(|t| t.0)
                .unwrap_or(0);
            app.world_mut().resource_mut::<ActionQueue>().push(
                tick,
                ActionSource::Agent,
                GameAction::NewGame {
                    seed,
                    map_size: None,
                },
            );
            app.update();
            make_response(ResponsePayload::Ok)
        }

        AgentCommand::SaveReplay { .. } => {
            // Stub — replay save not yet implemented.
            make_response(ResponsePayload::Ok)
        }

        AgentCommand::LoadReplay { .. } => {
            // Stub — replay load not yet implemented.
            make_response(ResponsePayload::Ok)
        }

        AgentCommand::Quit => make_response(ResponsePayload::Goodbye),
    }
}
