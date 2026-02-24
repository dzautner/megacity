//! SAVE-025: Post-load validation and repair for road segment data.
//!
//! The existing save system serializes `RoadSegmentStore` through
//! `SaveRoadSegmentStore` in the save crate, preserving Bezier control points
//! (p0–p3), segment IDs, node positions, and connectivity. However, two pieces
//! of derived state are not persisted and must be rebuilt after load:
//!
//! 1. **Arc length** — each `RoadSegment::arc_length` is restored as `0.0`.
//!    Since `rasterize_segment()` uses arc_length to determine sample density,
//!    a zero value causes under-sampling (only 4 points) on long/curved roads,
//!    producing gaps in the grid cache after load.
//!
//! 2. **Intersection reference validity** — `SegmentNode::connected_segments`
//!    may reference segment IDs that no longer exist if a save was edited or
//!    migrated. Dangling references can cause panics in rendering/pathfinding.
//!
//! This plugin registers a one-shot `FixedUpdate` system that runs on the first
//! tick after a load (gated on `PostLoadRebuildPending`), recomputes arc
//! lengths, prunes dangling intersection references, and re-rasterizes all
//! segments with correct sample density.

use bevy::prelude::*;

use crate::grid::WorldGrid;
use crate::post_load_rebuild::PostLoadRebuildPending;
use crate::road_segments::RoadSegmentStore;
use crate::roads::RoadNetwork;

pub struct RoadSegmentSavePlugin;

impl Plugin for RoadSegmentSavePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            rebuild_road_segment_derived_state
                .run_if(resource_exists::<PostLoadRebuildPending>)
                // Run before the main post-load rebuild so that the CSR graph
                // builder sees correct rasterized_cells (used for one-way edge
                // blocking) and arc_length values.
                .before(crate::post_load_rebuild::post_load_rebuild_derived_state)
                .in_set(crate::SimulationSet::PreSim),
        );
    }
}

/// Recompute arc lengths, prune dangling intersection references, and
/// re-rasterize all road segments after a save/load cycle.
fn rebuild_road_segment_derived_state(
    mut segments: ResMut<RoadSegmentStore>,
    mut grid: ResMut<WorldGrid>,
    mut roads: ResMut<RoadNetwork>,
) {
    if segments.segments.is_empty() {
        return;
    }

    // --- 1. Recompute arc lengths from Bezier control points ---
    let mut recomputed = 0u32;
    for segment in &mut segments.segments {
        let new_arc_length = segment.compute_arc_length();
        if (segment.arc_length - new_arc_length).abs() > 0.01 {
            recomputed += 1;
        }
        segment.arc_length = new_arc_length;
    }

    // --- 2. Validate intersection references ---
    // Build a set of valid segment IDs for fast lookup.
    let valid_ids: std::collections::HashSet<u32> =
        segments.segments.iter().map(|s| s.id.0).collect();

    let mut pruned_refs = 0u32;
    for node in &mut segments.nodes {
        let before = node.connected_segments.len();
        node.connected_segments
            .retain(|sid| valid_ids.contains(&sid.0));
        pruned_refs += (before - node.connected_segments.len()) as u32;
    }

    // --- 3. Re-rasterize all segments with correct arc lengths ---
    // This overwrites the initial rasterization (done in restore_resources
    // with arc_length=0.0) with properly-sampled curves.
    segments.rasterize_all(&mut grid, &mut roads);

    if recomputed > 0 || pruned_refs > 0 {
        info!(
            "Road segment post-load repair: recomputed {} arc lengths, \
             pruned {} dangling intersection refs, re-rasterized {} segments",
            recomputed,
            pruned_refs,
            segments.segments.len(),
        );
    } else {
        info!(
            "Road segment post-load validation: {} segments OK, \
             {} nodes OK, all intersection refs valid",
            segments.segments.len(),
            segments.nodes.len(),
        );
    }
}

/// Validate that all intersection references in a `RoadSegmentStore` point to
/// existing segments. Returns the count of dangling references found.
///
/// This is a pure function suitable for use in tests.
pub fn count_dangling_intersection_refs(store: &RoadSegmentStore) -> usize {
    let valid_ids: std::collections::HashSet<u32> =
        store.segments.iter().map(|s| s.id.0).collect();

    store
        .nodes
        .iter()
        .flat_map(|n| &n.connected_segments)
        .filter(|sid| !valid_ids.contains(&sid.0))
        .count()
}

/// Validate that every node referenced by a segment (start_node, end_node)
/// actually exists in the store. Returns the count of orphaned references.
pub fn count_orphaned_node_refs(store: &RoadSegmentStore) -> usize {
    let valid_node_ids: std::collections::HashSet<u32> =
        store.nodes.iter().map(|n| n.id.0).collect();

    store
        .segments
        .iter()
        .filter(|s| {
            !valid_node_ids.contains(&s.start_node.0)
                || !valid_node_ids.contains(&s.end_node.0)
        })
        .count()
}
