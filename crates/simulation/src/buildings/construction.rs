use bevy::prelude::*;

use crate::weather::ConstructionModifiers;

use super::types::{Building, UnderConstruction};

/// Advances construction progress each tick. When complete, removes the
/// `UnderConstruction` component so the building becomes operational.
/// While under construction, occupants are clamped to 0.
///
/// Progress is scaled by `ConstructionModifiers::speed_factor`:
/// - 0.0 = halted (storm), no progress
/// - 0.5 = half speed (rain), progress every other tick
/// - 1.0+ = normal or faster
pub fn progress_construction(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Building, &mut UnderConstruction)>,
    modifiers: Res<ConstructionModifiers>,
    tick: Res<crate::TickCounter>,
) {
    let speed = modifiers.speed_factor;

    for (entity, mut building, mut uc) in &mut query {
        // Ensure no occupants while under construction
        building.occupants = 0;

        if uc.ticks_remaining > 0 {
            // Determine whether to make progress this tick based on speed_factor.
            // speed >= 1.0: always progress (1 tick per tick)
            // 0 < speed < 1: progress on a fraction of ticks using modular arithmetic
            // speed == 0.0: halted (storm)
            let should_progress = if speed <= 0.0 {
                false
            } else if speed >= 1.0 {
                true
            } else {
                // Use tick counter to distribute progress evenly.
                // E.g., speed=0.5 -> progress every 2nd tick; speed=0.3 -> every ~3rd tick.
                let period = (1.0 / speed).round() as u64;
                period > 0 && tick.0.is_multiple_of(period)
            };

            if should_progress {
                uc.ticks_remaining -= 1;
            }
        }

        if uc.ticks_remaining == 0 {
            commands.entity(entity).remove::<UnderConstruction>();
        }
    }
}
