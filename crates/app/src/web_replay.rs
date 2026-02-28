//! WASM replay loading from URL query parameter.
//!
//! Usage:
//! `index.html?replay=assets/replays/example.replay`

#![cfg(target_arch = "wasm32")]

use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use simulation::replay::{ReplayPlayer, ReplayViewerInfo};
use simulation::time_of_day::GameClock;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

/// Query parameter source for replay JSON.
#[derive(Resource)]
pub struct WebReplaySource(pub String);

/// Shared slot used to bridge async fetch -> ECS world.
#[derive(Resource, Default, Clone)]
pub struct WebReplayLoadBuffer(pub Arc<Mutex<Option<Result<String, String>>>>);

/// Returns `Some(url)` if `?replay=...` is present in the browser URL.
pub fn query_replay_url() -> Option<String> {
    let window = web_sys::window()?;
    let search = window.location().search().ok()?;
    let params = web_sys::UrlSearchParams::new_with_str(&search).ok()?;
    let replay = params.get("replay")?;
    if replay.trim().is_empty() {
        None
    } else {
        Some(replay)
    }
}

/// Startup system: fetch replay JSON text from the query-param URL.
pub fn begin_web_replay_load(source: Res<WebReplaySource>, buffer: Res<WebReplayLoadBuffer>) {
    let url = source.0.clone();
    let slot = buffer.0.clone();

    wasm_bindgen_futures::spawn_local(async move {
        let result = fetch_text(&url).await;
        if let Ok(mut guard) = slot.lock() {
            *guard = Some(result);
        }
    });
}

/// Poll fetch result, parse replay JSON, and start playback.
pub fn poll_web_replay_load(
    source: Res<WebReplaySource>,
    buffer: Res<WebReplayLoadBuffer>,
    mut player: ResMut<ReplayPlayer>,
    mut clock: ResMut<GameClock>,
    mut commands: Commands,
) {
    let Ok(mut slot) = buffer.0.lock() else {
        return;
    };
    let Some(result) = slot.take() else {
        return;
    };

    match result {
        Ok(contents) => {
            let replay = match simulation::replay::ReplayFile::from_json(&contents) {
                Ok(r) => r,
                Err(e) => {
                    error!("Failed to parse web replay '{}': {}", source.0, e);
                    return;
                }
            };
            if let Err(e) = replay.validate() {
                warn!("Replay validation warning for '{}': {}", source.0, e);
            }
            info!(
                "Web replay loaded: {} entries, ticks {}..{}",
                replay.entries.len(),
                replay.header.start_tick,
                replay.footer.end_tick,
            );

            commands.insert_resource(ReplayViewerInfo {
                source: source.0.clone(),
                start_tick: replay.header.start_tick,
                end_tick: replay.footer.end_tick,
                entry_count: replay.entries.len() as u64,
            });
            player.load(replay);
            clock.paused = false;
            clock.speed = 1.0;
        }
        Err(e) => {
            error!("Failed to fetch replay '{}': {}", source.0, e);
        }
    }
}

async fn fetch_text(url: &str) -> Result<String, String> {
    let window = web_sys::window().ok_or_else(|| "window not available".to_string())?;
    let response_value = JsFuture::from(window.fetch_with_str(url))
        .await
        .map_err(|e| format!("fetch failed: {:?}", e))?;

    let response: web_sys::Response = response_value
        .dyn_into()
        .map_err(|_| "failed to cast fetch response".to_string())?;

    if !response.ok() {
        return Err(format!("HTTP {} while fetching {}", response.status(), url));
    }

    let text_promise = response
        .text()
        .map_err(|e| format!("response.text() failed: {:?}", e))?;
    let text_value = JsFuture::from(text_promise)
        .await
        .map_err(|e| format!("await response text failed: {:?}", e))?;
    text_value
        .as_string()
        .ok_or_else(|| "response text was not a string".to_string())
}
