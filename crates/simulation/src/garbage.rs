use crate::buildings::Building;
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::ZoneType;
use crate::services::{ServiceBuilding, ServiceType};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Resource)]
pub struct GarbageGrid {
    pub levels: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

impl Default for GarbageGrid {
    fn default() -> Self {
        Self {
            levels: vec![0; GRID_WIDTH * GRID_HEIGHT],
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
        }
    }
}

impl GarbageGrid {
    pub fn get(&self, x: usize, y: usize) -> u8 {
        self.levels[y * self.width + x]
    }
    pub fn set(&mut self, x: usize, y: usize, val: u8) {
        self.levels[y * self.width + x] = val;
    }
}

// =============================================================================
// Per-building waste generation (WASTE-001)
// =============================================================================

/// Component attached to each building that tracks its waste generation rate.
///
/// Residential buildings generate waste per person per day (lbs/person/day),
/// while commercial, industrial, and service buildings generate waste per
/// building per day (lbs/building/day).
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct WasteProducer {
    /// Waste generation rate in lbs/day.
    /// For residential: lbs per person per day (multiplied by occupants).
    /// For non-residential: lbs per building per day.
    pub waste_lbs_per_day: f32,
    /// Whether this building participates in the recycling program.
    /// When true, waste output is reduced by 30%.
    pub recycling_participation: bool,
}

impl WasteProducer {
    /// Returns the per-person-per-day waste rate for a residential building
    /// based on its zone type and building level.
    ///
    /// Level 1 = low-income, Level 2 = middle-income, Level 3+ = high-income.
    pub fn residential_rate(zone: ZoneType, level: u8) -> f32 {
        match (zone, level) {
            (ZoneType::ResidentialLow, 1) => 3.0,    // low-income
            (ZoneType::ResidentialLow, 2) => 4.5,    // middle-income
            (ZoneType::ResidentialLow, _) => 6.0,    // high-income
            (ZoneType::ResidentialMedium, 1) => 3.0, // low-income
            (ZoneType::ResidentialMedium, 2) => 4.5, // middle-income
            (ZoneType::ResidentialMedium, _) => 6.0, // high-income
            (ZoneType::ResidentialHigh, 1) => 3.0,   // low-income
            (ZoneType::ResidentialHigh, 2) => 4.5,   // middle-income
            (ZoneType::ResidentialHigh, _) => 6.0,   // high-income
            _ => 0.0,
        }
    }

    /// Returns the per-building-per-day waste rate for a commercial building.
    ///
    /// Low-density commercial = small shops (50 lbs/day).
    /// High-density commercial at level 1-2 = large commercial (300 lbs/day).
    /// High-density commercial at level 3+ approximates restaurants (200 lbs/day).
    pub fn commercial_rate(zone: ZoneType, level: u8) -> f32 {
        match (zone, level) {
            (ZoneType::CommercialLow, _) => 50.0,       // small commercial
            (ZoneType::CommercialHigh, 1..=2) => 300.0, // large commercial
            (ZoneType::CommercialHigh, _) => 200.0,     // restaurant-type
            _ => 0.0,
        }
    }

    /// Returns the per-building-per-day waste rate for an industrial building.
    ///
    /// Level 1-2 = light industry (500 lbs/day).
    /// Level 3+ = heavy industry (2000 lbs/day).
    pub fn industrial_rate(level: u8) -> f32 {
        match level {
            1..=2 => 500.0, // light industry
            _ => 2000.0,    // heavy industry
        }
    }

    /// Returns the per-facility-per-day waste rate for a service building.
    pub fn service_rate(service_type: ServiceType) -> f32 {
        match service_type {
            ServiceType::Hospital | ServiceType::MedicalCenter => 1500.0,
            ServiceType::ElementarySchool
            | ServiceType::HighSchool
            | ServiceType::Kindergarten
            | ServiceType::Library => 100.0,
            ServiceType::University => 200.0,
            ServiceType::Stadium => 500.0,
            _ => 50.0, // default for other service buildings
        }
    }

    /// Create a WasteProducer for a zoned building based on its type and level.
    pub fn for_building(zone: ZoneType, level: u8) -> Self {
        let rate = if zone.is_residential() {
            Self::residential_rate(zone, level)
        } else if zone.is_commercial() {
            Self::commercial_rate(zone, level)
        } else if zone == ZoneType::Industrial {
            Self::industrial_rate(level)
        } else {
            // Office buildings: moderate waste (similar to small commercial)
            50.0
        };
        Self {
            waste_lbs_per_day: rate,
            recycling_participation: false,
        }
    }

    /// Create a WasteProducer for a service building.
    pub fn for_service(service_type: ServiceType) -> Self {
        Self {
            waste_lbs_per_day: Self::service_rate(service_type),
            recycling_participation: false,
        }
    }

    /// Returns the effective daily waste output in lbs, accounting for
    /// recycling participation and optionally the building's occupant count
    /// (for residential buildings, which scale by population).
    pub fn effective_daily_waste(&self, occupants: u32, is_residential: bool) -> f32 {
        let base = if is_residential {
            self.waste_lbs_per_day * occupants as f32
        } else {
            self.waste_lbs_per_day
        };
        if self.recycling_participation {
            base * 0.7 // 30% reduction from recycling
        } else {
            base
        }
    }
}

/// City-wide waste statistics resource, updated periodically by the waste system.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct WasteSystem {
    /// Total waste generated across the city in tons (accumulated since last reset).
    pub total_generated_tons: f64,
    /// Total waste generated this update period in tons.
    pub period_generated_tons: f64,
    /// Per-capita waste in lbs/person/day (averaged over the last update period).
    pub per_capita_lbs_per_day: f32,
    /// Total population used for per-capita calculation.
    pub tracked_population: u32,
    /// Number of buildings with recycling participation.
    pub recycling_buildings: u32,
    /// Total number of waste-producing buildings tracked.
    pub total_producers: u32,
}

/// Attaches `WasteProducer` components to buildings that don't have one yet.
/// Runs on the slow tick to avoid overhead every frame.
pub fn attach_waste_producers(
    slow_timer: Res<crate::SlowTickTimer>,
    mut commands: Commands,
    buildings_without: Query<(Entity, &Building), Without<WasteProducer>>,
    services_without: Query<(Entity, &ServiceBuilding), Without<WasteProducer>>,
    policies: Res<crate::policies::Policies>,
) {
    if !slow_timer.should_run() {
        return;
    }
    let recycling_active = policies.is_active(crate::policies::Policy::RecyclingProgram);

    for (entity, building) in &buildings_without {
        let mut producer = WasteProducer::for_building(building.zone_type, building.level);
        producer.recycling_participation = recycling_active;
        commands.entity(entity).insert(producer);
    }

    for (entity, service) in &services_without {
        let mut producer = WasteProducer::for_service(service.service_type);
        producer.recycling_participation = recycling_active;
        commands.entity(entity).insert(producer);
    }
}

/// Updates recycling participation on all WasteProducers when the recycling
/// policy changes. Runs on the slow tick.
pub fn sync_recycling_policy(
    slow_timer: Res<crate::SlowTickTimer>,
    policies: Res<crate::policies::Policies>,
    mut producers: Query<&mut WasteProducer>,
) {
    if !slow_timer.should_run() {
        return;
    }
    let recycling_active = policies.is_active(crate::policies::Policy::RecyclingProgram);
    for mut producer in &mut producers {
        producer.recycling_participation = recycling_active;
    }
}

/// Aggregates waste generation across all buildings and updates the `WasteSystem`
/// resource with totals and per-capita metrics.
///
/// Runs on the slow tick (every ~10 seconds of game time).
/// The slow tick interval is 100 ticks at 10Hz = 10 game-seconds.
/// We treat each slow tick as representing roughly 1 game-day for waste calculations.
pub fn update_waste_generation(
    slow_timer: Res<crate::SlowTickTimer>,
    mut waste_system: ResMut<WasteSystem>,
    building_producers: Query<(&Building, &WasteProducer)>,
    service_producers: Query<(&ServiceBuilding, &WasteProducer)>,
    stats: Res<crate::stats::CityStats>,
) {
    if !slow_timer.should_run() {
        return;
    }

    let mut total_waste_lbs: f64 = 0.0;
    let mut recycling_count = 0u32;
    let mut producer_count = 0u32;

    // Zoned buildings (residential, commercial, industrial, office)
    for (building, producer) in &building_producers {
        let is_residential = building.zone_type.is_residential();
        let daily_waste = producer.effective_daily_waste(building.occupants, is_residential);
        total_waste_lbs += daily_waste as f64;
        producer_count += 1;
        if producer.recycling_participation {
            recycling_count += 1;
        }
    }

    // Service buildings (hospitals, schools, etc.)
    for (_service, producer) in &service_producers {
        let daily_waste = producer.effective_daily_waste(0, false);
        total_waste_lbs += daily_waste as f64;
        producer_count += 1;
        if producer.recycling_participation {
            recycling_count += 1;
        }
    }

    // Convert lbs to tons (1 ton = 2000 lbs)
    let period_tons = total_waste_lbs / 2000.0;

    let population = stats.population;
    let per_capita = if population > 0 {
        total_waste_lbs as f32 / population as f32
    } else {
        0.0
    };

    waste_system.period_generated_tons = period_tons;
    waste_system.total_generated_tons += period_tons;
    waste_system.per_capita_lbs_per_day = per_capita;
    waste_system.tracked_population = population;
    waste_system.recycling_buildings = recycling_count;
    waste_system.total_producers = producer_count;
}

pub fn update_garbage(
    slow_timer: Res<crate::SlowTickTimer>,
    mut garbage: ResMut<GarbageGrid>,
    buildings: Query<(&Building, Option<&WasteProducer>)>,
    services: Query<&ServiceBuilding>,
    policies: Res<crate::policies::Policies>,
) {
    if !slow_timer.should_run() {
        return;
    }
    // Buildings produce garbage proportional to waste generation rate or occupants
    let garbage_mult = policies.garbage_multiplier();
    for (building, maybe_producer) in &buildings {
        let production = if let Some(producer) = maybe_producer {
            // Use the detailed waste rate: convert lbs/day to grid units
            // Scale down so the grid u8 stays in a reasonable range
            let is_residential = building.zone_type.is_residential();
            let daily_lbs = producer.effective_daily_waste(building.occupants, is_residential);
            // Map ~0-2000 lbs/day range down to 0-10 grid units
            ((daily_lbs / 200.0).min(10.0) * garbage_mult) as u8
        } else {
            // Fallback: original formula for buildings without WasteProducer yet
            ((building.occupants / 5).min(10) as f32 * garbage_mult) as u8
        };
        let cur = garbage.get(building.grid_x, building.grid_y);
        garbage.set(
            building.grid_x,
            building.grid_y,
            cur.saturating_add(production),
        );
    }

    // Garbage service buildings collect in radius
    for service in &services {
        if !ServiceBuilding::is_garbage(service.service_type) {
            continue;
        }
        let radius = (service.radius / 16.0) as i32;
        let collection = match service.service_type {
            ServiceType::Landfill => 3u8,
            ServiceType::RecyclingCenter => 5u8,
            ServiceType::Incinerator => 8u8,
            ServiceType::TransferStation => 4u8,
            _ => 0,
        };
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let nx = service.grid_x as i32 + dx;
                let ny = service.grid_y as i32 + dy;
                if nx >= 0 && ny >= 0 && (nx as usize) < GRID_WIDTH && (ny as usize) < GRID_HEIGHT {
                    let cur = garbage.get(nx as usize, ny as usize);
                    garbage.set(nx as usize, ny as usize, cur.saturating_sub(collection));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_residential_waste_rates() {
        // Low-income (level 1)
        assert_eq!(
            WasteProducer::residential_rate(ZoneType::ResidentialLow, 1),
            3.0
        );
        assert_eq!(
            WasteProducer::residential_rate(ZoneType::ResidentialHigh, 1),
            3.0
        );
        // Middle-income (level 2)
        assert_eq!(
            WasteProducer::residential_rate(ZoneType::ResidentialLow, 2),
            4.5
        );
        assert_eq!(
            WasteProducer::residential_rate(ZoneType::ResidentialHigh, 2),
            4.5
        );
        // High-income (level 3+)
        assert_eq!(
            WasteProducer::residential_rate(ZoneType::ResidentialLow, 3),
            6.0
        );
        assert_eq!(
            WasteProducer::residential_rate(ZoneType::ResidentialHigh, 5),
            6.0
        );
    }

    #[test]
    fn test_commercial_waste_rates() {
        // Small commercial
        assert_eq!(
            WasteProducer::commercial_rate(ZoneType::CommercialLow, 1),
            50.0
        );
        assert_eq!(
            WasteProducer::commercial_rate(ZoneType::CommercialLow, 3),
            50.0
        );
        // Large commercial
        assert_eq!(
            WasteProducer::commercial_rate(ZoneType::CommercialHigh, 1),
            300.0
        );
        assert_eq!(
            WasteProducer::commercial_rate(ZoneType::CommercialHigh, 2),
            300.0
        );
        // Restaurant-type (high-density level 3+)
        assert_eq!(
            WasteProducer::commercial_rate(ZoneType::CommercialHigh, 3),
            200.0
        );
        assert_eq!(
            WasteProducer::commercial_rate(ZoneType::CommercialHigh, 5),
            200.0
        );
    }

    #[test]
    fn test_industrial_waste_rates() {
        // Light industry
        assert_eq!(WasteProducer::industrial_rate(1), 500.0);
        assert_eq!(WasteProducer::industrial_rate(2), 500.0);
        // Heavy industry
        assert_eq!(WasteProducer::industrial_rate(3), 2000.0);
        assert_eq!(WasteProducer::industrial_rate(5), 2000.0);
    }

    #[test]
    fn test_service_waste_rates() {
        assert_eq!(WasteProducer::service_rate(ServiceType::Hospital), 1500.0);
        assert_eq!(
            WasteProducer::service_rate(ServiceType::MedicalCenter),
            1500.0
        );
        assert_eq!(
            WasteProducer::service_rate(ServiceType::ElementarySchool),
            100.0
        );
        assert_eq!(WasteProducer::service_rate(ServiceType::HighSchool), 100.0);
        assert_eq!(WasteProducer::service_rate(ServiceType::University), 200.0);
    }

    #[test]
    fn test_effective_daily_waste_residential() {
        let producer = WasteProducer {
            waste_lbs_per_day: 4.5,
            recycling_participation: false,
        };
        // 10 occupants * 4.5 lbs/person/day = 45 lbs/day
        assert_eq!(producer.effective_daily_waste(10, true), 45.0);

        let producer_recycling = WasteProducer {
            waste_lbs_per_day: 4.5,
            recycling_participation: true,
        };
        // 10 occupants * 4.5 * 0.7 = 31.5
        assert!((producer_recycling.effective_daily_waste(10, true) - 31.5).abs() < 0.01);
    }

    #[test]
    fn test_effective_daily_waste_non_residential() {
        let producer = WasteProducer {
            waste_lbs_per_day: 300.0,
            recycling_participation: false,
        };
        // Non-residential ignores occupants
        assert_eq!(producer.effective_daily_waste(50, false), 300.0);
        assert_eq!(producer.effective_daily_waste(0, false), 300.0);

        let producer_recycling = WasteProducer {
            waste_lbs_per_day: 300.0,
            recycling_participation: true,
        };
        assert!((producer_recycling.effective_daily_waste(0, false) - 210.0).abs() < 0.01);
    }

    #[test]
    fn test_for_building_factory() {
        let res_low = WasteProducer::for_building(ZoneType::ResidentialLow, 1);
        assert_eq!(res_low.waste_lbs_per_day, 3.0);
        assert!(!res_low.recycling_participation);

        let comm_high = WasteProducer::for_building(ZoneType::CommercialHigh, 1);
        assert_eq!(comm_high.waste_lbs_per_day, 300.0);

        let industrial = WasteProducer::for_building(ZoneType::Industrial, 3);
        assert_eq!(industrial.waste_lbs_per_day, 2000.0);

        let office = WasteProducer::for_building(ZoneType::Office, 1);
        assert_eq!(office.waste_lbs_per_day, 50.0);
    }

    #[test]
    fn test_for_service_factory() {
        let hospital = WasteProducer::for_service(ServiceType::Hospital);
        assert_eq!(hospital.waste_lbs_per_day, 1500.0);

        let school = WasteProducer::for_service(ServiceType::ElementarySchool);
        assert_eq!(school.waste_lbs_per_day, 100.0);
    }

    #[test]
    fn test_waste_system_default() {
        let ws = WasteSystem::default();
        assert_eq!(ws.total_generated_tons, 0.0);
        assert_eq!(ws.period_generated_tons, 0.0);
        assert_eq!(ws.per_capita_lbs_per_day, 0.0);
        assert_eq!(ws.tracked_population, 0);
        assert_eq!(ws.recycling_buildings, 0);
        assert_eq!(ws.total_producers, 0);
    }

    #[test]
    fn test_non_residential_zone_returns_zero() {
        assert_eq!(
            WasteProducer::residential_rate(ZoneType::Industrial, 1),
            0.0
        );
        assert_eq!(WasteProducer::commercial_rate(ZoneType::Industrial, 1), 0.0);
    }
}
