//! Form-Based Transect Overlay System (ZONE-003).
//!
//! Implements form-based codes as an overlay on top of Euclidean zoning. The
//! transect (T1-T6) controls physical building form (height, FAR, lot coverage,
//! setbacks) independent of use. This allows players to say "I want medium-density
//! here" without specifying residential vs commercial.
//!
//! **TransectZone** enum: `None`, `T1Natural`, `T2Rural`, `T3Suburban`, `T4Urban`,
//! `T5Center`, `T6Core`. Each tier defines constraints on maximum stories, FAR,
//! lot coverage, and front setback.
//!
//! Transect data is stored in a separate `TransectGrid` resource (one entry per
//! cell), rather than modifying the `Cell` struct, to keep the overlay fully
//! decoupled from the base zoning system.
//!
//! **Key behaviours:**
//!
//! - `T1Natural` prevents all building spawning (natural preserve).
//! - Other tiers cap building level based on their FAR limit.
//! - `TransectZone::None` (the default) imposes no additional constraints,
//!   preserving backward compatibility.
//! - The `enforce_transect_constraints` system runs every slow tick and caps
//!   existing buildings that exceed their transect's FAR limit.
//!
//! The `TransectGrid` is registered with the `SaveableRegistry` for persistence.

mod grid;
mod systems;
mod types;

pub use grid::{max_level_for_transect, TransectGrid};
pub use systems::{enforce_transect_constraints, FormTransectPlugin};
pub use types::TransectZone;
