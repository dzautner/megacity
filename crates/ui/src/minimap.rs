//! UX-007: Minimap
//!
//! Corner minimap showing terrain, roads, buildings (zone-colored dots),
//! and camera viewport rectangle. Click on minimap to jump camera.
//!
//! Features:
//! - Minimap rendered in bottom-right corner
//! - Size: 200px (configurable 150-250)
//! - Shows terrain base colors, road lines, building dots
//! - Camera viewport shown as white rectangle
//! - Click on minimap moves camera focus with smooth transition
//! - Updated every 3 seconds (not real-time)
//! - Toggle visibility with M key

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use rendering::camera::OrbitCamera;
use simulation::config::{GRID_HEIGHT, GRID_WIDTH, WORLD_HEIGHT, WORLD_WIDTH};
use simulation::grid::{CellType, WorldGrid, ZoneType};

// =============================================================================
// Constants
// =============================================================================

/// Default minimap size in pixels.
const DEFAULT_SIZE: f32 = 200.0;

/// Margin from the screen edges.
const MARGIN: f32 = 16.0;

/// How often (in seconds) the minimap texture is regenerated.
const UPDATE_INTERVAL: f32 = 3.0;

/// Camera transition speed (units per second). Higher = faster snap.
const CAMERA_TRANSITION_SPEED: f32 = 5000.0;

// =============================================================================
// Colors
// =============================================================================

const COLOR_WATER: egui::Color32 = egui::Color32::from_rgb(60, 120, 200);
const COLOR_ROAD: egui::Color32 = egui::Color32::from_rgb(140, 140, 140);

const COLOR_RES_LOW: egui::Color32 = egui::Color32::from_rgb(80, 180, 80);
const COLOR_RES_MED: egui::Color32 = egui::Color32::from_rgb(60, 200, 60);
const COLOR_RES_HIGH: egui::Color32 = egui::Color32::from_rgb(40, 220, 40);
const COLOR_COM_LOW: egui::Color32 = egui::Color32::from_rgb(60, 100, 220);
const COLOR_COM_HIGH: egui::Color32 = egui::Color32::from_rgb(40, 80, 255);
const COLOR_INDUSTRIAL: egui::Color32 = egui::Color32::from_rgb(200, 180, 50);
const COLOR_OFFICE: egui::Color32 = egui::Color32::from_rgb(100, 80, 200);
const COLOR_MIXED_USE: egui::Color32 = egui::Color32::from_rgb(180, 100, 180);

const COLOR_VIEWPORT: egui::Color32 = egui::Color32::from_rgb(255, 255, 255);

// =============================================================================
// Plugin
// =============================================================================

pub struct MinimapPlugin;

impl Plugin for MinimapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MinimapState>()
            .init_resource::<MinimapTextureCache>()
            .init_resource::<CameraTransition>()
            .add_systems(
                Update,
                (minimap_toggle_keybind, minimap_ui, apply_camera_transition),
            );
    }
}

// =============================================================================
// Resources
// =============================================================================

/// Minimap visibility and configuration state.
#[derive(Resource)]
pub struct MinimapState {
    /// Whether the minimap is visible.
    pub visible: bool,
    /// Size of the minimap in pixels (square).
    pub size: f32,
}

impl Default for MinimapState {
    fn default() -> Self {
        Self {
            visible: true,
            size: DEFAULT_SIZE,
        }
    }
}

/// Cached minimap texture to avoid regenerating every frame.
#[derive(Resource, Default)]
struct MinimapTextureCache {
    /// The egui texture handle for the minimap.
    texture: Option<egui::TextureHandle>,
    /// Timer tracking when to next regenerate.
    timer: f32,
}

/// Smooth camera transition target when clicking the minimap.
#[derive(Resource, Default)]
struct CameraTransition {
    /// Target focus point, if a transition is in progress.
    target: Option<Vec3>,
}

// =============================================================================
// Systems
// =============================================================================

/// Toggle minimap visibility with M key.
fn minimap_toggle_keybind(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<MinimapState>,
    mut contexts: EguiContexts,
) {
    if contexts.ctx_mut().wants_keyboard_input() {
        return;
    }
    if keyboard.just_pressed(KeyCode::KeyM) {
        state.visible = !state.visible;
    }
}

/// Main minimap UI system — renders the minimap panel and handles click interaction.
#[allow(clippy::too_many_arguments)]
fn minimap_ui(
    mut contexts: EguiContexts,
    state: Res<MinimapState>,
    grid: Res<WorldGrid>,
    orbit: Res<OrbitCamera>,
    time: Res<Time>,
    mut cache: ResMut<MinimapTextureCache>,
    mut transition: ResMut<CameraTransition>,
) {
    if !state.visible {
        return;
    }

    // Update timer and regenerate texture if needed
    cache.timer += time.delta_secs();
    if cache.texture.is_none() || cache.timer >= UPDATE_INTERVAL {
        cache.timer = 0.0;
        let texture = generate_minimap_texture(contexts.ctx_mut(), &grid);
        cache.texture = Some(texture);
    }

    let Some(texture) = cache.texture.as_ref() else {
        return;
    };

    let map_size = state.size;
    let texture_id = texture.id();

    let ctx = contexts.ctx_mut();
    let screen_rect = ctx.screen_rect();

    // Position in bottom-right corner
    let panel_pos = egui::pos2(
        screen_rect.max.x - map_size - MARGIN - 4.0, // 4px for frame margin
        screen_rect.max.y - map_size - MARGIN - 4.0,
    );

    egui::Window::new("Minimap")
        .fixed_pos(panel_pos)
        .fixed_size(egui::vec2(map_size, map_size))
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .frame(
            egui::Frame::new()
                .fill(egui::Color32::from_rgba_unmultiplied(20, 22, 30, 230))
                .inner_margin(egui::Margin::same(2))
                .corner_radius(egui::CornerRadius::same(4)),
        )
        .show(ctx, |ui| {
            let (response, painter) =
                ui.allocate_painter(egui::vec2(map_size, map_size), egui::Sense::click());
            let rect = response.rect;

            // Draw the minimap texture
            painter.image(
                texture_id,
                rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );

            // Draw camera viewport rectangle
            draw_viewport_rect(&painter, rect, &orbit, map_size);

            // Handle click to move camera
            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    let rel_x = (pos.x - rect.min.x) / map_size;
                    let rel_y = (pos.y - rect.min.y) / map_size;
                    let world_x = rel_x * WORLD_WIDTH;
                    let world_z = rel_y * WORLD_HEIGHT;
                    transition.target = Some(Vec3::new(world_x, 0.0, world_z));
                }
            }
        });
}

/// Draw the camera viewport rectangle on the minimap.
fn draw_viewport_rect(
    painter: &egui::Painter,
    rect: egui::Rect,
    orbit: &OrbitCamera,
    map_size: f32,
) {
    // Estimate camera ground footprint from the orbit camera.
    // The visible area on the ground depends on distance and pitch.
    let half_visible_width = orbit.distance * orbit.pitch.cos() * 0.8;
    let half_visible_height = half_visible_width * 0.6; // approximate aspect ratio

    let focus_x = orbit.focus.x;
    let focus_z = orbit.focus.z;

    // Convert world coordinates to minimap coordinates
    let min_x = rect.min.x + ((focus_x - half_visible_width) / WORLD_WIDTH) * map_size;
    let min_y = rect.min.y + ((focus_z - half_visible_height) / WORLD_HEIGHT) * map_size;
    let max_x = rect.min.x + ((focus_x + half_visible_width) / WORLD_WIDTH) * map_size;
    let max_y = rect.min.y + ((focus_z + half_visible_height) / WORLD_HEIGHT) * map_size;

    // Clamp to minimap bounds
    let viewport_rect = egui::Rect::from_min_max(
        egui::pos2(min_x.max(rect.min.x), min_y.max(rect.min.y)),
        egui::pos2(max_x.min(rect.max.x), max_y.min(rect.max.y)),
    );

    painter.rect_stroke(
        viewport_rect,
        0.0,
        egui::Stroke::new(1.5, COLOR_VIEWPORT),
        egui::StrokeKind::Outside,
    );
}

/// Apply smooth camera transition when the player clicks on the minimap.
fn apply_camera_transition(
    time: Res<Time>,
    mut orbit: ResMut<OrbitCamera>,
    mut transition: ResMut<CameraTransition>,
) {
    let Some(target) = transition.target else {
        return;
    };

    let current = orbit.focus;
    let diff = target - current;
    let dist = diff.length();

    if dist < 5.0 {
        // Close enough — snap and finish
        orbit.focus = target;
        transition.target = None;
    } else {
        let step = CAMERA_TRANSITION_SPEED * time.delta_secs();
        let move_dist = step.min(dist);
        orbit.focus = current + diff.normalize() * move_dist;
    }
}

// =============================================================================
// Texture generation
// =============================================================================

/// Generate the minimap texture from the world grid.
/// Each grid cell maps to one pixel in the texture.
fn generate_minimap_texture(ctx: &egui::Context, grid: &WorldGrid) -> egui::TextureHandle {
    let width = GRID_WIDTH;
    let height = GRID_HEIGHT;
    let mut pixels = Vec::with_capacity(width * height);

    for y in 0..height {
        for x in 0..width {
            let cell = grid.get(x, y);
            let color = if cell.cell_type == CellType::Road {
                COLOR_ROAD
            } else if cell.cell_type == CellType::Water {
                COLOR_WATER
            } else if cell.building_id.is_some() {
                // Building present — color by zone type
                zone_color(cell.zone)
            } else if cell.zone != ZoneType::None {
                // Zoned but no building — lighter zone color
                zone_color_light(cell.zone)
            } else {
                // Terrain — vary green by elevation
                let elev = cell.elevation.clamp(0.0, 1.0);
                let g = (140.0 + elev * 40.0) as u8;
                egui::Color32::from_rgb(80, g, 60)
            };
            pixels.push(color);
        }
    }

    let color_image = egui::ColorImage {
        size: [width, height],
        pixels,
    };

    ctx.load_texture("minimap", color_image, egui::TextureOptions::LINEAR)
}

/// Get the minimap color for a zone type (occupied building).
fn zone_color(zone: ZoneType) -> egui::Color32 {
    match zone {
        ZoneType::None => egui::Color32::from_rgb(80, 140, 60),
        ZoneType::ResidentialLow => COLOR_RES_LOW,
        ZoneType::ResidentialMedium => COLOR_RES_MED,
        ZoneType::ResidentialHigh => COLOR_RES_HIGH,
        ZoneType::CommercialLow => COLOR_COM_LOW,
        ZoneType::CommercialHigh => COLOR_COM_HIGH,
        ZoneType::Industrial => COLOR_INDUSTRIAL,
        ZoneType::Office => COLOR_OFFICE,
        ZoneType::MixedUse => COLOR_MIXED_USE,
    }
}

/// Get a lighter zone color for zoned-but-unbuilt cells.
fn zone_color_light(zone: ZoneType) -> egui::Color32 {
    match zone {
        ZoneType::None => egui::Color32::from_rgb(80, 140, 60),
        ZoneType::ResidentialLow => egui::Color32::from_rgb(120, 200, 120),
        ZoneType::ResidentialMedium => egui::Color32::from_rgb(100, 220, 100),
        ZoneType::ResidentialHigh => egui::Color32::from_rgb(80, 240, 80),
        ZoneType::CommercialLow => egui::Color32::from_rgb(100, 140, 240),
        ZoneType::CommercialHigh => egui::Color32::from_rgb(80, 120, 255),
        ZoneType::Industrial => egui::Color32::from_rgb(220, 200, 90),
        ZoneType::Office => egui::Color32::from_rgb(140, 120, 220),
        ZoneType::MixedUse => egui::Color32::from_rgb(200, 140, 200),
    }
}
