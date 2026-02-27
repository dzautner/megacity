//! Structured outcomes for executed game actions.

use serde::{Deserialize, Serialize};

/// The outcome of executing a [`super::GameAction`].
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ActionResult {
    /// The action was applied successfully.
    Success,
    /// The action failed for a known reason.
    Error(ActionError),
}

/// Fine-grained error codes returned when an action cannot be applied.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActionError {
    /// A coordinate in the action falls outside the world grid.
    OutOfBounds,
    /// The city treasury cannot cover the cost of this action.
    InsufficientFunds,
    /// One or more target cells are occupied by a structure that prevents
    /// placement.
    BlockedByExistingStructure,
    /// The requested road segment has invalid geometry (e.g. zero length,
    /// non-axis-aligned when straight lines are required).
    InvalidRoadGeometry,
    /// Zones must be adjacent to at least one road cell.
    ZoneNotAdjacentToRoad,
    /// The requested feature is not yet unlocked (milestones, research, etc.).
    FeatureLocked,
    /// One or more parameters are semantically invalid (e.g. negative tax
    /// rate, unknown speed level).
    InvalidParameters,
    /// A save operation failed.
    SaveFailed(String),
    /// A load operation failed.
    LoadFailed(String),
}
