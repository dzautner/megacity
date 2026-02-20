use bevy::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
use bevy::render::view::screenshot::{save_to_disk, Screenshot};

use crate::input::StatusMessage;

pub struct ScreenshotPlugin;

impl Plugin for ScreenshotPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_screenshot_key);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn handle_screenshot_key(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut status: ResMut<StatusMessage>,
    bindings: Res<simulation::keybindings::KeyBindings>,
) {
    if bindings.screenshot.just_pressed(&keyboard) {
        // Create screenshots directory if it doesn't exist
        let dir = "screenshots";
        if std::fs::create_dir_all(dir).is_err() {
            status.set("Failed to create screenshots directory", true);
            return;
        }

        // Generate timestamp filename
        let now = std::time::SystemTime::now();
        let since_epoch = now
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        let secs = since_epoch.as_secs();

        // Convert epoch seconds to a human-readable timestamp
        // (manual conversion to avoid adding chrono dependency)
        let (year, month, day, hour, minute, second) = epoch_to_datetime(secs);
        let filename = format!(
            "{}/screenshot_{:04}-{:02}-{:02}_{:02}-{:02}-{:02}.png",
            dir, year, month, day, hour, minute, second
        );

        let display_name = filename.clone();
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(filename));

        status.set(format!("Screenshot saved: {}", display_name), false);
    }
}

#[cfg(target_arch = "wasm32")]
fn handle_screenshot_key(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut status: ResMut<StatusMessage>,
    bindings: Res<simulation::keybindings::KeyBindings>,
) {
    if bindings.screenshot.just_pressed(&keyboard) {
        status.set("Screenshots not supported in browser", true);
    }
}

/// Convert Unix epoch seconds to (year, month, day, hour, minute, second) in UTC.
#[cfg(not(target_arch = "wasm32"))]
fn epoch_to_datetime(epoch: u64) -> (u64, u64, u64, u64, u64, u64) {
    let second = epoch % 60;
    let minute = (epoch / 60) % 60;
    let hour = (epoch / 3600) % 24;

    // Days since epoch
    let mut days = epoch / 86400;

    // Calculate year
    let mut year = 1970u64;
    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }

    // Calculate month and day
    let days_in_months: [u64; 12] = if is_leap_year(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 0u64;
    for (i, &dim) in days_in_months.iter().enumerate() {
        if days < dim {
            month = i as u64 + 1;
            break;
        }
        days -= dim;
    }
    let day = days + 1;

    (year, month, day, hour, minute, second)
}

#[cfg(not(target_arch = "wasm32"))]
fn is_leap_year(year: u64) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}
