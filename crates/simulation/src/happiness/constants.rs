/// Bitflags for service coverage packed into a single byte per cell.
pub const COVERAGE_HEALTH: u8 = 0b0000_0001;
pub const COVERAGE_EDUCATION: u8 = 0b0000_0010;
pub const COVERAGE_POLICE: u8 = 0b0000_0100;
pub const COVERAGE_PARK: u8 = 0b0000_1000;
pub const COVERAGE_ENTERTAINMENT: u8 = 0b0001_0000;
pub const COVERAGE_TELECOM: u8 = 0b0010_0000;
pub const COVERAGE_TRANSPORT: u8 = 0b0100_0000;
pub const COVERAGE_FIRE: u8 = 0b1000_0000;

pub const BASE_HAPPINESS: f32 = 50.0;
pub const EMPLOYED_BONUS: f32 = 15.0;
pub const SHORT_COMMUTE_BONUS: f32 = 10.0;
pub const POWER_BONUS: f32 = 5.0;
pub const NO_POWER_PENALTY: f32 = 25.0;
pub const WATER_BONUS: f32 = 5.0;
pub const NO_WATER_PENALTY: f32 = 20.0;
pub const HEALTH_COVERAGE_BONUS: f32 = 5.0;
pub const EDUCATION_BONUS: f32 = 3.0;
pub const POLICE_BONUS: f32 = 5.0;
pub const PARK_BONUS: f32 = 8.0;
pub const ENTERTAINMENT_BONUS: f32 = 5.0;
pub const HIGH_TAX_PENALTY: f32 = 8.0;
pub const CONGESTION_PENALTY: f32 = 5.0;
pub const GARBAGE_PENALTY: f32 = 5.0;
pub const CRIME_PENALTY_MAX: f32 = 15.0;
pub const TELECOM_BONUS: f32 = 3.0;
pub const TRANSPORT_BONUS: f32 = 4.0;
pub const POOR_ROAD_PENALTY: f32 = 3.0;

/// Happiness penalty for homeless citizens (unsheltered).
pub const HOMELESS_PENALTY: f32 = 30.0;
/// Reduced happiness penalty for sheltered homeless citizens.
pub const SHELTERED_PENALTY: f32 = 10.0;
