//! Main tool input dispatch system.

use bevy::prelude::*;
use bevy_egui::EguiContexts;

use simulation::bulldoze_refund;
use simulation::curve_road_drawing::CurveDrawMode;
use simulation::economy::CityBudget;
use simulation::grid::{RoadType, WorldGrid, ZoneType};
use simulation::road_segments::RoadSegmentStore;
use simulation::roads::RoadNetwork;
use simulation::services::ServiceBuilding;
use simulation::undo_redo::CityAction;
use simulation::unlocks::UnlockState;
use simulation::urban_growth_boundary::UrbanGrowthBoundary;
use simulation::utilities::{UtilitySource, UtilityType};

use crate::angle_snap::AngleSnapState;
use crate::egui_input_guard::egui_wants_pointer;
use crate::terrain_render::{mark_chunk_dirty_at, ChunkDirty, TerrainChunk};

use super::placement::{
    apply_zone_brush, place_road_if_affordable, place_service_if_affordable,
    place_utility_if_affordable,
};
use super::road_drawing::handle_freeform_road;
use super::terrain_tools;
use super::types::{
    ActiveTool, CursorGridPos, DrawPhase, IntersectionSnap, RoadDrawState, SelectedBuilding,
    StatusMessage,
};
use super::unlock_guard::is_tool_locked;

#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn handle_tool_input(
    input: (
        Res<ButtonInput<MouseButton>>,
        Res<ButtonInput<KeyCode>>,
        Res<AngleSnapState>,
        Res<CurveDrawMode>,
        Res<UnlockState>,
        EguiContexts,
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
        EventWriter<CityAction>,
    ),
    mut district_map: ResMut<simulation::districts::DistrictMap>,
) {
    let (buttons, keys, angle_snap, curve_mode, unlocks, mut contexts) = input;

    // Prevent click-through: skip world actions when egui is handling pointer input.
    if egui_wants_pointer(&mut contexts) {
        return;
    }

    let (left_drag, ugb, snap, brush_size, freehand, mut action_writer) = misc;

    if left_drag.is_dragging {
        return;
    }

    if buttons.just_pressed(MouseButton::Right) {
        draw_state.phase = DrawPhase::Idle;
    }

    if !buttons.pressed(MouseButton::Left) || !cursor.valid {
        return;
    }

    // --- Unlock safety check: reject locked tools ---
    if is_tool_locked(&tool, &unlocks) {
        if buttons.just_pressed(MouseButton::Left) {
            status.set("Building not yet unlocked", true);
        }
        return;
    }

    let gx = cursor.grid_x as usize;
    let gy = cursor.grid_y as usize;

    if buttons.just_pressed(MouseButton::Left) && *tool != ActiveTool::Inspect {
        selected.0 = grid.get(gx, gy).building_id;
    }

    let ctrl_held = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let freehand_active = freehand.enabled;

    let freeform_road_type = if !ctrl_held && !freehand_active {
        tool.road_type()
    } else {
        None
    };

    // Handle freeform Bezier road drawing
    if let Some(road_type) = freeform_road_type {
        handle_freeform_road(
            road_type,
            &buttons,
            &cursor,
            &snap,
            &angle_snap,
            &curve_mode,
            &mut draw_state,
            &mut segments,
            &mut grid,
            &mut roads,
            &mut budget,
            &mut status,
            &chunks,
            &mut commands,
            &mut action_writer,
        );
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
            &mut action_writer,
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
            &mut action_writer,
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
            &mut action_writer,
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
            &mut action_writer,
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
            &mut action_writer,
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
            &mut action_writer,
        ),

        ActiveTool::Bulldoze => {
            let cell = grid.get(gx, gy);
            if let Some(entity) = cell.building_id {
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
                    let stype = service.service_type;
                    let r = bulldoze_refund::refund_for_service(stype);
                    action_writer.send(CityAction::BulldozeService {
                        service_type: stype,
                        grid_x: service.grid_x,
                        grid_y: service.grid_y,
                        refund: r,
                    });
                    r
                } else if let Ok(utility) = utility_q.get(entity) {
                    grid.get_mut(gx, gy).building_id = None;
                    grid.get_mut(gx, gy).zone = ZoneType::None;
                    let utype = utility.utility_type;
                    let r = bulldoze_refund::refund_for_utility(utype);
                    action_writer.send(CityAction::BulldozeUtility {
                        utility_type: utype,
                        grid_x: utility.grid_x,
                        grid_y: utility.grid_y,
                        refund: r,
                    });
                    r
                } else {
                    grid.get_mut(gx, gy).building_id = None;
                    grid.get_mut(gx, gy).zone = ZoneType::None;
                    0.0
                };
                budget.treasury += refund;
                commands.entity(entity).despawn();
                true
            } else if cell.zone != ZoneType::None {
                let old_zone = cell.zone;
                grid.get_mut(gx, gy).zone = ZoneType::None;
                action_writer.send(CityAction::BulldozeZone {
                    x: gx,
                    y: gy,
                    zone: old_zone,
                });
                true
            } else if cell.cell_type == simulation::grid::CellType::Road {
                let road_type = cell.road_type;
                if roads.remove_road(&mut grid, gx, gy) {
                    let refund = bulldoze_refund::refund_for_road(road_type);
                    budget.treasury += refund;
                    action_writer.send(CityAction::BulldozeRoad {
                        x: gx,
                        y: gy,
                        road_type,
                        refund,
                    });
                    true
                } else {
                    false
                }
            } else {
                false
            }
        }

        ActiveTool::Inspect => false,

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
            if !zoned_cells.is_empty() {
                let cost_per_cell = crate::zone_brush_preview::ZONE_COST_PER_CELL;
                let total_cost = zoned_cells.len() as f64 * cost_per_cell;
                let cells_with_zones: Vec<(usize, usize, ZoneType)> =
                    zoned_cells.iter().map(|&(zx, zy)| (zx, zy, zone)).collect();
                action_writer.send(CityAction::PlaceZone {
                    cells: cells_with_zones,
                    cost: total_cost,
                });
            }
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
            &mut action_writer,
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
            &mut action_writer,
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
            &mut action_writer,
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
            &mut action_writer,
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
            &mut action_writer,
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
            &mut action_writer,
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
            &mut action_writer,
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
            &mut action_writer,
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
            &mut action_writer,
        ),

        // --- Terrain tools ---
        ActiveTool::TerrainRaise => {
            terrain_tools::apply_terrain_raise(gx, gy, &mut grid, &chunks, &mut commands);
            true
        }
        ActiveTool::TerrainLower => {
            terrain_tools::apply_terrain_lower(gx, gy, &mut grid, &chunks, &mut commands);
            true
        }
        ActiveTool::TerrainLevel => {
            terrain_tools::apply_terrain_level(gx, gy, &mut grid, &chunks, &mut commands);
            true
        }
        ActiveTool::TerrainWater => {
            terrain_tools::apply_terrain_water(gx, gy, &mut grid, &chunks, &mut commands);
            true
        }

        // --- Trees/RoadUpgrade/AutoGrid (handled by separate systems) ---
        ActiveTool::TreePlant
        | ActiveTool::TreeRemove
        | ActiveTool::RoadUpgrade
        | ActiveTool::AutoGrid => false,

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
                    &mut action_writer,
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
