use bevy::prelude::*;

use simulation::oneway::{OneWayDirection, OneWayDirectionMap};
use simulation::road_segments::RoadSegmentStore;

/// Draw direction arrows on one-way road segments using gizmos.
///
/// Arrows are drawn along the segment at regular intervals, pointing
/// in the allowed traffic direction.
pub fn draw_oneway_arrows(
    store: Res<RoadSegmentStore>,
    oneway_map: Res<OneWayDirectionMap>,
    mut gizmos: Gizmos,
) {
    if oneway_map.directions.is_empty() {
        return;
    }

    let arrow_color = Color::srgba(0.1, 0.85, 0.3, 0.8);
    let arrow_y = 0.15; // slightly above road surface

    for segment in &store.segments {
        let Some(direction) = oneway_map.get(segment.id) else {
            continue;
        };

        // Place arrows every ~20 world units along the segment
        let arrow_spacing = 20.0_f32;
        let arrow_count = (segment.arc_length / arrow_spacing).ceil().max(1.0) as usize;

        for i in 0..arrow_count {
            // Parameter t for arrow center position
            let t = if arrow_count == 1 {
                0.5
            } else {
                (i as f32 + 0.5) / arrow_count as f32
            };

            let center = segment.evaluate(t);
            let tangent = segment.tangent(t).normalize_or_zero();

            // Flip tangent for reverse direction
            let dir = match direction {
                OneWayDirection::Forward => tangent,
                OneWayDirection::Reverse => -tangent,
            };

            // Arrow dimensions
            let arrow_len = 3.0_f32;
            let arrow_head_len = 1.5_f32;
            let arrow_half_width = 1.2_f32;

            let tip = center + dir * arrow_len * 0.5;
            let base = center - dir * arrow_len * 0.5;
            let head_base = tip - dir * arrow_head_len;

            // Perpendicular for arrow head wings
            let perp = Vec2::new(-dir.y, dir.x);
            let wing_l = head_base + perp * arrow_half_width;
            let wing_r = head_base - perp * arrow_half_width;

            // Draw arrow shaft
            gizmos.line(
                Vec3::new(base.x, arrow_y, base.y),
                Vec3::new(head_base.x, arrow_y, head_base.y),
                arrow_color,
            );

            // Draw arrow head (two lines from wings to tip)
            gizmos.line(
                Vec3::new(wing_l.x, arrow_y, wing_l.y),
                Vec3::new(tip.x, arrow_y, tip.y),
                arrow_color,
            );
            gizmos.line(
                Vec3::new(wing_r.x, arrow_y, wing_r.y),
                Vec3::new(tip.x, arrow_y, tip.y),
                arrow_color,
            );
        }
    }
}
