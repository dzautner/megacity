//! The canonical `GameAction` enum — every gameplay-affecting operation the
//! player, AI agent, or replay system can perform.

use serde::{Deserialize, Serialize};

use crate::grid::{RoadType, ZoneType};
use crate::services::ServiceType;
use crate::utilities::UtilityType;

/// A single, atomic game action.
///
/// Every mutation of simulation state should eventually flow through this
/// enum so that actions can be recorded, replayed, and validated uniformly.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GameAction {
    // ── Session ──────────────────────────────────────────────────────────
    /// Start a new game with the given random seed.
    NewGame { seed: u64 },

    // ── Simulation control ──────────────────────────────────────────────
    /// Pause or unpause the simulation.
    SetPaused { paused: bool },

    /// Set the simulation speed multiplier (1 = normal, 2 = fast, 3 = fastest).
    SetSpeed { speed: u32 },

    // ── Road building ───────────────────────────────────────────────────
    /// Place a straight road line between two grid cells (inclusive).
    PlaceRoadLine {
        start: (u32, u32),
        end: (u32, u32),
        road_type: RoadType,
    },

    // ── Zoning ──────────────────────────────────────────────────────────
    /// Paint a zone over an axis-aligned rectangle of cells.
    ZoneRect {
        min: (u32, u32),
        max: (u32, u32),
        zone_type: ZoneType,
    },

    // ── Infrastructure placement ────────────────────────────────────────
    /// Place a utility building (power plant, water tower, etc.).
    PlaceUtility {
        pos: (u32, u32),
        utility_type: UtilityType,
    },

    /// Place a service building (fire station, school, park, etc.).
    PlaceService {
        pos: (u32, u32),
        service_type: ServiceType,
    },

    // ── Demolition ──────────────────────────────────────────────────────
    /// Bulldoze everything inside an axis-aligned rectangle.
    BulldozeRect { min: (u32, u32), max: (u32, u32) },

    // ── Economy ─────────────────────────────────────────────────────────
    /// Set per-zone tax rates (values are fractions, e.g. 0.09 = 9 %).
    SetTaxRates {
        residential: f32,
        commercial: f32,
        industrial: f32,
        office: f32,
    },

    // ── Persistence ─────────────────────────────────────────────────────
    /// Save the current game to disk.
    Save { path: String },

    /// Load a saved game from disk.
    Load { path: String },
}
