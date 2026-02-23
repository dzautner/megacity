use bevy::prelude::*;

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Marker for scaffolding mesh entities attached to buildings under construction.
#[derive(Component)]
pub struct ScaffoldingMesh {
    /// The simulation entity (Building) this scaffolding tracks.
    pub tracked_entity: Entity,
}

/// Marker for crane prop entities attached to buildings under construction.
#[derive(Component)]
pub struct CraneProp {
    /// The simulation entity (Building) this crane tracks.
    pub tracked_entity: Entity,
}

// ---------------------------------------------------------------------------
// Shared assets (lazy-initialised)
// ---------------------------------------------------------------------------

/// Cached mesh and material handles for construction visuals, created once on
/// first use and reused for all construction sites.
#[derive(Resource, Clone)]
pub struct ConstructionAssets {
    pub scaffold_mesh: Handle<Mesh>,
    pub scaffold_material: Handle<StandardMaterial>,
    pub crane_base_mesh: Handle<Mesh>,
    pub crane_arm_mesh: Handle<Mesh>,
    pub crane_material: Handle<StandardMaterial>,
}
