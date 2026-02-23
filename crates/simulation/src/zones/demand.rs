use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::grid::ZoneType;

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct ZoneDemand {
    pub residential: f32,
    pub commercial: f32,
    pub industrial: f32,
    pub office: f32,
    /// Tracked vacancy rates (built capacity vs occupied) per zone category.
    #[serde(default)]
    pub vacancy_residential: f32,
    #[serde(default)]
    pub vacancy_commercial: f32,
    #[serde(default)]
    pub vacancy_industrial: f32,
    #[serde(default)]
    pub vacancy_office: f32,
}

impl Default for ZoneDemand {
    fn default() -> Self {
        Self {
            residential: 0.0,
            commercial: 0.0,
            industrial: 0.0,
            office: 0.0,
            vacancy_residential: 0.0,
            vacancy_commercial: 0.0,
            vacancy_industrial: 0.0,
            vacancy_office: 0.0,
        }
    }
}

impl ZoneDemand {
    pub fn demand_for(&self, zone: ZoneType) -> f32 {
        match zone {
            ZoneType::ResidentialLow | ZoneType::ResidentialMedium | ZoneType::ResidentialHigh => {
                self.residential
            }
            ZoneType::CommercialLow | ZoneType::CommercialHigh => self.commercial,
            ZoneType::Industrial => self.industrial,
            ZoneType::Office => self.office,
            // MixedUse responds to the higher of residential and commercial demand
            ZoneType::MixedUse => self.residential.max(self.commercial),
            ZoneType::None => 0.0,
        }
    }
}
