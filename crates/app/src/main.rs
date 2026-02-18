use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use bevy::window::PresentMode;
use bevy::winit::{UpdateMode, WinitSettings};

use rendering::camera::OrbitCamera;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "MegaCity".to_string(),
            resolution: (1280.0, 720.0).into(),
            present_mode: PresentMode::AutoVsync,
            ..default()
        }),
        ..default()
    }))
    .insert_resource(WinitSettings {
        focused_mode: UpdateMode::reactive_low_power(std::time::Duration::from_millis(16)),
        unfocused_mode: UpdateMode::reactive_low_power(std::time::Duration::from_millis(100)),
    })
    .add_plugins((
        simulation::SimulationPlugin,
        rendering::RenderingPlugin,
        ui::UiPlugin,
        save::SavePlugin,
    ));

    // Screenshot mode: takes preset screenshots and exits
    if std::env::var("MEGACITY_SCREENSHOTS").is_ok() {
        let world_cx = 128.0 * 16.0; // center of 256x256 grid
        let world_cz = 128.0 * 16.0;
        let center = Vec3::new(world_cx, 0.0, world_cz);

        // Tel Aviv camera presets (grid center = 128,128 → world 2048,2048)
        // Coast at ~x=55 grid → ~880 world. Jaffa at ~(58,48) grid → ~(936,776) world.
        let jaffa = Vec3::new(58.0 * 16.0, 0.0, 48.0 * 16.0);
        let white_city = Vec3::new(100.0 * 16.0, 0.0, 100.0 * 16.0);
        let coast_mid = Vec3::new(63.0 * 16.0, 0.0, 120.0 * 16.0);
        let ayalon = Vec3::new(185.0 * 16.0, 0.0, 100.0 * 16.0);
        let ramat_aviv = Vec3::new(110.0 * 16.0, 0.0, 210.0 * 16.0);

        app.insert_resource(ScreenshotQueue {
            frame: 0,
            current: 0,
            presets: vec![
                ShotPreset { name: "01_overview", focus: center, yaw: 0.0, pitch: 60f32.to_radians(), distance: 2500.0 },
                ShotPreset { name: "02_jaffa", focus: jaffa, yaw: 0.5, pitch: 40f32.to_radians(), distance: 300.0 },
                ShotPreset { name: "03_coast", focus: coast_mid, yaw: 0.8, pitch: 35f32.to_radians(), distance: 300.0 },
                ShotPreset { name: "04_white_city", focus: white_city, yaw: 0.0, pitch: 40f32.to_radians(), distance: 350.0 },
                ShotPreset { name: "05_white_city_close", focus: white_city, yaw: 0.2, pitch: 30f32.to_radians(), distance: 150.0 },
                ShotPreset { name: "06_ayalon_hwy", focus: ayalon, yaw: -0.3, pitch: 35f32.to_radians(), distance: 400.0 },
                ShotPreset { name: "07_ramat_aviv", focus: ramat_aviv, yaw: 0.0, pitch: 40f32.to_radians(), distance: 400.0 },
                ShotPreset { name: "08_downtown_tight", focus: white_city + Vec3::new(200.0, 0.0, 100.0), yaw: -0.2, pitch: 28f32.to_radians(), distance: 100.0 },
            ],
        });
        app.add_systems(Update, drive_screenshots);
    }

    app.run();
}

#[derive(Resource)]
struct ScreenshotQueue {
    frame: u32,
    current: usize,
    presets: Vec<ShotPreset>,
}

struct ShotPreset {
    name: &'static str,
    focus: Vec3,
    yaw: f32,
    pitch: f32,
    distance: f32,
}

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
