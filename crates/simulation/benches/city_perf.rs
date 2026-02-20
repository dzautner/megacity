//! Performance benchmarks for MegaCity simulation systems.
//!
//! These benchmarks measure the critical-path systems that must perform well
//! at city scale (50K real entities, 256x256 grid, targeting 1M virtual pop).
//!
//! Run with: cargo bench -p simulation
//! Compare results over time to catch performance regressions.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rand::Rng;

use simulation::config::{CELL_SIZE, GRID_HEIGHT, GRID_WIDTH};
use simulation::grid::{CellType, WorldGrid};
use simulation::pathfinding_sys::{find_path, nearest_road, nearest_road_grid};
use simulation::road_graph_csr::{csr_find_path, CsrGraph};
use simulation::roads::{RoadNetwork, RoadNode};
use simulation::spatial_grid::SpatialGrid;
use simulation::traffic::TrafficGrid;

use bevy::prelude::Entity;

// ---------------------------------------------------------------------------
// Helpers: build realistic road layouts
// ---------------------------------------------------------------------------

/// Build a grid road network with roads every `spacing` cells.
/// Returns (grid, network, road_count).
fn build_road_grid(spacing: usize) -> (WorldGrid, RoadNetwork) {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut network = RoadNetwork::default();

    for y in (0..GRID_HEIGHT).step_by(spacing) {
        for x in 0..GRID_WIDTH {
            network.place_road(&mut grid, x, y);
        }
    }
    for x in (0..GRID_WIDTH).step_by(spacing) {
        for y in 0..GRID_HEIGHT {
            if grid.get(x, y).cell_type != CellType::Road {
                network.place_road(&mut grid, x, y);
            }
        }
    }

    (grid, network)
}

/// Build a dense downtown + sparse suburb road layout (realistic city).
fn build_realistic_roads() -> (WorldGrid, RoadNetwork) {
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    let mut network = RoadNetwork::default();
    let cx = GRID_WIDTH / 2;
    let cy = GRID_HEIGHT / 2;

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let dx = (x as i32 - cx as i32).unsigned_abs() as usize;
            let dy = (y as i32 - cy as i32).unsigned_abs() as usize;
            let dist = dx.max(dy);

            let spacing = if dist < 20 {
                4
            } else if dist < 50 {
                6
            } else {
                8
            };

            if x % spacing == 0 || y % spacing == 0 {
                if grid.get(x, y).cell_type != CellType::Road {
                    network.place_road(&mut grid, x, y);
                }
            }
        }
    }

    (grid, network)
}

// ---------------------------------------------------------------------------
// 1. PATHFINDING BENCHMARKS
// ---------------------------------------------------------------------------

fn bench_pathfinding(c: &mut Criterion) {
    let mut group = c.benchmark_group("pathfinding");
    group.sample_size(30);

    // Test with different road densities
    for spacing in [4, 6, 8] {
        let (_, network) = build_road_grid(spacing);
        let road_count = network.edges.len();

        // Short path (within a neighborhood)
        group.bench_with_input(
            BenchmarkId::new(
                "astar_short",
                format!("spacing{}_roads{}", spacing, road_count),
            ),
            &network,
            |b, net| {
                b.iter(|| {
                    black_box(find_path(net, RoadNode(10, 0), RoadNode(30, 0)));
                });
            },
        );

        // Medium path (across town)
        group.bench_with_input(
            BenchmarkId::new(
                "astar_medium",
                format!("spacing{}_roads{}", spacing, road_count),
            ),
            &network,
            |b, net| {
                b.iter(|| {
                    black_box(find_path(net, RoadNode(10, 0), RoadNode(200, 0)));
                });
            },
        );

        // Long path (corner to corner, worst case)
        group.bench_with_input(
            BenchmarkId::new(
                "astar_long",
                format!("spacing{}_roads{}", spacing, road_count),
            ),
            &network,
            |b, net| {
                b.iter(|| {
                    black_box(find_path(net, RoadNode(0, 0), RoadNode(GRID_WIDTH - 1, 0)));
                });
            },
        );
    }

    group.finish();
}

fn bench_csr_pathfinding(c: &mut Criterion) {
    let mut group = c.benchmark_group("csr_pathfinding");
    group.sample_size(30);

    let (_, network) = build_road_grid(4);
    let csr = CsrGraph::from_road_network(&network);

    group.bench_function("csr_astar_short", |b| {
        b.iter(|| {
            black_box(csr_find_path(&csr, RoadNode(10, 0), RoadNode(30, 0)));
        });
    });

    group.bench_function("csr_astar_medium", |b| {
        b.iter(|| {
            black_box(csr_find_path(&csr, RoadNode(10, 0), RoadNode(200, 0)));
        });
    });

    group.bench_function("csr_astar_long", |b| {
        b.iter(|| {
            black_box(csr_find_path(
                &csr,
                RoadNode(0, 0),
                RoadNode(GRID_WIDTH - 1, 0),
            ));
        });
    });

    group.bench_function("csr_build_from_network", |b| {
        b.iter(|| {
            black_box(CsrGraph::from_road_network(&network));
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// 2. NEAREST_ROAD BENCHMARK (known O(R) bottleneck)
// ---------------------------------------------------------------------------

fn bench_nearest_road(c: &mut Criterion) {
    let mut group = c.benchmark_group("nearest_road");
    group.sample_size(50);

    for spacing in [4, 6, 8] {
        let (_, network) = build_road_grid(spacing);
        let road_count = network.edges.len();

        // Query from a non-road cell
        group.bench_with_input(
            BenchmarkId::new("linear_scan", format!("roads{}", road_count)),
            &network,
            |b, net| {
                b.iter(|| {
                    black_box(nearest_road(net, 3, 3));
                });
            },
        );
    }

    // Grid-accelerated nearest_road
    for spacing in [4, 6, 8] {
        let (grid, network) = build_road_grid(spacing);
        let road_count = network.edges.len();

        group.bench_with_input(
            BenchmarkId::new("grid_lookup", format!("roads{}", road_count)),
            &(grid, network),
            |b, (g, _n)| {
                b.iter(|| {
                    black_box(nearest_road_grid(g, 3, 3));
                });
            },
        );
    }

    // Burst: simulate 1000 citizens all calling nearest_road in one tick
    let (grid, network) = build_road_grid(4);
    let mut rng = rand::thread_rng();
    let queries: Vec<(usize, usize)> = (0..1000)
        .map(|_| (rng.gen_range(0..GRID_WIDTH), rng.gen_range(0..GRID_HEIGHT)))
        .collect();

    group.bench_function("burst_1000_hashmap", |b| {
        b.iter(|| {
            for &(x, y) in &queries {
                black_box(nearest_road(&network, x, y));
            }
        });
    });

    group.bench_function("burst_1000_grid", |b| {
        b.iter(|| {
            for &(x, y) in &queries {
                black_box(nearest_road_grid(&grid, x, y));
            }
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// 3. SPATIAL GRID BENCHMARKS
// ---------------------------------------------------------------------------

fn bench_spatial_grid(c: &mut Criterion) {
    let mut group = c.benchmark_group("spatial_grid");

    // Full rebuild with 50K entities (the per-tick cost)
    for count in [10_000u32, 25_000, 50_000] {
        let mut rng = rand::thread_rng();
        let positions: Vec<(f32, f32)> = (0..count)
            .map(|_| {
                (
                    rng.gen_range(0.0..(GRID_WIDTH as f32 * CELL_SIZE)),
                    rng.gen_range(0.0..(GRID_HEIGHT as f32 * CELL_SIZE)),
                )
            })
            .collect();

        group.bench_with_input(
            BenchmarkId::new("full_rebuild", count),
            &positions,
            |b, pos| {
                let mut grid = SpatialGrid::default();
                // Pre-populate so the HashMap is warm
                for (i, &(x, y)) in pos.iter().enumerate() {
                    grid.insert(Entity::from_raw(i as u32), x, y);
                }

                b.iter(|| {
                    grid.clear();
                    for (i, &(x, y)) in pos.iter().enumerate() {
                        grid.insert(Entity::from_raw(i as u32), x, y);
                    }
                    black_box(grid.entity_count());
                });
            },
        );
    }

    // Query performance
    {
        let mut rng = rand::thread_rng();
        let mut grid = SpatialGrid::default();
        for i in 0..50_000u32 {
            let x = rng.gen_range(0.0..(GRID_WIDTH as f32 * CELL_SIZE));
            let y = rng.gen_range(0.0..(GRID_HEIGHT as f32 * CELL_SIZE));
            grid.insert(Entity::from_raw(i), x, y);
        }

        group.bench_function("query_small_rect", |b| {
            b.iter(|| {
                black_box(grid.query_rect(100.0, 100.0, 500.0, 500.0));
            });
        });

        group.bench_function("query_large_rect", |b| {
            b.iter(|| {
                black_box(grid.query_rect(0.0, 0.0, 2000.0, 2000.0));
            });
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// 4. GRID OPERATIONS BENCHMARKS
// ---------------------------------------------------------------------------

fn bench_grid_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("grid_operations");

    // Full grid scan (used by pollution, land_value, zone_demand, etc.)
    let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);

    group.bench_function("full_grid_scan_256x256", |b| {
        b.iter(|| {
            let mut count = 0u32;
            for cell in &grid.cells {
                if cell.cell_type == CellType::Road {
                    count += 1;
                }
            }
            black_box(count);
        });
    });

    // Grid scan with neighbor lookups (land_value pattern)
    group.bench_function("grid_scan_with_neighbors", |b| {
        b.iter(|| {
            let mut sum = 0u32;
            for y in 1..GRID_HEIGHT - 1 {
                for x in 1..GRID_WIDTH - 1 {
                    let (n4, n4c) = grid.neighbors4(x, y);
                    for &(nx, ny) in &n4[..n4c] {
                        sum += grid.get(nx, ny).elevation as u32;
                    }
                }
            }
            black_box(sum);
        });
    });

    // TrafficGrid clear + fill (per-tick operation)
    group.bench_function("traffic_grid_clear", |b| {
        let mut traffic = TrafficGrid::default();
        b.iter(|| {
            traffic.clear();
            black_box(&traffic);
        });
    });

    // Pollution/ServiceCoverage-style radius writes
    group.bench_function("radius_write_r10_x100_buildings", |b| {
        let mut data = vec![0u8; GRID_WIDTH * GRID_HEIGHT];
        let mut rng = rand::thread_rng();
        let buildings: Vec<(usize, usize)> = (0..100)
            .map(|_| {
                (
                    rng.gen_range(20..GRID_WIDTH - 20),
                    rng.gen_range(20..GRID_HEIGHT - 20),
                )
            })
            .collect();

        b.iter(|| {
            data.fill(0);
            for &(cx, cy) in &buildings {
                let r = 10i32;
                let r2 = (r as f32 * CELL_SIZE) * (r as f32 * CELL_SIZE);
                for dy in -r..=r {
                    for dx in -r..=r {
                        let nx = cx as i32 + dx;
                        let ny = cy as i32 + dy;
                        if nx >= 0 && nx < GRID_WIDTH as i32 && ny >= 0 && ny < GRID_HEIGHT as i32 {
                            let dist_sq =
                                (dx as f32 * CELL_SIZE).powi(2) + (dy as f32 * CELL_SIZE).powi(2);
                            if dist_sq <= r2 {
                                data[ny as usize * GRID_WIDTH + nx as usize] = 1;
                            }
                        }
                    }
                }
            }
            black_box(&data);
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// 5. HAPPINESS SYSTEM (50K citizen lookups)
// ---------------------------------------------------------------------------

fn bench_happiness_pattern(c: &mut Criterion) {
    let mut group = c.benchmark_group("happiness");

    // Simulate the per-citizen lookup pattern:
    // For each citizen, do ~5 boolean grid lookups + 3 numeric grid lookups + arithmetic
    for citizen_count in [10_000u32, 25_000, 50_000] {
        let mut rng = rand::thread_rng();
        let coverage_health = vec![true; GRID_WIDTH * GRID_HEIGHT];
        let coverage_edu = vec![true; GRID_WIDTH * GRID_HEIGHT];
        let coverage_police = vec![false; GRID_WIDTH * GRID_HEIGHT];
        let coverage_park = vec![true; GRID_WIDTH * GRID_HEIGHT];
        let coverage_ent = vec![false; GRID_WIDTH * GRID_HEIGHT];
        let pollution = vec![5u8; GRID_WIDTH * GRID_HEIGHT];
        let land_value = vec![50u8; GRID_WIDTH * GRID_HEIGHT];
        let traffic = vec![10u16; GRID_WIDTH * GRID_HEIGHT];

        // Random home positions for citizens
        let home_positions: Vec<usize> = (0..citizen_count)
            .map(|_| {
                let x = rng.gen_range(0..GRID_WIDTH);
                let y = rng.gen_range(0..GRID_HEIGHT);
                y * GRID_WIDTH + x
            })
            .collect();

        group.bench_with_input(
            BenchmarkId::new("per_citizen_lookup", citizen_count),
            &home_positions,
            |b, positions| {
                b.iter(|| {
                    let mut total_happiness = 0.0f32;
                    for &idx in positions {
                        let mut h = 50.0f32;
                        if coverage_health[idx] {
                            h += 8.0;
                        }
                        if coverage_edu[idx] {
                            h += 6.0;
                        }
                        if coverage_police[idx] {
                            h += 5.0;
                        }
                        if coverage_park[idx] {
                            h += 7.0;
                        }
                        if coverage_ent[idx] {
                            h += 4.0;
                        }
                        h -= pollution[idx] as f32 * 0.3;
                        h += land_value[idx] as f32 * 0.05;
                        h -= traffic[idx] as f32 * 0.1;
                        h = h.clamp(0.0, 100.0);
                        total_happiness += h;
                    }
                    black_box(total_happiness);
                });
            },
        );
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// 6. SERVICE COVERAGE GRID COMPUTATION
// ---------------------------------------------------------------------------

fn bench_service_coverage(c: &mut Criterion) {
    let mut group = c.benchmark_group("service_coverage");

    // Clear 5 x 65536 boolean arrays
    group.bench_function("clear_5_grids", |b| {
        let mut grids: Vec<Vec<bool>> = (0..5)
            .map(|_| vec![false; GRID_WIDTH * GRID_HEIGHT])
            .collect();

        b.iter(|| {
            for g in grids.iter_mut() {
                for v in g.iter_mut() {
                    *v = false;
                }
            }
            black_box(&grids);
        });
    });

    // Stamp coverage: 50 service buildings with radius 10-20 cells
    group.bench_function("stamp_50_services_r15", |b| {
        let mut grid = vec![false; GRID_WIDTH * GRID_HEIGHT];
        let mut rng = rand::thread_rng();
        let services: Vec<(usize, usize, i32)> = (0..50)
            .map(|_| {
                (
                    rng.gen_range(20..GRID_WIDTH - 20),
                    rng.gen_range(20..GRID_HEIGHT - 20),
                    15,
                )
            })
            .collect();

        b.iter(|| {
            grid.fill(false);
            for &(cx, cy, r) in &services {
                let r2 = (r as f32 * CELL_SIZE).powi(2);
                for dy in -r..=r {
                    for dx in -r..=r {
                        let nx = cx as i32 + dx;
                        let ny = cy as i32 + dy;
                        if nx >= 0 && nx < GRID_WIDTH as i32 && ny >= 0 && ny < GRID_HEIGHT as i32 {
                            let dist_sq =
                                (dx as f32 * CELL_SIZE).powi(2) + (dy as f32 * CELL_SIZE).powi(2);
                            if dist_sq <= r2 {
                                grid[ny as usize * GRID_WIDTH + nx as usize] = true;
                            }
                        }
                    }
                }
            }
            black_box(&grid);
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// 7. COMMUTE BURST SIMULATION
// ---------------------------------------------------------------------------

fn bench_commute_burst(c: &mut Criterion) {
    let mut group = c.benchmark_group("commute_burst");
    group.sample_size(10); // These are expensive

    let (_, network) = build_road_grid(4);
    let road_nodes: Vec<RoadNode> = network.edges.keys().copied().collect();

    // Simulate N citizens all computing paths in one tick
    for burst_size in [100, 500, 1000] {
        let mut rng = rand::thread_rng();
        let queries: Vec<(RoadNode, RoadNode)> = (0..burst_size)
            .map(|_| {
                let start = road_nodes[rng.gen_range(0..road_nodes.len())];
                let goal = road_nodes[rng.gen_range(0..road_nodes.len())];
                (start, goal)
            })
            .collect();

        group.bench_with_input(
            BenchmarkId::new("astar_burst", burst_size),
            &queries,
            |b, qs| {
                b.iter(|| {
                    for &(start, goal) in qs {
                        black_box(find_path(&network, start, goal));
                    }
                });
            },
        );
    }

    // Same with CSR
    let csr = CsrGraph::from_road_network(&network);
    let mut rng = rand::thread_rng();
    let queries: Vec<(RoadNode, RoadNode)> = (0..500)
        .map(|_| {
            let start = road_nodes[rng.gen_range(0..road_nodes.len())];
            let goal = road_nodes[rng.gen_range(0..road_nodes.len())];
            (start, goal)
        })
        .collect();

    group.bench_function("csr_burst_500", |b| {
        b.iter(|| {
            for &(start, goal) in &queries {
                black_box(csr_find_path(&csr, start, goal));
            }
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// 8. ROAD NETWORK BUILD PERFORMANCE
// ---------------------------------------------------------------------------

fn bench_road_network(c: &mut Criterion) {
    let mut group = c.benchmark_group("road_network");
    group.sample_size(10);

    // Time to build a full city road grid
    group.bench_function("build_grid_spacing4", |b| {
        b.iter(|| {
            let (grid, network) = build_road_grid(4);
            black_box(network.edges.len());
            black_box(grid.width);
        });
    });

    group.bench_function("build_realistic_roads", |b| {
        b.iter(|| {
            let (grid, network) = build_realistic_roads();
            black_box(network.edges.len());
            black_box(grid.width);
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// 9. MEMORY LAYOUT MEASUREMENT (not timed, but reported)
// ---------------------------------------------------------------------------

fn bench_memory_footprint(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_footprint");

    // WorldGrid allocation
    group.bench_function("world_grid_alloc", |b| {
        b.iter(|| {
            let grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
            black_box(grid.cells.len());
        });
    });

    // CSR graph from dense network
    group.bench_function("csr_from_dense_network", |b| {
        let (_, network) = build_road_grid(4);
        b.iter(|| {
            let csr = CsrGraph::from_road_network(&network);
            black_box((csr.node_count(), csr.edge_count()));
        });
    });

    // TrafficGrid + 5 coverage grids allocation
    group.bench_function("all_data_grids_alloc", |b| {
        b.iter(|| {
            let t = TrafficGrid::default();
            let p = vec![0u8; GRID_WIDTH * GRID_HEIGHT];
            let lv = vec![0u8; GRID_WIDTH * GRID_HEIGHT];
            let c1 = vec![false; GRID_WIDTH * GRID_HEIGHT];
            let c2 = vec![false; GRID_WIDTH * GRID_HEIGHT];
            let c3 = vec![false; GRID_WIDTH * GRID_HEIGHT];
            let c4 = vec![false; GRID_WIDTH * GRID_HEIGHT];
            let c5 = vec![false; GRID_WIDTH * GRID_HEIGHT];
            black_box(
                t.density.len()
                    + p.len()
                    + lv.len()
                    + c1.len()
                    + c2.len()
                    + c3.len()
                    + c4.len()
                    + c5.len(),
            );
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// 10. TICK BUDGET ESTIMATION
// ---------------------------------------------------------------------------

/// This benchmark simulates a single game tick's worth of non-ECS work:
/// - Clear traffic grid
/// - 50K spatial grid inserts
/// - 50K happiness lookups
/// - 500 path computations (batch of commuters)
///
/// The total time here must stay under 100ms (10 Hz target) to maintain
/// smooth simulation at full load.
fn bench_full_tick_estimate(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_tick_estimate");
    group.sample_size(10);

    let (_, network) = build_road_grid(4);
    let road_nodes: Vec<RoadNode> = network.edges.keys().copied().collect();
    let mut rng = rand::thread_rng();

    // Pre-generate citizen data
    let citizen_count = 50_000usize;
    let positions: Vec<(f32, f32)> = (0..citizen_count)
        .map(|_| {
            (
                rng.gen_range(0.0..(GRID_WIDTH as f32 * CELL_SIZE)),
                rng.gen_range(0.0..(GRID_HEIGHT as f32 * CELL_SIZE)),
            )
        })
        .collect();

    let home_indices: Vec<usize> = (0..citizen_count)
        .map(|_| {
            let x = rng.gen_range(0..GRID_WIDTH);
            let y = rng.gen_range(0..GRID_HEIGHT);
            y * GRID_WIDTH + x
        })
        .collect();

    let path_queries: Vec<(RoadNode, RoadNode)> = (0..500)
        .map(|_| {
            let s = road_nodes[rng.gen_range(0..road_nodes.len())];
            let g = road_nodes[rng.gen_range(0..road_nodes.len())];
            (s, g)
        })
        .collect();

    let coverage = vec![true; GRID_WIDTH * GRID_HEIGHT];
    let pollution = vec![5u8; GRID_WIDTH * GRID_HEIGHT];
    let land_value = vec![50u8; GRID_WIDTH * GRID_HEIGHT];

    let (_grid_for_tick, _) = build_road_grid(4);
    let csr = CsrGraph::from_road_network(&network);

    group.bench_function("simulated_tick_50k_citizens", |b| {
        let mut traffic = TrafficGrid::default();
        let mut spatial = SpatialGrid::default();
        // Warm up spatial grid
        for (i, &(x, y)) in positions.iter().enumerate() {
            spatial.insert(Entity::from_raw(i as u32), x, y);
        }

        b.iter(|| {
            // 1. Clear traffic
            traffic.clear();

            // 2. Spatial grid rebuild
            spatial.clear();
            for (i, &(x, y)) in positions.iter().enumerate() {
                spatial.insert(Entity::from_raw(i as u32), x, y);
            }

            // 3. Happiness computation for all citizens
            let mut total_h = 0.0f32;
            for &idx in &home_indices {
                let mut h = 50.0f32;
                if coverage[idx] {
                    h += 8.0;
                }
                if coverage[idx] {
                    h += 6.0;
                }
                h -= pollution[idx] as f32 * 0.3;
                h += land_value[idx] as f32 * 0.05;
                h = h.clamp(0.0, 100.0);
                total_h += h;
            }
            black_box(total_h);

            // 4. Pathfinding burst using CSR + grid-accelerated nearest_road
            for &(start, goal) in &path_queries {
                black_box(csr_find_path(&csr, start, goal));
            }

            // 5. Traffic density updates (commuting citizens)
            for &idx in home_indices.iter().take(10_000) {
                let x = idx % GRID_WIDTH;
                let y = idx / GRID_WIDTH;
                let cur = traffic.get(x, y);
                traffic.set(x, y, cur.saturating_add(1));
            }

            black_box(spatial.entity_count());
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// 11. REAL ECS TICK BENCHMARK (Tel Aviv map with all systems)
// ---------------------------------------------------------------------------

/// Measures the actual cost of a single `FixedUpdate` schedule execution
/// using the full Tel Aviv map with ~10K citizens and ALL simulation systems
/// (weather, floods, economy, happiness, movement, pathfinding, etc.).
///
/// This is the ground-truth performance metric. The budget is 16ms (60 Hz)
/// for the simulation tick alone, leaving headroom for rendering.
fn bench_ecs_tick(c: &mut Criterion) {
    use bevy::prelude::*;
    use simulation::test_harness::TestCity;

    let mut group = c.benchmark_group("ecs_tick");
    group.sample_size(20);

    // Build the full Tel Aviv city once (expensive ~1s setup).
    let mut city = TestCity::with_tel_aviv();
    // Warm up: run a few ticks so lazy-init systems settle.
    city.tick(10);

    group.bench_function("tel_aviv_fixed_update", |b| {
        b.iter(|| {
            city.world_mut().run_schedule(FixedUpdate);
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Register all benchmark groups
// ---------------------------------------------------------------------------

criterion_group!(
    benches,
    bench_pathfinding,
    bench_csr_pathfinding,
    bench_nearest_road,
    bench_spatial_grid,
    bench_grid_operations,
    bench_happiness_pattern,
    bench_service_coverage,
    bench_commute_burst,
    bench_road_network,
    bench_memory_footprint,
    bench_full_tick_estimate,
    bench_ecs_tick,
);
criterion_main!(benches);
