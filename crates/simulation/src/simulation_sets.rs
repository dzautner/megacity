//! Deterministic simulation ordering via `SystemSet` phases.
//!
//! These sets establish a **contract** for system execution order within the
//! `FixedUpdate` and `Update` schedules.  Plugins place their systems into the
//! appropriate set so that inter-plugin ordering is explicit and testable rather
//! than relying on implicit timing assumptions.
//!
//! # Ordering rules (TEST-022 audit)
//!
//! Every system in `FixedUpdate` MUST be in one of these sets.  Systems that
//! write to a shared grid resource (e.g. `LandValueGrid`, `TrafficGrid`,
//! `PollutionGrid`, `NoisePollutionGrid`, `EnergyGrid`) MUST have an explicit
//! `.after()` on the primary system that computes that grid.  Systems that only
//! write to their own private resource are documented as order-independent.
//!
//! # FixedUpdate phases (`SimulationSet`)
//!
//! ```text
//! PreSim  →  Simulation  →  PostSim
//! ```
//!
//! * **PreSim** – Tick counters, game clock, zone demand, building/citizen
//!   spawning, job assignment.  These set up per-tick state that the core
//!   simulation reads.
//! * **Simulation** – The bulk of game logic: movement, traffic, happiness,
//!   economy, grid propagation (pollution, land-value, crime …), life
//!   simulation, utilities, weather effects, water/waste systems, production.
//! * **PostSim** – Aggregation and reporting: stats, chart snapshots, events,
//!   advisors, achievements, notifications.  These only *read* simulation
//!   state and never mutate it, so downstream systems (UI, rendering) can
//!   safely consume their output on the next frame.
//!
//! # Update phases (`SimulationUpdateSet`)
//!
//! ```text
//! Input  →  Visual
//! ```
//!
//! * **Input** – Per-frame input handling (keybindings, one-way toggles).
//! * **Visual** – Visual-only updates that don't affect simulation state (LOD,
//!   day/night, weather rendering, seasonal effects, tutorial UI).

use bevy::prelude::*;

// ---------------------------------------------------------------------------
// FixedUpdate phases
// ---------------------------------------------------------------------------

/// Ordered phases for systems running in the `FixedUpdate` schedule.
///
/// Configured as a chain: `PreSim` → `Simulation` → `PostSim`.
/// Individual plugins use `.in_set(SimulationSet::X)` when registering their
/// systems, which gives them automatic ordering relative to other phases
/// while retaining the ability to add fine-grained `.after()` / `.before()`
/// constraints within the same phase.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimulationSet {
    /// Pre-simulation setup: tick counters, game clock, zone demand,
    /// building spawning, citizen spawning, job assignment.
    PreSim,
    /// Core simulation: movement, traffic, happiness, economy, life
    /// events, grid propagation, weather, water/waste, production.
    Simulation,
    /// Post-simulation aggregation: stats, charts, events, advisors,
    /// achievements, specialization, notifications.
    PostSim,
}

// ---------------------------------------------------------------------------
// Update phases
// ---------------------------------------------------------------------------

/// Ordered phases for systems running in the `Update` schedule.
///
/// Configured as a chain: `Input` → `Visual`.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimulationUpdateSet {
    /// Per-frame input processing (keybindings, toggles).
    Input,
    /// Visual-only updates (LOD, day/night, weather rendering, tutorial).
    Visual,
}
