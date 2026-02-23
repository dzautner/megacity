//! Advisor system -- analyzes city state and generates contextual tips.
//!
//! Split into sub-modules:
//! - `types`: core types, enums, resources, and constants
//! - `advice_core`: helpers + finance/infrastructure/health/education advice
//! - `advice_city`: safety/environment/housing/traffic/zone-demand/fire-coverage advice

mod advice_city;
mod advice_core;
mod types;

// Re-export everything public so callers don't need to change imports.
pub use types::{
    AdvisorExtras, AdvisorJumpToLocation, AdvisorMessage, AdvisorPanel, AdvisorType,
    DismissedAdvisorTips, TipId,
};

use bevy::prelude::*;

use crate::economy::CityBudget;
use crate::grid::WorldGrid;
use crate::stats::CityStats;
use crate::TickCounter;

use advice_city::{
    environment_advice, fire_coverage_advice, housing_advice, safety_advice, traffic_advice,
    zone_demand_advice,
};
use advice_core::{education_advice, finance_advice, health_advice, infrastructure_advice};
use types::ADVISOR_INTERVAL;

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Analyzes city state every 200 ticks and generates contextual advisor messages.
#[allow(clippy::too_many_arguments)]
pub fn update_advisors(
    tick: Res<TickCounter>,
    stats: Res<CityStats>,
    budget: Res<CityBudget>,
    grid: Res<WorldGrid>,
    mut panel: ResMut<AdvisorPanel>,
    extras: AdvisorExtras,
    dismissed: Res<DismissedAdvisorTips>,
) {
    let t = tick.0;
    if !t.is_multiple_of(ADVISOR_INTERVAL) {
        return;
    }

    // Clear stale messages first
    panel.prune(t);

    // Collect new messages into a local vec, then push all at once
    let mut new_msgs: Vec<AdvisorMessage> = Vec::new();

    // ------ Finance ------
    finance_advice(t, &stats, &budget, &extras, &mut new_msgs);

    // ------ Infrastructure ------
    infrastructure_advice(t, &grid, &extras, &mut new_msgs);

    // ------ Health ------
    health_advice(t, &extras, &mut new_msgs);

    // ------ Education ------
    education_advice(t, &extras, &mut new_msgs);

    // ------ Safety ------
    safety_advice(t, &grid, &extras, &mut new_msgs);

    // ------ Environment ------
    environment_advice(t, &extras, &mut new_msgs);

    // ------ Housing ------
    housing_advice(t, &stats, &extras, &mut new_msgs);

    // ------ Traffic ------
    traffic_advice(t, &extras, &mut new_msgs);

    // ------ Zone Demand ------
    zone_demand_advice(t, &extras, &mut new_msgs);

    // ------ Fire Coverage ------
    fire_coverage_advice(t, &grid, &extras, &mut new_msgs);

    // Filter out dismissed tips
    new_msgs.retain(|msg| !dismissed.is_dismissed(msg.tip_id));

    for msg in new_msgs {
        panel.push(msg, t);
    }
}

// ---------------------------------------------------------------------------
// Saveable implementation for DismissedAdvisorTips
// ---------------------------------------------------------------------------

impl crate::Saveable for DismissedAdvisorTips {
    const SAVE_KEY: &'static str = "dismissed_advisor_tips";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.dismissed.is_empty() {
            return None;
        }
        Some(bitcode::encode(
            &self.dismissed.iter().copied().collect::<Vec<_>>(),
        ))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        let tips: Vec<TipId> = crate::decode_or_warn(Self::SAVE_KEY, bytes);
        DismissedAdvisorTips {
            dismissed: tips.into_iter().collect(),
        }
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct AdvisorsPlugin;

impl Plugin for AdvisorsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<crate::SaveableRegistry>()
            .init_resource::<AdvisorPanel>()
            .init_resource::<DismissedAdvisorTips>()
            .add_event::<AdvisorJumpToLocation>()
            .add_systems(
                FixedUpdate,
                update_advisors
                    .after(crate::stats::update_stats)
                    .in_set(crate::SimulationSet::PostSim),
            );

        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<DismissedAdvisorTips>();
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::advice_core::find_worst_cell;
    use super::types::MAX_MESSAGES;
    use super::*;

    #[test]
    fn test_advisor_panel_prune_removes_expired() {
        let mut panel = AdvisorPanel::default();
        panel.messages.push(AdvisorMessage {
            advisor_type: AdvisorType::Finance,
            tip_id: TipId::TreasuryCritical,
            message: "Old message".into(),
            priority: 3,
            suggestion: "Do something".into(),
            tick_created: 0,
            location: None,
        });
        panel.messages.push(AdvisorMessage {
            advisor_type: AdvisorType::Health,
            tip_id: TipId::HealthCoverageCritical,
            message: "New message".into(),
            priority: 4,
            suggestion: "Do something else".into(),
            tick_created: 600,
            location: None,
        });

        panel.prune(600);
        assert_eq!(panel.messages.len(), 1);
        assert_eq!(panel.messages[0].advisor_type, AdvisorType::Health);
    }

    #[test]
    fn test_advisor_panel_prune_sorts_by_priority() {
        let mut panel = AdvisorPanel::default();
        panel.messages.push(AdvisorMessage {
            advisor_type: AdvisorType::Education,
            tip_id: TipId::EducationLow,
            message: "Low priority".into(),
            priority: 1,
            suggestion: "".into(),
            tick_created: 100,
            location: None,
        });
        panel.messages.push(AdvisorMessage {
            advisor_type: AdvisorType::Finance,
            tip_id: TipId::TreasuryCritical,
            message: "High priority".into(),
            priority: 5,
            suggestion: "".into(),
            tick_created: 100,
            location: None,
        });
        panel.messages.push(AdvisorMessage {
            advisor_type: AdvisorType::Safety,
            tip_id: TipId::CrimeRising,
            message: "Medium priority".into(),
            priority: 3,
            suggestion: "".into(),
            tick_created: 100,
            location: None,
        });

        panel.prune(200);
        assert_eq!(panel.messages.len(), 3);
        assert_eq!(panel.messages[0].priority, 5);
        assert_eq!(panel.messages[1].priority, 3);
        assert_eq!(panel.messages[2].priority, 1);
    }

    #[test]
    fn test_advisor_panel_truncates_to_max() {
        let mut panel = AdvisorPanel::default();
        for i in 0..15 {
            panel.messages.push(AdvisorMessage {
                advisor_type: AdvisorType::Finance,
                tip_id: TipId::TreasuryCritical,
                message: format!("Message {}", i),
                priority: (i % 5 + 1) as u8,
                suggestion: "".into(),
                tick_created: 100,
                location: None,
            });
        }
        panel.prune(200);
        assert_eq!(panel.messages.len(), MAX_MESSAGES);
    }

    #[test]
    fn test_advisor_panel_push() {
        let mut panel = AdvisorPanel::default();
        let msg = AdvisorMessage {
            advisor_type: AdvisorType::Housing,
            tip_id: TipId::HomelessCritical,
            message: "Test".into(),
            priority: 4,
            suggestion: "Build more".into(),
            tick_created: 50,
            location: None,
        };
        panel.push(msg, 50);
        assert_eq!(panel.messages.len(), 1);
        assert_eq!(panel.messages[0].advisor_type, AdvisorType::Housing);
    }

    #[test]
    fn test_advisor_type_name() {
        assert_eq!(AdvisorType::Finance.name(), "Finance");
        assert_eq!(AdvisorType::Infrastructure.name(), "Infrastructure");
        assert_eq!(AdvisorType::Health.name(), "Health");
        assert_eq!(AdvisorType::Education.name(), "Education");
        assert_eq!(AdvisorType::Safety.name(), "Safety");
        assert_eq!(AdvisorType::Environment.name(), "Environment");
        assert_eq!(AdvisorType::Housing.name(), "Housing");
    }

    #[test]
    fn test_advisor_panel_default_is_empty() {
        let panel = AdvisorPanel::default();
        assert!(panel.messages.is_empty());
    }

    #[test]
    fn test_dismissed_tips_dismiss_and_restore() {
        let mut dismissed = DismissedAdvisorTips::default();
        assert!(!dismissed.is_dismissed(TipId::BudgetDeficit));

        dismissed.dismiss(TipId::BudgetDeficit);
        assert!(dismissed.is_dismissed(TipId::BudgetDeficit));

        dismissed.restore(TipId::BudgetDeficit);
        assert!(!dismissed.is_dismissed(TipId::BudgetDeficit));
    }

    #[test]
    fn test_dismissed_tips_restore_all() {
        let mut dismissed = DismissedAdvisorTips::default();
        dismissed.dismiss(TipId::BudgetDeficit);
        dismissed.dismiss(TipId::CrimeCritical);
        dismissed.dismiss(TipId::TrafficCongestion);
        assert_eq!(dismissed.dismissed.len(), 3);

        dismissed.restore_all();
        assert!(dismissed.dismissed.is_empty());
    }

    #[test]
    fn test_tip_id_labels() {
        let all_tips = [
            TipId::TreasuryCritical,
            TipId::BudgetDeficit,
            TipId::TrafficCongestion,
            TipId::FireCoverageGap,
            TipId::ZoneDemandResidential,
        ];
        for tip in all_tips {
            assert!(!tip.label().is_empty());
        }
    }

    #[test]
    fn test_advisor_message_with_location() {
        let msg = AdvisorMessage {
            advisor_type: AdvisorType::Safety,
            tip_id: TipId::ActiveFires,
            message: "Fire detected".into(),
            priority: 5,
            suggestion: "Build fire station".into(),
            tick_created: 100,
            location: Some((42, 87)),
        };
        assert_eq!(msg.location, Some((42, 87)));
    }

    #[test]
    fn test_find_worst_cell() {
        let mut levels = vec![0u8; 256 * 256];
        levels[100 * 256 + 50] = 200; // x=50, y=100
        let result = find_worst_cell(&levels, 256);
        assert_eq!(result, Some((50, 100)));
    }

    #[test]
    fn test_find_worst_cell_all_zero() {
        let levels = vec![0u8; 256 * 256];
        let result = find_worst_cell(&levels, 256);
        assert_eq!(result, None);
    }

    #[test]
    fn test_advisor_jump_event() {
        let event = AdvisorJumpToLocation {
            grid_x: 128,
            grid_y: 64,
        };
        assert_eq!(event.grid_x, 128);
        assert_eq!(event.grid_y, 64);
    }
}
