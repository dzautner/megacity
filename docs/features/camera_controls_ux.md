# Camera Controls and User Experience Design

## Megacity -- Complete UX Specification

This document covers every aspect of how the player physically interacts with their city. The camera system, input handling, data visualization, selection mechanics, and information display collectively form the interface through which all gameplay occurs. A city builder lives or dies by these systems: players spend 100% of their time looking through the camera and clicking through tools. Every millisecond of input lag, every unintuitive control, every occluded piece of information costs engagement.

This specification is written against Megacity's current architecture: a 256x256 grid at CELL_SIZE=16.0 (4096x4096 world units), Bevy ECS with a perspective 3D camera using an orbital model, chunk-based terrain rendering (CHUNK_SIZE=8, 32x32 chunks), a three-tier LOD system for citizens (Full/Simplified/Abstract), and an egui-based UI layer.

---

## Table of Contents

1. [Camera System Architecture](#1-camera-system-architecture)
2. [Zoom Levels and Transitions](#2-zoom-levels-and-transitions)
3. [Camera Rotation and Orbiting](#3-camera-rotation-and-orbiting)
4. [Camera Panning](#4-camera-panning)
5. [Camera Bounds and Constraints](#5-camera-bounds-and-constraints)
6. [Camera Follow and Tracking](#6-camera-follow-and-tracking)
7. [Cinematic Mode and Photo Mode](#7-cinematic-mode-and-photo-mode)
8. [Zoom-Level-of-Detail Integration](#8-zoom-level-of-detail-integration)
9. [Data Overlay System](#9-data-overlay-system)
10. [Selection and Interaction](#10-selection-and-interaction)
11. [Tool System UX](#11-tool-system-ux)
12. [Road Building UX](#12-road-building-ux)
13. [Information Display](#13-information-display)
14. [Keyboard Shortcuts and Hotkeys](#14-keyboard-shortcuts-and-hotkeys)
15. [Controller Support](#15-controller-support)
16. [Accessibility](#16-accessibility)
17. [Performance UX](#17-performance-ux)
18. [Reference Games UX Analysis](#18-reference-games-ux-analysis)

---

## 1. Camera System Architecture

### 1.1 Perspective vs. Orthographic Projection

City builders have historically used three camera approaches, each with distinct tradeoffs:

**Orthographic (top-down):** SimCity (1989), SimCity 2000 (isometric variant), Banished, and most mobile city builders use orthographic projection. The advantage is spatial clarity: every cell is the same visual size regardless of distance, making it trivially easy to count tiles, align grids, and judge distances. The disadvantage is that the city feels flat and lifeless. There is no depth parallax, no sense of scale when zooming in, no dramatic vista when you look across your skyline. Orthographic cameras also make 3D building models look uncanny because the lack of perspective foreshortening makes all vertical lines parallel, which the human eye reads as "wrong" for tall buildings.

**Perspective with steep angle (55-70 degrees pitch):** Cities: Skylines 1 uses approximately 60 degrees of pitch by default. This is the sweet spot for city builders: steep enough that you can see the grid layout clearly, shallow enough that buildings have visual depth and the skyline has drama. The perspective foreshortening gives a natural sense of scale -- distant buildings are smaller, nearby buildings loom. This projection makes screenshots and videos look beautiful, which is commercially important (organic marketing through player-shared screenshots accounts for significant discovery in the genre).

**Perspective with variable pitch (5-85 degrees):** Cities: Skylines 2 and Anno 1800 allow near-street-level camera angles. This enables the "walk your city" experience that players find deeply satisfying -- seeing their creation from a human perspective validates hundreds of hours of planning. However, low-angle views require dramatically more geometric detail to look good, since buildings that were acceptable as simplified shapes from above now need facades, windows, doors, and signage to be convincing.

**Recommendation for Megacity:** Use perspective projection with variable pitch, currently implemented as the `OrbitCamera` system with pitch clamped between 5 degrees (`MIN_PITCH`) and 80 degrees (`MAX_PITCH`). This is the correct architecture. The current default pitch of 45 degrees is slightly low for a city builder -- consider defaulting to 55-60 degrees, which better shows the grid layout while still providing depth. The 5-degree minimum allows near-street-level viewing which is a high-engagement feature. The 80-degree maximum prevents the awkward gimbal lock zone near vertical.

The current implementation in `camera.rs`:

```rust
const MIN_PITCH: f32 = 5.0 * std::f32::consts::PI / 180.0;  // 5 degrees
const MAX_PITCH: f32 = 80.0 * std::f32::consts::PI / 180.0;  // 80 degrees
```

This is sound. Two potential refinements:

1. **Dynamic pitch limits based on zoom:** At very far zoom (distance > 3000), clamp minimum pitch to 30 degrees. There is no visual value in a near-horizontal view of an entire 4km city -- you would just see a line of buildings on the horizon. At close zoom (distance < 100), allow pitch down to 2-3 degrees for near-ground-level viewing.

2. **Pitch-dependent FOV:** Consider narrowing the field of view slightly at low pitch angles (simulate a telephoto lens effect) and widening it at high pitch (simulate a wider planning view). This is subtle but makes the street-level view feel more cinematic and the planning view feel more spacious. CS2 uses a similar technique.

### 1.2 The Orbital Camera Model

The current `OrbitCamera` struct stores focus point, yaw, pitch, and distance. The camera position is computed via spherical-to-cartesian conversion in `orbit_to_transform()`:

```rust
let x = orbit.distance * orbit.pitch.cos() * orbit.yaw.sin();
let y = orbit.distance * orbit.pitch.sin();
let z = orbit.distance * orbit.pitch.cos() * orbit.yaw.cos();
let pos = orbit.focus + Vec3::new(x, y, z);
```

This is the standard orbital camera model used by virtually every 3D city builder. The focus point sits on the ground plane (Y=0), and the camera orbits around it at the specified yaw, pitch, and distance. The `looking_at` transform ensures the camera always faces the focus point.

**Critical design principle:** The focus point is the player's mental model of "where I am in my city." Every interaction -- panning, zooming, rotating -- should feel like manipulating the focus point, not the camera. When the player pans, the focus point moves along the ground. When they zoom, the camera moves toward or away from the focus point. When they rotate, the camera orbits the focus point. This mental model is intuitive because it matches how humans naturally think about looking at a tabletop model.

**Smooth interpolation:** The current implementation applies camera state changes immediately (no interpolation). This is acceptable for mouse-driven interactions where the input device provides natural smoothing, but keyboard panning feels jerky without easing. Consider adding exponential smoothing:

```rust
// Instead of directly setting orbit values, use a target + lerp approach:
struct OrbitCamera {
    focus: Vec3,
    yaw: f32,
    pitch: f32,
    distance: f32,
    // Smoothing targets
    target_focus: Vec3,
    target_yaw: f32,
    target_pitch: f32,
    target_distance: f32,
}

// In apply_orbit_camera:
let smoothing = 1.0 - (-15.0 * dt).exp();  // ~93% per frame at 60fps
orbit.focus = orbit.focus.lerp(orbit.target_focus, smoothing);
orbit.distance = orbit.distance + (orbit.target_distance - orbit.distance) * smoothing;
```

The exponential approach (`1 - e^(-k*dt)`) is frame-rate independent, which is essential. Linear lerp with a fixed fraction breaks at different frame rates.

### 1.3 Frame-Rate Independent Camera Movement

All camera movement must be frame-rate independent. The current WASD panning multiplies by `time.delta_secs()`, which is correct. Mouse drag panning operates on pixel deltas, which are inherently frame-rate independent (each mouse move event produces the same delta regardless of frame rate). Zoom uses multiplicative scaling per scroll event, which is also frame-rate independent.

However, if smoothing is added (as recommended above), the smoothing factor MUST use an exponential decay rather than a linear interpolation fraction, since linear lerp with a fixed factor produces different results at different frame rates:

- At 60 FPS with lerp(0.15): reaches 90% in ~15 frames = 250ms
- At 144 FPS with lerp(0.15): reaches 90% in ~15 frames = 104ms (too fast!)
- At 30 FPS with lerp(0.15): reaches 90% in ~15 frames = 500ms (too slow!)

The fix is `factor = 1.0 - e^(-speed * delta_time)`, which produces consistent behavior regardless of frame rate.

---

## 2. Zoom Levels and Transitions

### 2.1 Zoom Range and Scale

The current zoom range spans from `MIN_DISTANCE = 20.0` to `MAX_DISTANCE = 4000.0`, a ratio of 200:1. This is an excellent range for a 4096x4096 world:

**At MAX_DISTANCE (4000.0 units):** With a 45-degree pitch, the camera is at Y=2828 units above the ground, looking out approximately 2828 units horizontally. The viewport shows roughly 3000-4000 units across, meaning the entire 4096-unit map is nearly visible. Buildings at this zoom are approximately 1-3 pixels tall. This is the "full map overview" zoom level.

**At MIN_DISTANCE (20.0 units):** With a 45-degree pitch, the camera is at Y=14.1 units above the ground. Given a CELL_SIZE of 16.0, this means the camera is less than one cell-height above the ground. Individual building details, citizen meshes, and road markings fill the screen. This is "street level."

The zoom ratio should be defined in terms of meaningful gameplay tiers, not just min/max values. Here is the complete zoom level specification:

| Zoom Level | Distance Range | Camera Height (45 deg) | Visible Area | What's Visible | LOD Tier |
|---|---|---|---|---|---|
| Satellite | 3000-4000 | 2121-2828m | Full map | Terrain colors, major roads as lines, zone color blocks | Abstract |
| Regional | 1500-3000 | 1060-2121m | Half map | Neighborhood shapes, road network, building clusters | Abstract |
| City | 600-1500 | 424-1060m | District | Individual blocks, building outlines, park shapes | Abstract/Simplified |
| Neighborhood | 200-600 | 141-424m | Several blocks | Building shapes with zone colors, tree clusters, road types | Simplified |
| Block | 80-200 | 56-141m | One block | Individual buildings with detail, street trees, citizen dots | Full |
| Street | 30-80 | 21-56m | A few buildings | Building models with textures, vehicle meshes, road markings | Full |
| Pedestrian | 20-30 | 14-21m | Single building | High-detail everything, citizen animations, shop signs | Full |

### 2.2 Exponential Zoom

The current zoom implementation uses multiplicative scaling:

```rust
let factor = 1.0 - dy * ZOOM_SPEED;
orbit.distance = (orbit.distance * factor).clamp(MIN_DISTANCE, MAX_DISTANCE);
```

This is **exactly correct** and is the single most important camera behavior to get right. Multiplicative (exponential) zoom means that each scroll tick changes the distance by a percentage (currently 15% per scroll line), not a fixed amount. This produces several critical properties:

1. **Perceived zoom speed is constant:** At distance 4000, one scroll tick moves 600 units. At distance 100, one scroll tick moves 15 units. The visual result -- the amount the viewport changes -- is identical in both cases. Linear zoom would feel impossibly slow when zoomed out and impossibly fast when zoomed in.

2. **Zoom-to-cursor works naturally:** When implementing zoom-to-cursor (where the point under the mouse cursor stays fixed during zoom), the exponential zoom means the focus point adjustment is also proportional, preserving the visual anchor.

3. **Reversibility:** N scroll ticks up followed by N scroll ticks down returns to exactly the same zoom level. This would not be true with additive zoom.

**Current zoom speed (ZOOM_SPEED = 0.15):** This means each scroll tick zooms by 15%. Going from MAX_DISTANCE (4000) to MIN_DISTANCE (20) requires approximately `ln(4000/20) / ln(1.15) = 38` scroll ticks. This is reasonable -- it takes about 3-4 seconds of steady scrolling to traverse the full zoom range. CS1 is similar, CS2 is slightly slower (about 5 seconds).

**Zoom-to-cursor (not yet implemented):** This is a high-impact feature that should be added. When the player scrolls to zoom in, the point under the mouse cursor should stay in the same screen position. Without this, zooming feels "disconnected" -- the player has to zoom in and then pan to find what they wanted to look at. Implementation:

```rust
// Before zoom: cast ray from cursor to ground plane
let cursor_ground_before = ray_ground_intersection(cursor_screen_pos);

// Apply zoom
orbit.distance *= factor;

// After zoom: the same cursor position now maps to a different ground point
let cursor_ground_after = ray_ground_intersection(cursor_screen_pos);

// Shift focus to compensate
orbit.focus += cursor_ground_before - cursor_ground_after;
```

This is sometimes called "dolly zoom" or "zoom to point." It is implemented in CS1, CS2, Google Maps, and essentially every modern mapping/viewing application. It should be considered mandatory.

### 2.3 Smooth Zoom Transitions

When using keyboard shortcuts to jump between zoom presets (e.g., pressing a key to go to "overview" zoom), the transition should be animated, not instant. An instant jump is disorienting because the player loses spatial context -- they cannot mentally track where they "are" in the city.

Use exponential interpolation (the same as the smoothing from section 1.2) with a faster rate:

```rust
// For preset zoom jumps, use a 200ms transition
let zoom_smoothing = 1.0 - (-10.0 * dt).exp();
orbit.distance += (target_distance - orbit.distance) * zoom_smoothing;
```

The 200ms duration is fast enough to feel responsive but slow enough for the brain to track the spatial transition. CS2 uses approximately 300ms, which feels slightly sluggish.

### 2.4 Zoom Level Events

The zoom level should emit events when crossing tier boundaries, enabling other systems to respond:

```rust
enum ZoomTier {
    Satellite,    // > 3000
    Regional,     // 1500-3000
    City,         // 600-1500
    Neighborhood, // 200-600
    Block,        // 80-200
    Street,       // 30-80
    Pedestrian,   // < 30
}

// Emit event on tier change
fn detect_zoom_tier_change(
    orbit: Res<OrbitCamera>,
    mut last_tier: Local<ZoomTier>,
    mut events: EventWriter<ZoomTierChanged>,
) {
    let current = ZoomTier::from_distance(orbit.distance);
    if current != *last_tier {
        events.send(ZoomTierChanged {
            old: *last_tier,
            new: current
        });
        *last_tier = current;
    }
}
```

These events drive LOD transitions, UI changes (show/hide minimap), sound design (ambient sound mix changes with altitude), and overlay density. Without explicit tier events, every system that cares about zoom level has to independently check the camera distance, leading to inconsistent behavior and threshold bugs.

---

## 3. Camera Rotation and Orbiting

### 3.1 Free Rotation vs. 90-Degree Snap

This is one of the most debated design decisions in city builder camera design. The two approaches have fundamentally different design philosophies:

**90-degree snap rotation (SimCity 4, SimCity 2013):** The camera can only face north, south, east, or west. Pressing Q or E rotates by 90 degrees. Advantages:
- Grid-aligned roads and buildings always look perfectly aligned on screen
- The minimap orientation matches the viewport (or differs by a known 90-degree rotation)
- Simpler mental model for players to maintain ("I'm looking north right now")
- UI elements can use cardinal directions unambiguously
- Easier to implement consistent building facades (buildings only need to look good from 4 angles)
- Screenshot consistency: every player sees the same city from the same angles

**Free rotation (CS1, CS2, Anno 1800):** The camera can rotate to any angle. Advantages:
- Feels more natural and fluid, especially with middle-mouse drag
- Allows finding the "perfect angle" for screenshots
- Better for exploring curving road layouts and non-grid urban design
- More immersive when combined with low-angle viewing
- Feels "modern" -- players expect it in 2024+

**Recommendation for Megacity:** Free rotation (already implemented via right-mouse drag). The Bezier road system already encourages organic, non-grid road layouts. Forcing 90-degree snap on a game with freeform roads would feel contradictory. However, provide a snap-to-90 feature:

- **Q/E keys:** Rotate 90 degrees with smooth animated transition (300ms)
- **Right-click drag:** Free rotation at `ORBIT_SENSITIVITY = 0.005` (current implementation)
- **Shift+Q/Shift+E:** Rotate 45 degrees for diagonal views
- **Home key or double-tap Q/E:** Reset rotation to north-facing (yaw = 0)

The 90-degree snap shortcuts give players the benefits of grid alignment when they want it, without removing the freedom of continuous rotation.

### 3.2 Rotation Smoothing

The current implementation applies rotation immediately on mouse delta:

```rust
orbit.yaw += delta.x * ORBIT_SENSITIVITY;
orbit.pitch = (orbit.pitch - delta.y * ORBIT_SENSITIVITY).clamp(MIN_PITCH, MAX_PITCH);
```

This is correct for mouse-driven rotation -- adding smoothing to mouse orbit would feel "laggy" and imprecise, because the player expects the viewport to track their hand movement exactly. However, for keyboard-triggered rotation (Q/E snap), smoothing is essential to avoid spatial disorientation.

Implement rotation smoothing only for programmatic rotations:

```rust
struct OrbitCamera {
    // ... existing fields ...
    target_yaw: Option<f32>,  // When Some, smoothly interpolate yaw toward target
}

fn apply_yaw_smoothing(orbit: &mut OrbitCamera, dt: f32) {
    if let Some(target) = orbit.target_yaw {
        let diff = angle_diff(orbit.yaw, target);
        if diff.abs() < 0.01 {
            orbit.yaw = target;
            orbit.target_yaw = None;
        } else {
            let speed = 8.0; // radians per second (full 90-degree turn in ~200ms)
            let step = diff.signum() * speed * dt;
            if step.abs() > diff.abs() {
                orbit.yaw = target;
                orbit.target_yaw = None;
            } else {
                orbit.yaw += step;
            }
        }
    }
}
```

### 3.3 Rotation Axis Behavior

The current pitch implementation inverts the mouse Y delta:

```rust
orbit.pitch = (orbit.pitch - delta.y * ORBIT_SENSITIVITY).clamp(MIN_PITCH, MAX_PITCH);
```

The negative sign means dragging the mouse down tilts the view up (increases pitch), which matches the "grab the world and rotate it" metaphor. This is the correct convention for a "virtual trackball" style orbit. Some games invert this (dragging down tilts down), but that matches a "move the camera" metaphor which is less intuitive for city builders where the player mentally "holds" the map.

**Yaw wrapping:** The yaw value is unbounded -- it can go to arbitrarily large positive or negative values. While this does not cause rendering issues (sin/cos handle any input), it can cause precision issues over long play sessions. Add periodic normalization:

```rust
orbit.yaw = orbit.yaw.rem_euclid(std::f32::consts::TAU);
```

---

## 4. Camera Panning

### 4.1 Pan Input Methods

The current implementation supports three pan methods, which is good coverage:

1. **WASD/Arrow keys** (`camera_pan_keyboard`): Movement direction is rotated by current yaw, so "W" always moves "forward" relative to the camera view. Speed scales with distance (`orbit.distance / 1000.0`), providing zoom-proportional panning.

2. **Middle-mouse drag** (`camera_pan_drag`): Direct 1:1 mapping from screen pixel movement to world movement. Also scales with distance.

3. **Left-click drag** (`camera_left_drag`): Same as middle-mouse drag but with a 5-pixel threshold to distinguish clicks from drags. This is critical because left-click is the primary tool interaction button.

A fourth method should be added:

4. **Edge scrolling:** When the mouse cursor is within N pixels of the screen edge, pan in that direction. This is expected behavior in all PC strategy games. Implementation:

```rust
fn camera_edge_scroll(
    windows: Query<&Window>,
    time: Res<Time>,
    mut orbit: ResMut<OrbitCamera>,
    settings: Res<CameraSettings>,
) {
    if !settings.edge_scroll_enabled { return; }

    let Ok(window) = windows.get_single() else { return };
    let Some(cursor) = window.cursor_position() else { return };

    let edge_zone = settings.edge_scroll_margin; // 20-40 pixels
    let width = window.width();
    let height = window.height();

    let mut dir = Vec2::ZERO;
    if cursor.x < edge_zone { dir.x -= 1.0; }
    if cursor.x > width - edge_zone { dir.x += 1.0; }
    if cursor.y < edge_zone { dir.y -= 1.0; }
    if cursor.y > height - edge_zone { dir.y += 1.0; }

    // Apply with yaw rotation and distance scaling
    // (same logic as camera_pan_keyboard)
}
```

Edge scrolling should be **disabled by default** if the game runs in windowed mode (too easy to accidentally trigger while moving the mouse to the taskbar) and **enabled by default** in fullscreen. Provide a toggle in settings.

### 4.2 Pan Speed Scaling

The current speed scaling uses a linear relationship with camera distance:

```rust
let scale = orbit.distance / 1000.0;
```

At the default distance of 2000, this gives a scale factor of 2.0. At minimum distance (20), scale is 0.02. At maximum (4000), scale is 4.0. The base `PAN_SPEED` is 500.0 units/second.

This means:
- Street level (distance 20): 10 units/second = 0.625 cells/second (very slow, appropriate)
- Default view (distance 2000): 1000 units/second = 62.5 cells/second (good)
- Full zoom out (distance 4000): 2000 units/second = 125 cells/second (fast, appropriate for crossing the map)

The linear relationship is adequate but not optimal. At very high zoom, the pan speed should increase faster than linearly, since the player is trying to navigate across the entire map. At very close zoom, the pan speed should be even slower to allow precise positioning. Consider a quadratic component at the extremes:

```rust
let base_scale = orbit.distance / 1000.0;
let scale = if orbit.distance > 2000.0 {
    base_scale * (1.0 + (orbit.distance - 2000.0) / 4000.0)
} else {
    base_scale
};
```

### 4.3 Pan Inertia

Many modern city builders add slight inertia to panning -- when the player releases the middle mouse button after a fast drag, the camera continues to glide in that direction and decelerates. This makes navigation feel "smooth" and "polished" at the cost of slight imprecision.

**Recommendation:** Add optional pan inertia, disabled by default. Players who want precise control (most city builder players) find inertia annoying. Players who want a cinematic experience appreciate it.

```rust
struct PanInertia {
    velocity: Vec2,  // world units per second
    friction: f32,   // deceleration rate (units/sec^2)
    enabled: bool,
}

fn apply_pan_inertia(
    mut inertia: ResMut<PanInertia>,
    mut orbit: ResMut<OrbitCamera>,
    drag: Res<CameraDrag>,
    time: Res<Time>,
) {
    if !inertia.enabled { return; }

    if drag.dragging {
        // Record velocity from drag deltas
        // (computed in camera_pan_drag system)
        return;
    }

    if inertia.velocity.length_squared() > 0.1 {
        let dt = time.delta_secs();
        orbit.focus.x += inertia.velocity.x * dt;
        orbit.focus.z += inertia.velocity.y * dt;

        // Exponential decay
        let decay = (-5.0 * dt).exp();
        inertia.velocity *= decay;
    }
}
```

### 4.4 Pan Direction Relative to Camera

The current implementation correctly rotates the pan direction by the camera's yaw:

```rust
let cos_yaw = orbit.yaw.cos();
let sin_yaw = orbit.yaw.sin();
let world_x = dir.x * cos_yaw + dir.y * sin_yaw;
let world_z = -dir.x * sin_yaw + dir.y * cos_yaw;
```

This means "W" always moves the view "forward" (away from the camera), regardless of camera rotation. This is the expected behavior and is non-negotiable. Games that pan in world-aligned directions regardless of camera rotation (pressing W always moves north) are deeply frustrating to use when the camera is rotated.

However, for mouse drag panning, the direction should match the "grab the world" metaphor: dragging left moves the world left (camera pans right). The current implementation uses negative deltas:

```rust
let world_x = -delta.x * cos_yaw - delta.y * sin_yaw;
let world_z = delta.x * sin_yaw - delta.y * cos_yaw;
```

This is correct -- dragging the mouse in a direction moves the view as if you are pushing the map in that direction, so the visible area slides in the opposite direction from the mouse movement. This is the Google Maps convention and is what players expect.

---

## 5. Camera Bounds and Constraints

### 5.1 Map Edge Behavior

The current implementation clamps the focus point with a 500-unit margin:

```rust
fn clamp_focus(focus: &mut Vec3) {
    let margin = 500.0;
    focus.x = focus.x.clamp(-margin, WORLD_WIDTH + margin);
    focus.z = focus.z.clamp(-margin, WORLD_HEIGHT + margin);
}
```

With `WORLD_WIDTH = WORLD_HEIGHT = 4096.0`, this allows the focus point to range from -500 to 4596 in both axes. The 500-unit margin is approximately 31 cells worth of overflow, which is appropriate -- it allows the player to position their city near the edge while still centering the camera view on it.

**Dynamic margin based on zoom:** The margin should increase with zoom distance. When fully zoomed out (distance 4000), the viewport shows approximately 3000 units across. If the player wants to see the northeast corner of the map, they need the focus point at approximately (4096 - 1500, 0, 4096 - 1500), which is 2596 units from the edge -- well within the current bounds. But if they want to center the corner on screen, they need (4096, 0, 4096), which requires margin >= 0. The current 500-unit margin handles this.

However, when zoomed in close (distance 50), a 500-unit margin means the player can pan 31 cells off the edge of the map, looking at empty void. The margin should scale with zoom:

```rust
fn clamp_focus(focus: &mut Vec3, distance: f32) {
    let base_margin = 100.0;
    let zoom_margin = distance * 0.3;
    let margin = base_margin + zoom_margin;
    focus.x = focus.x.clamp(-margin, WORLD_WIDTH + margin);
    focus.z = focus.z.clamp(-margin, WORLD_HEIGHT + margin);
}
```

### 5.2 Vertical Constraints

The focus point is currently constrained to Y=0 (ground plane). This is correct for a flat-world city builder. If terrain elevation is added (which the terrain raise/lower tools suggest is planned), the focus point Y should track the terrain height at the focus position:

```rust
// After clamping X/Z:
let grid_x = (focus.x / CELL_SIZE) as usize;
let grid_z = (focus.z / CELL_SIZE) as usize;
if grid.in_bounds(grid_x, grid_z) {
    focus.y = grid.get(grid_x, grid_z).elevation * ELEVATION_SCALE;
}
```

This keeps the camera focused on the terrain surface rather than clipping through hills when panning.

### 5.3 Underground View

City builders often have underground infrastructure (water pipes, subway lines, utility tunnels). CS1 handles this with a discrete toggle that replaces the surface view with a transparent grid showing underground networks. CS2 integrates it more smoothly with transparency.

For Megacity, the recommended approach is:

1. **Toggle key (U):** Press to enter underground mode
2. **In underground mode:** Surface terrain becomes 80% transparent, buildings become 50% transparent outlines, underground networks (water pipes, power lines, subway) are rendered at full opacity
3. **Camera pitch lock:** In underground mode, clamp pitch to 30-80 degrees (prevent near-horizontal viewing of transparencies, which looks terrible)
4. **Color coding:** Water pipes = blue, power cables = yellow, sewage = brown, subway = orange

This is a future feature but should be architecturally planned now by reserving render layers in Bevy:

```rust
const LAYER_TERRAIN: u8 = 0;
const LAYER_ROADS: u8 = 1;
const LAYER_BUILDINGS: u8 = 2;
const LAYER_CITIZENS: u8 = 3;
const LAYER_PROPS: u8 = 4;
const LAYER_OVERLAYS: u8 = 5;
const LAYER_UNDERGROUND: u8 = 6;
const LAYER_UI_WORLD: u8 = 7;  // 3D UI elements like status icons
```

---

## 6. Camera Follow and Tracking

### 6.1 Entity Follow Mode

Clicking on a citizen, vehicle, or transit vehicle and pressing a "follow" button (or double-clicking) should enter a camera follow mode where the camera tracks the entity as it moves through the city. This is one of the most-requested features in city builders and is a significant engagement driver -- players spend hours watching individual citizens commute.

**Implementation architecture:**

```rust
#[derive(Resource, Default)]
pub struct CameraFollow {
    pub target: Option<Entity>,
    pub mode: FollowMode,
    pub offset: Vec3,
}

enum FollowMode {
    /// Camera orbits around entity, maintaining user's zoom/pitch/yaw
    Orbit,
    /// Camera follows behind entity (third-person)
    ThirdPerson,
    /// First-person through entity's eyes
    FirstPerson,
}
```

**Orbit follow:** The focus point is set to the entity's position each frame. The player retains control over yaw, pitch, and distance. This is the simplest and most commonly used mode.

```rust
fn camera_follow_orbit(
    follow: Res<CameraFollow>,
    mut orbit: ResMut<OrbitCamera>,
    positions: Query<&Position>,
) {
    if let Some(target) = follow.target {
        if let Ok(pos) = positions.get(target) {
            // Smooth tracking with lead prediction
            let target_focus = Vec3::new(pos.x, 0.0, pos.y);
            let smoothing = 0.1;
            orbit.focus = orbit.focus.lerp(target_focus, smoothing);
        }
    }
}
```

**Lead prediction:** When following a moving entity, the focus point should lead slightly ahead of the entity in its direction of travel. This prevents the entity from constantly sitting at the center of the screen (boring) and gives the player a view of where the entity is going:

```rust
let velocity = Vec3::new(vel.x, 0.0, vel.y);
let lead_time = 2.0; // seconds of prediction
let predicted = Vec3::new(pos.x, 0.0, pos.y) + velocity * lead_time;
let focus = current_pos * 0.4 + predicted * 0.6; // blend 60% ahead
```

**Third-person follow:** Camera positioned behind and above the entity, looking forward in the direction of travel. This requires computing a "behind" position based on the entity's heading:

```rust
let heading = vel.y.atan2(vel.x);
let behind_offset = Vec3::new(-heading.cos() * 30.0, 15.0, -heading.sin() * 30.0);
orbit.focus = entity_pos;
// Override orbit position with behind_offset
```

**First-person mode:** Camera at entity height (Y = 1.7 units for a human, Y = 1.5 for a car), looking in the direction of travel. This is the CS2 "first-person" feature that proved extremely popular. Implementation:

```rust
fn first_person_follow(
    follow: Res<CameraFollow>,
    positions: Query<(&Position, &Velocity)>,
    mut camera_transform: Query<&mut Transform, With<Camera3d>>,
) {
    if follow.mode != FollowMode::FirstPerson { return; }
    if let Some(target) = follow.target {
        if let Ok((pos, vel)) = positions.get(target) {
            let eye = Vec3::new(pos.x, 1.7, pos.y); // human eye height
            let forward = Vec3::new(vel.x, 0.0, vel.y).normalize_or_zero();
            let look_at = eye + forward * 10.0;
            let mut transform = camera_transform.single_mut();
            *transform = Transform::from_translation(eye).looking_at(look_at, Vec3::Y);
        }
    }
}
```

**Exit conditions:** The follow mode should exit when:
- The player clicks anywhere on the map (to interact)
- The player presses Escape
- The followed entity is despawned (citizen dies, vehicle removed)
- The player uses WASD keys (intent to manually navigate)

### 6.2 Location Bookmarks

Players should be able to bookmark up to 4-8 camera positions and instantly jump to them. CS1 uses Ctrl+F1 through Ctrl+F4 to set bookmarks and F1-F4 to recall. This is a power-user feature that significantly improves workflow for experienced players:

```rust
#[derive(Resource)]
pub struct CameraBookmarks {
    pub slots: [Option<BookmarkState>; 8],
}

struct BookmarkState {
    focus: Vec3,
    yaw: f32,
    pitch: f32,
    distance: f32,
}

fn bookmark_system(
    keys: Res<ButtonInput<KeyCode>>,
    orbit: Res<OrbitCamera>,
    mut bookmarks: ResMut<CameraBookmarks>,
) {
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);

    for (i, key) in [KeyCode::F1, KeyCode::F2, KeyCode::F3, KeyCode::F4,
                     KeyCode::F5, KeyCode::F6, KeyCode::F7, KeyCode::F8].iter().enumerate() {
        if keys.just_pressed(*key) {
            if ctrl {
                // Save bookmark
                bookmarks.slots[i] = Some(BookmarkState {
                    focus: orbit.focus,
                    yaw: orbit.yaw,
                    pitch: orbit.pitch,
                    distance: orbit.distance,
                });
            } else if let Some(bm) = &bookmarks.slots[i] {
                // Restore bookmark (with smooth transition)
                orbit.target_focus = Some(bm.focus);
                orbit.target_yaw = Some(bm.yaw);
                orbit.target_pitch = Some(bm.pitch);
                orbit.target_distance = Some(bm.distance);
            }
        }
    }
}
```

### 6.3 Jump-to-Location

Notifications, event alerts, and advisor messages should include a "jump to" button that smoothly transitions the camera to the relevant location. The transition should use the same exponential smoothing as other camera movements, with a duration of approximately 500ms for cross-city jumps and 200ms for nearby jumps.

```rust
fn jump_to_location(
    orbit: &mut OrbitCamera,
    target_pos: Vec2,
    target_zoom: f32,
) {
    orbit.target_focus = Some(Vec3::new(target_pos.x, 0.0, target_pos.y));
    orbit.target_distance = Some(target_zoom);
    // Keep current yaw and pitch
}
```

---

## 7. Cinematic Mode and Photo Mode

### 7.1 Photo Mode

Photo mode freezes the simulation and gives the player enhanced camera controls for taking screenshots. This is a significant marketing feature -- beautiful screenshots drive organic discovery on social media and forums.

**Photo mode controls:**
- **Depth of field:** Adjustable focus distance and aperture (blur background or foreground)
- **Time of day override:** Set the sun position independently of simulation time (golden hour shots)
- **Weather override:** Clear, cloudy, rainy, foggy, snowy
- **FOV slider:** 20-120 degrees (narrow for telephoto compression, wide for dramatic perspectives)
- **Grid/UI toggle:** Hide all UI elements, grid lines, and overlays
- **Tilt-shift:** Fake miniature effect (strong depth of field with blur at top and bottom of frame)
- **Resolution multiplier:** Render at 2x or 4x for high-quality screenshots
- **Camera position lock:** Fine-tune position with arrow keys at very slow speed

**Implementation priority:** Photo mode is a "nice to have" feature that can be implemented late in development. The core camera controls should be solid first.

### 7.2 Cinematic Camera Paths

Allow players to define camera paths for time-lapse videos. The player places keyframe points, the system interpolates between them using Catmull-Rom splines, and the camera plays back along the path at a configurable speed while the simulation runs at configurable speed.

```rust
struct CameraPath {
    keyframes: Vec<CameraKeyframe>,
    duration: f32,
    looping: bool,
}

struct CameraKeyframe {
    focus: Vec3,
    yaw: f32,
    pitch: f32,
    distance: f32,
    time: f32,  // normalized [0, 1]
}
```

This is a low-priority feature but architecturally simple to add once camera smoothing is implemented.

### 7.3 Time-Lapse Recording

Combine cinematic camera paths with accelerated simulation speed (16x, 32x) to create time-lapse recordings of city growth. The system records frames at a fixed interval and exports them as an image sequence or video file. This requires Bevy's `screenshot` functionality combined with the camera path system.

---

## 8. Zoom-Level-of-Detail Integration

### 8.1 Current LOD System Analysis

The existing LOD system uses three tiers for citizens:

```rust
pub enum LodTier {
    Full,       // ~5K citizens -- individual pathfinding, rendered with GLTF models
    Simplified, // ~50K citizens -- pre-computed paths, rendered as smaller meshes
    Abstract,   // ~200K citizens -- state machine only, not rendered
}
```

LOD assignment in `assign_lod_tiers()` is based on viewport bounds with margins:
- **Full:** Within viewport + 500 unit margin
- **Simplified:** Within viewport + 1500 unit margin
- **Abstract:** Everything else

This is a good foundation but has two limitations:

1. **Camera distance is not considered:** At satellite zoom (distance 4000), ALL citizens visible in the viewport are assigned Full tier, even though they are invisible at that zoom level. The LOD system should consider both viewport bounds AND camera distance.

2. **No building LOD:** Buildings are always rendered at full detail. At satellite zoom, a single building is sub-pixel. Rendering 10,000 GLTF building scenes at 1/4 pixel each is wasteful.

### 8.2 Comprehensive LOD Budget

The rendering budget should be allocated across zoom levels. Target: 60 FPS on mid-range hardware (RTX 3060 / Apple M2).

**Satellite view (distance > 3000):**
- Terrain: 32x32 chunk meshes with vertex colors (current system, ~65K triangles total, very efficient)
- Roads: Road segment meshes remain visible (they are large enough to see as lines). Consider reducing tessellation segments at far zoom.
- Buildings: Replace individual GLTF scenes with a single instanced "dot" mesh per chunk. Each chunk calculates an average height and dominant zone color, then renders a single extruded quad. This reduces 10,000+ draw calls to 1,024 (32x32 chunks).
- Citizens: All Abstract tier. No rendering.
- Trees: Not rendered. Tree visibility = 0.
- Props: Not rendered. Street lamp visibility = 0.
- Overlays: Full resolution on terrain chunks.
- **Triangle budget:** ~100K (terrain + simplified buildings + roads)

**Regional view (distance 1500-3000):**
- Terrain: Same chunk meshes.
- Roads: Full segment meshes.
- Buildings: Instanced low-poly representations. Group buildings by zone type within each chunk and render as colored boxes (no GLTF loading). Each building becomes a single-color cuboid at its grid position.
- Citizens: All Abstract. No rendering.
- Trees: Clustered -- render one larger tree per 4x4 cell area containing trees, instead of individual trees.
- Props: Not rendered.
- **Triangle budget:** ~300K

**City view (distance 600-1500):**
- Terrain: Same chunk meshes.
- Roads: Full segment meshes with lane markings.
- Buildings: GLTF scenes for buildings in viewport, simplified cuboids for buildings outside viewport but within render range.
- Citizens: Abstract in viewport, not rendered.
- Trees: Individual tree props visible within viewport.
- Props: Street lamps visible within viewport.
- **Triangle budget:** ~500K

**Neighborhood view (distance 200-600):**
- Terrain: Full detail.
- Roads: Full detail with all markings and curbs.
- Buildings: Full GLTF scenes within viewport + 200 unit margin. Cuboids beyond.
- Citizens: Simplified tier within viewport. Small colored capsules representing citizens.
- Trees: Full detail.
- Props: Full detail (lamps, parked cars, benches).
- **Triangle budget:** ~1M

**Block/Street/Pedestrian view (distance < 200):**
- Everything at full detail within viewport.
- Full-tier citizens with GLTF character/vehicle models.
- Building facades with construction animations.
- Road markings, crosswalks, traffic signals.
- Status icons over buildings.
- **Triangle budget:** ~2M

### 8.3 LOD Transition Strategies

The transition between LOD levels must be smooth. Abrupt "popping" (where a simplified representation suddenly swaps to a detailed one) is visually jarring and draws the player's eye to the wrong things.

**Cross-fade (recommended for buildings):** During the transition zone, render both the simplified and detailed versions simultaneously, with the simplified version fading out (alpha approaching 0) as the detailed version fades in (alpha approaching 1). This requires:
- Both meshes to exist simultaneously (memory cost)
- Alpha blending on building materials (rendering cost)
- A transition zone width of at least 50 world units to be imperceptible

**Scale-up (current implementation for citizens):** The `CitizenMeshKind` swaps between humanoid and car meshes, and the `lod_factor` adjusts scale:

```rust
let lod_factor = match lod {
    LodTier::Simplified => 0.5,
    _ => 1.0,
};
transform.scale = Vec3::splat(base_scale * lod_factor);
```

This works because citizens are small and moving -- the player does not notice a size change. For buildings, which are large and stationary, scale changes would be very noticeable.

**Hysteresis band:** To prevent flickering when the camera oscillates near a LOD boundary, use a hysteresis band:

```rust
// When transitioning FROM Full TO Simplified:
let downgrade_distance = 600.0;

// When transitioning FROM Simplified TO Full:
let upgrade_distance = 550.0;  // 50 units closer before upgrading

// This means a building at 575 units stays at whatever tier it's currently at
```

Without hysteresis, a building at exactly 600 units would flicker between Full and Simplified every frame as floating-point noise moves it across the threshold.

### 8.4 Bevy Implementation: Visibility and Render Layers

Bevy provides `Visibility` components that can be `Visible`, `Hidden`, or `Inherited`. The current system uses these for citizen visibility based on state (AtHome = Hidden). LOD-based visibility should extend this:

```rust
fn update_prop_visibility(
    orbit: Res<OrbitCamera>,
    mut props: Query<(&GlobalTransform, &mut Visibility), With<PropEntity>>,
) {
    let show_props = orbit.distance < 600.0;
    for (transform, mut vis) in &mut props {
        if show_props {
            *vis = Visibility::Inherited;
        } else {
            *vis = Visibility::Hidden;
        }
    }
}
```

For more granular control, Bevy render layers can separate objects into groups:

```rust
// Props on layer 1, buildings on layer 0
commands.spawn((
    PropEntity,
    RenderLayers::layer(1),
    // ...
));

// Camera includes layer 1 only when zoomed in
fn update_camera_render_layers(
    orbit: Res<OrbitCamera>,
    mut camera: Query<&mut RenderLayers, With<Camera3d>>,
) {
    let mut layers = RenderLayers::layer(0); // always show terrain/buildings
    if orbit.distance < 600.0 {
        layers = layers.with(1); // show props
    }
    if orbit.distance < 200.0 {
        layers = layers.with(2); // show detailed citizen models
    }
    *camera.single_mut() = layers;
}
```

### 8.5 Chunk-Based Culling

The current terrain rendering uses 32x32 chunks (CHUNK_SIZE=8 cells per chunk, 8*16=128 world units per chunk side). Chunks outside the camera frustum should not be rendered. Bevy performs frustum culling automatically on meshes with `Aabb` (axis-aligned bounding boxes), which is computed automatically for `Mesh3d` entities. However, building entities, prop entities, and citizen entities should also benefit from spatial culling.

For buildings and props, the current system spawns them all with default Visibility. Bevy's frustum culling will handle them automatically for `Mesh3d` entities. For `SceneRoot` entities (GLTF scenes), frustum culling depends on the scene hierarchy having correct bounds.

**Recommendation:** Verify that GLTF scene bounds are computed correctly by Bevy's automatic AABB calculation. If not, manually insert `Aabb` components with correct bounds.

---

## 9. Data Overlay System

### 9.1 Current Overlay Implementation

The current system defines nine overlay modes:

```rust
pub enum OverlayMode {
    None, Power, Water, Traffic, Pollution,
    LandValue, Education, Garbage, Noise, WaterPollution,
}
```

Overlays are rendered by modifying the vertex colors of terrain chunk meshes in `build_chunk_mesh()`. When an overlay is active, the `apply_overlay()` function blends a data-dependent color over the base terrain color. This approach has several strengths:

- Zero additional draw calls (overlay is baked into existing terrain mesh)
- No transparency sorting issues
- Works at any zoom level
- Simple implementation

And several weaknesses:

- Requires rebuilding all 1,024 chunk meshes when overlay data changes (expensive)
- Cannot show overlays independently of terrain (e.g., over buildings)
- Limited to 256x256 resolution (one value per cell)
- No smooth interpolation between cells

The current `dirty_chunks_on_overlay_change` system marks all chunks dirty when overlay data changes, triggering a full rebuild. For overlays that change frequently (traffic congestion updates every few seconds), this is a significant performance cost: 1,024 chunks times ~100 vertices each = ~100K vertices rebuilt per frame during overlay changes.

### 9.2 Overlay Color Ramps

The current overlay colors use hardcoded RGBA values per data type. A more systematic approach would use three types of color ramps:

**Diverging ramps (good/neutral/bad):**
- Green (high value) -- White/Gray (neutral) -- Red (low value)
- Used for: Land Value, Happiness, Education, Services Coverage
- Example: Land value 0 = dark red, 128 = white, 255 = dark green

**Sequential ramps (none-to-lots):**
- Transparent (no value) -- Intense color (high value)
- Used for: Pollution, Noise, Traffic, Garbage, Water Pollution
- Example: Pollution 0 = base terrain, 255 = opaque dark red/brown

**Categorical ramps (type-based):**
- Distinct colors per category, not interpolated
- Used for: Zone types, District assignment, Power/Water (binary yes/no)
- Example: Residential = green, Commercial = blue, Industrial = yellow

**Color ramp implementation:**

```rust
struct ColorRamp {
    stops: Vec<(f32, Color)>,  // (position 0-1, color)
}

impl ColorRamp {
    fn sample(&self, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        for i in 0..self.stops.len() - 1 {
            let (t0, c0) = self.stops[i];
            let (t1, c1) = self.stops[i + 1];
            if t >= t0 && t <= t1 {
                let local_t = (t - t0) / (t1 - t0);
                return color_lerp(c0, c1, local_t);
            }
        }
        self.stops.last().unwrap().1
    }

    fn diverging_red_green() -> Self {
        Self {
            stops: vec![
                (0.0, Color::srgb(0.7, 0.1, 0.1)),   // dark red
                (0.25, Color::srgb(0.9, 0.4, 0.2)),   // orange
                (0.5, Color::srgb(0.95, 0.95, 0.9)),   // near-white
                (0.75, Color::srgb(0.4, 0.8, 0.3)),   // light green
                (1.0, Color::srgb(0.1, 0.6, 0.1)),    // dark green
            ],
        }
    }
}
```

### 9.3 Overlay Rendering Improvements

**Separate overlay mesh layer:** Instead of baking overlay colors into terrain vertex colors, render overlays as a separate transparent mesh layer above the terrain. This allows:
- Overlays to extend over buildings (tint building rooftops)
- Smooth bilinear interpolation between cell values
- No chunk rebuilds when overlay data changes (just update the overlay mesh)
- Multiple overlays visible simultaneously

```rust
// Overlay mesh: one quad per cell, slightly above terrain (Y=0.1)
// Uses a custom shader with vertex colors and alpha blending
fn build_overlay_mesh(grid: &OverlayGrid, color_ramp: &ColorRamp) -> Mesh {
    // Similar to terrain chunk mesh but:
    // - Positioned at Y=0.1 (above terrain)
    // - Alpha from overlay intensity (transparent where no data)
    // - Color from color_ramp.sample(value / max_value)
}
```

**Smooth interpolation:** The current per-cell coloring creates a blocky appearance at close zoom. Bilinear interpolation across cell boundaries produces much smoother gradients:

```rust
// For each vertex at (gx, gy) corner, average the 4 adjacent cell values
let value = (
    grid.get(gx, gy) + grid.get(gx-1, gy) +
    grid.get(gx, gy-1) + grid.get(gx-1, gy-1)
) / 4.0;
```

### 9.4 Flow Visualization

Some data is inherently directional -- traffic flow, wind direction, water flow. These should be visualized with animated arrows or particles rather than static colors.

**Traffic flow arrows:**

```rust
// For each road cell with traffic data, spawn a moving arrow glyph
// Arrow moves in the direction of traffic flow
// Arrow opacity proportional to traffic volume
// Arrow color: green (free flow) to red (congested)

fn spawn_traffic_flow_arrows(
    grid: &WorldGrid,
    traffic: &TrafficGrid,
    // ...
) {
    for each road cell {
        let flow_direction = traffic.flow_direction(gx, gy);
        let volume = traffic.volume(gx, gy);
        // Spawn arrow entity with animated UV offset
    }
}
```

**Wind direction:** When the wind overlay is active, render animated streamlines across the map showing wind direction and speed. This is useful for planning wind turbine placement and understanding pollution dispersal.

### 9.5 Network Visualization

When placing utilities (power plants, water towers), the player needs to see the connected network -- which cells have power, which do not, and where the network boundaries are.

The current Power and Water overlays use binary coloring (yellow = has power, red = no power). This is functional but does not show:
- Network connectivity (which cells are connected to which source)
- Network capacity (is a power plant at max capacity?)
- Network path (which roads carry the power lines?)

**Improved network visualization:**
- Highlight the source building with a pulsing glow
- Draw animated "pulse" lines along the network paths (like electricity flowing through wires)
- Show capacity as a fill bar on each source building
- Color code cells by their source (e.g., cells powered by Plant A in blue, Plant B in orange)

### 9.6 Overlay Legend

When an overlay is active, a legend should be visible showing the color ramp and value range. Currently, there is no legend -- the player has to guess what the colors mean.

```rust
fn draw_overlay_legend(
    ui: &mut egui::Ui,
    overlay: &OverlayMode,
    // data range information
) {
    // Draw a vertical gradient bar (150px tall, 20px wide)
    // Label the top with max value
    // Label the bottom with min value
    // Label the middle with the overlay name
    // Position in the bottom-left corner of the screen
}
```

### 9.7 Multiple Overlay Blending

Some analysis requires seeing two data layers simultaneously -- for example, pollution + land value to understand the correlation. The current system allows only one overlay at a time.

**Recommendation:** Allow two overlays simultaneously with blending:
- Primary overlay: full color ramp as normal
- Secondary overlay: rendered as a hatching pattern (diagonal lines whose density represents value)
- This avoids the color confusion of trying to blend two color ramps

Alternative: Split-screen overlay mode where the left half shows overlay A and the right half shows overlay B, with a draggable divider.

---

## 10. Selection and Interaction

### 10.1 Click-to-Select

The current selection system uses `SelectedBuilding(pub Option<Entity>)` to track the currently inspected building. Selection happens in `handle_tool_input` when the active tool is `Inspect`:

```rust
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
```

This is functional but limited. A comprehensive selection system needs to handle:

1. **Building selection:** Click on any cell belonging to a building to select the building entity. For multi-cell buildings (service buildings with footprints), clicking any cell of the footprint should select the building.

2. **Road selection:** Click on a road cell to select the road segment. Show segment info (road type, traffic volume, connected intersections). For Bezier road segments, the selected segment should be highlighted with a colored outline.

3. **Citizen selection:** Click on a visible citizen mesh to select the citizen. Show citizen info (name, age, job, happiness, current activity, home/work locations). This requires raycasting against citizen meshes, which is more expensive than grid-based building lookup.

4. **District selection:** Click on a cell to show its district assignment and district statistics.

5. **Empty cell selection:** Click on an empty grass cell to show cell info (elevation, land value, zone type, pollution level, nearby services).

**Selection priority:** When multiple selectable entities overlap (e.g., a citizen standing in front of a building), use this priority order:
1. Citizens (if visible at current zoom)
2. Buildings
3. Road segments
4. Empty cells

### 10.2 Selection Visual Feedback

When an entity is selected, it needs clear visual feedback:

**Outline glow:** Render a bright outline (2-3 pixels) around the selected entity. In Bevy, this can be implemented with a post-processing shader or by rendering a slightly scaled-up version of the mesh in a solid color behind the original:

```rust
// Spawn a "selection highlight" entity as a child of the selected building
commands.spawn((
    SelectionHighlight,
    Mesh3d(building_mesh.clone()),
    MeshMaterial3d(highlight_material), // bright semi-transparent color
    Transform::from_scale(Vec3::splat(1.05)), // 5% larger
));
```

**Pulsing animation:** The selection highlight should pulse gently (alpha oscillating between 0.3 and 0.6 at 2Hz) to draw the eye without being distracting.

**Connected entity highlighting:** When a building is selected, highlight related entities:
- Residents (citizens who live in this building): show lines connecting building to citizens
- Workplace (for residential buildings, show where residents work)
- Service coverage (for service buildings, show the coverage radius as a circle on the ground)

### 10.3 Box Selection

Drag to select multiple entities. This is essential for:
- Selecting multiple buildings to bulldoze
- Selecting an area to zone
- Selecting multiple road segments to upgrade

**Implementation:**

```rust
#[derive(Resource, Default)]
pub struct BoxSelect {
    pub active: bool,
    pub start_screen: Vec2,
    pub start_world: Vec2,
    pub selected_entities: Vec<Entity>,
}

fn box_select_system(
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    cursor: Res<CursorGridPos>,
    mut box_select: ResMut<BoxSelect>,
    // ...
) {
    // Start box select on Shift+Left-Click
    if buttons.just_pressed(MouseButton::Left) && keys.pressed(KeyCode::ShiftLeft) {
        box_select.active = true;
        box_select.start_world = cursor.world_pos;
    }

    if box_select.active {
        // Draw selection rectangle (use gizmos)
        // On release, select all entities within the rectangle
    }
}
```

The selection rectangle should be rendered as a translucent colored overlay on the ground plane, with the corners at the start and current cursor positions.

### 10.4 Right-Click Context Menu vs. Tool Panels

Two UI paradigms exist for context-sensitive actions:

**Right-click context menu (Anno 1800, most RTS games):** Right-clicking on a building shows a floating menu with options (inspect, upgrade, demolish, relocate). Advantages: discoverable, doesn't require memorizing tools, direct manipulation.

**Tool panel (CS1, SimCity):** The player selects a tool first (bulldoze, upgrade), then clicks on targets. Advantages: faster for batch operations (bulldoze 20 buildings), clearer mental model ("I am in bulldoze mode").

**Recommendation for Megacity:** Use the tool panel approach (already implemented) as the primary interaction model, but add a right-click context menu as a secondary option for discovery and single-entity operations. Right-clicking on any entity shows a small menu with the most common actions for that entity type.

```rust
// Context menu items by entity type:
// Building: [Inspect, Bulldoze, Upgrade, Set Policy, Toggle Landmark]
// Road: [Inspect, Upgrade, Downgrade, Bulldoze, One-Way Toggle]
// Citizen: [Follow, View Details, Find Home, Find Work]
// Empty Cell: [Zone As..., Place Service..., View Cell Info]
```

**Important caveat:** Right-click is currently used for camera orbit drag. These two uses conflict. Resolution options:
1. Right-click drag = orbit, right-click release without drag = context menu (already somewhat handled by the drag threshold concept)
2. Use a different key for context menu (Alt+click, or double-right-click)
3. Remove right-click orbit in favor of middle-mouse orbit only

Option 1 is recommended: if the right mouse button is pressed and released without significant mouse movement (less than 5 pixels), show the context menu. If the mouse moves more than 5 pixels while held, treat it as an orbit drag.

---

## 11. Tool System UX

### 11.1 Current Tool Architecture

The current `ActiveTool` enum has 90+ variants covering roads, zones, utilities, services, terrain tools, district tools, and environment tools. The toolbar UI in `toolbar.rs` organizes these into 15 categories displayed as buttons along the bottom of the screen.

**Tool activation flow:**
1. Player clicks a category button in the bottom toolbar (e.g., "Roads")
2. A popup appears above the toolbar showing items in that category
3. Player clicks an item (e.g., "Boulevard")
4. The tool is activated, cursor preview shows the tool ghost
5. Player clicks/drags on the map to use the tool
6. Tool remains active until another tool is selected or Escape is pressed

This flow is sound and matches the CS1 model. However, the popup system has UX issues:

### 11.2 Tool Activation UX Improvements

**Problem 1: Popup covers map.** The category popup appears directly above the bottom toolbar, obscuring the part of the map the player is likely looking at. When the player selects a tool, the popup closes, but the player then has to re-find the location they wanted to build on.

**Solution:** Make the popup smaller and position it as a horizontal strip rather than a grid. Or use a sliding panel from the left/right edge.

**Problem 2: No keyboard shortcut for sub-tools.** The current keyboard shortcuts (1-9) map to broad categories, but there is no way to select specific tools within a category via keyboard.

**Solution:** Two-key system: press the category number, then a sub-number. E.g., "1" opens Roads, then "3" selects Boulevard. This is the CS1 approach.

**Problem 3: Tool cost not visible until tool is selected.** Players cannot compare costs across tools in a category without selecting each one individually. The toolbar shows costs but requires selection first.

**Solution:** The current popup grid already shows cost next to each tool name. This is good. Add a tooltip on hover that shows additional info (maintenance cost, coverage radius, capacity).

### 11.3 Tool Preview (Ghost/Transparent)

The current cursor preview system (`cursor_preview.rs`) spawns a single cuboid that follows the cursor and changes color based on validity:

```rust
// Green-ish: valid placement
// Red: invalid placement
// Size: scales to building footprint
```

This is a good foundation but needs enhancement:

**Building preview:** Instead of a generic white cuboid, show a simplified outline of the actual building that will be placed. For service buildings, show the coverage radius as a circle on the ground.

```rust
fn update_cursor_preview(/* ... */) {
    // ... existing code ...

    // If placing a service building, draw coverage radius
    if let Some(st) = tool.service_type() {
        let radius = ServiceBuilding::coverage_radius(st);
        gizmos.circle(
            Isometry3d::new(
                Vec3::new(wx, 0.1, wz),
                Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)
            ),
            radius,
            Color::srgba(0.3, 0.8, 0.3, 0.5),
        );
    }
}
```

**Road preview:** The current Bezier preview (`draw_bezier_preview`) draws a yellow line from the start point to the cursor. This should be enhanced to show:
- The full road width (not just a center line)
- Estimated cost of the road segment (displayed as floating text near the cursor)
- Intersection markers where the new road would cross existing roads
- Warning indicators for steep grade changes

**Zone preview:** When painting zones, show the zone color fill for the cells that would be zoned, with invalid cells highlighted in red. Currently, zones are painted one cell at a time on click/drag. Consider showing a larger brush preview (3x3 or 5x5 area).

### 11.4 Undo/Redo System

Undo/redo is critical for city builders because mistakes are expensive. The player invests money and time in placements, and an accidental bulldoze of a leveled-up building is devastating without undo.

**Architecture:**

```rust
#[derive(Resource)]
pub struct ActionHistory {
    undo_stack: Vec<CityAction>,
    redo_stack: Vec<CityAction>,
    max_depth: usize,  // 100 actions
}

enum CityAction {
    PlaceRoad { segment_id: SegmentId, road_type: RoadType, cost: f64 },
    PlaceZone { gx: usize, gy: usize, old_zone: ZoneType, new_zone: ZoneType },
    PlaceBuilding { entity: Entity, service_type: ServiceType, gx: usize, gy: usize, cost: f64 },
    Bulldoze { /* snapshot of what was destroyed */ },
    PlaceUtility { entity: Entity, utility_type: UtilityType, gx: usize, gy: usize, cost: f64 },
    TerrainEdit { cells: Vec<(usize, usize, f32, f32)> }, // (gx, gy, old_elevation, new_elevation)
    // Composite actions (e.g., "draw road" is multiple PlaceRoad + auto-intersection)
    Composite(Vec<CityAction>),
}

impl ActionHistory {
    fn push(&mut self, action: CityAction) {
        self.undo_stack.push(action);
        self.redo_stack.clear(); // new action invalidates redo history
        if self.undo_stack.len() > self.max_depth {
            self.undo_stack.remove(0);
        }
    }

    fn undo(&mut self) -> Option<CityAction> {
        if let Some(action) = self.undo_stack.pop() {
            self.redo_stack.push(action.clone());
            Some(action)
        } else {
            None
        }
    }
}
```

**Important design decision:** What is the granularity of an "action"? If the player drags to place 50 cells of road, is that 50 actions or 1? It should be 1 -- the "drag to place road" gesture is a single logical action, and undoing it should remove the entire road segment, not one cell.

This means the action system must group sub-actions during a drag operation:

```rust
// During drag:
action_history.begin_composite();
for each cell placed {
    action_history.add_sub_action(PlaceRoad { ... });
}
action_history.end_composite();  // wraps all sub-actions into one Composite
```

**Memory cost:** Each action stores the data needed to reverse it. For road placement, this is a few hundred bytes. For terrain editing (which can affect a radius of cells), it could be several KB per action. With a 100-action limit, total memory is bounded at a few MB -- negligible.

**Keyboard binding:** Ctrl+Z for undo, Ctrl+Y (or Ctrl+Shift+Z) for redo. These are universal and non-negotiable.

---

## 12. Road Building UX

### 12.1 Current Road Drawing System

The current system uses a two-click Bezier segment approach:

1. First click places the start point
2. Second click places the end point, creating a straight segment (control points at 1/3 and 2/3 for linear Bezier)
3. After placement, the end point becomes the start of the next segment (chaining)
4. Escape or right-click cancels

This is functional and handles the basic case well. The Bezier preview (`draw_bezier_preview`) shows the planned road as a yellow line.

### 12.2 Road Drawing Mode Enhancements

**Straight mode (current):** Click start, click end. Control points are placed for a straight line. This handles grid-aligned roads well.

**Curve mode (new):** Click start, then as the mouse moves, the road curves to maintain continuity with the previous segment. The control points are computed to create a smooth curve:

```rust
// When chaining from a previous segment:
// The new segment's P1 control point should be the reflection of the
// previous segment's P2 control point across the junction point.
// This ensures C1 continuity (smooth tangent at the junction).

let prev_tangent = (prev_segment.p3 - prev_segment.p2).normalize();
let new_p1 = junction_point + prev_tangent * (new_segment_length / 3.0);
```

**Freehand mode (new):** Hold the mouse button and draw a freehand path. The system fits Bezier curves to the drawn path using a curve fitting algorithm. This produces organic, realistic road layouts.

**Parallel road mode (new):** After placing a road, hold Shift and move the mouse to place a parallel road at a configurable offset. This is essential for boulevard and highway construction where you need matched pairs.

### 12.3 Road Upgrade UX

Currently, upgrading a road requires bulldozing it and replacing it. This is tedious and loses road connections temporarily. A dedicated upgrade tool should:

1. Click on an existing road segment
2. Show a popup with available upgrade/downgrade options
3. Display the cost difference (upgrade cost minus salvage value)
4. Apply the upgrade in-place without disrupting traffic

### 12.4 Intersection Auto-Detection

The current `RoadSegmentStore` handles intersection detection when new segments cross existing ones. The UX should show intersection previews:

- When the cursor crosses an existing road during placement, show an intersection marker (a colored dot at the crossing point)
- Show the intersection type that would be created (T-intersection, 4-way, merging)
- Allow the player to adjust the intersection by moving the end point

### 12.5 Grade and Elevation Indicators

For bridges and tunnels (future features), the road building UX needs to communicate vertical changes:

- Show elevation markers along the road preview (numbers showing height at regular intervals)
- Color-code the preview by grade: green for flat (0-3%), yellow for moderate (3-6%), red for steep (6%+)
- Show bridge/tunnel indicators where the road crosses water or terrain above/below

### 12.6 Road Snapping

The current system does not implement sophisticated snapping. Enhanced snapping should provide:

**Grid snapping:** Roads snap to cell boundaries and centers. Current behavior for Ctrl+held grid-mode roads.

**Intersection snapping:** When the cursor is near an existing intersection, snap the road endpoint to the exact intersection position. This prevents near-miss connections that look aligned but are not actually connected.

**Angle snapping:** When Shift is held, constrain the road angle to 15-degree increments (0, 15, 30, 45, 60, 75, 90, ...). This makes it easy to draw precise diagonal roads.

**Parallel snapping:** When drawing a road near an existing road, offer a snap to maintain a constant offset (useful for one-way pair streets or adding parallel service roads).

### 12.7 Road Cost Display

During road placement, show the estimated cost in real-time:

```rust
// In draw_bezier_preview or a companion system:
fn draw_road_cost_preview(
    draw_state: Res<RoadDrawState>,
    cursor: Res<CursorGridPos>,
    tool: Res<ActiveTool>,
    budget: Res<CityBudget>,
    // ...
) {
    if draw_state.phase != DrawPhase::PlacedStart { return; }

    let length = (cursor.world_pos - draw_state.start_pos).length();
    let cells = (length / CELL_SIZE).ceil() as usize;
    let cost_per_cell = tool.cost().unwrap_or(0.0);
    let total_cost = cost_per_cell * cells as f64;

    let color = if budget.treasury >= total_cost {
        Color::GREEN
    } else {
        Color::RED
    };

    // Render cost text near cursor using egui or 3D text
}
```

---

## 13. Information Display

### 13.1 HUD (Heads-Up Display)

The current top bar shows:
- Milestone name (population tier)
- Population count
- Treasury
- Date/time + season
- Speed controls (pause, 1x, 2x, 4x)
- Happiness percentage
- Save/Load/New Game buttons
- Active tool + cost

This is a solid HUD but is missing several important indicators:

**RCI Demand Bars:** The most critical missing element. Every city builder since SimCity (1989) shows Residential/Commercial/Industrial demand as colored bars. The current `ZoneDemand` resource exists but is not displayed in the toolbar (the `_demand` parameter is prefixed with underscore, indicating it was planned but not implemented).

Demand bars should be prominently visible, showing:
- Green bar (upward): positive demand (build more of this zone)
- Red bar (downward): negative demand (surplus, stop building)
- Three bars: R, C, I (with sub-bars for low/high density if space permits)

```rust
// In toolbar_ui, add demand bars:
let r_demand = demand.residential;
let c_demand = demand.commercial;
let i_demand = demand.industrial;

ui.separator();
ui.label("R");
draw_demand_bar(ui, r_demand, Color::GREEN);
ui.label("C");
draw_demand_bar(ui, c_demand, Color::BLUE);
ui.label("I");
draw_demand_bar(ui, i_demand, Color::YELLOW);
```

**Income/expense indicator:** Show the current income rate (+$X/month) next to the treasury. If expenses exceed income, show it in red. This is more actionable than just showing the total treasury -- a player with $50,000 but -$2,000/month is in trouble.

**Simulation speed indicator:** The current speed buttons (||, 1x, 2x, 4x) work but are not visually prominent enough. Consider adding a colored indicator: paused = red dot, 1x = green, 2x = yellow, 4x = orange.

### 13.2 Building Info Panel

The current building inspection panel (`building_inspection_ui`) shows comprehensive information when a building is selected. Based on the info_panel.rs code (which is extensive at 100KB+), it appears to show:
- Building type and level
- Zone type
- Occupancy
- Grid position
- Land value
- Service coverage (fire, police, health, education, garbage, death care)
- Utility status (power, water)
- Pollution levels
- Happiness factors
- And much more

This is very thorough. UX recommendations for the info panel:

**Tab organization:** With this much information, organize it into tabs:
- **Overview:** Building type, level, occupancy, happiness (one-glance summary)
- **Services:** Coverage and quality for each service type
- **Economy:** Land value, rent income, property value, taxes
- **Residents/Workers:** List of citizens who live/work here (clickable to follow)
- **Environment:** Pollution, noise, green space, land value factors

**Progressive disclosure:** Show the most important information first (type, level, happiness) and let the player expand sections for more detail. Do not dump 30 stats in a flat list.

### 13.3 Tooltips on Hover

When hovering over any cell (without clicking), show a brief tooltip with the most relevant information for that cell:

```rust
fn show_cell_tooltip(
    cursor: Res<CursorGridPos>,
    grid: Res<WorldGrid>,
    mut egui_ctx: EguiContexts,
) {
    if !cursor.valid { return; }

    let cell = grid.get(cursor.grid_x as usize, cursor.grid_y as usize);

    // Show tooltip near cursor (offset 20px right and down)
    egui::Area::new("cell_tooltip")
        .fixed_pos(cursor_screen_pos + Vec2::new(20.0, 20.0))
        .show(egui_ctx.ctx_mut(), |ui| {
            match cell.cell_type {
                CellType::Grass => {
                    if cell.zone != ZoneType::None {
                        ui.label(format!("Zone: {:?}", cell.zone));
                    } else {
                        ui.label("Empty land");
                    }
                    ui.label(format!("Elevation: {:.2}", cell.elevation));
                }
                CellType::Road => {
                    ui.label(format!("Road: {:?}", cell.road_type));
                }
                CellType::Water => {
                    ui.label("Water");
                }
            }
        });
}
```

Tooltips should be delayed (appear after 500ms of hovering to avoid flickering) and should not appear during drag operations.

### 13.4 Minimap

A minimap in the corner of the screen is standard for city builders and provides essential spatial awareness:

```rust
struct MinimapConfig {
    position: MinimapPosition,  // TopRight, TopLeft, BottomRight, BottomLeft
    size: f32,                  // 150-250 pixels
    show_viewport_rect: bool,   // rectangle showing current camera view
    show_buildings: bool,       // colored dots for buildings
    show_roads: bool,           // lines for roads
    click_to_navigate: bool,    // click on minimap to jump camera
}
```

**Minimap rendering:** The minimap should show:
- Terrain base colors (same as satellite zoom)
- Road network as thin lines
- Building clusters as colored dots (zone colors)
- Current camera viewport as a white rectangle outline
- Zoom level indicated by rectangle size

**Click interaction:** Clicking on the minimap should instantly (with smooth transition) move the camera focus to the clicked location. This is the fastest way to navigate across the city.

**Minimap rendering approach:** Render the minimap to an offscreen texture using an orthographic camera looking straight down at the city. This texture is then displayed as an egui image. Update the texture at low frequency (every 2-5 seconds) since the minimap does not need to be real-time.

### 13.5 Notification System

City events (fire, crime spike, milestone reached, building abandoned, budget warning) should generate notifications that:

1. Appear as a scrolling ticker at the top of the screen (below the HUD bar)
2. Are color-coded by priority: red (emergency), orange (warning), yellow (attention), blue (info), green (positive)
3. Are clickable -- clicking a notification jumps the camera to the relevant location
4. Persist in a log that the player can review later (the current Event Journal)
5. Auto-dismiss after 10-15 seconds for low-priority, persist until dismissed for emergencies

```rust
struct Notification {
    text: String,
    priority: NotificationPriority,
    location: Option<Vec2>,  // world position to jump to
    timestamp: f32,          // game time when created
    dismissed: bool,
}

enum NotificationPriority {
    Emergency,  // Fire, tornado, budget crisis
    Warning,    // Service shortage, high crime area
    Attention,  // Building abandoned, traffic congestion
    Info,       // Milestone reached, new unlock
    Positive,   // Population milestone, happiness increase
}
```

### 13.6 Charts and Graphs Panel

The current `graphs.rs` module records history data and displays it. This should include:

**Population chart:** Line graph showing total population over time, with optional sub-lines for residential/commercial/industrial workers.

**Budget chart:** Stacked area chart showing income sources (taxes by zone type) and expense categories (services, utilities, maintenance) over time. The net line (income minus expenses) shows budget trends.

**Traffic chart:** Bar chart showing average congestion by time of day (24-hour cycle). Helps identify rush hour patterns.

**Service coverage chart:** Radar/spider chart showing coverage percentages for each service type. Quickly identifies which services need more facilities.

**Happiness breakdown:** Stacked bar showing happiness factors (positive and negative) for the average citizen. Identifies the biggest happiness detractors.

### 13.7 Advisor System

Context-sensitive tips that trigger based on city state. The current `advisors.rs` module exists, and the `AdvisorPanel` resource is referenced in `info_panel.rs`.

Advisors should be:
- **Non-intrusive:** Appear as a small icon in the notification area, not as a modal popup
- **Actionable:** Each tip should include a "show me" button that jumps to the relevant location and/or activates the relevant tool
- **Dismissible:** Players who know what they're doing can permanently dismiss tips
- **Contextual:** Only trigger when the condition is true and the player has not already addressed it

Example advisor messages:
- "Traffic congestion is high on the road connecting Downtown to the Industrial District. Consider adding parallel routes or upgrading to a Boulevard." [Show Location] [Dismiss]
- "Your city has no fire coverage in the northwest neighborhood. Residents are at risk." [Place Fire Station] [Show Area]
- "You have excess commercial demand. Zone more commercial areas near existing roads." [Zone Commercial] [Dismiss]
- "Your budget deficit of -$500/month will deplete your treasury in 6 months. Consider raising tax rates or reducing service budgets." [Open Budget] [Dismiss]

---

## 14. Keyboard Shortcuts and Hotkeys

### 14.1 Current Keybindings

The current implementation has:

**Camera:**
- WASD / Arrow Keys: Pan
- Middle-mouse drag: Pan
- Left-click drag (with threshold): Pan
- Right-click drag: Orbit (yaw + pitch)
- Scroll wheel: Zoom

**Tools (number keys):**
- 1: Road (Local)
- 2: Residential Low
- 3: Commercial Low
- 4: Industrial
- 5: Bulldoze
- 6: Residential High
- 7: Commercial High
- 8: Office
- 9: Inspect

**Overlays (letter keys):**
- P: Power
- O: Water
- T: Traffic
- N: Pollution
- L: Land Value
- E: Education
- G: Garbage
- M: Noise
- U: Water Pollution

**Other:**
- Escape: Cancel road drawing
- Right-click: Cancel road drawing
- Ctrl+held: Legacy grid-snap road mode

### 14.2 Recommended Complete Keybinding Layout

The current keybindings conflict with each other and with potential future features. Here is a comprehensive non-conflicting layout:

**Camera:**
- WASD: Pan (relative to camera direction)
- Arrow Keys: Pan (alternative)
- Middle-mouse drag: Pan
- Right-mouse drag: Orbit
- Scroll wheel: Zoom
- Q: Rotate camera 90 degrees left
- E: Rotate camera 90 degrees right
- Home: Reset camera to default position
- F: Focus on selected entity
- Numpad +/-: Zoom in/out (alternative to scroll wheel)

**Simulation:**
- Space: Toggle pause/play
- 1: Speed 1x
- 2: Speed 2x
- 3: Speed 4x

**Tools (via category shortcut):**
- R: Open Roads category
- Z: Open Zones category
- B: Bulldoze (direct)
- I: Inspect (direct)
- V: Open Views/Overlays category
- K: Open Parks category
- G: Open Emergency/Government category
- Tab: Cycle through overlay modes
- Shift+Tab: Cycle overlays in reverse

**Actions:**
- Escape: Cancel current action, close panels, deselect
- Delete: Bulldoze selected entity
- Ctrl+Z: Undo
- Ctrl+Y: Redo
- Ctrl+S: Save game
- Ctrl+L: Load game
- Ctrl+N: New game
- F5: Quick save
- F9: Quick load
- F12: Screenshot

**Bookmarks:**
- Ctrl+F1 through Ctrl+F8: Set bookmark
- F1 through F8: Jump to bookmark

**UI Panels:**
- J: Toggle event journal
- C: Toggle charts panel
- A: Toggle advisor panel
- P: Toggle policies panel (note: conflicts with current Power overlay key)

### 14.3 Keybinding Conflicts and Resolution

The current system has several conflicts:

1. **P key:** Currently toggles Power overlay. Also a natural shortcut for Policies. Resolution: Use Tab to cycle overlays, free up letter keys for other functions.

2. **E key:** Currently toggles Education overlay. Also used for camera rotation (Q/E is standard in many games). Resolution: Move overlay toggles to Tab-cycling, use E for camera rotation.

3. **T key:** Currently toggles Traffic overlay. Also could be used for Transit tools. Resolution: Same -- Tab-cycling for overlays.

4. **Number keys 1-9:** Currently map to tools. Also useful for simulation speed. Resolution: Use 1-3 for speed (Space toggles pause), use R/Z/B/I/V for tool categories.

### 14.4 Customizable Keybindings

Essential for accessibility and player preference. Store keybindings in a configuration file:

```rust
#[derive(Resource, Serialize, Deserialize)]
pub struct KeyBindings {
    pub pan_forward: KeyCode,
    pub pan_backward: KeyCode,
    pub pan_left: KeyCode,
    pub pan_right: KeyCode,
    pub rotate_left: KeyCode,
    pub rotate_right: KeyCode,
    pub zoom_in: KeyCode,
    pub zoom_out: KeyCode,
    pub pause_play: KeyCode,
    pub speed_1: KeyCode,
    pub speed_2: KeyCode,
    pub speed_3: KeyCode,
    pub tool_road: KeyCode,
    pub tool_zone: KeyCode,
    pub tool_bulldoze: KeyCode,
    pub tool_inspect: KeyCode,
    pub undo: KeyBinding,     // KeyBinding = KeyCode + modifiers
    pub redo: KeyBinding,
    // ... etc
}
```

Provide a UI screen where players can view and rebind all keys. Detect and warn about conflicts. Provide a "Reset to Defaults" button.

---

## 15. Controller Support

### 15.1 City Builders on Console

City builders have been successfully adapted to console controllers. Cities: Skylines on PS4/Xbox One proved it is possible with careful UX design. SimCity BuildIt on mobile proved touch controls work. The key challenges:

1. **Cursor precision:** A thumbstick is far less precise than a mouse. Grid-based placement (cell-by-cell) mitigates this -- the cursor snaps to cell centers, so the player only needs to get "close enough."

2. **Camera control vs. cursor control:** Both need analog stick input, but there are only two sticks. Solution: one stick controls the cursor, the other controls the camera.

3. **Tool selection speed:** A mouse-driven toolbar can show 15 categories simultaneously. A controller needs a different paradigm -- radial menus or sequential navigation.

### 15.2 Controller Layout

**Left Stick:** Move cursor on the ground plane (with acceleration -- slow for precision, fast for traversal)

**Right Stick:** Move camera (pan focus point)

**Right Stick Click (R3):** Toggle between "camera pan" and "camera orbit" mode for the right stick

**Left Trigger (LT):** Zoom out (held)

**Right Trigger (RT):** Zoom in (held)

**Left Bumper (LB):** Rotate camera 90 degrees left

**Right Bumper (RB):** Rotate camera 90 degrees right

**A Button:** Primary action (place/select, equivalent to left-click)

**B Button:** Cancel (equivalent to Escape)

**X Button:** Open tool wheel (radial menu)

**Y Button:** Open info panel for item under cursor

**D-Pad Up/Down:** Simulation speed control

**D-Pad Left/Right:** Cycle through overlay modes

**Menu Button:** Pause menu

**View Button:** Toggle minimap / UI visibility

### 15.3 Radial Menu Design

The tool wheel (triggered by X button) should use a radial menu:

```
         Roads
    Env  /    \  Zones
        |      |
   Terrain-  -Utilities
        |      |
    Tools \    / Services
         Transport
```

- Hold X to open the wheel
- Move left stick to highlight a category
- Release X to select (opens sub-wheel for that category)
- Sub-wheel shows individual items (e.g., Local Road, Avenue, Boulevard, Highway)

Radial menus are optimal for controllers because:
- 8 directions are easy to distinguish with a thumbstick
- Selection speed is fast (point and release, no sequential navigation)
- The visual layout provides spatial memory (players remember "Roads is up-right")

### 15.4 Cursor Acceleration

The cursor controlled by the left stick should use acceleration:

```rust
fn controller_cursor(
    axes: Res<Axis<GamepadAxis>>,
    time: Res<Time>,
    mut cursor: ResMut<CursorGridPos>,
    orbit: Res<OrbitCamera>,
) {
    let x = axes.get(GamepadAxis::LeftStickX).unwrap_or(0.0);
    let y = axes.get(GamepadAxis::LeftStickY).unwrap_or(0.0);

    let deadzone = 0.15;
    let x = if x.abs() < deadzone { 0.0 } else { (x - x.signum() * deadzone) / (1.0 - deadzone) };
    let y = if y.abs() < deadzone { 0.0 } else { (y - y.signum() * deadzone) / (1.0 - deadzone) };

    // Cursor speed scales with zoom (faster when zoomed out)
    let base_speed = 200.0 * orbit.distance / 1000.0;

    // Non-linear response curve: slow for fine control, fast for traversal
    let speed_x = x.signum() * x.abs().powf(2.0) * base_speed;
    let speed_y = y.signum() * y.abs().powf(2.0) * base_speed;

    cursor.world_pos.x += speed_x * time.delta_secs();
    cursor.world_pos.y -= speed_y * time.delta_secs();  // Y axis inverted

    // Snap to grid
    cursor.grid_x = (cursor.world_pos.x / CELL_SIZE) as i32;
    cursor.grid_y = (cursor.world_pos.y / CELL_SIZE) as i32;
}
```

The quadratic response curve (`x.abs().powf(2.0)`) means small stick deflections produce slow, precise cursor movement, while full deflection produces fast traversal. This is the standard approach for console strategy games.

### 15.5 Snap-to-Building

When the cursor is near a building, automatically snap to the building's center. This compensates for thumbstick imprecision:

```rust
fn snap_cursor_to_nearest_building(
    cursor: &mut CursorGridPos,
    grid: &WorldGrid,
    snap_radius: usize,  // 2-3 cells
) {
    let gx = cursor.grid_x as usize;
    let gy = cursor.grid_y as usize;

    // Search in a radius for the nearest building
    let mut best_dist = f32::MAX;
    let mut best_pos = None;

    for dy in -(snap_radius as i32)..=(snap_radius as i32) {
        for dx in -(snap_radius as i32)..=(snap_radius as i32) {
            let nx = gx as i32 + dx;
            let ny = gy as i32 + dy;
            if nx >= 0 && ny >= 0 {
                let nx = nx as usize;
                let ny = ny as usize;
                if grid.in_bounds(nx, ny) && grid.get(nx, ny).building_id.is_some() {
                    let dist = ((dx * dx + dy * dy) as f32).sqrt();
                    if dist < best_dist {
                        best_dist = dist;
                        best_pos = Some((nx, ny));
                    }
                }
            }
        }
    }

    if let Some((bx, by)) = best_pos {
        if best_dist < snap_radius as f32 {
            cursor.grid_x = bx as i32;
            cursor.grid_y = by as i32;
        }
    }
}
```

### 15.6 UI Scaling for TV Distance

Console players sit 2-3 meters from their screen (compared to 0.5m for PC). All UI elements need to be larger:

- Minimum text size: 18pt (vs 12pt for PC)
- Button minimum size: 48x48 pixels (vs 24x24 for PC)
- Tooltip text: 16pt minimum
- Info panel: wider margins, larger font, less dense layout
- Icon minimum size: 32x32 pixels

These should be controlled by a UI scale setting:
```rust
enum UIScale {
    PC,         // 1.0x
    Console,    // 1.5x
    Large,      // 2.0x (for accessibility or very large screens)
    Custom(f32),
}
```

---

## 16. Accessibility

### 16.1 Colorblind Modes

Approximately 8% of males and 0.5% of females have some form of color vision deficiency. For a city builder that uses color extensively (zone colors, overlay colors, notification colors), accessibility is not optional.

**Three modes to support:**

1. **Deuteranopia (red-green, most common, ~6% of males):** Cannot distinguish red from green. The standard zone colors (green residential, blue commercial, yellow industrial) are partially affected. Overlay color ramps that use red-green diverging scales are completely unusable.

2. **Protanopia (red-green, ~1% of males):** Similar to deuteranopia but reds appear darker. Same mitigation needed.

3. **Tritanopia (blue-yellow, ~0.01%):** Cannot distinguish blue from yellow. The standard commercial (blue) and industrial (yellow) zone colors are confusable.

**Implementation approach:**

```rust
#[derive(Resource)]
pub struct ColorblindSettings {
    pub mode: ColorblindMode,
}

enum ColorblindMode {
    None,
    Deuteranopia,
    Protanopia,
    Tritanopia,
}

// Color palette lookup table
fn zone_color(zone: ZoneType, mode: ColorblindMode) -> Color {
    match (zone, mode) {
        // Normal palette
        (ZoneType::ResidentialLow, ColorblindMode::None) => Color::srgb(0.2, 0.7, 0.2),
        (ZoneType::CommercialLow, ColorblindMode::None) => Color::srgb(0.2, 0.3, 0.8),
        (ZoneType::Industrial, ColorblindMode::None) => Color::srgb(0.8, 0.7, 0.1),

        // Deuteranopia palette: use blue/orange/purple instead of green/blue/yellow
        (ZoneType::ResidentialLow, ColorblindMode::Deuteranopia) => Color::srgb(0.1, 0.4, 0.8),
        (ZoneType::CommercialLow, ColorblindMode::Deuteranopia) => Color::srgb(0.9, 0.5, 0.1),
        (ZoneType::Industrial, ColorblindMode::Deuteranopia) => Color::srgb(0.6, 0.3, 0.7),

        // ... etc for other modes
    }
}
```

**Additionally:** Use shape/pattern coding in addition to color. Overlay maps should use both color AND hatching patterns:
- Residential zones: horizontal lines
- Commercial zones: dots
- Industrial zones: diagonal lines
- This allows colorblind players to distinguish zones without relying on color

**Overlay color ramps for colorblind modes:**
- Replace red-green diverging with blue-orange diverging (safe for all common types)
- Use viridis-like perceptually uniform colormaps (blue-green-yellow) which are inherently colorblind-safe

### 16.2 Screen Reader Support

For visually impaired players, critical UI elements should be tagged for screen reader compatibility:

- Building inspection: read out building type, level, and key stats
- Notifications: read out notification text with priority
- Toolbar: announce tool name on selection
- Status messages: read out error messages

In egui, this means using `AccessKit` labels:

```rust
ui.label("Population: 45,000")
    .on_hover_text("Current city population count");
// egui's built-in accessibility support handles the rest
```

### 16.3 Text Size and Contrast

**Adjustable text size:** Provide a text scale slider from 75% to 200% of default. The current egui UI uses fixed font sizes -- these should be made relative to a global scale factor.

**High contrast mode:** Option to render UI with maximum contrast:
- Pure white text on pure black backgrounds
- Bold fonts only
- No transparent/semi-transparent overlays
- Border outlines on all UI elements

**Minimum contrast ratios:** Follow WCAG 2.1 guidelines:
- Normal text: minimum 4.5:1 contrast ratio
- Large text: minimum 3:1 contrast ratio
- UI components: minimum 3:1 contrast ratio

### 16.4 Reduced Motion

Some players are sensitive to motion effects. Provide a "reduced motion" option that:

- Disables camera animation (smooth zoom/pan transitions become instant)
- Disables particle effects (construction dust, weather particles)
- Disables pulsing UI elements (selection highlights become static)
- Disables day/night cycle lighting transitions (fixed noon lighting)
- Reduces or eliminates camera shake (if any)

```rust
#[derive(Resource)]
pub struct MotionSettings {
    pub reduced_motion: bool,
    pub camera_smoothing: bool,
    pub particle_effects: bool,
    pub ui_animations: bool,
}
```

### 16.5 One-Handed Play Modes

**Mouse-only mode:** All keyboard shortcuts should have mouse/UI equivalents. The toolbar already provides mouse-based tool selection. Camera controls need mouse-only alternatives:
- Scroll wheel: zoom (already works)
- Edge scrolling: pan (needs implementation)
- Right-click drag: orbit (already works)
- Left-click drag: pan (already works with threshold)

With these four, all camera operations are available via mouse only.

**Keyboard-only mode:** More challenging. Requires:
- WASD: pan (already works)
- Q/E: rotate (needs implementation)
- +/-: zoom (needs implementation)
- Tab: cycle tools (needs implementation)
- Enter: select/place (needs implementation, equivalent to left-click)
- Arrow keys: move cursor between grid cells (needs implementation)
- Escape: cancel

Full keyboard-only play requires a "keyboard cursor" -- a visible cursor on the map that moves with arrow keys and snaps to grid cells. This is similar to the controller cursor described in section 15.

### 16.6 Subtitles and Text Alternatives

If the game includes any voiced content (advisor voice lines, event announcements, tutorial narration):
- Provide subtitles, enabled by default
- Allow subtitle text size adjustment
- Provide speaker identification for multiple speakers
- Include descriptions of significant sound effects (e.g., "[fire alarm sounds]")

Even without voice, provide text alternatives for:
- Sound-based notifications (a building catches fire -- show visual alert, not just audio)
- Music mood (if the soundtrack changes based on city state, show a visual indicator)

---

## 17. Performance UX

### 17.1 Loading Screen

City loading (generating terrain, spawning buildings, initializing simulation) should show:

1. **Progress bar:** Showing percentage complete with estimated time remaining
2. **Loading phase text:** "Generating terrain..." "Placing buildings..." "Initializing economy..."
3. **City preview:** If loading a saved game, show a screenshot of the city taken at save time
4. **Tips and facts:** Rotating set of gameplay tips while loading
5. **City statistics preview:** Show basic stats (population, treasury) from the save file before the full simulation loads

```rust
#[derive(Resource)]
pub struct LoadingScreen {
    pub phase: LoadingPhase,
    pub progress: f32,        // 0.0 to 1.0
    pub tip_index: usize,
    pub show_preview: bool,
    pub preview_image: Option<Handle<Image>>,
}

enum LoadingPhase {
    GeneratingTerrain,
    PlacingRoads,
    SpawningBuildings,
    InitializingCitizens,
    ComputingPathing,
    FinalSetup,
}
```

### 17.2 Frame Rate Management

**Frame rate display:** Optional FPS counter in the corner. Toggle with F3 (standard in many games) or via settings menu. Show:
- Current FPS
- Average FPS (rolling 1-second average)
- Frame time in milliseconds (1/FPS * 1000)
- Optionally: GPU usage percentage

**Target frame rate:** 60 FPS is the target. When FPS drops below threshold:
1. **50-59 FPS:** No action (acceptable)
2. **40-49 FPS:** Reduce prop render distance, reduce citizen LOD
3. **30-39 FPS:** Disable shadows, reduce road marking detail
4. **Below 30 FPS:** Show a warning suggesting lower graphics settings

**Adaptive quality:** Automatically adjust quality settings based on recent frame rate performance:

```rust
fn adaptive_quality(
    mut quality: ResMut<QualitySettings>,
    mut fps_history: Local<Vec<f32>>,
    time: Res<Time>,
) {
    let fps = 1.0 / time.delta_secs();
    fps_history.push(fps);
    if fps_history.len() > 60 { fps_history.remove(0); }

    let avg_fps = fps_history.iter().sum::<f32>() / fps_history.len() as f32;

    if avg_fps < 40.0 && quality.level > QualityLevel::Low {
        quality.level = quality.level.lower();
        // Apply reduced settings
    } else if avg_fps > 55.0 && quality.level < QualityLevel::Ultra {
        quality.level = quality.level.higher();
        // Apply increased settings
    }
}
```

### 17.3 Graphics Quality Presets

**Low:**
- No shadows
- No road markings (just colored asphalt)
- No props (trees, lamps, parked cars)
- Citizens render as colored dots (flat sprites)
- Chunk meshes at half resolution (4-cell chunks instead of 8-cell)
- No day/night cycle (fixed noon lighting)
- No weather effects

**Medium:**
- Basic shadows (low resolution shadow map)
- Road markings (center lines only)
- Trees visible but no street lamps or parked cars
- Citizens render as simple meshes (cube/capsule)
- Full chunk resolution
- Day/night cycle
- Basic weather (rain particles)

**High:**
- Full shadows
- Full road markings (center lines, edge lines, crosswalks)
- All props visible
- Citizens render as GLTF models
- Full terrain detail
- Day/night cycle with color grading
- Full weather effects
- Status icons over buildings

**Ultra:**
- Everything in High plus:
- Anti-aliasing (MSAA or TAA)
- Ambient occlusion
- Higher shadow resolution
- Extended prop render distance
- Extended citizen render distance
- Reflections on water

### 17.4 Display Options

**Resolution:** List of common resolutions, plus "native" option. For retina/HiDPI displays, show the physical resolution and the logical resolution.

**Display mode:**
- Fullscreen (exclusive)
- Borderless windowed (fullscreen but not exclusive -- allows Alt+Tab)
- Windowed (resizable)

**VSync:**
- On: caps at monitor refresh rate, eliminates tearing
- Off: uncapped, may tear but lower latency
- Adaptive: VSync on when above refresh rate, off when below (reduces stutter)

**Multi-monitor:** Support spanning across multiple monitors for panoramic city views. This requires careful UI positioning -- HUD elements should appear on the primary monitor, not stretched across the seam.

### 17.5 Save/Load Progress

Saving and loading large cities (1M+ citizens, complex road networks) can take several seconds. During this time:

1. **Show progress indicator:** "Saving... 45%" or "Loading... 78%"
2. **Do not freeze the screen:** Continue rendering the current frame (or show a static screenshot) while the save/load happens on a background thread
3. **Prevent input:** Disable all mouse/keyboard input during save/load to prevent race conditions
4. **Post-save confirmation:** Show "Game saved successfully" status message for 3 seconds

The current save system uses `SaveGameEvent` and `LoadGameEvent`. These should trigger the progress indicator:

```rust
fn handle_save_progress(
    mut save_state: ResMut<SaveState>,
    mut status: ResMut<StatusMessage>,
) {
    match save_state.phase {
        SavePhase::SerializingGrid => status.set("Saving world grid...", false),
        SavePhase::SerializingCitizens => status.set("Saving citizens...", false),
        SavePhase::WritingFile => status.set("Writing to disk...", false),
        SavePhase::Complete => status.set("Game saved!", false),
    }
}
```

---

## 18. Reference Games UX Analysis

### 18.1 Cities: Skylines 1 (2015)

**Camera:**
- Perspective projection with approximately 60-degree default pitch
- Free rotation via middle-mouse drag
- Smooth zoom with exponential scaling
- Zoom range allows street-level to full-city overview
- Pan via WASD, middle-click drag, or edge scrolling
- No first-person mode in base game (added by mods)

**Strengths:**
- Camera feel is excellent -- smooth, responsive, predictable
- Zoom-to-cursor works perfectly
- Edge scrolling is well-calibrated
- Multiple pan input methods

**Weaknesses:**
- Underground toggle is abrupt (instant switch between surface and underground view)
- No camera follow mode for citizens (mod-only)
- Edge scrolling cannot be disabled (frustrating in windowed mode)
- Camera sometimes clips through terrain at extreme zoom/pitch combinations

**Lessons for Megacity:**
- The camera feel should be the benchmark -- smooth, responsive, zero lag
- Underground view needs a gradual transition, not a hard toggle
- Edge scrolling must be configurable (on/off, margin size)
- Camera boundary handling should prevent clipping

### 18.2 Cities: Skylines 2 (2023)

**Camera:**
- Improved perspective projection with physically-based rendering
- First-person "photo mode" -- hugely popular, drives social media engagement
- Cinematic camera mode with depth of field
- Free rotation with smoother interpolation
- Dynamic FOV that changes slightly with zoom level

**Strengths:**
- First-person mode is the single most-discussed feature
- Photo mode produces stunning screenshots
- Improved overlay visualization with transparency and animation
- Better data panel design with tabbed organization

**Weaknesses:**
- Performance is poor -- camera stutters during zoom at high detail levels
- The beautiful rendering came at the cost of frame rate, especially on console
- Loading times are very long (2-3 minutes for large cities)
- First-person mode reveals the low quality of many building facades (assets not designed for close viewing)

**Lessons for Megacity:**
- First-person mode is worth implementing but requires building assets designed for close viewing
- Performance must be maintained at all zoom levels -- a beautiful 20 FPS city builder is worse than a clean 60 FPS one
- Photo mode is a high-value feature for marketing (player-generated content drives discovery)
- Do not sacrifice simulation performance for rendering quality

### 18.3 SimCity 4 (2003)

**Camera:**
- Orthographic-like perspective with 90-degree snap rotation (north/south/east/west only)
- Three zoom levels (discrete, not continuous) with smooth transition
- No free rotation

**Strengths:**
- Data visualization is arguably the best in the genre -- the "data views" (overlay maps) are clean, readable, and comprehensive
- The 90-degree rotation constraint makes grid alignment trivially clear
- Query tool shows excellent information density on click
- The "My Sim" feature (individual citizen simulation with narrative) was ahead of its time

**Weaknesses:**
- The camera feels restrictive by modern standards
- Only three zoom levels (now we expect continuous zoom)
- No street-level view
- Dated UI design (but the underlying information architecture is excellent)

**Lessons for Megacity:**
- SimCity 4's data views should be studied as a model for overlay design
- The query tool's information density (showing multiple data points in a compact panel) is the right approach
- Individual citizen tracking ("My Sim") is engaging and should be implemented
- 90-degree rotation is clean but feels outdated -- keep free rotation with snap shortcuts

### 18.4 Anno 1800 (2019)

**Camera:**
- Perspective projection with variable pitch (similar to CS1)
- Free rotation
- Smooth zoom with excellent feel
- Camera travels smoothly to selected production buildings
- Seamless transition between island view and close-up

**Strengths:**
- Production chain visualization is the gold standard for showing complex systems accessibly
- The "info mode" highlights buildings with color-coded icons showing their status
- Camera transitions when clicking on buildings are smooth and oriented (the camera moves to show the building from a good angle, not just centering it)
- UI panels are elegant with good information hierarchy

**Weaknesses:**
- Can be overwhelming for new players (too many icons, overlays, and panels)
- No underground view (but underground infrastructure is not part of the game)
- Limited keyboard shortcuts compared to CS1

**Lessons for Megacity:**
- Study Anno's production chain visualization for any supply-chain features
- Camera transitions to selected buildings should orient the view to show the building well
- Information hierarchy (what to show first) is critical for complex data
- Status icons over buildings (already partially implemented) should be clear and non-overlapping

### 18.5 Factorio (2020)

**Camera:**
- Pure orthographic top-down
- Free zoom from map overview to individual-machine level
- No rotation (map is always north-up)
- Massive zoom range (world is virtually infinite)

**Strengths:**
- The gold standard for information density without overwhelming the player
- Progressive disclosure is masterful: at each zoom level, exactly the right information is shown
- Tooltip system is the best in any game: hover shows every relevant statistic
- The "alt mode" (showing item flows on belts) is the perfect example of a toggleable overlay
- Zero lag -- the UI is always responsive regardless of factory size

**Weaknesses:**
- Not applicable to 3D city builders (2D top-down is a different paradigm)
- No sense of "place" -- the city/factory is a schematic, not a world

**Lessons for Megacity:**
- Progressive disclosure by zoom level: at each zoom, show only the information that's relevant
- Tooltips should be immediate, comprehensive, and never require a click to see basic info
- "Alt mode" (toggling additional information on/off) is better than always-on overlay modes
- UI responsiveness is non-negotiable -- if the UI lags, the player feels the game is broken

### 18.6 Dwarf Fortress: Steam Edition (2022)

**Camera:**
- Tilemap with 3D visualization
- Discrete zoom levels (scaling the tilemap)
- Layer-by-layer vertical navigation (Z-levels)
- Keyboard-driven with mouse supplement

**Strengths:**
- Demonstrates that extremely complex data CAN be presented accessibly
- The Steam edition took 20 years of ASCII-only UI and made it mouse-friendly without dumbing down the complexity
- Information panels are thorough but organized with clear categories
- Managed to make a keyboard-centric game work with mouse controls without compromising either

**Weaknesses:**
- Performance issues with large fortresses
- The learning curve, while improved from ASCII, is still very steep

**Lessons for Megacity:**
- Complexity is not the enemy of accessibility -- poor organization is
- A game can be deep AND approachable if information is structured well
- Transitioning from one input paradigm to another (keyboard to mouse) requires careful design but is achievable
- Persistent legends and contextual help make complex systems learnable

### 18.7 Comparative UX Feature Matrix

| Feature | CS1 | CS2 | SC4 | Anno | Factorio | DF:Steam | Megacity (Current) | Megacity (Target) |
|---|---|---|---|---|---|---|---|---|
| Free rotation | Yes | Yes | No | Yes | No | No | Yes | Yes |
| Continuous zoom | Yes | Yes | No | Yes | Yes | Partial | Yes | Yes |
| Street-level view | Mod | Yes | No | No | N/A | No | Partial | Yes |
| First-person | Mod | Yes | No | No | N/A | No | No | Yes |
| Zoom-to-cursor | Yes | Yes | N/A | Yes | Yes | N/A | No | Yes |
| Edge scrolling | Yes | Yes | Yes | Yes | No | No | No | Yes |
| Camera follow | Mod | Yes | No | No | Yes | No | No | Yes |
| Camera bookmarks | Yes | Yes | No | No | No | No | No | Yes |
| Data overlays | Good | Great | Best | Good | Alt-mode | Complex | Good | Great |
| Undo/redo | No | Yes | No | No | Yes | Yes | No | Yes |
| Controller support | Yes | Yes | No | No | No | Yes | No | Yes |
| Colorblind mode | Mod | Yes | No | No | Yes | No | No | Yes |
| Photo mode | Mod | Yes | No | No | No | No | No | Yes |
| Minimap | Yes | Yes | Yes | Yes | Map | Yes | No | Yes |
| Tooltip on hover | Partial | Yes | Yes | Yes | Best | Yes | No | Yes |
| Context menu | No | Partial | No | Yes | No | Yes | No | Yes |

---

## Appendix A: Implementation Priority

Based on impact, complexity, and dependencies, here is the recommended implementation order:

### Phase 1: Camera Polish (High Impact, Low Complexity)
1. Zoom-to-cursor (section 2.2)
2. Q/E rotation with smooth transition (section 3.1)
3. Edge scrolling (section 4.1)
4. Camera smoothing for keyboard-driven movement (section 1.2)
5. Dynamic pan margin based on zoom (section 5.1)

### Phase 2: Information Display (High Impact, Medium Complexity)
6. RCI demand bars in HUD (section 13.1)
7. Tooltip on hover (section 13.3)
8. Overlay legend (section 9.6)
9. Income/expense rate display (section 13.1)
10. Minimap (section 13.4)

### Phase 3: Selection and Interaction (High Impact, Medium Complexity)
11. Multi-entity selection (road, citizen, empty cell) (section 10.1)
12. Selection visual feedback with outline glow (section 10.2)
13. Right-click context menu (section 10.4)
14. Box selection (section 10.3)

### Phase 4: Tool UX Enhancement (Medium Impact, Medium Complexity)
15. Road cost preview during drawing (section 12.7)
16. Service building coverage radius preview (section 11.3)
17. Road snapping improvements (section 12.6)
18. Undo/redo system (section 11.4)

### Phase 5: LOD and Performance (High Impact, High Complexity)
19. Zoom-based prop culling (section 8.4)
20. Building LOD for satellite zoom (section 8.2)
21. Adaptive quality system (section 17.2)
22. Graphics quality presets (section 17.3)

### Phase 6: Camera Features (Medium Impact, Medium Complexity)
23. Camera follow mode (orbit follow) (section 6.1)
24. Camera bookmarks (section 6.2)
25. Jump-to-location from notifications (section 6.3)
26. First-person follow mode (section 6.1)

### Phase 7: Accessibility (Medium Impact, Low-Medium Complexity)
27. Colorblind modes for overlays and zones (section 16.1)
28. Adjustable text size (section 16.3)
29. High contrast mode (section 16.3)
30. Reduced motion option (section 16.4)
31. Customizable keybindings (section 14.4)

### Phase 8: Advanced Features (Low Impact, High Complexity)
32. Controller support (section 15)
33. Photo mode (section 7.1)
34. Cinematic camera paths (section 7.2)
35. Multiple overlay blending (section 9.7)
36. Flow visualization (animated traffic arrows) (section 9.4)

---

## Appendix B: Bevy-Specific Implementation Notes

### B.1 System Ordering

Camera systems must run in a specific order:

```rust
app.add_systems(Update, (
    // 1. Read input (keyboard, mouse, controller)
    camera_pan_keyboard,
    camera_pan_drag,
    camera_left_drag,
    camera_orbit_drag,
    camera_zoom,
    camera_edge_scroll,
    // 2. Apply smoothing and constraints
    apply_camera_smoothing,
    clamp_camera_bounds,
    // 3. Convert orbit state to transform
    apply_orbit_camera,
    // 4. Update viewport bounds for LOD
    update_viewport_bounds,
    // 5. Assign LOD tiers based on new viewport
    assign_lod_tiers,
).chain());
```

The `.chain()` ensures these run in order. Without chaining, Bevy may execute them in any order within the same system set, causing one-frame-delayed LOD updates or camera jitter.

### B.2 Resource vs. Component for Camera State

The current `OrbitCamera` is a `Resource`, which is correct -- there is exactly one camera in a city builder (ignoring the minimap camera, which is a separate concern). Using a resource avoids the query overhead of filtering for a specific camera entity.

However, the camera transform must be applied to a `Camera3d` entity, so `apply_orbit_camera` queries for `Transform` on that entity. This is a minor inefficiency but is standard Bevy practice.

### B.3 Input Consumption

When the egui UI consumes mouse input (clicking on a toolbar button), the camera and tool systems should not also process that input. The current implementation does not explicitly handle this, which means clicking on a UI button might also place a road on the map behind the button.

Bevy's `bevy_egui` integration provides `EguiContexts::ctx().is_pointer_over_area()` to check if the mouse is over UI. Use this to gate camera and tool input:

```rust
fn camera_zoom(
    mut scroll_evts: EventReader<MouseWheel>,
    mut orbit: ResMut<OrbitCamera>,
    contexts: EguiContexts,
) {
    if contexts.ctx().is_pointer_over_area() {
        scroll_evts.clear();  // consume events so they don't pile up
        return;
    }
    // ... existing zoom logic
}
```

This check should be added to ALL input-handling systems: `camera_pan_drag`, `camera_left_drag`, `camera_orbit_drag`, `camera_zoom`, `handle_tool_input`, and `handle_tree_tool`.

### B.4 Camera Animation State Machine

If multiple camera features can request camera movement (follow mode, jump-to-location, bookmark recall, smooth pan), a state machine prevents conflicts:

```rust
enum CameraMode {
    /// Player has full manual control
    Free,
    /// Camera is following an entity
    Following(Entity),
    /// Camera is transitioning to a target position
    Transitioning { target: OrbitState, elapsed: f32, duration: f32 },
    /// Camera is on a cinematic path
    Cinematic { path: CameraPath, time: f32 },
}
```

In `Free` mode, all manual input systems operate normally. In `Following` mode, manual pan is disabled but zoom and orbit are allowed. In `Transitioning` mode, all manual input is ignored until the transition completes (or the player presses any key to cancel). In `Cinematic` mode, all input is ignored except Escape to cancel.

### B.5 Performance Considerations

**Camera system performance:** The camera systems run every frame and are very lightweight (simple arithmetic, no queries). Total camera system time should be under 0.01ms per frame.

**Viewport bounds calculation:** `update_viewport_bounds` performs 4 ray-plane intersections per frame. This is negligible (~0.001ms).

**LOD tier assignment:** `assign_lod_tiers` iterates over ALL citizen entities every frame to compare positions against viewport bounds. With 1M citizens, this is the most expensive viewport-related system. Consider running it at lower frequency (every 4-8 frames) or only when the camera has moved significantly:

```rust
fn assign_lod_tiers(
    // ... existing params ...
    orbit: Res<OrbitCamera>,
    mut last_orbit: Local<(Vec3, f32)>,
) {
    // Skip if camera hasn't moved significantly
    let moved = (orbit.focus - last_orbit.0).length_squared() > 100.0
        || (orbit.distance - last_orbit.1).abs() > 50.0;
    if !moved { return; }

    *last_orbit = (orbit.focus, orbit.distance);
    // ... existing logic
}
```

---

## Appendix C: Mathematical Reference

### C.1 Exponential Smoothing

Frame-rate-independent exponential smoothing:

```
value = value + (target - value) * (1 - e^(-speed * dt))
```

Where:
- `value`: current value
- `target`: target value
- `speed`: smoothing rate (higher = faster convergence)
- `dt`: frame delta time in seconds
- `e`: Euler's number (2.71828...)

At `speed = 10.0`:
- After 0.1s (6 frames at 60fps): 63% of the way to target
- After 0.2s (12 frames): 86%
- After 0.3s (18 frames): 95%
- After 0.5s (30 frames): 99.3%

### C.2 Orbital Camera Spherical Coordinates

Converting from orbit parameters to Cartesian position:

```
x = distance * cos(pitch) * sin(yaw)
y = distance * sin(pitch)
z = distance * cos(pitch) * cos(yaw)
camera_position = focus_point + (x, y, z)
```

Where:
- `pitch`: elevation angle from ground plane (0 = horizontal, PI/2 = vertical)
- `yaw`: horizontal rotation around Y axis (0 = looking along +Z)
- `distance`: distance from focus point

### C.3 Zoom-to-Cursor Math

Given:
- `cursor_screen`: 2D pixel position of cursor on screen
- `focus_before`: 3D focus point before zoom
- `distance_before`: camera distance before zoom
- `distance_after`: camera distance after zoom

1. Compute ground point under cursor before zoom:
```
ray = camera.viewport_to_world(cursor_screen)
t = -ray.origin.y / ray.direction.y
ground_before = ray.origin + ray.direction * t
```

2. Apply zoom: `distance = distance_after`

3. Compute ground point under cursor after zoom:
```
// Recompute ray with new camera position
ray_after = camera.viewport_to_world(cursor_screen)
t_after = -ray_after.origin.y / ray_after.direction.y
ground_after = ray_after.origin + ray_after.direction * t_after
```

4. Adjust focus to keep cursor point fixed:
```
focus = focus + (ground_before - ground_after)
```

### C.4 Hysteresis Band for LOD Transitions

To prevent flickering between LOD levels, use different thresholds for upgrading vs. downgrading:

```
upgrade_threshold = base_threshold - hysteresis / 2
downgrade_threshold = base_threshold + hysteresis / 2

if current_tier == Low && distance < upgrade_threshold:
    current_tier = High

if current_tier == High && distance > downgrade_threshold:
    current_tier = Low
```

With `base_threshold = 600` and `hysteresis = 100`:
- Upgrade from Simplified to Full when distance < 550
- Downgrade from Full to Simplified when distance > 650
- Between 550 and 650: stay at whatever tier you're currently at

This 100-unit dead zone eliminates all flickering from camera jitter or smooth zoom oscillation.
