use bevy::prelude::*;

use crate::buildings::Building;
use crate::citizen::{
    Citizen, CitizenDetails, Family, HomeLocation, Needs, Personality, WorkLocation,
};
use crate::grid::WorldGrid;
use crate::time_of_day::GameClock;

use super::LifeSimTimer;

// ---------------------------------------------------------------------------
// System: retire_workers
// Citizens who reach retirement age lose their WorkLocation.
// ---------------------------------------------------------------------------

pub fn retire_workers(
    clock: Res<GameClock>,
    slow_tick: Res<crate::SlowTickTimer>,
    mut commands: Commands,
    citizens: Query<(Entity, &CitizenDetails, &WorkLocation), With<Citizen>>,
    mut buildings: Query<&mut Building>,
) {
    if clock.paused {
        return;
    }
    if !slow_tick.should_run() {
        return;
    }

    for (entity, details, work) in &citizens {
        if details.age >= 65 {
            if let Ok(mut building) = buildings.get_mut(work.building) {
                building.occupants = building.occupants.saturating_sub(1);
            }
            commands.entity(entity).remove::<WorkLocation>();
        }
    }
}

// ---------------------------------------------------------------------------
// System: evolve_personality
// Life experiences shape personality over time.
// Repeated failure erodes ambition. Loneliness reduces sociability.
// Success reinforces traits. Hardship builds resilience (or breaks it).
// ---------------------------------------------------------------------------

#[allow(clippy::type_complexity)]
pub fn evolve_personality(
    clock: Res<GameClock>,
    mut timer: ResMut<LifeSimTimer>,
    mut citizens: Query<
        (
            &mut Personality,
            &CitizenDetails,
            &Needs,
            Option<&WorkLocation>,
            &Family,
        ),
        With<Citizen>,
    >,
) {
    if clock.paused {
        return;
    }
    timer.personality_tick += 1;
    if timer.personality_tick < super::PERSONALITY_INTERVAL {
        return;
    }
    timer.personality_tick = 0;

    // Small drift per cycle (personality changes slowly)
    const DRIFT: f32 = 0.01;

    for (mut personality, details, needs, work, family) in &mut citizens {
        // --- Ambition ---
        // Unemployed working-age adult: ambition erodes from repeated failure
        if details.life_stage().can_work() && work.is_none() {
            personality.ambition = (personality.ambition - DRIFT * 2.0).max(0.05);
        }
        // Well-educated + employed: ambition reinforced
        if work.is_some() && details.education >= 2 {
            personality.ambition = (personality.ambition + DRIFT * 0.5).min(1.0);
        }
        // Low happiness for extended periods: ambition fades
        if details.happiness < 30.0 {
            personality.ambition = (personality.ambition - DRIFT).max(0.05);
        }

        // --- Sociability ---
        // Chronically lonely (low social need): sociability decreases (withdrawal)
        if needs.social < 20.0 {
            personality.sociability = (personality.sociability - DRIFT).max(0.05);
        }
        // Has partner and children: sociability reinforced
        if family.partner.is_some() && !family.children.is_empty() {
            personality.sociability = (personality.sociability + DRIFT * 0.5).min(1.0);
        }
        // Isolated (no partner, no children): sociability slowly fades
        if family.partner.is_none() && family.children.is_empty() && details.age > 35 {
            personality.sociability = (personality.sociability - DRIFT * 0.3).max(0.05);
        }

        // --- Materialism ---
        // High savings: materialism increases (got used to comfort)
        if details.savings > details.salary * 12.0 {
            personality.materialism = (personality.materialism + DRIFT * 0.3).min(1.0);
        }
        // Low savings / low salary: materialism can go either way
        // but generally increases (want what you don't have)
        if details.savings < details.salary && details.life_stage().can_work() {
            personality.materialism = (personality.materialism + DRIFT * 0.5).min(1.0);
        }

        // --- Resilience ---
        // Surviving hardship builds resilience (low happiness but still here)
        if details.happiness < 40.0 && details.health > 50.0 {
            personality.resilience = (personality.resilience + DRIFT * 0.5).min(1.0);
        }
        // Prolonged comfort weakens resilience
        if details.happiness > 80.0 && needs.overall_satisfaction() > 0.8 {
            personality.resilience = (personality.resilience - DRIFT * 0.2).max(0.05);
        }
        // Very poor health breaks resilience
        if details.health < 30.0 {
            personality.resilience = (personality.resilience - DRIFT).max(0.05);
        }

        // Aging naturally shifts personality
        if details.age > 55 {
            // Older people tend to become less ambitious, more resilient
            personality.ambition = (personality.ambition - DRIFT * 0.2).max(0.05);
            personality.resilience = (personality.resilience + DRIFT * 0.3).min(1.0);
        }
    }
}

// ---------------------------------------------------------------------------
// System: update_health
// Health changes based on age, needs, pollution, services, and lifestyle.
// ---------------------------------------------------------------------------

pub fn update_health(
    clock: Res<GameClock>,
    mut timer: ResMut<LifeSimTimer>,
    grid: Res<WorldGrid>,
    pollution_grid: Res<crate::pollution::PollutionGrid>,
    coverage: Res<crate::happiness::ServiceCoverageGrid>,
    mut citizens: Query<(&mut CitizenDetails, &HomeLocation, &Needs, &Personality), With<Citizen>>,
) {
    if clock.paused {
        return;
    }
    timer.health_tick += 1;
    if timer.health_tick < super::HEALTH_INTERVAL {
        return;
    }
    timer.health_tick = 0;

    for (mut details, home, needs, personality) in &mut citizens {
        let mut health_delta: f32 = 0.0;

        // Natural aging: health degrades over time, especially after 50
        if details.age > 50 {
            health_delta -= (details.age as f32 - 50.0) * 0.02;
        }

        // Starvation (very low hunger) damages health
        if needs.hunger < 15.0 {
            health_delta -= 2.0;
        } else if needs.hunger < 30.0 {
            health_delta -= 0.5;
        }

        // Exhaustion (very low energy) damages health
        if needs.energy < 15.0 {
            health_delta -= 1.5;
        }

        // Good needs satisfaction helps recovery
        if needs.overall_satisfaction() > 0.7 {
            health_delta += 0.5;
        }

        // Pollution exposure at home
        let poll = pollution_grid.get(home.grid_x, home.grid_y) as f32;
        if poll > 50.0 {
            health_delta -= (poll - 50.0) * 0.02;
        }

        // Healthcare coverage helps recovery
        let idx = home.grid_y * grid.width + home.grid_x;
        if coverage.has_health(idx) {
            health_delta += 0.3;
            // Healthcare helps more when sick
            if details.health < 50.0 {
                health_delta += 0.5;
            }
        }

        // Resilient personalities resist health decline
        health_delta *= 1.0 - (personality.resilience * 0.3);

        details.health = (details.health + health_delta).clamp(0.0, 100.0);

        // Very low health increases unhappiness (handled in happiness system via needs)
        // and eventually leads to death (handled in lifecycle)
    }
}
