//! Criterion benchmarks for grid operations.
//!
//! Benchmarks:
//!   - neighbors4 at center (128,128) — 4 valid neighbors
//!   - neighbors4 at corner (0,0) — 2 valid neighbors
//!   - world_to_grid coordinate conversion
//!   - grid_to_world coordinate conversion
//!
//! Budget: all operations < 100ns.
//!
//! Run with: cargo bench -p simulation --bench grid_bench

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use simulation::config::{GRID_HEIGHT, GRID_WIDTH};
use simulation::grid::WorldGrid;

// ---------------------------------------------------------------------------
// Benchmark: neighbors4
// ---------------------------------------------------------------------------

fn bench_neighbors4(c: &mut Criterion) {
    let mut group = c.benchmark_group("grid_neighbors4");
    group.sample_size(1000);

    let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);

    // Center cell (128,128) — all 4 neighbors valid
    group.bench_function("center_128_128", |b| {
        b.iter(|| black_box(grid.neighbors4(black_box(128), black_box(128))));
    });

    // Corner cell (0,0) — only 2 neighbors valid
    group.bench_function("corner_0_0", |b| {
        b.iter(|| black_box(grid.neighbors4(black_box(0), black_box(0))));
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: world_to_grid / grid_to_world conversions
// ---------------------------------------------------------------------------

fn bench_coordinate_conversions(c: &mut Criterion) {
    let mut group = c.benchmark_group("grid_coord_conversion");
    group.sample_size(1000);

    // world_to_grid: convert a world-space position to grid coordinates
    group.bench_function("world_to_grid", |b| {
        b.iter(|| {
            black_box(WorldGrid::world_to_grid(
                black_box(1234.5),
                black_box(678.9),
            ))
        });
    });

    // grid_to_world: convert grid coordinates to world-space center
    group.bench_function("grid_to_world", |b| {
        b.iter(|| black_box(WorldGrid::grid_to_world(black_box(128), black_box(64))));
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Register groups
// ---------------------------------------------------------------------------

criterion_group!(benches, bench_neighbors4, bench_coordinate_conversions);
criterion_main!(benches);
