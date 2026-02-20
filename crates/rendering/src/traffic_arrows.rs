//! Traffic Flow Arrows Overlay
//!
//! When the traffic overlay is active, draws animated arrow glyphs on road
//! segments using Bevy gizmos. Arrow direction follows traffic flow along the
//! road segment tangent. Arrow color ramps from green (free flow) to red
//! (congested), and opacity is proportional to traffic volume. Arrows are
//! LOD'd based on camera distance for performance.

use bevy::prelude::*;

use simulation::road_segments::RoadSegmentStore;
use simulation::traffic::TrafficGrid;

use crate::camera::OrbitCamera;
use crate::overlay::{OverlayMode, OverlayState};

/// Maximum camera distance at which traffic arrows are drawn.
/// Beyond this distance, arrows are culled for performance.
const MAX_ARROW_DISTANCE: f32 = 800.0;

/// Spacing between arrows along a segment (in world units).
const ARROW_SPACING: f32 = 24.0;

/// Height above ground for arrow rendering.
const ARROW_Y: f32 = 0.2;

/// Speed of arrow animation (world units per second).
const ARROW_ANIM_SPEED: f32 = 12.0;

/// Minimum traffic density to show arrows (avoids clutter on empty roads).
const MIN_DENSITY_THRESHOLD: u16 = 1;

/// Returns a color from green (free flow) to red (congested) based on
/// congestion level (0.0 = free, 1.0 = fully congested).
fn congestion_color(congestion: f32, opacity: f32) -> Color {
    // Green -> Yellow -> Red ramp
    let r = (congestion * 2.0).min(1.0);
    let g = ((1.0 - congestion) * 2.0).min(1.0);
    Color::srgba(r, g, 0.1, opacity)
}

/// Compute the average traffic density along a road segment by sampling
/// its rasterized cells.
fn segment_traffic_info(
    segment: &simulation::road_segments::RoadSegment,
    traffic: &TrafficGrid,
) -> (f32, u16) {
    if segment.rasterized_cells.is_empty() {
        return (0.0, 0);
    }

    let mut total_density: u32 = 0;
    let mut max_density: u16 = 0;
    let mut count = 0u32;

    for &(cx, cy) in &segment.rasterized_cells {
        if cx < traffic.width && cy < traffic.height {
            let d = traffic.get(cx, cy);
            total_density += d as u32;
            if d > max_density {
                max_density = d;
            }
            count += 1;
        }
    }

    if count == 0 {
        return (0.0, 0);
    }

    let avg_density = total_density as f32 / count as f32;
    let congestion = (avg_density / 20.0).min(1.0);
    (congestion, max_density)
}

/// Returns the visual half-width of a road type (matching road_render.rs values).
fn road_half_width(rt: simulation::grid::RoadType) -> f32 {
    use simulation::grid::RoadType;
    match rt {
        RoadType::Path => 1.5,
        RoadType::OneWay => 3.0,
        RoadType::Local => 4.0,
        RoadType::Avenue => 6.0,
        RoadType::Boulevard => 8.0,
        RoadType::Highway => 10.0,
    }
}

/// System that draws animated traffic flow arrows on road segments when the
/// traffic overlay is active.
#[allow(clippy::too_many_arguments)]
pub fn draw_traffic_flow_arrows(
    overlay: Res<OverlayState>,
    store: Res<RoadSegmentStore>,
    traffic: Res<TrafficGrid>,
    orbit: Res<OrbitCamera>,
    time: Res<Time>,
    mut gizmos: Gizmos,
) {
    // Only draw when traffic overlay is active
    if overlay.mode != OverlayMode::Traffic {
        return;
    }

    // LOD: skip drawing when camera is too far away
    if orbit.distance > MAX_ARROW_DISTANCE {
        return;
    }

    // Fade arrows out as camera gets further away
    let distance_fade = 1.0
        - ((orbit.distance - MAX_ARROW_DISTANCE * 0.5) / (MAX_ARROW_DISTANCE * 0.5))
            .clamp(0.0, 1.0);

    let elapsed = time.elapsed_secs();

    // Camera focus for frustum-like culling (only draw near visible area)
    let cam_focus_x = orbit.focus.x;
    let cam_focus_z = orbit.focus.z;
    // Visible radius increases with camera distance
    let visible_radius = orbit.distance * 1.5;

    for segment in &store.segments {
        // Quick distance check: skip segments far from camera focus
        let seg_mid = segment.evaluate(0.5);
        let dx = seg_mid.x - cam_focus_x;
        let dz = seg_mid.y - cam_focus_z;
        if dx * dx + dz * dz > visible_radius * visible_radius {
            continue;
        }

        // Get traffic info for this segment
        let (congestion, max_density) = segment_traffic_info(segment, &traffic);

        // Skip segments with no meaningful traffic
        if max_density < MIN_DENSITY_THRESHOLD {
            continue;
        }

        // Opacity proportional to traffic volume (more traffic = more visible)
        let volume_opacity = (max_density as f32 / 15.0).clamp(0.15, 0.9);
        let final_opacity = volume_opacity * distance_fade;

        if final_opacity < 0.02 {
            continue;
        }

        let color = congestion_color(congestion, final_opacity);

        // Number of arrows along the segment
        let arrow_count = (segment.arc_length / ARROW_SPACING).ceil().max(1.0) as usize;

        // Animation offset: arrows slide along the segment over time
        let anim_offset = if segment.arc_length > 0.0 {
            (elapsed * ARROW_ANIM_SPEED / ARROW_SPACING) % 1.0
        } else {
            0.0
        };

        for i in 0..arrow_count {
            // Parameter t with animation offset
            let base_t = if arrow_count == 1 {
                0.5
            } else {
                (i as f32 + anim_offset) / arrow_count as f32
            };

            // Wrap t to [0, 1]
            let t = base_t % 1.0;

            let center = segment.evaluate(t);
            let tangent = segment.tangent(t).normalize_or_zero();

            // Skip if tangent is degenerate
            if tangent.length_squared() < 0.01 {
                continue;
            }

            // Arrow dimensions scale slightly with road type width
            let road_hw = road_half_width(segment.road_type);
            let arrow_scale = (road_hw / 10.0).clamp(0.5, 1.5);

            let arrow_len = 3.5 * arrow_scale;
            let arrow_head_len = 2.0 * arrow_scale;
            let arrow_half_width = 1.5 * arrow_scale;

            let dir = tangent;
            let tip = center + dir * arrow_len * 0.5;
            let base = center - dir * arrow_len * 0.5;
            let head_base = tip - dir * arrow_head_len;

            // Perpendicular for arrow head wings
            let perp = Vec2::new(-dir.y, dir.x);
            let wing_l = head_base + perp * arrow_half_width;
            let wing_r = head_base - perp * arrow_half_width;

            // Draw arrow shaft
            gizmos.line(
                Vec3::new(base.x, ARROW_Y, base.y),
                Vec3::new(head_base.x, ARROW_Y, head_base.y),
                color,
            );

            // Draw arrow head (two lines from wings to tip)
            gizmos.line(
                Vec3::new(wing_l.x, ARROW_Y, wing_l.y),
                Vec3::new(tip.x, ARROW_Y, tip.y),
                color,
            );
            gizmos.line(
                Vec3::new(wing_r.x, ARROW_Y, wing_r.y),
                Vec3::new(tip.x, ARROW_Y, tip.y),
                color,
            );
        }
    }
}

pub struct TrafficArrowsPlugin;

impl Plugin for TrafficArrowsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_traffic_flow_arrows);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_congestion_color_green_at_zero() {
        let c = congestion_color(0.0, 1.0).to_srgba();
        // At zero congestion, should be greenish (G > R)
        assert!(
            c.green > c.red,
            "Free flow should be green: r={} g={}",
            c.red,
            c.green
        );
    }

    #[test]
    fn test_congestion_color_red_at_max() {
        let c = congestion_color(1.0, 1.0).to_srgba();
        // At full congestion, should be reddish (R > G)
        assert!(
            c.red > c.green,
            "Full congestion should be red: r={} g={}",
            c.red,
            c.green
        );
    }

    #[test]
    fn test_congestion_color_opacity() {
        let c1 = congestion_color(0.5, 0.3).to_srgba();
        let c2 = congestion_color(0.5, 0.9).to_srgba();
        assert!(
            (c1.alpha - 0.3).abs() < 0.01,
            "Opacity should be 0.3: {}",
            c1.alpha
        );
        assert!(
            (c2.alpha - 0.9).abs() < 0.01,
            "Opacity should be 0.9: {}",
            c2.alpha
        );
    }

    #[test]
    fn test_congestion_color_yellow_at_midpoint() {
        let c = congestion_color(0.5, 1.0).to_srgba();
        // At midpoint, both red and green should be significant
        assert!(c.red > 0.3, "Mid congestion should have red: r={}", c.red);
        assert!(
            c.green > 0.3,
            "Mid congestion should have green: g={}",
            c.green
        );
    }

    #[test]
    fn test_segment_traffic_info_empty_cells() {
        use simulation::grid::RoadType;
        use simulation::road_segments::{RoadSegment, SegmentId, SegmentNodeId};

        let segment = RoadSegment {
            id: SegmentId(0),
            start_node: SegmentNodeId(0),
            end_node: SegmentNodeId(1),
            p0: Vec2::ZERO,
            p1: Vec2::new(10.0, 0.0),
            p2: Vec2::new(20.0, 0.0),
            p3: Vec2::new(30.0, 0.0),
            road_type: RoadType::Local,
            arc_length: 30.0,
            rasterized_cells: vec![],
        };

        let traffic = TrafficGrid::default();
        let (congestion, max_density) = segment_traffic_info(&segment, &traffic);
        assert_eq!(congestion, 0.0);
        assert_eq!(max_density, 0);
    }

    #[test]
    fn test_segment_traffic_info_with_density() {
        use simulation::grid::RoadType;
        use simulation::road_segments::{RoadSegment, SegmentId, SegmentNodeId};

        let segment = RoadSegment {
            id: SegmentId(0),
            start_node: SegmentNodeId(0),
            end_node: SegmentNodeId(1),
            p0: Vec2::ZERO,
            p1: Vec2::new(10.0, 0.0),
            p2: Vec2::new(20.0, 0.0),
            p3: Vec2::new(30.0, 0.0),
            road_type: RoadType::Local,
            arc_length: 30.0,
            rasterized_cells: vec![(5, 5), (6, 5), (7, 5)],
        };

        let mut traffic = TrafficGrid::default();
        traffic.set(5, 5, 10);
        traffic.set(6, 5, 20);
        traffic.set(7, 5, 5);

        let (congestion, max_density) = segment_traffic_info(&segment, &traffic);
        assert_eq!(max_density, 20);
        // Average density = (10 + 20 + 5) / 3 = 11.67, congestion = 11.67/20 = 0.583
        assert!(
            congestion > 0.5 && congestion < 0.7,
            "congestion={}",
            congestion
        );
    }
}
