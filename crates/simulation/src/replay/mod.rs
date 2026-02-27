//! Deterministic replay format with recorder and player.
//!
//! Operates at the `GameAction` level — records player/agent actions by tick
//! and replays them through the same `ActionQueue` executor path.

pub mod format;
pub mod player;
pub mod plugin;
pub mod recorder;

pub use format::{ReplayEntry, ReplayFile, ReplayFooter, ReplayHeader};
pub use player::ReplayPlayer;
pub use plugin::ReplayPlugin;
pub use recorder::ReplayRecorder;
