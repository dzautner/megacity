//! Save/load performance benchmarks at scale (SAVE-030).
//!
//! Measures serialization and deserialization performance for save files at
//! different city scales: 10K, 50K, 100K, and 500K citizens.
//!
//! Run with: `cargo bench -p save --bench save_bench`
//!
//! Performance budget (from issue #726):
//!   - Snapshot (ECS -> SaveData): <16ms
//!   - Encode (bitcode): <500ms
//!   - Full save (encode + compress + write): <1s
//!   - Full load (read + decompress + decode): <3s

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::collections::BTreeMap;

use save::serialization::{
    SaveBudget, SaveBuilding, SaveCell, SaveCitizen, SaveClock, SaveData, SaveDemand, SaveGrid,
    SaveRoadNetwork, CURRENT_SAVE_VERSION,
};

// ---------------------------------------------------------------------------
// Helpers: build synthetic SaveData at various scales
// ---------------------------------------------------------------------------

const GRID_W: usize = 256;
const GRID_H: usize = 256;

/// Build a synthetic SaveData with the given citizen count and proportional
/// buildings/services/utilities. Grid is 256x256 with ~20% road coverage.
fn build_synthetic_save(citizen_count: usize) -> SaveData {
    let cells: Vec<SaveCell> = (0..GRID_W * GRID_H)
        .map(|i| {
            let x = i % GRID_W;
            let y = i / GRID_W;
            let is_road = x % 4 == 0 || y % 4 == 0;
            SaveCell {
                elevation: 10.0 + (x as f32 * 0.01) + (y as f32 * 0.02),
                cell_type: u8::from(is_road),
                zone: if is_road { 0 } else { ((x + y) % 5) as u8 },
                road_type: u8::from(is_road),
                has_power: true,
                has_water: x < 200 && y < 200,
            }
        })
        .collect();

    let road_positions: Vec<(usize, usize)> = (0..GRID_W * GRID_H)
        .filter_map(|i| {
            let x = i % GRID_W;
            let y = i / GRID_W;
            if x % 4 == 0 || y % 4 == 0 {
                Some((x, y))
            } else {
                None
            }
        })
        .collect();

    let building_count = citizen_count / 10;
    let buildings: Vec<SaveBuilding> = (0..building_count)
        .map(|i| SaveBuilding {
            zone_type: ((i % 4) + 1) as u8,
            level: ((i % 3) + 1) as u8,
            grid_x: (i * 3) % GRID_W,
            grid_y: (i * 7) % GRID_H,
            capacity: 12,
            occupants: (i % 12) as u32,
            commercial_capacity: 0,
            commercial_occupants: 0,
            residential_capacity: 0,
            residential_occupants: 0,
        })
        .collect();

    let citizens: Vec<SaveCitizen> = (0..citizen_count)
        .map(|i| build_synthetic_citizen(i))
        .collect();

    SaveData {
        version: CURRENT_SAVE_VERSION,
        grid: SaveGrid {
            cells,
            width: GRID_W,
            height: GRID_H,
        },
        roads: SaveRoadNetwork { road_positions },
        clock: SaveClock {
            day: 42,
            hour: 14.5,
            speed: 1.0,
        },
        budget: SaveBudget {
            treasury: 500_000.0,
            tax_rate: 0.10,
            last_collection_day: 41,
        },
        demand: SaveDemand {
            residential: 0.6,
            commercial: 0.4,
            industrial: 0.3,
            office: 0.2,
            vacancy_residential: 0.05,
            vacancy_commercial: 0.08,
            vacancy_industrial: 0.12,
            vacancy_office: 0.10,
        },
        buildings,
        citizens,
        utility_sources: vec![],
        service_buildings: vec![],
        road_segments: None,
        policies: None,
        weather: None,
        unlock_state: None,
        extended_budget: None,
        loan_book: None,
        lifecycle_timer: None,
        virtual_population: None,
        life_sim_timer: None,
        stormwater_grid: None,
        water_sources: None,
        degree_days: None,
        construction_modifiers: None,
        recycling_state: None,
        wind_damage_state: None,
        uhi_grid: None,
        drought_state: None,
        heat_wave_state: None,
        composting_state: None,
        cold_snap_state: None,
        water_treatment_state: None,
        groundwater_depletion_state: None,
        wastewater_state: None,
        hazardous_waste_state: None,
        storm_drainage_state: None,
        landfill_capacity_state: None,
        flood_state: None,
        reservoir_state: None,
        landfill_gas_state: None,
        cso_state: None,
        water_conservation_state: None,
        fog_state: None,
        urban_growth_boundary: None,
        snow_state: None,
        agriculture_state: None,
        extensions: BTreeMap::new(),
    }
}

fn build_synthetic_citizen(i: usize) -> SaveCitizen {
    SaveCitizen {
        age: 20 + (i % 60) as u8,
        happiness: 50.0 + (i % 50) as f32,
        education: (i % 4) as u8,
        state: (i % 10) as u8,
        home_x: (i * 3) % GRID_W,
        home_y: (i * 7) % GRID_H,
        work_x: (i * 5 + 10) % GRID_W,
        work_y: (i * 11 + 20) % GRID_H,
        path_waypoints: vec![
            ((i * 3 + 1) % GRID_W, (i * 7 + 1) % GRID_H),
            ((i * 3 + 2) % GRID_W, (i * 7 + 2) % GRID_H),
            ((i * 5 + 10) % GRID_W, (i * 11 + 20) % GRID_H),
        ],
        path_current_index: 0,
        velocity_x: 0.5,
        velocity_y: -0.3,
        pos_x: ((i * 3) % GRID_W) as f32 * 16.0 + 8.0,
        pos_y: ((i * 7) % GRID_H) as f32 * 16.0 + 8.0,
        gender: (i % 2) as u8,
        health: 70.0 + (i % 30) as f32,
        salary: 2000.0 + (i % 5000) as f32,
        savings: 5000.0 + (i % 20000) as f32,
        ambition: 0.3 + (i % 7) as f32 * 0.1,
        sociability: 0.4 + (i % 6) as f32 * 0.1,
        materialism: 0.2 + (i % 8) as f32 * 0.1,
        resilience: 0.5 + (i % 5) as f32 * 0.1,
        need_hunger: 60.0 + (i % 40) as f32,
        need_energy: 50.0 + (i % 50) as f32,
        need_social: 40.0 + (i % 60) as f32,
        need_fun: 30.0 + (i % 70) as f32,
        need_comfort: 50.0 + (i % 50) as f32,
        activity_timer: (i % 100) as u32,
        family_partner: if i % 3 == 0 { (i + 1) as u32 } else { u32::MAX },
        family_children: if i % 5 == 0 {
            vec![(i + 2) as u32]
        } else {
            vec![]
        },
        family_parent: u32::MAX,
    }
}

// ---------------------------------------------------------------------------
// 1. BITCODE ENCODE (SaveData -> bytes)
// ---------------------------------------------------------------------------

fn bench_encode(c: &mut Criterion) {
    let mut group = c.benchmark_group("save_encode");
    group.sample_size(10);

    for &count in &[10_000usize, 50_000, 100_000, 500_000] {
        let save = build_synthetic_save(count);
        group.bench_with_input(
            BenchmarkId::new("bitcode_encode", format!("{count}_citizens")),
            &save,
            |b, save| {
                b.iter(|| black_box(save.encode()));
            },
        );
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// 2. BITCODE DECODE (bytes -> SaveData)
// ---------------------------------------------------------------------------

fn bench_decode(c: &mut Criterion) {
    let mut group = c.benchmark_group("save_decode");
    group.sample_size(10);

    for &count in &[10_000usize, 50_000, 100_000, 500_000] {
        let save = build_synthetic_save(count);
        let encoded = save.encode();
        group.bench_with_input(
            BenchmarkId::new("bitcode_decode", format!("{count}_citizens")),
            &encoded,
            |b, bytes| {
                b.iter(|| black_box(SaveData::decode(bytes).unwrap()));
            },
        );
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// 3. LZ4 COMPRESSION
// ---------------------------------------------------------------------------

fn bench_lz4_compress(c: &mut Criterion) {
    let mut group = c.benchmark_group("save_lz4_compress");
    group.sample_size(10);

    for &count in &[10_000usize, 50_000, 100_000, 500_000] {
        let save = build_synthetic_save(count);
        let encoded = save.encode();
        group.bench_with_input(
            BenchmarkId::new("lz4_compress", format!("{count}_citizens")),
            &encoded,
            |b, bytes| {
                b.iter(|| black_box(lz4_flex::compress_prepend_size(bytes)));
            },
        );
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// 4. LZ4 DECOMPRESSION
// ---------------------------------------------------------------------------

fn bench_lz4_decompress(c: &mut Criterion) {
    let mut group = c.benchmark_group("save_lz4_decompress");
    group.sample_size(10);

    for &count in &[10_000usize, 50_000, 100_000, 500_000] {
        let save = build_synthetic_save(count);
        let encoded = save.encode();
        let compressed = lz4_flex::compress_prepend_size(&encoded);
        group.bench_with_input(
            BenchmarkId::new("lz4_decompress", format!("{count}_citizens")),
            &compressed,
            |b, bytes| {
                b.iter(|| black_box(lz4_flex::decompress_size_prepended(bytes).unwrap()));
            },
        );
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// 5. FULL SAVE PIPELINE: encode + compress
// ---------------------------------------------------------------------------

fn bench_full_save_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("save_full_pipeline");
    group.sample_size(10);

    for &count in &[10_000usize, 50_000, 100_000, 500_000] {
        let save = build_synthetic_save(count);
        group.bench_with_input(
            BenchmarkId::new("encode_and_compress", format!("{count}_citizens")),
            &save,
            |b, save| {
                b.iter(|| {
                    let encoded = save.encode();
                    let compressed = lz4_flex::compress_prepend_size(&encoded);
                    black_box(compressed.len())
                });
            },
        );
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// 6. FULL LOAD PIPELINE: decompress + decode
// ---------------------------------------------------------------------------

fn bench_full_load_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("load_full_pipeline");
    group.sample_size(10);

    for &count in &[10_000usize, 50_000, 100_000, 500_000] {
        let save = build_synthetic_save(count);
        let encoded = save.encode();
        let compressed = lz4_flex::compress_prepend_size(&encoded);
        group.bench_with_input(
            BenchmarkId::new("decompress_and_decode", format!("{count}_citizens")),
            &compressed,
            |b, bytes| {
                b.iter(|| {
                    let decompressed = lz4_flex::decompress_size_prepended(bytes).unwrap();
                    let save = SaveData::decode(&decompressed).unwrap();
                    black_box(save.citizens.len())
                });
            },
        );
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// Register all benchmark groups
// ---------------------------------------------------------------------------

criterion_group!(
    benches,
    bench_encode,
    bench_decode,
    bench_lz4_compress,
    bench_lz4_decompress,
    bench_full_save_pipeline,
    bench_full_load_pipeline,
);
criterion_main!(benches);
