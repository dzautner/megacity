use bevy::prelude::*;
use simulation::time_of_day::GameClock;
use std::f32::consts::PI;

/// Updates the directional light (sun), its transform, and the ambient light
/// based on the current game hour to create a day/night cycle.
pub fn update_day_night_cycle(
    clock: Res<GameClock>,
    mut sun_query: Query<&mut DirectionalLight>,
    mut sun_transform_query: Query<&mut Transform, With<DirectionalLight>>,
    mut ambient: ResMut<AmbientLight>,
) {
    let hour = clock.hour;

    // --- Sun illuminance and color ---
    let (sun_illuminance, sun_color) = sun_light_for_hour(hour);

    for mut sun in sun_query.iter_mut() {
        sun.illuminance = sun_illuminance;
        sun.color = sun_color;
    }

    // --- Sun rotation (elevation + azimuth from hour) ---
    //
    // Map the 24-hour cycle so that:
    //   - At hour 6 (sunrise), the sun is at the horizon (elevation ~0)
    //   - At hour 12 (noon), the sun is at maximum elevation
    //   - At hour 18 (sunset), the sun is back at the horizon
    //   - Between 18 and 6, the sun is below the horizon
    //
    // Sun angle: hour * (PI / 12) gives a full PI rotation over 12 hours.
    // We offset so that hour=6 maps to elevation=0 and hour=12 maps to elevation=PI/2.
    //
    // Elevation: use sin to get a smooth arc.
    // At hour 6: sin((6 - 6) * PI/12) = sin(0) = 0 (horizon)
    // At hour 12: sin((12 - 6) * PI/12) = sin(PI/2) = 1 (zenith)
    // At hour 18: sin((18 - 6) * PI/12) = sin(PI) = 0 (horizon)
    // At hour 0/24: sin((24 - 6) * PI/12) = sin(3PI/2) = -1 (nadir)
    let elevation = ((hour - 6.0) * PI / 12.0).sin();
    // Clamp elevation angle: map [-1, 1] to [-PI/2, PI/2]
    let elevation_angle = elevation * (PI / 2.0);

    // Azimuth: the sun moves east to west. Use a simple linear sweep.
    // At hour 6: east (PI/3), at 12: south (0), at 18: west (-PI/3)
    let azimuth = PI / 3.0 - (hour - 6.0) * (PI / 18.0);

    for mut transform in sun_transform_query.iter_mut() {
        // Build rotation: first pitch down by elevation, then rotate around Y for azimuth.
        // Negative elevation_angle because Bevy's directional light points in -Z,
        // so a negative X rotation tilts the light downward.
        *transform = Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -elevation_angle,   // pitch (positive elevation_angle = higher sun = more negative X rotation)
            azimuth,            // yaw
            0.0,
        ));
    }

    // --- Ambient light ---
    let (ambient_brightness, ambient_color) = ambient_light_for_hour(hour);
    ambient.brightness = ambient_brightness;
    ambient.color = ambient_color;
}

/// Compute sun illuminance and color for a given hour.
fn sun_light_for_hour(hour: f32) -> (f32, Color) {
    if hour >= 5.0 && hour < 7.0 {
        // Dawn: 5:00 - 7:00
        let t = (hour - 5.0) / 2.0; // 0.0 at 5:00, 1.0 at 7:00
        let illuminance = lerp(1000.0, 10000.0, t);
        let color = color_lerp(
            Color::srgb(1.0, 0.6, 0.3),  // warm orange
            Color::srgb(1.0, 0.95, 0.9), // warm white
            t,
        );
        (illuminance, color)
    } else if hour >= 7.0 && hour < 17.0 {
        // Day: 7:00 - 17:00
        (10000.0, Color::srgb(1.0, 0.95, 0.9))
    } else if hour >= 17.0 && hour < 19.0 {
        // Dusk: 17:00 - 19:00
        let t = (hour - 17.0) / 2.0; // 0.0 at 17:00, 1.0 at 19:00
        let illuminance = lerp(10000.0, 1000.0, t);
        let color = color_lerp(
            Color::srgb(1.0, 0.95, 0.9), // warm white
            Color::srgb(1.0, 0.6, 0.3),  // warm orange
            t,
        );
        (illuminance, color)
    } else {
        // Night: 19:00 - 5:00
        (500.0, Color::srgb(0.5, 0.55, 0.8)) // blue-ish moonlight
    }
}

/// Compute ambient light brightness and color for a given hour.
fn ambient_light_for_hour(hour: f32) -> (f32, Color) {
    if hour >= 5.0 && hour < 7.0 {
        // Dawn transition
        let t = (hour - 5.0) / 2.0;
        let brightness = lerp(50.0, 300.0, t);
        let color = color_lerp(
            Color::srgb(0.4, 0.45, 0.7), // night blue
            Color::srgb(0.9, 0.9, 1.0),  // warm white
            t,
        );
        (brightness, color)
    } else if hour >= 7.0 && hour < 17.0 {
        // Day
        (300.0, Color::srgb(0.9, 0.9, 1.0))
    } else if hour >= 17.0 && hour < 19.0 {
        // Dusk transition
        let t = (hour - 17.0) / 2.0;
        let brightness = lerp(300.0, 50.0, t);
        let color = color_lerp(
            Color::srgb(0.9, 0.9, 1.0),  // warm white
            Color::srgb(0.4, 0.45, 0.7), // night blue
            t,
        );
        (brightness, color)
    } else {
        // Night
        (50.0, Color::srgb(0.4, 0.45, 0.7))
    }
}

/// Linear interpolation between two f32 values.
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Linear interpolation between two sRGB colors.
fn color_lerp(a: Color, b: Color, t: f32) -> Color {
    let a = a.to_srgba();
    let b = b.to_srgba();
    Color::srgb(
        lerp(a.red, b.red, t),
        lerp(a.green, b.green, t),
        lerp(a.blue, b.blue, t),
    )
}
