use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActionResult {
    Success,
    Error(ActionError),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActionError {
    OutOfBounds,
    InsufficientFunds,
    BlockedByWater,
    BlockedByBuilding,
    InvalidRoadGeometry,
    ZoneNotAdjacentToRoad,
    FeatureLocked,
    DependencyMissing,
    AlreadyExists,
    NotSupported,
    InternalError,
    NotFound,
    InvalidParameter,
}
