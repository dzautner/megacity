use bevy::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use bevy::window::ExitCondition;
use bevy::window::PresentMode;
use bevy::winit::{UpdateMode, WinitSettings};

#[cfg(not(target_arch = "wasm32"))]
use bevy::render::camera::RenderTarget;
#[cfg(not(target_arch = "wasm32"))]
use bevy::render::render_asset::RenderAssetUsages;
#[cfg(not(target_arch = "wasm32"))]
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
#[cfg(not(target_arch = "wasm32"))]
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
#[cfg(not(target_arch = "wasm32"))]
use rendering::camera::OrbitCamera;

#[cfg(not(target_arch = "wasm32"))]
mod agent_mode;
#[cfg(not(target_arch = "wasm32"))]
mod record_replay;
#[cfg(target_arch = "wasm32")]
mod web_replay;

fn main() {
    // -- CLI argument parsing -------------------------------------------------
    let args: Vec<String> = std::env::args().collect();
    let is_agent = args.iter().any(|a| a == "--agent");

    // -- Agent mode: headless JSON protocol over stdin/stdout ----------------
    #[cfg(not(target_arch = "wasm32"))]
    if is_agent {
        // Parse optional --seed <N> for agent mode (native only).
        let seed: Option<u64> = args
            .windows(2)
            .find(|w| w[0] == "--seed")
            .and_then(|w| w[1].parse().ok());
        agent_mode::run_agent_mode(seed);
        return;
    }
    #[cfg(target_arch = "wasm32")]
    if is_agent {
        panic!("Agent mode is not supported on WASM");
    }

    // Parse replay source for graphical playback.
    // Native: `--replay <path>`
    // WASM:   `?replay=<url>`
    #[cfg(not(target_arch = "wasm32"))]
    let replay_source: Option<String> = args
        .windows(2)
        .find(|w| w[0] == "--replay")
        .map(|w| w[1].clone());
    #[cfg(target_arch = "wasm32")]
    let replay_source: Option<String> = web_replay::query_replay_url();
    let replay_mode = replay_source.is_some();

    // Parse optional --record <output_dir> for replay frame capture (native only).
    #[cfg(not(target_arch = "wasm32"))]
    let record_dir: Option<String> = args
        .windows(2)
        .find(|w| w[0] == "--record")
        .map(|w| w[1].clone());
    #[cfg(not(target_arch = "wasm32"))]
    let record_mode = replay_mode && record_dir.is_some();
    #[cfg(target_arch = "wasm32")]
    let record_mode = false;

    let mut app = App::new();

    // --- Window configuration ---------------------------------------------------
    // On WASM we must point Bevy at the existing <canvas id="bevy-canvas"> so
    // winit attaches event listeners to the right element. We also enable
    // `fit_canvas_to_parent` so the drawing-buffer resolution stays in sync with
    // the CSS layout size, preventing coordinate mismatches.
    #[allow(unused_mut)]
    let mut window_plugin = WindowPlugin {
        primary_window: {
            #[cfg(not(target_arch = "wasm32"))]
            {
                if record_mode {
                    // Headless replay recording: no game window is created.
                    None
                } else {
                    Some(Window {
                        title: "MegaCity".to_string(),
                        resolution: (1280.0, 720.0).into(),
                        present_mode: PresentMode::AutoVsync,
                        ..default()
                    })
                }
            }

            #[cfg(target_arch = "wasm32")]
            {
                let mut win = Window {
                    title: "MegaCity".to_string(),
                    resolution: (1280.0, 720.0).into(),
                    present_mode: PresentMode::AutoVsync,
                    ..default()
                };
                win.canvas = Some("#bevy-canvas".to_string());
                win.fit_canvas_to_parent = true;
                win.prevent_default_event_handling = true;
                Some(win)
            }
        },
        ..default()
    };

    #[cfg(not(target_arch = "wasm32"))]
    if record_mode {
        // Without windows, default exit-on-window-close would terminate instantly.
        window_plugin.exit_condition = ExitCondition::DontExit;
        window_plugin.close_when_requested = false;
    }

    app.add_plugins(DefaultPlugins.set(window_plugin))
        .insert_resource(WinitSettings {
            // Continuous ensures the game loop runs every frame (driven by
            // requestAnimationFrame on WASM). reactive_low_power was previously used
            // here, but on WASM the timer-based wake could stall the event loop
            // and make the simulation appear frozen.
            focused_mode: UpdateMode::Continuous,
            unfocused_mode: UpdateMode::reactive_low_power(std::time::Duration::from_millis(100)),
        });

    // In record mode we render into an offscreen image and capture that target.
    #[cfg(not(target_arch = "wasm32"))]
    if record_mode {
        const RECORD_WIDTH: u32 = 1280;
        const RECORD_HEIGHT: u32 = 720;
        let size = Extent3d {
            width: RECORD_WIDTH,
            height: RECORD_HEIGHT,
            depth_or_array_layers: 1,
        };
        let mut render_target_image = Image::new_fill(
            size,
            TextureDimension::D2,
            &[0; 4],
            TextureFormat::bevy_default(),
            RenderAssetUsages::default(),
        );
        render_target_image.texture_descriptor.usage = TextureUsages::COPY_SRC
            | TextureUsages::COPY_DST
            | TextureUsages::RENDER_ATTACHMENT
            | TextureUsages::TEXTURE_BINDING;

        let render_target_handle = {
            let mut images = app.world_mut().resource_mut::<Assets<Image>>();
            images.add(render_target_image)
        };

        app.insert_resource(rendering::HeadlessRecordMode);
        app.insert_resource(rendering::CameraRenderTarget(RenderTarget::Image(
            render_target_handle.clone(),
        )));
        app.insert_resource(record_replay::ReplayCaptureTarget(render_target_handle));
    }

    // Screenshot mode needs the Tel Aviv map and active simulation;
    // normal startup boots into MainMenu with an empty grid.
    #[cfg(not(target_arch = "wasm32"))]
    let screenshot_mode = std::env::var("MEGACITY_SCREENSHOTS").is_ok();
    #[cfg(target_arch = "wasm32")]
    let screenshot_mode = false;

    if screenshot_mode {
        app.insert_state(simulation::AppState::Playing);
        app.add_systems(Startup, simulation::world_init::init_world);
    } else if replay_mode {
        // Replay mode: skip main menu, start directly in Playing state.
        // The replay starts from a blank grid (same as agent mode).
        app.insert_state(simulation::AppState::Playing);
    } else {
        app.insert_state(simulation::AppState::MainMenu);
    }

    // Replay viewer mode is watch-only (camera + playback controls only).
    if replay_mode {
        app.insert_resource(simulation::replay::ReplayViewerMode);
    }

    if record_mode {
        // UI depends on egui window contexts; skip it in headless record mode.
        app.add_plugins((
            simulation::SimulationPlugin,
            rendering::RenderingPlugin,
            save::SavePlugin,
        ));
    } else {
        app.add_plugins((
            simulation::SimulationPlugin,
            rendering::RenderingPlugin,
            ui::UiPlugin,
            save::SavePlugin,
        ));
    }

    // Replay mode: register startup system to load the replay file (native only)
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(path) = replay_source {
        app.insert_resource(ReplayFilePath(path));
        app.add_systems(Startup, load_replay_file);
    }

    // WASM replay mode: fetch replay JSON from URL and start playback.
    #[cfg(target_arch = "wasm32")]
    if let Some(url) = replay_source {
        app.insert_resource(web_replay::WebReplaySource(url));
        app.init_resource::<web_replay::WebReplayLoadBuffer>();
        app.add_systems(Startup, web_replay::begin_web_replay_load);
        app.add_systems(Update, web_replay::poll_web_replay_load);
    }

    // Replay record mode (native only): capture PNG frames during replay.
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(dir) = record_dir {
        if !replay_mode {
            eprintln!("Warning: --record requires --replay; ignoring --record.");
        } else if let Err(e) = std::fs::create_dir_all(&dir) {
            eprintln!(
                "Failed to create --record output directory '{}': {}",
                dir, e
            );
        } else {
            app.insert_resource(record_replay::ReplayRecordState::new(dir));
            app.add_systems(Update, record_replay::drive_replay_recording);
        }
    }

    // Screenshot mode: takes preset screenshots and exits (native only)
    #[cfg(not(target_arch = "wasm32"))]
    if screenshot_mode {
        let world_cx = 128.0 * 16.0; // center of 256x256 grid
        let world_cz = 128.0 * 16.0;
        let center = Vec3::new(world_cx, 0.0, world_cz);

        let jaffa = Vec3::new(58.0 * 16.0, 0.0, 48.0 * 16.0);
        let white_city = Vec3::new(100.0 * 16.0, 0.0, 100.0 * 16.0);
        let coast_mid = Vec3::new(63.0 * 16.0, 0.0, 120.0 * 16.0);
        let ayalon = Vec3::new(185.0 * 16.0, 0.0, 100.0 * 16.0);
        let ramat_aviv = Vec3::new(110.0 * 16.0, 0.0, 210.0 * 16.0);

        app.insert_resource(ScreenshotQueue {
            frame: 0,
            current: 0,
            presets: vec![
                ShotPreset {
                    name: "01_overview",
                    focus: center,
                    yaw: 0.0,
                    pitch: 60f32.to_radians(),
                    distance: 2500.0,
                },
                ShotPreset {
                    name: "02_jaffa",
                    focus: jaffa,
                    yaw: 0.5,
                    pitch: 40f32.to_radians(),
                    distance: 300.0,
                },
                ShotPreset {
                    name: "03_coast",
                    focus: coast_mid,
                    yaw: 0.8,
                    pitch: 35f32.to_radians(),
                    distance: 300.0,
                },
                ShotPreset {
                    name: "04_white_city",
                    focus: white_city,
                    yaw: 0.0,
                    pitch: 40f32.to_radians(),
                    distance: 350.0,
                },
                ShotPreset {
                    name: "05_white_city_close",
                    focus: white_city,
                    yaw: 0.2,
                    pitch: 30f32.to_radians(),
                    distance: 150.0,
                },
                ShotPreset {
                    name: "06_ayalon_hwy",
                    focus: ayalon,
                    yaw: -0.3,
                    pitch: 35f32.to_radians(),
                    distance: 400.0,
                },
                ShotPreset {
                    name: "07_ramat_aviv",
                    focus: ramat_aviv,
                    yaw: 0.0,
                    pitch: 40f32.to_radians(),
                    distance: 400.0,
                },
                ShotPreset {
                    name: "08_downtown_tight",
                    focus: white_city + Vec3::new(200.0, 0.0, 100.0),
                    yaw: -0.2,
                    pitch: 28f32.to_radians(),
                    distance: 100.0,
                },
            ],
        });
        app.add_systems(Update, drive_screenshots);
    }

    app.run();
}

// ---------------------------------------------------------------------------
// Replay playback (native only)
// ---------------------------------------------------------------------------

/// Resource holding the path to the replay file, consumed by the startup system.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Resource)]
struct ReplayFilePath(String);

/// Startup system that reads a replay file from disk and loads it into the
/// `ReplayPlayer` for automatic playback. The `feed_replay_actions` system
/// (registered by `ReplayPlugin`) handles injecting actions each tick.
#[cfg(not(target_arch = "wasm32"))]
fn load_replay_file(
    replay_path: Res<ReplayFilePath>,
    mut player: ResMut<simulation::replay::ReplayPlayer>,
    mut commands: Commands,
) {
    info!("Loading replay from: {}", replay_path.0);
    let contents = match std::fs::read_to_string(&replay_path.0) {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to read replay file '{}': {e}", replay_path.0);
            return;
        }
    };
    let replay = match simulation::replay::ReplayFile::from_json(&contents) {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to parse replay file '{}': {e}", replay_path.0);
            return;
        }
    };
    if let Err(e) = replay.validate() {
        warn!("Replay validation warning: {}", e);
    }
    info!(
        "Replay loaded: {} entries, ticks {}..{}",
        replay.entries.len(),
        replay.header.start_tick,
        replay.footer.end_tick,
    );
    commands.insert_resource(simulation::replay::ReplayViewerInfo {
        source: replay_path.0.clone(),
        start_tick: replay.header.start_tick,
        end_tick: replay.footer.end_tick,
        entry_count: replay.entries.len() as u64,
    });
    player.load(replay);
}

// ---------------------------------------------------------------------------
// Screenshot mode (native only)
// ---------------------------------------------------------------------------

#[cfg(not(target_arch = "wasm32"))]
#[derive(Resource)]
struct ScreenshotQueue {
    frame: u32,
    current: usize,
    presets: Vec<ShotPreset>,
}

#[cfg(not(target_arch = "wasm32"))]
struct ShotPreset {
    name: &'static str,
    focus: Vec3,
    yaw: f32,
    pitch: f32,
    distance: f32,
}

#[cfg(not(target_arch = "wasm32"))]
fn drive_screenshots(
    mut commands: Commands,
    mut queue: ResMut<ScreenshotQueue>,
    mut orbit: ResMut<OrbitCamera>,
    mut exit: EventWriter<AppExit>,
) {
    queue.frame += 1;

    // Wait for initial render + let citizens start commuting (clock starts 6AM, commute at 7-8AM)
    if queue.frame < 300 {
        return;
    }

    let idx = queue.current;
    if idx >= queue.presets.len() {
        // All done — wait a few frames for the last save, then exit
        if queue.frame > 300 + queue.presets.len() as u32 * 12 + 20 {
            exit.send(AppExit::Success);
        }
        return;
    }

    let phase = (queue.frame - 300) % 12;

    if phase == 0 {
        // Move camera to preset
        let p = &queue.presets[idx];
        orbit.focus = p.focus;
        orbit.yaw = p.yaw;
        orbit.pitch = p.pitch;
        orbit.distance = p.distance;
    } else if phase == 6 {
        // Take screenshot (after 6 frames for render to settle)
        let name = queue.presets[idx].name;
        let path = format!("/tmp/megacity_{}.png", name);
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path));
        queue.current += 1;
    }
}
