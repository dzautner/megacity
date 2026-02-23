//! Construction animation visuals: scaffolding, cranes, and progress tracking.

mod meshes;
mod tests;
pub mod types;

mod systems;

pub use systems::{
    animate_crane_rotation, cleanup_construction_props, cleanup_orphan_construction_props,
    spawn_construction_props, update_construction_anim,
};
pub use types::{ConstructionAssets, CraneProp, ScaffoldingMesh};
