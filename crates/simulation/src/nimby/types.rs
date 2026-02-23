//! Data types and serialization for the NIMBY/YIMBY system.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::ZoneType;
use crate::Saveable;

// =============================================================================
// Constants
// =============================================================================

/// Radius (in grid cells) within which citizens react to zone changes.
pub(crate) const REACTION_RADIUS: i32 = 8;

/// Maximum number of zone change events tracked at once (ring buffer).
pub(crate) const MAX_ZONE_CHANGES: usize = 64;

/// Number of ticks that a zone change event remains active before decaying.
pub(crate) const OPINION_DURATION_TICKS: u32 = 200;

/// Happiness penalty per unit of net opposition (scaled by land value).
pub(crate) const HAPPINESS_PENALTY_PER_OPPOSITION: f32 = 0.3;

/// Maximum happiness penalty from NIMBY opposition per citizen.
pub(crate) const MAX_NIMBY_HAPPINESS_PENALTY: f32 = 15.0;

/// Opposition threshold above which a protest event is triggered.
pub(crate) const PROTEST_THRESHOLD: f32 = 50.0;

/// Additional construction ticks added per unit of net opposition.
pub(crate) const CONSTRUCTION_SLOWDOWN_PER_OPPOSITION: f32 = 0.5;

/// Maximum additional construction ticks from opposition.
pub(crate) const MAX_CONSTRUCTION_SLOWDOWN: u32 = 50;

/// Happiness penalty when Eminent Domain policy is active.
pub const EMINENT_DOMAIN_HAPPINESS_PENALTY: f32 = 5.0;

/// Minimum ticks between protest events for the same zone change.
pub(crate) const PROTEST_COOLDOWN_TICKS: u32 = 100;

// =============================================================================
// Zone Change Event
// =============================================================================

/// Represents a zone change event that nearby citizens react to.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneChangeEvent {
    /// Grid coordinates of the zone change.
    pub grid_x: usize,
    pub grid_y: usize,
    /// The previous zone type at this location.
    pub old_zone: ZoneType,
    /// The new zone type at this location.
    pub new_zone: ZoneType,
    /// Game tick when this change occurred.
    pub created_tick: u64,
    /// Remaining ticks before this event expires.
    pub remaining_ticks: u32,
    /// Whether a protest has been triggered for this event.
    pub protest_triggered: bool,
    /// Cooldown counter for re-triggering protests.
    pub protest_cooldown: u32,
}

// =============================================================================
// NIMBY State Resource
// =============================================================================

/// Resource tracking all active zone changes and aggregate NIMBY statistics.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct NimbyState {
    /// Active zone change events that citizens are reacting to.
    pub zone_changes: Vec<ZoneChangeEvent>,
    /// Per-cell opposition score grid (0.0 = neutral/support, positive = opposition).
    /// Stored flat, indexed by `y * GRID_WIDTH + x`.
    pub opposition_grid: Vec<f32>,
    /// Total active protests in the city.
    pub active_protests: u32,
    /// Total zone changes processed since game start.
    pub total_changes_processed: u64,
}

impl Default for NimbyState {
    fn default() -> Self {
        Self {
            zone_changes: Vec::new(),
            opposition_grid: vec![0.0; GRID_WIDTH * GRID_HEIGHT],
            active_protests: 0,
            total_changes_processed: 0,
        }
    }
}

impl NimbyState {
    /// Get the opposition score at a given cell.
    #[inline]
    pub fn opposition_at(&self, x: usize, y: usize) -> f32 {
        self.opposition_grid[y * GRID_WIDTH + x]
    }
}

impl Saveable for NimbyState {
    const SAVE_KEY: &'static str = "nimby_state";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        // Skip saving if no active zone changes
        if self.zone_changes.is_empty() {
            return None;
        }

        let mut buf = Vec::new();

        // total_changes_processed (8 bytes)
        buf.extend_from_slice(&self.total_changes_processed.to_le_bytes());

        // active_protests (4 bytes)
        buf.extend_from_slice(&self.active_protests.to_le_bytes());

        // zone_changes count (4 bytes)
        let count = self.zone_changes.len() as u32;
        buf.extend_from_slice(&count.to_le_bytes());

        // Each zone change event
        for event in &self.zone_changes {
            buf.extend_from_slice(&(event.grid_x as u32).to_le_bytes());
            buf.extend_from_slice(&(event.grid_y as u32).to_le_bytes());
            buf.push(zone_type_to_u8(event.old_zone));
            buf.push(zone_type_to_u8(event.new_zone));
            buf.extend_from_slice(&event.created_tick.to_le_bytes());
            buf.extend_from_slice(&event.remaining_ticks.to_le_bytes());
            buf.push(event.protest_triggered as u8);
            buf.extend_from_slice(&event.protest_cooldown.to_le_bytes());
        }

        Some(buf)
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        let mut state = NimbyState::default();

        if bytes.len() < 16 {
            warn!(
                "Saveable {}: expected >= 16 bytes, got {}, falling back to default",
                Self::SAVE_KEY,
                bytes.len()
            );
            return state;
        }

        let mut offset = 0;

        // total_changes_processed
        if let Some(slice) = bytes.get(offset..offset + 8) {
            state.total_changes_processed = u64::from_le_bytes(slice.try_into().unwrap_or([0; 8]));
            offset += 8;
        }

        // active_protests
        if let Some(slice) = bytes.get(offset..offset + 4) {
            state.active_protests = u32::from_le_bytes(slice.try_into().unwrap_or([0; 4]));
            offset += 4;
        }

        // zone_changes count
        let count = if let Some(slice) = bytes.get(offset..offset + 4) {
            u32::from_le_bytes(slice.try_into().unwrap_or([0; 4])) as usize
        } else {
            return state;
        };
        offset += 4;

        // Each zone change event (27 bytes each: 4+4+1+1+8+4+1+4)
        for _ in 0..count.min(MAX_ZONE_CHANGES) {
            if offset + 27 > bytes.len() {
                break;
            }
            let grid_x =
                u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap_or([0; 4])) as usize;
            offset += 4;
            let grid_y =
                u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap_or([0; 4])) as usize;
            offset += 4;
            let old_zone = zone_type_from_u8(bytes[offset]);
            offset += 1;
            let new_zone = zone_type_from_u8(bytes[offset]);
            offset += 1;
            let created_tick =
                u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap_or([0; 8]));
            offset += 8;
            let remaining_ticks =
                u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap_or([0; 4]));
            offset += 4;
            let protest_triggered = bytes[offset] != 0;
            offset += 1;
            let protest_cooldown =
                u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap_or([0; 4]));
            offset += 4;

            state.zone_changes.push(ZoneChangeEvent {
                grid_x,
                grid_y,
                old_zone,
                new_zone,
                created_tick,
                remaining_ticks,
                protest_triggered,
                protest_cooldown,
            });
        }

        state
    }
}

// =============================================================================
// Zone Change Snapshot (for detecting changes)
// =============================================================================

/// Snapshot of the zone grid from the previous tick, used to detect rezoning.
#[derive(Resource)]
pub struct ZoneSnapshot {
    pub zones: Vec<ZoneType>,
}

impl Default for ZoneSnapshot {
    fn default() -> Self {
        Self {
            zones: vec![ZoneType::None; GRID_WIDTH * GRID_HEIGHT],
        }
    }
}

// =============================================================================
// Helper functions
// =============================================================================

/// Convert a ZoneType to a u8 for serialization.
pub(crate) fn zone_type_to_u8(zone: ZoneType) -> u8 {
    match zone {
        ZoneType::None => 0,
        ZoneType::ResidentialLow => 1,
        ZoneType::ResidentialMedium => 2,
        ZoneType::ResidentialHigh => 3,
        ZoneType::CommercialLow => 4,
        ZoneType::CommercialHigh => 5,
        ZoneType::Industrial => 6,
        ZoneType::Office => 7,
        ZoneType::MixedUse => 8,
    }
}

/// Convert a u8 back to a ZoneType for deserialization.
pub(crate) fn zone_type_from_u8(val: u8) -> ZoneType {
    match val {
        0 => ZoneType::None,
        1 => ZoneType::ResidentialLow,
        2 => ZoneType::ResidentialMedium,
        3 => ZoneType::ResidentialHigh,
        4 => ZoneType::CommercialLow,
        5 => ZoneType::CommercialHigh,
        6 => ZoneType::Industrial,
        7 => ZoneType::Office,
        8 => ZoneType::MixedUse,
        _ => ZoneType::None,
    }
}

/// Human-readable name for a zone type.
pub(crate) fn zone_type_name(zone: ZoneType) -> &'static str {
    match zone {
        ZoneType::None => "empty",
        ZoneType::ResidentialLow => "low-density residential",
        ZoneType::ResidentialMedium => "medium-density residential",
        ZoneType::ResidentialHigh => "high-density residential",
        ZoneType::CommercialLow => "low-density commercial",
        ZoneType::CommercialHigh => "high-density commercial",
        ZoneType::Industrial => "industrial",
        ZoneType::Office => "office",
        ZoneType::MixedUse => "mixed-use",
    }
}
