use bevy::prelude::*;

use simulation::bulldoze_refund;
use simulation::config::CELL_SIZE;
use simulation::economy::CityBudget;
use simulation::grid::{RoadType, WorldGrid, ZoneType};
use simulation::road_segments::RoadSegmentStore;
use simulation::roads::RoadNetwork;
use simulation::services::{self, ServiceBuilding, ServiceType};
use simulation::urban_growth_boundary::UrbanGrowthBoundary;
use simulation::utilities::{UtilitySource, UtilityType};

use crate::angle_snap::AngleSnapState;
use crate::terrain_render::{mark_chunk_dirty_at, ChunkDirty, TerrainChunk};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Resource)]
pub enum ActiveTool {
    Road,
    RoadAvenue,
    RoadBoulevard,
    RoadHighway,
    RoadOneWay,
    RoadPath,
    Bulldoze,
    #[default]
    Inspect,
    ZoneResidentialLow,
    ZoneResidentialMedium,
    ZoneResidentialHigh,
    ZoneCommercialLow,
    ZoneCommercialHigh,
    ZoneIndustrial,
    ZoneOffice,
    ZoneMixedUse,
    PlacePowerPlant,
    PlaceSolarFarm,
    PlaceWindTurbine,
    PlaceWaterTower,
    PlaceSewagePlant,
    PlaceNuclearPlant,
    PlaceGeothermal,
    PlacePumpingStation,
    PlaceWaterTreatment,
    PlaceFireStation,
    PlaceFireHouse,
    PlaceFireHQ,
    PlacePoliceStation,
    PlacePoliceKiosk,
    PlacePoliceHQ,
    PlacePrison,
    PlaceHospital,
    PlaceMedicalClinic,
    PlaceMedicalCenter,
    PlaceElementarySchool,
    PlaceHighSchool,
    PlaceUniversity,
    PlaceLibrary,
    PlaceKindergarten,
    PlaceSmallPark,
    PlaceLargePark,
    PlacePlayground,
    PlacePlaza,
    PlaceSportsField,
    PlaceStadium,
    PlaceLandfill,
    PlaceRecyclingCenter,
    PlaceIncinerator,
    PlaceTransferStation,
    PlaceCemetery,
    PlaceCrematorium,
    PlaceCityHall,
    PlaceMuseum,
    PlaceCathedral,
    PlaceTVStation,
    PlaceBusDepot,
    PlaceTrainStation,
    PlaceSubwayStation,
    PlaceTramDepot,
    PlaceFerryPier,
    PlaceSmallAirstrip,
    PlaceRegionalAirport,
    PlaceInternationalAirport,
    PlaceCellTower,
    PlaceDataCenter,
    // Terrain tools
    TerrainRaise,
    TerrainLower,
    TerrainLevel,
    TerrainWater,
    // District tools
    DistrictPaint(usize),
    DistrictErase,
    // Environment tools
    TreePlant,
    TreeRemove,
    // Road upgrade tool
    RoadUpgrade,
}

impl ActiveTool {
    /// Returns the cost of the active tool, or None for free tools
    pub fn cost(&self) -> Option<f64> {
        match self {
            ActiveTool::Road => Some(RoadType::Local.cost()),
            ActiveTool::RoadAvenue => Some(RoadType::Avenue.cost()),
            ActiveTool::RoadBoulevard => Some(RoadType::Boulevard.cost()),
            ActiveTool::RoadHighway => Some(RoadType::Highway.cost()),
            ActiveTool::RoadOneWay => Some(RoadType::OneWay.cost()),
            ActiveTool::RoadPath => Some(RoadType::Path.cost()),
            ActiveTool::Bulldoze
            | ActiveTool::Inspect
            | ActiveTool::TerrainRaise
            | ActiveTool::TerrainLower
            | ActiveTool::TerrainLevel
            | ActiveTool::TerrainWater
            | ActiveTool::DistrictPaint(_)
            | ActiveTool::DistrictErase
            | ActiveTool::TreeRemove
            | ActiveTool::RoadUpgrade => None,
            ActiveTool::TreePlant => Some(simulation::trees::TREE_PLANT_COST),
            ActiveTool::ZoneResidentialLow
            | ActiveTool::ZoneResidentialMedium
            | ActiveTool::ZoneResidentialHigh
            | ActiveTool::ZoneCommercialLow
            | ActiveTool::ZoneCommercialHigh
            | ActiveTool::ZoneIndustrial
            | ActiveTool::ZoneOffice
            | ActiveTool::ZoneMixedUse => None,
            // Utilities
            ActiveTool::PlacePowerPlant => Some(services::utility_cost(UtilityType::PowerPlant)),
            ActiveTool::PlaceSolarFarm => Some(services::utility_cost(UtilityType::SolarFarm)),
            ActiveTool::PlaceWindTurbine => Some(services::utility_cost(UtilityType::WindTurbine)),
            ActiveTool::PlaceWaterTower => Some(services::utility_cost(UtilityType::WaterTower)),
            ActiveTool::PlaceSewagePlant => Some(services::utility_cost(UtilityType::SewagePlant)),
            ActiveTool::PlaceNuclearPlant => {
                Some(services::utility_cost(UtilityType::NuclearPlant))
            }
            ActiveTool::PlaceGeothermal => Some(services::utility_cost(UtilityType::Geothermal)),
            ActiveTool::PlacePumpingStation => {
                Some(services::utility_cost(UtilityType::PumpingStation))
            }
            ActiveTool::PlaceWaterTreatment => {
                Some(services::utility_cost(UtilityType::WaterTreatment))
            }
            // Services
            _ => self.service_type().map(services::ServiceBuilding::cost),
        }
    }

    /// Map tool to ServiceType if applicable
    pub fn service_type(&self) -> Option<ServiceType> {
        match self {
            ActiveTool::PlaceFireStation => Some(ServiceType::FireStation),
            ActiveTool::PlaceFireHouse => Some(ServiceType::FireHouse),
            ActiveTool::PlaceFireHQ => Some(ServiceType::FireHQ),
            ActiveTool::PlacePoliceStation => Some(ServiceType::PoliceStation),
            ActiveTool::PlacePoliceKiosk => Some(ServiceType::PoliceKiosk),
            ActiveTool::PlacePoliceHQ => Some(ServiceType::PoliceHQ),
            ActiveTool::PlacePrison => Some(ServiceType::Prison),
            ActiveTool::PlaceHospital => Some(ServiceType::Hospital),
            ActiveTool::PlaceMedicalClinic => Some(ServiceType::MedicalClinic),
            ActiveTool::PlaceMedicalCenter => Some(ServiceType::MedicalCenter),
            ActiveTool::PlaceElementarySchool => Some(ServiceType::ElementarySchool),
            ActiveTool::PlaceHighSchool => Some(ServiceType::HighSchool),
            ActiveTool::PlaceUniversity => Some(ServiceType::University),
            ActiveTool::PlaceLibrary => Some(ServiceType::Library),
            ActiveTool::PlaceKindergarten => Some(ServiceType::Kindergarten),
            ActiveTool::PlaceSmallPark => Some(ServiceType::SmallPark),
            ActiveTool::PlaceLargePark => Some(ServiceType::LargePark),
            ActiveTool::PlacePlayground => Some(ServiceType::Playground),
            ActiveTool::PlacePlaza => Some(ServiceType::Plaza),
            ActiveTool::PlaceSportsField => Some(ServiceType::SportsField),
            ActiveTool::PlaceStadium => Some(ServiceType::Stadium),
            ActiveTool::PlaceLandfill => Some(ServiceType::Landfill),
            ActiveTool::PlaceRecyclingCenter => Some(ServiceType::RecyclingCenter),
            ActiveTool::PlaceIncinerator => Some(ServiceType::Incinerator),
            ActiveTool::PlaceTransferStation => Some(ServiceType::TransferStation),
            ActiveTool::PlaceCemetery => Some(ServiceType::Cemetery),
            ActiveTool::PlaceCrematorium => Some(ServiceType::Crematorium),
            ActiveTool::PlaceCityHall => Some(ServiceType::CityHall),
            ActiveTool::PlaceMuseum => Some(ServiceType::Museum),
            ActiveTool::PlaceCathedral => Some(ServiceType::Cathedral),
            ActiveTool::PlaceTVStation => Some(ServiceType::TVStation),
            ActiveTool::PlaceBusDepot => Some(ServiceType::BusDepot),
            ActiveTool::PlaceTrainStation => Some(ServiceType::TrainStation),
            ActiveTool::PlaceSubwayStation => Some(ServiceType::SubwayStation),
            ActiveTool::PlaceTramDepot => Some(ServiceType::TramDepot),
            ActiveTool::PlaceFerryPier => Some(ServiceType::FerryPier),
            ActiveTool::PlaceSmallAirstrip => Some(ServiceType::SmallAirstrip),
            ActiveTool::PlaceRegionalAirport => Some(ServiceType::RegionalAirport),
            ActiveTool::PlaceInternationalAirport => Some(ServiceType::InternationalAirport),
            ActiveTool::PlaceCellTower => Some(ServiceType::CellTower),
            ActiveTool::PlaceDataCenter => Some(ServiceType::DataCenter),
            _ => None,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ActiveTool::Road => "Local Road",
            ActiveTool::RoadAvenue => "Avenue",
            ActiveTool::RoadBoulevard => "Boulevard",
            ActiveTool::RoadHighway => "Highway",
            ActiveTool::RoadOneWay => "One-Way",
            ActiveTool::RoadPath => "Path",
            ActiveTool::Bulldoze => "Bulldoze",
            ActiveTool::Inspect => "Inspect",
            ActiveTool::ZoneResidentialLow => "Low-Density Residential",
            ActiveTool::ZoneResidentialMedium => "Medium-Density Residential",
            ActiveTool::ZoneResidentialHigh => "High-Density Residential",
            ActiveTool::ZoneCommercialLow => "Low-Density Commercial",
            ActiveTool::ZoneCommercialHigh => "High-Density Commercial",
            ActiveTool::ZoneIndustrial => "Industrial",
            ActiveTool::ZoneOffice => "Office",
            ActiveTool::ZoneMixedUse => "Mixed-Use",
            ActiveTool::PlacePowerPlant => "Power Plant",
            ActiveTool::PlaceSolarFarm => "Solar Farm",
            ActiveTool::PlaceWindTurbine => "Wind Turbine",
            ActiveTool::PlaceWaterTower => "Water Tower",
            ActiveTool::PlaceSewagePlant => "Sewage Plant",
            ActiveTool::PlaceNuclearPlant => "Nuclear Plant",
            ActiveTool::PlaceGeothermal => "Geothermal",
            ActiveTool::PlacePumpingStation => "Pumping Station",
            ActiveTool::PlaceWaterTreatment => "Water Treatment",
            ActiveTool::PlaceFireStation => "Fire Station",
            ActiveTool::PlaceFireHouse => "Fire House",
            ActiveTool::PlaceFireHQ => "Fire HQ",
            ActiveTool::PlacePoliceStation => "Police Station",
            ActiveTool::PlacePoliceKiosk => "Police Kiosk",
            ActiveTool::PlacePoliceHQ => "Police HQ",
            ActiveTool::PlacePrison => "Prison",
            ActiveTool::PlaceHospital => "Hospital",
            ActiveTool::PlaceMedicalClinic => "Medical Clinic",
            ActiveTool::PlaceMedicalCenter => "Medical Center",
            ActiveTool::PlaceElementarySchool => "Elementary School",
            ActiveTool::PlaceHighSchool => "High School",
            ActiveTool::PlaceUniversity => "University",
            ActiveTool::PlaceLibrary => "Library",
            ActiveTool::PlaceKindergarten => "Kindergarten",
            ActiveTool::PlaceSmallPark => "Small Park",
            ActiveTool::PlaceLargePark => "Large Park",
            ActiveTool::PlacePlayground => "Playground",
            ActiveTool::PlacePlaza => "Plaza",
            ActiveTool::PlaceSportsField => "Sports Field",
            ActiveTool::PlaceStadium => "Stadium",
            ActiveTool::PlaceLandfill => "Landfill",
            ActiveTool::PlaceRecyclingCenter => "Recycling Center",
            ActiveTool::PlaceIncinerator => "Incinerator",
            ActiveTool::PlaceTransferStation => "Transfer Station",
            ActiveTool::PlaceCemetery => "Cemetery",
            ActiveTool::PlaceCrematorium => "Crematorium",
            ActiveTool::PlaceCityHall => "City Hall",
            ActiveTool::PlaceMuseum => "Museum",
            ActiveTool::PlaceCathedral => "Cathedral",
            ActiveTool::PlaceTVStation => "TV Station",
            ActiveTool::PlaceBusDepot => "Bus Depot",
            ActiveTool::PlaceTrainStation => "Train Station",
            ActiveTool::PlaceSubwayStation => "Subway Station",
            ActiveTool::PlaceTramDepot => "Tram Depot",
            ActiveTool::PlaceFerryPier => "Ferry Pier",
            ActiveTool::PlaceSmallAirstrip => "Small Airstrip",
            ActiveTool::PlaceRegionalAirport => "Regional Airport",
            ActiveTool::PlaceInternationalAirport => "Int'l Airport",
            ActiveTool::PlaceCellTower => "Cell Tower",
            ActiveTool::PlaceDataCenter => "Data Center",
            ActiveTool::TerrainRaise => "Raise Terrain",
            ActiveTool::TerrainLower => "Lower Terrain",
            ActiveTool::TerrainLevel => "Level Terrain",
            ActiveTool::TerrainWater => "Place Water",
            ActiveTool::DistrictPaint(_) => "Paint District",
            ActiveTool::DistrictErase => "Erase District",
            ActiveTool::TreePlant => "Plant Tree",
            ActiveTool::TreeRemove => "Remove Tree",
            ActiveTool::RoadUpgrade => "Upgrade Road",
        }
    }

    /// Returns the `RoadType` for road tools, or `None` for non-road tools.
    pub fn road_type(&self) -> Option<RoadType> {
        match self {
            ActiveTool::Road => Some(RoadType::Local),
            ActiveTool::RoadAvenue => Some(RoadType::Avenue),
            ActiveTool::RoadBoulevard => Some(RoadType::Boulevard),
            ActiveTool::RoadHighway => Some(RoadType::Highway),
            ActiveTool::RoadOneWay => Some(RoadType::OneWay),
            ActiveTool::RoadPath => Some(RoadType::Path),
            _ => None,
        }
    }

    /// Returns the `ZoneType` for zone tools, or `None` for non-zone tools.
    pub fn zone_type(&self) -> Option<ZoneType> {
        match self {
            ActiveTool::ZoneResidentialLow => Some(ZoneType::ResidentialLow),
            ActiveTool::ZoneResidentialMedium => Some(ZoneType::ResidentialMedium),
            ActiveTool::ZoneResidentialHigh => Some(ZoneType::ResidentialHigh),
            ActiveTool::ZoneCommercialLow => Some(ZoneType::CommercialLow),
            ActiveTool::ZoneCommercialHigh => Some(ZoneType::CommercialHigh),
            ActiveTool::ZoneIndustrial => Some(ZoneType::Industrial),
            ActiveTool::ZoneOffice => Some(ZoneType::Office),
            ActiveTool::ZoneMixedUse => Some(ZoneType::MixedUse),
            _ => None,
        }
    }
}

/// Grid snap mode: when enabled, cursor snaps to cell centers for precise placement.
#[derive(Resource, Default)]
pub struct GridSnap {
    pub enabled: bool,
}

#[derive(Resource, Default)]
pub struct CursorGridPos {
    pub grid_x: i32,
    pub grid_y: i32,
    pub world_pos: Vec2,
    pub valid: bool,
}

/// Currently selected building entity for inspection
#[derive(Resource, Default)]
pub struct SelectedBuilding(pub Option<Entity>);

/// Status message shown briefly on screen
#[derive(Resource, Default)]
pub struct StatusMessage {
    pub text: String,
    pub timer: f32,
    pub is_error: bool,
}

impl StatusMessage {
    pub fn set(&mut self, text: impl Into<String>, is_error: bool) {
        self.text = text.into();
        self.timer = 3.0;
        self.is_error = is_error;
    }

    pub fn active(&self) -> bool {
        self.timer > 0.0
    }
}

/// State machine for freeform Bezier road drawing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DrawPhase {
    /// No road drawing in progress.
    Idle,
    /// Start point has been placed; waiting for end point click.
    PlacedStart,
}

#[derive(Resource)]
pub struct RoadDrawState {
    pub phase: DrawPhase,
    pub start_pos: Vec2,
}

impl Default for RoadDrawState {
    fn default() -> Self {
        Self {
            phase: DrawPhase::Idle,
            start_pos: Vec2::ZERO,
        }
    }
}

/// Snap radius for intersection snapping (1 cell distance in world units).
const INTERSECTION_SNAP_RADIUS: f32 = CELL_SIZE;

/// Tracks whether the cursor is currently snapped to an existing intersection node.
#[derive(Resource, Default)]
pub struct IntersectionSnap {
    /// When `Some`, the cursor should snap to this world position.
    pub snapped_pos: Option<Vec2>,
}

/// Each frame, check if the cursor is near an existing segment node (intersection)
/// and update `IntersectionSnap` accordingly.
pub fn update_intersection_snap(
    cursor: Res<CursorGridPos>,
    tool: Res<ActiveTool>,
    segments: Res<RoadSegmentStore>,
    mut snap: ResMut<IntersectionSnap>,
) {
    snap.snapped_pos = None;

    if !cursor.valid {
        return;
    }

    // Only snap for road tools
    let is_road_tool = matches!(
        *tool,
        ActiveTool::Road
            | ActiveTool::RoadAvenue
            | ActiveTool::RoadBoulevard
            | ActiveTool::RoadHighway
            | ActiveTool::RoadOneWay
            | ActiveTool::RoadPath
    );
    if !is_road_tool {
        return;
    }

    let cursor_pos = cursor.world_pos;
    let mut best_dist = INTERSECTION_SNAP_RADIUS;
    let mut best_pos: Option<Vec2> = None;

    for node in &segments.nodes {
        let dist = (node.position - cursor_pos).length();
        if dist < best_dist {
            best_dist = dist;
            best_pos = Some(node.position);
        }
    }

    snap.snapped_pos = best_pos;
}

pub fn tick_status_message(time: Res<Time>, mut status: ResMut<StatusMessage>) {
    if status.timer > 0.0 {
        status.timer -= time.delta_secs();
    }
}

pub fn update_cursor_grid_pos(
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut cursor: ResMut<CursorGridPos>,
    grid: Res<WorldGrid>,
    grid_snap: Res<GridSnap>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };
    let Ok((camera, cam_transform)) = camera_q.get_single() else {
        return;
    };

    if let Some(screen_pos) = window.cursor_position() {
        // Ray-plane intersection against Y=0 ground plane
        if let Ok(ray) = camera.viewport_to_world(cam_transform, screen_pos) {
            if ray.direction.y.abs() > 0.001 {
                let t = -ray.origin.y / ray.direction.y;
                if t > 0.0 {
                    let hit = ray.origin + ray.direction * t;
                    // 3D: hit.x -> grid X, hit.z -> grid Y
                    let (gx, gy) = WorldGrid::world_to_grid(hit.x, hit.z);

                    // When grid snap is enabled, snap world_pos to the cell center
                    if grid_snap.enabled && gx >= 0 && gy >= 0 {
                        let (cx, cz) = WorldGrid::grid_to_world(gx as usize, gy as usize);
                        cursor.world_pos = Vec2::new(cx, cz);
                    } else {
                        cursor.world_pos = Vec2::new(hit.x, hit.z);
                    }

                    cursor.grid_x = gx;
                    cursor.grid_y = gy;
                    cursor.valid = gx >= 0 && gy >= 0 && grid.in_bounds(gx as usize, gy as usize);
                    return;
                }
            }
        }
        cursor.valid = false;
    } else {
        cursor.valid = false;
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn handle_tool_input(
    input: (
        Res<ButtonInput<MouseButton>>,
        Res<ButtonInput<KeyCode>>,
        Res<AngleSnapState>,
    ),
    cursor: Res<CursorGridPos>,
    tool: Res<ActiveTool>,
    mut grid: ResMut<WorldGrid>,
    mut roads: ResMut<RoadNetwork>,
    mut segments: ResMut<RoadSegmentStore>,
    mut budget: ResMut<CityBudget>,
    mut status: ResMut<StatusMessage>,
    mut selected: ResMut<SelectedBuilding>,
    mut draw_state: ResMut<RoadDrawState>,
    chunks: Query<(Entity, &TerrainChunk), Without<ChunkDirty>>,
    mut commands: Commands,
    service_q: Query<&ServiceBuilding>,
    utility_q: Query<&UtilitySource>,
    misc: (
        Res<crate::camera::LeftClickDrag>,
        Res<UrbanGrowthBoundary>,
        Res<IntersectionSnap>,
        Res<crate::zone_brush_preview::ZoneBrushSize>,
        Res<simulation::freehand_road::FreehandDrawState>,
    ),
    mut district_map: ResMut<simulation::districts::DistrictMap>,
) {
    let (buttons, keys, angle_snap) = input;
    let (left_drag, ugb, snap, brush_size, freehand) = misc;

    // Suppress tool actions when left-click is being used for camera panning
    if left_drag.is_dragging {
        return;
    }

    // Right click cancels drawing
    if buttons.just_pressed(MouseButton::Right) {
        draw_state.phase = DrawPhase::Idle;
    }

    if !buttons.pressed(MouseButton::Left) || !cursor.valid {
        return;
    }

    let gx = cursor.grid_x as usize;
    let gy = cursor.grid_y as usize;

    // Update selected building on click for non-Inspect tools.
    // In Inspect mode, the enhanced_select system handles priority-based selection.
    if buttons.just_pressed(MouseButton::Left) && *tool != ActiveTool::Inspect {
        selected.0 = grid.get(gx, gy).building_id;
    }

    // Check if Ctrl is held for legacy grid-snap mode
    let ctrl_held = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);

    // Skip freeform road drawing when freehand mode is active (handled by freehand_draw system)
    let freehand_active = freehand.enabled;

    // Determine if this is a freeform road tool
    let freeform_road_type = if !ctrl_held && !freehand_active {
        match *tool {
            ActiveTool::Road => Some(RoadType::Local),
            ActiveTool::RoadAvenue => Some(RoadType::Avenue),
            ActiveTool::RoadBoulevard => Some(RoadType::Boulevard),
            ActiveTool::RoadHighway => Some(RoadType::Highway),
            ActiveTool::RoadOneWay => Some(RoadType::OneWay),
            ActiveTool::RoadPath => Some(RoadType::Path),
            _ => None,
        }
    } else {
        None
    };

    // Handle freeform Bezier road drawing
    if let Some(road_type) = freeform_road_type {
        if buttons.just_pressed(MouseButton::Left) {
            match draw_state.phase {
                DrawPhase::Idle => {
                    // First click: place start point (snap to intersection if close)
                    draw_state.start_pos = snap.snapped_pos.unwrap_or(cursor.world_pos);
                    draw_state.phase = DrawPhase::PlacedStart;
                    status.set(
                        "Click to place end point (Shift=snap angle, Esc=cancel)",
                        false,
                    );
                }
                DrawPhase::PlacedStart => {
                    // Second click: place end point and commit segment
                    // Intersection snap takes precedence over angle snap
                    let end_pos = if let Some(snapped) = snap.snapped_pos {
                        snapped
                    } else if angle_snap.active {
                        angle_snap.snapped_pos
                    } else {
                        cursor.world_pos
                    };
                    let start_pos = draw_state.start_pos;

                    // Minimum length check
                    if (end_pos - start_pos).length() < CELL_SIZE {
                        status.set("Road too short", true);
                        return;
                    }

                    // Estimate cost based on arc length
                    let approx_cells = ((end_pos - start_pos).length() / CELL_SIZE).ceil() as usize;
                    let total_cost = road_type.cost() * approx_cells as f64;
                    if budget.treasury < total_cost {
                        status.set("Not enough money", true);
                        return;
                    }

                    let (_seg_id, cells) = segments.add_straight_segment(
                        start_pos, end_pos, road_type, 24.0, &mut grid, &mut roads,
                    );

                    let actual_cost = road_type.cost() * cells.len() as f64;
                    budget.treasury -= actual_cost;

                    // Mark dirty chunks for all affected cells
                    for &(cx, cy) in &cells {
                        mark_chunk_dirty_at(cx, cy, &chunks, &mut commands);
                    }

                    // Chain: end becomes new start for next segment
                    draw_state.start_pos = end_pos;
                    // Stay in PlacedStart phase for chaining
                }
            }
        }
        return;
    }

    // Reset draw state when using non-road tools
    draw_state.phase = DrawPhase::Idle;

    let changed = match *tool {
        // Roads (legacy grid-snap with Ctrl held)
        ActiveTool::Road => place_road_if_affordable(
            &mut roads,
            &mut grid,
            &mut budget,
            &mut status,
            &buttons,
            RoadType::Local,
            gx,
            gy,
        ),
        ActiveTool::RoadAvenue => place_road_if_affordable(
            &mut roads,
            &mut grid,
            &mut budget,
            &mut status,
            &buttons,
            RoadType::Avenue,
            gx,
            gy,
        ),
        ActiveTool::RoadBoulevard => place_road_if_affordable(
            &mut roads,
            &mut grid,
            &mut budget,
            &mut status,
            &buttons,
            RoadType::Boulevard,
            gx,
            gy,
        ),
        ActiveTool::RoadHighway => place_road_if_affordable(
            &mut roads,
            &mut grid,
            &mut budget,
            &mut status,
            &buttons,
            RoadType::Highway,
            gx,
            gy,
        ),
        ActiveTool::RoadOneWay => place_road_if_affordable(
            &mut roads,
            &mut grid,
            &mut budget,
            &mut status,
            &buttons,
            RoadType::OneWay,
            gx,
            gy,
        ),
        ActiveTool::RoadPath => place_road_if_affordable(
            &mut roads,
            &mut grid,
            &mut budget,
            &mut status,
            &buttons,
            RoadType::Path,
            gx,
            gy,
        ),
        ActiveTool::Bulldoze => {
            let cell = grid.get(gx, gy);
            if let Some(entity) = cell.building_id {
                // Compute refund before clearing the entity
                let refund = if let Ok(service) = service_q.get(entity) {
                    let (fw, fh) = ServiceBuilding::footprint(service.service_type);
                    let sx = service.grid_x;
                    let sy = service.grid_y;
                    for fy in sy..sy + fh {
                        for fx in sx..sx + fw {
                            if grid.in_bounds(fx, fy) {
                                grid.get_mut(fx, fy).building_id = None;
                                grid.get_mut(fx, fy).zone = ZoneType::None;
                                mark_chunk_dirty_at(fx, fy, &chunks, &mut commands);
                            }
                        }
                    }
                    bulldoze_refund::refund_for_service(service.service_type)
                } else if let Ok(utility) = utility_q.get(entity) {
                    grid.get_mut(gx, gy).building_id = None;
                    grid.get_mut(gx, gy).zone = ZoneType::None;
                    bulldoze_refund::refund_for_utility(utility.utility_type)
                } else {
                    grid.get_mut(gx, gy).building_id = None;
                    grid.get_mut(gx, gy).zone = ZoneType::None;
                    0.0
                };
                budget.treasury += refund;
                // Despawn the entity so mesh cleanup picks it up
                commands.entity(entity).despawn();
                true
            } else if cell.zone != ZoneType::None {
                grid.get_mut(gx, gy).zone = ZoneType::None;
                true
            } else if cell.cell_type == simulation::grid::CellType::Road {
                let road_type = cell.road_type;
                if roads.remove_road(&mut grid, gx, gy) {
                    budget.treasury += bulldoze_refund::refund_for_road(road_type);
                    true
                } else {
                    false
                }
            } else {
                false
            }
        }
        ActiveTool::Inspect => {
            // Selection is handled by the enhanced_select system (UX-009)
            // which provides priority-based selection:
            // citizens > buildings > roads > cells.
            false
        }

        // --- Zones (with brush size support) ---
        ActiveTool::ZoneResidentialLow
        | ActiveTool::ZoneResidentialMedium
        | ActiveTool::ZoneResidentialHigh
        | ActiveTool::ZoneCommercialLow
        | ActiveTool::ZoneCommercialHigh
        | ActiveTool::ZoneIndustrial
        | ActiveTool::ZoneOffice
        | ActiveTool::ZoneMixedUse => {
            let zone = tool.zone_type().unwrap();
            let zoned_cells = apply_zone_brush(
                &mut grid,
                &mut status,
                &mut budget,
                &buttons,
                gx as i32,
                gy as i32,
                zone,
                &ugb,
                &brush_size,
            );
            for (zx, zy) in &zoned_cells {
                mark_chunk_dirty_at(*zx, *zy, &chunks, &mut commands);
            }
            !zoned_cells.is_empty()
        }

        // --- Utilities ---
        ActiveTool::PlacePowerPlant => place_utility_if_affordable(
            &mut commands,
            &mut grid,
            &mut budget,
            &mut status,
            &buttons,
            UtilityType::PowerPlant,
            gx,
            gy,
        ),
        ActiveTool::PlaceSolarFarm => place_utility_if_affordable(
            &mut commands,
            &mut grid,
            &mut budget,
            &mut status,
            &buttons,
            UtilityType::SolarFarm,
            gx,
            gy,
        ),
        ActiveTool::PlaceWindTurbine => place_utility_if_affordable(
            &mut commands,
            &mut grid,
            &mut budget,
            &mut status,
            &buttons,
            UtilityType::WindTurbine,
            gx,
            gy,
        ),
        ActiveTool::PlaceWaterTower => place_utility_if_affordable(
            &mut commands,
            &mut grid,
            &mut budget,
            &mut status,
            &buttons,
            UtilityType::WaterTower,
            gx,
            gy,
        ),
        ActiveTool::PlaceSewagePlant => place_utility_if_affordable(
            &mut commands,
            &mut grid,
            &mut budget,
            &mut status,
            &buttons,
            UtilityType::SewagePlant,
            gx,
            gy,
        ),
        ActiveTool::PlaceNuclearPlant => place_utility_if_affordable(
            &mut commands,
            &mut grid,
            &mut budget,
            &mut status,
            &buttons,
            UtilityType::NuclearPlant,
            gx,
            gy,
        ),
        ActiveTool::PlaceGeothermal => place_utility_if_affordable(
            &mut commands,
            &mut grid,
            &mut budget,
            &mut status,
            &buttons,
            UtilityType::Geothermal,
            gx,
            gy,
        ),
        ActiveTool::PlacePumpingStation => place_utility_if_affordable(
            &mut commands,
            &mut grid,
            &mut budget,
            &mut status,
            &buttons,
            UtilityType::PumpingStation,
            gx,
            gy,
        ),
        ActiveTool::PlaceWaterTreatment => place_utility_if_affordable(
            &mut commands,
            &mut grid,
            &mut budget,
            &mut status,
            &buttons,
            UtilityType::WaterTreatment,
            gx,
            gy,
        ),

        // --- Terrain tools ---
        ActiveTool::TerrainRaise => {
            let radius = 3i32;
            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    let nx = gx as i32 + dx;
                    let ny = gy as i32 + dy;
                    if nx >= 0
                        && ny >= 0
                        && (nx as usize) < grid.width
                        && (ny as usize) < grid.height
                    {
                        let dist = ((dx * dx + dy * dy) as f32).sqrt();
                        if dist <= radius as f32 {
                            let strength = 0.01 * (1.0 - dist / radius as f32);
                            let cell = grid.get_mut(nx as usize, ny as usize);
                            cell.elevation = (cell.elevation + strength).min(1.0);
                            if cell.elevation > 0.35
                                && cell.cell_type == simulation::grid::CellType::Water
                            {
                                cell.cell_type = simulation::grid::CellType::Grass;
                            }
                            mark_chunk_dirty_at(nx as usize, ny as usize, &chunks, &mut commands);
                        }
                    }
                }
            }
            true
        }
        ActiveTool::TerrainLower => {
            let radius = 3i32;
            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    let nx = gx as i32 + dx;
                    let ny = gy as i32 + dy;
                    if nx >= 0
                        && ny >= 0
                        && (nx as usize) < grid.width
                        && (ny as usize) < grid.height
                    {
                        let dist = ((dx * dx + dy * dy) as f32).sqrt();
                        if dist <= radius as f32 {
                            let strength = 0.01 * (1.0 - dist / radius as f32);
                            let cell = grid.get_mut(nx as usize, ny as usize);
                            cell.elevation = (cell.elevation - strength).max(0.0);
                            mark_chunk_dirty_at(nx as usize, ny as usize, &chunks, &mut commands);
                        }
                    }
                }
            }
            true
        }
        ActiveTool::TerrainLevel => {
            let target_elev = grid.get(gx, gy).elevation;
            let radius = 3i32;
            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    let nx = gx as i32 + dx;
                    let ny = gy as i32 + dy;
                    if nx >= 0
                        && ny >= 0
                        && (nx as usize) < grid.width
                        && (ny as usize) < grid.height
                    {
                        let dist = ((dx * dx + dy * dy) as f32).sqrt();
                        if dist <= radius as f32 {
                            let cell = grid.get_mut(nx as usize, ny as usize);
                            cell.elevation += (target_elev - cell.elevation) * 0.3;
                            mark_chunk_dirty_at(nx as usize, ny as usize, &chunks, &mut commands);
                        }
                    }
                }
            }
            true
        }
        ActiveTool::TerrainWater => {
            let radius = 2i32;
            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    let nx = gx as i32 + dx;
                    let ny = gy as i32 + dy;
                    if nx >= 0
                        && ny >= 0
                        && (nx as usize) < grid.width
                        && (ny as usize) < grid.height
                    {
                        let dist = ((dx * dx + dy * dy) as f32).sqrt();
                        if dist <= radius as f32 {
                            let cell = grid.get_mut(nx as usize, ny as usize);
                            cell.cell_type = simulation::grid::CellType::Water;
                            cell.elevation = 0.3;
                            mark_chunk_dirty_at(nx as usize, ny as usize, &chunks, &mut commands);
                        }
                    }
                }
            }
            true
        }

        // --- Trees (handled by separate system to stay within param limit) ---
        ActiveTool::TreePlant | ActiveTool::TreeRemove | ActiveTool::RoadUpgrade => false,

        // --- Districts ---
        ActiveTool::DistrictPaint(di) => {
            district_map.assign_cell_to_district(gx, gy, di);
            false
        }
        ActiveTool::DistrictErase => {
            district_map.remove_cell_from_district(gx, gy);
            false
        }

        // --- Services (use service_type() helper) ---
        _ => {
            if let Some(st) = tool.service_type() {
                place_service_if_affordable(
                    &mut commands,
                    &mut grid,
                    &mut budget,
                    &mut status,
                    &buttons,
                    st,
                    gx,
                    gy,
                )
            } else {
                false
            }
        }
    };

    if changed {
        mark_chunk_dirty_at(gx, gy, &chunks, &mut commands);
    }
}

// ---------------------------------------------------------------------------
// Helper: road placement with cost
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn place_road_if_affordable(
    roads: &mut RoadNetwork,
    grid: &mut WorldGrid,
    budget: &mut CityBudget,
    status: &mut StatusMessage,
    buttons: &ButtonInput<MouseButton>,
    road_type: RoadType,
    gx: usize,
    gy: usize,
) -> bool {
    let cost = road_type.cost();
    if budget.treasury >= cost {
        if roads.place_road_typed(grid, gx, gy, road_type) {
            budget.treasury -= cost;
            true
        } else {
            false
        }
    } else {
        if buttons.just_pressed(MouseButton::Left) {
            status.set("Not enough money", true);
        }
        false
    }
}

// ---------------------------------------------------------------------------
// Helper: zone placement
// ---------------------------------------------------------------------------

enum ZoneResult {
    Success,
    NotAdjacentToRoad,
    OutsideUgb,
    InvalidCell,
}

fn try_zone(
    grid: &WorldGrid,
    x: usize,
    y: usize,
    zone: ZoneType,
    ugb: &UrbanGrowthBoundary,
) -> ZoneResult {
    let cell = grid.get(x, y);
    if cell.cell_type != simulation::grid::CellType::Grass {
        return ZoneResult::InvalidCell;
    }
    if cell.zone == zone {
        return ZoneResult::InvalidCell;
    }
    // Urban Growth Boundary: block zoning outside the boundary (ZONE-009).
    if !ugb.allows_zoning(x, y) {
        return ZoneResult::OutsideUgb;
    }
    let (n4, n4c) = grid.neighbors4(x, y);
    let has_road = n4[..n4c]
        .iter()
        .any(|(nx, ny)| grid.get(*nx, *ny).cell_type == simulation::grid::CellType::Road);
    if !has_road {
        return ZoneResult::NotAdjacentToRoad;
    }
    ZoneResult::Success
}

#[allow(clippy::too_many_arguments)]
fn apply_zone_brush(
    grid: &mut WorldGrid,
    status: &mut StatusMessage,
    budget: &mut simulation::economy::CityBudget,
    buttons: &ButtonInput<MouseButton>,
    cx: i32,
    cy: i32,
    zone: ZoneType,
    ugb: &UrbanGrowthBoundary,
    brush: &crate::zone_brush_preview::ZoneBrushSize,
) -> Vec<(usize, usize)> {
    let half = brush.half_extent;
    let cost_per_cell = crate::zone_brush_preview::ZONE_COST_PER_CELL;

    // Collect valid cells in the brush area
    let mut valid_cells = Vec::new();
    for dy in -half..=half {
        for dx in -half..=half {
            let gx = cx + dx;
            let gy = cy + dy;
            if gx >= 0 && gy >= 0 {
                let ux = gx as usize;
                let uy = gy as usize;
                if grid.in_bounds(ux, uy) {
                    let result = try_zone(grid, ux, uy, zone, ugb);
                    if matches!(result, ZoneResult::Success) {
                        valid_cells.push((ux, uy));
                    }
                }
            }
        }
    }

    if valid_cells.is_empty() {
        if buttons.just_pressed(MouseButton::Left) {
            // Show reason for center cell
            let ux = cx as usize;
            let uy = cy as usize;
            if grid.in_bounds(ux, uy) {
                match try_zone(grid, ux, uy, zone, ugb) {
                    ZoneResult::NotAdjacentToRoad => {
                        status.set("Zone must be adjacent to road", true);
                    }
                    ZoneResult::OutsideUgb => {
                        status.set("Cannot zone outside urban growth boundary", true);
                    }
                    _ => {}
                }
            }
        }
        return Vec::new();
    }

    let total_cost = valid_cells.len() as f64 * cost_per_cell;
    if budget.treasury < total_cost {
        if buttons.just_pressed(MouseButton::Left) {
            status.set("Not enough money", true);
        }
        return Vec::new();
    }

    // Apply zones to all valid cells
    budget.treasury -= total_cost;
    for (gx, gy) in &valid_cells {
        grid.get_mut(*gx, *gy).zone = zone;
    }
    valid_cells
}

// ---------------------------------------------------------------------------
// Helper: utility placement
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn place_utility_if_affordable(
    commands: &mut Commands,
    grid: &mut WorldGrid,
    budget: &mut CityBudget,
    status: &mut StatusMessage,
    buttons: &ButtonInput<MouseButton>,
    utility_type: UtilityType,
    gx: usize,
    gy: usize,
) -> bool {
    let cost = services::utility_cost(utility_type);
    if budget.treasury >= cost {
        if services::place_utility_source(commands, grid, utility_type, gx, gy) {
            budget.treasury -= cost;
            true
        } else {
            false
        }
    } else {
        if buttons.just_pressed(MouseButton::Left) {
            status.set("Not enough money", true);
        }
        false
    }
}

// ---------------------------------------------------------------------------
// Helper: service placement
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn place_service_if_affordable(
    commands: &mut Commands,
    grid: &mut WorldGrid,
    budget: &mut CityBudget,
    status: &mut StatusMessage,
    buttons: &ButtonInput<MouseButton>,
    service_type: ServiceType,
    gx: usize,
    gy: usize,
) -> bool {
    use simulation::services::ServiceBuilding;
    let cost = ServiceBuilding::cost(service_type);
    if budget.treasury >= cost {
        if services::place_service(commands, grid, service_type, gx, gy) {
            budget.treasury -= cost;
            true
        } else {
            false
        }
    } else {
        if buttons.just_pressed(MouseButton::Left) {
            status.set("Not enough money", true);
        }
        false
    }
}

// ---------------------------------------------------------------------------
// Keyboard shortcuts (core tools only; extended tools via UI toolbar)
// ---------------------------------------------------------------------------

/// Toggle grid snap mode with the F key.
pub fn toggle_grid_snap(
    keys: Res<ButtonInput<KeyCode>>,
    mut grid_snap: ResMut<GridSnap>,
    bindings: Res<simulation::keybindings::KeyBindings>,
) {
    if bindings.toggle_grid_snap.just_pressed(&keys) {
        grid_snap.enabled = !grid_snap.enabled;
    }
}

/// Quick-access tool shortcuts (R/Z/B/I/V).
/// Digit keys 1-3 are reserved for simulation speed; overlays use Tab cycling.
pub fn keyboard_tool_switch(
    keys: Res<ButtonInput<KeyCode>>,
    mut tool: ResMut<ActiveTool>,
    bindings: Res<simulation::keybindings::KeyBindings>,
) {
    if bindings.tool_road.just_pressed(&keys) {
        *tool = ActiveTool::Road;
    }
    if bindings.tool_zone_res.just_pressed(&keys) {
        *tool = ActiveTool::ZoneResidentialLow;
    }
    if bindings.tool_bulldoze.just_pressed(&keys) {
        *tool = ActiveTool::Bulldoze;
    }
    if bindings.tool_inspect.just_pressed(&keys) {
        *tool = ActiveTool::Inspect;
    }
    if bindings.tool_zone_com.just_pressed(&keys) {
        *tool = ActiveTool::ZoneCommercialLow;
    }
}

// ---------------------------------------------------------------------------
// Delete key bulldozes the currently selected building
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub fn delete_selected_building(
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<simulation::keybindings::KeyBindings>,
    mut selected: ResMut<SelectedBuilding>,
    mut grid: ResMut<WorldGrid>,
    mut budget: ResMut<CityBudget>,
    mut status: ResMut<StatusMessage>,
    mut commands: Commands,
    service_q: Query<&ServiceBuilding>,
    utility_q: Query<&UtilitySource>,
    chunks: Query<(Entity, &TerrainChunk), Without<ChunkDirty>>,
) {
    if !bindings.delete_building.just_pressed(&keys)
        && !bindings.delete_building_alt.just_pressed(&keys)
    {
        return;
    }

    let Some(entity) = selected.0 else {
        return;
    };

    // Compute refund based on entity type
    let refund = if let Ok(service) = service_q.get(entity) {
        let (fw, fh) = ServiceBuilding::footprint(service.service_type);
        let sx = service.grid_x;
        let sy = service.grid_y;
        for fy in sy..sy + fh {
            for fx in sx..sx + fw {
                if grid.in_bounds(fx, fy) {
                    grid.get_mut(fx, fy).building_id = None;
                    grid.get_mut(fx, fy).zone = ZoneType::None;
                    mark_chunk_dirty_at(fx, fy, &chunks, &mut commands);
                }
            }
        }
        bulldoze_refund::refund_for_service(service.service_type)
    } else if let Ok(utility) = utility_q.get(entity) {
        let ux = utility.grid_x;
        let uy = utility.grid_y;
        if grid.in_bounds(ux, uy) {
            grid.get_mut(ux, uy).building_id = None;
            grid.get_mut(ux, uy).zone = ZoneType::None;
            mark_chunk_dirty_at(ux, uy, &chunks, &mut commands);
        }
        bulldoze_refund::refund_for_utility(utility.utility_type)
    } else {
        // Regular building: scan grid for matching entity
        for y in 0..grid.height {
            for x in 0..grid.width {
                if grid.get(x, y).building_id == Some(entity) {
                    grid.get_mut(x, y).building_id = None;
                    grid.get_mut(x, y).zone = ZoneType::None;
                    mark_chunk_dirty_at(x, y, &chunks, &mut commands);
                }
            }
        }
        0.0
    };

    budget.treasury += refund;
    commands.entity(entity).despawn();
    selected.0 = None;
    let msg = if refund > 0.0 {
        format!("Building demolished (refund: ${:.0})", refund)
    } else {
        "Building demolished".to_string()
    };
    status.set(msg, false);
}

// ---------------------------------------------------------------------------
// Tree tool system (separate from handle_tool_input to stay within param limit)
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub fn handle_tree_tool(
    buttons: Res<ButtonInput<MouseButton>>,
    cursor: Res<CursorGridPos>,
    tool: Res<ActiveTool>,
    grid: Res<WorldGrid>,
    mut budget: ResMut<CityBudget>,
    mut status: ResMut<StatusMessage>,
    mut tree_grid: ResMut<simulation::trees::TreeGrid>,
    planted_trees: Query<(Entity, &simulation::trees::PlantedTree)>,
    mut commands: Commands,
    left_drag: Res<crate::camera::LeftClickDrag>,
    chunks: Query<(Entity, &TerrainChunk), Without<ChunkDirty>>,
) {
    if left_drag.is_dragging {
        return;
    }

    let is_tree_tool = matches!(*tool, ActiveTool::TreePlant | ActiveTool::TreeRemove);
    if !is_tree_tool {
        return;
    }

    if !buttons.just_pressed(MouseButton::Left) || !cursor.valid {
        return;
    }

    let gx = cursor.grid_x as usize;
    let gy = cursor.grid_y as usize;

    let changed = match *tool {
        ActiveTool::TreePlant => {
            if tree_grid.has_tree(gx, gy) {
                status.set("Tree already here", true);
                false
            } else if grid.get(gx, gy).cell_type != simulation::grid::CellType::Grass {
                status.set("Can only plant trees on grass", true);
                false
            } else if grid.get(gx, gy).building_id.is_some() {
                status.set("Cell occupied by a building", true);
                false
            } else if budget.treasury < simulation::trees::TREE_PLANT_COST {
                status.set("Not enough money", true);
                false
            } else {
                budget.treasury -= simulation::trees::TREE_PLANT_COST;
                tree_grid.set(gx, gy, true);
                commands.spawn(simulation::trees::PlantedTree {
                    grid_x: gx,
                    grid_y: gy,
                });
                true
            }
        }
        ActiveTool::TreeRemove => {
            if !tree_grid.has_tree(gx, gy) {
                status.set("No tree here", true);
                false
            } else {
                tree_grid.set(gx, gy, false);
                for (entity, planted) in &planted_trees {
                    if planted.grid_x == gx && planted.grid_y == gy {
                        commands.entity(entity).despawn();
                        break;
                    }
                }
                true
            }
        }
        _ => false,
    };

    if changed {
        mark_chunk_dirty_at(gx, gy, &chunks, &mut commands);
    }
}

// ---------------------------------------------------------------------------
// Road upgrade tool system (separate from handle_tool_input for param limit)
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub fn handle_road_upgrade_tool(
    buttons: Res<ButtonInput<MouseButton>>,
    cursor: Res<CursorGridPos>,
    tool: Res<ActiveTool>,
    mut grid: ResMut<WorldGrid>,
    mut roads: ResMut<RoadNetwork>,
    mut segments: ResMut<RoadSegmentStore>,
    mut budget: ResMut<CityBudget>,
    mut status: ResMut<StatusMessage>,
    left_drag: Res<crate::camera::LeftClickDrag>,
    chunks: Query<(Entity, &TerrainChunk), Without<ChunkDirty>>,
    mut commands: Commands,
) {
    if left_drag.is_dragging {
        return;
    }

    if *tool != ActiveTool::RoadUpgrade {
        return;
    }

    if !buttons.just_pressed(MouseButton::Left) || !cursor.valid {
        return;
    }

    // Find the closest segment to the cursor position
    let seg_id = match simulation::road_upgrade::find_segment_near(
        cursor.world_pos,
        &segments,
        CELL_SIZE * 2.0,
    ) {
        Some(id) => id,
        None => {
            status.set("No road segment here", true);
            return;
        }
    };

    // Get current type for status message
    let current_type = match segments.get_segment(seg_id) {
        Some(seg) => seg.road_type,
        None => {
            status.set("Segment not found", true);
            return;
        }
    };

    match simulation::road_upgrade::upgrade_segment(
        seg_id,
        &mut segments,
        &mut grid,
        &mut roads,
        &mut budget,
    ) {
        Ok(new_type) => {
            // Mark all affected chunks dirty for re-rendering
            if let Some(seg) = segments.get_segment(seg_id) {
                for &(gx, gy) in &seg.rasterized_cells {
                    mark_chunk_dirty_at(gx, gy, &chunks, &mut commands);
                }
            }
            status.set(
                format!("Upgraded {:?} to {:?}", current_type, new_type),
                false,
            );
        }
        Err(reason) => {
            status.set(reason, true);
        }
    }
}

// ---------------------------------------------------------------------------
// Escape key cascade: cancel draw -> deselect building -> reset tool
// ---------------------------------------------------------------------------

/// Handles the Escape key with cascading behavior:
/// 1. Cancel active road drawing (if `RoadDrawState` is not Idle)
/// 2. Deselect the selected building (if `SelectedBuilding` has a value)
/// 3. Reset the active tool back to `Inspect`
///
/// Each press handles exactly one level.
pub fn handle_escape_key(
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<simulation::keybindings::KeyBindings>,
    mut draw_state: ResMut<RoadDrawState>,
    mut selected: ResMut<SelectedBuilding>,
    mut tool: ResMut<ActiveTool>,
    mut selection_kind: ResMut<crate::enhanced_select::SelectionKind>,
    mut freehand: ResMut<simulation::freehand_road::FreehandDrawState>,
) {
    if !bindings.escape.just_pressed(&keys) {
        return;
    }

    // Level 0: Cancel active freehand stroke
    if freehand.drawing {
        freehand.reset_stroke();
        return;
    }

    // Level 1: Cancel active road drawing
    if draw_state.phase != DrawPhase::Idle {
        draw_state.phase = DrawPhase::Idle;
        return;
    }

    // Level 2: Deselect any selection (building, citizen, road, cell)
    if selected.0.is_some() || *selection_kind != crate::enhanced_select::SelectionKind::None {
        selected.0 = None;
        *selection_kind = crate::enhanced_select::SelectionKind::None;
        return;
    }

    // Level 3: Reset to Inspect tool
    if *tool != ActiveTool::Inspect {
        *tool = ActiveTool::Inspect;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simulation::road_segments::{SegmentNode, SegmentNodeId};

    /// Helper: create a snap resource and test snapping logic directly.
    fn find_snap_target(cursor_pos: Vec2, nodes: &[SegmentNode]) -> Option<Vec2> {
        let mut best_dist = INTERSECTION_SNAP_RADIUS;
        let mut best_pos: Option<Vec2> = None;
        for node in nodes {
            let dist = (node.position - cursor_pos).length();
            if dist < best_dist {
                best_dist = dist;
                best_pos = Some(node.position);
            }
        }
        best_pos
    }

    #[test]
    fn test_intersection_snap_within_radius() {
        let node_pos = Vec2::new(100.0, 200.0);
        let nodes = vec![SegmentNode {
            id: SegmentNodeId(0),
            position: node_pos,
            connected_segments: vec![],
        }];

        // Cursor within 1 cell distance (CELL_SIZE = 16.0)
        let cursor_pos = Vec2::new(110.0, 200.0); // 10 units away < 16
        let result = find_snap_target(cursor_pos, &nodes);
        assert_eq!(result, Some(node_pos));
    }

    #[test]
    fn test_intersection_snap_outside_radius() {
        let node_pos = Vec2::new(100.0, 200.0);
        let nodes = vec![SegmentNode {
            id: SegmentNodeId(0),
            position: node_pos,
            connected_segments: vec![],
        }];

        // Cursor more than 1 cell away
        let cursor_pos = Vec2::new(120.0, 200.0); // 20 units away > 16
        let result = find_snap_target(cursor_pos, &nodes);
        assert_eq!(result, None);
    }

    #[test]
    fn test_intersection_snap_picks_closest_node() {
        let node_a = Vec2::new(100.0, 200.0);
        let node_b = Vec2::new(108.0, 200.0);
        let nodes = vec![
            SegmentNode {
                id: SegmentNodeId(0),
                position: node_a,
                connected_segments: vec![],
            },
            SegmentNode {
                id: SegmentNodeId(1),
                position: node_b,
                connected_segments: vec![],
            },
        ];

        // Cursor at 105, equidistant-ish but closer to node_b
        let cursor_pos = Vec2::new(106.0, 200.0);
        let result = find_snap_target(cursor_pos, &nodes);
        assert_eq!(result, Some(node_b)); // 2 units away vs 6 units
    }

    #[test]
    fn test_intersection_snap_no_nodes() {
        let nodes: Vec<SegmentNode> = vec![];
        let cursor_pos = Vec2::new(100.0, 200.0);
        let result = find_snap_target(cursor_pos, &nodes);
        assert_eq!(result, None);
    }

    #[test]
    fn test_intersection_snap_exact_position() {
        let node_pos = Vec2::new(100.0, 200.0);
        let nodes = vec![SegmentNode {
            id: SegmentNodeId(0),
            position: node_pos,
            connected_segments: vec![],
        }];

        // Cursor exactly at node position
        let cursor_pos = Vec2::new(100.0, 200.0);
        let result = find_snap_target(cursor_pos, &nodes);
        assert_eq!(result, Some(node_pos));
    }

    #[test]
    fn test_intersection_snap_at_boundary() {
        let node_pos = Vec2::new(100.0, 200.0);
        let nodes = vec![SegmentNode {
            id: SegmentNodeId(0),
            position: node_pos,
            connected_segments: vec![],
        }];

        // Cursor at exactly CELL_SIZE distance (should NOT snap since we use strict <)
        let cursor_pos = Vec2::new(100.0 + CELL_SIZE, 200.0);
        let result = find_snap_target(cursor_pos, &nodes);
        assert_eq!(result, None);

        // Just inside the radius
        let cursor_pos = Vec2::new(100.0 + CELL_SIZE - 0.1, 200.0);
        let result = find_snap_target(cursor_pos, &nodes);
        assert_eq!(result, Some(node_pos));
    }
}
