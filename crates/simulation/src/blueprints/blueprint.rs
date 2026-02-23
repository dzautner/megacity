//! Core `Blueprint` struct with capture and placement logic.
//!
//! A `Blueprint` stores a position-independent snapshot of a rectangular map
//! region (road segments + zone cells). It can later be stamped onto the map
//! at a different location.

use bevy::prelude::*;
use bitcode::{Decode, Encode};

use crate::config::{CELL_SIZE, GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, RoadType, WorldGrid, ZoneType};
use crate::road_segments::RoadSegmentStore;
use crate::roads::RoadNetwork;

use super::types::{BlueprintSegment, BlueprintZoneCell, PlaceResult};

/// A single blueprint capturing a road+zone layout relative to an origin.
#[derive(Debug, Clone, Encode, Decode)]
pub struct Blueprint {
    /// Human-readable name for the blueprint.
    pub name: String,
    /// Width of the captured area in grid cells.
    pub width: u32,
    /// Height of the captured area in grid cells.
    pub height: u32,
    /// Road segments with control points relative to the origin.
    pub segments: Vec<BlueprintSegment>,
    /// Zone cells with offsets relative to the origin.
    pub zone_cells: Vec<BlueprintZoneCell>,
}

impl Blueprint {
    /// Capture a blueprint from the given rectangular region of the map.
    ///
    /// `origin_x, origin_y` is the top-left corner in grid coordinates.
    /// `w, h` is the size in grid cells.
    pub fn capture(
        grid: &WorldGrid,
        segment_store: &RoadSegmentStore,
        origin_x: usize,
        origin_y: usize,
        w: usize,
        h: usize,
        name: String,
    ) -> Self {
        let origin_world_x = origin_x as f32 * CELL_SIZE;
        let origin_world_y = origin_y as f32 * CELL_SIZE;

        // Capture zone cells
        let mut zone_cells = Vec::new();
        for dy in 0..h {
            for dx in 0..w {
                let gx = origin_x + dx;
                let gy = origin_y + dy;
                if !grid.in_bounds(gx, gy) {
                    continue;
                }
                let cell = grid.get(gx, gy);
                if cell.zone != ZoneType::None {
                    zone_cells.push(BlueprintZoneCell {
                        dx: dx as i32,
                        dy: dy as i32,
                        zone_type: cell.zone.into(),
                    });
                }
            }
        }

        // Capture road segments that have at least one rasterized cell in the region
        let mut segments = Vec::new();
        for seg in &segment_store.segments {
            let in_region = seg.rasterized_cells.iter().any(|&(cx, cy)| {
                cx >= origin_x && cx < origin_x + w && cy >= origin_y && cy < origin_y + h
            });
            if in_region {
                segments.push(BlueprintSegment {
                    p0: [seg.p0.x - origin_world_x, seg.p0.y - origin_world_y],
                    p1: [seg.p1.x - origin_world_x, seg.p1.y - origin_world_y],
                    p2: [seg.p2.x - origin_world_x, seg.p2.y - origin_world_y],
                    p3: [seg.p3.x - origin_world_x, seg.p3.y - origin_world_y],
                    road_type: seg.road_type.into(),
                });
            }
        }

        Blueprint {
            name,
            width: w as u32,
            height: h as u32,
            segments,
            zone_cells,
        }
    }

    /// Place this blueprint onto the map at the given grid origin.
    ///
    /// Returns the number of road segments placed and zone cells set.
    pub fn place(
        &self,
        grid: &mut WorldGrid,
        segment_store: &mut RoadSegmentStore,
        roads: &mut RoadNetwork,
        target_x: usize,
        target_y: usize,
    ) -> PlaceResult {
        let target_world_x = target_x as f32 * CELL_SIZE;
        let target_world_y = target_y as f32 * CELL_SIZE;

        let mut segments_placed = 0u32;
        let mut zones_placed = 0u32;

        // Place road segments
        for seg in &self.segments {
            let p0 = Vec2::new(seg.p0[0] + target_world_x, seg.p0[1] + target_world_y);
            let p1 = Vec2::new(seg.p1[0] + target_world_x, seg.p1[1] + target_world_y);
            let p2 = Vec2::new(seg.p2[0] + target_world_x, seg.p2[1] + target_world_y);
            let p3 = Vec2::new(seg.p3[0] + target_world_x, seg.p3[1] + target_world_y);

            // Bounds check: ensure segment endpoints are within the grid
            let (gx0, gy0) = WorldGrid::world_to_grid(p0.x, p0.y);
            let (gx3, gy3) = WorldGrid::world_to_grid(p3.x, p3.y);
            if gx0 < 0
                || gy0 < 0
                || gx3 < 0
                || gy3 < 0
                || (gx0 as usize) >= GRID_WIDTH
                || (gy0 as usize) >= GRID_HEIGHT
                || (gx3 as usize) >= GRID_WIDTH
                || (gy3 as usize) >= GRID_HEIGHT
            {
                continue;
            }

            let road_type: RoadType = seg.road_type.into();
            let start = segment_store.find_or_create_node(p0, 16.0);
            let end = segment_store.find_or_create_node(p3, 16.0);
            segment_store.add_segment(start, end, p0, p1, p2, p3, road_type, grid, roads);
            segments_placed += 1;
        }

        // Place zone cells
        for zc in &self.zone_cells {
            let gx = target_x as i32 + zc.dx;
            let gy = target_y as i32 + zc.dy;
            if gx < 0 || gy < 0 {
                continue;
            }
            let gx = gx as usize;
            let gy = gy as usize;
            if !grid.in_bounds(gx, gy) {
                continue;
            }
            let cell = grid.get(gx, gy);
            // Only place zones on non-water, non-road cells without buildings
            if cell.cell_type != CellType::Water
                && cell.cell_type != CellType::Road
                && cell.building_id.is_none()
            {
                let zone: ZoneType = zc.zone_type.into();
                grid.get_mut(gx, gy).zone = zone;
                zones_placed += 1;
            }
        }

        PlaceResult {
            segments_placed,
            zones_placed,
        }
    }
}
