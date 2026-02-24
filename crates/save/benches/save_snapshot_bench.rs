//! Realistic save/load snapshot benchmark using the Tel Aviv map (SAVE-030).
//!
//! Measures the full pipeline: ECS world -> SaveData -> encode -> compress,
//! and the reverse load pipeline, using the actual Tel Aviv map with ~10K
//! real citizens and all simulation systems.
//!
//! Run with: `cargo bench -p save --bench save_snapshot_bench`

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use save::serialization::{
    assemble_save_data, collect_disaster_stage, collect_economy_stage, collect_entity_stage,
    collect_environment_stage, collect_grid_stage, collect_policy_stage, CitizenSaveInput,
    SaveData,
};

use simulation::agriculture::AgricultureState;
use simulation::budget::ExtendedBudget;
use simulation::buildings::{Building, MixedUseBuilding};
use simulation::citizen::{
    CitizenDetails, CitizenStateComp, Family, HomeLocation, Needs, PathCache, Personality,
    Position, Velocity, WorkLocation,
};
use simulation::cold_snap::ColdSnapState;
use simulation::composting::CompostingState;
use simulation::cso::SewerSystemState;
use simulation::degree_days::DegreeDays;
use simulation::drought::DroughtState;
use simulation::economy::CityBudget;
use simulation::flood_simulation::FloodState;
use simulation::fog::FogState;
use simulation::grid::WorldGrid;
use simulation::groundwater_depletion::GroundwaterDepletionState;
use simulation::hazardous_waste::HazardousWasteState;
use simulation::heat_wave::HeatWaveState;
use simulation::landfill_gas::LandfillGasState;
use simulation::landfill_warning::LandfillCapacityState;
use simulation::life_simulation::LifeSimTimer;
use simulation::lifecycle::LifecycleTimer;
use simulation::loans::LoanBook;
use simulation::movement::ActivityTimer;
use simulation::policies::Policies;
use simulation::recycling::{RecyclingEconomics, RecyclingState};
use simulation::reservoir::ReservoirState;
use simulation::road_segments::RoadSegmentStore;
use simulation::roads::RoadNetwork;
use simulation::services::ServiceBuilding;
use simulation::snow::{SnowGrid, SnowPlowingState};
use simulation::storm_drainage::StormDrainageState;
use simulation::stormwater::StormwaterGrid;
use simulation::test_harness::TestCity;
use simulation::time_of_day::GameClock;
use simulation::unlocks::UnlockState;
use simulation::urban_growth_boundary::UrbanGrowthBoundary;
use simulation::urban_heat_island::UhiGrid;
use simulation::utilities::UtilitySource;
use simulation::virtual_population::VirtualPopulation;
use simulation::wastewater::WastewaterState;
use simulation::water_conservation::WaterConservationState;
use simulation::water_sources::WaterSource;
use simulation::water_treatment::WaterTreatmentState;
use simulation::weather::{ClimateZone, ConstructionModifiers, Weather};
use simulation::wind_damage::WindDamageState;
use simulation::zones::ZoneDemand;

use bevy::prelude::*;

// ---------------------------------------------------------------------------
// Collect SaveData from a live ECS World
// ---------------------------------------------------------------------------

/// Collect SaveData from a live ECS world (mirrors the exclusive_save logic).
#[allow(clippy::too_many_lines)]
fn collect_save_from_world(world: &mut World) -> SaveData {
    // -- Entity queries --
    let building_data: Vec<(Building, Option<MixedUseBuilding>)> = {
        let mut q = world.query::<(&Building, Option<&MixedUseBuilding>)>();
        q.iter(world)
            .map(|(b, mu)| (b.clone(), mu.cloned()))
            .collect()
    };

    let citizen_data: Vec<CitizenSaveInput> = {
        let mut q = world.query::<(
            Entity,
            &CitizenDetails,
            &CitizenStateComp,
            &HomeLocation,
            &WorkLocation,
            &PathCache,
            &Velocity,
            &Position,
            &Personality,
            &Needs,
            &ActivityTimer,
            &Family,
        )>();
        q.iter(world)
            .map(
                |(entity, d, state, home, work, path, vel, pos, pers, needs, timer, family)| {
                    CitizenSaveInput {
                        entity,
                        details: d.clone(),
                        state: state.0,
                        home_x: home.grid_x,
                        home_y: home.grid_y,
                        work_x: work.grid_x,
                        work_y: work.grid_y,
                        path: path.clone(),
                        velocity: vel.clone(),
                        position: pos.clone(),
                        personality: pers.clone(),
                        needs: needs.clone(),
                        activity_timer: timer.0,
                        family: family.clone(),
                    }
                },
            )
            .collect()
    };

    let utility_data: Vec<UtilitySource> = {
        let mut q = world.query::<&UtilitySource>();
        q.iter(world).cloned().collect()
    };
    let service_data: Vec<(ServiceBuilding,)> = {
        let mut q = world.query::<&ServiceBuilding>();
        q.iter(world).map(|sb| (sb.clone(),)).collect()
    };
    let water_source_data: Vec<WaterSource> = {
        let mut q = world.query::<&WaterSource>();
        q.iter(world).cloned().collect()
    };

    // -- Resource collection stages --
    let grid_stage = {
        let grid = world.resource::<WorldGrid>();
        let roads = world.resource::<RoadNetwork>();
        let segments = world.resource::<RoadSegmentStore>();
        let seg_ref = if segments.segments.is_empty() {
            None
        } else {
            Some(segments.as_ref())
        };
        collect_grid_stage(grid, roads, seg_ref)
    };

    let economy_stage = {
        let clock = world.resource::<GameClock>();
        let budget = world.resource::<CityBudget>();
        let demand = world.resource::<ZoneDemand>();
        let ext = world.resource::<ExtendedBudget>();
        let loans = world.resource::<LoanBook>();
        collect_economy_stage(clock, budget, demand, Some(ext), Some(loans))
    };

    let entity_stage = collect_entity_stage(
        &building_data,
        &citizen_data,
        &utility_data,
        &service_data,
        if water_source_data.is_empty() {
            None
        } else {
            Some(&water_source_data)
        },
    );

    let environment_stage = {
        let weather = world.resource::<Weather>();
        let climate = world.resource::<ClimateZone>();
        let uhi = world.resource::<UhiGrid>();
        let sw = world.resource::<StormwaterGrid>();
        let dd = world.resource::<DegreeDays>();
        let cm = world.resource::<ConstructionModifiers>();
        let sg = world.resource::<SnowGrid>();
        let sp = world.resource::<SnowPlowingState>();
        let ag = world.resource::<AgricultureState>();
        let fog = world.resource::<FogState>();
        let ugb = world.resource::<UrbanGrowthBoundary>();
        collect_environment_stage(
            Some(weather),
            Some(climate),
            Some(uhi),
            Some(sw),
            Some(dd),
            Some(cm),
            Some((sg, sp)),
            Some(ag),
            Some(fog),
            Some(ugb),
        )
    };

    let disaster_stage = {
        let dr = world.resource::<DroughtState>();
        let hw = world.resource::<HeatWaveState>();
        let cs = world.resource::<ColdSnapState>();
        let fl = world.resource::<FloodState>();
        let wd = world.resource::<WindDamageState>();
        let rs = world.resource::<ReservoirState>();
        let lg = world.resource::<LandfillGasState>();
        let cso = world.resource::<SewerSystemState>();
        let hz = world.resource::<HazardousWasteState>();
        let ww = world.resource::<WastewaterState>();
        let sd = world.resource::<StormDrainageState>();
        let lc = world.resource::<LandfillCapacityState>();
        let gw = world.resource::<GroundwaterDepletionState>();
        let wt = world.resource::<WaterTreatmentState>();
        let wc = world.resource::<WaterConservationState>();
        collect_disaster_stage(
            Some(dr),
            Some(hw),
            Some(cs),
            Some(fl),
            Some(wd),
            Some(rs),
            Some(lg),
            Some(cso),
            Some(hz),
            Some(ww),
            Some(sd),
            Some(lc),
            Some(gw),
            Some(wt),
            Some(wc),
        )
    };

    let policy_stage = {
        let pol = world.resource::<Policies>();
        let us = world.resource::<UnlockState>();
        let rs = world.resource::<RecyclingState>();
        let re = world.resource::<RecyclingEconomics>();
        let co = world.resource::<CompostingState>();
        let lc = world.resource::<LifecycleTimer>();
        let ls = world.resource::<LifeSimTimer>();
        let vp = world.resource::<VirtualPopulation>();
        collect_policy_stage(
            Some(pol),
            Some(us),
            Some((rs, re)),
            Some(co),
            Some(lc),
            Some(ls),
            Some(vp),
        )
    };

    assemble_save_data(
        grid_stage,
        economy_stage,
        entity_stage,
        environment_stage,
        disaster_stage,
        policy_stage,
    )
}

// ---------------------------------------------------------------------------
// Benchmark: Tel Aviv realistic save/load
// ---------------------------------------------------------------------------

fn bench_tel_aviv_save(c: &mut Criterion) {
    let mut group = c.benchmark_group("save_tel_aviv");
    group.sample_size(10);

    // Build full Tel Aviv city (expensive ~1s setup).
    let mut city = TestCity::with_tel_aviv();
    city.tick(5);

    // Collect once to get baseline data + sizes.
    let save = collect_save_from_world(city.world_mut());
    let citizen_count = save.citizens.len();
    let building_count = save.buildings.len();
    eprintln!(
        "Tel Aviv snapshot: {} citizens, {} buildings",
        citizen_count, building_count
    );

    // Benchmark: ECS snapshot collection (world queries -> SaveData).
    group.bench_function("ecs_snapshot_collection", |b| {
        b.iter(|| black_box(collect_save_from_world(city.world_mut())));
    });

    // Benchmark: bitcode encode.
    group.bench_function("bitcode_encode", |b| {
        b.iter(|| black_box(save.encode()));
    });

    // Benchmark: LZ4 compress.
    let encoded = save.encode();
    eprintln!(
        "Encoded size: {} bytes ({:.2} MB)",
        encoded.len(),
        encoded.len() as f64 / 1_048_576.0
    );

    group.bench_function("lz4_compress", |b| {
        b.iter(|| black_box(lz4_flex::compress_prepend_size(&encoded)));
    });

    let compressed = lz4_flex::compress_prepend_size(&encoded);
    eprintln!(
        "Compressed size: {} bytes ({:.2} MB, {:.1}% ratio)",
        compressed.len(),
        compressed.len() as f64 / 1_048_576.0,
        compressed.len() as f64 / encoded.len() as f64 * 100.0
    );

    // Benchmark: full save pipeline (snapshot + encode + compress).
    group.bench_function("full_save_pipeline", |b| {
        b.iter(|| {
            let s = collect_save_from_world(city.world_mut());
            let enc = s.encode();
            let comp = lz4_flex::compress_prepend_size(&enc);
            black_box(comp.len())
        });
    });

    // Benchmark: full load pipeline (decompress + decode).
    group.bench_function("full_load_pipeline", |b| {
        b.iter(|| {
            let dec = lz4_flex::decompress_size_prepended(&compressed).unwrap();
            let s = SaveData::decode(&dec).unwrap();
            black_box(s.citizens.len())
        });
    });

    // Benchmark: bitcode decode only.
    group.bench_function("bitcode_decode", |b| {
        b.iter(|| black_box(SaveData::decode(&encoded).unwrap()));
    });

    // Benchmark: LZ4 decompress only.
    group.bench_function("lz4_decompress", |b| {
        b.iter(|| black_box(lz4_flex::decompress_size_prepended(&compressed).unwrap()));
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Register
// ---------------------------------------------------------------------------

criterion_group!(benches, bench_tel_aviv_save);
criterion_main!(benches);
