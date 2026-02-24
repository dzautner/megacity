//! Cultural Buildings Prestige system (SVC-014).
//!
//! Cultural buildings (Museum, Cathedral, Stadium, TVStation) provide city-wide
//! prestige bonuses and unique effects:
//!
//! - **Museum**: +5 education quality boost in radius, +10 tourism attraction
//! - **Cathedral**: +8 tourism, community building effect
//! - **Stadium**: periodic events with city-wide happiness (+3) and tourism (+15)
//! - **TVStation**: immigration visibility boost (+5)
//!
//! A `CulturalPrestige` resource tracks the aggregate prestige score derived
//! from cultural building count, and feeds into tourism and immigration systems.

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::education::EducationGrid;
use crate::immigration::CityAttractiveness;
use crate::services::{ServiceBuilding, ServiceType};
use crate::tourism::Tourism;
use crate::TickCounter;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Prestige points per cultural building type.
const MUSEUM_PRESTIGE: f32 = 15.0;
const CATHEDRAL_PRESTIGE: f32 = 12.0;
const STADIUM_PRESTIGE: f32 = 10.0;
const TV_STATION_PRESTIGE: f32 = 8.0;

/// Museum education bonus: applied to cells within this grid radius.
const MUSEUM_EDUCATION_RADIUS: usize = 20;
/// Education level boost from museums (added to existing education level, capped at 3).
const MUSEUM_EDUCATION_BOOST: u8 = 1;

/// Stadium event interval in ticks.
const STADIUM_EVENT_INTERVAL: u64 = 500;
/// Stadium event duration in ticks.
const STADIUM_EVENT_DURATION: u64 = 100;
/// City-wide happiness bonus during stadium events.
const STADIUM_EVENT_HAPPINESS_BONUS: f32 = 3.0;
/// Tourism visitor multiplier during stadium events.
const STADIUM_EVENT_TOURISM_MULTIPLIER: f32 = 1.15;

/// TV Station immigration attractiveness bonus (added to overall score).
const TV_STATION_IMMIGRATION_BONUS: f32 = 5.0;

/// How often (in ticks) the prestige system recalculates.
const PRESTIGE_UPDATE_INTERVAL: u64 = 100;

/// Tourism bonus per point of prestige (scaled into cultural_facilities_score).
const PRESTIGE_TOURISM_FACTOR: f32 = 0.5;

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// City-wide cultural prestige derived from cultural buildings.
#[derive(Resource, Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct CulturalPrestige {
    /// Aggregate prestige score (0+).
    pub prestige_score: f32,
    /// Number of museums in the city.
    pub museum_count: u32,
    /// Number of cathedrals in the city.
    pub cathedral_count: u32,
    /// Number of stadiums in the city.
    pub stadium_count: u32,
    /// Number of TV stations in the city.
    pub tv_station_count: u32,
    /// Whether a stadium event is currently active.
    pub stadium_event_active: bool,
    /// Tick when the current/last stadium event started.
    pub stadium_event_start_tick: u64,
    /// City-wide happiness bonus from active stadium events.
    pub active_happiness_bonus: f32,
    /// Tourism multiplier from active stadium events.
    pub active_tourism_multiplier: f32,
}

impl Default for CulturalPrestige {
    fn default() -> Self {
        Self {
            prestige_score: 0.0,
            museum_count: 0,
            cathedral_count: 0,
            stadium_count: 0,
            tv_station_count: 0,
            stadium_event_active: false,
            stadium_event_start_tick: 0,
            active_happiness_bonus: 0.0,
            active_tourism_multiplier: 1.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Saveable
// ---------------------------------------------------------------------------

impl crate::Saveable for CulturalPrestige {
    const SAVE_KEY: &'static str = "cultural_prestige";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.prestige_score == 0.0
            && self.museum_count == 0
            && self.cathedral_count == 0
            && self.stadium_count == 0
            && self.tv_station_count == 0
        {
            return None;
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Compute prestige score from cultural building counts and manage stadium events.
pub fn update_cultural_prestige(
    tick: Res<TickCounter>,
    services: Query<&ServiceBuilding>,
    mut prestige: ResMut<CulturalPrestige>,
) {
    if !tick.0.is_multiple_of(PRESTIGE_UPDATE_INTERVAL) {
        return;
    }

    // Count cultural buildings
    let mut museums = 0u32;
    let mut cathedrals = 0u32;
    let mut stadiums = 0u32;
    let mut tv_stations = 0u32;

    for service in &services {
        match service.service_type {
            ServiceType::Museum => museums += 1,
            ServiceType::Cathedral => cathedrals += 1,
            ServiceType::Stadium => stadiums += 1,
            ServiceType::TVStation => tv_stations += 1,
            _ => {}
        }
    }

    prestige.museum_count = museums;
    prestige.cathedral_count = cathedrals;
    prestige.stadium_count = stadiums;
    prestige.tv_station_count = tv_stations;

    // Calculate prestige score
    prestige.prestige_score = museums as f32 * MUSEUM_PRESTIGE
        + cathedrals as f32 * CATHEDRAL_PRESTIGE
        + stadiums as f32 * STADIUM_PRESTIGE
        + tv_stations as f32 * TV_STATION_PRESTIGE;

    // Stadium event management
    if stadiums > 0 {
        if prestige.stadium_event_active {
            // Check if event should end
            let elapsed = tick.0.saturating_sub(prestige.stadium_event_start_tick);
            if elapsed >= STADIUM_EVENT_DURATION {
                prestige.stadium_event_active = false;
                prestige.active_happiness_bonus = 0.0;
                prestige.active_tourism_multiplier = 1.0;
            }
        } else {
            // Check if a new event should start
            let since_last = tick.0.saturating_sub(prestige.stadium_event_start_tick);
            if since_last >= STADIUM_EVENT_INTERVAL {
                prestige.stadium_event_active = true;
                prestige.stadium_event_start_tick = tick.0;
                prestige.active_happiness_bonus = STADIUM_EVENT_HAPPINESS_BONUS;
                prestige.active_tourism_multiplier = STADIUM_EVENT_TOURISM_MULTIPLIER;
            }
        }
    } else {
        prestige.stadium_event_active = false;
        prestige.active_happiness_bonus = 0.0;
        prestige.active_tourism_multiplier = 1.0;
    }
}

/// Boost education grid near museums.
///
/// Runs on the slow tick alongside education propagation.
pub fn museum_education_boost(
    slow_tick: Res<crate::SlowTickTimer>,
    services: Query<&ServiceBuilding>,
    mut edu_grid: ResMut<EducationGrid>,
) {
    if !slow_tick.should_run() {
        return;
    }

    for service in &services {
        if service.service_type != ServiceType::Museum {
            continue;
        }

        let cx = service.grid_x;
        let cy = service.grid_y;
        let r = MUSEUM_EDUCATION_RADIUS;

        let x_min = cx.saturating_sub(r);
        let x_max = (cx + r).min(edu_grid.width - 1);
        let y_min = cy.saturating_sub(r);
        let y_max = (cy + r).min(edu_grid.height - 1);

        for y in y_min..=y_max {
            for x in x_min..=x_max {
                let dx = (x as i32 - cx as i32).unsigned_abs() as usize;
                let dy = (y as i32 - cy as i32).unsigned_abs() as usize;
                if dx * dx + dy * dy <= r * r {
                    let current = edu_grid.get(x, y);
                    if current > 0 && current < 3 {
                        edu_grid.set(x, y, (current + MUSEUM_EDUCATION_BOOST).min(3));
                    }
                }
            }
        }
    }
}

/// Apply cultural prestige bonus to tourism.
///
/// Adds a prestige-based bonus to the tourism cultural facilities score
/// after the main tourism system has run.
pub fn cultural_tourism_bonus(
    tick: Res<TickCounter>,
    prestige: Res<CulturalPrestige>,
    mut tourism: ResMut<Tourism>,
) {
    // Runs at the same interval as the prestige system
    if !tick.0.is_multiple_of(PRESTIGE_UPDATE_INTERVAL) {
        return;
    }

    // Add prestige bonus to cultural facilities score (clamped to 100)
    let bonus = prestige.prestige_score * PRESTIGE_TOURISM_FACTOR;
    tourism.cultural_facilities_score =
        (tourism.cultural_facilities_score + bonus).clamp(0.0, 100.0);

    // Apply stadium event tourism multiplier to visitor count
    if prestige.stadium_event_active {
        let boosted = (tourism.monthly_visitors as f32 * prestige.active_tourism_multiplier) as u32;
        tourism.monthly_visitors = boosted;
    }
}

/// Apply TV station immigration visibility boost.
///
/// Each TV station adds a flat bonus to the city's overall attractiveness
/// score, making the city more visible to potential immigrants.
pub fn tv_station_immigration_boost(
    tick: Res<TickCounter>,
    prestige: Res<CulturalPrestige>,
    mut attractiveness: ResMut<CityAttractiveness>,
) {
    if !tick.0.is_multiple_of(PRESTIGE_UPDATE_INTERVAL) {
        return;
    }

    if prestige.tv_station_count > 0 {
        // Diminishing returns: first TV station gives full bonus, subsequent ones less
        let bonus = TV_STATION_IMMIGRATION_BONUS
            * (1.0 - (-0.5 * prestige.tv_station_count as f32).exp());
        attractiveness.overall_score =
            (attractiveness.overall_score + bonus).clamp(0.0, 100.0);
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct CulturalBuildingsPlugin;

impl Plugin for CulturalBuildingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CulturalPrestige>();

        app.add_systems(
            FixedUpdate,
            (
                update_cultural_prestige,
                cultural_tourism_bonus
                    .after(crate::tourism::update_tourism)
                    .after(update_cultural_prestige),
                tv_station_immigration_boost.after(update_cultural_prestige),
                museum_education_boost.after(crate::education::propagate_education),
            )
                .in_set(crate::SimulationSet::Simulation),
        );

        // Register for save/load
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<CulturalPrestige>();
    }
}
