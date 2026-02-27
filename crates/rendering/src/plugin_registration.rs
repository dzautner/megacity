use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;

use crate::*;
use simulation::app_state::AppState;
use simulation::SaveLoadState;

/// Register all rendering plugins and systems.
///
/// Each plugin is registered on its own line for conflict-free parallel additions.
/// When adding a new rendering plugin, just append a new `app.add_plugins(...)` line
/// at the end of the appropriate section.
///
/// Systems that query game entities (buildings, citizens, etc.) are gated behind
/// `SaveLoadState::Idle` to prevent races with the exclusive load/new-game systems
/// that despawn entities with direct world access (issue #1604).
pub(crate) fn register_rendering_systems(app: &mut App) {
    let idle = in_state(SaveLoadState::Idle);
    let playing = in_state(AppState::Playing);

    app.add_systems(
        Startup,
        (
            camera::setup_camera,
            camera_smoothing::init_camera_target,
            super::setup_lighting,
            terrain_render::spawn_terrain_chunks,
            building_preview_mesh::setup_building_preview_meshes,
            cursor_preview::spawn_cursor_preview,
            building_meshes::load_building_models,
        )
            .chain(),
    );

    // Camera controls pipeline:
    //   1. sync_target_from_external_changes — detect external OrbitCamera writes
    //   2. Camera input systems — write to CameraTarget (Playing only)
    //   3. smooth_camera_to_target — lerp OrbitCamera toward CameraTarget
    //   4. apply_orbit_camera — update the Transform from OrbitCamera
    //
    // Camera input is gated behind AppState::Playing so that the camera
    // cannot be moved from the main menu or while paused (issue #1733).
    // The smoothing + apply systems always run so the camera stays valid.
    app.add_systems(
        Update,
        (
            camera::camera_pan_keyboard,
            camera::camera_pan_drag,
            camera::camera_left_drag,
            camera::camera_orbit_drag,
            camera::camera_zoom,
            camera::camera_zoom_keyboard,
            camera::camera_rotate_keyboard,
        )
            .after(camera_smoothing::sync_target_from_external_changes)
            .run_if(playing.clone()),
    );
    app.add_systems(
        Update,
        (
            camera_smoothing::sync_target_from_external_changes,
            camera_smoothing::smooth_camera_to_target
                .after(camera::camera_pan_keyboard)
                .after(camera::camera_pan_drag)
                .after(camera::camera_left_drag)
                .after(camera::camera_orbit_drag)
                .after(camera::camera_zoom)
                .after(camera::camera_zoom_keyboard)
                .after(camera::camera_rotate_keyboard),
            camera::apply_orbit_camera.after(camera_smoothing::smooth_camera_to_target),
        ),
    );

    // Input and tool handling — gated behind SaveLoadState::Idle (entity safety)
    // and AppState::Playing (no input on main menu or pause, issue #1733).
    app.add_systems(
        Update,
        (
            input::update_cursor_grid_pos,
            angle_snap::update_angle_snap,
            input::update_intersection_snap,
            grid_align::align_cursor_to_grid
                .after(angle_snap::update_angle_snap)
                .after(parallel_snap::apply_parallel_snap_to_cursor)
                .before(input::handle_tool_input),
            grid_align::align_angle_snap_to_grid
                .after(angle_snap::update_angle_snap)
                .before(input::handle_tool_input),
            grid_align::align_intersection_snap_to_grid
                .after(input::update_intersection_snap)
                .before(input::handle_tool_input),
            input::handle_tool_input,
            input::handle_tree_tool,
            input::handle_road_upgrade_tool,
            input::keyboard_tool_switch,
            input::toggle_grid_snap,
            input::toggle_curve_draw_mode,
            input::handle_escape_key,
            input::delete_selected_building,
            input::tick_status_message,
            overlay::toggle_overlay_keys,
        )
            .run_if(idle.clone())
            .run_if(playing.clone()),
    );

    // Terrain and road rendering
    app.add_systems(
        Update,
        (
            terrain_render::dirty_chunks_on_overlay_change,
            terrain_render::rebuild_dirty_chunks,
            cursor_preview::update_cursor_preview,
            cursor_preview::draw_bezier_preview,
            angle_snap::draw_angle_snap_indicator,
            cursor_preview::draw_intersection_snap_indicator,
            road_render::sync_road_segment_meshes,
            lane_markings::sync_lane_marking_meshes,
            intersection_markings::sync_intersection_markings,
            road_grade::draw_road_grade_indicators,
        )
            .run_if(idle.clone()),
    );

    // Day/night cycle — safe, no game entity queries
    app.add_systems(Update, day_night::update_day_night_cycle);
    app.add_systems(Update, day_night::update_fog_rendering);

    // Building rendering — queries Building, ServiceBuilding, UtilitySource entities
    app.add_systems(
        Update,
        (
            building_render::spawn_building_meshes,
            building_render::update_building_meshes,
            building_render::update_construction_visuals,
            building_render::cleanup_orphan_building_meshes
                .run_if(on_timer(std::time::Duration::from_secs(1))),
            props::spawn_tree_props,
            props::spawn_road_props,
            props::spawn_parked_cars,
        )
            .run_if(idle.clone()),
    );

    // Citizen rendering — queries Citizen entities
    app.add_systems(
        Update,
        (
            citizen_render::spawn_citizen_sprites,
            citizen_render::trigger_lod_fade,
            citizen_render::update_citizen_sprites,
            citizen_render::update_lod_fade,
            citizen_render::despawn_abstract_sprites,
        )
            .chain()
            .run_if(idle.clone()),
    );

    // Status icons and trees — queries Building entities
    app.add_systems(
        Update,
        (
            building_render::spawn_planted_tree_meshes,
            building_render::cleanup_planted_tree_meshes
                .run_if(on_timer(std::time::Duration::from_secs(1))),
            status_icons::update_building_status_icons
                .run_if(on_timer(std::time::Duration::from_secs(2))),
            building_status_enhanced::update_enhanced_status_icons
                .run_if(on_timer(std::time::Duration::from_secs(2))),
            building_status_enhanced::lod_enhanced_status_icons,
        )
            .run_if(idle.clone()),
    );

    // Construction animations — queries Building/UnderConstruction entities
    app.add_systems(
        Update,
        (
            construction_anim::spawn_construction_props,
            construction_anim::update_construction_anim,
            construction_anim::animate_crane_rotation,
            construction_anim::cleanup_construction_props,
            construction_anim::cleanup_orphan_construction_props
                .run_if(on_timer(std::time::Duration::from_secs(1))),
        )
            .run_if(idle.clone()),
    );

    // Selection highlights — queries Building, ServiceBuilding, UtilitySource entities
    app.add_systems(
        Update,
        (
            selection_highlight::manage_selection_highlights,
            selection_highlight::animate_selection_highlights,
            selection_highlight::draw_connected_highlights,
        )
            .run_if(idle.clone()),
    );

    // One-way arrows
    app.add_systems(
        Update,
        oneway_arrows::draw_oneway_arrows.run_if(idle.clone()),
    );

    // Feature plugins
    app.add_plugins(traffic_los_render::TrafficLosRenderPlugin);
    app.add_plugins(traffic_arrows::TrafficArrowsPlugin);
    app.add_plugins(wind_streamlines::WindStreamlinesPlugin);
    app.add_plugins(tree_props::TreePropsPlugin);
    app.add_plugins(network_viz::NetworkVizPlugin);

    // Screenshot plugin (F12 to capture)
    app.add_plugins(screenshot::ScreenshotPlugin);

    // Building mesh variant plugin (level-aware model selection)
    app.add_plugins(building_mesh_variants::BuildingMeshVariantsPlugin);

    // Satellite view at maximum zoom-out
    app.add_plugins(satellite_view::SatelliteViewPlugin);

    // Parallel road snapping (UX-026)
    app.add_plugins(parallel_snap::ParallelSnapPlugin);

    // Parallel road drawing mode (UX-021)
    app.add_plugins(parallel_draw::ParallelDrawPlugin);

    // Box selection (UX-011)
    app.add_plugins(box_selection::BoxSelectionPlugin);

    // Zone brush preview (UX-018)
    app.add_plugins(zone_brush_preview::ZoneBrushPreviewPlugin);

    // Enhanced click-to-select with priority ordering (UX-009)
    app.add_plugins(enhanced_select::EnhancedSelectPlugin);

    // Intersection auto-detection preview (UX-023)
    app.add_plugins(intersection_preview::IntersectionPreviewPlugin);

    // Freehand road drawing (UX-020)
    app.add_plugins(freehand_draw::FreehandDrawPlugin);

    // Auto-grid road placement (TRAF-010)
    app.add_plugins(auto_grid_draw::AutoGridDrawPlugin);

    // Power grid overlay (POWER-020)
    app.add_plugins(power_overlay::PowerOverlayPlugin);

    // Zoning visual feedback (PLAY-P1-01)
    app.add_plugins(zoning_feedback::ZoningFeedbackPlugin);

    // Audio playback — consumes PlaySfxEvent (PLAY-007)
    app.add_plugins(audio_playback::AudioPlaybackPlugin);
}
