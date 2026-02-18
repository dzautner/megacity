use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::buildings::Building;
use crate::citizen::{Citizen, CitizenDetails, HomeLocation, WorkLocation};
use crate::grid::ZoneType;
use crate::TickCounter;

// ---------------------------------------------------------------------------
// Job types and requirements
// ---------------------------------------------------------------------------

/// Requirements for a job slot: minimum education level, optional maximum,
/// and a salary multiplier applied on top of the citizen's base salary.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct JobRequirement {
    pub min_education: u8,
    pub max_education: Option<u8>,
    pub salary_multiplier: f32,
}

/// Broad categories of jobs available in the city.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum JobType {
    /// Manual labor, warehousing, basic assembly (education 0)
    Unskilled,
    /// Retail, food service, customer-facing roles (education 0-1)
    Service,
    /// Technicians, skilled trades, supervisors (education 1-2)
    Skilled,
    /// Engineers, accountants, managers (education 2-3)
    Professional,
    /// C-suite, directors, senior partners (education 3+)
    Executive,
}

impl JobType {
    /// The job requirement associated with this job type.
    pub fn requirement(self) -> JobRequirement {
        match self {
            JobType::Unskilled => JobRequirement {
                min_education: 0,
                max_education: None,
                salary_multiplier: 0.8,
            },
            JobType::Service => JobRequirement {
                min_education: 0,
                max_education: Some(1),
                salary_multiplier: 1.0,
            },
            JobType::Skilled => JobRequirement {
                min_education: 1,
                max_education: Some(2),
                salary_multiplier: 1.3,
            },
            JobType::Professional => JobRequirement {
                min_education: 2,
                max_education: Some(3),
                salary_multiplier: 1.8,
            },
            JobType::Executive => JobRequirement {
                min_education: 3,
                max_education: None,
                salary_multiplier: 2.5,
            },
        }
    }

    /// All job types for iteration.
    pub fn all() -> &'static [JobType] {
        &[
            JobType::Unskilled,
            JobType::Service,
            JobType::Skilled,
            JobType::Professional,
            JobType::Executive,
        ]
    }

    /// Display name for the UI.
    pub fn name(self) -> &'static str {
        match self {
            JobType::Unskilled => "Unskilled",
            JobType::Service => "Service",
            JobType::Skilled => "Skilled",
            JobType::Professional => "Professional",
            JobType::Executive => "Executive",
        }
    }
}

// ---------------------------------------------------------------------------
// Job slot and workplace details component
// ---------------------------------------------------------------------------

/// A single slot in a workplace that can hold one worker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSlot {
    pub filled: bool,
    pub worker_entity: Option<Entity>,
    pub education_req: u8,
    pub job_type: JobType,
}

/// Attached to non-residential buildings to describe the types of jobs offered.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct WorkplaceDetails {
    pub job_type: JobType,
    pub job_slots: Vec<JobSlot>,
    pub filled_slots: u32,
    pub required_education: u8,
}

impl WorkplaceDetails {
    /// Count how many unfilled slots match a given education level.
    pub fn available_slots_for_education(&self, education: u8) -> u32 {
        self.job_slots
            .iter()
            .filter(|s| !s.filled && education >= s.education_req)
            .count() as u32
    }
}

// ---------------------------------------------------------------------------
// Employment statistics resource
// ---------------------------------------------------------------------------

/// City-wide employment statistics, updated by the job_matching system.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct EmploymentStats {
    pub total_employed: u32,
    pub total_unemployed: u32,
    pub unemployment_rate: f32,
    /// (filled, total) slots per job type.
    pub jobs_by_type: HashMap<JobType, (u32, u32)>,
}

impl Default for EmploymentStats {
    fn default() -> Self {
        let mut jobs_by_type = HashMap::new();
        for &jt in JobType::all() {
            jobs_by_type.insert(jt, (0, 0));
        }
        Self {
            total_employed: 0,
            total_unemployed: 0,
            unemployment_rate: 0.0,
            jobs_by_type,
        }
    }
}

// ---------------------------------------------------------------------------
// System: assign_workplace_details
// Runs once per newly-spawned Building to attach WorkplaceDetails.
// ---------------------------------------------------------------------------

/// Generates the appropriate job slot distribution for a given zone type
/// and building capacity.
fn generate_slots(zone_type: ZoneType, capacity: u32) -> Option<(JobType, Vec<JobSlot>)> {
    // Only job zones get workplace details.
    if zone_type.is_residential() || zone_type == ZoneType::None {
        return None;
    }

    let total = capacity as usize;
    if total == 0 {
        return None;
    }

    // Determine the dominant job type and the slot distribution.
    // Returns: (primary JobType, Vec of (JobType, fraction) pairs)
    let (primary, distribution): (JobType, Vec<(JobType, f32)>) = match zone_type {
        ZoneType::Industrial => (
            JobType::Unskilled,
            vec![
                (JobType::Unskilled, 0.65),
                (JobType::Skilled, 0.25),
                (JobType::Service, 0.10),
            ],
        ),
        ZoneType::CommercialLow => (
            JobType::Service,
            vec![
                (JobType::Service, 0.55),
                (JobType::Unskilled, 0.25),
                (JobType::Skilled, 0.20),
            ],
        ),
        ZoneType::CommercialHigh => (
            JobType::Service,
            vec![
                (JobType::Service, 0.40),
                (JobType::Skilled, 0.30),
                (JobType::Professional, 0.20),
                (JobType::Executive, 0.10),
            ],
        ),
        ZoneType::Office => (
            JobType::Professional,
            vec![
                (JobType::Professional, 0.40),
                (JobType::Executive, 0.15),
                (JobType::Skilled, 0.25),
                (JobType::Service, 0.20),
            ],
        ),
        _ => return None,
    };

    let mut slots = Vec::with_capacity(total);
    let mut remaining = total;

    // Allocate slots according to the fractional distribution.
    for (i, &(jt, fraction)) in distribution.iter().enumerate() {
        let count = if i == distribution.len() - 1 {
            // Last category gets the remainder to avoid off-by-one.
            remaining
        } else {
            let c = (total as f32 * fraction).round() as usize;
            c.min(remaining)
        };
        remaining = remaining.saturating_sub(count);

        let req = jt.requirement();
        for _ in 0..count {
            slots.push(JobSlot {
                filled: false,
                worker_entity: None,
                education_req: req.min_education,
                job_type: jt,
            });
        }
    }

    Some((primary, slots))
}

/// Automatically attaches `WorkplaceDetails` to newly-spawned buildings
/// that are job zones (Commercial, Industrial, Office).
pub fn assign_workplace_details(
    mut commands: Commands,
    new_buildings: Query<(Entity, &Building), Added<Building>>,
    existing_details: Query<&WorkplaceDetails>,
) {
    for (entity, building) in &new_buildings {
        // Skip if already has details (e.g. loaded from save).
        if existing_details.get(entity).is_ok() {
            continue;
        }

        if let Some((primary, slots)) = generate_slots(building.zone_type, building.capacity) {
            let req = primary.requirement();
            commands.entity(entity).insert(WorkplaceDetails {
                job_type: primary,
                filled_slots: 0,
                required_education: req.min_education,
                job_slots: slots,
            });
        }
    }
}

// ---------------------------------------------------------------------------
// System: job_matching
// Every 20 ticks, match unemployed citizens to available job slots.
// ---------------------------------------------------------------------------

/// Happiness penalty for overqualified workers (per education level gap).
const OVERQUALIFIED_HAPPINESS_PENALTY: f32 = 5.0;

#[allow(clippy::too_many_arguments)]
pub fn job_matching(
    tick: Res<TickCounter>,
    mut commands: Commands,
    mut unemployed: Query<
        (Entity, &mut CitizenDetails, &HomeLocation),
        (With<Citizen>, Without<WorkLocation>),
    >,
    employed: Query<Entity, (With<Citizen>, With<WorkLocation>)>,
    mut workplaces: Query<(Entity, &Building, &mut WorkplaceDetails)>,
    mut stats: ResMut<EmploymentStats>,
) {
    // Always update stats every 20 ticks (even if no matching happens).
    if !tick.0.is_multiple_of(20) {
        return;
    }

    // --- Recount employment stats ---
    let employed_count = employed.iter().count() as u32;
    let mut unemployed_count = 0u32;

    // Reset per-type counters.
    for val in stats.jobs_by_type.values_mut() {
        *val = (0, 0);
    }

    // Count total and filled slots from workplace details.
    for (_entity, _building, details) in &workplaces {
        for slot in &details.job_slots {
            let entry = stats.jobs_by_type.entry(slot.job_type).or_insert((0, 0));
            entry.1 += 1; // total
            if slot.filled {
                entry.0 += 1; // filled
            }
        }
    }

    // --- Collect unemployed working-age citizens ---
    let mut seekers: Vec<(Entity, u8, usize, usize)> = Vec::new();
    for (entity, details, home) in &unemployed {
        if !details.life_stage().can_work() {
            continue;
        }
        seekers.push((entity, details.education, home.grid_x, home.grid_y));
        unemployed_count += 1;
    }

    // Sort seekers by education descending so higher-educated citizens get first pick.
    seekers.sort_by(|a, b| b.1.cmp(&a.1));

    // --- Collect available workplaces with open slots ---
    // For each workplace, gather (workplace_entity, grid_x, grid_y, slot_index, education_req, job_type, salary_mult)
    struct OpenSlot {
        workplace_entity: Entity,
        slot_index: usize,
        education_req: u8,
        job_type: JobType,
        salary_mult: f32,
    }

    let mut open_slots: Vec<OpenSlot> = Vec::new();
    for (wp_entity, _building, details) in &workplaces {
        for (i, slot) in details.job_slots.iter().enumerate() {
            if slot.filled {
                continue;
            }
            let req = slot.job_type.requirement();
            open_slots.push(OpenSlot {
                workplace_entity: wp_entity,
                slot_index: i,
                education_req: slot.education_req,
                job_type: slot.job_type,
                salary_mult: req.salary_multiplier,
            });
        }
    }

    // Sort open slots by salary multiplier descending (best jobs first).
    open_slots.sort_by(|a, b| {
        b.salary_mult
            .partial_cmp(&a.salary_mult)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // --- Match seekers to slots ---
    let mut placed = 0u32;
    let max_per_tick = 100u32;

    // Track which slots have been claimed this tick (slot might appear multiple times
    // if we used indices, but since we mutate at the end, track by (entity, slot_index)).
    let mut claimed: Vec<(Entity, usize, Entity, u8, f32, JobType)> = Vec::new();
    // (workplace_entity, slot_index, citizen_entity, citizen_edu, salary_mult, job_type)

    'outer: for (citizen_entity, education, _home_gx, _home_gy) in &seekers {
        if placed >= max_per_tick {
            break;
        }

        // Find the best unclaimed slot this citizen qualifies for.
        // Higher-education citizens prefer the highest salary_mult slot they qualify for.
        // Underqualified citizens cannot fill slots above their education.
        for slot in &open_slots {
            // Check this slot hasn't been claimed already.
            if claimed
                .iter()
                .any(|c| c.0 == slot.workplace_entity && c.1 == slot.slot_index)
            {
                continue;
            }

            // Education check: citizen must meet minimum requirement.
            if *education < slot.education_req {
                continue;
            }

            // Prefer closer workplaces when multiple slots have the same salary tier.
            // (We accept the first matching slot in salary-descending order for simplicity.)
            let base_salary = CitizenDetails::base_salary_for_education(*education);
            let salary = base_salary * slot.salary_mult;

            claimed.push((
                slot.workplace_entity,
                slot.slot_index,
                *citizen_entity,
                *education,
                salary,
                slot.job_type,
            ));
            placed += 1;
            continue 'outer;
        }
    }

    // --- Apply matches ---
    for (wp_entity, slot_idx, citizen_entity, citizen_edu, salary, job_type) in &claimed {
        if let Ok((_, building, mut details)) = workplaces.get_mut(*wp_entity) {
            if let Some(slot) = details.job_slots.get_mut(*slot_idx) {
                slot.filled = true;
                slot.worker_entity = Some(*citizen_entity);
                details.filled_slots += 1;
            }

            // Assign work location to citizen.
            commands.entity(*citizen_entity).insert(WorkLocation {
                grid_x: building.grid_x,
                grid_y: building.grid_y,
                building: *wp_entity,
            });

            // Update citizen salary and apply overqualification penalty.
            if let Ok((_entity, mut cit_details, _home)) = unemployed.get_mut(*citizen_entity) {
                cit_details.salary = *salary;

                // Overqualification penalty: if citizen's education exceeds slot requirement
                let req = job_type.requirement();
                if *citizen_edu > req.min_education {
                    let gap = (*citizen_edu - req.min_education) as f32;
                    cit_details.happiness =
                        (cit_details.happiness - gap * OVERQUALIFIED_HAPPINESS_PENALTY).max(0.0);
                }
            }
        }
    }

    // --- Finalize stats ---
    unemployed_count = unemployed_count.saturating_sub(placed);
    stats.total_employed = employed_count + placed;
    stats.total_unemployed = unemployed_count;
    let total_workforce = stats.total_employed + stats.total_unemployed;
    stats.unemployment_rate = if total_workforce > 0 {
        stats.total_unemployed as f32 / total_workforce as f32
    } else {
        0.0
    };
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_type_requirements() {
        let req = JobType::Unskilled.requirement();
        assert_eq!(req.min_education, 0);
        assert!(req.salary_multiplier < 1.0);

        let req = JobType::Executive.requirement();
        assert_eq!(req.min_education, 3);
        assert!(req.salary_multiplier > 2.0);
    }

    #[test]
    fn test_generate_slots_residential_returns_none() {
        assert!(generate_slots(ZoneType::ResidentialLow, 10).is_none());
        assert!(generate_slots(ZoneType::ResidentialHigh, 100).is_none());
        assert!(generate_slots(ZoneType::None, 50).is_none());
    }

    #[test]
    fn test_generate_slots_industrial() {
        let (primary, slots) = generate_slots(ZoneType::Industrial, 20).unwrap();
        assert_eq!(primary, JobType::Unskilled);
        assert_eq!(slots.len(), 20);

        // Most slots should be unskilled.
        let unskilled_count = slots.iter().filter(|s| s.job_type == JobType::Unskilled).count();
        assert!(unskilled_count >= 10, "expected mostly unskilled slots, got {}", unskilled_count);
    }

    #[test]
    fn test_generate_slots_office() {
        let (primary, slots) = generate_slots(ZoneType::Office, 30).unwrap();
        assert_eq!(primary, JobType::Professional);
        assert_eq!(slots.len(), 30);

        // Should have executive and professional slots.
        let exec_count = slots.iter().filter(|s| s.job_type == JobType::Executive).count();
        let prof_count = slots.iter().filter(|s| s.job_type == JobType::Professional).count();
        assert!(exec_count > 0, "office should have executive slots");
        assert!(prof_count > 0, "office should have professional slots");
    }

    #[test]
    fn test_generate_slots_commercial_high() {
        let (primary, slots) = generate_slots(ZoneType::CommercialHigh, 100).unwrap();
        assert_eq!(primary, JobType::Service);
        assert_eq!(slots.len(), 100);

        // Should have a mix of service, skilled, professional, executive.
        let service_count = slots.iter().filter(|s| s.job_type == JobType::Service).count();
        let skilled_count = slots.iter().filter(|s| s.job_type == JobType::Skilled).count();
        assert!(service_count > 30, "commercial high should have many service slots");
        assert!(skilled_count > 0, "commercial high should have skilled slots");
    }

    #[test]
    fn test_employment_stats_default() {
        let stats = EmploymentStats::default();
        assert_eq!(stats.total_employed, 0);
        assert_eq!(stats.total_unemployed, 0);
        assert_eq!(stats.unemployment_rate, 0.0);
        assert_eq!(stats.jobs_by_type.len(), 5);
    }

    #[test]
    fn test_workplace_available_slots() {
        let details = WorkplaceDetails {
            job_type: JobType::Service,
            filled_slots: 1,
            required_education: 0,
            job_slots: vec![
                JobSlot {
                    filled: true,
                    worker_entity: None,
                    education_req: 0,
                    job_type: JobType::Unskilled,
                },
                JobSlot {
                    filled: false,
                    worker_entity: None,
                    education_req: 0,
                    job_type: JobType::Service,
                },
                JobSlot {
                    filled: false,
                    worker_entity: None,
                    education_req: 2,
                    job_type: JobType::Professional,
                },
            ],
        };

        // Education 0: can fill unskilled (filled) and service (open) = 1 available
        assert_eq!(details.available_slots_for_education(0), 1);
        // Education 2: can fill service + professional = 2 available
        assert_eq!(details.available_slots_for_education(2), 2);
        // Education 3: can fill both open slots = 2
        assert_eq!(details.available_slots_for_education(3), 2);
    }
}
