use bevy::pbr::{DistanceFog, FogFalloff};
use bevy::prelude::*;
use simulation::day_night_controls::DayNightControls;
use simulation::fog::FogState;
use std::f32::consts::PI;

/// Updates the directional light (sun), its transform, and the ambient light
/// based on the current game hour to create a day/night cycle.
///
/// Uses `DayNightControls::effective_hour()` so that time-lock and cycle-speed
/// settings are respected.
pub fn update_day_night_cycle(
    controls: Res<DayNightControls>,
    mut sun_query: Query<&mut DirectionalLight>,
    mut sun_transform_query: Query<&mut Transform, With<DirectionalLight>>,
    mut ambient: ResMut<AmbientLight>,
) {
    let hour = controls.effective_hour();

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
            -elevation_angle, // pitch (positive elevation_angle = higher sun = more negative X rotation)
            azimuth,          // yaw
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
    if (5.0..7.0).contains(&hour) {
        // Dawn: 5:00 - 7:00
        let t = (hour - 5.0) / 2.0; // 0.0 at 5:00, 1.0 at 7:00
        let illuminance = lerp(1000.0, 10000.0, t);
        let color = color_lerp(
            Color::srgb(1.0, 0.6, 0.3),  // warm orange
            Color::srgb(1.0, 0.95, 0.9), // warm white
            t,
        );
        (illuminance, color)
    } else if (7.0..17.0).contains(&hour) {
        // Day: 7:00 - 17:00
        (10000.0, Color::srgb(1.0, 0.95, 0.9))
    } else if (17.0..19.0).contains(&hour) {
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
    if (5.0..7.0).contains(&hour) {
        // Dawn transition
        let t = (hour - 5.0) / 2.0;
        let brightness = lerp(50.0, 300.0, t);
        let color = color_lerp(
            Color::srgb(0.4, 0.45, 0.7), // night blue
            Color::srgb(0.9, 0.9, 1.0),  // warm white
            t,
        );
        (brightness, color)
    } else if (7.0..17.0).contains(&hour) {
        // Day
        (300.0, Color::srgb(0.9, 0.9, 1.0))
    } else if (17.0..19.0).contains(&hour) {
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

/// Updates distance fog on the camera based on the current fog state.
///
/// When fog is active, adds a `DistanceFog` component to the camera with
/// exponential falloff tuned to the visibility distance. When fog clears,
/// removes the component.
pub fn update_fog_rendering(
    fog: Res<FogState>,
    mut commands: Commands,
    cameras: Query<(Entity, Option<&DistanceFog>), With<Camera3d>>,
) {
    for (entity, existing_fog) in &cameras {
        if fog.active {
            // Fog color: grey-white during day, blue-grey at night
            let fog_color = Color::srgba(0.85, 0.87, 0.90, 1.0);

            // Exponential fog density inversely proportional to visibility
            // density = 3.0 / visibility_m gives good visual results:
            //   Dense (100m vis) -> density 0.03 (thick)
            //   Moderate (500m vis) -> density 0.006
            //   Mist (3000m vis) -> density 0.001
            let density = (3.0 / fog.visibility_m).clamp(0.0005, 0.05);

            let distance_fog = DistanceFog {
                color: fog_color,
                falloff: FogFalloff::Exponential { density },
                ..default()
            };

            if existing_fog.is_some() {
                // Update existing fog
                commands.entity(entity).insert(distance_fog);
            } else {
                // Add fog component
                commands.entity(entity).insert(distance_fog);
            }
        } else if existing_fog.is_some() {
            // Fog cleared: remove the component
            commands.entity(entity).remove::<DistanceFog>();
        }
    }
}
