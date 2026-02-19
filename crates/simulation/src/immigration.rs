use bevy::prelude::*;

use crate::buildings::{Building, UnderConstruction};
use crate::citizen::{
    Citizen, CitizenDetails, CitizenState, CitizenStateComp, Family, Gender, HomeLocation, Needs,
    PathCache, Personality, Position, Velocity, WorkLocation,
};
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::economy::CityBudget;
use crate::education_jobs::EmploymentStats;
use crate::grid::WorldGrid;
use crate::happiness::{
    ServiceCoverageGrid, COVERAGE_EDUCATION, COVERAGE_FIRE, COVERAGE_HEALTH, COVERAGE_POLICE,
};
use crate::movement::ActivityTimer;
use crate::stats::CityStats;
use crate::virtual_population::VirtualPopulation;
use crate::TickCounter;

// ---------------------------------------------------------------------------
// City Attractiveness
// ---------------------------------------------------------------------------

/// Breakdown of city attractiveness factors, each scored 0.0-1.0.
#[derive(Resource, Debug, Clone)]
pub struct CityAttractiveness {
    pub overall_score: f32,
    pub employment_factor: f32,
    pub happiness_factor: f32,
    pub services_factor: f32,
    pub housing_factor: f32,
    pub tax_factor: f32,
}

impl Default for CityAttractiveness {
    fn default() -> Self {
        Self {
            overall_score: 50.0,
            employment_factor: 0.5,
            happiness_factor: 0.5,
            services_factor: 0.5,
            housing_factor: 0.5,
            tax_factor: 0.5,
        }
    }
}

// ---------------------------------------------------------------------------
// Immigration Statistics
// ---------------------------------------------------------------------------

#[derive(Resource, Debug, Clone, Default)]
pub struct ImmigrationStats {
    pub immigrants_this_month: u32,
    pub emigrants_this_month: u32,
    pub net_migration: i32,
    /// Tick of last monthly reset
    last_reset_tick: u64,
}

// ---------------------------------------------------------------------------
// Weights for the attractiveness formula
// ---------------------------------------------------------------------------

const WEIGHT_EMPLOYMENT: f32 = 25.0;
const WEIGHT_HAPPINESS: f32 = 25.0;
const WEIGHT_SERVICES: f32 = 20.0;
const WEIGHT_HOUSING: f32 = 15.0;
const WEIGHT_TAX: f32 = 15.0;

/// Interval in ticks between attractiveness recomputation.
const ATTRACTIVENESS_INTERVAL: u64 = 50;
/// Interval in ticks between immigration wave checks.
const IMMIGRATION_INTERVAL: u64 = 100;
/// Monthly stats reset interval (roughly 1000 ticks ~ 100 seconds).
const MONTHLY_RESET_INTERVAL: u64 = 1000;

// ---------------------------------------------------------------------------
// System: compute_attractiveness
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub fn compute_attractiveness(
    tick: Res<TickCounter>,
    employment_stats: Res<EmploymentStats>,
    city_stats: Res<CityStats>,
    budget: Res<CityBudget>,
    coverage: Res<ServiceCoverageGrid>,
    buildings: Query<&Building, Without<UnderConstruction>>,
    mut attractiveness: ResMut<CityAttractiveness>,
) {
    if !tick.0.is_multiple_of(ATTRACTIVENESS_INTERVAL) {
        return;
    }

    // --- Employment factor ---
    // Low unemployment = attractive. 0% unemployment -> 1.0, 20%+ -> 0.0
    let unemployment = employment_stats.unemployment_rate;
    let employment_factor = (1.0 - unemployment * 5.0).clamp(0.0, 1.0);

    // --- Happiness factor ---
    // Average happiness mapped from 0-100 to 0.0-1.0
    let happiness_factor = (city_stats.average_happiness / 100.0).clamp(0.0, 1.0);

    // --- Services factor ---
    // Fraction of populated cells covered by health, education, police, fire
    // Sample every 4th cell for performance
    let total_cells = GRID_WIDTH * GRID_HEIGHT;
    let mut health_count = 0u32;
    let mut edu_count = 0u32;
    let mut police_count = 0u32;
    let mut fire_count = 0u32;
    let mut sampled = 0u32;

    for idx in (0..total_cells).step_by(4) {
        sampled += 1;
        let flags = coverage.flags[idx];
        if flags & COVERAGE_HEALTH != 0 {
            health_count += 1;
        }
        if flags & COVERAGE_EDUCATION != 0 {
            edu_count += 1;
        }
        if flags & COVERAGE_POLICE != 0 {
            police_count += 1;
        }
        if flags & COVERAGE_FIRE != 0 {
            fire_count += 1;
        }
    }

    let services_factor = if sampled > 0 {
        let health_cov = health_count as f32 / sampled as f32;
        let edu_cov = edu_count as f32 / sampled as f32;
        let police_cov = police_count as f32 / sampled as f32;
        let fire_cov = fire_count as f32 / sampled as f32;
        ((health_cov + edu_cov + police_cov + fire_cov) / 4.0).clamp(0.0, 1.0)
    } else {
        0.0
    };

    // --- Housing factor ---
    // Available residential capacity vs current occupancy
    // MixedUse buildings also provide residential capacity
    let mut total_res_capacity = 0u32;
    let mut total_res_occupants = 0u32;
    for b in &buildings {
        if b.zone_type.is_residential() || b.zone_type.is_mixed_use() {
            total_res_capacity += b.capacity;
            total_res_occupants += b.occupants;
        }
    }

    let housing_factor = if total_res_capacity > 0 {
        let vacancy_rate = (total_res_capacity.saturating_sub(total_res_occupants)) as f32
            / total_res_capacity as f32;
        // Ideal vacancy is 5-15%. Too low = no room, too high = ghost town.
        // Peak attractiveness at ~10% vacancy
        if vacancy_rate < 0.02 {
            0.1 // Almost no housing available
        } else if vacancy_rate < 0.05 {
            0.3 + (vacancy_rate - 0.02) * 10.0 // 0.3-0.6
        } else if vacancy_rate <= 0.20 {
            0.6 + (0.20 - (vacancy_rate - 0.10).abs()) * 2.0 // peak around 10%
        } else {
            // Too much vacancy suggests abandonment
            (1.0 - (vacancy_rate - 0.20) * 2.0).max(0.2)
        }
        .clamp(0.0, 1.0)
    } else {
        0.0 // No residential buildings at all
    };

    // --- Tax factor ---
    // 10% baseline is neutral (0.5). Lower = more attractive, higher = less.
    let baseline_tax = 0.10;
    let tax_diff = budget.tax_rate - baseline_tax;
    // Each 1% deviation from baseline shifts factor by 0.05
    let tax_factor = (0.5 - tax_diff * 5.0).clamp(0.0, 1.0);

    // --- Overall score ---
    let overall = employment_factor * WEIGHT_EMPLOYMENT
        + happiness_factor * WEIGHT_HAPPINESS
        + services_factor * WEIGHT_SERVICES
        + housing_factor * WEIGHT_HOUSING
        + tax_factor * WEIGHT_TAX;

    attractiveness.overall_score = overall.clamp(0.0, 100.0);
    attractiveness.employment_factor = employment_factor;
    attractiveness.happiness_factor = happiness_factor;
    attractiveness.services_factor = services_factor;
    attractiveness.housing_factor = housing_factor;
    attractiveness.tax_factor = tax_factor;
}

// ---------------------------------------------------------------------------
// System: immigration_wave
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub fn immigration_wave(
    tick: Res<TickCounter>,
    attractiveness: Res<CityAttractiveness>,
    mut commands: Commands,
    mut buildings: Query<(Entity, &mut Building), Without<UnderConstruction>>,
    citizens: Query<(Entity, &CitizenDetails, &HomeLocation), With<Citizen>>,
    mut virtual_pop: ResMut<VirtualPopulation>,
    mut imm_stats: ResMut<ImmigrationStats>,
) {
    if !tick.0.is_multiple_of(IMMIGRATION_INTERVAL) {
        return;
    }

    // Reset monthly stats periodically
    if tick.0.wrapping_sub(imm_stats.last_reset_tick) >= MONTHLY_RESET_INTERVAL {
        imm_stats.immigrants_this_month = 0;
        imm_stats.emigrants_this_month = 0;
        imm_stats.net_migration = 0;
        imm_stats.last_reset_tick = tick.0;
    }

    let score = attractiveness.overall_score;

    // Tick-based pseudo-random: hash the tick counter to get varied spawn counts
    let pseudo_rand = tick_pseudo_random(tick.0);

    if score > 60.0 {
        // Immigration wave
        let (min_families, max_families) = if score > 80.0 {
            (3u32, 10u32) // Boom times
        } else {
            (1u32, 5u32) // Normal attraction
        };
        let range = max_families - min_families + 1;
        let family_count = min_families + (pseudo_rand % range);

        spawn_immigrant_families(
            family_count,
            tick.0,
            &mut commands,
            &mut buildings,
            &mut virtual_pop,
            &mut imm_stats,
        );
    } else if score < 30.0 {
        // Emigration wave
        let (min_leave, max_leave) = if score < 15.0 {
            (5u32, 10u32) // Mass exodus
        } else {
            (1u32, 3u32) // Mild emigration
        };
        let range = max_leave - min_leave + 1;
        let leave_count = min_leave + (pseudo_rand % range);

        remove_unhappiest_citizens(
            leave_count,
            &mut commands,
            &citizens,
            &mut buildings,
            &mut virtual_pop,
            &mut imm_stats,
        );
    }
}

// ---------------------------------------------------------------------------
// Spawn immigrant families
// ---------------------------------------------------------------------------

fn spawn_immigrant_families(
    family_count: u32,
    tick: u64,
    commands: &mut Commands,
    buildings: &mut Query<(Entity, &mut Building), Without<UnderConstruction>>,
    virtual_pop: &mut ResMut<VirtualPopulation>,
    imm_stats: &mut ResMut<ImmigrationStats>,
) {
    // Collect residential buildings with capacity (including MixedUse)
    let homes_with_capacity: Vec<Entity> = buildings
        .iter()
        .filter(|(_, b)| {
            (b.zone_type.is_residential() || b.zone_type.is_mixed_use())
                && b.occupants < b.capacity
        })
        .map(|(e, _)| e)
        .collect();

    if homes_with_capacity.is_empty() {
        return;
    }

    // Collect workplaces with capacity
    let workplaces: Vec<(Entity, usize, usize)> = buildings
        .iter()
        .filter(|(_, b)| b.zone_type.is_job_zone() && b.occupants < b.capacity)
        .map(|(e, b)| (e, b.grid_x, b.grid_y))
        .collect();

    if workplaces.is_empty() {
        return;
    }

    let mut spawned = 0u32;

    for i in 0..family_count {
        if homes_with_capacity.is_empty() {
            break;
        }

        // Pick a home using tick-based pseudo-random
        let home_idx = tick_pseudo_random(tick.wrapping_add(i as u64 * 7)) as usize
            % homes_with_capacity.len();
        let home_entity = homes_with_capacity[home_idx];

        // Check if this home still has capacity
        let (home_gx, home_gy, has_capacity) = {
            if let Ok((_, b)) = buildings.get(home_entity) {
                (b.grid_x, b.grid_y, b.occupants < b.capacity)
            } else {
                continue;
            }
        };

        if !has_capacity {
            continue;
        }

        // Pick a workplace
        let work_idx =
            tick_pseudo_random(tick.wrapping_add(i as u64 * 13 + 3)) as usize % workplaces.len();
        let (work_entity, work_gx, work_gy) = workplaces[work_idx];

        let (home_wx, home_wy) = WorldGrid::grid_to_world(home_gx, home_gy);

        // Generate citizen attributes from tick-based pseudo-random
        let seed = tick.wrapping_add(i as u64 * 31);
        let age = 18 + (tick_pseudo_random(seed) % 47) as u8;
        let gender = if tick_pseudo_random(seed.wrapping_add(1)).is_multiple_of(2) {
            Gender::Male
        } else {
            Gender::Female
        };
        let edu = match age {
            18..=22 => (tick_pseudo_random(seed.wrapping_add(2)) % 2) as u8,
            23..=30 => (tick_pseudo_random(seed.wrapping_add(2)) % 3).min(2) as u8,
            _ => (tick_pseudo_random(seed.wrapping_add(2)) % 4).min(3) as u8,
        };
        let salary = CitizenDetails::base_salary_for_education(edu)
            * (1.0 + age.saturating_sub(18) as f32 * 0.01);

        let pr = |offset: u64| -> f32 {
            (tick_pseudo_random(seed.wrapping_add(offset)) % 90 + 10) as f32 / 100.0
        };

        // Check real citizen cap
        if virtual_pop.max_real_citizens > 0 {
            commands.spawn((
                Citizen,
                Position {
                    x: home_wx,
                    y: home_wy,
                },
                Velocity { x: 0.0, y: 0.0 },
                HomeLocation {
                    grid_x: home_gx,
                    grid_y: home_gy,
                    building: home_entity,
                },
                WorkLocation {
                    grid_x: work_gx,
                    grid_y: work_gy,
                    building: work_entity,
                },
                CitizenStateComp(CitizenState::AtHome),
                PathCache::new(Vec::new()),
                CitizenDetails {
                    age,
                    gender,
                    education: edu,
                    happiness: 55.0, // Immigrants start slightly above neutral
                    health: 80.0 + (tick_pseudo_random(seed.wrapping_add(10)) % 20) as f32,
                    salary,
                    savings: salary
                        * (1.0 + (tick_pseudo_random(seed.wrapping_add(11)) % 30) as f32 / 10.0),
                },
                Personality {
                    ambition: pr(20),
                    sociability: pr(21),
                    materialism: pr(22),
                    resilience: pr(23),
                },
                Needs::default(),
                Family::default(),
                ActivityTimer::default(),
            ));
        }

        // Update building occupancy
        if let Ok((_, mut home_b)) = buildings.get_mut(home_entity) {
            home_b.occupants += 1;
        }
        if let Ok((_, mut work_b)) = buildings.get_mut(work_entity) {
            work_b.occupants += 1;
        }

        spawned += 1;
    }

    imm_stats.immigrants_this_month += spawned;
    imm_stats.net_migration += spawned as i32;
}

// ---------------------------------------------------------------------------
// Remove unhappiest citizens (emigration)
// ---------------------------------------------------------------------------

fn remove_unhappiest_citizens(
    count: u32,
    commands: &mut Commands,
    citizens: &Query<(Entity, &CitizenDetails, &HomeLocation), With<Citizen>>,
    buildings: &mut Query<(Entity, &mut Building), Without<UnderConstruction>>,
    virtual_pop: &mut ResMut<VirtualPopulation>,
    imm_stats: &mut ResMut<ImmigrationStats>,
) {
    // Collect all citizens sorted by happiness ascending (unhappiest first)
    let mut sorted_citizens: Vec<(Entity, f32, Entity)> = citizens
        .iter()
        .map(|(entity, details, home)| (entity, details.happiness, home.building))
        .collect();

    sorted_citizens.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    let mut removed = 0u32;

    for (entity, _happiness, home_building) in sorted_citizens.iter() {
        if removed >= count {
            break;
        }

        if let Ok((_, mut building)) = buildings.get_mut(*home_building) {
            building.occupants = building.occupants.saturating_sub(1);
        }

        virtual_pop.total_virtual = virtual_pop.total_virtual.saturating_sub(1);
        commands.entity(*entity).despawn();
        removed += 1;
    }

    imm_stats.emigrants_this_month += removed;
    imm_stats.net_migration -= removed as i32;
}

// ---------------------------------------------------------------------------
// Tick-based pseudo-random number generator
// ---------------------------------------------------------------------------

/// Simple hash-based pseudo-random from a tick value.
/// Returns a u32 suitable for modulo operations.
fn tick_pseudo_random(tick: u64) -> u32 {
    // Mix bits using a simple multiplicative hash (splitmix-inspired)
    let mut x = tick.wrapping_mul(0x9E3779B97F4A7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D049BB133111EB);
    x = x ^ (x >> 31);
    x as u32
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_attractiveness() {
        let attr = CityAttractiveness::default();
        assert!((attr.overall_score - 50.0).abs() < 0.01);
        assert!((attr.employment_factor - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_default_immigration_stats() {
        let stats = ImmigrationStats::default();
        assert_eq!(stats.immigrants_this_month, 0);
        assert_eq!(stats.emigrants_this_month, 0);
        assert_eq!(stats.net_migration, 0);
    }

    #[test]
    fn test_tick_pseudo_random_deterministic() {
        // Same tick should produce same result
        assert_eq!(tick_pseudo_random(42), tick_pseudo_random(42));
    }

    #[test]
    fn test_tick_pseudo_random_varies() {
        // Different ticks should produce different results
        let a = tick_pseudo_random(100);
        let b = tick_pseudo_random(101);
        let c = tick_pseudo_random(102);
        // Extremely unlikely all three are equal
        assert!(a != b || b != c);
    }

    #[test]
    fn test_tick_pseudo_random_distribution() {
        // Check that modulo 10 produces a roughly even distribution
        let mut buckets = [0u32; 10];
        for i in 0..1000u64 {
            let val = tick_pseudo_random(i) % 10;
            buckets[val as usize] += 1;
        }
        // Each bucket should have roughly 100 (+/- 50 for statistical noise)
        for &count in &buckets {
            assert!(count > 50, "bucket too low: {}", count);
            assert!(count < 200, "bucket too high: {}", count);
        }
    }

    #[test]
    fn test_weight_sum() {
        // Weights should sum to 100
        let total =
            WEIGHT_EMPLOYMENT + WEIGHT_HAPPINESS + WEIGHT_SERVICES + WEIGHT_HOUSING + WEIGHT_TAX;
        assert!((total - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_max_attractiveness() {
        // All factors at 1.0 should yield score of 100
        let score = 1.0 * WEIGHT_EMPLOYMENT
            + 1.0 * WEIGHT_HAPPINESS
            + 1.0 * WEIGHT_SERVICES
            + 1.0 * WEIGHT_HOUSING
            + 1.0 * WEIGHT_TAX;
        assert!((score - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_min_attractiveness() {
        // All factors at 0.0 should yield score of 0
        let score = 0.0 * WEIGHT_EMPLOYMENT
            + 0.0 * WEIGHT_HAPPINESS
            + 0.0 * WEIGHT_SERVICES
            + 0.0 * WEIGHT_HOUSING
            + 0.0 * WEIGHT_TAX;
        assert!((score - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_tax_factor_baseline() {
        // At 10% tax rate (baseline), factor should be 0.5
        let baseline_tax = 0.10f32;
        let tax_diff = baseline_tax - 0.10;
        let factor = (0.5 - tax_diff * 5.0).clamp(0.0, 1.0);
        assert!((factor - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_tax_factor_low_tax() {
        // At 0% tax, factor should be 1.0
        let tax_rate = 0.0f32;
        let tax_diff = tax_rate - 0.10;
        let factor = (0.5 - tax_diff * 5.0).clamp(0.0, 1.0);
        assert!((factor - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_tax_factor_high_tax() {
        // At 20% tax, factor should be 0.0
        let tax_rate = 0.20f32;
        let tax_diff = tax_rate - 0.10;
        let factor = (0.5 - tax_diff * 5.0).clamp(0.0, 1.0);
        assert!((factor - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_employment_factor() {
        // 0% unemployment -> 1.0
        let factor: f32 = (1.0 - 0.0f32 * 5.0).clamp(0.0, 1.0);
        assert!((factor - 1.0).abs() < 0.01);

        // 10% unemployment -> 0.5
        let factor: f32 = (1.0 - 0.10f32 * 5.0).clamp(0.0, 1.0);
        assert!((factor - 0.5).abs() < 0.01);

        // 20%+ unemployment -> 0.0
        let factor: f32 = (1.0 - 0.20f32 * 5.0).clamp(0.0, 1.0);
        assert!((factor - 0.0).abs() < 0.01);
    }
}
