//! Warning indicators for the waste management dashboard.
//!
//! Provides severity classification and color coding for landfill capacity,
//! uncollected waste, and collection overflow warnings.

use bevy_egui::egui;

use simulation::garbage::WasteSystem;
use simulation::landfill_warning::{LandfillCapacityState, LandfillWarningTier};

/// Warning severity level for the dashboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarningSeverity {
    None,
    Low,
    High,
    Critical,
}

/// Returns the warning severity for landfill capacity.
pub fn landfill_warning_severity(landfill: &LandfillCapacityState) -> WarningSeverity {
    match landfill.current_tier {
        LandfillWarningTier::Normal => WarningSeverity::None,
        LandfillWarningTier::Low => WarningSeverity::Low,
        LandfillWarningTier::Critical => WarningSeverity::High,
        LandfillWarningTier::VeryLow | LandfillWarningTier::Emergency => WarningSeverity::Critical,
    }
}

/// Returns the warning severity for uncollected waste.
pub fn uncollected_warning_severity(waste: &WasteSystem) -> WarningSeverity {
    if waste.uncovered_buildings == 0 {
        WarningSeverity::None
    } else if waste.uncovered_buildings < 10 {
        WarningSeverity::Low
    } else if waste.uncovered_buildings < 50 {
        WarningSeverity::High
    } else {
        WarningSeverity::Critical
    }
}

/// Returns the warning severity for collection overflow (capacity < generation).
pub fn overflow_warning_severity(waste: &WasteSystem) -> WarningSeverity {
    if waste.collection_rate >= 1.0 {
        WarningSeverity::None
    } else if waste.collection_rate >= 0.8 {
        WarningSeverity::Low
    } else if waste.collection_rate >= 0.5 {
        WarningSeverity::High
    } else {
        WarningSeverity::Critical
    }
}

/// Returns the egui color for a warning severity.
pub fn warning_color(severity: WarningSeverity) -> egui::Color32 {
    match severity {
        WarningSeverity::None => egui::Color32::from_rgb(80, 200, 80),
        WarningSeverity::Low => egui::Color32::from_rgb(220, 200, 50),
        WarningSeverity::High => egui::Color32::from_rgb(240, 140, 40),
        WarningSeverity::Critical => egui::Color32::from_rgb(255, 60, 60),
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Warning severity tests
    // =========================================================================

    #[test]
    fn test_landfill_warning_normal() {
        let state = LandfillCapacityState {
            current_tier: LandfillWarningTier::Normal,
            ..Default::default()
        };
        assert_eq!(landfill_warning_severity(&state), WarningSeverity::None);
    }

    #[test]
    fn test_landfill_warning_low() {
        let state = LandfillCapacityState {
            current_tier: LandfillWarningTier::Low,
            ..Default::default()
        };
        assert_eq!(landfill_warning_severity(&state), WarningSeverity::Low);
    }

    #[test]
    fn test_landfill_warning_critical() {
        let state = LandfillCapacityState {
            current_tier: LandfillWarningTier::Critical,
            ..Default::default()
        };
        assert_eq!(landfill_warning_severity(&state), WarningSeverity::High);
    }

    #[test]
    fn test_landfill_warning_very_low() {
        let state = LandfillCapacityState {
            current_tier: LandfillWarningTier::VeryLow,
            ..Default::default()
        };
        assert_eq!(landfill_warning_severity(&state), WarningSeverity::Critical);
    }

    #[test]
    fn test_landfill_warning_emergency() {
        let state = LandfillCapacityState {
            current_tier: LandfillWarningTier::Emergency,
            ..Default::default()
        };
        assert_eq!(landfill_warning_severity(&state), WarningSeverity::Critical);
    }

    #[test]
    fn test_uncollected_warning_none() {
        let waste = WasteSystem {
            uncovered_buildings: 0,
            ..Default::default()
        };
        assert_eq!(uncollected_warning_severity(&waste), WarningSeverity::None);
    }

    #[test]
    fn test_uncollected_warning_low() {
        let waste = WasteSystem {
            uncovered_buildings: 5,
            ..Default::default()
        };
        assert_eq!(uncollected_warning_severity(&waste), WarningSeverity::Low);
    }

    #[test]
    fn test_uncollected_warning_high() {
        let waste = WasteSystem {
            uncovered_buildings: 25,
            ..Default::default()
        };
        assert_eq!(uncollected_warning_severity(&waste), WarningSeverity::High);
    }

    #[test]
    fn test_uncollected_warning_critical() {
        let waste = WasteSystem {
            uncovered_buildings: 100,
            ..Default::default()
        };
        assert_eq!(
            uncollected_warning_severity(&waste),
            WarningSeverity::Critical
        );
    }

    #[test]
    fn test_overflow_warning_none() {
        let waste = WasteSystem {
            collection_rate: 1.0,
            ..Default::default()
        };
        assert_eq!(overflow_warning_severity(&waste), WarningSeverity::None);
    }

    #[test]
    fn test_overflow_warning_low() {
        let waste = WasteSystem {
            collection_rate: 0.85,
            ..Default::default()
        };
        assert_eq!(overflow_warning_severity(&waste), WarningSeverity::Low);
    }

    #[test]
    fn test_overflow_warning_high() {
        let waste = WasteSystem {
            collection_rate: 0.6,
            ..Default::default()
        };
        assert_eq!(overflow_warning_severity(&waste), WarningSeverity::High);
    }

    #[test]
    fn test_overflow_warning_critical() {
        let waste = WasteSystem {
            collection_rate: 0.3,
            ..Default::default()
        };
        assert_eq!(overflow_warning_severity(&waste), WarningSeverity::Critical);
    }

    // =========================================================================
    // Warning color tests
    // =========================================================================

    #[test]
    fn test_warning_colors_distinct() {
        let none = warning_color(WarningSeverity::None);
        let low = warning_color(WarningSeverity::Low);
        let high = warning_color(WarningSeverity::High);
        let crit = warning_color(WarningSeverity::Critical);
        // All colors should be different
        assert_ne!(none, low);
        assert_ne!(low, high);
        assert_ne!(high, crit);
    }

    // =========================================================================
    // Warning appears when landfill below 25% (test plan item 3)
    // =========================================================================

    #[test]
    fn test_warning_at_25_pct_remaining() {
        let state = LandfillCapacityState {
            current_tier: LandfillWarningTier::Low,
            remaining_pct: 25.0,
            ..Default::default()
        };
        let severity = landfill_warning_severity(&state);
        assert_ne!(severity, WarningSeverity::None);
    }

    #[test]
    fn test_no_warning_above_25_pct() {
        let state = LandfillCapacityState {
            current_tier: LandfillWarningTier::Normal,
            remaining_pct: 50.0,
            ..Default::default()
        };
        let severity = landfill_warning_severity(&state);
        assert_eq!(severity, WarningSeverity::None);
    }

    #[test]
    fn test_warning_at_10_pct_remaining() {
        let state = LandfillCapacityState {
            current_tier: LandfillWarningTier::Critical,
            remaining_pct: 10.0,
            ..Default::default()
        };
        let severity = landfill_warning_severity(&state);
        assert_eq!(severity, WarningSeverity::High);
    }
}
