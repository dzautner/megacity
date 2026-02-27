//! Canonical gameplay action types, error codes, and action queue.
//!
//! This module defines the `GameAction` enum representing every player or
//! agent action, `ActionResult` / `ActionError` for structured outcomes,
//! and `ActionQueue` for enqueuing actions from any source.

mod actions;
mod queue;
mod results;

pub use actions::*;
pub use queue::*;
pub use results::*;
