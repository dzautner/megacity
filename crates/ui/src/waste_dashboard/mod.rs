//! Waste Management Dashboard UI Panel (WASTE-011).
//!
//! Displays a comprehensive waste management overview including:
//! - Total waste generated, collected, and uncollected (tons/day)
//! - Diversion metrics: recycling rate, composting rate, WTE rate
//! - Landfill capacity: current fill percentage and years remaining
//! - Waste stream breakdown (paper, food, yard, plastics, metals, glass, wood, textiles, other)
//! - Collection coverage: percentage of buildings served
//! - Monthly waste budget: collection cost, processing cost, recycling revenue, net cost
//! - Warning indicators for low landfill capacity, uncollected waste, and overflow

mod dashboard_ui;
mod formatting;
mod warnings;

use bevy::prelude::*;
use simulation::app_state::AppState;

// Re-export all public items so the rest of the crate sees the same API.
pub use dashboard_ui::waste_dashboard_ui;
pub use formatting::{fmt_dollars, fmt_pct, fmt_tons};
pub use warnings::{
    landfill_warning_severity, overflow_warning_severity, uncollected_warning_severity,
    warning_color, WarningSeverity,
};

// =============================================================================
// Visibility resource
// =============================================================================

/// Resource controlling whether the waste management dashboard is visible.
/// Toggle with 'G' key.
#[derive(Resource, Default)]
pub struct WasteDashboardVisible(pub bool);

// =============================================================================
// Plugin
// =============================================================================

pub struct WasteDashboardPlugin;

impl Plugin for WasteDashboardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WasteDashboardVisible>()
            .add_systems(Update, waste_dashboard_ui.run_if(in_state(AppState::Playing)));
    }
}
