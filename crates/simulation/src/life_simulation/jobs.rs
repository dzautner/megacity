use bevy::prelude::*;
use rand::Rng;

use crate::buildings::Building;
use crate::citizen::{Citizen, CitizenDetails, HomeLocation, WorkLocation};
use crate::education::EducationGrid;
use crate::grid::ZoneType;
use crate::time_of_day::GameClock;

use super::LifeSimTimer;

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
    if timer.education_tick < super::EDUCATION_INTERVAL {
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
    if timer.salary_tick < super::SALARY_INTERVAL {
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
    if timer.job_seek_tick < super::JOB_SEEK_INTERVAL {
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

pub(crate) fn find_matching_job(
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

pub(crate) fn calculate_salary(education: u8, age: u8) -> f32 {
    let base = CitizenDetails::base_salary_for_education(education);
    // Seniority bonus: +1% per year of experience (starting from age 18)
    let experience_years = age.saturating_sub(18) as f32;
    let seniority = 1.0 + (experience_years * 0.01).min(0.5); // max +50%
    base * seniority
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
