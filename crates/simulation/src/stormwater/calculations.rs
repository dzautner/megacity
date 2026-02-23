use crate::config::CELL_SIZE;
use crate::grid::{CellType, ZoneType};
use crate::weather::{Weather, WeatherCondition};

/// Cell area in square meters (CELL_SIZE x CELL_SIZE).
pub(crate) const CELL_AREA: f32 = CELL_SIZE * CELL_SIZE;

/// Default soil permeability coefficient (m/s equivalent, unitless for simulation).
/// Represents how easily water passes through soil when not covered by impervious surfaces.
pub(crate) const SOIL_PERMEABILITY: f32 = 0.6;

/// Drain rate per tick: fraction of accumulated runoff that drains to downstream cells.
pub(crate) const DRAIN_RATE: f32 = 0.1;

/// Returns the imperviousness coefficient for a cell based on its surface type.
///
/// Values represent the fraction of rainfall that becomes surface runoff:
/// - Road/Building: 0.95 (asphalt/concrete, nearly impervious)
/// - Parking/Industrial: 0.90
/// - Concrete/Commercial: 0.85
/// - Compacted soil (empty with building nearby): 0.70
/// - Grass (default empty): 0.35
/// - Forest/Park: 0.15
/// - Green roof: 0.25
/// - Pervious pavement: 0.40
pub fn imperviousness(cell_type: CellType, zone: ZoneType, has_building: bool) -> f32 {
    match cell_type {
        CellType::Road => 0.95,
        CellType::Water => 0.0,
        CellType::Grass => {
            if has_building {
                // Building footprint: nearly impervious
                0.95
            } else {
                match zone {
                    ZoneType::Industrial => 0.90,
                    ZoneType::CommercialHigh | ZoneType::CommercialLow | ZoneType::Office => 0.85,
                    ZoneType::MixedUse => 0.85,
                    ZoneType::ResidentialHigh | ZoneType::ResidentialMedium => 0.70,
                    ZoneType::ResidentialLow => 0.40,
                    ZoneType::None => 0.35,
                }
            }
        }
    }
}

/// Calculate runoff volume for a single cell given rainfall intensity.
///
/// `runoff = rainfall_intensity * imperviousness * cell_area`
pub fn runoff(rainfall_intensity: f32, imperv: f32) -> f32 {
    rainfall_intensity * imperv * CELL_AREA
}

/// Calculate infiltration volume for a single cell given rainfall intensity.
///
/// `infiltration = rainfall_intensity * (1.0 - imperviousness) * soil_permeability`
pub fn infiltration(rainfall_intensity: f32, imperv: f32) -> f32 {
    rainfall_intensity * (1.0 - imperv) * SOIL_PERMEABILITY
}

/// Rainfall intensity derived from weather condition.
/// Returns a value in the range [0.0, 1.0] representing precipitation rate.
pub(crate) fn rainfall_intensity(weather: &Weather) -> f32 {
    match weather.current_event {
        WeatherCondition::Rain => 0.3,
        WeatherCondition::HeavyRain => 0.6,
        WeatherCondition::Storm => 1.0,
        // Snow melts slowly; minimal immediate runoff
        WeatherCondition::Snow => 0.05,
        _ => 0.0,
    }
}
