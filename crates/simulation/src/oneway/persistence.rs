use bevy::prelude::*;
use std::collections::HashMap;

use crate::Saveable;

use super::types::{OneWayDirection, OneWayDirectionMap};

impl Saveable for OneWayDirectionMap {
    const SAVE_KEY: &'static str = "oneway_direction_map";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.directions.is_empty() {
            return None;
        }

        let mut buf = Vec::new();

        // Entry count (4 bytes)
        let count = self.directions.len() as u32;
        buf.extend_from_slice(&count.to_le_bytes());

        // Each entry: segment_id (4 bytes) + direction (1 byte)
        for (&seg_id, &direction) in &self.directions {
            buf.extend_from_slice(&seg_id.to_le_bytes());
            buf.push(match direction {
                OneWayDirection::Forward => 0,
                OneWayDirection::Reverse => 1,
            });
        }

        // Generation (4 bytes)
        buf.extend_from_slice(&self.generation.to_le_bytes());

        Some(buf)
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        if bytes.len() < 4 {
            warn!(
                "Saveable {}: expected >= 4 bytes, got {}, falling back to default",
                Self::SAVE_KEY,
                bytes.len()
            );
            return Self::default();
        }

        let count = u32::from_le_bytes(bytes[0..4].try_into().unwrap_or([0; 4])) as usize;
        let mut directions = HashMap::new();

        let mut offset = 4;
        for _ in 0..count {
            if offset + 5 > bytes.len() {
                break;
            }
            let seg_id = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap_or([0; 4]));
            let dir_byte = bytes[offset + 4];
            let direction = match dir_byte {
                0 => OneWayDirection::Forward,
                _ => OneWayDirection::Reverse,
            };
            directions.insert(seg_id, direction);
            offset += 5;
        }

        let generation = if offset + 4 <= bytes.len() {
            u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap_or([0; 4]))
        } else {
            0
        };

        Self {
            directions,
            generation,
        }
    }
}
