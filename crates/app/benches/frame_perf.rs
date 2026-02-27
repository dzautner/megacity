//! Full-frame performance benchmarks including rendering systems.
//!
//! These benchmarks measure the CPU-side cost of the Update schedule,
//! which includes all rendering systems (citizen sprites, building meshes,
//! terrain chunks, road meshes, props, overlays, etc.).
//!
//! Run with: cargo bench -p megacity --bench frame_perf --features megacity/bench

use bevy::app::ScheduleRunnerPlugin;
use bevy::gizmos::GizmoPlugin;
use bevy::prelude::*;
use bevy::render::render_resource::Shader;
use bevy::scene::ScenePlugin;
use bevy::state::app::StatesPlugin;

use criterion::{criterion_group, criterion_main, Criterion};

use simulation::tutorial::TutorialState;

/// Build a headless app with both SimulationPlugin and RenderingPlugin.
///
/// Uses MinimalPlugins plus the specific plugins rendering systems depend on
/// (assets, scenes, gizmos, transforms) without requiring a GPU or display.
fn build_headless_rendering_app() -> App {
    let mut app = App::new();

    // Core scheduling + time
    app.add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::default()));

    // Asset infrastructure (needed for Handle<Scene>, Handle<Mesh>, etc.)
    app.add_plugins(AssetPlugin::default());
    app.add_plugins(StatesPlugin);
    app.add_plugins(ScenePlugin);
    app.add_plugins(TransformPlugin);
    app.add_plugins(HierarchyPlugin);

    // Register asset types that rendering and gizmo systems need
    app.init_asset::<Mesh>();
    app.init_asset::<StandardMaterial>();
    app.init_asset::<Image>();
    app.init_asset::<Shader>();

    // Input (needed for keyboard/mouse handling in rendering systems)
    app.add_plugins(bevy::input::InputPlugin);

    // Window events (rendering systems query window/cursor state)
    app.add_plugins(bevy::window::WindowPlugin::default());

    // Gizmos (used by cursor_preview, road_grade, oneway_arrows, selection_highlight)
    app.add_plugins(GizmoPlugin);

    // Skip tutorial
    app.insert_resource(TutorialState {
        completed: true,
        active: false,
        ..Default::default()
    });

    // Start in Playing state so simulation systems run during the benchmark.
    app.insert_state(simulation::AppState::Playing);

    // Simulation
    app.add_plugins(simulation::SimulationPlugin);

    // Tel Aviv map init — not registered by default, add explicitly for benchmarks.
    app.add_systems(Startup, simulation::world_init::init_world);

    // Rendering (CPU-side only — no GPU backend)
    app.add_plugins(rendering::RenderingPlugin);

    // First update: startup systems run (load_building_models, etc.)
    app.update();

    // Clean up tutorial state
    if let Some(mut tutorial) = app.world_mut().get_resource_mut::<TutorialState>() {
        tutorial.completed = true;
        tutorial.active = false;
        tutorial.paused_by_tutorial = false;
    }
    if let Some(mut clock) = app
        .world_mut()
        .get_resource_mut::<simulation::time_of_day::GameClock>()
    {
        clock.paused = false;
    }

    app
}

/// Benchmark the full Update schedule (all rendering + simulation Update systems).
fn bench_update_schedule(c: &mut Criterion) {
    let mut group = c.benchmark_group("rendering");
    group.sample_size(20);

    let mut app = build_headless_rendering_app();

    // Warm up: run a few frames so one-shot systems (props, initial spawns) settle
    for _ in 0..5 {
        app.update();
    }

    group.bench_function("full_update_schedule", |b| {
        b.iter(|| {
            app.world_mut().run_schedule(Update);
        });
    });

    group.finish();
}

/// Benchmark the FixedUpdate + Update combined (a "sim frame" that does both).
fn bench_sim_frame(c: &mut Criterion) {
    let mut group = c.benchmark_group("sim_frame");
    group.sample_size(20);

    let mut app = build_headless_rendering_app();

    for _ in 0..5 {
        app.update();
    }

    group.bench_function("fixed_plus_update", |b| {
        b.iter(|| {
            app.world_mut().run_schedule(FixedUpdate);
            app.world_mut().run_schedule(Update);
        });
    });

    group.finish();
}

criterion_group!(benches, bench_update_schedule, bench_sim_frame);
criterion_main!(benches);
