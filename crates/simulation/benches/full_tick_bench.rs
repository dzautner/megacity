//! Criterion benchmark: full simulation tick at scale.
//!
//! Measures the wall-clock time of a single `FixedUpdate` schedule execution
//! with varying citizen counts (1K, 10K, 50K, 100K). Each benchmark scenario
//! builds a realistic city with road infrastructure, zoned buildings,
//! services, utilities, and the requested number of citizens.
//!
//! Acceptance criteria (from issue #806):
//!   - `create_benchmark_app()` helper spawns N citizens with infrastructure
//!   - Benchmark at 1K, 10K, 50K, 100K citizen counts
//!   - 30-second measurement time, 10 samples
//!   - Budget: full tick < 16ms at 100K citizens
//!
//! Run with: cargo bench -p simulation --bench full_tick_bench --features bench

use bevy::prelude::*;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::time::Duration;

use simulation::buildings::Building;
use simulation::citizen::{
    Citizen, CitizenDetails, CitizenState, CitizenStateComp, Family, Gender, HomeLocation, Needs,
    PathCache, Personality, Position, Velocity, WorkLocation,
};
use simulation::config::{GRID_HEIGHT, GRID_WIDTH};
use simulation::grid::{RoadType, WorldGrid, ZoneType};
use simulation::lod::LodTier;
use simulation::mode_choice::ChosenTransportMode;
use simulation::movement::ActivityTimer;
use simulation::test_harness::TestCity;

// ---------------------------------------------------------------------------
// Helper: create a benchmark app with N citizens
// ---------------------------------------------------------------------------

/// Build a fully-wired `TestCity` with `citizen_count` citizens, road
/// infrastructure, residential and commercial buildings, services, and
/// utilities.
///
/// Layout strategy (for a 256x256 grid):
///   - Horizontal roads every 8 rows, vertical roads every 8 columns
///   - Residential zones (high-density) between road corridors
///   - Commercial zones along alternate corridors
///   - Services and utilities at fixed positions for coverage
///
/// Citizens are spawned directly into the ECS world with valid home/work
/// building references to avoid the per-citizen builder overhead.
fn create_benchmark_app(citizen_count: u32) -> TestCity {
    // Start with an empty city (256x256 grass grid, all simulation plugins).
    let mut city = TestCity::new();

    // -- Roads: grid pattern every 8 cells ------------------------------------
    // Place horizontal roads
    for row in (0..GRID_HEIGHT).step_by(8) {
        city = city.with_road(0, row, GRID_WIDTH - 1, row, RoadType::TwoLane);
    }
    // Place vertical roads
    for col in (0..GRID_WIDTH).step_by(8) {
        city = city.with_road(col, 0, col, GRID_HEIGHT - 1, RoadType::TwoLane);
    }

    // Rebuild CSR graph so pathfinding works.
    city = city.rebuild_csr();

    // -- Buildings: fill grid blocks with residential and commercial ----------
    let world = city.world_mut();
    let mut home_entities: Vec<Entity> = Vec::new();
    let mut work_entities: Vec<Entity> = Vec::new();

    {
        let mut grid = world.resource_mut::<WorldGrid>();

        // Each 8x8 block has interior cells at offsets 2..6 in both axes.
        // Alternate blocks between residential and commercial.
        for block_row in 0..(GRID_HEIGHT / 8) {
            for block_col in 0..(GRID_WIDTH / 8) {
                let is_residential = (block_row + block_col) % 2 == 0;
                let zone = if is_residential {
                    ZoneType::ResidentialHigh
                } else {
                    ZoneType::CommercialHigh
                };

                // Place one building per block at cell (base+3, base+3)
                let bx = block_col * 8 + 3;
                let by = block_row * 8 + 3;
                if !grid.in_bounds(bx, by) {
                    continue;
                }
                grid.get_mut(bx, by).zone = zone;
            }
        }
    }

    // Spawn building entities and register them in the grid.
    // We need to collect block positions first, then spawn.
    let mut res_positions: Vec<(usize, usize)> = Vec::new();
    let mut com_positions: Vec<(usize, usize)> = Vec::new();

    for block_row in 0..(GRID_HEIGHT / 8) {
        for block_col in 0..(GRID_WIDTH / 8) {
            let bx = block_col * 8 + 3;
            let by = block_row * 8 + 3;
            if bx >= GRID_WIDTH || by >= GRID_HEIGHT {
                continue;
            }
            let is_residential = (block_row + block_col) % 2 == 0;
            if is_residential {
                res_positions.push((bx, by));
            } else {
                com_positions.push((bx, by));
            }
        }
    }

    // Spawn residential buildings (high-density, level 5 = 2000 capacity each)
    for &(bx, by) in &res_positions {
        let capacity = Building::capacity_for_level(ZoneType::ResidentialHigh, 5);
        let entity = world
            .spawn(Building {
                zone_type: ZoneType::ResidentialHigh,
                level: 5,
                grid_x: bx,
                grid_y: by,
                capacity,
                occupants: 0,
            })
            .id();
        world
            .resource_mut::<WorldGrid>()
            .get_mut(bx, by)
            .building_id = Some(entity);
        home_entities.push(entity);
    }

    // Spawn commercial buildings (high-density, level 5)
    for &(bx, by) in &com_positions {
        let capacity = Building::capacity_for_level(ZoneType::CommercialHigh, 5);
        let entity = world
            .spawn(Building {
                zone_type: ZoneType::CommercialHigh,
                level: 5,
                grid_x: bx,
                grid_y: by,
                capacity,
                occupants: 0,
            })
            .id();
        world
            .resource_mut::<WorldGrid>()
            .get_mut(bx, by)
            .building_id = Some(entity);
        work_entities.push(entity);
    }

    // -- Spawn citizens with valid home/work references -----------------------
    if !home_entities.is_empty() && !work_entities.is_empty() {
        let home_count = home_entities.len();
        let work_count = work_entities.len();

        // Pre-read grid positions for homes and workplaces.
        let home_positions: Vec<(usize, usize)> = res_positions.clone();
        let work_positions: Vec<(usize, usize)> = com_positions.clone();

        for i in 0..citizen_count {
            let hi = (i as usize) % home_count;
            let wi = (i as usize) % work_count;

            let home_entity = home_entities[hi];
            let work_entity = work_entities[wi];
            let (hx, hy) = home_positions[hi];
            let (wx, wy) = work_positions[wi];
            let (hwx, hwy) = WorldGrid::grid_to_world(hx, hy);

            let age = 20 + (i % 45) as u8;
            let gender = if i % 2 == 0 {
                Gender::Male
            } else {
                Gender::Female
            };
            let edu = (i % 3) as u8;
            let salary = 2000.0 + (edu as f32) * 1500.0;

            world.spawn((
                Citizen,
                Position { x: hwx, y: hwy },
                Velocity { x: 0.0, y: 0.0 },
                HomeLocation {
                    grid_x: hx,
                    grid_y: hy,
                    building: home_entity,
                },
                WorkLocation {
                    grid_x: wx,
                    grid_y: wy,
                    building: work_entity,
                },
                CitizenStateComp(CitizenState::AtHome),
                PathCache::new(Vec::new()),
                CitizenDetails {
                    age,
                    gender,
                    education: edu,
                    happiness: 50.0,
                    health: 85.0,
                    salary,
                    savings: salary * 2.0,
                },
                Personality {
                    ambition: 0.5,
                    sociability: 0.5,
                    materialism: 0.5,
                    resilience: 0.5,
                },
                Needs::default(),
                Family::default(),
                ActivityTimer::default(),
                LodTier::default(),
                ChosenTransportMode::default(),
            ));
        }
    }

    // Warm up: run a few ticks so lazy-init systems settle.
    city.tick(5);

    city
}

// ---------------------------------------------------------------------------
// Benchmark: full simulation tick at varying citizen counts
// ---------------------------------------------------------------------------

fn bench_full_sim_tick(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_sim_tick");
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(10);

    for &count in &[1_000u32, 10_000, 50_000, 100_000] {
        // Build the city once per parameter (expensive setup).
        let mut city = create_benchmark_app(count);

        group.bench_with_input(
            BenchmarkId::new("fixed_update", format!("{count}_citizens")),
            &count,
            |b, _| {
                b.iter(|| {
                    city.world_mut().run_schedule(FixedUpdate);
                });
            },
        );
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Register benchmark group
// ---------------------------------------------------------------------------

criterion_group!(benches, bench_full_sim_tick);
criterion_main!(benches);
