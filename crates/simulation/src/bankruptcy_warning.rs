//! PLAY-021: Bankruptcy and Game Over Warning
//!
//! Monitors the city treasury and emits notifications when funds are running low.
//! Tracks a `BankruptcyLevel` state machine with four tiers:
//!
//! - **Normal**: Treasury >= $5,000
//! - **Warning**: Treasury < $5,000 (yellow notification)
//! - **Critical**: Treasury < $1,000 (red notification)
//! - **Bankrupt**: Treasury <= $0 (emergency notification)
//!
//! Notifications are emitted once per state transition (not every tick).
//! The state is saved/loaded so players resume at the correct warning level.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::economy::CityBudget;
use crate::notifications::{NotificationEvent, NotificationPriority};
use crate::SlowTickTimer;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Treasury threshold below which we enter Warning level.
pub const WARNING_THRESHOLD: f64 = 5_000.0;
/// Treasury threshold below which we enter Critical level.
pub const CRITICAL_THRESHOLD: f64 = 1_000.0;

// ---------------------------------------------------------------------------
// Bankruptcy level enum
// ---------------------------------------------------------------------------

/// Current financial health of the city.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, Encode, Decode,
)]
pub enum BankruptcyLevel {
    #[default]
    Normal,
    Warning,
    Critical,
    Bankrupt,
}

impl BankruptcyLevel {
    /// Determine the appropriate level for a given treasury balance.
    pub fn from_treasury(treasury: f64) -> Self {
        if treasury <= 0.0 {
            BankruptcyLevel::Bankrupt
        } else if treasury < CRITICAL_THRESHOLD {
            BankruptcyLevel::Critical
        } else if treasury < WARNING_THRESHOLD {
            BankruptcyLevel::Warning
        } else {
            BankruptcyLevel::Normal
        }
    }
}

// ---------------------------------------------------------------------------
// Resource
// ---------------------------------------------------------------------------

/// Tracks the current bankruptcy warning level for the city.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize, Encode, Decode)]
pub struct BankruptcyState {
    pub level: BankruptcyLevel,
}

// ---------------------------------------------------------------------------
// Saveable implementation
// ---------------------------------------------------------------------------

impl crate::Saveable for BankruptcyState {
    const SAVE_KEY: &'static str = "bankruptcy_state";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.level == BankruptcyLevel::Normal {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Checks treasury balance each slow tick and emits notifications on state
/// transitions. Only fires once per transition to avoid notification spam.
pub fn check_bankruptcy(
    slow_timer: Res<SlowTickTimer>,
    budget: Res<CityBudget>,
    mut state: ResMut<BankruptcyState>,
    mut notifications: EventWriter<NotificationEvent>,
) {
    if !slow_timer.should_run() {
        return;
    }

    let new_level = BankruptcyLevel::from_treasury(budget.treasury);

    if new_level == state.level {
        return;
    }

    let old_level = state.level;
    state.level = new_level;

    // Only emit notifications for worsening conditions or recovery
    match new_level {
        BankruptcyLevel::Warning => {
            notifications.send(NotificationEvent {
                text: format!(
                    "Treasury low: ${:.0}. Consider raising taxes or cutting expenses.",
                    budget.treasury
                ),
                priority: NotificationPriority::Attention,
                location: None,
            });
        }
        BankruptcyLevel::Critical => {
            notifications.send(NotificationEvent {
                text: format!(
                    "Treasury critical: ${:.0}! The city is running out of money.",
                    budget.treasury
                ),
                priority: NotificationPriority::Warning,
                location: None,
            });
        }
        BankruptcyLevel::Bankrupt => {
            notifications.send(NotificationEvent {
                text: "The city is bankrupt! Services will deteriorate without funds.".to_string(),
                priority: NotificationPriority::Emergency,
                location: None,
            });
        }
        BankruptcyLevel::Normal => {
            // Only notify recovery if we were in a bad state before
            if old_level != BankruptcyLevel::Normal {
                notifications.send(NotificationEvent {
                    text: "Treasury has recovered to healthy levels.".to_string(),
                    priority: NotificationPriority::Positive,
                    location: None,
                });
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct BankruptcyWarningPlugin;

impl Plugin for BankruptcyWarningPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BankruptcyState>().add_systems(
            FixedUpdate,
            check_bankruptcy
                .after(crate::economy::collect_taxes)
                .in_set(crate::SimulationSet::PostSim),
        );

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<BankruptcyState>();
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Saveable;

    #[test]
    fn test_level_from_treasury() {
        assert_eq!(BankruptcyLevel::from_treasury(10000.0), BankruptcyLevel::Normal);
        assert_eq!(BankruptcyLevel::from_treasury(5000.0), BankruptcyLevel::Normal);
        assert_eq!(BankruptcyLevel::from_treasury(4999.0), BankruptcyLevel::Warning);
        assert_eq!(BankruptcyLevel::from_treasury(1000.0), BankruptcyLevel::Warning);
        assert_eq!(BankruptcyLevel::from_treasury(999.0), BankruptcyLevel::Critical);
        assert_eq!(BankruptcyLevel::from_treasury(1.0), BankruptcyLevel::Critical);
        assert_eq!(BankruptcyLevel::from_treasury(0.0), BankruptcyLevel::Bankrupt);
        assert_eq!(BankruptcyLevel::from_treasury(-500.0), BankruptcyLevel::Bankrupt);
    }

    #[test]
    fn test_default_state_is_normal() {
        let state = BankruptcyState::default();
        assert_eq!(state.level, BankruptcyLevel::Normal);
    }

    #[test]
    fn test_saveable_roundtrip() {
        let state = BankruptcyState {
            level: BankruptcyLevel::Critical,
        };
        let bytes = state.save_to_bytes().expect("should produce bytes");
        let restored = BankruptcyState::load_from_bytes(&bytes);
        assert_eq!(restored.level, BankruptcyLevel::Critical);
    }

    #[test]
    fn test_normal_state_save_returns_none() {
        let state = BankruptcyState::default();
        assert!(state.save_to_bytes().is_none());
    }

    #[test]
    fn test_saveable_key() {
        assert_eq!(BankruptcyState::SAVE_KEY, "bankruptcy_state");
    }
}
