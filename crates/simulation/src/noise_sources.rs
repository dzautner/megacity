//! POLL-019: Complete Noise Source Type Table
//!
//! Defines a 17-source noise emission table with dB levels and activity
//! patterns. Inactive sources produce no noise outside their time window.

use bevy::prelude::*;

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::noise::{attenuated_db, db_to_grid_u8, max_radius, NoisePollutionGrid};
use crate::time_of_day::GameClock;

// ---------------------------------------------------------------------------
// Activity patterns
// ---------------------------------------------------------------------------

/// When a noise source is active (produces noise).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityPattern {
    /// Active 06:00-22:00
    Daytime,
    /// Active 22:00-06:00
    Nighttime,
    /// Active 24 hours
    Always,
    /// Active during events (evening hours 18:00-23:00)
    EventDriven,
}

impl ActivityPattern {
    pub fn is_active(self, hour: f32) -> bool {
        match self {
            Self::Always => true,
            Self::Daytime => (6.0..22.0).contains(&hour),
            Self::Nighttime => !(6.0..22.0).contains(&hour),
            Self::EventDriven => (18.0..23.0).contains(&hour),
        }
    }
}

// ---------------------------------------------------------------------------
// Noise source definition
// ---------------------------------------------------------------------------

/// A single entry in the noise source type table.
#[derive(Debug, Clone, Copy)]
pub struct NoiseSourceEntry {
    pub source_type: NoiseSourceType,
    pub db_level: f32,
    pub activity: ActivityPattern,
}

/// All 17 noise source types from the specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NoiseSourceType {
    Highway,
    Arterial,
    LocalRoad,
    RailCorridor,
    Airport,
    Construction,
    HeavyIndustry,
    LightIndustry,
    CommercialHvac,
    Nightclub,
    FireStation,
    PowerPlant,
    Stadium,
    School,
    Park,
    ParkingStructure,
    TrainStation,
}

// ---------------------------------------------------------------------------
// Source table
// ---------------------------------------------------------------------------

/// The full 17-source noise emission table.
pub const NOISE_SOURCE_TABLE: [NoiseSourceEntry; 17] = [
    NoiseSourceEntry { source_type: NoiseSourceType::Highway, db_level: 75.0, activity: ActivityPattern::Always },
    NoiseSourceEntry { source_type: NoiseSourceType::Arterial, db_level: 70.0, activity: ActivityPattern::Always },
    NoiseSourceEntry { source_type: NoiseSourceType::LocalRoad, db_level: 55.0, activity: ActivityPattern::Always },
    NoiseSourceEntry { source_type: NoiseSourceType::RailCorridor, db_level: 80.0, activity: ActivityPattern::Always },
    NoiseSourceEntry { source_type: NoiseSourceType::Airport, db_level: 105.0, activity: ActivityPattern::Always },
    NoiseSourceEntry { source_type: NoiseSourceType::Construction, db_level: 90.0, activity: ActivityPattern::Daytime },
    NoiseSourceEntry { source_type: NoiseSourceType::HeavyIndustry, db_level: 85.0, activity: ActivityPattern::Always },
    NoiseSourceEntry { source_type: NoiseSourceType::LightIndustry, db_level: 70.0, activity: ActivityPattern::Daytime },
    NoiseSourceEntry { source_type: NoiseSourceType::CommercialHvac, db_level: 60.0, activity: ActivityPattern::Daytime },
    NoiseSourceEntry { source_type: NoiseSourceType::Nightclub, db_level: 95.0, activity: ActivityPattern::Nighttime },
    NoiseSourceEntry { source_type: NoiseSourceType::FireStation, db_level: 80.0, activity: ActivityPattern::Always },
    NoiseSourceEntry { source_type: NoiseSourceType::PowerPlant, db_level: 75.0, activity: ActivityPattern::Always },
    NoiseSourceEntry { source_type: NoiseSourceType::Stadium, db_level: 95.0, activity: ActivityPattern::EventDriven },
    NoiseSourceEntry { source_type: NoiseSourceType::School, db_level: 70.0, activity: ActivityPattern::Daytime },
    NoiseSourceEntry { source_type: NoiseSourceType::Park, db_level: 35.0, activity: ActivityPattern::Daytime },
    NoiseSourceEntry { source_type: NoiseSourceType::ParkingStructure, db_level: 65.0, activity: ActivityPattern::Daytime },
    NoiseSourceEntry { source_type: NoiseSourceType::TrainStation, db_level: 75.0, activity: ActivityPattern::Always },
];

// ---------------------------------------------------------------------------
// Lookup helpers
// ---------------------------------------------------------------------------

/// Look up a noise source entry by type.
pub fn lookup_source(source_type: NoiseSourceType) -> Option<&'static NoiseSourceEntry> {
    NOISE_SOURCE_TABLE
        .iter()
        .find(|e| e.source_type == source_type)
}

/// Effective dB for a source type at the given hour. Returns 0.0 if inactive.
pub fn effective_db(source_type: NoiseSourceType, hour: f32) -> f32 {
    match lookup_source(source_type) {
        Some(entry) if entry.activity.is_active(hour) => entry.db_level,
        _ => 0.0,
    }
}

// ---------------------------------------------------------------------------
// Resource
// ---------------------------------------------------------------------------

/// Runtime-accessible noise source table resource.
#[derive(Resource, Debug)]
pub struct NoiseSourceTableRes {
    pub entries: &'static [NoiseSourceEntry; 17],
}

impl Default for NoiseSourceTableRes {
    fn default() -> Self {
        Self { entries: &NOISE_SOURCE_TABLE }
    }
}

impl NoiseSourceTableRes {
    pub fn effective_db(&self, source_type: NoiseSourceType, hour: f32) -> f32 {
        effective_db(source_type, hour)
    }

    pub fn active_sources(&self, hour: f32) -> Vec<&NoiseSourceEntry> {
        self.entries.iter().filter(|e| e.activity.is_active(hour)).collect()
    }
}

// ---------------------------------------------------------------------------
// Propagation helper (uses public API from noise module)
// ---------------------------------------------------------------------------

fn propagate(noise: &mut NoisePollutionGrid, sx: usize, sy: usize, source_db: f32) {
    let radius = max_radius(source_db);
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let nx = sx as i32 + dx;
            let ny = sy as i32 + dy;
            if nx < 0 || ny < 0 || nx as usize >= GRID_WIDTH || ny as usize >= GRID_HEIGHT {
                continue;
            }
            let dist = ((dx * dx + dy * dy) as f32).sqrt();
            let db = attenuated_db(source_db, dist);
            if db > 0.0 {
                let val = db_to_grid_u8(db);
                if val > 0 {
                    let idx = ny as usize * noise.width + nx as usize;
                    noise.levels[idx] = noise.levels[idx].saturating_add(val).min(100);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// System
// ---------------------------------------------------------------------------

/// Adds noise from sources not covered by the base noise system (fire
/// stations, power plants, schools, train stations, parks) with activity-
/// pattern awareness based on time of day.
pub fn apply_noise_source_table(
    slow_timer: Res<crate::SlowTickTimer>,
    clock: Res<GameClock>,
    mut noise: ResMut<NoisePollutionGrid>,
    services: Query<&crate::services::ServiceBuilding>,
    utilities: Query<&crate::utilities::UtilitySource>,
    _table: Res<NoiseSourceTableRes>,
) {
    if !slow_timer.should_run() {
        return;
    }

    let hour = clock.hour;

    for service in &services {
        let source_type = match service.service_type {
            crate::services::ServiceType::FireStation
            | crate::services::ServiceType::FireHouse
            | crate::services::ServiceType::FireHQ => NoiseSourceType::FireStation,
            crate::services::ServiceType::TrainStation => NoiseSourceType::TrainStation,
            crate::services::ServiceType::ElementarySchool
            | crate::services::ServiceType::HighSchool
            | crate::services::ServiceType::Kindergarten => NoiseSourceType::School,
            crate::services::ServiceType::SmallPark
            | crate::services::ServiceType::LargePark
            | crate::services::ServiceType::Playground
            | crate::services::ServiceType::Plaza
            | crate::services::ServiceType::SportsField => NoiseSourceType::Park,
            // Stadium and airports handled by base noise system
            _ => continue,
        };

        let db = effective_db(source_type, hour);
        if db > 0.0 {
            propagate(&mut noise, service.grid_x, service.grid_y, db);
        }
    }

    for utility in &utilities {
        let source_type = match utility.utility_type {
            crate::utilities::UtilityType::PowerPlant
            | crate::utilities::UtilityType::NuclearPlant
            | crate::utilities::UtilityType::Geothermal
            | crate::utilities::UtilityType::HydroDam => NoiseSourceType::PowerPlant,
            _ => continue,
        };

        let db = effective_db(source_type, hour);
        if db > 0.0 {
            propagate(&mut noise, utility.grid_x, utility.grid_y, db);
        }
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct NoiseSourcesPlugin;

impl Plugin for NoiseSourcesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NoiseSourceTableRes>().add_systems(
            FixedUpdate,
            apply_noise_source_table
                .after(crate::noise::update_noise_pollution)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_has_17_entries() {
        assert_eq!(NOISE_SOURCE_TABLE.len(), 17);
    }

    #[test]
    fn test_source_db_levels() {
        let cases: &[(NoiseSourceType, f32)] = &[
            (NoiseSourceType::Highway, 75.0),
            (NoiseSourceType::Arterial, 70.0),
            (NoiseSourceType::LocalRoad, 55.0),
            (NoiseSourceType::RailCorridor, 80.0),
            (NoiseSourceType::Airport, 105.0),
            (NoiseSourceType::Construction, 90.0),
            (NoiseSourceType::HeavyIndustry, 85.0),
            (NoiseSourceType::LightIndustry, 70.0),
            (NoiseSourceType::CommercialHvac, 60.0),
            (NoiseSourceType::Nightclub, 95.0),
            (NoiseSourceType::FireStation, 80.0),
            (NoiseSourceType::PowerPlant, 75.0),
            (NoiseSourceType::Stadium, 95.0),
            (NoiseSourceType::School, 70.0),
            (NoiseSourceType::Park, 35.0),
            (NoiseSourceType::ParkingStructure, 65.0),
            (NoiseSourceType::TrainStation, 75.0),
        ];
        for (st, expected) in cases {
            let e = lookup_source(*st).unwrap();
            assert!((e.db_level - expected).abs() < f32::EPSILON, "{:?}", st);
        }
    }

    #[test]
    fn test_source_activity_patterns() {
        let cases: &[(NoiseSourceType, ActivityPattern)] = &[
            (NoiseSourceType::Highway, ActivityPattern::Always),
            (NoiseSourceType::Construction, ActivityPattern::Daytime),
            (NoiseSourceType::Nightclub, ActivityPattern::Nighttime),
            (NoiseSourceType::Stadium, ActivityPattern::EventDriven),
            (NoiseSourceType::School, ActivityPattern::Daytime),
            (NoiseSourceType::Park, ActivityPattern::Daytime),
            (NoiseSourceType::FireStation, ActivityPattern::Always),
            (NoiseSourceType::PowerPlant, ActivityPattern::Always),
            (NoiseSourceType::TrainStation, ActivityPattern::Always),
        ];
        for (st, expected) in cases {
            let e = lookup_source(*st).unwrap();
            assert_eq!(e.activity, *expected, "{:?}", st);
        }
    }

    #[test]
    fn test_daytime_boundaries() {
        assert!(!ActivityPattern::Daytime.is_active(5.9));
        assert!(ActivityPattern::Daytime.is_active(6.0));
        assert!(ActivityPattern::Daytime.is_active(21.9));
        assert!(!ActivityPattern::Daytime.is_active(22.0));
    }

    #[test]
    fn test_nighttime_boundaries() {
        assert!(ActivityPattern::Nighttime.is_active(0.0));
        assert!(ActivityPattern::Nighttime.is_active(5.9));
        assert!(!ActivityPattern::Nighttime.is_active(6.0));
        assert!(!ActivityPattern::Nighttime.is_active(21.9));
        assert!(ActivityPattern::Nighttime.is_active(22.0));
    }

    #[test]
    fn test_always_every_hour() {
        for h in 0..24 {
            assert!(ActivityPattern::Always.is_active(h as f32));
        }
    }

    #[test]
    fn test_event_driven_boundaries() {
        assert!(!ActivityPattern::EventDriven.is_active(17.9));
        assert!(ActivityPattern::EventDriven.is_active(18.0));
        assert!(ActivityPattern::EventDriven.is_active(22.9));
        assert!(!ActivityPattern::EventDriven.is_active(23.0));
    }

    #[test]
    fn test_effective_db_active_vs_inactive() {
        assert!((effective_db(NoiseSourceType::Highway, 12.0) - 75.0).abs() < f32::EPSILON);
        assert!((effective_db(NoiseSourceType::Nightclub, 12.0)).abs() < f32::EPSILON);
        assert!((effective_db(NoiseSourceType::Nightclub, 0.0) - 95.0).abs() < f32::EPSILON);
        assert!((effective_db(NoiseSourceType::Construction, 23.0)).abs() < f32::EPSILON);
        assert!((effective_db(NoiseSourceType::Construction, 10.0) - 90.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_active_sources_count_at_noon() {
        let table = NoiseSourceTableRes::default();
        // Always(9) + Daytime(6) = 15
        assert_eq!(table.active_sources(12.0).len(), 15);
    }

    #[test]
    fn test_active_sources_count_at_midnight() {
        let table = NoiseSourceTableRes::default();
        // Always(9) + Nighttime(1) = 10
        assert_eq!(table.active_sources(0.0).len(), 10);
    }

    #[test]
    fn test_all_source_types_unique() {
        let mut seen = std::collections::HashSet::new();
        for entry in &NOISE_SOURCE_TABLE {
            assert!(seen.insert(entry.source_type), "dup: {:?}", entry.source_type);
        }
    }

    #[test]
    fn test_all_db_levels_positive() {
        for entry in &NOISE_SOURCE_TABLE {
            assert!(entry.db_level > 0.0, "{:?}", entry.source_type);
        }
    }
}
// POLL-019
