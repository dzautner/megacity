//! Replay-to-video frame capture: captures screenshots during replay playback.
//!
//! When `--replay <path> --record <output_dir>` are both passed, this module
//! captures every Nth rendered frame to disk as numbered PNGs. After the
//! replay finishes (all entries fed), it waits a few extra frames for rendering
//! to settle, then sends `AppExit::Success` to shut down cleanly.
//!
//! The resulting frame sequence can be stitched into a video via ffmpeg:
//! ```sh
//! ffmpeg -framerate 30 -i frame_%05d.png -c:v libx264 -pix_fmt yuv420p out.mp4
//! ```

#[cfg(not(target_arch = "wasm32"))]
use bevy::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use bevy::render::view::screenshot::{save_to_disk, Screenshot};

/// Resource that drives frame capture during replay recording.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Resource)]
pub struct ReplayRecordState {
    /// Directory where frame PNGs are written.
    pub output_dir: String,
    /// Total frames elapsed since app start.
    pub frame_count: u32,
    /// Next frame number for the output filename (1-based).
    pub capture_index: u32,
    /// Capture every Nth frame (default: 2).
    pub capture_interval: u32,
    /// Number of frames to wait before starting capture (let scene render).
    pub warmup_frames: u32,
    /// Set to `true` once the replay player reports finished.
    pub replay_done: bool,
    /// Frame at which we detected replay completion (for cooldown).
    pub done_at_frame: Option<u32>,
    /// Extra frames to render after replay ends before exiting.
    pub cooldown_frames: u32,
}

#[cfg(not(target_arch = "wasm32"))]
impl ReplayRecordState {
    pub fn new(output_dir: String) -> Self {
        Self {
            output_dir,
            frame_count: 0,
            capture_index: 0,
            capture_interval: 2,
            warmup_frames: 10,
            replay_done: false,
            done_at_frame: None,
            cooldown_frames: 30,
        }
    }
}

/// System that captures frames during replay recording and exits when done.
///
/// Runs every frame in `Update`. It:
/// 1. Waits `warmup_frames` for the initial render to settle.
/// 2. Captures a screenshot every `capture_interval` frames.
/// 3. Detects when the replay player has finished.
/// 4. Waits `cooldown_frames` after the replay ends, then sends `AppExit`.
#[cfg(not(target_arch = "wasm32"))]
pub fn drive_replay_recording(
    mut commands: Commands,
    mut state: ResMut<ReplayRecordState>,
    player: Res<simulation::replay::ReplayPlayer>,
    mut exit: EventWriter<AppExit>,
) {
    state.frame_count += 1;

    // Wait for initial rendering to settle.
    if state.frame_count < state.warmup_frames {
        return;
    }

    // Detect replay completion: cursor > 0 means it was loaded and started,
    // is_finished() means all entries have been consumed.
    if !state.replay_done && player.is_finished() && player.cursor() > 0 {
        state.replay_done = true;
        state.done_at_frame = Some(state.frame_count);
        info!(
            "Replay recording: replay entries exhausted at frame {}",
            state.frame_count
        );
    }

    // After cooldown, exit the application.
    if let Some(done_frame) = state.done_at_frame {
        if state.frame_count >= done_frame + state.cooldown_frames {
            info!(
                "Replay recording complete: {} frames captured to '{}'",
                state.capture_index, state.output_dir
            );
            exit.send(AppExit::Success);
            return;
        }
    }

    // Capture every Nth frame.
    let frames_since_warmup = state.frame_count - state.warmup_frames;
    if frames_since_warmup % state.capture_interval != 0 {
        return;
    }

    state.capture_index += 1;
    let path = format!("{}/frame_{:05}.png", state.output_dir, state.capture_index);
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path));
}
