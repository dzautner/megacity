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
    EminentDomain,
    CumulativeZoning,

    // Transport
    EncourageBiking,

    // --- New tradeoff policies (issue #613) ---
    CombustionEngineBan,
    SmallBusinessEnthusiast,
    HeavyTrafficBan,
    SmokeDetectorDistribution,
    OldTownHistoric,
    IndustrialSpacePlanning,
    RentControl,
    MinimumWage,
    TaxIncentiveZone,
    PetBan,
    ParksAndRec,
}

impl Policy {
    /// Monthly upkeep cost for having this policy active
    pub fn monthly_cost(self) -> f64 {
        match self {
            Policy::FreePublicTransport => 50.0,
            Policy::HeavyIndustryTaxBreak => 0.0,
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
            Policy::EminentDomain => 20.0,
            Policy::CumulativeZoning => 0.0,
            Policy::EncourageBiking => 15.0,
            Policy::CombustionEngineBan => 30.0,
            Policy::SmallBusinessEnthusiast => 20.0,
            Policy::HeavyTrafficBan => 10.0,
            Policy::SmokeDetectorDistribution => 15.0,
            Policy::OldTownHistoric => 5.0,
            Policy::IndustrialSpacePlanning => 25.0,
            Policy::RentControl => 10.0,
            Policy::MinimumWage => 20.0,
            Policy::TaxIncentiveZone => 0.0,
            Policy::PetBan => 5.0,
            Policy::ParksAndRec => 20.0,
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
            Policy::EminentDomain => "Eminent Domain",
            Policy::CumulativeZoning => "Cumulative Zoning",
            Policy::EncourageBiking => "Encourage Biking",
            Policy::CombustionEngineBan => "Combustion Engine Ban",
            Policy::SmallBusinessEnthusiast => "Small Business Enthusiast",
            Policy::HeavyTrafficBan => "Heavy Traffic Ban",
            Policy::SmokeDetectorDistribution => "Smoke Detector Distribution",
            Policy::OldTownHistoric => "Old Town / Historic",
            Policy::IndustrialSpacePlanning => "Industrial Space Planning",
            Policy::RentControl => "Rent Control",
            Policy::MinimumWage => "Minimum Wage",
            Policy::TaxIncentiveZone => "Tax Incentive Zone",
            Policy::PetBan => "Pet Ban",
            Policy::ParksAndRec => "Parks & Rec",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Policy::FreePublicTransport => {
                "Transit is free: +30% ridership, all transit revenue lost"
            }
            Policy::HeavyIndustryTaxBreak => {
                "Attracts industry, reduces industrial tax income by 50%"
            }
            Policy::TourismPromotion => "Increases commercial demand from tourism",
            Policy::SmallBusinessGrant => "Boosts commercial growth, costs money",
            Policy::RecyclingProgram => {
                "Reduces garbage by 30%, costs +10% garbage budget"
            }
            Policy::IndustrialAirFilters => "Reduces industrial pollution by 40%",
            Policy::WaterConservation => {
                "Reduces water consumption, saves money long-term"
            }
            Policy::GreenSpaceInitiative => "Boosts park effectiveness by 50%",
            Policy::EducationPush => {
                "Faster education progression, increases education spending"
            }
            Policy::HealthcareForAll => "Increases health coverage, expensive",
            Policy::SmokeDetectorMandate => "Reduces fire risk, small upkeep",
            Policy::NeighborhoodWatch => "Reduces crime, small upkeep",
            Policy::HighRiseBan => {
                "Caps building level at 2, preserves neighborhood character"
            }
            Policy::NightShiftBan => {
                "Increases happiness +3, reduces commercial output"
            }
            Policy::IndustrialZoningRestriction => {
                "Limits new industrial zoning near residential"
            }
            Policy::EminentDomain => {
                "Override citizen opposition at a happiness cost"
            }
            Policy::CumulativeZoning => {
                "Higher-intensity zones allow lower-intensity uses"
            }
            Policy::EncourageBiking => {
                "+15% cycling rate, -10% car trips when bike infra exists"
            }
            Policy::CombustionEngineBan => {
                "Bans private cars: forces transit/walking, cuts pollution 30%"
            }
            Policy::SmallBusinessEnthusiast => {
                "Caps commercial at level 2, +20% small biz growth"
            }
            Policy::HeavyTrafficBan => {
                "Bans trucks: -40% road noise, -15% industrial output"
            }
            Policy::SmokeDetectorDistribution => {
                "-50% fire hazard, costs $0.5/citizen/month"
            }
            Policy::OldTownHistoric => {
                "Prevents building changes, +15% tourism, -20% growth"
            }
            Policy::IndustrialSpacePlanning => "+50% industrial output, +10% pollution",
            Policy::RentControl => {
                "Prevents rent increases, -25% new construction rate"
            }
            Policy::MinimumWage => {
                "Sets wage floor: -20% poverty, +10% business costs"
            }
            Policy::TaxIncentiveZone => "-50% property tax, +25% construction rate",
            Policy::PetBan => "-10% garbage, -5 happiness",
            Policy::ParksAndRec => {
                "+10% park land value boost, +10% parks budget cost"
            }
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
            Policy::EminentDomain,
            Policy::CumulativeZoning,
            Policy::EncourageBiking,
            Policy::CombustionEngineBan,
            Policy::SmallBusinessEnthusiast,
            Policy::HeavyTrafficBan,
            Policy::SmokeDetectorDistribution,
            Policy::OldTownHistoric,
            Policy::IndustrialSpacePlanning,
            Policy::RentControl,
            Policy::MinimumWage,
            Policy::TaxIncentiveZone,
            Policy::PetBan,
            Policy::ParksAndRec,
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
        let mut mult = 1.0_f32;
        if self.is_active(Policy::IndustrialAirFilters) {
            mult *= 0.6;
        }
        if self.is_active(Policy::CombustionEngineBan) {
            mult *= 0.7;
        }
        if self.is_active(Policy::IndustrialSpacePlanning) {
            mult *= 1.1;
        }
        mult
    }

    /// Garbage multiplier
    pub fn garbage_multiplier(&self) -> f32 {
        let mut mult = 1.0_f32;
        if self.is_active(Policy::RecyclingProgram) {
            mult *= 0.7;
        }
        if self.is_active(Policy::PetBan) {
            mult *= 0.9;
        }
        mult
    }

    /// Park effectiveness multiplier
    pub fn park_multiplier(&self) -> f32 {
        let mut mult = 1.0_f32;
        if self.is_active(Policy::GreenSpaceInitiative) {
            mult *= 1.5;
        }
        if self.is_active(Policy::ParksAndRec) {
            mult *= 1.1;
        }
        mult
    }

    /// Max building level allowed (3 normally, 2 with HighRiseBan)
    pub fn max_building_level(&self) -> u8 {
        if self.is_active(Policy::HighRiseBan) {
            2
        } else {
            3
        }
    }

    /// Max commercial building level (3 normally, 2 with SmallBusinessEnthusiast)
    pub fn max_commercial_level(&self) -> u8 {
        if self.is_active(Policy::SmallBusinessEnthusiast) {
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

    /// Property tax multiplier (for TaxIncentiveZone)
    pub fn property_tax_multiplier(&self) -> f32 {
        if self.is_active(Policy::TaxIncentiveZone) {
            0.5
        } else {
            1.0
        }
    }

    /// Commercial demand bonus from tourism and business policies
    pub fn commercial_demand_bonus(&self) -> f32 {
        let mut bonus = 0.0;
        if self.is_active(Policy::TourismPromotion) {
            bonus += 0.15;
        }
        if self.is_active(Policy::SmallBusinessGrant) {
            bonus += 0.10;
        }
        if self.is_active(Policy::SmallBusinessEnthusiast) {
            bonus += 0.20;
        }
        if self.is_active(Policy::OldTownHistoric) {
            bonus += 0.15;
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
        if self.is_active(Policy::PetBan) {
            bonus -= 5.0;
        }
        if self.is_active(Policy::ParksAndRec) {
            bonus += 2.0;
        }
        if self.is_active(Policy::RentControl) {
            bonus += 3.0;
        }
        if self.is_active(Policy::MinimumWage) {
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

    /// Industrial demand bonus from tax breaks and planning
    pub fn industrial_demand_bonus(&self) -> f32 {
        let mut bonus = 0.0;
        if self.is_active(Policy::HeavyIndustryTaxBreak) {
            bonus += 0.15;
        }
        if self.is_active(Policy::IndustrialSpacePlanning) {
            bonus += 0.50;
        }
        bonus
    }
}
