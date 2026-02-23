//! Resources for citizen info panel selection and camera follow.

use bevy::prelude::*;

/// Resource tracking the currently selected citizen entity.
#[derive(Resource, Default)]
pub struct SelectedCitizen(pub Option<Entity>);

/// Resource indicating whether the camera should follow a citizen.
#[derive(Resource, Default)]
pub struct FollowCitizen(pub Option<Entity>);
