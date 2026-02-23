//! Water Supply Dashboard UI Panel (WATER-012).
//!
//! Displays a comprehensive water supply dashboard showing:
//! - Total demand (MGD) and total supply (MGD) with surplus/deficit
//! - Source breakdown: wells, surface intake, reservoir, desalination contributions
//! - Groundwater level indicator with depletion warning
//! - Reservoir level: % full, days of storage
//! - Service coverage: % of buildings with water service
//! - Water quality: treatment level and output quality
//! - Sewage treatment: % of wastewater treated, treatment level
//! - Monthly water budget: treatment costs, revenue from water rates

mod panels;
mod tests;
pub mod types;
mod ui_system;

pub use types::WaterDashboardVisible;
pub use ui_system::water_dashboard_ui;
