//! Blueprint / Template System (UX-041).
//!
//! Provides a `BlueprintLibrary` resource that stores reusable road+zone layouts.
//! Players can capture a rectangular area of the map as a blueprint, then stamp
//! copies of that blueprint at different locations.
//!
//! Blueprints store road segments and zone cells relative to an origin, making
//! them position-independent and reusable.

use bevy::prelude::*;
use bitcode::{Decode, Encode};

use crate::config::{CELL_SIZE, GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, RoadType, WorldGrid, ZoneType};
use crate::road_segments::RoadSegmentStore;
use crate::roads::RoadNetwork;
use crate::Saveable;

// =============================================================================
// Types
// =============================================================================

/// A road segment stored relative to the blueprint origin.
#[derive(Debug, Clone, Encode, Decode)]
pub struct BlueprintSegment {
    /// Control points relative to the blueprint origin (world units).
    pub p0: [f32; 2],
    pub p1: [f32; 2],
    pub p2: [f32; 2],
    pub p3: [f32; 2],
    pub road_type: BlueprintRoadType,
}

/// A zone cell stored relative to the blueprint origin.
#[derive(Debug, Clone, Encode, Decode)]
pub struct BlueprintZoneCell {
    /// Offset from the blueprint origin in grid cells.
    pub dx: i32,
    pub dy: i32,
    pub zone_type: BlueprintZoneType,
}

/// Serializable mirror of `RoadType` for bitcode encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
pub enum BlueprintRoadType {
    Local,
    Avenue,
    Boulevard,
    Highway,
    OneWay,
    Path,
}

impl From<RoadType> for BlueprintRoadType {
    fn from(rt: RoadType) -> Self {
        match rt {
            RoadType::Local => BlueprintRoadType::Local,
            RoadType::Avenue => BlueprintRoadType::Avenue,
            RoadType::Boulevard => BlueprintRoadType::Boulevard,
            RoadType::Highway => BlueprintRoadType::Highway,
            RoadType::OneWay => BlueprintRoadType::OneWay,
            RoadType::Path => BlueprintRoadType::Path,
        }
    }
}

impl From<BlueprintRoadType> for RoadType {
    fn from(brt: BlueprintRoadType) -> Self {
        match brt {
            BlueprintRoadType::Local => RoadType::Local,
            BlueprintRoadType::Avenue => RoadType::Avenue,
            BlueprintRoadType::Boulevard => RoadType::Boulevard,
            BlueprintRoadType::Highway => RoadType::Highway,
            BlueprintRoadType::OneWay => RoadType::OneWay,
            BlueprintRoadType::Path => RoadType::Path,
        }
    }
}

/// Serializable mirror of `ZoneType` for bitcode encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
pub enum BlueprintZoneType {
    None,
    ResidentialLow,
    ResidentialMedium,
    ResidentialHigh,
    CommercialLow,
    CommercialHigh,
    Industrial,
    Office,
    MixedUse,
}

impl From<ZoneType> for BlueprintZoneType {
    fn from(zt: ZoneType) -> Self {
        match zt {
            ZoneType::None => BlueprintZoneType::None,
            ZoneType::ResidentialLow => BlueprintZoneType::ResidentialLow,
            ZoneType::ResidentialMedium => BlueprintZoneType::ResidentialMedium,
            ZoneType::ResidentialHigh => BlueprintZoneType::ResidentialHigh,
            ZoneType::CommercialLow => BlueprintZoneType::CommercialLow,
            ZoneType::CommercialHigh => BlueprintZoneType::CommercialHigh,
            ZoneType::Industrial => BlueprintZoneType::Industrial,
            ZoneType::Office => BlueprintZoneType::Office,
            ZoneType::MixedUse => BlueprintZoneType::MixedUse,
        }
    }
}

impl From<BlueprintZoneType> for ZoneType {
    fn from(bzt: BlueprintZoneType) -> Self {
        match bzt {
            BlueprintZoneType::None => ZoneType::None,
            BlueprintZoneType::ResidentialLow => ZoneType::ResidentialLow,
            BlueprintZoneType::ResidentialMedium => ZoneType::ResidentialMedium,
            BlueprintZoneType::ResidentialHigh => ZoneType::ResidentialHigh,
            BlueprintZoneType::CommercialLow => ZoneType::CommercialLow,
            BlueprintZoneType::CommercialHigh => ZoneType::CommercialHigh,
            BlueprintZoneType::Industrial => ZoneType::Industrial,
            BlueprintZoneType::Office => ZoneType::Office,
            BlueprintZoneType::MixedUse => ZoneType::MixedUse,
        }
    }
}

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

/// Result of placing a blueprint on the map.
#[derive(Debug, Clone, Copy)]
pub struct PlaceResult {
    pub segments_placed: u32,
    pub zones_placed: u32,
}

// =============================================================================
// Events
// =============================================================================

/// Event to capture a blueprint from a rectangular region.
#[derive(Event)]
pub struct CaptureBlueprint {
    pub origin_x: usize,
    pub origin_y: usize,
    pub width: usize,
    pub height: usize,
    pub name: String,
}

/// Event to place a blueprint at a target location.
#[derive(Event)]
pub struct PlaceBlueprint {
    /// Index of the blueprint in the library.
    pub blueprint_index: usize,
    /// Target grid origin for placement.
    pub target_x: usize,
    pub target_y: usize,
}

/// Event fired after a blueprint is successfully captured.
#[derive(Event)]
pub struct BlueprintCaptured {
    pub name: String,
    pub index: usize,
}

/// Event fired after a blueprint is successfully placed.
#[derive(Event)]
pub struct BlueprintPlaced {
    pub name: String,
    pub segments_placed: u32,
    pub zones_placed: u32,
}

// =============================================================================
// Resource
// =============================================================================

/// Serializable form of the blueprint library for save/load.
#[derive(Encode, Decode, Default)]
struct BlueprintLibrarySave {
    blueprints: Vec<Blueprint>,
}

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

// =============================================================================
// Systems
// =============================================================================

/// System that processes `CaptureBlueprint` events.
fn handle_capture_blueprint(
    mut events: EventReader<CaptureBlueprint>,
    mut library: ResMut<BlueprintLibrary>,
    grid: Res<WorldGrid>,
    segments: Res<RoadSegmentStore>,
    mut captured_events: EventWriter<BlueprintCaptured>,
) {
    for ev in events.read() {
        let blueprint = Blueprint::capture(
            &grid,
            &segments,
            ev.origin_x,
            ev.origin_y,
            ev.width,
            ev.height,
            ev.name.clone(),
        );
        let index = library.add(blueprint);
        info!(
            "Blueprint '{}' captured (index {}) from ({},{}) size {}x{}",
            ev.name, index, ev.origin_x, ev.origin_y, ev.width, ev.height
        );
        captured_events.send(BlueprintCaptured {
            name: ev.name.clone(),
            index,
        });
    }
}

/// System that processes `PlaceBlueprint` events.
fn handle_place_blueprint(
    mut events: EventReader<PlaceBlueprint>,
    library: Res<BlueprintLibrary>,
    mut grid: ResMut<WorldGrid>,
    mut segments: ResMut<RoadSegmentStore>,
    mut roads: ResMut<RoadNetwork>,
    mut placed_events: EventWriter<BlueprintPlaced>,
) {
    for ev in events.read() {
        let Some(blueprint) = library.get(ev.blueprint_index) else {
            warn!(
                "PlaceBlueprint: invalid index {} (library has {} blueprints)",
                ev.blueprint_index,
                library.count()
            );
            continue;
        };
        let name = blueprint.name.clone();
        let result = blueprint.place(
            &mut grid,
            &mut segments,
            &mut roads,
            ev.target_x,
            ev.target_y,
        );
        info!(
            "Blueprint '{}' placed at ({},{}) — {} segments, {} zones",
            name, ev.target_x, ev.target_y, result.segments_placed, result.zones_placed
        );
        placed_events.send(BlueprintPlaced {
            name,
            segments_placed: result.segments_placed,
            zones_placed: result.zones_placed,
        });
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct BlueprintPlugin;

impl Plugin for BlueprintPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BlueprintLibrary>()
            .add_event::<CaptureBlueprint>()
            .add_event::<PlaceBlueprint>()
            .add_event::<BlueprintCaptured>()
            .add_event::<BlueprintPlaced>()
            .add_systems(
                FixedUpdate,
                (handle_capture_blueprint, handle_place_blueprint)
                    .in_set(crate::SimulationSet::Simulation),
            );

        // Register with save system
        let mut registry = app.world_mut().resource_mut::<crate::SaveableRegistry>();
        registry.register::<BlueprintLibrary>();
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blueprint_road_type_roundtrip() {
        let types = [
            RoadType::Local,
            RoadType::Avenue,
            RoadType::Boulevard,
            RoadType::Highway,
            RoadType::OneWay,
            RoadType::Path,
        ];
        for rt in types {
            let brt: BlueprintRoadType = rt.into();
            let back: RoadType = brt.into();
            assert_eq!(rt, back);
        }
    }

    #[test]
    fn test_blueprint_zone_type_roundtrip() {
        let types = [
            ZoneType::None,
            ZoneType::ResidentialLow,
            ZoneType::ResidentialMedium,
            ZoneType::ResidentialHigh,
            ZoneType::CommercialLow,
            ZoneType::CommercialHigh,
            ZoneType::Industrial,
            ZoneType::Office,
            ZoneType::MixedUse,
        ];
        for zt in types {
            let bzt: BlueprintZoneType = zt.into();
            let back: ZoneType = bzt.into();
            assert_eq!(zt, back);
        }
    }

    #[test]
    fn test_blueprint_library_add_and_get() {
        let mut lib = BlueprintLibrary::default();
        assert!(lib.is_empty());
        assert_eq!(lib.count(), 0);

        let bp = Blueprint {
            name: "Test".to_string(),
            width: 10,
            height: 10,
            segments: vec![],
            zone_cells: vec![],
        };
        let idx = lib.add(bp);
        assert_eq!(idx, 0);
        assert_eq!(lib.count(), 1);
        assert!(!lib.is_empty());
        assert_eq!(lib.get(0).unwrap().name, "Test");
    }

    #[test]
    fn test_blueprint_library_remove() {
        let mut lib = BlueprintLibrary::default();
        lib.add(Blueprint {
            name: "A".to_string(),
            width: 5,
            height: 5,
            segments: vec![],
            zone_cells: vec![],
        });
        lib.add(Blueprint {
            name: "B".to_string(),
            width: 10,
            height: 10,
            segments: vec![],
            zone_cells: vec![],
        });
        assert_eq!(lib.count(), 2);

        let removed = lib.remove(0);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().name, "A");
        assert_eq!(lib.count(), 1);
        assert_eq!(lib.get(0).unwrap().name, "B");
    }

    #[test]
    fn test_blueprint_library_remove_out_of_bounds() {
        let mut lib = BlueprintLibrary::default();
        assert!(lib.remove(0).is_none());
        assert!(lib.remove(100).is_none());
    }

    #[test]
    fn test_saveable_roundtrip() {
        let mut lib = BlueprintLibrary::default();
        lib.add(Blueprint {
            name: "Grid Layout".to_string(),
            width: 20,
            height: 15,
            segments: vec![BlueprintSegment {
                p0: [0.0, 0.0],
                p1: [50.0, 0.0],
                p2: [100.0, 0.0],
                p3: [150.0, 0.0],
                road_type: BlueprintRoadType::Avenue,
            }],
            zone_cells: vec![
                BlueprintZoneCell {
                    dx: 1,
                    dy: 0,
                    zone_type: BlueprintZoneType::ResidentialLow,
                },
                BlueprintZoneCell {
                    dx: 2,
                    dy: 1,
                    zone_type: BlueprintZoneType::CommercialHigh,
                },
            ],
        });

        let bytes = lib.save_to_bytes().expect("should produce bytes");
        let restored = BlueprintLibrary::load_from_bytes(&bytes);
        assert_eq!(restored.count(), 1);
        let bp = restored.get(0).unwrap();
        assert_eq!(bp.name, "Grid Layout");
        assert_eq!(bp.width, 20);
        assert_eq!(bp.height, 15);
        assert_eq!(bp.segments.len(), 1);
        assert_eq!(bp.segments[0].road_type, BlueprintRoadType::Avenue);
        assert_eq!(bp.zone_cells.len(), 2);
        assert_eq!(
            bp.zone_cells[0].zone_type,
            BlueprintZoneType::ResidentialLow
        );
        assert_eq!(
            bp.zone_cells[1].zone_type,
            BlueprintZoneType::CommercialHigh
        );
    }

    #[test]
    fn test_saveable_empty_returns_none() {
        let lib = BlueprintLibrary::default();
        assert!(lib.save_to_bytes().is_none());
    }

    #[test]
    fn test_capture_empty_region() {
        let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let segments = RoadSegmentStore::default();
        let bp = Blueprint::capture(&grid, &segments, 10, 10, 5, 5, "Empty".to_string());
        assert_eq!(bp.name, "Empty");
        assert_eq!(bp.width, 5);
        assert_eq!(bp.height, 5);
        assert!(bp.segments.is_empty());
        assert!(bp.zone_cells.is_empty());
    }

    #[test]
    fn test_capture_zone_cells() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(12, 13).zone = ZoneType::ResidentialLow;
        grid.get_mut(14, 15).zone = ZoneType::Industrial;

        let segments = RoadSegmentStore::default();
        let bp = Blueprint::capture(&grid, &segments, 10, 10, 10, 10, "Zoned".to_string());
        assert_eq!(bp.zone_cells.len(), 2);

        // Check relative coordinates
        let zc0 = &bp.zone_cells[0];
        assert_eq!(zc0.dx, 2); // 12 - 10
        assert_eq!(zc0.dy, 3); // 13 - 10
        assert_eq!(zc0.zone_type, BlueprintZoneType::ResidentialLow);

        let zc1 = &bp.zone_cells[1];
        assert_eq!(zc1.dx, 4); // 14 - 10
        assert_eq!(zc1.dy, 5); // 15 - 10
        assert_eq!(zc1.zone_type, BlueprintZoneType::Industrial);
    }

    #[test]
    fn test_place_zones_on_grass() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut segments = RoadSegmentStore::default();
        let mut roads = RoadNetwork::default();

        let bp = Blueprint {
            name: "Test".to_string(),
            width: 5,
            height: 5,
            segments: vec![],
            zone_cells: vec![
                BlueprintZoneCell {
                    dx: 0,
                    dy: 0,
                    zone_type: BlueprintZoneType::ResidentialLow,
                },
                BlueprintZoneCell {
                    dx: 1,
                    dy: 1,
                    zone_type: BlueprintZoneType::CommercialLow,
                },
            ],
        };

        let result = bp.place(&mut grid, &mut segments, &mut roads, 50, 50);
        assert_eq!(result.zones_placed, 2);
        assert_eq!(grid.get(50, 50).zone, ZoneType::ResidentialLow);
        assert_eq!(grid.get(51, 51).zone, ZoneType::CommercialLow);
    }

    #[test]
    fn test_place_zones_skip_water() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        grid.get_mut(50, 50).cell_type = CellType::Water;
        let mut segments = RoadSegmentStore::default();
        let mut roads = RoadNetwork::default();

        let bp = Blueprint {
            name: "Test".to_string(),
            width: 5,
            height: 5,
            segments: vec![],
            zone_cells: vec![BlueprintZoneCell {
                dx: 0,
                dy: 0,
                zone_type: BlueprintZoneType::ResidentialLow,
            }],
        };

        let result = bp.place(&mut grid, &mut segments, &mut roads, 50, 50);
        assert_eq!(result.zones_placed, 0);
        assert_eq!(grid.get(50, 50).zone, ZoneType::None);
    }

    #[test]
    fn test_place_out_of_bounds_skipped() {
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        let mut segments = RoadSegmentStore::default();
        let mut roads = RoadNetwork::default();

        let bp = Blueprint {
            name: "Edge".to_string(),
            width: 10,
            height: 10,
            segments: vec![],
            zone_cells: vec![BlueprintZoneCell {
                dx: 5,
                dy: 5,
                zone_type: BlueprintZoneType::Industrial,
            }],
        };

        // Place near the edge so dx=5 goes out of bounds (255 + 5 = 260 > 255)
        let result = bp.place(&mut grid, &mut segments, &mut roads, 255, 255);
        assert_eq!(result.zones_placed, 0);
    }
}
