//! Per-zone-type procedural mesh generators.
//!
//! Each function returns a [`Mesh`] that represents the preview silhouette
//! for a particular [`ZoneType`].  All meshes are built in a 1×1 cell-size
//! coordinate system centered at the origin; the cursor-preview system
//! applies translation and scaling at runtime.

use bevy::prelude::*;

use simulation::config::CELL_SIZE;

use super::mesh_data::PreviewMeshData;

// ---------------------------------------------------------------------------
// Generators
// ---------------------------------------------------------------------------

/// Residential Low: a small suburban house with a pitched roof.
pub(crate) fn generate_residential_low() -> Mesh {
    let mut m = PreviewMeshData::new();
    let s = CELL_SIZE;
    let color = [0.55, 0.78, 0.55, 1.0]; // soft green tint

    // Main house body
    let hw = s * 0.35;
    let hh = s * 0.22;
    let hd = s * 0.30;
    m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);

    // Pitched roof
    let roof_color = [0.65, 0.35, 0.25, 1.0]; // brown/terra cotta
    m.add_roof_prism(
        0.0,
        hh * 2.0,
        0.0,
        hw * 1.05,
        hh * 0.6,
        hd * 1.05,
        roof_color,
    );

    m.into_mesh()
}

/// Residential Medium: a taller townhouse/duplex.
pub(crate) fn generate_residential_medium() -> Mesh {
    let mut m = PreviewMeshData::new();
    let s = CELL_SIZE;
    let color = [0.45, 0.72, 0.45, 1.0];

    // Taller, narrower body
    let hw = s * 0.30;
    let hh = s * 0.35;
    let hd = s * 0.32;
    m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);

    // Flat roof accent
    let roof_color = [0.50, 0.50, 0.50, 1.0];
    m.add_cuboid(
        0.0,
        hh * 2.0 + s * 0.02,
        0.0,
        hw * 0.9,
        s * 0.02,
        hd * 0.9,
        roof_color,
    );

    m.into_mesh()
}

/// Residential High: a tall apartment tower.
pub(crate) fn generate_residential_high() -> Mesh {
    let mut m = PreviewMeshData::new();
    let s = CELL_SIZE;
    let color = [0.40, 0.68, 0.40, 1.0];

    // Tall tower
    let hw = s * 0.28;
    let hh = s * 0.55;
    let hd = s * 0.28;
    m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);

    // Setback upper portion (stepped silhouette)
    let upper_color = [0.45, 0.72, 0.45, 1.0];
    let upper_hw = hw * 0.75;
    let upper_hh = s * 0.15;
    m.add_cuboid(
        0.0,
        hh * 2.0 + upper_hh,
        0.0,
        upper_hw,
        upper_hh,
        upper_hw,
        upper_color,
    );

    m.into_mesh()
}

/// Commercial Low: a medium-height shop building.
pub(crate) fn generate_commercial_low() -> Mesh {
    let mut m = PreviewMeshData::new();
    let s = CELL_SIZE;
    let color = [0.45, 0.50, 0.82, 1.0]; // blue tint

    // Main body
    let hw = s * 0.38;
    let hh = s * 0.30;
    let hd = s * 0.35;
    m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);

    // Storefront awning
    let awning_color = [0.35, 0.40, 0.70, 1.0];
    m.add_cuboid(
        0.0,
        hh * 0.35,
        hd + s * 0.04,
        hw * 0.9,
        s * 0.015,
        s * 0.04,
        awning_color,
    );

    m.into_mesh()
}

/// Commercial High: a tall skyscraper-like tower.
pub(crate) fn generate_commercial_high() -> Mesh {
    let mut m = PreviewMeshData::new();
    let s = CELL_SIZE;
    let color = [0.40, 0.45, 0.78, 1.0];

    // Base podium
    let base_hw = s * 0.38;
    let base_hh = s * 0.15;
    let base_hd = s * 0.38;
    m.add_cuboid(0.0, base_hh, 0.0, base_hw, base_hh, base_hd, color);

    // Tower portion (narrower, much taller)
    let tower_color = [0.50, 0.55, 0.85, 1.0];
    let tower_hw = s * 0.25;
    let tower_hh = s * 0.50;
    m.add_cuboid(
        0.0,
        base_hh * 2.0 + tower_hh,
        0.0,
        tower_hw,
        tower_hh,
        tower_hw,
        tower_color,
    );

    m.into_mesh()
}

/// Industrial: a wide, low warehouse/factory.
pub(crate) fn generate_industrial() -> Mesh {
    let mut m = PreviewMeshData::new();
    let s = CELL_SIZE;
    let color = [0.78, 0.72, 0.35, 1.0]; // yellow tint

    // Wide, low main body
    let hw = s * 0.42;
    let hh = s * 0.18;
    let hd = s * 0.38;
    m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);

    // Smokestack / vent
    let stack_color = [0.55, 0.50, 0.30, 1.0];
    m.add_cuboid(
        hw * 0.7,
        hh * 2.0 + s * 0.12,
        -hd * 0.6,
        s * 0.035,
        s * 0.12,
        s * 0.035,
        stack_color,
    );

    // Pitched roof over main body (sawtooth factory look)
    let roof_color = [0.60, 0.55, 0.30, 1.0];
    m.add_roof_prism(
        0.0,
        hh * 2.0,
        0.0,
        hw * 1.02,
        hh * 0.35,
        hd * 1.02,
        roof_color,
    );

    m.into_mesh()
}

/// Office: a tall glass tower silhouette.
pub(crate) fn generate_office() -> Mesh {
    let mut m = PreviewMeshData::new();
    let s = CELL_SIZE;
    let color = [0.58, 0.52, 0.80, 1.0]; // purple tint

    // Tall tower
    let hw = s * 0.28;
    let hh = s * 0.55;
    let hd = s * 0.28;
    m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);

    // Crown / spire at top
    let crown_color = [0.65, 0.60, 0.88, 1.0];
    let crown_hw = hw * 0.5;
    let crown_hh = s * 0.08;
    m.add_cuboid(
        0.0,
        hh * 2.0 + crown_hh,
        0.0,
        crown_hw,
        crown_hh,
        crown_hw,
        crown_color,
    );

    m.into_mesh()
}

/// MixedUse: a medium multi-story building.
pub(crate) fn generate_mixed_use() -> Mesh {
    let mut m = PreviewMeshData::new();
    let s = CELL_SIZE;

    // Ground floor commercial (blue)
    let comm_color = [0.45, 0.50, 0.72, 1.0];
    let hw = s * 0.35;
    let ground_hh = s * 0.14;
    let hd = s * 0.35;
    m.add_cuboid(0.0, ground_hh, 0.0, hw, ground_hh, hd, comm_color);

    // Upper residential floors (green, slightly narrower)
    let res_color = [0.50, 0.70, 0.45, 1.0];
    let upper_hw = hw * 0.92;
    let upper_hh = s * 0.28;
    m.add_cuboid(
        0.0,
        ground_hh * 2.0 + upper_hh,
        0.0,
        upper_hw,
        upper_hh,
        hd * 0.92,
        res_color,
    );

    m.into_mesh()
}
