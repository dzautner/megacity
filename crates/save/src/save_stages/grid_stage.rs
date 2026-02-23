use crate::save_codec::*;
use crate::save_types::*;

use simulation::grid::WorldGrid;
use simulation::road_segments::RoadSegmentStore;
use simulation::roads::RoadNetwork;

/// Grid, road network, and road segment data.
pub struct GridStageOutput {
    pub grid: SaveGrid,
    pub roads: SaveRoadNetwork,
    pub road_segments: Option<SaveRoadSegmentStore>,
}

/// Collect grid, road network, and road segment data.
pub fn collect_grid_stage(
    grid: &WorldGrid,
    roads: &RoadNetwork,
    segment_store: Option<&RoadSegmentStore>,
) -> GridStageOutput {
    let save_cells: Vec<SaveCell> = grid
        .cells
        .iter()
        .map(|c| SaveCell {
            elevation: c.elevation,
            cell_type: match c.cell_type {
                simulation::grid::CellType::Grass => 0,
                simulation::grid::CellType::Water => 1,
                simulation::grid::CellType::Road => 2,
            },
            zone: zone_type_to_u8(c.zone),
            road_type: road_type_to_u8(c.road_type),
            has_power: c.has_power,
            has_water: c.has_water,
        })
        .collect();

    GridStageOutput {
        grid: SaveGrid {
            cells: save_cells,
            width: grid.width,
            height: grid.height,
        },
        roads: SaveRoadNetwork {
            road_positions: roads.edges.keys().map(|n| (n.0, n.1)).collect(),
        },
        road_segments: segment_store.map(|store| SaveRoadSegmentStore {
            nodes: store
                .nodes
                .iter()
                .map(|n| SaveSegmentNode {
                    id: n.id.0,
                    x: n.position.x,
                    y: n.position.y,
                    connected_segments: n.connected_segments.iter().map(|s| s.0).collect(),
                })
                .collect(),
            segments: store
                .segments
                .iter()
                .map(|s| SaveRoadSegment {
                    id: s.id.0,
                    start_node: s.start_node.0,
                    end_node: s.end_node.0,
                    p0_x: s.p0.x,
                    p0_y: s.p0.y,
                    p1_x: s.p1.x,
                    p1_y: s.p1.y,
                    p2_x: s.p2.x,
                    p2_y: s.p2.y,
                    p3_x: s.p3.x,
                    p3_y: s.p3.y,
                    road_type: road_type_to_u8(s.road_type),
                })
                .collect(),
        }),
    }
}
