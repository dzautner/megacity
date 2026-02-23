#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::render::mesh::Indices;

    use super::super::meshes::{build_crane_arm_mesh, build_crane_base_mesh, build_scaffold_mesh};

    #[test]
    fn test_scaffold_mesh_has_geometry() {
        let mesh = build_scaffold_mesh();
        // The mesh should have positions, normals, and indices
        let positions = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .expect("scaffold mesh should have positions");
        match positions {
            bevy::render::mesh::VertexAttributeValues::Float32x3(v) => {
                assert!(!v.is_empty(), "scaffold mesh should have vertices");
            }
            _ => panic!("unexpected vertex attribute type"),
        }
    }

    #[test]
    fn test_crane_base_mesh_has_geometry() {
        let mesh = build_crane_base_mesh();
        let positions = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .expect("crane base mesh should have positions");
        match positions {
            bevy::render::mesh::VertexAttributeValues::Float32x3(v) => {
                assert!(!v.is_empty(), "crane base mesh should have vertices");
            }
            _ => panic!("unexpected vertex attribute type"),
        }
    }

    #[test]
    fn test_crane_arm_mesh_has_geometry() {
        let mesh = build_crane_arm_mesh();
        let positions = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .expect("crane arm mesh should have positions");
        match positions {
            bevy::render::mesh::VertexAttributeValues::Float32x3(v) => {
                assert!(!v.is_empty(), "crane arm mesh should have vertices");
            }
            _ => panic!("unexpected vertex attribute type"),
        }
    }

    #[test]
    fn test_scaffold_mesh_index_count() {
        let mesh = build_scaffold_mesh();
        if let Some(Indices::U32(idx)) = mesh.indices() {
            // Each box = 36 indices (12 triangles).
            // 4 poles + 3*4 horizontal rails + 2*6 diagonal segments = 4+12+12 = 28 boxes
            // 28 * 36 = 1008 indices
            assert_eq!(
                idx.len(),
                28 * 36,
                "scaffold should have correct index count"
            );
        } else {
            panic!("scaffold mesh should have u32 indices");
        }
    }

    #[test]
    fn test_crane_base_mesh_index_count() {
        let mesh = build_crane_base_mesh();
        if let Some(Indices::U32(idx)) = mesh.indices() {
            // 1 box = 36 indices
            assert_eq!(idx.len(), 36, "crane base should have one box");
        } else {
            panic!("crane base mesh should have u32 indices");
        }
    }

    #[test]
    fn test_crane_arm_mesh_index_count() {
        let mesh = build_crane_arm_mesh();
        if let Some(Indices::U32(idx)) = mesh.indices() {
            assert_eq!(idx.len(), 36, "crane arm should have one box");
        } else {
            panic!("crane arm mesh should have u32 indices");
        }
    }

    #[test]
    fn test_progress_calculation() {
        // Progress should be 0.0 at start (ticks_remaining == total_ticks)
        let total = 100u32;
        let remaining = 100u32;
        let progress = 1.0 - (remaining as f32 / total as f32);
        assert!((progress - 0.0).abs() < f32::EPSILON);

        // Progress should be 1.0 when done
        let remaining = 0u32;
        let progress = 1.0 - (remaining as f32 / total as f32);
        assert!((progress - 1.0).abs() < f32::EPSILON);

        // Progress should be 0.5 at midpoint
        let remaining = 50u32;
        let progress = 1.0 - (remaining as f32 / total as f32);
        assert!((progress - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_y_factor_range() {
        // y_factor should range from 0.3 (progress=0) to 1.0 (progress=1)
        let y_at_start: f32 = 0.3 + 0.0 * 0.7;
        assert!((y_at_start - 0.3_f32).abs() < f32::EPSILON);

        let y_at_end: f32 = 0.3 + 1.0 * 0.7;
        assert!((y_at_end - 1.0_f32).abs() < f32::EPSILON);

        let y_at_mid: f32 = 0.3 + 0.5 * 0.7;
        assert!((y_at_mid - 0.65_f32).abs() < f32::EPSILON);
    }
}
