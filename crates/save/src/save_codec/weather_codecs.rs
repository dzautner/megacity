// ---------------------------------------------------------------------------
// Weather & climate codecs: WeatherEvent, Season, ClimateZone, FogDensity,
// WindDamageTier, DroughtTier, HeatWaveSeverity, ColdSnapTier
// ---------------------------------------------------------------------------

use simulation::weather::{ClimateZone, Season, WeatherCondition, WeatherEvent};

pub fn weather_event_to_u8(w: WeatherEvent) -> u8 {
    match w {
        WeatherCondition::Sunny => 0,
        WeatherCondition::Rain => 1,
        WeatherCondition::PartlyCloudy => 2,
        WeatherCondition::Overcast => 3,
        WeatherCondition::Storm => 4,
        WeatherCondition::HeavyRain => 5,
        WeatherCondition::Snow => 6,
    }
}

pub fn u8_to_weather_event(v: u8) -> WeatherEvent {
    match v {
        0 => WeatherCondition::Sunny,
        1 => WeatherCondition::Rain,
        2 => WeatherCondition::PartlyCloudy, // was HeatWave, now PartlyCloudy
        3 => WeatherCondition::Overcast,     // was ColdSnap, now Overcast
        4 => WeatherCondition::Storm,
        5 => WeatherCondition::HeavyRain,
        6 => WeatherCondition::Snow,
        _ => WeatherCondition::Sunny,
    }
}

pub fn season_to_u8(s: Season) -> u8 {
    match s {
        Season::Spring => 0,
        Season::Summer => 1,
        Season::Autumn => 2,
        Season::Winter => 3,
    }
}

pub fn u8_to_season(v: u8) -> Season {
    match v {
        0 => Season::Spring,
        1 => Season::Summer,
        2 => Season::Autumn,
        3 => Season::Winter,
        _ => Season::Spring,
    }
}

pub fn climate_zone_to_u8(z: ClimateZone) -> u8 {
    match z {
        ClimateZone::Temperate => 0,
        ClimateZone::Tropical => 1,
        ClimateZone::Arid => 2,
        ClimateZone::Mediterranean => 3,
        ClimateZone::Continental => 4,
        ClimateZone::Subarctic => 5,
        ClimateZone::Oceanic => 6,
    }
}

pub fn u8_to_climate_zone(v: u8) -> ClimateZone {
    match v {
        0 => ClimateZone::Temperate,
        1 => ClimateZone::Tropical,
        2 => ClimateZone::Arid,
        3 => ClimateZone::Mediterranean,
        4 => ClimateZone::Continental,
        5 => ClimateZone::Subarctic,
        6 => ClimateZone::Oceanic,
        _ => ClimateZone::Temperate, // fallback
    }
}

pub fn fog_density_to_u8(d: simulation::fog::FogDensity) -> u8 {
    use simulation::fog::FogDensity;
    match d {
        FogDensity::None => 0,
        FogDensity::Mist => 1,
        FogDensity::Moderate => 2,
        FogDensity::Dense => 3,
    }
}

pub fn u8_to_fog_density(v: u8) -> simulation::fog::FogDensity {
    use simulation::fog::FogDensity;
    match v {
        0 => FogDensity::None,
        1 => FogDensity::Mist,
        2 => FogDensity::Moderate,
        3 => FogDensity::Dense,
        _ => FogDensity::None, // fallback
    }
}

pub fn wind_damage_tier_to_u8(t: simulation::wind_damage::WindDamageTier) -> u8 {
    use simulation::wind_damage::WindDamageTier;
    match t {
        WindDamageTier::Calm => 0,
        WindDamageTier::Breezy => 1,
        WindDamageTier::Strong => 2,
        WindDamageTier::Gale => 3,
        WindDamageTier::Storm => 4,
        WindDamageTier::Severe => 5,
        WindDamageTier::HurricaneForce => 6,
        WindDamageTier::Extreme => 7,
    }
}

pub fn u8_to_wind_damage_tier(v: u8) -> simulation::wind_damage::WindDamageTier {
    use simulation::wind_damage::WindDamageTier;
    match v {
        0 => WindDamageTier::Calm,
        1 => WindDamageTier::Breezy,
        2 => WindDamageTier::Strong,
        3 => WindDamageTier::Gale,
        4 => WindDamageTier::Storm,
        5 => WindDamageTier::Severe,
        6 => WindDamageTier::HurricaneForce,
        7 => WindDamageTier::Extreme,
        _ => WindDamageTier::Calm, // fallback
    }
}

pub fn drought_tier_to_u8(t: simulation::drought::DroughtTier) -> u8 {
    use simulation::drought::DroughtTier;
    match t {
        DroughtTier::Normal => 0,
        DroughtTier::Moderate => 1,
        DroughtTier::Severe => 2,
        DroughtTier::Extreme => 3,
    }
}

pub fn u8_to_drought_tier(v: u8) -> simulation::drought::DroughtTier {
    use simulation::drought::DroughtTier;
    match v {
        0 => DroughtTier::Normal,
        1 => DroughtTier::Moderate,
        2 => DroughtTier::Severe,
        3 => DroughtTier::Extreme,
        _ => DroughtTier::Normal, // fallback
    }
}

pub fn heat_wave_severity_to_u8(s: simulation::heat_wave::HeatWaveSeverity) -> u8 {
    use simulation::heat_wave::HeatWaveSeverity;
    match s {
        HeatWaveSeverity::None => 0,
        HeatWaveSeverity::Moderate => 1,
        HeatWaveSeverity::Severe => 2,
        HeatWaveSeverity::Extreme => 3,
    }
}

pub fn u8_to_heat_wave_severity(v: u8) -> simulation::heat_wave::HeatWaveSeverity {
    use simulation::heat_wave::HeatWaveSeverity;
    match v {
        0 => HeatWaveSeverity::None,
        1 => HeatWaveSeverity::Moderate,
        2 => HeatWaveSeverity::Severe,
        3 => HeatWaveSeverity::Extreme,
        _ => HeatWaveSeverity::None, // fallback
    }
}

pub fn cold_snap_tier_to_u8(t: simulation::cold_snap::ColdSnapTier) -> u8 {
    use simulation::cold_snap::ColdSnapTier;
    match t {
        ColdSnapTier::Normal => 0,
        ColdSnapTier::Watch => 1,
        ColdSnapTier::Warning => 2,
        ColdSnapTier::Emergency => 3,
    }
}

pub fn u8_to_cold_snap_tier(v: u8) -> simulation::cold_snap::ColdSnapTier {
    use simulation::cold_snap::ColdSnapTier;
    match v {
        0 => ColdSnapTier::Normal,
        1 => ColdSnapTier::Watch,
        2 => ColdSnapTier::Warning,
        3 => ColdSnapTier::Emergency,
        _ => ColdSnapTier::Normal, // fallback
    }
}
