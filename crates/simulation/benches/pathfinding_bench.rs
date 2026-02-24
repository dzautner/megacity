//! Criterion benchmarks for CSR A* pathfinding at various distances.
//!
//! Benchmarks 4 distance tiers on a grid road network (roads every 8 cells):
//!   - short_10:   ~10 cell path
//!   - medium_50:  ~50 cell path
//!   - long_200:   ~200 cell path
//!   - cross_map:  corner-to-corner (~248 cells diagonal, ~496 Manhattan)
//!
//! Budget: single A* call < 1 ms.
//!
//! Run with: cargo bench -p simulation --bench pathfinding_bench --features bench

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

use simulation::config::{GRID_HEIGHT, GRID_WIDTH};
use simulation::grid::{CellType, WorldGrid};
use simulation::road_graph_csr::{csr_find_path, csr_find_path_with_traffic, CsrGraph};
use simulation::roads::{RoadNetwork, RoadNode};
use simulation::traffic::TrafficGrid;

// ---------------------------------------------------------------------------
// Fixture: grid road network with roads every 8 cells
// ---------------------------------------------------------------------------

/// Build a grid road network with roads every `spacing` cells on a 256x256 map.
/// Returns `(WorldGrid, RoadNetwork, CsrGraph)`.
fn build_grid_fixture(spacing: usize) -> (WorldGrid, RoadNetwork, CsrGraph) {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut network = RoadNetwork::default();

    // Horizontal roads every `spacing` rows
    for y in (0..GRID_HEIGHT).step_by(spacing) {
        for x in 0..GRID_WIDTH {
            network.place_road(&mut grid, x, y);
        }
    }
    // Vertical roads every `spacing` columns
    for x in (0..GRID_WIDTH).step_by(spacing) {
        for y in 0..GRID_HEIGHT {
            if grid.get(x, y).cell_type != CellType::Road {
                network.place_road(&mut grid, x, y);
            }
        }
    }

    let csr = CsrGraph::from_road_network(&network);
    (grid, network, csr)
}

// ---------------------------------------------------------------------------
// Distance-tier endpoints on the spacing-8 grid
// ---------------------------------------------------------------------------

// All endpoints lie on the road grid (multiples of 8) so they are valid nodes.

/// Short path: ~10 cells along a single road.
/// From (0,0) to (10,0) — but 10 is not on the grid, so use (0,0) to (8,0)
/// which is 8 cells, close enough. Actually let's pick (0,0) to (0,8) for a
/// clear 8-cell hop, then for ~10 we pick (0,0) to (8,8) which is ~11
/// Manhattan via the grid.
const SHORT_START: RoadNode = RoadNode(0, 0);
const SHORT_GOAL: RoadNode = RoadNode(8, 8);

/// Medium path: ~50 cells. (0,0) to (24,24) = 48 Manhattan on the grid.
const MEDIUM_START: RoadNode = RoadNode(0, 0);
const MEDIUM_GOAL: RoadNode = RoadNode(24, 24);

/// Long path: ~200 cells. (0,0) to (96,96) = 192 Manhattan on the grid.
const LONG_START: RoadNode = RoadNode(0, 0);
const LONG_GOAL: RoadNode = RoadNode(96, 96);

/// Cross-map: corner to corner. (0,0) to (248,248) — 248 is the last
/// multiple of 8 within the 256-wide grid. Manhattan ~496 cells.
const CROSS_START: RoadNode = RoadNode(0, 0);
const CROSS_GOAL: RoadNode = RoadNode(248, 248);

// ---------------------------------------------------------------------------
// Benchmark: CSR A* at 4 distance tiers
// ---------------------------------------------------------------------------

fn bench_csr_astar_distances(c: &mut Criterion) {
    let mut group = c.benchmark_group("csr_astar_distance");
    // Use enough samples for statistical confidence (default 100 is fine
    // for sub-ms benchmarks; criterion auto-adjusts measurement time).
    group.sample_size(100);

    let (grid, _network, csr) = build_grid_fixture(8);

    // Verify endpoints exist in the CSR graph (panic early if fixture is wrong).
    for (label, start, goal) in [
        ("short_10", SHORT_START, SHORT_GOAL),
        ("medium_50", MEDIUM_START, MEDIUM_GOAL),
        ("long_200", LONG_START, LONG_GOAL),
        ("cross_map", CROSS_START, CROSS_GOAL),
    ] {
        assert!(
            csr.find_node_index(&start).is_some(),
            "{label}: start node {start:?} missing from CSR graph"
        );
        assert!(
            csr.find_node_index(&goal).is_some(),
            "{label}: goal node {goal:?} missing from CSR graph"
        );
        // Sanity: path should exist
        let path = csr_find_path(&csr, start, goal);
        assert!(
            path.is_some(),
            "{label}: no path found from {start:?} to {goal:?}"
        );
    }

    // 1. Short (~10 cells)
    group.bench_function("short_10", |b| {
        b.iter(|| black_box(csr_find_path(&csr, SHORT_START, SHORT_GOAL)));
    });

    // 2. Medium (~50 cells)
    group.bench_function("medium_50", |b| {
        b.iter(|| black_box(csr_find_path(&csr, MEDIUM_START, MEDIUM_GOAL)));
    });

    // 3. Long (~200 cells)
    group.bench_function("long_200", |b| {
        b.iter(|| black_box(csr_find_path(&csr, LONG_START, LONG_GOAL)));
    });

    // 4. Cross-map (corner to corner)
    group.bench_function("cross_map", |b| {
        b.iter(|| black_box(csr_find_path(&csr, CROSS_START, CROSS_GOAL)));
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: traffic-aware CSR A* at 4 distance tiers
// ---------------------------------------------------------------------------

fn bench_csr_astar_with_traffic(c: &mut Criterion) {
    let mut group = c.benchmark_group("csr_astar_traffic");
    group.sample_size(100);

    let (grid, _network, csr) = build_grid_fixture(8);
    let traffic = TrafficGrid::default(); // zero traffic baseline

    group.bench_function("short_10", |b| {
        b.iter(|| {
            black_box(csr_find_path_with_traffic(
                &csr,
                SHORT_START,
                SHORT_GOAL,
                &grid,
                &traffic,
            ))
        });
    });

    group.bench_function("medium_50", |b| {
        b.iter(|| {
            black_box(csr_find_path_with_traffic(
                &csr,
                MEDIUM_START,
                MEDIUM_GOAL,
                &grid,
                &traffic,
            ))
        });
    });

    group.bench_function("long_200", |b| {
        b.iter(|| {
            black_box(csr_find_path_with_traffic(
                &csr, LONG_START, LONG_GOAL, &grid, &traffic,
            ))
        });
    });

    group.bench_function("cross_map", |b| {
        b.iter(|| {
            black_box(csr_find_path_with_traffic(
                &csr,
                CROSS_START,
                CROSS_GOAL,
                &grid,
                &traffic,
            ))
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: CSR graph construction from road network
// ---------------------------------------------------------------------------

fn bench_csr_graph_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("csr_graph_build");
    group.sample_size(20);

    let (_grid, network, _csr) = build_grid_fixture(8);

    group.bench_function("from_road_network_spacing8", |b| {
        b.iter(|| black_box(CsrGraph::from_road_network(&network)));
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Register groups
// ---------------------------------------------------------------------------

criterion_group!(
    benches,
    bench_csr_astar_distances,
    bench_csr_astar_with_traffic,
    bench_csr_graph_build,
);
criterion_main!(benches);
