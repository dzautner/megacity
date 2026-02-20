use bevy::prelude::*;
use rand::Rng;

use crate::citizen::{Citizen, CitizenDetails};
use crate::economy::CityBudget;
use crate::imports_exports::TradeConnections;
use crate::stats::CityStats;
use crate::time_of_day::GameClock;
use crate::SlowTickTimer;

// =============================================================================
// Event Types
// =============================================================================

#[derive(Debug, Clone)]
pub enum CityEventType {
    MilestoneReached(String),   // "Reached Town (1,000 pop)"
    BuildingFire(usize, usize), // grid coords
    DisasterStrike(String),     // "Tornado hit downtown"
    NewPolicy(String),          // "Enacted Green Energy Policy"
    BudgetCrisis,               // treasury went negative
    PopulationBoom,             // rapid population growth
    Epidemic,                   // health crisis
    Festival,                   // happiness boost event
    EconomicBoom,               // trade income surge
    ResourceDepleted(String),   // "Oil deposit at (x,y) depleted"
}

// =============================================================================
// City Event
// =============================================================================

#[derive(Debug, Clone)]
pub struct CityEvent {
    pub event_type: CityEventType,
    pub day: u32,
    pub hour: f32,
    pub description: String,
}

// =============================================================================
// Event Journal Resource
// =============================================================================

#[derive(Resource)]
pub struct EventJournal {
    pub events: Vec<CityEvent>,
    pub max_events: usize,
}

impl Default for EventJournal {
    fn default() -> Self {
        Self {
            events: Vec::new(),
            max_events: 200,
        }
    }
}

impl EventJournal {
    /// Push a new event into the journal, trimming old events if over capacity.
    pub fn push(&mut self, event: CityEvent) {
        self.events.push(event);
        if self.events.len() > self.max_events {
            let excess = self.events.len() - self.max_events;
            self.events.drain(0..excess);
        }
    }
}

// =============================================================================
// Active City Effects Resource
// =============================================================================

#[derive(Resource, Default)]
pub struct ActiveCityEffects {
    pub festival_ticks: u32,
    pub economic_boom_ticks: u32,
    pub epidemic_ticks: u32,
}

// =============================================================================
// Population milestone tracking
// =============================================================================

const POPULATION_MILESTONES: &[(u32, &str)] = &[
    (1_000, "Town"),
    (5_000, "Small City"),
    (10_000, "City"),
    (25_000, "Large City"),
    (50_000, "Metropolis"),
    (100_000, "Major Metropolis"),
    (250_000, "Megacity"),
    (500_000, "Megalopolis"),
    (1_000_000, "World Capital"),
];

/// Tracks which population milestones have already been logged to the journal.
#[derive(Resource, Default)]
pub struct MilestoneTracker {
    pub reached_milestones: Vec<u32>,
}

// =============================================================================
// Random City Events System
// =============================================================================

/// Runs on SlowTickTimer ticks. Generates random city events and checks
/// for budget crises and population milestones.
#[allow(clippy::too_many_arguments)]
pub fn random_city_events(
    slow_tick: Res<SlowTickTimer>,
    clock: Res<GameClock>,
    stats: Res<CityStats>,
    budget: Res<CityBudget>,
    trade: Res<TradeConnections>,
    mut journal: ResMut<EventJournal>,
    mut effects: ResMut<ActiveCityEffects>,
    mut milestones: ResMut<MilestoneTracker>,
    mut citizens: Query<&mut CitizenDetails, With<Citizen>>,
) {
    if !slow_tick.should_run() {
        return;
    }

    let mut rng = rand::thread_rng();

    // --- Decrement active effect timers ---
    if effects.festival_ticks > 0 {
        effects.festival_ticks = effects.festival_ticks.saturating_sub(1);
    }
    if effects.economic_boom_ticks > 0 {
        effects.economic_boom_ticks = effects.economic_boom_ticks.saturating_sub(1);
    }
    if effects.epidemic_ticks > 0 {
        effects.epidemic_ticks = effects.epidemic_ticks.saturating_sub(1);
    }

    // --- Random events ---

    // 2% chance: Festival (boost happiness by 5 for all citizens for 10 ticks)
    if rng.gen::<f32>() < 0.02 {
        effects.festival_ticks = 10;
        for mut details in citizens.iter_mut() {
            details.happiness = (details.happiness + 5.0).min(100.0);
        }
        journal.push(CityEvent {
            event_type: CityEventType::Festival,
            day: clock.day,
            hour: clock.hour,
            description: "A city festival is underway! Citizens are happier.".to_string(),
        });
    }

    // 1% chance: Economic Boom (double trade income for 20 ticks)
    if rng.gen::<f32>() < 0.01 {
        effects.economic_boom_ticks = 20;
        journal.push(CityEvent {
            event_type: CityEventType::EconomicBoom,
            day: clock.day,
            hour: clock.hour,
            description: format!(
                "Economic boom! Trade income surges (export rate: {:.1}/building).",
                trade.export_income_per_industrial * 2.0
            ),
        });
    }

    // 0.5% chance: Epidemic (reduce health for citizens by 5)
    if rng.gen::<f32>() < 0.005 {
        effects.epidemic_ticks = 10;
        for mut details in citizens.iter_mut() {
            details.health = (details.health - 5.0).max(0.0);
        }
        journal.push(CityEvent {
            event_type: CityEventType::Epidemic,
            day: clock.day,
            hour: clock.hour,
            description: "An epidemic has broken out! Citizens' health is declining.".to_string(),
        });
    }

    // --- Budget crisis check ---
    if budget.treasury < 0.0 {
        // Only log once per crisis (check if the last event was already a budget crisis today)
        let already_logged = journal
            .events
            .last()
            .map(|e| matches!(e.event_type, CityEventType::BudgetCrisis) && e.day == clock.day)
            .unwrap_or(false);
        if !already_logged {
            journal.push(CityEvent {
                event_type: CityEventType::BudgetCrisis,
                day: clock.day,
                hour: clock.hour,
                description: format!(
                    "Budget crisis! Treasury is ${:.0}. Raise taxes or cut spending.",
                    budget.treasury
                ),
            });
        }
    }

    // --- Population milestone check ---
    for &(threshold, name) in POPULATION_MILESTONES {
        if stats.population >= threshold && !milestones.reached_milestones.contains(&threshold) {
            milestones.reached_milestones.push(threshold);
            let description = format!(
                "Reached {} ({} population)!",
                name,
                format_population(threshold)
            );
            journal.push(CityEvent {
                event_type: CityEventType::MilestoneReached(description.clone()),
                day: clock.day,
                hour: clock.hour,
                description,
            });
        }
    }
}

/// Format population numbers with commas for readability.
fn format_population(n: u32) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.0}K", n as f64 / 1_000.0)
    } else {
        format!("{}", n)
    }
}

// =============================================================================
// Apply Active Effects System
// =============================================================================

/// Applies ongoing effects from active city events each slow tick.
/// - Festival: +5 happiness boost (already applied on trigger, this maintains awareness)
/// - Economic Boom: doubled trade income is handled via the effects resource
/// - Epidemic: ongoing health drain while active
pub fn apply_active_effects(
    slow_tick: Res<SlowTickTimer>,
    effects: Res<ActiveCityEffects>,
    mut citizens: Query<&mut CitizenDetails, With<Citizen>>,
) {
    if !slow_tick.should_run() {
        return;
    }

    // Ongoing festival happiness boost (small per-tick bonus while active)
    if effects.festival_ticks > 0 {
        for mut details in citizens.iter_mut() {
            details.happiness = (details.happiness + 1.0).min(100.0);
        }
    }

    // Ongoing epidemic health drain while active
    if effects.epidemic_ticks > 0 {
        for mut details in citizens.iter_mut() {
            details.health = (details.health - 0.5).max(0.0);
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_journal_push_and_trim() {
        let mut journal = EventJournal {
            events: Vec::new(),
            max_events: 3,
        };

        for i in 0..5 {
            journal.push(CityEvent {
                event_type: CityEventType::Festival,
                day: i,
                hour: 12.0,
                description: format!("Event {}", i),
            });
        }

        assert_eq!(journal.events.len(), 3);
        // Oldest events should have been trimmed
        assert_eq!(journal.events[0].day, 2);
        assert_eq!(journal.events[1].day, 3);
        assert_eq!(journal.events[2].day, 4);
    }

    #[test]
    fn test_journal_default() {
        let journal = EventJournal::default();
        assert_eq!(journal.max_events, 200);
        assert!(journal.events.is_empty());
    }

    #[test]
    fn test_active_effects_default() {
        let effects = ActiveCityEffects::default();
        assert_eq!(effects.festival_ticks, 0);
        assert_eq!(effects.economic_boom_ticks, 0);
        assert_eq!(effects.epidemic_ticks, 0);
    }

    #[test]
    fn test_format_population() {
        assert_eq!(format_population(500), "500");
        assert_eq!(format_population(1_000), "1K");
        assert_eq!(format_population(10_000), "10K");
        assert_eq!(format_population(1_000_000), "1.0M");
    }
}

pub struct EventsPlugin;

impl Plugin for EventsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EventJournal>()
            .init_resource::<ActiveCityEffects>()
            .init_resource::<MilestoneTracker>()
            .add_systems(
                FixedUpdate,
                (random_city_events, apply_active_effects)
                    .chain()
                    .after(crate::stats::update_stats),
            );
    }
}
