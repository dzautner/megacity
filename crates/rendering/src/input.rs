use bevy::prelude::*;

use simulation::config::CELL_SIZE;
use simulation::economy::CityBudget;
use simulation::grid::{RoadType, WorldGrid, ZoneType};
use simulation::road_segments::RoadSegmentStore;
use simulation::roads::RoadNetwork;
use simulation::services::{self, ServiceBuilding, ServiceType};
use simulation::utilities::UtilityType;

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
    ZoneResidentialHigh,
    ZoneCommercialLow,
    ZoneCommercialHigh,
    ZoneIndustrial,
    ZoneOffice,
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
            | ActiveTool::TreeRemove => None,
            ActiveTool::TreePlant => Some(simulation::trees::TREE_PLANT_COST),
            ActiveTool::ZoneResidentialLow
            | ActiveTool::ZoneResidentialHigh
            | ActiveTool::ZoneCommercialLow
            | ActiveTool::ZoneCommercialHigh
            | ActiveTool::ZoneIndustrial
            | ActiveTool::ZoneOffice => None,
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
            ActiveTool::ZoneResidentialHigh => "High-Density Residential",
            ActiveTool::ZoneCommercialLow => "Low-Density Commercial",
            ActiveTool::ZoneCommercialHigh => "High-Density Commercial",
            ActiveTool::ZoneIndustrial => "Industrial",
            ActiveTool::ZoneOffice => "Office",
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
        }
    }
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
                    cursor.world_pos = Vec2::new(hit.x, hit.z);
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

#[allow(clippy::too_many_arguments)]
pub fn handle_tool_input(
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
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
    left_drag: Res<crate::camera::LeftClickDrag>,
    mut district_map: ResMut<simulation::districts::DistrictMap>,
) {
    // Suppress tool actions when left-click is being used for camera panning
    if left_drag.is_dragging {
        return;
    }

    // Cancel freeform road drawing on Escape or tool change
    if keys.just_pressed(KeyCode::Escape) {
        draw_state.phase = DrawPhase::Idle;
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

    // Always update selected building on click, regardless of active tool.
    // This powers the Building Inspector panel.
    if buttons.just_pressed(MouseButton::Left) {
        selected.0 = grid.get(gx, gy).building_id;
    }

    // Check if Ctrl is held for legacy grid-snap mode
    let ctrl_held = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);

    // Determine if this is a freeform road tool
    let freeform_road_type = if !ctrl_held {
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
                    // First click: place start point
                    draw_state.start_pos = cursor.world_pos;
                    draw_state.phase = DrawPhase::PlacedStart;
                    status.set("Click to place end point (Esc to cancel)", false);
                }
                DrawPhase::PlacedStart => {
                    // Second click: place end point and commit segment
                    let end_pos = cursor.world_pos;
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
                // Check if it's a multi-cell service building
                if let Ok(service) = service_q.get(entity) {
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
                } else {
                    grid.get_mut(gx, gy).building_id = None;
                    grid.get_mut(gx, gy).zone = ZoneType::None;
                }
                // Despawn the entity so mesh cleanup picks it up
                commands.entity(entity).despawn();
                true
            } else if cell.zone != ZoneType::None {
                grid.get_mut(gx, gy).zone = ZoneType::None;
                true
            } else if cell.cell_type == simulation::grid::CellType::Road {
                roads.remove_road(&mut grid, gx, gy)
            } else {
                false
            }
        }
        ActiveTool::Inspect => {
            if buttons.just_pressed(MouseButton::Left) {
                let cell = grid.get(gx, gy);
                selected.0 = cell.building_id;
                if cell.building_id.is_none() {
                    status.set("No building here", false);
                }
            }
            false
        }

        // --- Zones ---
        ActiveTool::ZoneResidentialLow => apply_zone(
            &mut grid,
            &mut status,
            &buttons,
            gx,
            gy,
            ZoneType::ResidentialLow,
        ),
        ActiveTool::ZoneResidentialHigh => apply_zone(
            &mut grid,
            &mut status,
            &buttons,
            gx,
            gy,
            ZoneType::ResidentialHigh,
        ),
        ActiveTool::ZoneCommercialLow => apply_zone(
            &mut grid,
            &mut status,
            &buttons,
            gx,
            gy,
            ZoneType::CommercialLow,
        ),
        ActiveTool::ZoneCommercialHigh => apply_zone(
            &mut grid,
            &mut status,
            &buttons,
            gx,
            gy,
            ZoneType::CommercialHigh,
        ),
        ActiveTool::ZoneIndustrial => apply_zone(
            &mut grid,
            &mut status,
            &buttons,
            gx,
            gy,
            ZoneType::Industrial,
        ),
        ActiveTool::ZoneOffice => {
            apply_zone(&mut grid, &mut status, &buttons, gx, gy, ZoneType::Office)
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
        ActiveTool::TreePlant | ActiveTool::TreeRemove => false,

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
    InvalidCell,
}

fn try_zone(grid: &WorldGrid, x: usize, y: usize, zone: ZoneType) -> ZoneResult {
    let cell = grid.get(x, y);
    if cell.cell_type != simulation::grid::CellType::Grass {
        return ZoneResult::InvalidCell;
    }
    if cell.zone == zone {
        return ZoneResult::InvalidCell;
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

fn apply_zone(
    grid: &mut WorldGrid,
    status: &mut StatusMessage,
    buttons: &ButtonInput<MouseButton>,
    gx: usize,
    gy: usize,
    zone: ZoneType,
) -> bool {
    let result = try_zone(grid, gx, gy, zone);
    match result {
        ZoneResult::Success => {
            grid.get_mut(gx, gy).zone = zone;
            true
        }
        ZoneResult::NotAdjacentToRoad => {
            if buttons.just_pressed(MouseButton::Left) {
                status.set("Zone must be adjacent to road", true);
            }
            false
        }
        ZoneResult::InvalidCell => false,
    }
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

pub fn keyboard_tool_switch(keys: Res<ButtonInput<KeyCode>>, mut tool: ResMut<ActiveTool>) {
    if keys.just_pressed(KeyCode::Digit1) {
        *tool = ActiveTool::Road;
    }
    if keys.just_pressed(KeyCode::Digit2) {
        *tool = ActiveTool::ZoneResidentialLow;
    }
    if keys.just_pressed(KeyCode::Digit3) {
        *tool = ActiveTool::ZoneCommercialLow;
    }
    if keys.just_pressed(KeyCode::Digit4) {
        *tool = ActiveTool::ZoneIndustrial;
    }
    if keys.just_pressed(KeyCode::Digit5) {
        *tool = ActiveTool::Bulldoze;
    }
    if keys.just_pressed(KeyCode::Digit6) {
        *tool = ActiveTool::ZoneResidentialHigh;
    }
    if keys.just_pressed(KeyCode::Digit7) {
        *tool = ActiveTool::ZoneCommercialHigh;
    }
    if keys.just_pressed(KeyCode::Digit8) {
        *tool = ActiveTool::ZoneOffice;
    }
    if keys.just_pressed(KeyCode::Digit9) {
        *tool = ActiveTool::Inspect;
    }
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
