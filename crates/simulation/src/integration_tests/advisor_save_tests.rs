//! Integration tests for advisor panel save/load roundtrips.
//!
//! Verifies that `DismissedAdvisorTips` and `AdvisorPanel` state
//! persists correctly across save/load cycles.

use crate::advisors::{
    AdvisorMessage, AdvisorPanel, AdvisorType, DismissedAdvisorTips, TipId,
};
use crate::test_harness::TestCity;
use crate::SaveableRegistry;

// ====================================================================
// Roundtrip helper
// ====================================================================

fn roundtrip(city: &mut TestCity) {
    let world = city.world_mut();
    let registry = world.remove_resource::<SaveableRegistry>().unwrap();

    let extensions = registry.save_all(world);

    registry.reset_all(world);
    registry.load_all(world, &extensions);

    world.insert_resource(registry);
}

// ====================================================================
// DismissedAdvisorTips roundtrip
// ====================================================================

#[test]
fn test_dismissed_advisor_tips_save_load_roundtrip() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut dismissed = world.resource_mut::<DismissedAdvisorTips>();
        dismissed.dismiss(TipId::BudgetDeficit);
        dismissed.dismiss(TipId::CrimeCritical);
        dismissed.dismiss(TipId::TrafficCongestion);
    }

    roundtrip(&mut city);

    let dismissed = city.resource::<DismissedAdvisorTips>();
    assert!(dismissed.is_dismissed(TipId::BudgetDeficit));
    assert!(dismissed.is_dismissed(TipId::CrimeCritical));
    assert!(dismissed.is_dismissed(TipId::TrafficCongestion));
    assert!(!dismissed.is_dismissed(TipId::TreasuryCritical));
}

#[test]
fn test_dismissed_tips_empty_skips_save() {
    let mut city = TestCity::new();

    // With no dismissed tips, save_to_bytes should return None.
    let dismissed = city.resource::<DismissedAdvisorTips>();
    assert!(dismissed.dismissed.is_empty());

    roundtrip(&mut city);

    let dismissed = city.resource::<DismissedAdvisorTips>();
    assert!(dismissed.dismissed.is_empty());
}

// ====================================================================
// AdvisorPanel roundtrip
// ====================================================================

#[test]
fn test_advisor_panel_save_load_roundtrip() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut panel = world.resource_mut::<AdvisorPanel>();
        panel.messages.push(AdvisorMessage {
            advisor_type: AdvisorType::Finance,
            tip_id: TipId::TreasuryCritical,
            message: "Treasury is critically low!".to_string(),
            priority: 5,
            suggestion: "Raise taxes or cut spending.".to_string(),
            tick_created: 200,
            location: None,
        });
        panel.messages.push(AdvisorMessage {
            advisor_type: AdvisorType::Safety,
            tip_id: TipId::CrimeCritical,
            message: "Crime is out of control!".to_string(),
            priority: 4,
            suggestion: "Build more police stations.".to_string(),
            tick_created: 400,
            location: Some((128, 64)),
        });
    }

    roundtrip(&mut city);

    let panel = city.resource::<AdvisorPanel>();
    assert_eq!(panel.messages.len(), 2);

    assert_eq!(panel.messages[0].advisor_type, AdvisorType::Finance);
    assert_eq!(panel.messages[0].tip_id, TipId::TreasuryCritical);
    assert_eq!(panel.messages[0].message, "Treasury is critically low!");
    assert_eq!(panel.messages[0].priority, 5);
    assert_eq!(panel.messages[0].tick_created, 200);
    assert_eq!(panel.messages[0].location, None);

    assert_eq!(panel.messages[1].advisor_type, AdvisorType::Safety);
    assert_eq!(panel.messages[1].tip_id, TipId::CrimeCritical);
    assert_eq!(panel.messages[1].location, Some((128, 64)));
}

#[test]
fn test_advisor_panel_empty_skips_save() {
    let mut city = TestCity::new();

    let panel = city.resource::<AdvisorPanel>();
    assert!(panel.messages.is_empty());

    roundtrip(&mut city);

    let panel = city.resource::<AdvisorPanel>();
    assert!(panel.messages.is_empty());
}

#[test]
fn test_advisor_panel_no_duplicates_after_load() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut panel = world.resource_mut::<AdvisorPanel>();
        panel.messages.push(AdvisorMessage {
            advisor_type: AdvisorType::Housing,
            tip_id: TipId::HomelessCritical,
            message: "Homelessness is critical.".to_string(),
            priority: 4,
            suggestion: "Zone more residential.".to_string(),
            tick_created: 100,
            location: None,
        });
    }

    // Roundtrip twice to ensure no duplication occurs.
    roundtrip(&mut city);
    roundtrip(&mut city);

    let panel = city.resource::<AdvisorPanel>();
    assert_eq!(panel.messages.len(), 1);
    assert_eq!(panel.messages[0].tip_id, TipId::HomelessCritical);
}

// ====================================================================
// Combined: dismissed tips + panel history
// ====================================================================

#[test]
fn test_advisor_save_load_combined() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();

        let mut dismissed = world.resource_mut::<DismissedAdvisorTips>();
        dismissed.dismiss(TipId::PollutionHigh);
        dismissed.dismiss(TipId::EducationLow);

        let mut panel = world.resource_mut::<AdvisorPanel>();
        panel.messages.push(AdvisorMessage {
            advisor_type: AdvisorType::Environment,
            tip_id: TipId::PollutionRising,
            message: "Pollution is rising.".to_string(),
            priority: 3,
            suggestion: "Plant trees.".to_string(),
            tick_created: 300,
            location: Some((50, 50)),
        });
    }

    roundtrip(&mut city);

    // Dismissed tips preserved.
    let dismissed = city.resource::<DismissedAdvisorTips>();
    assert!(dismissed.is_dismissed(TipId::PollutionHigh));
    assert!(dismissed.is_dismissed(TipId::EducationLow));
    assert!(!dismissed.is_dismissed(TipId::PollutionRising));

    // Panel history preserved.
    let panel = city.resource::<AdvisorPanel>();
    assert_eq!(panel.messages.len(), 1);
    assert_eq!(panel.messages[0].tip_id, TipId::PollutionRising);
    assert_eq!(panel.messages[0].location, Some((50, 50)));
}
