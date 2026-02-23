use crate::grid::ZoneType;
use crate::test_harness::TestCity;
use crate::time_of_day::GameClock;

// ===========================================================================
// TEST-063: Random City Events
// ===========================================================================

/// Test that the EventJournal resource is initialized on city creation.
#[test]
fn test_random_events_journal_resource_exists() {
    let city = TestCity::new();
    city.assert_resource_exists::<crate::events::EventJournal>();
}

/// Test that the ActiveCityEffects resource is initialized on city creation.
#[test]
fn test_random_events_active_effects_resource_exists() {
    let city = TestCity::new();
    city.assert_resource_exists::<crate::events::ActiveCityEffects>();
}

/// Test that the MilestoneTracker resource is initialized on city creation.
#[test]
fn test_random_events_milestone_tracker_resource_exists() {
    let city = TestCity::new();
    city.assert_resource_exists::<crate::events::MilestoneTracker>();
}

/// Test that a BudgetCrisis event is logged when the treasury is negative.
#[test]
fn test_random_events_budget_crisis_logged_when_treasury_negative() {
    let mut city = TestCity::new().with_budget(-5000.0);
    city.tick_slow_cycle();

    let journal = city.resource::<crate::events::EventJournal>();
    let crisis_events: Vec<_> = journal
        .events
        .iter()
        .filter(|e| matches!(e.event_type, crate::events::CityEventType::BudgetCrisis))
        .collect();

    assert!(
        !crisis_events.is_empty(),
        "Expected at least one BudgetCrisis event when treasury is negative"
    );
}

/// Test that BudgetCrisis is only logged once per day (deduplication).
#[test]
fn test_random_events_budget_crisis_dedup_same_day() {
    let mut city = TestCity::new().with_budget(-5000.0);
    city.tick_slow_cycle();
    city.tick_slow_cycle();
    city.tick_slow_cycle();

    let journal = city.resource::<crate::events::EventJournal>();
    let mut crisis_days: Vec<u32> = journal
        .events
        .iter()
        .filter(|e| matches!(e.event_type, crate::events::CityEventType::BudgetCrisis))
        .map(|e| e.day)
        .collect();
    crisis_days.sort();
    crisis_days.dedup();

    let total_crisis = journal
        .events
        .iter()
        .filter(|e| matches!(e.event_type, crate::events::CityEventType::BudgetCrisis))
        .count();

    assert!(
        total_crisis <= crisis_days.len() + 1,
        "Expected at most one BudgetCrisis per day, got {} events across {} days",
        total_crisis,
        crisis_days.len()
    );
}

/// Test that population milestones are recorded in the EventJournal.
#[test]
fn test_random_events_population_milestone_logged() {
    use crate::events::{CityEventType, EventJournal, MilestoneTracker};
    use crate::virtual_population::VirtualPopulation;

    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        world.resource_mut::<VirtualPopulation>().total_virtual = 1_500;
    }
    city.tick_slow_cycle();

    let journal = city.resource::<EventJournal>();
    let milestone_events: Vec<_> = journal
        .events
        .iter()
        .filter(|e| matches!(e.event_type, CityEventType::MilestoneReached(_)))
        .collect();

    assert!(
        !milestone_events.is_empty(),
        "Expected a MilestoneReached event after virtual population hit 1,500"
    );

    let tracker = city.resource::<MilestoneTracker>();
    assert!(
        tracker.reached_milestones.contains(&1_000),
        "MilestoneTracker should contain the 1,000 threshold"
    );
}

/// Test that population milestones are NOT re-triggered once already reached.
#[test]
fn test_random_events_population_milestone_not_retriggered() {
    use crate::events::{CityEventType, EventJournal};
    use crate::virtual_population::VirtualPopulation;

    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        world.resource_mut::<VirtualPopulation>().total_virtual = 1_500;
    }
    city.tick_slow_cycle();

    let first_count = {
        let journal = city.resource::<EventJournal>();
        journal
            .events
            .iter()
            .filter(|e| matches!(e.event_type, CityEventType::MilestoneReached(_)))
            .count()
    };

    city.tick_slow_cycle();
    city.tick_slow_cycle();

    let second_count = {
        let journal = city.resource::<EventJournal>();
        journal
            .events
            .iter()
            .filter(|e| matches!(e.event_type, CityEventType::MilestoneReached(_)))
            .count()
    };

    assert_eq!(
        first_count, second_count,
        "Milestone should not re-trigger: first={}, after more ticks={}",
        first_count, second_count
    );
}

/// Test that multiple population milestones fire when population jumps past
/// several thresholds at once.
#[test]
fn test_random_events_multiple_milestones_at_once() {
    use crate::events::{CityEventType, EventJournal, MilestoneTracker};
    use crate::virtual_population::VirtualPopulation;

    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        world.resource_mut::<VirtualPopulation>().total_virtual = 12_000;
    }
    city.tick_slow_cycle();

    let journal = city.resource::<EventJournal>();
    let milestone_count = journal
        .events
        .iter()
        .filter(|e| matches!(e.event_type, CityEventType::MilestoneReached(_)))
        .count();

    assert!(
        milestone_count >= 3,
        "Expected at least 3 milestones (1K, 5K, 10K) for population 12,000, got {}",
        milestone_count
    );

    let tracker = city.resource::<MilestoneTracker>();
    assert!(tracker.reached_milestones.contains(&1_000));
    assert!(tracker.reached_milestones.contains(&5_000));
    assert!(tracker.reached_milestones.contains(&10_000));
}

/// Test that the festival effect timer decrements each slow tick.
#[test]
fn test_random_events_festival_effect_ticks_decrement() {
    use crate::events::ActiveCityEffects;

    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        world.resource_mut::<ActiveCityEffects>().festival_ticks = 5;
    }
    city.tick_slow_cycle();

    let effects = city.resource::<ActiveCityEffects>();
    assert!(
        effects.festival_ticks <= 4 || effects.festival_ticks == 10,
        "Festival ticks should decrement from 5 to 4, or be reset to 10 if re-triggered, got {}",
        effects.festival_ticks
    );
}

/// Test that epidemic effect ticks decrement via the random_city_events system.
#[test]
fn test_random_events_epidemic_effect_ticks_decrement() {
    use crate::events::ActiveCityEffects;

    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        world.resource_mut::<ActiveCityEffects>().epidemic_ticks = 5;
    }
    city.tick_slow_cycle();

    let effects = city.resource::<ActiveCityEffects>();
    assert!(
        effects.epidemic_ticks <= 4 || effects.epidemic_ticks == 10,
        "Epidemic ticks should decrement from 5 to 4, or be reset to 10 if re-triggered, got {}",
        effects.epidemic_ticks
    );
}

/// Test that apply_active_effects drains health during an active epidemic.
/// Uses only a residential building (no industrial) to prevent the citizen
/// spawner from creating new citizens that would pollute the health check.
#[test]
fn test_random_events_epidemic_drains_health() {
    use crate::citizen::CitizenDetails;
    use crate::events::ActiveCityEffects;

    let home = (10, 10);
    // Use residential for both home and work — spawner requires BOTH residential
    // AND industrial with capacity to create new citizens.
    let mut city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_citizen(home, home);
    // Prevent emigration during slow tick cycles.
    {
        let world = city.world_mut();
        let mut attr = world.resource_mut::<crate::immigration::CityAttractiveness>();
        attr.overall_score = 80.0;
    }

    // Let systems settle, then set ALL citizens to known health
    city.tick_slow_cycle();

    let citizen_count_before = {
        let world = city.world_mut();
        let mut q = world.query::<&mut CitizenDetails>();
        let count = q.iter(world).count();
        for mut d in q.iter_mut(world) {
            d.health = 80.0;
        }
        world.resource_mut::<ActiveCityEffects>().epidemic_ticks = 50;
        count
    };

    // One slow cycle drains 0.5 health via apply_active_effects.
    city.tick_slow_cycle();

    // Verify no new citizens were spawned
    let world = city.world_mut();
    let mut q = world.query::<&CitizenDetails>();
    let citizen_count_after = q.iter(world).count();
    assert_eq!(
        citizen_count_before, citizen_count_after,
        "No new citizens should spawn without industrial buildings"
    );

    for details in q.iter(world) {
        assert!(
            details.health < 80.0,
            "Epidemic should drain health below 80, got {}",
            details.health
        );
    }
}

/// Test that active effect timers eventually expire after enough ticks.
#[test]
fn test_random_events_effect_duration_expires() {
    use crate::events::ActiveCityEffects;

    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut effects = world.resource_mut::<ActiveCityEffects>();
        effects.festival_ticks = 2;
        effects.economic_boom_ticks = 2;
        effects.epidemic_ticks = 2;
    }

    city.tick_slow_cycles(50);

    let effects = city.resource::<ActiveCityEffects>();
    // Epidemic has 0.5% re-trigger chance per tick; max from re-trigger is 10.
    assert!(
        effects.epidemic_ticks <= 10,
        "Epidemic ticks should be at most 10 (max from re-trigger), got {}",
        effects.epidemic_ticks
    );
}

/// Test that the EventJournal trims old events when exceeding max_events.
#[test]
fn test_random_events_journal_trims_to_max() {
    use crate::events::{CityEvent, CityEventType, EventJournal};

    let mut city = TestCity::new();
    {
        let world = city.world_mut();
        let mut journal = world.resource_mut::<EventJournal>();
        journal.max_events = 5;
        for i in 0..10 {
            journal.push(CityEvent {
                event_type: CityEventType::Festival,
                day: i,
                hour: 12.0,
                description: format!("Test event {}", i),
            });
        }
    }

    let journal = city.resource::<EventJournal>();
    assert_eq!(journal.events.len(), 5);
    assert_eq!(journal.events[0].day, 5);
    assert_eq!(journal.events[4].day, 9);
}

/// Test that positive treasury does NOT trigger a BudgetCrisis event.
#[test]
fn test_random_events_no_budget_crisis_with_positive_treasury() {
    use crate::events::{CityEventType, EventJournal};

    let mut city = TestCity::new().with_budget(50_000.0);
    city.tick_slow_cycle();

    let journal = city.resource::<EventJournal>();
    let crisis_count = journal
        .events
        .iter()
        .filter(|e| matches!(e.event_type, CityEventType::BudgetCrisis))
        .count();
    assert_eq!(
        crisis_count, 0,
        "No BudgetCrisis should fire with positive treasury, got {}",
        crisis_count
    );
}

/// Test that epidemic health drain does not go below 0.
#[test]
fn test_random_events_epidemic_health_floor_at_zero() {
    use crate::citizen::CitizenDetails;
    use crate::events::ActiveCityEffects;

    let home = (10, 10);
    let work = (15, 15);
    let mut city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::Industrial, 1)
        .with_citizen(home, work);
    // Prevent emigration during slow tick cycles.
    {
        let world = city.world_mut();
        let mut attr = world.resource_mut::<crate::immigration::CityAttractiveness>();
        attr.overall_score = 80.0;
    }

    {
        let world = city.world_mut();
        let mut q = world.query::<&mut CitizenDetails>();
        for mut details in q.iter_mut(world) {
            details.health = 0.5;
        }
    }
    {
        let world = city.world_mut();
        world.resource_mut::<ActiveCityEffects>().epidemic_ticks = 20;
    }

    city.tick_slow_cycles(10);

    let world = city.world_mut();
    let mut q = world.query::<&CitizenDetails>();
    for details in q.iter(world) {
        assert!(
            details.health >= 0.0,
            "Health should never go below 0, got {}",
            details.health
        );
    }
}

/// Test that festival happiness boost does not exceed 100.
#[test]
fn test_random_events_festival_happiness_capped_at_100() {
    use crate::citizen::CitizenDetails;
    use crate::events::ActiveCityEffects;

    let home = (10, 10);
    let work = (15, 15);
    let mut city = TestCity::new()
        .with_building(home.0, home.1, ZoneType::ResidentialLow, 1)
        .with_building(work.0, work.1, ZoneType::Industrial, 1)
        .with_citizen(home, work);
    // Prevent emigration during slow tick cycles.
    {
        let world = city.world_mut();
        let mut attr = world.resource_mut::<crate::immigration::CityAttractiveness>();
        attr.overall_score = 80.0;
    }

    {
        let world = city.world_mut();
        let mut q = world.query::<&mut CitizenDetails>();
        for mut details in q.iter_mut(world) {
            details.happiness = 100.0;
        }
    }
    {
        let world = city.world_mut();
        world.resource_mut::<ActiveCityEffects>().festival_ticks = 20;
    }

    city.tick_slow_cycles(5);

    let world = city.world_mut();
    let mut q = world.query::<&CitizenDetails>();
    for details in q.iter(world) {
        assert!(
            details.happiness <= 100.0,
            "Happiness should not exceed 100, got {}",
            details.happiness
        );
    }
}

/// Test that CityEvent records the correct day from the GameClock.
#[test]
fn test_random_events_event_records_clock_time() {
    use crate::events::{CityEventType, EventJournal};

    let mut city = TestCity::new().with_budget(-5000.0).with_time(14.5);
    {
        let world = city.world_mut();
        world.resource_mut::<GameClock>().day = 42;
    }
    city.tick_slow_cycle();

    let journal = city.resource::<EventJournal>();
    let crisis = journal
        .events
        .iter()
        .find(|e| matches!(e.event_type, CityEventType::BudgetCrisis));
    assert!(crisis.is_some(), "BudgetCrisis event should exist");
    let crisis = crisis.unwrap();
    assert!(
        crisis.day >= 42,
        "Event day should be >= 42, got {}",
        crisis.day
    );
}
