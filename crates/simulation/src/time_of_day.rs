use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct GameClock {
    pub day: u32,
    pub hour: f32,
    pub speed: f32,
    pub paused: bool,
}

impl Default for GameClock {
    fn default() -> Self {
        Self {
            day: 1,
            hour: 6.0, // start at 6 AM
            speed: 1.0,
            paused: false,
        }
    }
}

impl GameClock {
    /// Minutes per sim tick at 1x speed
    const MINUTES_PER_TICK: f32 = 1.0;

    pub fn tick(&mut self) {
        if self.paused {
            return;
        }
        // Speed is handled by scaling the FixedUpdate timestep (sync_fixed_timestep),
        // so each tick always advances by the same amount of game time.
        self.hour += Self::MINUTES_PER_TICK / 60.0;
        if self.hour >= 24.0 {
            self.hour -= 24.0;
            self.day += 1;
        }
    }

    pub fn hour_of_day(&self) -> u32 {
        self.hour as u32
    }

    pub fn is_morning_commute(&self) -> bool {
        let h = self.hour_of_day();
        (7..=8).contains(&h)
    }

    pub fn is_evening_commute(&self) -> bool {
        let h = self.hour_of_day();
        (17..=18).contains(&h)
    }

    pub fn formatted(&self) -> String {
        let h = self.hour as u32;
        let m = ((self.hour - h as f32) * 60.0) as u32;
        format!("Day {} {:02}:{:02}", self.day, h, m)
    }
}

pub fn tick_game_clock(mut clock: ResMut<GameClock>) {
    clock.tick();
}

/// Scales the FixedUpdate timestep based on GameClock speed.
/// Base rate is 10 Hz (100 ms). At 2x speed it becomes 50 ms, at 4x -> 25 ms, etc.
pub fn sync_fixed_timestep(
    clock: Res<GameClock>,
    mut time: ResMut<Time<Fixed>>,
) {
    let base_hz = std::time::Duration::from_millis(100); // 10 Hz
    let effective = if clock.paused || clock.speed <= 0.0 {
        // When paused, keep the timestep but the tick_game_clock won't advance
        base_hz
    } else {
        base_hz.div_f32(clock.speed.clamp(0.25, 16.0))
    };
    time.set_timestep(effective);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clock_wraps_at_24h() {
        let mut clock = GameClock {
            day: 1,
            hour: 23.9,
            speed: 1.0,
            paused: false,
        };
        // Tick enough to pass midnight
        for _ in 0..20 {
            clock.tick();
        }
        assert_eq!(clock.day, 2);
        assert!(clock.hour < 24.0);
        assert!(clock.hour >= 0.0);
    }

    #[test]
    fn test_clock_paused() {
        let mut clock = GameClock {
            paused: true,
            ..Default::default()
        };
        let hour_before = clock.hour;
        clock.tick();
        assert_eq!(clock.hour, hour_before);
    }

    #[test]
    fn test_commute_times() {
        let clock = GameClock {
            hour: 7.5,
            ..Default::default()
        };
        assert!(clock.is_morning_commute());
        assert!(!clock.is_evening_commute());

        let clock2 = GameClock {
            hour: 17.5,
            ..Default::default()
        };
        assert!(!clock2.is_morning_commute());
        assert!(clock2.is_evening_commute());
    }
}
