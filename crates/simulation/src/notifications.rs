//! UX-008: Notification System with Priority and Navigation.
//!
//! Provides categorized notifications with priority levels, location data,
//! and an event journal for persistent logging. Other simulation systems
//! emit `NotificationEvent`s which are collected into `NotificationLog`.
//!
//! Emergency notifications persist until manually dismissed; lower-priority
//! notifications auto-dismiss after a configurable timer.

use bevy::prelude::*;

use crate::time_of_day::GameClock;
use crate::SlowTickTimer;

// =============================================================================
// Priority Levels
// =============================================================================

/// Notification priority, from most to least urgent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NotificationPriority {
    /// Critical city-wide emergencies (fires, disasters). Persists until dismissed.
    Emergency,
    /// Significant warnings (budget deficit, service failures).
    Warning,
    /// Noteworthy situations that need attention.
    Attention,
    /// General information (new buildings, milestones).
    Info,
    /// Good news (population growth, surplus).
    Positive,
}

impl NotificationPriority {
    /// Auto-dismiss duration in simulation ticks. `None` means persist until dismissed.
    pub fn auto_dismiss_ticks(&self) -> Option<u32> {
        match self {
            NotificationPriority::Emergency => None, // persist until dismissed
            NotificationPriority::Warning => Some(1500), // ~150 seconds
            NotificationPriority::Attention => Some(1000), // ~100 seconds
            NotificationPriority::Info => Some(600), // ~60 seconds
            NotificationPriority::Positive => Some(600), // ~60 seconds
        }
    }

    /// Short label for display.
    pub fn label(&self) -> &'static str {
        match self {
            NotificationPriority::Emergency => "EMERGENCY",
            NotificationPriority::Warning => "WARNING",
            NotificationPriority::Attention => "ATTENTION",
            NotificationPriority::Info => "INFO",
            NotificationPriority::Positive => "POSITIVE",
        }
    }
}

// =============================================================================
// Notification Struct
// =============================================================================

/// A single notification with text, priority, optional world location, and timing.
#[derive(Debug, Clone)]
pub struct Notification {
    /// Unique ID for this notification.
    pub id: u64,
    /// Human-readable notification text.
    pub text: String,
    /// Priority level (determines color, auto-dismiss, and ordering).
    pub priority: NotificationPriority,
    /// Optional world-space location (x, z) to jump camera to on click.
    pub location: Option<(f32, f32)>,
    /// Game day when the notification was created.
    pub day: u32,
    /// Game hour when the notification was created.
    pub hour: f32,
    /// Tick when the notification was created (used for auto-dismiss timing).
    pub created_tick: u64,
    /// Whether the notification has been dismissed by the user.
    pub dismissed: bool,
}

// =============================================================================
// Journal Entry (archived notification)
// =============================================================================

/// An archived notification stored in the persistent event journal.
#[derive(Debug, Clone)]
pub struct JournalEntry {
    pub text: String,
    pub priority: NotificationPriority,
    pub location: Option<(f32, f32)>,
    pub day: u32,
    pub hour: f32,
}

// =============================================================================
// Bevy Event
// =============================================================================

/// Event emitted by other systems to create a notification.
///
/// # Example
/// ```ignore
/// fn my_system(mut events: EventWriter<NotificationEvent>) {
///     events.send(NotificationEvent {
///         text: "Fire in sector 7!".to_string(),
///         priority: NotificationPriority::Emergency,
///         location: Some((128.0, 256.0)),
///     });
/// }
/// ```
#[derive(Event, Debug, Clone)]
pub struct NotificationEvent {
    pub text: String,
    pub priority: NotificationPriority,
    /// Optional world-space location (x, z).
    pub location: Option<(f32, f32)>,
}

// =============================================================================
// NotificationLog Resource
// =============================================================================

/// Active notifications and archived journal entries.
#[derive(Resource)]
pub struct NotificationLog {
    /// Currently active (visible) notifications.
    pub active: Vec<Notification>,
    /// Archived journal of all past notifications.
    pub journal: Vec<JournalEntry>,
    /// Maximum journal size before old entries are trimmed.
    pub max_journal: usize,
    /// Next notification ID counter.
    next_id: u64,
}

impl Default for NotificationLog {
    fn default() -> Self {
        Self {
            active: Vec::new(),
            journal: Vec::new(),
            max_journal: 500,
            next_id: 1,
        }
    }
}

impl NotificationLog {
    /// Allocate the next unique notification ID.
    fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Add a notification from an event and return a reference to it.
    pub fn push(&mut self, event: &NotificationEvent, clock: &GameClock, tick: u64) {
        let id = self.next_id();
        self.active.push(Notification {
            id,
            text: event.text.clone(),
            priority: event.priority,
            location: event.location,
            day: clock.day,
            hour: clock.hour,
            created_tick: tick,
            dismissed: false,
        });

        // Also archive immediately in the journal
        self.journal.push(JournalEntry {
            text: event.text.clone(),
            priority: event.priority,
            location: event.location,
            day: clock.day,
            hour: clock.hour,
        });

        // Trim journal if over capacity
        if self.journal.len() > self.max_journal {
            let excess = self.journal.len() - self.max_journal;
            self.journal.drain(0..excess);
        }
    }

    /// Dismiss a notification by ID (moves it out of active list).
    pub fn dismiss(&mut self, id: u64) {
        if let Some(n) = self.active.iter_mut().find(|n| n.id == id) {
            n.dismissed = true;
        }
    }

    /// Remove all dismissed and auto-expired notifications from the active list.
    pub fn sweep(&mut self, current_tick: u64) {
        self.active.retain(|n| {
            if n.dismissed {
                return false;
            }
            if let Some(ttl) = n.priority.auto_dismiss_ticks() {
                let elapsed = current_tick.saturating_sub(n.created_tick);
                if elapsed >= ttl as u64 {
                    return false;
                }
            }
            true
        });
    }
}

// =============================================================================
// Systems
// =============================================================================

/// Collects `NotificationEvent`s and pushes them into `NotificationLog`.
fn collect_notifications(
    mut events: EventReader<NotificationEvent>,
    mut log: ResMut<NotificationLog>,
    clock: Res<GameClock>,
    tick: Res<crate::TickCounter>,
) {
    for event in events.read() {
        log.push(event, &clock, tick.0);
    }
}

/// Periodically sweeps expired notifications from the active list.
fn sweep_expired_notifications(
    mut log: ResMut<NotificationLog>,
    tick: Res<crate::TickCounter>,
    slow: Res<SlowTickTimer>,
) {
    // Run every 10 ticks (~1 second) rather than every tick, for efficiency
    if tick.0 % 10 != 0 && !slow.should_run() {
        return;
    }
    log.sweep(tick.0);
}

// =============================================================================
// Plugin
// =============================================================================

pub struct NotificationsPlugin;

impl Plugin for NotificationsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NotificationLog>()
            .add_event::<NotificationEvent>()
            .add_systems(
                FixedUpdate,
                (collect_notifications, sweep_expired_notifications).chain(),
            );
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_ordering() {
        assert!(NotificationPriority::Emergency < NotificationPriority::Warning);
        assert!(NotificationPriority::Warning < NotificationPriority::Attention);
        assert!(NotificationPriority::Attention < NotificationPriority::Info);
        assert!(NotificationPriority::Info < NotificationPriority::Positive);
    }

    #[test]
    fn test_emergency_persists() {
        assert!(NotificationPriority::Emergency
            .auto_dismiss_ticks()
            .is_none());
    }

    #[test]
    fn test_info_auto_dismisses() {
        assert!(NotificationPriority::Info.auto_dismiss_ticks().is_some());
    }

    #[test]
    fn test_notification_log_push_and_journal() {
        let mut log = NotificationLog::default();
        let clock = GameClock::default();
        let event = NotificationEvent {
            text: "Test notification".to_string(),
            priority: NotificationPriority::Info,
            location: Some((100.0, 200.0)),
        };
        log.push(&event, &clock, 0);

        assert_eq!(log.active.len(), 1);
        assert_eq!(log.journal.len(), 1);
        assert_eq!(log.active[0].text, "Test notification");
        assert_eq!(log.journal[0].text, "Test notification");
    }

    #[test]
    fn test_notification_dismiss() {
        let mut log = NotificationLog::default();
        let clock = GameClock::default();
        let event = NotificationEvent {
            text: "Dismiss me".to_string(),
            priority: NotificationPriority::Emergency,
            location: None,
        };
        log.push(&event, &clock, 0);
        let id = log.active[0].id;

        log.dismiss(id);
        assert!(log.active[0].dismissed);

        log.sweep(0);
        assert!(log.active.is_empty());
        // Journal persists
        assert_eq!(log.journal.len(), 1);
    }

    #[test]
    fn test_sweep_auto_dismiss() {
        let mut log = NotificationLog::default();
        let clock = GameClock::default();

        // Info notification with 600 tick TTL
        let event = NotificationEvent {
            text: "Info event".to_string(),
            priority: NotificationPriority::Info,
            location: None,
        };
        log.push(&event, &clock, 100);

        // Not expired yet
        log.sweep(500);
        assert_eq!(log.active.len(), 1);

        // Expired (100 + 600 = 700)
        log.sweep(701);
        assert!(log.active.is_empty());
    }

    #[test]
    fn test_emergency_never_auto_expires() {
        let mut log = NotificationLog::default();
        let clock = GameClock::default();

        let event = NotificationEvent {
            text: "Emergency!".to_string(),
            priority: NotificationPriority::Emergency,
            location: Some((50.0, 50.0)),
        };
        log.push(&event, &clock, 0);

        // Even after a very long time, emergency persists
        log.sweep(999_999);
        assert_eq!(log.active.len(), 1);
    }

    #[test]
    fn test_journal_trimming() {
        let mut log = NotificationLog::default();
        log.max_journal = 5;
        let clock = GameClock::default();

        for i in 0..10 {
            let event = NotificationEvent {
                text: format!("Event {}", i),
                priority: NotificationPriority::Info,
                location: None,
            };
            log.push(&event, &clock, i);
        }

        assert_eq!(log.journal.len(), 5);
        assert_eq!(log.journal[0].text, "Event 5"); // oldest kept
        assert_eq!(log.journal[4].text, "Event 9"); // newest
    }

    #[test]
    fn test_unique_ids() {
        let mut log = NotificationLog::default();
        let clock = GameClock::default();

        for _ in 0..5 {
            let event = NotificationEvent {
                text: "test".to_string(),
                priority: NotificationPriority::Info,
                location: None,
            };
            log.push(&event, &clock, 0);
        }

        let ids: Vec<u64> = log.active.iter().map(|n| n.id).collect();
        for i in 0..ids.len() {
            for j in (i + 1)..ids.len() {
                assert_ne!(ids[i], ids[j], "IDs must be unique");
            }
        }
    }
}
