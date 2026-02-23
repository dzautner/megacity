use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::crime::CrimeGrid;
use crate::education::EducationGrid;
use crate::education_jobs::EmploymentStats;
use crate::fire::FireGrid;
use crate::health::HealthGrid;
use crate::homelessness::HomelessnessStats;
use crate::loans::LoanBook;
use crate::pollution::PollutionGrid;
use crate::road_maintenance::RoadMaintenanceStats;
use crate::traffic::TrafficGrid;
use crate::zones::ZoneDemand;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Categories of city advisors, each monitoring a different domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AdvisorType {
    Finance,
    Infrastructure,
    Health,
    Education,
    Safety,
    Environment,
    Housing,
}

impl AdvisorType {
    /// Display name for this advisor.
    pub fn name(self) -> &'static str {
        match self {
            AdvisorType::Finance => "Finance",
            AdvisorType::Infrastructure => "Infrastructure",
            AdvisorType::Health => "Health",
            AdvisorType::Education => "Education",
            AdvisorType::Safety => "Safety",
            AdvisorType::Environment => "Environment",
            AdvisorType::Housing => "Housing",
        }
    }

    /// Icon character for this advisor type (for the notification area).
    pub fn icon(self) -> &'static str {
        match self {
            AdvisorType::Finance => "$",
            AdvisorType::Infrastructure => "I",
            AdvisorType::Health => "+",
            AdvisorType::Education => "E",
            AdvisorType::Safety => "!",
            AdvisorType::Environment => "~",
            AdvisorType::Housing => "H",
        }
    }
}

/// Unique identifier for each distinct advisor tip type.
/// Used for permanent dismissal tracking -- when the player dismisses a tip,
/// its `TipId` is recorded so the same tip category never appears again.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    bitcode::Encode,
    bitcode::Decode,
)]
pub enum TipId {
    // Finance
    TreasuryCritical,
    TreasuryLow,
    BudgetDeficit,
    DebtCritical,
    DebtElevated,
    HighTaxUnhappy,
    LowTaxDeclining,

    // Infrastructure
    PowerCoverageCritical,
    PowerCoverageLow,
    WaterCoverageCritical,
    WaterCoverageLow,
    RoadsCritical,
    RoadsPoor,

    // Health
    HealthCoverageCritical,
    HealthCoverageLow,
    PollutionHealthRisk,

    // Education
    EducationVeryLow,
    EducationLow,
    UnemploymentEducation,

    // Safety
    CrimeCritical,
    CrimeRising,
    ActiveFiresCritical,
    ActiveFires,
    HighCrimeCells,

    // Environment
    PollutionHigh,
    PollutionRising,
    PollutionHotspots,

    // Housing
    HomelessCritical,
    HomelessModerate,
    HighUnemployment,
    EmptyResidential,

    // Traffic
    TrafficCongestion,
    RoadHierarchyViolation,

    // Zone demand
    ZoneDemandResidential,
    ZoneDemandCommercial,
    ZoneDemandIndustrial,

    // Fire coverage
    FireCoverageGap,
}

impl TipId {
    /// Human-readable label for this tip type (used in dismiss confirmation).
    pub fn label(self) -> &'static str {
        match self {
            TipId::TreasuryCritical => "Treasury Critical",
            TipId::TreasuryLow => "Treasury Low",
            TipId::BudgetDeficit => "Budget Deficit",
            TipId::DebtCritical => "Debt Critical",
            TipId::DebtElevated => "Debt Elevated",
            TipId::HighTaxUnhappy => "High Tax / Unhappy",
            TipId::LowTaxDeclining => "Low Tax / Declining",
            TipId::PowerCoverageCritical => "Power Coverage Critical",
            TipId::PowerCoverageLow => "Power Coverage Low",
            TipId::WaterCoverageCritical => "Water Coverage Critical",
            TipId::WaterCoverageLow => "Water Coverage Low",
            TipId::RoadsCritical => "Roads Critical",
            TipId::RoadsPoor => "Roads Poor",
            TipId::HealthCoverageCritical => "Health Coverage Critical",
            TipId::HealthCoverageLow => "Health Coverage Low",
            TipId::PollutionHealthRisk => "Pollution Health Risk",
            TipId::EducationVeryLow => "Education Very Low",
            TipId::EducationLow => "Education Low",
            TipId::UnemploymentEducation => "Unemployment + Education",
            TipId::CrimeCritical => "Crime Critical",
            TipId::CrimeRising => "Crime Rising",
            TipId::ActiveFiresCritical => "Active Fires Critical",
            TipId::ActiveFires => "Active Fires",
            TipId::HighCrimeCells => "High Crime Cells",
            TipId::PollutionHigh => "Pollution High",
            TipId::PollutionRising => "Pollution Rising",
            TipId::PollutionHotspots => "Pollution Hotspots",
            TipId::HomelessCritical => "Homeless Critical",
            TipId::HomelessModerate => "Homeless Moderate",
            TipId::HighUnemployment => "High Unemployment",
            TipId::EmptyResidential => "Empty Residential",
            TipId::TrafficCongestion => "Traffic Congestion",
            TipId::RoadHierarchyViolation => "Road Hierarchy Violation",
            TipId::ZoneDemandResidential => "Residential Demand",
            TipId::ZoneDemandCommercial => "Commercial Demand",
            TipId::ZoneDemandIndustrial => "Industrial Demand",
            TipId::FireCoverageGap => "Fire Coverage Gap",
        }
    }
}

/// A single advisor message displayed in the advisor panel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvisorMessage {
    pub advisor_type: AdvisorType,
    pub tip_id: TipId,
    pub message: String,
    pub priority: u8, // 1 (low) to 5 (critical)
    pub suggestion: String,
    pub tick_created: u64,
    /// Optional grid coordinates (col, row) for "Show Location" button.
    /// When present, the UI can jump the camera to this location.
    pub location: Option<(usize, usize)>,
}

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Maximum number of messages kept in the panel at any time.
pub(crate) const MAX_MESSAGES: usize = 10;

/// Messages expire after this many ticks.
pub(crate) const EXPIRY_TICKS: u64 = 500;

/// The advisor system runs every N ticks.
pub(crate) const ADVISOR_INTERVAL: u64 = 200;

/// Resource that holds the current set of advisor messages shown to the player.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct AdvisorPanel {
    pub messages: Vec<AdvisorMessage>,
}

impl AdvisorPanel {
    /// Remove expired messages and keep at most `MAX_MESSAGES`, sorted by priority descending.
    pub(crate) fn prune(&mut self, current_tick: u64) {
        self.messages
            .retain(|m| current_tick.saturating_sub(m.tick_created) < EXPIRY_TICKS);
        self.messages.sort_by(|a, b| b.priority.cmp(&a.priority));
        self.messages.truncate(MAX_MESSAGES);
    }

    /// Push a message, then prune.
    pub(crate) fn push(&mut self, msg: AdvisorMessage, current_tick: u64) {
        self.messages.push(msg);
        self.prune(current_tick);
    }
}

/// Tracks which tip types the player has permanently dismissed.
/// Dismissed tips will never appear again (until the player resets them).
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct DismissedAdvisorTips {
    pub dismissed: HashSet<TipId>,
}

impl DismissedAdvisorTips {
    /// Returns true if this tip type has been dismissed.
    pub fn is_dismissed(&self, tip_id: TipId) -> bool {
        self.dismissed.contains(&tip_id)
    }

    /// Dismiss a tip type permanently.
    pub fn dismiss(&mut self, tip_id: TipId) {
        self.dismissed.insert(tip_id);
    }

    /// Restore a previously dismissed tip type.
    pub fn restore(&mut self, tip_id: TipId) {
        self.dismissed.remove(&tip_id);
    }

    /// Restore all dismissed tips.
    pub fn restore_all(&mut self) {
        self.dismissed.clear();
    }
}

/// Event sent from the UI to request the camera jump to a world position.
/// The rendering crate listens for this event and moves the `OrbitCamera` focus.
#[derive(Event, Debug, Clone)]
pub struct AdvisorJumpToLocation {
    /// Grid coordinates (col, row) to jump to.
    pub grid_x: usize,
    pub grid_y: usize,
}

/// Bundled secondary resources so the system stays within Bevy's parameter limits.
#[derive(bevy::ecs::system::SystemParam)]
pub struct AdvisorExtras<'w> {
    pub employment: Res<'w, EmploymentStats>,
    pub homeless: Res<'w, HomelessnessStats>,
    pub loan_book: Res<'w, LoanBook>,
    pub pollution: Res<'w, PollutionGrid>,
    pub education_grid: Res<'w, EducationGrid>,
    pub crime: Res<'w, CrimeGrid>,
    pub fire: Res<'w, FireGrid>,
    pub health: Res<'w, HealthGrid>,
    pub road_stats: Res<'w, RoadMaintenanceStats>,
    pub traffic: Res<'w, TrafficGrid>,
    pub zone_demand: Res<'w, ZoneDemand>,
}
