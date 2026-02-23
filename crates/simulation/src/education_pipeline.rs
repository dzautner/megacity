//! Education Pipeline — multi-stage education with graduation rates.
//!
//! Citizens progress through Elementary (ages 6–11, 95% base grad rate),
//! High School (ages 12–17, 85%), and University (ages 18–22, 70%).
//! Graduation rates are modulated by school quality (nearby EducationGrid
//! level) and capacity pressure (ratio of students to available school slots).

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::citizen::{Citizen, CitizenDetails, HomeLocation};
use crate::education::EducationGrid;
use crate::services::{ServiceBuilding, ServiceType};
use crate::SlowTickTimer;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// The discrete education levels a citizen can hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum EducationLevel {
    None = 0,
    Elementary = 1,
    HighSchool = 2,
    University = 3,
}

impl EducationLevel {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::None,
            1 => Self::Elementary,
            2 => Self::HighSchool,
            _ => Self::University,
        }
    }

    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Describes one stage of the education pipeline.
#[derive(Debug, Clone, Copy)]
pub struct PipelineStage {
    /// Minimum age to enter this stage.
    pub min_age: u8,
    /// Maximum age at which graduation can happen.
    pub max_age: u8,
    /// Base graduation rate (0.0–1.0) under ideal conditions.
    pub base_grad_rate: f32,
    /// The education level required on the EducationGrid for this stage.
    pub required_grid_level: u8,
    /// The education level granted upon graduation.
    pub grants_level: EducationLevel,
    /// The education level required before entering this stage.
    pub prerequisite: EducationLevel,
}

/// The three stages of the education pipeline.
pub const STAGES: [PipelineStage; 3] = [
    PipelineStage {
        min_age: 6,
        max_age: 11,
        base_grad_rate: 0.95,
        required_grid_level: 1,
        grants_level: EducationLevel::Elementary,
        prerequisite: EducationLevel::None,
    },
    PipelineStage {
        min_age: 12,
        max_age: 17,
        base_grad_rate: 0.85,
        required_grid_level: 2,
        grants_level: EducationLevel::HighSchool,
        prerequisite: EducationLevel::Elementary,
    },
    PipelineStage {
        min_age: 18,
        max_age: 22,
        base_grad_rate: 0.70,
        required_grid_level: 3,
        grants_level: EducationLevel::University,
        prerequisite: EducationLevel::HighSchool,
    },
];

// ---------------------------------------------------------------------------
// Enrollment tracking component
// ---------------------------------------------------------------------------

/// Attached to citizens currently enrolled in a stage of the pipeline.
/// Removed when they graduate or age out.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Enrollment {
    /// Index into `STAGES` (0 = elementary, 1 = high school, 2 = university).
    pub stage_index: u8,
    /// Number of slow-tick cycles the citizen has been enrolled.
    pub ticks_enrolled: u32,
}

// ---------------------------------------------------------------------------
// Pipeline statistics resource
// ---------------------------------------------------------------------------

/// City-wide education pipeline statistics, updated each slow tick.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct EducationPipelineStats {
    /// Number of citizens currently enrolled per stage.
    pub enrolled: [u32; 3],
    /// Cumulative graduates per stage (since city start / last load).
    pub graduates: [u32; 3],
    /// Cumulative dropouts per stage.
    pub dropouts: [u32; 3],
    /// Effective graduation rate per stage (rolling average).
    pub effective_grad_rate: [f32; 3],
}

// ---------------------------------------------------------------------------
// School capacity helper
// ---------------------------------------------------------------------------

/// Count the total capacity of education service buildings per stage.
fn count_school_slots(services: &Query<&ServiceBuilding>) -> [u32; 3] {
    let mut slots = [0u32; 3];
    for svc in services.iter() {
        match svc.service_type {
            ServiceType::ElementarySchool | ServiceType::Kindergarten => slots[0] += 1,
            ServiceType::HighSchool => slots[1] += 1,
            ServiceType::University => slots[2] += 1,
            _ => {}
        }
    }
    // Each school supports a base number of students.
    [slots[0] * 200, slots[1] * 150, slots[2] * 300]
}

/// Compute the capacity modifier for a given stage.
/// When students exceed capacity, graduation rate drops.
/// Returns a value in \[0.5, 1.0\].
fn capacity_modifier(enrolled: u32, capacity: u32) -> f32 {
    if capacity == 0 {
        return 0.5;
    }
    let ratio = enrolled as f32 / capacity as f32;
    if ratio <= 1.0 {
        1.0
    } else {
        (1.0 - (ratio - 1.0) * 0.5).clamp(0.5, 1.0)
    }
}

// ---------------------------------------------------------------------------
// System: enroll_citizens
// ---------------------------------------------------------------------------

#[allow(clippy::type_complexity)]
pub fn enroll_citizens(
    slow_tick: Res<SlowTickTimer>,
    edu_grid: Res<EducationGrid>,
    mut commands: Commands,
    candidates: Query<
        (Entity, &CitizenDetails, &HomeLocation),
        (With<Citizen>, Without<Enrollment>),
    >,
) {
    if !slow_tick.should_run() {
        return;
    }

    for (entity, details, home) in &candidates {
        let current_level = EducationLevel::from_u8(details.education);
        let available = edu_grid.get(home.grid_x, home.grid_y);

        for (i, stage) in STAGES.iter().enumerate() {
            if details.age < stage.min_age || details.age > stage.max_age {
                continue;
            }
            if current_level != stage.prerequisite {
                continue;
            }
            if available < stage.required_grid_level {
                continue;
            }
            commands.entity(entity).insert(Enrollment {
                stage_index: i as u8,
                ticks_enrolled: 0,
            });
            break;
        }
    }
}

// ---------------------------------------------------------------------------
// System: process_graduations
// ---------------------------------------------------------------------------

/// Minimum ticks enrolled before graduation is possible.
const MIN_ENROLLMENT_TICKS: u32 = 3;

#[allow(clippy::too_many_arguments)]
pub fn process_graduations(
    slow_tick: Res<SlowTickTimer>,
    edu_grid: Res<EducationGrid>,
    services: Query<&ServiceBuilding>,
    mut stats: ResMut<EducationPipelineStats>,
    mut citizens: Query<
        (Entity, &mut CitizenDetails, &HomeLocation, &mut Enrollment),
        With<Citizen>,
    >,
    mut commands: Commands,
) {
    if !slow_tick.should_run() {
        return;
    }

    let school_capacity = count_school_slots(&services);
    let enrolled_counts = stats.enrolled;

    for (entity, mut details, home, mut enrollment) in &mut citizens {
        let idx = enrollment.stage_index as usize;
        if idx >= STAGES.len() {
            commands.entity(entity).remove::<Enrollment>();
            continue;
        }
        let stage = &STAGES[idx];

        enrollment.ticks_enrolled += 1;

        // Aged out — dropout
        if details.age > stage.max_age {
            stats.dropouts[idx] += 1;
            stats.enrolled[idx] = stats.enrolled[idx].saturating_sub(1);
            commands.entity(entity).remove::<Enrollment>();
            continue;
        }

        // School no longer available — dropout
        let available = edu_grid.get(home.grid_x, home.grid_y);
        if available < stage.required_grid_level {
            stats.dropouts[idx] += 1;
            stats.enrolled[idx] = stats.enrolled[idx].saturating_sub(1);
            commands.entity(entity).remove::<Enrollment>();
            continue;
        }

        // Not enough time enrolled yet
        if enrollment.ticks_enrolled < MIN_ENROLLMENT_TICKS {
            continue;
        }

        // Calculate effective graduation rate
        let cap_mod = capacity_modifier(enrolled_counts[idx], school_capacity[idx]);
        let effective_rate = stage.base_grad_rate * cap_mod;

        // Deterministic graduation check using entity index + ticks
        let hash = entity
            .index()
            .wrapping_mul(31)
            .wrapping_add(enrollment.ticks_enrolled.wrapping_mul(17));
        let roll = (hash % 1000) as f32 / 1000.0;

        if roll < effective_rate {
            details.education = stage.grants_level.as_u8();
            details.salary = CitizenDetails::base_salary_for_education(details.education);
            stats.graduates[idx] += 1;
            stats.enrolled[idx] = stats.enrolled[idx].saturating_sub(1);

            let total = stats.graduates[idx] + stats.dropouts[idx];
            if total > 0 {
                stats.effective_grad_rate[idx] = stats.graduates[idx] as f32 / total as f32;
            }

            commands.entity(entity).remove::<Enrollment>();
        }
    }
}

// ---------------------------------------------------------------------------
// System: update_enrollment_counts
// ---------------------------------------------------------------------------

pub fn update_enrollment_counts(
    slow_tick: Res<SlowTickTimer>,
    mut stats: ResMut<EducationPipelineStats>,
    enrolled: Query<&Enrollment, With<Citizen>>,
) {
    if !slow_tick.should_run() {
        return;
    }

    let mut counts = [0u32; 3];
    for enrollment in &enrolled {
        let idx = enrollment.stage_index as usize;
        if idx < 3 {
            counts[idx] += 1;
        }
    }
    stats.enrolled = counts;
}

// ---------------------------------------------------------------------------
// Saveable implementation
// ---------------------------------------------------------------------------

impl crate::Saveable for EducationPipelineStats {
    const SAVE_KEY: &'static str = "education_pipeline";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        bitcode::encode(self).ok()
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct EducationPipelinePlugin;

impl Plugin for EducationPipelinePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EducationPipelineStats>().add_systems(
            FixedUpdate,
            (
                enroll_citizens,
                process_graduations,
                update_enrollment_counts,
            )
                .chain()
                .after(crate::education::propagate_education)
                .in_set(crate::SimulationSet::Simulation),
        );

        // Register for save/load
        let mut registry = app
            .world_mut()
            .get_resource_or_insert_with(crate::SaveableRegistry::default);
        registry.register::<EducationPipelineStats>();
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_education_level_roundtrip() {
        for v in 0..=3 {
            let level = EducationLevel::from_u8(v);
            assert_eq!(level.as_u8(), v);
        }
    }

    #[test]
    fn test_education_level_ordering() {
        assert!(EducationLevel::None < EducationLevel::Elementary);
        assert!(EducationLevel::Elementary < EducationLevel::HighSchool);
        assert!(EducationLevel::HighSchool < EducationLevel::University);
    }

    #[test]
    fn test_capacity_modifier_no_schools() {
        assert_eq!(capacity_modifier(10, 0), 0.5);
    }

    #[test]
    fn test_capacity_modifier_under_capacity() {
        assert_eq!(capacity_modifier(50, 200), 1.0);
    }

    #[test]
    fn test_capacity_modifier_at_capacity() {
        assert_eq!(capacity_modifier(200, 200), 1.0);
    }

    #[test]
    fn test_capacity_modifier_over_capacity() {
        let m = capacity_modifier(400, 200);
        assert!(m < 1.0, "over-capacity should reduce modifier");
        assert!(m >= 0.5, "modifier should not drop below 0.5");
    }

    #[test]
    fn test_capacity_modifier_extreme_overcapacity() {
        let m = capacity_modifier(1000, 200);
        assert_eq!(m, 0.5, "extreme overcapacity should hit floor of 0.5");
    }

    #[test]
    fn test_pipeline_stages_valid() {
        for stage in &STAGES {
            assert!(stage.min_age <= stage.max_age);
            assert!(stage.base_grad_rate > 0.0 && stage.base_grad_rate <= 1.0);
        }
    }

    #[test]
    fn test_stage_age_ranges_no_overlap() {
        assert!(STAGES[0].max_age < STAGES[1].min_age);
        assert!(STAGES[1].max_age < STAGES[2].min_age);
    }

    #[test]
    fn test_stage_prerequisites_chain() {
        assert_eq!(STAGES[0].prerequisite, EducationLevel::None);
        assert_eq!(STAGES[1].prerequisite, EducationLevel::Elementary);
        assert_eq!(STAGES[2].prerequisite, EducationLevel::HighSchool);
    }

    #[test]
    fn test_stage_grants_match_levels() {
        assert_eq!(STAGES[0].grants_level, EducationLevel::Elementary);
        assert_eq!(STAGES[1].grants_level, EducationLevel::HighSchool);
        assert_eq!(STAGES[2].grants_level, EducationLevel::University);
    }

    #[test]
    fn test_stats_default() {
        let stats = EducationPipelineStats::default();
        assert_eq!(stats.enrolled, [0, 0, 0]);
        assert_eq!(stats.graduates, [0, 0, 0]);
        assert_eq!(stats.dropouts, [0, 0, 0]);
    }
}
