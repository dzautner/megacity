//! Road hierarchy enforcement and warnings (TRAF-003).
//!
//! Detects connections that violate the road hierarchy principle: each road type
//! has a *level* and connections that skip more than one level (e.g. Local road
//! connecting directly to Highway) are flagged as violations.
//!
//! Hierarchy levels:
//!   Path = 0, Local = 1, OneWay = 2, Avenue = 2, Boulevard = 3, Highway = 4
//!
//! A violation occurs when two segments meeting at a node differ by more than 1
//! level. The system produces a list of [`HierarchyViolation`] entries stored in
//! [`RoadHierarchyState`], which the advisor system can consume.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::advisors::{AdvisorMessage, AdvisorPanel, AdvisorType, TipId};
use crate::grid::RoadType;
use crate::road_segments::RoadSegmentStore;
use crate::Saveable;
use crate::TickCounter;

// ---------------------------------------------------------------------------
// Hierarchy levels
// ---------------------------------------------------------------------------

/// Return the hierarchy level for a road type.
///
/// Higher values mean higher-capacity roads. A jump of more than 1 level
/// between two connected segments is a violation.
pub fn hierarchy_level(road_type: RoadType) -> u8 {
    match road_type {
        RoadType::Path => 0,
        RoadType::Local => 1,
        RoadType::OneWay | RoadType::Avenue => 2,
        RoadType::Boulevard => 3,
        RoadType::Highway => 4,
    }
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A single detected hierarchy violation at a node.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Encode, Decode, Serialize, Deserialize)]
pub struct HierarchyViolation {
    /// The node where the violation occurs.
    pub node_id: u32,
    /// Grid coordinate (col) of the node (approximate, clamped to grid).
    pub grid_x: usize,
    /// Grid coordinate (row) of the node (approximate, clamped to grid).
    pub grid_y: usize,
    /// The lower-level segment id.
    pub low_segment_id: u32,
    /// The higher-level segment id.
    pub high_segment_id: u32,
    /// Road type discriminant of the lower-level segment.
    pub low_road_type: u8,
    /// Road type discriminant of the higher-level segment.
    pub high_road_type: u8,
    /// How many levels were skipped (difference - 1).
    pub levels_skipped: u8,
}

/// Resource holding the current set of road hierarchy violations.
#[derive(Resource, Default, Clone, Debug, Encode, Decode, Serialize, Deserialize)]
pub struct RoadHierarchyState {
    pub violations: Vec<HierarchyViolation>,
}

impl Saveable for RoadHierarchyState {
    const SAVE_KEY: &'static str = "road_hierarchy";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.violations.is_empty() {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// How often (in ticks) the hierarchy check runs.
const CHECK_INTERVAL: u64 = 200;

/// Scan all nodes in the road segment store and detect hierarchy violations.
pub fn update_road_hierarchy(
    tick: Res<TickCounter>,
    segments: Res<RoadSegmentStore>,
    mut state: ResMut<RoadHierarchyState>,
) {
    if !tick.0.is_multiple_of(CHECK_INTERVAL) {
        return;
    }

    let mut violations = Vec::new();

    for node in &segments.nodes {
        let connected: Vec<_> = node
            .connected_segments
            .iter()
            .filter_map(|&sid| segments.get_segment(sid))
            .collect();

        for i in 0..connected.len() {
            for j in (i + 1)..connected.len() {
                let a = connected[i];
                let b = connected[j];
                let level_a = hierarchy_level(a.road_type);
                let level_b = hierarchy_level(b.road_type);
                let diff = level_a.abs_diff(level_b);

                if diff > 1 {
                    let (low, high) = if level_a < level_b { (a, b) } else { (b, a) };

                    let (gx_i32, gy_i32) =
                        crate::grid::WorldGrid::world_to_grid(node.position.x, node.position.y);
                    let gx = (gx_i32.max(0) as usize).min(crate::config::GRID_WIDTH - 1);
                    let gy = (gy_i32.max(0) as usize).min(crate::config::GRID_HEIGHT - 1);

                    violations.push(HierarchyViolation {
                        node_id: node.id.0,
                        grid_x: gx,
                        grid_y: gy,
                        low_segment_id: low.id.0,
                        high_segment_id: high.id.0,
                        low_road_type: low.road_type as u8,
                        high_road_type: high.road_type as u8,
                        levels_skipped: diff - 1,
                    });
                }
            }
        }
    }

    state.violations = violations;
}

/// Feed hierarchy violations into the advisor panel as Infrastructure warnings.
pub fn advise_road_hierarchy(
    tick: Res<TickCounter>,
    state: Res<RoadHierarchyState>,
    mut panel: ResMut<AdvisorPanel>,
) {
    if !tick.0.is_multiple_of(CHECK_INTERVAL) {
        return;
    }

    if state.violations.is_empty() {
        return;
    }

    let count = state.violations.len();

    // Pick the worst violation for the suggestion message
    let worst = state
        .violations
        .iter()
        .max_by_key(|v| v.levels_skipped)
        .unwrap();

    let low_name = road_type_display_name(worst.low_road_type);
    let high_name = road_type_display_name(worst.high_road_type);

    let message = if count == 1 {
        format!(
            "A {} connects directly to a {} \u{2014} this will cause bottlenecks.",
            low_name, high_name,
        )
    } else {
        format!(
            "{} road connections violate the hierarchy. A {} connects directly to a {}.",
            count, low_name, high_name,
        )
    };

    let suggestion = format!(
        "Add intermediate road types between {} and {} to smooth the transition.",
        low_name, high_name,
    );

    // Priority scales with violation count: 2 (low) up to 4 (high)
    let priority = (2 + count.min(2)) as u8;

    let t = tick.0;
    panel.messages.push(AdvisorMessage {
        advisor_type: AdvisorType::Infrastructure,
        tip_id: TipId::RoadHierarchyViolation,
        message,
        priority,
        suggestion,
        tick_created: t,
        location: Some((worst.grid_x, worst.grid_y)),
    });
}

/// Map a `RoadType` discriminant (u8) back to a display name.
fn road_type_display_name(raw: u8) -> &'static str {
    // Discriminant order: Local=0, Avenue=1, Boulevard=2, Highway=3, OneWay=4, Path=5
    match raw {
        0 => "Local Road",
        1 => "Avenue",
        2 => "Boulevard",
        3 => "Highway",
        4 => "One-Way Road",
        5 => "Path",
        _ => "Road",
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct RoadHierarchyPlugin;

impl Plugin for RoadHierarchyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RoadHierarchyState>().add_systems(
            FixedUpdate,
            (update_road_hierarchy, advise_road_hierarchy)
                .chain()
                .in_set(crate::SimulationSet::Simulation),
        );

        // Register for save/load
        let mut registry = app
            .world_mut()
            .get_resource_or_insert_with(crate::SaveableRegistry::default);
        registry.register::<RoadHierarchyState>();
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hierarchy_levels_are_ordered() {
        assert!(hierarchy_level(RoadType::Path) < hierarchy_level(RoadType::Local));
        assert!(hierarchy_level(RoadType::Local) < hierarchy_level(RoadType::Avenue));
        assert_eq!(
            hierarchy_level(RoadType::Avenue),
            hierarchy_level(RoadType::OneWay)
        );
        assert!(hierarchy_level(RoadType::Avenue) < hierarchy_level(RoadType::Boulevard));
        assert!(hierarchy_level(RoadType::Boulevard) < hierarchy_level(RoadType::Highway));
    }

    #[test]
    fn test_adjacent_levels_no_violation() {
        // Local (1) -> Avenue (2) = diff of 1, should not be a violation
        let diff = hierarchy_level(RoadType::Avenue).abs_diff(hierarchy_level(RoadType::Local));
        assert_eq!(diff, 1, "Adjacent levels should differ by 1");
        assert!(diff <= 1, "Adjacent levels should not trigger a violation");
    }

    #[test]
    fn test_local_to_highway_is_violation() {
        let diff = hierarchy_level(RoadType::Highway).abs_diff(hierarchy_level(RoadType::Local));
        assert_eq!(diff, 3, "Local to Highway should differ by 3 levels");
        assert!(diff > 1, "Should be a violation");
    }

    #[test]
    fn test_local_to_boulevard_is_violation() {
        let diff = hierarchy_level(RoadType::Boulevard).abs_diff(hierarchy_level(RoadType::Local));
        assert_eq!(diff, 2, "Local to Boulevard should differ by 2 levels");
        assert!(diff > 1, "Should be a violation");
    }

    #[test]
    fn test_avenue_to_highway_is_violation() {
        let diff = hierarchy_level(RoadType::Highway).abs_diff(hierarchy_level(RoadType::Avenue));
        assert_eq!(diff, 2, "Avenue to Highway should differ by 2 levels");
        assert!(diff > 1, "Should be a violation");
    }

    #[test]
    fn test_boulevard_to_highway_no_violation() {
        let diff =
            hierarchy_level(RoadType::Highway).abs_diff(hierarchy_level(RoadType::Boulevard));
        assert_eq!(diff, 1, "Boulevard to Highway should differ by 1 level");
        assert!(diff <= 1, "Should not be a violation");
    }

    #[test]
    fn test_path_to_local_no_violation() {
        let diff = hierarchy_level(RoadType::Local).abs_diff(hierarchy_level(RoadType::Path));
        assert_eq!(diff, 1);
        assert!(diff <= 1, "Path to Local should not be a violation");
    }

    #[test]
    fn test_path_to_avenue_is_violation() {
        let diff = hierarchy_level(RoadType::Avenue).abs_diff(hierarchy_level(RoadType::Path));
        assert_eq!(diff, 2);
        assert!(diff > 1, "Path to Avenue should be a violation");
    }

    #[test]
    fn test_saveable_roundtrip() {
        let state = RoadHierarchyState {
            violations: vec![HierarchyViolation {
                node_id: 42,
                grid_x: 10,
                grid_y: 20,
                low_segment_id: 1,
                high_segment_id: 2,
                low_road_type: RoadType::Local as u8,
                high_road_type: RoadType::Highway as u8,
                levels_skipped: 2,
            }],
        };

        let bytes = state.save_to_bytes().expect("should serialize");
        let restored = RoadHierarchyState::load_from_bytes(&bytes);
        assert_eq!(restored.violations.len(), 1);
        assert_eq!(restored.violations[0].node_id, 42);
        assert_eq!(restored.violations[0].levels_skipped, 2);
    }

    #[test]
    fn test_saveable_empty_returns_none() {
        let state = RoadHierarchyState::default();
        assert!(
            state.save_to_bytes().is_none(),
            "Empty state should not produce save bytes"
        );
    }

    #[test]
    fn test_road_type_display_names() {
        assert_eq!(road_type_display_name(RoadType::Local as u8), "Local Road");
        assert_eq!(road_type_display_name(RoadType::Avenue as u8), "Avenue");
        assert_eq!(
            road_type_display_name(RoadType::Boulevard as u8),
            "Boulevard"
        );
        assert_eq!(road_type_display_name(RoadType::Highway as u8), "Highway");
        assert_eq!(
            road_type_display_name(RoadType::OneWay as u8),
            "One-Way Road"
        );
        assert_eq!(road_type_display_name(RoadType::Path as u8), "Path");
    }
}
