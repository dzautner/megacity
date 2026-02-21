use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum WealthTier {
    #[default]
    LowIncome,
    MiddleIncome,
    HighIncome,
}

impl WealthTier {
    pub fn from_education(education: u8) -> WealthTier {
        match education {
            0 => WealthTier::LowIncome,
            1 => WealthTier::LowIncome,
            2 => WealthTier::MiddleIncome,
            3 => WealthTier::HighIncome,
            _ => WealthTier::HighIncome,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            WealthTier::LowIncome => "Low Income",
            WealthTier::MiddleIncome => "Middle Income",
            WealthTier::HighIncome => "High Income",
        }
    }

    /// Tax revenue multiplier per citizen
    pub fn tax_multiplier(self) -> f32 {
        match self {
            WealthTier::LowIncome => 0.5,
            WealthTier::MiddleIncome => 1.0,
            WealthTier::HighIncome => 2.5,
        }
    }

    /// Preferred zone density
    pub fn preferred_density(self) -> crate::grid::ZoneType {
        match self {
            WealthTier::LowIncome => crate::grid::ZoneType::ResidentialHigh, // apartments
            WealthTier::MiddleIncome => crate::grid::ZoneType::ResidentialMedium, // townhouses
            WealthTier::HighIncome => crate::grid::ZoneType::ResidentialLow, // luxury houses
        }
    }

    /// Happiness weight adjustments per tier
    pub fn happiness_weights(self) -> WealthHappinessWeights {
        match self {
            WealthTier::LowIncome => WealthHappinessWeights {
                employment: 1.5,
                services: 1.0,
                parks: 0.5,
                pollution: 0.8,
                land_value: 0.3,
                entertainment: 0.5,
            },
            WealthTier::MiddleIncome => WealthHappinessWeights {
                employment: 1.0,
                services: 1.0,
                parks: 1.0,
                pollution: 1.0,
                land_value: 1.0,
                entertainment: 1.0,
            },
            WealthTier::HighIncome => WealthHappinessWeights {
                employment: 0.7,
                services: 1.2,
                parks: 1.5,
                pollution: 1.5,
                land_value: 2.0,
                entertainment: 1.5,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct WealthHappinessWeights {
    pub employment: f32,
    pub services: f32,
    pub parks: f32,
    pub pollution: f32,
    pub land_value: f32,
    pub entertainment: f32,
}

/// Aggregate wealth stats for the city
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct WealthStats {
    pub low_income_count: u32,
    pub middle_income_count: u32,
    pub high_income_count: u32,
}

impl WealthStats {
    pub fn total(&self) -> u32 {
        self.low_income_count + self.middle_income_count + self.high_income_count
    }

    pub fn percentage(&self, tier: WealthTier) -> f32 {
        let total = self.total() as f32;
        if total == 0.0 {
            return 0.0;
        }
        match tier {
            WealthTier::LowIncome => self.low_income_count as f32 / total,
            WealthTier::MiddleIncome => self.middle_income_count as f32 / total,
            WealthTier::HighIncome => self.high_income_count as f32 / total,
        }
    }
}

/// System: compute wealth stats from citizen education levels
pub fn update_wealth_stats(
    slow_tick: Res<crate::SlowTickTimer>,
    mut wealth_stats: ResMut<WealthStats>,
    citizens: Query<&crate::citizen::CitizenDetails, With<crate::citizen::Citizen>>,
) {
    if !slow_tick.should_run() {
        return;
    }
    wealth_stats.low_income_count = 0;
    wealth_stats.middle_income_count = 0;
    wealth_stats.high_income_count = 0;

    for details in &citizens {
        match WealthTier::from_education(details.education) {
            WealthTier::LowIncome => wealth_stats.low_income_count += 1,
            WealthTier::MiddleIncome => wealth_stats.middle_income_count += 1,
            WealthTier::HighIncome => wealth_stats.high_income_count += 1,
        }
    }
}

pub struct WealthPlugin;

impl Plugin for WealthPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WealthStats>().add_systems(
            FixedUpdate,
            update_wealth_stats
                .after(crate::imports_exports::process_trade)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}
