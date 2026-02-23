//! `BlueprintLibrary` resource and save/load support.
//!
//! The library stores all blueprints the player has captured. It implements the
//! `Saveable` trait so blueprints persist across save/load cycles.

use bevy::prelude::*;
use bitcode::{Decode, Encode};

use crate::Saveable;

use super::blueprint::Blueprint;

// =============================================================================
// Serializable save wrapper
// =============================================================================

/// Serializable form of the blueprint library for save/load.
#[derive(Encode, Decode, Default)]
struct BlueprintLibrarySave {
    blueprints: Vec<Blueprint>,
}

// =============================================================================
// Resource
// =============================================================================

/// Library of saved blueprints available to the player.
#[derive(Resource, Debug, Clone, Default)]
pub struct BlueprintLibrary {
    pub blueprints: Vec<Blueprint>,
}

impl BlueprintLibrary {
    /// Add a blueprint to the library.
    pub fn add(&mut self, blueprint: Blueprint) -> usize {
        let index = self.blueprints.len();
        self.blueprints.push(blueprint);
        index
    }

    /// Remove a blueprint by index.
    pub fn remove(&mut self, index: usize) -> Option<Blueprint> {
        if index < self.blueprints.len() {
            Some(self.blueprints.remove(index))
        } else {
            None
        }
    }

    /// Get a blueprint by index.
    pub fn get(&self, index: usize) -> Option<&Blueprint> {
        self.blueprints.get(index)
    }

    /// Number of stored blueprints.
    pub fn count(&self) -> usize {
        self.blueprints.len()
    }

    /// Check if the library is empty.
    pub fn is_empty(&self) -> bool {
        self.blueprints.is_empty()
    }
}

// =============================================================================
// Saveable
// =============================================================================

impl Saveable for BlueprintLibrary {
    const SAVE_KEY: &'static str = "blueprint_library";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.is_empty() {
            return None;
        }
        let save = BlueprintLibrarySave {
            blueprints: self.blueprints.clone(),
        };
        Some(bitcode::encode(&save))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        let save: BlueprintLibrarySave = crate::decode_or_warn(Self::SAVE_KEY, bytes);
        BlueprintLibrary {
            blueprints: save.blueprints,
        }
    }
}
