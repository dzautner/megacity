use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, WorldGrid};
use crate::weather::{Weather, WeatherCondition};

/// Counts how many cardinal-direction road neighbors a cell has.
/// A cell with road neighbors in 3+ directions is considered an intersection.
pub(crate) fn road_neighbor_directions(grid: &WorldGrid, x: usize, y: usize) -> u8 {
    let mut dirs = 0u8;
    if x > 0 && grid.get(x - 1, y).cell_type == CellType::Road {
        dirs += 1;
    }
    if x + 1 < GRID_WIDTH && grid.get(x + 1, y).cell_type == CellType::Road {
        dirs += 1;
    }
    if y > 0 && grid.get(x, y - 1).cell_type == CellType::Road {
        dirs += 1;
    }
    if y + 1 < GRID_HEIGHT && grid.get(x, y + 1).cell_type == CellType::Road {
        dirs += 1;
    }
    dirs
}

/// Returns a weather-based accident probability multiplier.
pub(crate) fn weather_accident_multiplier(weather: &Weather) -> f32 {
    match weather.current_event {
        WeatherCondition::Storm => 2.5,
        WeatherCondition::Snow => 2.0,
        WeatherCondition::HeavyRain => 1.8,
        WeatherCondition::Rain => 1.5,
        WeatherCondition::Overcast => 1.1,
        WeatherCondition::PartlyCloudy | WeatherCondition::Sunny => 1.0,
    }
}
