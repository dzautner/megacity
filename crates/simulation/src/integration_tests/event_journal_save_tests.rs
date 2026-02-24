//! Integration tests for EventJournal, ActiveCityEffects, and MilestoneTracker
//! save/load roundtrips (SAVE-027).

use crate::events::{
    ActiveCityEffects, CityEvent, CityEventType, EventJournal, MilestoneTracker,
};
use crate::test_harness::TestCity;
use crate::SaveableRegistry;

// ====================================================================
// Roundtrip helper
// ====================================================================

/// Save all registered saveables, reset them, then restore from the saved
/// bytes. Operates entirely through `world_mut()`.
fn roundtrip(city: &mut TestCity) {
    let world = city.world_mut();
    let registry = world.remove_resource::<SaveableRegistry>().unwrap();
    let extensions = registry.save_all(world);
    registry.reset_all(world);
    registry.load_all(world, &extensions);
    world.insert_resource(registry);
}

// ====================================================================
// EventJournal roundtrip tests
// ====================================================================

#[test]
fn test_event_journal_save_load_roundtrip() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut journal = world.resource_mut::<EventJournal>();
        journal.push(CityEvent {
            event_type: CityEventType::Festival,
            day: 1,
            hour: 10.0,
            description: "Festival on day 1".to_string(),
        });
        journal.push(CityEvent {
            event_type: CityEventType::BudgetCrisis,
            day: 2,
            hour: 14.5,
            description: "Budget crisis on day 2".to_string(),
        });
        journal.push(CityEvent {
            event_type: CityEventType::MilestoneReached("Town".to_string()),
            day: 3,
            hour: 8.0,
            description: "Reached Town".to_string(),
        });
    }

    roundtrip(&mut city);

    let journal = city.resource::<EventJournal>();
    assert_eq!(journal.events.len(), 3, "All 3 events should survive roundtrip");
    assert_eq!(journal.events[0].day, 1);
    assert_eq!(journal.events[1].day, 2);
    assert_eq!(journal.events[2].day, 3);
    assert_eq!(journal.events[0].description, "Festival on day 1");
    assert_eq!(journal.events[1].description, "Budget crisis on day 2");
    assert_eq!(journal.events[2].description, "Reached Town");
}

#[test]
fn test_event_journal_empty_skips_save() {
    let mut city = TestCity::new();

    // Journal is empty by default; save should produce None for the key
    let world = city.world_mut();
    let registry = world.remove_resource::<SaveableRegistry>().unwrap();
    let extensions = registry.save_all(world);
    world.insert_resource(registry);

    assert!(
        !extensions.contains_key("event_journal"),
        "Empty journal should not produce save data"
    );
}

#[test]
fn test_event_journal_preserves_event_types() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut journal = world.resource_mut::<EventJournal>();
        journal.push(CityEvent {
            event_type: CityEventType::BuildingFire(10, 20),
            day: 5,
            hour: 3.0,
            description: "Fire!".to_string(),
        });
        journal.push(CityEvent {
            event_type: CityEventType::EconomicBoom,
            day: 6,
            hour: 12.0,
            description: "Boom!".to_string(),
        });
        journal.push(CityEvent {
            event_type: CityEventType::Epidemic,
            day: 7,
            hour: 0.0,
            description: "Epidemic!".to_string(),
        });
        journal.push(CityEvent {
            event_type: CityEventType::DisasterStrike("Tornado".to_string()),
            day: 8,
            hour: 6.0,
            description: "Tornado!".to_string(),
        });
        journal.push(CityEvent {
            event_type: CityEventType::ResourceDepleted("Oil".to_string()),
            day: 9,
            hour: 9.0,
            description: "Oil depleted".to_string(),
        });
    }

    roundtrip(&mut city);

    let journal = city.resource::<EventJournal>();
    assert_eq!(journal.events.len(), 5);
    assert!(matches!(journal.events[0].event_type, CityEventType::BuildingFire(10, 20)));
    assert!(matches!(journal.events[1].event_type, CityEventType::EconomicBoom));
    assert!(matches!(journal.events[2].event_type, CityEventType::Epidemic));
    assert!(matches!(
        journal.events[3].event_type,
        CityEventType::DisasterStrike(ref s) if s == "Tornado"
    ));
    assert!(matches!(
        journal.events[4].event_type,
        CityEventType::ResourceDepleted(ref s) if s == "Oil"
    ));
}

#[test]
fn test_event_journal_max_events_preserved() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut journal = world.resource_mut::<EventJournal>();
        journal.max_events = 50;
        journal.push(CityEvent {
            event_type: CityEventType::Festival,
            day: 1,
            hour: 0.0,
            description: "Test".to_string(),
        });
    }

    roundtrip(&mut city);

    let journal = city.resource::<EventJournal>();
    assert_eq!(journal.max_events, 50, "max_events should roundtrip");
}

// ====================================================================
// ActiveCityEffects roundtrip tests
// ====================================================================

#[test]
fn test_active_city_effects_save_load_roundtrip() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut effects = world.resource_mut::<ActiveCityEffects>();
        effects.festival_ticks = 7;
        effects.economic_boom_ticks = 15;
        effects.epidemic_ticks = 3;
    }

    roundtrip(&mut city);

    let effects = city.resource::<ActiveCityEffects>();
    assert_eq!(effects.festival_ticks, 7);
    assert_eq!(effects.economic_boom_ticks, 15);
    assert_eq!(effects.epidemic_ticks, 3);
}

#[test]
fn test_active_city_effects_default_skips_save() {
    let mut city = TestCity::new();

    let world = city.world_mut();
    let registry = world.remove_resource::<SaveableRegistry>().unwrap();
    let extensions = registry.save_all(world);
    world.insert_resource(registry);

    assert!(
        !extensions.contains_key("active_city_effects"),
        "Default effects (all zeros) should not produce save data"
    );
}

// ====================================================================
// MilestoneTracker roundtrip tests
// ====================================================================

#[test]
fn test_milestone_tracker_save_load_roundtrip() {
    let mut city = TestCity::new();

    {
        let world = city.world_mut();
        let mut tracker = world.resource_mut::<MilestoneTracker>();
        tracker.reached_milestones.push(1_000);
        tracker.reached_milestones.push(5_000);
        tracker.reached_milestones.push(10_000);
    }

    roundtrip(&mut city);

    let tracker = city.resource::<MilestoneTracker>();
    assert_eq!(tracker.reached_milestones.len(), 3);
    assert!(tracker.reached_milestones.contains(&1_000));
    assert!(tracker.reached_milestones.contains(&5_000));
    assert!(tracker.reached_milestones.contains(&10_000));
}

#[test]
fn test_milestone_tracker_empty_skips_save() {
    let mut city = TestCity::new();

    let world = city.world_mut();
    let registry = world.remove_resource::<SaveableRegistry>().unwrap();
    let extensions = registry.save_all(world);
    world.insert_resource(registry);

    assert!(
        !extensions.contains_key("milestone_tracker"),
        "Empty milestone tracker should not produce save data"
    );
}

#[test]
fn test_milestone_tracker_prevents_re_trigger_after_load() {
    use crate::virtual_population::VirtualPopulation;

    let mut city = TestCity::new();

    // Simulate reaching a milestone
    {
        let world = city.world_mut();
        world.resource_mut::<VirtualPopulation>().total_virtual = 1_500;
    }
    city.tick_slow_cycle();

    let events_before = {
        let journal = city.resource::<EventJournal>();
        journal
            .events
            .iter()
            .filter(|e| matches!(e.event_type, CityEventType::MilestoneReached(_)))
            .count()
    };

    // Roundtrip
    roundtrip(&mut city);

    // Tick again — milestone should NOT re-fire
    city.tick_slow_cycle();

    let events_after = {
        let journal = city.resource::<EventJournal>();
        journal
            .events
            .iter()
            .filter(|e| matches!(e.event_type, CityEventType::MilestoneReached(_)))
            .count()
    };

    assert_eq!(
        events_before, events_after,
        "Milestones should not re-trigger after save/load"
    );
}

// ====================================================================
// Event history visible after load
// ====================================================================

#[test]
fn test_event_history_visible_after_load() {
    let mut city = TestCity::new();

    // Add some events
    {
        let world = city.world_mut();
        let mut journal = world.resource_mut::<EventJournal>();
        for i in 0..5 {
            journal.push(CityEvent {
                event_type: CityEventType::NewPolicy(format!("Policy {}", i)),
                day: i,
                hour: 12.0,
                description: format!("Enacted Policy {}", i),
            });
        }
    }

    roundtrip(&mut city);

    // Verify all events are accessible
    let journal = city.resource::<EventJournal>();
    assert_eq!(journal.events.len(), 5);
    for (i, event) in journal.events.iter().enumerate() {
        assert_eq!(event.day, i as u32);
        assert_eq!(event.description, format!("Enacted Policy {}", i));
    }
}
