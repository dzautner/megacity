use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Encode, Decode)]
pub enum ActionResult {
    Success,
    /// The action succeeded but produced a warning the caller should see
    /// (e.g. zone overwrite information).
    SuccessWithWarning(String),
    Error(ActionError),
}

impl ActionResult {
    /// Returns `true` for both `Success` and `SuccessWithWarning`.
    pub fn is_success(&self) -> bool {
        matches!(self, ActionResult::Success | ActionResult::SuccessWithWarning(_))
    }

    /// Extract the warning string if present.
    pub fn warning(&self) -> Option<&str> {
        match self {
            ActionResult::SuccessWithWarning(w) => Some(w.as_str()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Encode, Decode)]
pub enum ActionError {
    OutOfBounds,
    InsufficientFunds,
    BlockedByWater,
    BlockedByRoad,
    BlockedByBuilding,
    InvalidRoadGeometry,
    ZoneNotAdjacentToRoad,
    FeatureLocked,
    DependencyMissing,
    AlreadyExists,
    NotSupported,
    InternalError,
    NotFound,
    InvalidParameter(String),
    NoCellsZoned,
}
