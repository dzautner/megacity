//! Criterion benchmarks for TrafficGrid operations.
//!
//! Benchmarks:
//!   - full_clear:          clear entire 256x256 density grid
//!   - congestion_lookup:   single-cell congestion_level() query
//!   - path_cost_with_road: path_cost_with_road() for each RoadType variant
//!
//! Budget: traffic grid update < 2 ms for 100K citizens.
//!
//! Run with: cargo bench -p simulation --bench traffic_bench

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

use simulation::config::{GRID_HEIGHT, GRID_WIDTH};
use simulation::grid::RoadType;
use simulation::traffic::TrafficGrid;

// ---------------------------------------------------------------------------
// Benchmark: full grid clear (256x256)
// ---------------------------------------------------------------------------

fn bench_full_clear(c: &mut Criterion) {
    let mut group = c.benchmark_group("traffic_grid_clear");
    group.sample_size(200);

    let mut traffic = TrafficGrid::default();

    // Fill the grid with non-zero values so clear does real work
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            traffic.set(x, y, ((x + y) % 30) as u16);
        }
    }

    group.bench_function("full_clear_256x256", |b| {
        b.iter(|| {
            traffic.clear();
            black_box(&traffic);
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: congestion_level lookup
// ---------------------------------------------------------------------------

fn bench_congestion_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("traffic_congestion_lookup");
    group.sample_size(500);

    let mut traffic = TrafficGrid::default();

    // Set up varied congestion across the grid
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            traffic.set(x, y, ((x * 7 + y * 13) % 40) as u16);
        }
    }

    // Single-cell lookup at a representative cell
    group.bench_function("single_cell", |b| {
        b.iter(|| black_box(traffic.congestion_level(black_box(128), black_box(128))));
    });

    // Scan an entire row (256 lookups) to measure throughput
    group.bench_function("full_row_256", |b| {
        b.iter(|| {
            let mut total = 0.0f32;
            for x in 0..GRID_WIDTH {
                total += traffic.congestion_level(x, 128);
            }
            black_box(total)
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: path_cost_with_road for each RoadType
// ---------------------------------------------------------------------------

fn bench_path_cost_with_road(c: &mut Criterion) {
    let mut group = c.benchmark_group("traffic_path_cost_with_road");
    group.sample_size(500);

    let mut traffic = TrafficGrid::default();

    // Set moderate congestion at the test cell
    traffic.set(100, 100, 10);

    let road_types = [
        ("Local", RoadType::Local),
        ("Avenue", RoadType::Avenue),
        ("Boulevard", RoadType::Boulevard),
        ("Highway", RoadType::Highway),
        ("OneWay", RoadType::OneWay),
        ("Path", RoadType::Path),
    ];

    for (label, road_type) in road_types {
        group.bench_with_input(
            BenchmarkId::new("road_type", label),
            &road_type,
            |b, &rt| {
                b.iter(|| {
                    black_box(traffic.path_cost_with_road(black_box(100), black_box(100), rt))
                });
            },
        );
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: bulk update simulation (100K citizen positions)
// ---------------------------------------------------------------------------

fn bench_bulk_density_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("traffic_bulk_update");
    group.sample_size(50);

    // Simulate 100K citizen positions
    let positions: Vec<(usize, usize)> = (0..100_000)
        .map(|i| (i % GRID_WIDTH, (i * 7) % GRID_HEIGHT))
        .collect();

    group.bench_function("100k_citizens_clear_and_tally", |b| {
        let mut traffic = TrafficGrid::default();
        b.iter(|| {
            traffic.clear();
            for &(x, y) in &positions {
                let current = traffic.get(x, y);
                traffic.set(x, y, current.saturating_add(1));
            }
            black_box(&traffic);
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Register groups
// ---------------------------------------------------------------------------

criterion_group!(
    benches,
    bench_full_clear,
    bench_congestion_lookup,
    bench_path_cost_with_road,
    bench_bulk_density_update,
);
criterion_main!(benches);
