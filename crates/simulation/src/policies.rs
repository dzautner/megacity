use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// City-wide and district-level policies that modify simulation parameters
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct Policies {
    pub active: Vec<Policy>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Policy {
    // Economy
    FreePublicTransport,
    HeavyIndustryTaxBreak,
    TourismPromotion,
    SmallBusinessGrant,

    // Environment
    RecyclingProgram,
    IndustrialAirFilters,
    WaterConservation,
    GreenSpaceInitiative,

    // Social
    EducationPush,
    HealthcareForAll,
    SmokeDetectorMandate,
    NeighborhoodWatch,

    // Zoning
    HighRiseBan,
    NightShiftBan,
    IndustrialZoningRestriction,
}

impl Policy {
    /// Monthly upkeep cost for having this policy active
    pub fn monthly_cost(self) -> f64 {
        match self {
            Policy::FreePublicTransport => 50.0,
            Policy::HeavyIndustryTaxBreak => 0.0, // revenue reduction, not direct cost
            Policy::TourismPromotion => 30.0,
            Policy::SmallBusinessGrant => 25.0,
            Policy::RecyclingProgram => 20.0,
            Policy::IndustrialAirFilters => 35.0,
            Policy::WaterConservation => 10.0,
            Policy::GreenSpaceInitiative => 15.0,
            Policy::EducationPush => 40.0,
            Policy::HealthcareForAll => 45.0,
            Policy::SmokeDetectorMandate => 10.0,
            Policy::NeighborhoodWatch => 15.0,
            Policy::HighRiseBan => 0.0,
            Policy::NightShiftBan => 0.0,
            Policy::IndustrialZoningRestriction => 0.0,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Policy::FreePublicTransport => "Free Public Transport",
            Policy::HeavyIndustryTaxBreak => "Heavy Industry Tax Break",
            Policy::TourismPromotion => "Tourism Promotion",
            Policy::SmallBusinessGrant => "Small Business Grant",
            Policy::RecyclingProgram => "Recycling Program",
            Policy::IndustrialAirFilters => "Industrial Air Filters",
            Policy::WaterConservation => "Water Conservation",
            Policy::GreenSpaceInitiative => "Green Space Initiative",
            Policy::EducationPush => "Education Push",
            Policy::HealthcareForAll => "Healthcare For All",
            Policy::SmokeDetectorMandate => "Smoke Detector Mandate",
            Policy::NeighborhoodWatch => "Neighborhood Watch",
            Policy::HighRiseBan => "High-Rise Ban",
            Policy::NightShiftBan => "Night Shift Ban",
            Policy::IndustrialZoningRestriction => "Industrial Zoning Restriction",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Policy::FreePublicTransport => "Transit is free, reducing traffic but costing money",
            Policy::HeavyIndustryTaxBreak => {
                "Attracts industry, reduces industrial tax income by 50%"
            }
            Policy::TourismPromotion => "Increases commercial demand from tourism",
            Policy::SmallBusinessGrant => "Boosts commercial growth, costs money",
            Policy::RecyclingProgram => "Reduces garbage by 30%, increases service cost",
            Policy::IndustrialAirFilters => "Reduces industrial pollution by 40%",
            Policy::WaterConservation => "Reduces water consumption, saves money long-term",
            Policy::GreenSpaceInitiative => "Boosts park effectiveness by 50%",
            Policy::EducationPush => "Faster education progression, increases education spending",
            Policy::HealthcareForAll => "Increases health coverage, expensive",
            Policy::SmokeDetectorMandate => "Reduces fire risk (future), small upkeep",
            Policy::NeighborhoodWatch => "Reduces crime (future), small upkeep",
            Policy::HighRiseBan => "Caps building level at 2 city-wide",
            Policy::NightShiftBan => "Increases happiness +3, reduces commercial output",
            Policy::IndustrialZoningRestriction => "Limits new industrial zoning near residential",
        }
    }

    /// All available policies
    pub fn all() -> &'static [Policy] {
        &[
            Policy::FreePublicTransport,
            Policy::HeavyIndustryTaxBreak,
            Policy::TourismPromotion,
            Policy::SmallBusinessGrant,
            Policy::RecyclingProgram,
            Policy::IndustrialAirFilters,
            Policy::WaterConservation,
            Policy::GreenSpaceInitiative,
            Policy::EducationPush,
            Policy::HealthcareForAll,
            Policy::SmokeDetectorMandate,
            Policy::NeighborhoodWatch,
            Policy::HighRiseBan,
            Policy::NightShiftBan,
            Policy::IndustrialZoningRestriction,
        ]
    }
}

impl Policies {
    pub fn is_active(&self, policy: Policy) -> bool {
        self.active.contains(&policy)
    }

    pub fn toggle(&mut self, policy: Policy) {
        if let Some(idx) = self.active.iter().position(|&p| p == policy) {
            self.active.remove(idx);
        } else {
            self.active.push(policy);
        }
    }

    pub fn total_monthly_cost(&self) -> f64 {
        self.active.iter().map(|p| p.monthly_cost()).sum()
    }

    /// Pollution multiplier (1.0 = normal, lower = less pollution)
    pub fn pollution_multiplier(&self) -> f32 {
        if self.is_active(Policy::IndustrialAirFilters) {
            0.6
        } else {
            1.0
        }
    }

    /// Garbage multiplier
    pub fn garbage_multiplier(&self) -> f32 {
        if self.is_active(Policy::RecyclingProgram) {
            0.7
        } else {
            1.0
        }
    }

    /// Park effectiveness multiplier
    pub fn park_multiplier(&self) -> f32 {
        if self.is_active(Policy::GreenSpaceInitiative) {
            1.5
        } else {
            1.0
        }
    }

    /// Max building level allowed (3 normally, 2 with HighRiseBan)
    pub fn max_building_level(&self) -> u8 {
        if self.is_active(Policy::HighRiseBan) {
            2
        } else {
            3
        }
    }

    /// Industrial tax multiplier (1.0 normal, 0.5 with tax break)
    pub fn industrial_tax_multiplier(&self) -> f32 {
        if self.is_active(Policy::HeavyIndustryTaxBreak) {
            0.5
        } else {
            1.0
        }
    }

    /// Commercial demand bonus from tourism
    pub fn commercial_demand_bonus(&self) -> f32 {
        let mut bonus = 0.0;
        if self.is_active(Policy::TourismPromotion) {
            bonus += 0.15;
        }
        if self.is_active(Policy::SmallBusinessGrant) {
            bonus += 0.10;
        }
        bonus
    }

    /// Happiness bonus from social policies
    pub fn happiness_bonus(&self) -> f32 {
        let mut bonus = 0.0;
        if self.is_active(Policy::FreePublicTransport) {
            bonus += 3.0;
        }
        if self.is_active(Policy::NightShiftBan) {
            bonus += 3.0;
        }
        if self.is_active(Policy::HealthcareForAll) {
            bonus += 2.0;
        }
        if self.is_active(Policy::NeighborhoodWatch) {
            bonus += 2.0;
        }
        bonus
    }

    /// Education speed multiplier
    pub fn education_multiplier(&self) -> f32 {
        if self.is_active(Policy::EducationPush) {
            1.5
        } else {
            1.0
        }
    }

    /// Industrial demand bonus from tax breaks
    pub fn industrial_demand_bonus(&self) -> f32 {
        if self.is_active(Policy::HeavyIndustryTaxBreak) {
            0.15
        } else {
            0.0
        }
    }
}
