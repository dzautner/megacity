//! Extended policy effect multipliers for the tradeoff system (POL-001).
//!
//! These methods compute the new policy effects introduced as part of the
//! tradeoff expansion. They extend `Policies` with additional multipliers
//! used by `PolicyTradeoffEffects`.

use crate::policies::{Policies, Policy};

impl Policies {
    /// Industrial output multiplier (new tradeoff policies).
    pub fn industrial_output_multiplier(&self) -> f32 {
        let mut mult = 1.0_f32;
        if self.is_active(Policy::IndustrialSpacePlanning) {
            mult *= 1.5;
        }
        if self.is_active(Policy::HeavyTrafficBan) {
            mult *= 0.85;
        }
        mult
    }

    /// Noise multiplier (1.0 = normal, lower = less noise).
    pub fn noise_multiplier(&self) -> f32 {
        let mut mult = 1.0_f32;
        if self.is_active(Policy::HeavyTrafficBan) {
            mult *= 0.6;
        }
        if self.is_active(Policy::CombustionEngineBan) {
            mult *= 0.7;
        }
        mult
    }

    /// Fire hazard multiplier (1.0 = normal, lower = safer).
    pub fn fire_hazard_multiplier(&self) -> f32 {
        let mut mult = 1.0_f32;
        if self.is_active(Policy::SmokeDetectorMandate) {
            mult *= 0.7;
        }
        if self.is_active(Policy::SmokeDetectorDistribution) {
            mult *= 0.5;
        }
        mult
    }

    /// Construction rate multiplier.
    pub fn construction_rate_multiplier(&self) -> f32 {
        let mut mult = 1.0_f32;
        if self.is_active(Policy::TaxIncentiveZone) {
            mult *= 1.25;
        }
        if self.is_active(Policy::RentControl) {
            mult *= 0.75;
        }
        if self.is_active(Policy::OldTownHistoric) {
            mult = 0.0;
        }
        mult
    }

    /// Whether building modifications are blocked (OldTownHistoric).
    pub fn building_changes_blocked(&self) -> bool {
        self.is_active(Policy::OldTownHistoric)
    }

    /// Whether private cars are banned (CombustionEngineBan).
    pub fn private_cars_banned(&self) -> bool {
        self.is_active(Policy::CombustionEngineBan)
    }

    /// Whether heavy trucks are banned (HeavyTrafficBan).
    pub fn heavy_trucks_banned(&self) -> bool {
        self.is_active(Policy::HeavyTrafficBan)
    }

    /// Transit ridership bonus (additive fraction).
    pub fn transit_ridership_bonus(&self) -> f32 {
        let mut bonus = 0.0;
        if self.is_active(Policy::FreePublicTransport) {
            bonus += 0.30;
        }
        if self.is_active(Policy::CombustionEngineBan) {
            bonus += 0.40;
        }
        bonus
    }

    /// Cycling rate bonus (additive fraction).
    pub fn cycling_rate_bonus(&self) -> f32 {
        let mut bonus = 0.0;
        if self.is_active(Policy::EncourageBiking) {
            bonus += 0.15;
        }
        if self.is_active(Policy::CombustionEngineBan) {
            bonus += 0.20;
        }
        bonus
    }

    /// Car trip reduction multiplier (1.0 = normal, lower = fewer car trips).
    pub fn car_trip_multiplier(&self) -> f32 {
        let mut mult = 1.0_f32;
        if self.is_active(Policy::EncourageBiking) {
            mult *= 0.90;
        }
        if self.is_active(Policy::CombustionEngineBan) {
            mult = 0.0;
        }
        mult
    }

    /// Poverty reduction factor (additive, 0.0 = no reduction).
    pub fn poverty_reduction(&self) -> f32 {
        let mut reduction = 0.0;
        if self.is_active(Policy::MinimumWage) {
            reduction += 0.20;
        }
        if self.is_active(Policy::HealthcareForAll) {
            reduction += 0.10;
        }
        reduction
    }

    /// Business cost multiplier (1.0 = normal, higher = more expensive).
    pub fn business_cost_multiplier(&self) -> f32 {
        let mut mult = 1.0_f32;
        if self.is_active(Policy::MinimumWage) {
            mult *= 1.10;
        }
        mult
    }
}
