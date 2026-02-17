use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy::winit::{UpdateMode, WinitSettings};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
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
        ))
        .run();
}
