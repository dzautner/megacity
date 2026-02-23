//! ECS systems and plugin for roundabout traffic and throughput tracking.

use bevy::prelude::*;

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::road_graph_csr::CsrGraph;
use crate::roads::RoadNode;
use crate::traffic::TrafficGrid;
use crate::SlowTickTimer;

use super::RoundaboutRegistry;

/// Weight multiplier applied to edges entering a roundabout (yield-on-entry).
/// Higher values discourage entry when the roundabout is busy.
const YIELD_ENTRY_WEIGHT_MULTIPLIER: u32 = 3;

/// Update traffic weights in the CSR graph based on roundabout traffic rules.
///
/// For each roundabout:
/// - Edges from approach roads INTO the roundabout ring get increased weight
///   (yield-on-entry), scaled by current roundabout traffic density.
/// - Edges within the roundabout ring maintain normal weight (priority).
fn update_roundabout_traffic(
    registry: Res<RoundaboutRegistry>,
    mut csr: ResMut<CsrGraph>,
    traffic: Res<TrafficGrid>,
    timer: Res<SlowTickTimer>,
) {
    // Only run every few ticks (matching slow tick)
    if !timer.should_run() {
        return;
    }

    if registry.roundabouts.is_empty() {
        return;
    }

    // Build a set of roundabout ring nodes for fast lookup
    let mut ring_node_set = std::collections::HashSet::new();
    for roundabout in &registry.roundabouts {
        for &(rx, ry) in &roundabout.ring_cells {
            ring_node_set.insert(RoadNode(rx, ry));
        }
    }

    // Adjust weights for edges entering the roundabout
    for node_idx in 0..csr.node_count() {
        let node = csr.nodes[node_idx];
        let is_ring_node = ring_node_set.contains(&node);

        let start = csr.node_offsets[node_idx] as usize;
        let end = csr.node_offsets[node_idx + 1] as usize;

        for edge_pos in start..end {
            let neighbor_idx = csr.edges[edge_pos] as usize;
            let neighbor = csr.nodes[neighbor_idx];
            let neighbor_is_ring = ring_node_set.contains(&neighbor);

            if !is_ring_node && neighbor_is_ring {
                // Edge entering the roundabout: apply yield-on-entry penalty
                let ring_traffic = traffic.get(neighbor.0, neighbor.1);
                let penalty = if ring_traffic > 0 {
                    YIELD_ENTRY_WEIGHT_MULTIPLIER * (1 + ring_traffic as u32 / 5)
                } else {
                    YIELD_ENTRY_WEIGHT_MULTIPLIER
                };
                csr.weights[edge_pos] = csr.weights[edge_pos].max(1) * penalty;
            }
            // Edges within the ring or exiting the ring keep default weight (priority)
        }
    }
}

/// Track roundabout throughput statistics.
///
/// For each roundabout, sums the traffic density on its ring cells and updates
/// rolling average statistics.
fn roundabout_efficiency(
    mut registry: ResMut<RoundaboutRegistry>,
    traffic: Res<TrafficGrid>,
    timer: Res<SlowTickTimer>,
) {
    if !timer.should_run() {
        return;
    }

    let roundabout_count = registry.roundabouts.len();

    // Ensure stats vec matches roundabout count
    registry
        .stats
        .resize_with(roundabout_count, Default::default);

    for i in 0..roundabout_count {
        let throughput: u32 = registry.roundabouts[i]
            .ring_cells
            .iter()
            .map(|&(rx, ry)| {
                if rx < GRID_WIDTH && ry < GRID_HEIGHT {
                    traffic.get(rx, ry) as u32
                } else {
                    0
                }
            })
            .sum();

        let stats = &mut registry.stats[i];
        stats.current_throughput = throughput;
        stats.sample_count += 1;

        // Exponential moving average (alpha = 0.1)
        let alpha = 0.1_f32;
        stats.average_throughput =
            alpha * throughput as f32 + (1.0 - alpha) * stats.average_throughput;
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct RoundaboutPlugin;

impl Plugin for RoundaboutPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RoundaboutRegistry>()
            .add_systems(FixedUpdate, update_roundabout_traffic)
            .add_systems(FixedUpdate, roundabout_efficiency);

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<RoundaboutRegistry>();
    }
}
