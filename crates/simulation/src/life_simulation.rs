use bevy::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::buildings::Building;
use crate::citizen::{
    Citizen, CitizenDetails, CitizenState, CitizenStateComp, Family, Gender, HomeLocation, Needs,
    PathCache, Personality, Position, Velocity, WorkLocation,
};
use crate::education::EducationGrid;
use crate::grid::{WorldGrid, ZoneType};
use crate::time_of_day::GameClock;

// ---------------------------------------------------------------------------
// Timers
// ---------------------------------------------------------------------------

#[derive(Resource, Default, Serialize, Deserialize, Clone, Debug)]
pub struct LifeSimTimer {
    pub needs_tick: u32,
    pub life_event_tick: u32,
    pub salary_tick: u32,
    pub education_tick: u32,
    pub job_seek_tick: u32,
    pub personality_tick: u32,
    pub health_tick: u32,
}

const NEEDS_INTERVAL: u32 = 10; // every 10 ticks (~1 game minute)
const LIFE_EVENT_INTERVAL: u32 = 600; // every 600 ticks (~1 game hour)
const SALARY_INTERVAL: u32 = 43200; // every 43200 ticks (~30 game days)
const EDUCATION_INTERVAL: u32 = 1440; // every 1440 ticks (~1 game day)
const JOB_SEEK_INTERVAL: u32 = 300; // every 300 ticks (~30 game minutes)
const PERSONALITY_INTERVAL: u32 = 2880; // every 2880 ticks (~2 game days)
const HEALTH_INTERVAL: u32 = 1440; // every 1440 ticks (~1 game day)

// ---------------------------------------------------------------------------
// Needs decay/fulfillment rates (per NEEDS_INTERVAL = 10 ticks)
// ---------------------------------------------------------------------------

// Decay rates (per interval, while active)
const HUNGER_DECAY: f32 = 2.1; // empty in ~8h
const ENERGY_DECAY: f32 = 1.0; // empty in ~16h
const SOCIAL_DECAY: f32 = 0.23; // empty in ~3 days
const FUN_DECAY: f32 = 0.35; // empty in ~2 days

// Restoration rates (per interval, at appropriate activity)
const HUNGER_RESTORE_HOME: f32 = 8.0;
const HUNGER_RESTORE_SHOP: f32 = 5.0;
const ENERGY_RESTORE_HOME_NIGHT: f32 = 4.0;
const ENERGY_RESTORE_HOME_DAY: f32 = 1.5;
const SOCIAL_RESTORE_WORK: f32 = 0.5;
const SOCIAL_RESTORE_LEISURE: f32 = 3.0;
const SOCIAL_RESTORE_SCHOOL: f32 = 2.0;
const FUN_RESTORE_LEISURE: f32 = 5.0;
const FUN_RESTORE_SHOP: f32 = 1.5;
const FUN_DRAIN_WORK: f32 = 0.3; // extra fun drain while working

// ---------------------------------------------------------------------------
// System: update_needs
// ---------------------------------------------------------------------------

pub fn update_needs(
    clock: Res<GameClock>,
    mut timer: ResMut<LifeSimTimer>,
    grid: Res<WorldGrid>,
    mut citizens: Query<
        (
            &CitizenStateComp,
            &mut Needs,
            &HomeLocation,
            &CitizenDetails,
        ),
        With<Citizen>,
    >,
) {
    if clock.paused {
        return;
    }
    timer.needs_tick += 1;
    if timer.needs_tick < NEEDS_INTERVAL {
        return;
    }
    timer.needs_tick = 0;

    let is_night = clock.hour < 6.0 || clock.hour >= 22.0;

    for (state, mut needs, home, _details) in &mut citizens {
        // --- Decay ---
        needs.hunger = (needs.hunger - HUNGER_DECAY).max(0.0);
        needs.energy = (needs.energy - ENERGY_DECAY).max(0.0);
        needs.social = (needs.social - SOCIAL_DECAY).max(0.0);
        needs.fun = (needs.fun - FUN_DECAY).max(0.0);

        // --- Fulfillment based on current activity ---
        match state.0 {
            CitizenState::AtHome => {
                // Eating restores hunger
                if needs.hunger < 80.0 {
                    needs.hunger = (needs.hunger + HUNGER_RESTORE_HOME).min(100.0);
                }
                // Resting restores energy
                if is_night {
                    needs.energy = (needs.energy + ENERGY_RESTORE_HOME_NIGHT).min(100.0);
                } else {
                    needs.energy = (needs.energy + ENERGY_RESTORE_HOME_DAY).min(100.0);
                }
            }
            CitizenState::Working => {
                needs.fun = (needs.fun - FUN_DRAIN_WORK).max(0.0);
                needs.social = (needs.social + SOCIAL_RESTORE_WORK).min(100.0);
            }
            CitizenState::Shopping => {
                needs.hunger = (needs.hunger + HUNGER_RESTORE_SHOP).min(100.0);
                needs.fun = (needs.fun + FUN_RESTORE_SHOP).min(100.0);
            }
            CitizenState::AtLeisure => {
                needs.fun = (needs.fun + FUN_RESTORE_LEISURE).min(100.0);
                needs.social = (needs.social + SOCIAL_RESTORE_LEISURE).min(100.0);
            }
            CitizenState::AtSchool => {
                needs.social = (needs.social + SOCIAL_RESTORE_SCHOOL).min(100.0);
            }
            _ => {} // commuting states: just decay
        }

        // --- Comfort: based on housing quality ---
        let home_cell = grid.get(home.grid_x, home.grid_y);
        let mut comfort = 40.0; // base
        if home_cell.has_power {
            comfort += 20.0;
        }
        if home_cell.has_water {
            comfort += 20.0;
        }
        // Smooth towards target comfort (don't snap)
        needs.comfort += (comfort - needs.comfort) * 0.1;
        needs.comfort = needs.comfort.clamp(0.0, 100.0);
    }
}

// ---------------------------------------------------------------------------
// System: education_advancement
// Citizens gain education from nearby schools over time.
// ---------------------------------------------------------------------------

pub fn education_advancement(
    clock: Res<GameClock>,
    mut timer: ResMut<LifeSimTimer>,
    edu_grid: Res<EducationGrid>,
    mut citizens: Query<(&mut CitizenDetails, &HomeLocation), With<Citizen>>,
) {
    if clock.paused {
        return;
    }
    timer.education_tick += 1;
    if timer.education_tick < EDUCATION_INTERVAL {
        return;
    }
    timer.education_tick = 0;

    for (mut details, home) in &mut citizens {
        let available_level = edu_grid.get(home.grid_x, home.grid_y);
        if available_level <= details.education {
            continue;
        }

        // Age requirements for education levels
        let can_advance = match available_level {
            1 => details.age >= 6 && details.age <= 30,  // Elementary
            2 => details.age >= 12 && details.age <= 35, // High School
            3 => details.age >= 18 && details.age <= 40, // University
            _ => false,
        };

        if can_advance {
            details.education = available_level;
            // Update salary based on new education
            details.salary = CitizenDetails::base_salary_for_education(details.education);
        }
    }
}

// ---------------------------------------------------------------------------
// System: salary_payment
// Monthly salary deposits into savings.
// ---------------------------------------------------------------------------

pub fn salary_payment(
    clock: Res<GameClock>,
    mut timer: ResMut<LifeSimTimer>,
    mut citizens: Query<(&mut CitizenDetails, Option<&WorkLocation>), With<Citizen>>,
) {
    if clock.paused {
        return;
    }
    timer.salary_tick += 1;
    if timer.salary_tick < SALARY_INTERVAL {
        return;
    }
    timer.salary_tick = 0;

    for (mut details, work) in &mut citizens {
        if work.is_some() && details.life_stage().can_work() {
            // Monthly income
            details.savings += details.salary;
            // Monthly expenses (rent, food, etc.) -- roughly 70% of salary
            details.savings -= details.salary * 0.7;
            details.savings = details.savings.max(0.0);
        }
    }
}

// ---------------------------------------------------------------------------
// System: job_seeking
// Unemployed citizens look for work.
// ---------------------------------------------------------------------------

#[allow(clippy::type_complexity)]
pub fn job_seeking(
    clock: Res<GameClock>,
    mut timer: ResMut<LifeSimTimer>,
    mut commands: Commands,
    citizens_without_work: Query<
        (Entity, &CitizenDetails, &HomeLocation),
        (With<Citizen>, Without<WorkLocation>),
    >,
    mut buildings: Query<(Entity, &mut Building)>,
) {
    if clock.paused {
        return;
    }
    timer.job_seek_tick += 1;
    if timer.job_seek_tick < JOB_SEEK_INTERVAL {
        return;
    }
    timer.job_seek_tick = 0;

    let mut rng = rand::thread_rng();

    // Collect available workplaces with mutable remaining slot counts.
    // The remaining_slots field is decremented as citizens are assigned
    // within this tick to prevent overfilling beyond capacity.
    let mut available_jobs: Vec<(Entity, usize, usize, ZoneType, u32)> = buildings
        .iter()
        .filter(|(_, b)| b.zone_type.is_job_zone() && b.occupants < b.capacity)
        .map(|(e, b)| (e, b.grid_x, b.grid_y, b.zone_type, b.capacity - b.occupants))
        .collect();

    if available_jobs.is_empty() {
        return;
    }

    let mut placed = 0u32;
    for (citizen_entity, details, home) in &citizens_without_work {
        if !details.life_stage().can_work() {
            continue;
        }
        if placed >= 50 {
            break; // limit per tick
        }

        // Find best matching job based on education
        let best_job = find_matching_job(
            &available_jobs,
            details.education,
            home.grid_x,
            home.grid_y,
            &mut rng,
        );

        if let Some((job_entity, job_gx, job_gy)) = best_job {
            commands.entity(citizen_entity).insert(WorkLocation {
                grid_x: job_gx,
                grid_y: job_gy,
                building: job_entity,
            });
            if let Ok((_, mut building)) = buildings.get_mut(job_entity) {
                building.occupants += 1;
            }

            // Decrement remaining slots in the snapshot to prevent
            // assigning more citizens than capacity in a single tick.
            if let Some(job) = available_jobs
                .iter_mut()
                .find(|(e, _, _, _, _)| *e == job_entity)
            {
                job.4 = job.4.saturating_sub(1);
                // Remove fully-filled jobs so they are not considered again
                if job.4 == 0 {
                    available_jobs.retain(|(e, _, _, _, _)| *e != job_entity);
                }
            }

            // Set salary based on education and job match
            commands.entity(citizen_entity).insert(CitizenDetails {
                salary: calculate_salary(details.education, details.age),
                ..details.clone()
            });

            placed += 1;
        }
    }
}

fn find_matching_job(
    jobs: &[(Entity, usize, usize, ZoneType, u32)],
    education: u8,
    home_x: usize,
    home_y: usize,
    rng: &mut impl Rng,
) -> Option<(Entity, usize, usize)> {
    // Preferred zone types based on education
    let preferred: &[ZoneType] = match education {
        0 => &[ZoneType::Industrial],
        1 => &[ZoneType::Industrial, ZoneType::CommercialLow],
        2 => &[
            ZoneType::CommercialLow,
            ZoneType::CommercialHigh,
            ZoneType::Industrial,
        ],
        _ => &[ZoneType::Office, ZoneType::CommercialHigh],
    };

    // First try preferred zones, sorted by distance (only those with remaining slots)
    let mut candidates: Vec<_> = jobs
        .iter()
        .filter(|(_, _, _, zt, remaining)| *remaining > 0 && preferred.contains(zt))
        .collect();

    if candidates.is_empty() {
        // Fall back to any available job with remaining slots
        candidates = jobs
            .iter()
            .filter(|(_, _, _, _, remaining)| *remaining > 0)
            .collect();
    }

    if candidates.is_empty() {
        return None;
    }

    // Sort by distance to home (prefer nearby jobs)
    candidates.sort_by_key(|(_, gx, gy, _, _)| {
        let dx = (*gx as i32 - home_x as i32).abs();
        let dy = (*gy as i32 - home_y as i32).abs();
        dx + dy
    });

    // Pick from top 5 nearest (with some randomness)
    let pick = rng.gen_range(0..candidates.len().min(5));
    let &(entity, gx, gy, _, _) = candidates[pick];
    Some((entity, gx, gy))
}

fn calculate_salary(education: u8, age: u8) -> f32 {
    let base = CitizenDetails::base_salary_for_education(education);
    // Seniority bonus: +1% per year of experience (starting from age 18)
    let experience_years = age.saturating_sub(18) as f32;
    let seniority = 1.0 + (experience_years * 0.01).min(0.5); // max +50%
    base * seniority
}

// ---------------------------------------------------------------------------
// System: life_events
// Handles marriage, births, and major life changes.
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub fn life_events(
    clock: Res<GameClock>,
    mut timer: ResMut<LifeSimTimer>,
    mut commands: Commands,
    mut citizens: Query<
        (
            Entity,
            &mut CitizenDetails,
            &mut Family,
            &HomeLocation,
            &Personality,
        ),
        With<Citizen>,
    >,
    mut buildings: Query<&mut Building>,
) {
    if clock.paused {
        return;
    }
    timer.life_event_tick += 1;
    if timer.life_event_tick < LIFE_EVENT_INTERVAL {
        return;
    }
    timer.life_event_tick = 0;

    let mut rng = rand::thread_rng();

    // Collect single adults in the same building for potential marriage
    let mut singles_by_building: std::collections::HashMap<Entity, Vec<(Entity, Gender, u8, f32)>> =
        std::collections::HashMap::new();

    for (entity, details, family, home, _personality) in &citizens {
        if family.partner.is_some() {
            continue;
        }
        if details.age < 20 || details.age > 55 {
            continue;
        }
        singles_by_building.entry(home.building).or_default().push((
            entity,
            details.gender,
            details.age,
            details.happiness,
        ));
    }

    // --- Marriage ---
    // Track matched entities to enforce one-to-one pairing within a single tick.
    // Without this, the same female could be paired with multiple males.
    let mut matched: std::collections::HashSet<Entity> = std::collections::HashSet::new();
    let mut marriages: Vec<(Entity, Entity)> = Vec::new();
    for singles in singles_by_building.values() {
        let males: Vec<_> = singles.iter().filter(|s| s.1 == Gender::Male).collect();
        let females: Vec<_> = singles.iter().filter(|s| s.1 == Gender::Female).collect();

        for &&(m_entity, _, m_age, m_happy) in &males {
            if matched.contains(&m_entity) {
                continue;
            }
            if m_happy < 30.0 {
                continue; // too unhappy to marry
            }
            for &&(f_entity, _, f_age, f_happy) in &females {
                if matched.contains(&f_entity) {
                    continue;
                }
                if f_happy < 30.0 {
                    continue;
                }
                // Age compatibility: within 10 years
                let age_diff = (m_age as i32 - f_age as i32).unsigned_abs();
                if age_diff > 10 {
                    continue;
                }
                // Marriage probability
                if rng.gen::<f32>() < 0.05 {
                    matched.insert(m_entity);
                    matched.insert(f_entity);
                    marriages.push((m_entity, f_entity));
                    break; // one marriage per male per cycle
                }
            }
        }
    }

    for (m, f) in &marriages {
        if let Ok([(_, _, mut m_family, _, _), (_, _, mut f_family, _, _)]) =
            citizens.get_many_mut([*m, *f])
        {
            m_family.partner = Some(*f);
            f_family.partner = Some(*m);
        }
    }

    // --- Births ---
    // Collect couples where female is 20-38, have < 3 children
    let mut births: Vec<(Entity, Entity, usize, usize)> = Vec::new(); // (parent_entity, home_building, gx, gy)

    for (entity, details, family, home, personality) in &citizens {
        if details.gender != Gender::Female {
            continue;
        }
        if family.partner.is_none() {
            continue;
        }
        if details.age < 20 || details.age > 38 {
            continue;
        }
        if family.children.len() >= 3 {
            continue;
        }

        // Birth probability: influenced by sociability and age
        let age_factor = 1.0 - ((details.age as f32 - 25.0).abs() / 15.0).min(1.0);
        let prob = 0.02 * personality.sociability * age_factor;
        if rng.gen::<f32>() < prob {
            births.push((entity, home.building, home.grid_x, home.grid_y));
        }
    }

    for (parent_entity, home_building, home_gx, home_gy) in &births {
        // Check building has capacity
        if let Ok(mut building) = buildings.get_mut(*home_building) {
            if building.occupants >= building.capacity {
                continue;
            }
            building.occupants += 1;
        } else {
            continue;
        }

        let gender = if rng.gen::<f32>() < 0.5 {
            Gender::Male
        } else {
            Gender::Female
        };

        let (wx, wy) = WorldGrid::grid_to_world(*home_gx, *home_gy);

        let child = commands
            .spawn((
                Citizen,
                Position { x: wx, y: wy },
                Velocity { x: 0.0, y: 0.0 },
                HomeLocation {
                    grid_x: *home_gx,
                    grid_y: *home_gy,
                    building: *home_building,
                },
                CitizenStateComp(CitizenState::AtHome),
                PathCache::new(Vec::new()),
                CitizenDetails {
                    age: 0,
                    gender,
                    education: 0,
                    happiness: 80.0,
                    health: 100.0,
                    salary: 0.0,
                    savings: 0.0,
                },
                Personality::random(&mut rng),
                Needs {
                    hunger: 100.0,
                    energy: 100.0,
                    social: 100.0,
                    fun: 100.0,
                    comfort: 80.0,
                },
                Family {
                    parent: Some(*parent_entity),
                    ..Default::default()
                },
            ))
            .id();

        // Add child to parent's family
        if let Ok((_, _, mut parent_family, _, _)) = citizens.get_mut(*parent_entity) {
            parent_family.children.push(child);
        }
        // Also add to partner's family
        if let Ok((_, _, parent_family, _, _)) = citizens.get(*parent_entity) {
            if let Some(partner) = parent_family.partner {
                if let Ok((_, _, mut partner_family, _, _)) = citizens.get_mut(partner) {
                    partner_family.children.push(child);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// System: retire_workers
// Citizens who reach retirement age lose their WorkLocation.
// ---------------------------------------------------------------------------

pub fn retire_workers(
    clock: Res<GameClock>,
    slow_tick: Res<crate::SlowTickTimer>,
    mut commands: Commands,
    citizens: Query<(Entity, &CitizenDetails, &WorkLocation), With<Citizen>>,
    mut buildings: Query<&mut Building>,
) {
    if clock.paused {
        return;
    }
    if !slow_tick.should_run() {
        return;
    }

    for (entity, details, work) in &citizens {
        if details.age >= 65 {
            if let Ok(mut building) = buildings.get_mut(work.building) {
                building.occupants = building.occupants.saturating_sub(1);
            }
            commands.entity(entity).remove::<WorkLocation>();
        }
    }
}

// ---------------------------------------------------------------------------
// System: evolve_personality
// Life experiences shape personality over time.
// Repeated failure erodes ambition. Loneliness reduces sociability.
// Success reinforces traits. Hardship builds resilience (or breaks it).
// ---------------------------------------------------------------------------

#[allow(clippy::type_complexity)]
pub fn evolve_personality(
    clock: Res<GameClock>,
    mut timer: ResMut<LifeSimTimer>,
    mut citizens: Query<
        (
            &mut Personality,
            &CitizenDetails,
            &Needs,
            Option<&WorkLocation>,
            &Family,
        ),
        With<Citizen>,
    >,
) {
    if clock.paused {
        return;
    }
    timer.personality_tick += 1;
    if timer.personality_tick < PERSONALITY_INTERVAL {
        return;
    }
    timer.personality_tick = 0;

    // Small drift per cycle (personality changes slowly)
    const DRIFT: f32 = 0.01;

    for (mut personality, details, needs, work, family) in &mut citizens {
        // --- Ambition ---
        // Unemployed working-age adult: ambition erodes from repeated failure
        if details.life_stage().can_work() && work.is_none() {
            personality.ambition = (personality.ambition - DRIFT * 2.0).max(0.05);
        }
        // Well-educated + employed: ambition reinforced
        if work.is_some() && details.education >= 2 {
            personality.ambition = (personality.ambition + DRIFT * 0.5).min(1.0);
        }
        // Low happiness for extended periods: ambition fades
        if details.happiness < 30.0 {
            personality.ambition = (personality.ambition - DRIFT).max(0.05);
        }

        // --- Sociability ---
        // Chronically lonely (low social need): sociability decreases (withdrawal)
        if needs.social < 20.0 {
            personality.sociability = (personality.sociability - DRIFT).max(0.05);
        }
        // Has partner and children: sociability reinforced
        if family.partner.is_some() && !family.children.is_empty() {
            personality.sociability = (personality.sociability + DRIFT * 0.5).min(1.0);
        }
        // Isolated (no partner, no children): sociability slowly fades
        if family.partner.is_none() && family.children.is_empty() && details.age > 35 {
            personality.sociability = (personality.sociability - DRIFT * 0.3).max(0.05);
        }

        // --- Materialism ---
        // High savings: materialism increases (got used to comfort)
        if details.savings > details.salary * 12.0 {
            personality.materialism = (personality.materialism + DRIFT * 0.3).min(1.0);
        }
        // Low savings / low salary: materialism can go either way
        // but generally increases (want what you don't have)
        if details.savings < details.salary && details.life_stage().can_work() {
            personality.materialism = (personality.materialism + DRIFT * 0.5).min(1.0);
        }

        // --- Resilience ---
        // Surviving hardship builds resilience (low happiness but still here)
        if details.happiness < 40.0 && details.health > 50.0 {
            personality.resilience = (personality.resilience + DRIFT * 0.5).min(1.0);
        }
        // Prolonged comfort weakens resilience
        if details.happiness > 80.0 && needs.overall_satisfaction() > 0.8 {
            personality.resilience = (personality.resilience - DRIFT * 0.2).max(0.05);
        }
        // Very poor health breaks resilience
        if details.health < 30.0 {
            personality.resilience = (personality.resilience - DRIFT).max(0.05);
        }

        // Aging naturally shifts personality
        if details.age > 55 {
            // Older people tend to become less ambitious, more resilient
            personality.ambition = (personality.ambition - DRIFT * 0.2).max(0.05);
            personality.resilience = (personality.resilience + DRIFT * 0.3).min(1.0);
        }
    }
}

// ---------------------------------------------------------------------------
// System: update_health
// Health changes based on age, needs, pollution, services, and lifestyle.
// ---------------------------------------------------------------------------

pub fn update_health(
    clock: Res<GameClock>,
    mut timer: ResMut<LifeSimTimer>,
    grid: Res<WorldGrid>,
    pollution_grid: Res<crate::pollution::PollutionGrid>,
    coverage: Res<crate::happiness::ServiceCoverageGrid>,
    mut citizens: Query<(&mut CitizenDetails, &HomeLocation, &Needs, &Personality), With<Citizen>>,
) {
    if clock.paused {
        return;
    }
    timer.health_tick += 1;
    if timer.health_tick < HEALTH_INTERVAL {
        return;
    }
    timer.health_tick = 0;

    for (mut details, home, needs, personality) in &mut citizens {
        let mut health_delta: f32 = 0.0;

        // Natural aging: health degrades over time, especially after 50
        if details.age > 50 {
            health_delta -= (details.age as f32 - 50.0) * 0.02;
        }

        // Starvation (very low hunger) damages health
        if needs.hunger < 15.0 {
            health_delta -= 2.0;
        } else if needs.hunger < 30.0 {
            health_delta -= 0.5;
        }

        // Exhaustion (very low energy) damages health
        if needs.energy < 15.0 {
            health_delta -= 1.5;
        }

        // Good needs satisfaction helps recovery
        if needs.overall_satisfaction() > 0.7 {
            health_delta += 0.5;
        }

        // Pollution exposure at home
        let poll = pollution_grid.get(home.grid_x, home.grid_y) as f32;
        if poll > 50.0 {
            health_delta -= (poll - 50.0) * 0.02;
        }

        // Healthcare coverage helps recovery
        let idx = home.grid_y * grid.width + home.grid_x;
        if coverage.has_health(idx) {
            health_delta += 0.3;
            // Healthcare helps more when sick
            if details.health < 50.0 {
                health_delta += 0.5;
            }
        }

        // Resilient personalities resist health decline
        health_delta *= 1.0 - (personality.resilience * 0.3);

        details.health = (details.health + health_delta).clamp(0.0, 100.0);

        // Very low health increases unhappiness (handled in happiness system via needs)
        // and eventually leads to death (handled in lifecycle)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_salary_calculation() {
        let salary_18 = calculate_salary(0, 18);
        let salary_40 = calculate_salary(0, 40);
        assert!(salary_40 > salary_18, "seniority should increase salary");

        let salary_uni = calculate_salary(3, 30);
        let salary_none = calculate_salary(0, 30);
        assert!(salary_uni > salary_none, "education should increase salary");
    }

    #[test]
    fn test_find_matching_job_prefers_education() {
        let jobs = vec![
            (Entity::from_raw(1), 10, 10, ZoneType::Industrial, 5),
            (Entity::from_raw(2), 10, 10, ZoneType::Office, 5),
        ];
        let mut rng = rand::thread_rng();

        // University grad should prefer office
        let result = find_matching_job(&jobs, 3, 10, 10, &mut rng);
        assert!(result.is_some());
        assert_eq!(result.unwrap().0, Entity::from_raw(2));

        // No education should prefer industrial
        let result = find_matching_job(&jobs, 0, 10, 10, &mut rng);
        assert!(result.is_some());
        assert_eq!(result.unwrap().0, Entity::from_raw(1));
    }
}

pub struct LifeSimulationPlugin;

impl Plugin for LifeSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LifeSimTimer>()
            .add_systems(
                FixedUpdate,
                (
                    update_needs,
                    education_advancement,
                    salary_payment,
                    job_seeking,
                    life_events,
                    retire_workers,
                )
                    .after(crate::happiness::update_happiness),
            )
            .add_systems(
                FixedUpdate,
                (evolve_personality, update_health).after(update_needs),
            );
    }
}
