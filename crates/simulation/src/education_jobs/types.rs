use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
