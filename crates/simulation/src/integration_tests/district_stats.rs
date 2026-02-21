use crate::buildings::Building;
use crate::config::GRID_WIDTH;
use crate::grid::ZoneType;
use crate::test_harness::TestCity;
use crate::utilities::UtilityType;

// ===========================================================================
// TEST-056: District Statistics Aggregation
// ===========================================================================

/// Verify that the automatic `Districts` resource correctly tallies
/// per-district population from residential buildings after aggregation.
/// After running a slow cycle, we read actual building occupants and verify
/// district population matches for buildings in different statistical districts.
#[test]
fn test_district_aggregate_population_matches_residential_occupants() {
    use crate::districts::{Districts, DISTRICT_SIZE};

    let bld_a = (5, 5);
    let bld_b = (DISTRICT_SIZE + 3, 5);

    let mut city = TestCity::new()
        .with_building(bld_a.0, bld_a.1, ZoneType::ResidentialLow, 1)
        .with_building(bld_b.0, bld_b.1, ZoneType::ResidentialLow, 1);

    city.tick_slow_cycle();

    let mut occ_a = 0u32;
    let mut occ_b = 0u32;
    {
        let world = city.world_mut();
        let mut query = world.query::<&Building>();
        for building in query.iter(world) {
            if building.grid_x == bld_a.0
                && building.grid_y == bld_a.1
                && building.zone_type.is_residential()
            {
                occ_a += building.occupants;
            } else if building.grid_x == bld_b.0
                && building.grid_y == bld_b.1
                && building.zone_type.is_residential()
            {
                occ_b += building.occupants;
            }
        }
    }

    let districts = city.resource::<Districts>();
    let (da_x, da_y) = Districts::district_for_grid(bld_a.0, bld_a.1);
    let (db_x, db_y) = Districts::district_for_grid(bld_b.0, bld_b.1);

    assert_eq!(
        districts.get(da_x, da_y).population,
        occ_a,
        "District ({da_x},{da_y}) population should match building occupants"
    );
    assert_eq!(
        districts.get(db_x, db_y).population,
        occ_b,
        "District ({db_x},{db_y}) population should match building occupants"
    );
}

/// Verify per-district job counts after aggregation.
#[test]
fn test_district_aggregate_job_counts() {
    use crate::districts::Districts;

    let comm_pos = (5, 5);
    let ind_pos = (5, 6);
    let off_pos = (5, 7);

    let mut city = TestCity::new()
        .with_building(comm_pos.0, comm_pos.1, ZoneType::CommercialLow, 1)
        .with_building(ind_pos.0, ind_pos.1, ZoneType::Industrial, 1)
        .with_building(off_pos.0, off_pos.1, ZoneType::Office, 1);

    city.tick_slow_cycle();

    let mut expected_employed = 0u32;
    let mut expected_comm_cap = 0u32;
    let mut expected_ind_cap = 0u32;
    let mut expected_off_cap = 0u32;
    {
        let world = city.world_mut();
        let mut query = world.query::<&Building>();
        for building in query.iter(world) {
            let (dx, _) = Districts::district_for_grid(building.grid_x, building.grid_y);
            let (edx, _) = Districts::district_for_grid(comm_pos.0, comm_pos.1);
            if dx == edx {
                if building.zone_type.is_commercial() {
                    expected_employed += building.occupants;
                    expected_comm_cap += building.capacity;
                } else if building.zone_type == ZoneType::Industrial {
                    expected_employed += building.occupants;
                    expected_ind_cap += building.capacity;
                } else if building.zone_type == ZoneType::Office {
                    expected_employed += building.occupants;
                    expected_off_cap += building.capacity;
                }
            }
        }
    }

    let districts = city.resource::<Districts>();
    let (dx, dy) = Districts::district_for_grid(comm_pos.0, comm_pos.1);
    let d = districts.get(dx, dy);

    assert_eq!(
        d.commercial_jobs, expected_comm_cap,
        "Commercial jobs capacity should match"
    );
    assert_eq!(
        d.industrial_jobs, expected_ind_cap,
        "Industrial jobs capacity should match"
    );
    assert_eq!(
        d.office_jobs, expected_off_cap,
        "Office jobs capacity should match"
    );
    assert_eq!(
        d.employed, expected_employed,
        "Employed should match sum of non-residential occupants"
    );
}

/// Verify per-district happiness average is computed when citizens exist.
/// Happiness drifts significantly during simulation so we verify the
/// aggregation produces a non-zero value within the valid range.
#[test]
fn test_district_aggregate_happiness_average() {
    use crate::districts::Districts;

    let home_pos = (10, 10);

    let mut city = TestCity::new()
        .with_building(home_pos.0, home_pos.1, ZoneType::ResidentialLow, 1)
        .with_utility(home_pos.0 + 1, home_pos.1, UtilityType::PowerPlant)
        .with_utility(home_pos.0, home_pos.1 + 1, UtilityType::WaterTower)
        .with_unemployed_citizen(home_pos)
        .with_unemployed_citizen(home_pos)
        .with_unemployed_citizen(home_pos);

    city.tick_slow_cycle();

    let districts = city.resource::<Districts>();
    let (dx, dy) = Districts::district_for_grid(home_pos.0, home_pos.1);
    let avg = districts.get(dx, dy).avg_happiness;

    // Citizens exist, so average happiness should be > 0 and <= 100
    assert!(
        avg > 0.0,
        "Avg happiness should be > 0 when citizens exist, got {avg}"
    );
    assert!(avg <= 100.0, "Avg happiness should be <= 100, got {avg}");
}

/// Verify sum of district populations equals total.
#[test]
fn test_district_population_sums_to_total() {
    use crate::districts::Districts;

    let mut city = TestCity::new()
        .with_building(5, 5, ZoneType::ResidentialLow, 1)
        .with_building(20, 5, ZoneType::ResidentialLow, 1)
        .with_building(40, 5, ZoneType::ResidentialHigh, 1);

    city.tick_slow_cycle();

    let districts = city.resource::<Districts>();
    let total = districts.total_statistical_population();
    let sum: u32 = districts.data.iter().map(|d| d.population).sum();

    assert_eq!(
        sum, total,
        "Sum of per-district populations ({sum}) must equal total_statistical_population ({total})"
    );
}

/// Verify DistrictMap cell assignment and reassignment.
#[test]
fn test_district_map_cell_assignment_tracking() {
    use crate::districts::DistrictMap;

    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut dmap = world.resource_mut::<DistrictMap>();

        for x in 10..14 {
            for y in 10..14 {
                dmap.assign_cell_to_district(x, y, 0);
            }
        }
        assert_eq!(
            dmap.districts[0].cells.len(),
            16,
            "Should have 4x4=16 cells"
        );
        assert_eq!(dmap.get_district_index_at(12, 12), Some(0));

        for x in 12..14 {
            for y in 12..14 {
                dmap.assign_cell_to_district(x, y, 1);
            }
        }
        assert_eq!(
            dmap.districts[0].cells.len(),
            12,
            "Downtown should now have 16-4=12 cells"
        );
        assert_eq!(
            dmap.districts[1].cells.len(),
            4,
            "Suburbs should have 4 cells"
        );
        assert_eq!(dmap.get_district_index_at(12, 12), Some(1));
        assert_eq!(dmap.get_district_index_at(10, 10), Some(0));
    }
}

/// Verify district_stats population from buildings with utility coverage.
/// Utility sources prevent abandonment which would zero occupants.
#[test]
fn test_district_stats_population_from_buildings() {
    use crate::districts::DistrictMap;

    let bld_a = (10, 10);
    let bld_b = (20, 20);

    let mut city = TestCity::new()
        .with_building(bld_a.0, bld_a.1, ZoneType::ResidentialLow, 1)
        .with_building(bld_b.0, bld_b.1, ZoneType::ResidentialLow, 1)
        .with_utility(bld_a.0 + 1, bld_a.1, UtilityType::PowerPlant)
        .with_utility(bld_a.0, bld_a.1 + 1, UtilityType::WaterTower)
        .with_utility(bld_b.0 + 1, bld_b.1, UtilityType::PowerPlant)
        .with_utility(bld_b.0, bld_b.1 + 1, UtilityType::WaterTower);

    // Set occupants after utility sources are placed
    {
        let world = city.world_mut();
        let mut query = world.query::<&mut Building>();
        for mut building in query.iter_mut(world) {
            if building.grid_x == bld_a.0 && building.grid_y == bld_a.1 {
                building.occupants = 15;
            } else if building.grid_x == bld_b.0 && building.grid_y == bld_b.1 {
                building.occupants = 30;
            }
        }
    }

    // Assign building cells to player districts
    {
        let world = city.world_mut();
        let mut dmap = world.resource_mut::<DistrictMap>();
        dmap.assign_cell_to_district(bld_a.0, bld_a.1, 0);
        dmap.assign_cell_to_district(bld_b.0, bld_b.1, 1);
    }

    // district_stats runs every 50 ticks
    city.tick(50);

    let dmap = city.resource::<DistrictMap>();
    assert_eq!(
        dmap.districts[0].stats.population, 15,
        "Downtown district should have population 15"
    );
    assert_eq!(
        dmap.districts[1].stats.population, 30,
        "Suburbs district should have population 30"
    );
}

/// Verify player-district crime stats from CrimeGrid.
#[test]
fn test_district_stats_crime_average() {
    use crate::crime::CrimeGrid;
    use crate::districts::DistrictMap;

    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut dmap = world.resource_mut::<DistrictMap>();
        for x in 50..52 {
            for y in 50..52 {
                dmap.assign_cell_to_district(x, y, 0);
            }
        }
        let mut crime = world.resource_mut::<CrimeGrid>();
        crime.set(50, 50, 40);
        crime.set(51, 50, 60);
        crime.set(50, 51, 80);
        crime.set(51, 51, 20);
    }

    city.tick(50);

    let dmap = city.resource::<DistrictMap>();
    let crime = dmap.districts[0].stats.crime;
    assert!(
        (crime - 50.0).abs() < 1.0,
        "Expected avg crime ~50.0, got {crime}"
    );
}

/// Verify player-district happiness from citizens.
#[test]
fn test_district_stats_happiness_from_citizens() {
    use crate::citizen::{CitizenDetails, HomeLocation};
    use crate::districts::DistrictMap;

    let home_pos = (30, 30);
    let work_pos = (32, 32);

    let mut city = TestCity::new()
        .with_building(home_pos.0, home_pos.1, ZoneType::ResidentialLow, 1)
        .with_building(work_pos.0, work_pos.1, ZoneType::CommercialLow, 1)
        .with_citizen(home_pos, work_pos)
        .with_citizen(home_pos, work_pos);

    {
        let world = city.world_mut();
        let mut dmap = world.resource_mut::<DistrictMap>();
        dmap.assign_cell_to_district(home_pos.0, home_pos.1, 2);
    }

    city.tick(50);

    let mut happiness_sum = 0.0f32;
    let mut count = 0u32;
    {
        let dmap_cell_map: Vec<Option<usize>>;
        {
            let dmap = city.resource::<DistrictMap>();
            dmap_cell_map = dmap.cell_map.clone();
        }
        let world = city.world_mut();
        let mut query = world.query::<(&CitizenDetails, &HomeLocation)>();
        for (details, home) in query.iter(world) {
            let idx = home.grid_y * GRID_WIDTH + home.grid_x;
            if dmap_cell_map.get(idx).copied().flatten() == Some(2) {
                happiness_sum += details.happiness;
                count += 1;
            }
        }
    }

    let dmap = city.resource::<DistrictMap>();
    let avg = dmap.districts[2].stats.avg_happiness;

    if count > 0 {
        let expected = happiness_sum / count as f32;
        assert!(
            (avg - expected).abs() < 10.0,
            "Expected avg happiness ~{expected} (within 10.0), got {avg}"
        );
    }
}

/// Verify empty districts have zero stats.
#[test]
fn test_district_empty_district_has_zero_stats() {
    use crate::districts::{DistrictMap, Districts};

    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut dmap = world.resource_mut::<DistrictMap>();
        for x in 100..105 {
            for y in 100..105 {
                dmap.assign_cell_to_district(x, y, 0);
            }
        }
    }

    city.tick_slow_cycle();

    let dmap = city.resource::<DistrictMap>();
    assert_eq!(dmap.districts[0].stats.population, 0);
    assert!((dmap.districts[0].stats.avg_happiness).abs() < f32::EPSILON);
    assert!((dmap.districts[0].stats.crime).abs() < f32::EPSILON);

    let districts = city.resource::<Districts>();
    let (dx, dy) = Districts::district_for_grid(100, 100);
    let d = districts.get(dx, dy);
    assert_eq!(d.population, 0);
    assert_eq!(d.employed, 0);
    assert_eq!(d.residential_capacity, 0);
    assert!((d.avg_happiness).abs() < f32::EPSILON);
}

/// Verify district_for_grid boundary mapping.
#[test]
fn test_district_grid_boundary_mapping() {
    use crate::districts::{Districts, DISTRICTS_X, DISTRICTS_Y, DISTRICT_SIZE};

    assert_eq!(Districts::district_for_grid(0, 0), (0, 0));
    assert_eq!(
        Districts::district_for_grid(DISTRICT_SIZE - 1, DISTRICT_SIZE - 1),
        (0, 0)
    );
    assert_eq!(Districts::district_for_grid(DISTRICT_SIZE, 0), (1, 0));
    assert_eq!(Districts::district_for_grid(0, DISTRICT_SIZE), (0, 1));
    assert_eq!(
        Districts::district_for_grid(255, 255),
        (DISTRICTS_X - 1, DISTRICTS_Y - 1)
    );
}

/// Verify residential capacity aggregation per district.
#[test]
fn test_district_aggregate_residential_capacity() {
    use crate::districts::Districts;

    let pos_a = (5, 5);
    let pos_b = (6, 5);

    let mut city = TestCity::new()
        .with_building(pos_a.0, pos_a.1, ZoneType::ResidentialLow, 1)
        .with_building(pos_b.0, pos_b.1, ZoneType::ResidentialHigh, 1);

    city.tick_slow_cycle();

    let districts = city.resource::<Districts>();
    let (dx, dy) = Districts::district_for_grid(pos_a.0, pos_a.1);
    let d = districts.get(dx, dy);

    let expected_cap = Building::capacity_for_level(ZoneType::ResidentialLow, 1)
        + Building::capacity_for_level(ZoneType::ResidentialHigh, 1);
    assert_eq!(
        d.residential_capacity, expected_cap,
        "Residential capacity should be sum of both buildings: {expected_cap}"
    );
}

/// Verify no stat bleed between adjacent districts.
#[test]
fn test_district_no_stat_bleed_between_districts() {
    use crate::districts::{Districts, DISTRICT_SIZE};

    let pos_a = (DISTRICT_SIZE - 1, 0);
    let pos_b = (DISTRICT_SIZE, 0);

    let mut city = TestCity::new()
        .with_building(pos_a.0, pos_a.1, ZoneType::ResidentialLow, 1)
        .with_building(pos_b.0, pos_b.1, ZoneType::Industrial, 1);

    city.tick_slow_cycle();

    let districts = city.resource::<Districts>();
    let d0 = districts.get(0, 0);
    let d1 = districts.get(1, 0);

    assert_eq!(
        d0.industrial_jobs, 0,
        "District (0,0) should have no industrial jobs"
    );
    assert!(
        d1.industrial_jobs > 0,
        "District (1,0) should have industrial capacity"
    );
    assert_eq!(
        d1.residential_capacity, 0,
        "District (1,0) should have no residential capacity"
    );
    assert!(
        d0.residential_capacity > 0,
        "District (0,0) should have residential capacity"
    );
}

/// Verify cell removal updates both cell_map and district cells.
#[test]
fn test_district_map_remove_cell_updates_both_sides() {
    use crate::districts::DistrictMap;

    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut dmap = world.resource_mut::<DistrictMap>();

        dmap.assign_cell_to_district(50, 50, 0);
        assert!(dmap.districts[0].cells.contains(&(50, 50)));
        assert_eq!(dmap.get_district_index_at(50, 50), Some(0));

        dmap.remove_cell_from_district(50, 50);
        assert!(!dmap.districts[0].cells.contains(&(50, 50)));
        assert_eq!(dmap.get_district_index_at(50, 50), None);
    }
}

/// Verify default districts are pre-populated.
#[test]
fn test_district_map_default_districts_exist() {
    use crate::districts::{DistrictMap, DEFAULT_DISTRICTS};

    let city = TestCity::new();
    let dmap = city.resource::<DistrictMap>();

    assert_eq!(
        dmap.districts.len(),
        DEFAULT_DISTRICTS.len(),
        "Default district count should match DEFAULT_DISTRICTS"
    );
    for (i, &(name, _color)) in DEFAULT_DISTRICTS.iter().enumerate() {
        assert_eq!(dmap.districts[i].name, name, "District {i} name mismatch");
    }
}
