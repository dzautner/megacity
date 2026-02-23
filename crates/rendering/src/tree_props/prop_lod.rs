//! Prop LOD -- hides `TreeProp`, `StreetLamp`, and `ParkedCar` entities when the
//! camera distance exceeds configurable thresholds, reducing draw calls at wide
//! zoom levels.

use bevy::prelude::*;

use crate::camera::OrbitCamera;
use crate::props::{ParkedCar, StreetLamp, TreeProp};

// =============================================================================
// Constants
// =============================================================================

/// Camera distance beyond which tree props are hidden.
const TREE_LOD_DISTANCE: f32 = 2500.0;

/// Camera distance beyond which street lamp props are hidden.
const LAMP_LOD_DISTANCE: f32 = 3000.0;

/// Camera distance beyond which parked car props are hidden.
const CAR_LOD_DISTANCE: f32 = 2000.0;

// =============================================================================
// Pure helper functions
// =============================================================================

/// Determine whether a prop should be visible given the camera distance and the
/// LOD threshold for that prop type.
pub fn should_show_prop(camera_distance: f32, lod_threshold: f32) -> bool {
    camera_distance <= lod_threshold
}

// =============================================================================
// Systems
// =============================================================================

/// LOD system: hide/show prop entities based on camera distance.
///
/// Runs every frame and toggles `Visibility` for trees, lamps, and parked cars
/// depending on the current orbit camera distance.
#[allow(clippy::type_complexity)]
pub fn update_prop_lod(
    orbit: Res<OrbitCamera>,
    mut trees: Query<&mut Visibility, (With<TreeProp>, Without<StreetLamp>, Without<ParkedCar>)>,
    mut lamps: Query<&mut Visibility, (With<StreetLamp>, Without<TreeProp>, Without<ParkedCar>)>,
    mut cars: Query<&mut Visibility, (With<ParkedCar>, Without<TreeProp>, Without<StreetLamp>)>,
) {
    let dist = orbit.distance;

    let tree_vis = if should_show_prop(dist, TREE_LOD_DISTANCE) {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    let lamp_vis = if should_show_prop(dist, LAMP_LOD_DISTANCE) {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    let car_vis = if should_show_prop(dist, CAR_LOD_DISTANCE) {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };

    for mut vis in trees.iter_mut() {
        *vis = tree_vis;
    }
    for mut vis in lamps.iter_mut() {
        *vis = lamp_vis;
    }
    for mut vis in cars.iter_mut() {
        *vis = car_vis;
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_show_prop_within_threshold() {
        assert!(should_show_prop(100.0, TREE_LOD_DISTANCE));
        assert!(should_show_prop(100.0, LAMP_LOD_DISTANCE));
        assert!(should_show_prop(100.0, CAR_LOD_DISTANCE));
    }

    #[test]
    fn test_should_show_prop_at_threshold() {
        assert!(should_show_prop(TREE_LOD_DISTANCE, TREE_LOD_DISTANCE));
        assert!(should_show_prop(LAMP_LOD_DISTANCE, LAMP_LOD_DISTANCE));
        assert!(should_show_prop(CAR_LOD_DISTANCE, CAR_LOD_DISTANCE));
    }

    #[test]
    fn test_should_hide_prop_beyond_threshold() {
        assert!(!should_show_prop(
            TREE_LOD_DISTANCE + 1.0,
            TREE_LOD_DISTANCE
        ));
        assert!(!should_show_prop(
            LAMP_LOD_DISTANCE + 1.0,
            LAMP_LOD_DISTANCE
        ));
        assert!(!should_show_prop(CAR_LOD_DISTANCE + 1.0, CAR_LOD_DISTANCE));
    }

    #[test]
    fn test_lod_order_cars_hide_first() {
        assert!(
            CAR_LOD_DISTANCE < TREE_LOD_DISTANCE,
            "cars should hide before trees"
        );
        assert!(
            TREE_LOD_DISTANCE < LAMP_LOD_DISTANCE,
            "trees should hide before lamps"
        );
    }

    #[test]
    fn test_lod_thresholds_positive() {
        assert!(TREE_LOD_DISTANCE > 0.0);
        assert!(LAMP_LOD_DISTANCE > 0.0);
        assert!(CAR_LOD_DISTANCE > 0.0);
    }
}
