mod plugin;
mod systems;
mod types;

pub use plugin::EducationJobsPlugin;
pub use systems::{assign_workplace_details, job_matching};
pub use types::{EmploymentStats, JobRequirement, JobSlot, JobType, WorkplaceDetails};
