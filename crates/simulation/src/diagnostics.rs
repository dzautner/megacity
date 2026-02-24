//! Bevy diagnostics and tracing integration for development profiling.
//!
//! # Debug-build diagnostics
//!
//! In debug builds (`cfg(debug_assertions)`), this plugin automatically adds:
//! - `FrameTimeDiagnosticsPlugin` — tracks frame time, FPS, and frame count
//! - `EntityCountDiagnosticsPlugin` — tracks total ECS entity count
//!
//! These metrics are available via Bevy's `DiagnosticsStore` resource and can
//! be displayed in UI overlays or logged to the console.
//!
//! # Tracy integration
//!
//! Critical simulation systems are instrumented with `bevy::log::info_span!`
//! trace spans that appear as regions in Tracy or any `tracing`-compatible
//! profiler. To enable Tracy capture:
//!
//! 1. Add the `trace_tracy` feature to Bevy in your workspace Cargo.toml:
//!    ```toml
//!    bevy = { version = "0.15", features = ["trace_tracy"] }
//!    ```
//! 2. Build with the `trace` feature flag:
//!    ```sh
//!    cargo run --features trace
//!    ```
//! 3. Open the Tracy profiler (<https://github.com/wolfpld/tracy>) and connect
//!    to the running application. The following spans will appear:
//!    - `update_happiness` — per-tick citizen happiness recalculation
//!    - `move_citizens` — per-tick citizen movement along paths
//!    - `building_spawner` — zone-demand-driven building placement
//!    - `update_traffic` — traffic density grid recalculation
//!
//! Without the `trace` feature, the `info_span!` calls compile to no-ops and
//! have zero runtime cost.

use bevy::prelude::*;

pub struct DiagnosticsPlugin;

impl Plugin for DiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        // Add diagnostic plugins only in debug builds to avoid overhead in release.
        #[cfg(debug_assertions)]
        {
            app.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin);
            app.add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin);
        }
    }
}
