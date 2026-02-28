//! Replay frame capture for replay-to-video workflows.
//!
//! Enabled by launching the app with:
//! `--replay <path> --record <output_dir>`
//!
//! The system captures numbered PNGs and exits automatically after playback.

use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};

/// Capture target used by replay recording.
#[derive(Resource, Clone)]
pub struct ReplayCaptureTarget(pub Handle<Image>);

/// Runtime state for replay frame capture.
#[derive(Resource)]
pub struct ReplayRecordState {
    /// Directory where `frame_00001.png` etc are written.
    pub output_dir: String,
    /// Total rendered frames since startup.
    pub frame_count: u32,
    /// Captured frame counter (1-based in filenames).
    pub capture_index: u32,
    /// Capture every Nth rendered frame.
    pub capture_interval: u32,
    /// Startup warmup to let first frames settle.
    pub warmup_frames: u32,
    /// Extra frames after replay completion before exit.
    pub cooldown_frames: u32,
    /// If replay never starts, auto-exit after this many frames.
    pub start_timeout_frames: u32,
    /// Frame number when replay completion was first observed.
    pub done_at_frame: Option<u32>,
}

impl ReplayRecordState {
    pub fn new(output_dir: String) -> Self {
        Self {
            output_dir,
            frame_count: 0,
            capture_index: 0,
            capture_interval: 2,
            warmup_frames: 10,
            cooldown_frames: 30,
            start_timeout_frames: 600,
            done_at_frame: None,
        }
    }
}

/// Capture replay frames and exit after replay completion or startup timeout.
pub fn drive_replay_recording(
    mut commands: Commands,
    mut state: ResMut<ReplayRecordState>,
    capture_target: Option<Res<ReplayCaptureTarget>>,
    player: Res<simulation::replay::ReplayPlayer>,
    mut exit: EventWriter<AppExit>,
) {
    state.frame_count += 1;

    if state.frame_count < state.warmup_frames {
        return;
    }

    if state.capture_interval == 0 {
        warn!("Replay recording capture_interval was 0; forcing to 1");
        state.capture_interval = 1;
    }

    // If replay never starts (e.g. bad path/parse), don't hang forever.
    if !player.is_playing() {
        if state.frame_count >= state.warmup_frames + state.start_timeout_frames {
            warn!(
                "Replay recording timed out waiting for playback start after {} frames",
                state.frame_count
            );
            exit.send(AppExit::Success);
        }
        return;
    }

    if state.done_at_frame.is_none() && player.is_finished() {
        state.done_at_frame = Some(state.frame_count);
        info!(
            "Replay recording: playback finished at frame {}",
            state.frame_count
        );
    }

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

    let frames_since_warmup = state.frame_count - state.warmup_frames;
    if !frames_since_warmup.is_multiple_of(state.capture_interval) {
        return;
    }

    state.capture_index += 1;
    let path = format!("{}/frame_{:05}.png", state.output_dir, state.capture_index);
    let screenshot = match capture_target {
        Some(target) => Screenshot::image(target.0.clone()),
        None => Screenshot::primary_window(),
    };
    commands.spawn(screenshot).observe(save_to_disk(path));
}
