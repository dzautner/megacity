// =============================================================================
// Tel Aviv road network: Bezier road segments for the city map.
// =============================================================================

use bevy::math::Vec2;

use crate::grid::{RoadType, WorldGrid};
use crate::road_segments::RoadSegmentStore;
use crate::roads::RoadNetwork;

use super::coastline_x;

// =============================================================================
// Road helpers
// =============================================================================

/// Add a straight Bezier road between two grid positions.
#[allow(clippy::too_many_arguments)]
fn road_straight(
    seg: &mut RoadSegmentStore,
    grid: &mut WorldGrid,
    roads: &mut RoadNetwork,
    gx0: usize,
    gy0: usize,
    gx1: usize,
    gy1: usize,
    rt: RoadType,
) {
    let (wx0, wy0) = WorldGrid::grid_to_world(gx0, gy0);
    let (wx1, wy1) = WorldGrid::grid_to_world(gx1, gy1);
    seg.add_straight_segment(
        Vec2::new(wx0, wy0),
        Vec2::new(wx1, wy1),
        rt,
        16.0,
        grid,
        roads,
    );
}

/// Add a curved Bezier road with explicit control points (world coords).
#[allow(clippy::too_many_arguments)]
fn road_curve(
    seg: &mut RoadSegmentStore,
    grid: &mut WorldGrid,
    roads: &mut RoadNetwork,
    from: Vec2,
    c1: Vec2,
    c2: Vec2,
    to: Vec2,
    rt: RoadType,
) {
    let start = seg.find_or_create_node(from, 16.0);
    let end = seg.find_or_create_node(to, 16.0);
    seg.add_segment(start, end, from, c1, c2, to, rt, grid, roads);
}

/// Convert grid coords to world Vec2 (center of cell).
fn gw(gx: usize, gy: usize) -> Vec2 {
    let (wx, wy) = WorldGrid::grid_to_world(gx, gy);
    Vec2::new(wx, wy)
}

// =============================================================================
// Tel Aviv road network
// =============================================================================

pub fn build_tel_aviv_roads(
    seg: &mut RoadSegmentStore,
    grid: &mut WorldGrid,
    roads: &mut RoadNetwork,
) {
    // --- 1. Jaffa old city: winding local roads near the coast (SW) ---
    // Yefet Street: main road through Jaffa, slightly curving
    let jaffa_n = gw(62, 65);
    let jaffa_s = gw(55, 35);
    let jaffa_mid = gw(58, 50);
    road_curve(
        seg,
        grid,
        roads,
        jaffa_s,
        jaffa_s + Vec2::new(30.0, 80.0),
        jaffa_mid + Vec2::new(-10.0, 80.0),
        jaffa_n,
        RoadType::Local,
    );
    // Jaffa side streets
    road_straight(seg, grid, roads, 55, 42, 62, 42, RoadType::Local);
    road_straight(seg, grid, roads, 53, 50, 62, 50, RoadType::Local);
    road_straight(seg, grid, roads, 56, 58, 65, 58, RoadType::Local);

    // --- 2. Coastal Boulevard (Herbert Samuel -> HaYarkon) ---
    // Runs north along the coast from Jaffa to the Yarkon river
    road_straight(seg, grid, roads, 63, 65, 63, 90, RoadType::Boulevard);
    road_straight(seg, grid, roads, 63, 90, 62, 120, RoadType::Boulevard);
    road_straight(seg, grid, roads, 62, 120, 62, 150, RoadType::Boulevard);
    road_straight(seg, grid, roads, 62, 150, 63, 180, RoadType::Boulevard);

    // --- 3. Allenby Street: coast to city center (NW to SE diagonal) ---
    let allenby_coast = gw(65, 82);
    let allenby_mid = gw(95, 88);
    let allenby_end = gw(140, 92);
    road_curve(
        seg,
        grid,
        roads,
        allenby_coast,
        allenby_coast + Vec2::new(200.0, 20.0),
        allenby_mid + Vec2::new(200.0, 30.0),
        allenby_end,
        RoadType::Avenue,
    );

    // --- 4. Rothschild Boulevard: the iconic tree-lined boulevard ---
    let roth_start = gw(78, 72);
    let roth_mid = gw(95, 88);
    let roth_end = gw(118, 108);
    road_curve(
        seg,
        grid,
        roads,
        roth_start,
        roth_start + Vec2::new(150.0, 100.0),
        roth_mid + Vec2::new(100.0, 100.0),
        roth_end,
        RoadType::Boulevard,
    );

    // --- 5. Dizengoff Street (N-S avenue through the White City) ---
    road_straight(seg, grid, roads, 102, 75, 102, 105, RoadType::Avenue);
    road_straight(seg, grid, roads, 102, 105, 102, 135, RoadType::Avenue);
    road_straight(seg, grid, roads, 102, 135, 102, 170, RoadType::Avenue);

    // --- 6. Ibn Gabirol Street (N-S avenue, east of Dizengoff) ---
    road_straight(seg, grid, roads, 125, 75, 125, 105, RoadType::Avenue);
    road_straight(seg, grid, roads, 125, 105, 125, 135, RoadType::Avenue);
    road_straight(seg, grid, roads, 125, 135, 125, 170, RoadType::Avenue);

    // --- 7. King George Street (E-W) ---
    road_straight(seg, grid, roads, 80, 120, 110, 120, RoadType::Avenue);
    road_straight(seg, grid, roads, 110, 120, 145, 120, RoadType::Avenue);

    // --- 8. Ben Gurion Boulevard (E-W, from coast to center) ---
    road_straight(seg, grid, roads, 63, 105, 95, 105, RoadType::Boulevard);
    road_straight(seg, grid, roads, 95, 105, 125, 105, RoadType::Boulevard);

    // --- 9. Arlozorov Street (E-W, major crosstown) ---
    road_straight(seg, grid, roads, 63, 155, 100, 155, RoadType::Avenue);
    road_straight(seg, grid, roads, 100, 155, 140, 155, RoadType::Avenue);
    road_straight(seg, grid, roads, 140, 155, 185, 155, RoadType::Avenue);

    // --- 10. Ayalon Highway (N-S expressway on the east) ---
    road_straight(seg, grid, roads, 185, 25, 185, 60, RoadType::Highway);
    road_straight(seg, grid, roads, 185, 60, 185, 100, RoadType::Highway);
    road_straight(seg, grid, roads, 185, 100, 185, 140, RoadType::Highway);
    road_straight(seg, grid, roads, 185, 140, 185, 180, RoadType::Highway);
    road_straight(seg, grid, roads, 185, 180, 185, 220, RoadType::Highway);

    // --- 11. Namir Road / Begin Road (N-S, center to north) ---
    road_straight(seg, grid, roads, 140, 108, 140, 140, RoadType::Boulevard);
    road_straight(seg, grid, roads, 140, 140, 140, 170, RoadType::Boulevard);

    // --- 12. Eilat Street (E-W through south) ---
    road_straight(seg, grid, roads, 62, 65, 90, 65, RoadType::Avenue);
    road_straight(seg, grid, roads, 90, 65, 130, 65, RoadType::Avenue);
    road_straight(seg, grid, roads, 130, 65, 185, 65, RoadType::Avenue);

    // --- 13. Highway on-ramps connecting Ayalon to city grid ---
    road_straight(seg, grid, roads, 145, 92, 185, 92, RoadType::Avenue);
    road_straight(seg, grid, roads, 145, 120, 185, 120, RoadType::Avenue);
    road_straight(seg, grid, roads, 145, 170, 185, 170, RoadType::Avenue);

    // --- 14. White City local grid streets (E-W, between the major avenues) ---
    // Between Eilat (y=65) and Arlozorov (y=155), every ~8 cells
    for &gy in &[75, 82, 92, 100, 112, 128, 140, 148] {
        road_straight(seg, grid, roads, 68, gy, 100, gy, RoadType::Local);
        road_straight(seg, grid, roads, 100, gy, 125, gy, RoadType::Local);
        road_straight(seg, grid, roads, 125, gy, 145, gy, RoadType::Local);
    }

    // --- 15. White City local grid streets (N-S, between the major avenues) ---
    for &gx in &[75, 82, 90, 110, 118, 132, 138] {
        road_straight(seg, grid, roads, gx, 68, gx, 95, RoadType::Local);
        road_straight(seg, grid, roads, gx, 95, gx, 120, RoadType::Local);
        road_straight(seg, grid, roads, gx, 120, gx, 150, RoadType::Local);
    }

    // --- 16. Ramat Aviv (north of Yarkon River, wider spacing) ---
    road_straight(seg, grid, roads, 75, 192, 75, 240, RoadType::Local);
    road_straight(seg, grid, roads, 100, 192, 100, 240, RoadType::Avenue);
    road_straight(seg, grid, roads, 125, 192, 125, 240, RoadType::Avenue);
    road_straight(seg, grid, roads, 150, 192, 150, 240, RoadType::Local);
    road_straight(seg, grid, roads, 75, 200, 150, 200, RoadType::Local);
    road_straight(seg, grid, roads, 75, 215, 150, 215, RoadType::Avenue);
    road_straight(seg, grid, roads, 75, 230, 150, 230, RoadType::Local);

    // Bridges over Yarkon River
    road_straight(seg, grid, roads, 100, 178, 100, 192, RoadType::Avenue);
    road_straight(seg, grid, roads, 125, 178, 125, 192, RoadType::Avenue);
    road_straight(seg, grid, roads, 140, 178, 140, 192, RoadType::Boulevard);

    // --- 17. Eastern areas (between city grid and Ayalon) ---
    for &gy in &[75, 92, 112, 135, 148] {
        road_straight(seg, grid, roads, 145, gy, 180, gy, RoadType::Local);
    }
    for &gx in &[155, 168] {
        road_straight(seg, grid, roads, gx, 68, gx, 100, RoadType::Local);
        road_straight(seg, grid, roads, gx, 100, gx, 150, RoadType::Local);
    }

    // --- 18. Waterfront promenade (path along the beach) ---
    for &(gy0, gy1) in &[(35, 65), (65, 90), (90, 120), (120, 150), (150, 180)] {
        let coast_x0 = (coastline_x(gy0 as f32) + 2.0) as usize;
        let coast_x1 = (coastline_x(gy1 as f32) + 2.0) as usize;
        road_straight(
            seg,
            grid,
            roads,
            coast_x0,
            gy0,
            coast_x1,
            gy1,
            RoadType::Path,
        );
    }
}
