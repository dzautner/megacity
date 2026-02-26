use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy::winit::{UpdateMode, WinitSettings};

#[cfg(not(target_arch = "wasm32"))]
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
#[cfg(not(target_arch = "wasm32"))]
use rendering::camera::OrbitCamera;

fn main() {
    let mut app = App::new();

    // --- Window configuration ---------------------------------------------------
    // On WASM we must point Bevy at the existing <canvas id="bevy-canvas"> so
    // winit attaches event listeners to the right element. We also enable
    // `fit_canvas_to_parent` so the drawing-buffer resolution stays in sync with
    // the CSS layout size, preventing the coordinate mismatch that made every
    // click land in the wrong place and the UI appear unresponsive.
    let primary_window = {
        #[allow(unused_mut)]
        let mut win = Window {
            title: "MegaCity".to_string(),
            resolution: (1280.0, 720.0).into(),
            present_mode: PresentMode::AutoVsync,
            ..default()
        };

        // WASM-specific canvas binding
        #[cfg(target_arch = "wasm32")]
        {
            win.canvas = Some("#bevy-canvas".to_string());
            win.fit_canvas_to_parent = true;
            win.prevent_default_event_handling = true;
        }

        win
    };

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(primary_window),
        ..default()
    }))
    .insert_resource(WinitSettings {
        // Continuous ensures the game loop runs every frame (driven by
        // requestAnimationFrame on WASM). reactive_low_power was previously used
        // here, but on WASM the timer-based wake could stall the event loop
        // and make the simulation appear frozen.
        focused_mode: UpdateMode::Continuous,
        unfocused_mode: UpdateMode::reactive_low_power(std::time::Duration::from_millis(100)),
    });

    // Start in Playing state so the simulation runs immediately.
    // A future main-menu feature will remove this and let the menu UI
    // trigger the transition from MainMenu to Playing.
    app.insert_state(simulation::AppState::Playing);

    app.add_plugins((
        simulation::SimulationPlugin,
        rendering::RenderingPlugin,
        ui::UiPlugin,
        save::SavePlugin,
    ));

    // Screenshot mode: takes preset screenshots and exits (native only)
    #[cfg(not(target_arch = "wasm32"))]
    if std::env::var("MEGACITY_SCREENSHOTS").is_ok() {
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
