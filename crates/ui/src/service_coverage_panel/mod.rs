//! Service Coverage Detail Panel (UI-002).
//!
//! Displays a comprehensive overview of all service categories with:
//! - Coverage percentage per category computed from `ServiceCoverageGrid`
//! - Color coding: green (>80%), yellow (50-80%), red (<50%)
//! - Clickable rows to activate the corresponding overlay mode
//! - Per-service-type breakdown within each category (building count, maintenance)
//! - Total capacity (number of service buildings) and current demand
//!   (number of zoned/developed cells) for each category
//! - Monthly maintenance cost per category
//! - Additional "Other Services" section for services without coverage tracking
//! - Keybind: K key to toggle visibility

mod categories;
mod panel_ui;
mod stats;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_computation;

pub use categories::{OtherServiceGroup, ServiceCategory, OTHER_SERVICE_TYPES};
pub use panel_ui::{
    service_coverage_keybind, service_coverage_panel_ui, ExpandedCategories,
    ServiceCoveragePanelPlugin, ServiceCoveragePanelVisible,
};
pub use stats::{
    compute_category_stats, compute_service_type_stats, coverage_color, coverage_label,
    CategoryStats, ServiceTypeStats,
};
