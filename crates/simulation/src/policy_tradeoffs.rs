//! Policy tradeoff definitions and computed effects resource (POL-001).
//!
//! Each policy has clearly defined positive and negative effects, organized
//! as `PolicyTradeoff` structs. The `PolicyTradeoffEffects` resource is
//! recomputed every slow tick from the active policies in `Policies`.

use bevy::prelude::*;

use crate::policies::Policy;

// =============================================================================
// Tradeoff data structure
// =============================================================================

/// A single positive or negative effect of a policy.
#[derive(Debug, Clone)]
pub struct PolicyEffect {
    /// Human-readable description of the effect.
    pub description: &'static str,
    /// Magnitude of the effect (positive = benefit, negative = cost).
    /// Expressed as a percentage or absolute value depending on context.
    pub magnitude: f32,
}

/// Complete tradeoff definition for a policy.
#[derive(Debug, Clone)]
pub struct PolicyTradeoff {
    /// The policy this tradeoff describes.
    pub policy: Policy,
    /// Category for UI grouping.
    pub category: PolicyCategory,
    /// Positive effects (benefits) of enabling this policy.
    pub benefits: &'static [(&'static str, f32)],
    /// Negative effects (costs) of enabling this policy.
    pub drawbacks: &'static [(&'static str, f32)],
}

/// Policy categories for UI grouping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PolicyCategory {
    Economy,
    Environment,
    Social,
    Zoning,
    Transport,
    PublicSafety,
}

impl PolicyCategory {
    pub fn name(self) -> &'static str {
        match self {
            PolicyCategory::Economy => "Economy",
            PolicyCategory::Environment => "Environment",
            PolicyCategory::Social => "Social",
            PolicyCategory::Zoning => "Zoning",
            PolicyCategory::Transport => "Transport",
            PolicyCategory::PublicSafety => "Public Safety",
        }
    }
}

// =============================================================================
// Tradeoff definitions for all 29 policies
// =============================================================================

/// Get the tradeoff definition for a policy.
pub fn get_tradeoff(policy: Policy) -> PolicyTradeoff {
    match policy {
        Policy::FreePublicTransport => PolicyTradeoff {
            policy,
            category: PolicyCategory::Transport,
            benefits: &[
                ("Transit ridership +30%", 30.0),
                ("Happiness +3", 3.0),
                ("Reduced traffic congestion", 15.0),
            ],
            drawbacks: &[
                ("All transit revenue lost", -100.0),
                ("Monthly cost $50", -50.0),
            ],
        },
        Policy::HeavyIndustryTaxBreak => PolicyTradeoff {
            policy,
            category: PolicyCategory::Economy,
            benefits: &[
                ("Industrial demand +15%", 15.0),
                ("Attracts heavy industry", 10.0),
            ],
            drawbacks: &[("Industrial tax income -50%", -50.0)],
        },
        Policy::TourismPromotion => PolicyTradeoff {
            policy,
            category: PolicyCategory::Economy,
            benefits: &[("Commercial demand +15%", 15.0)],
            drawbacks: &[("Monthly cost $30", -30.0)],
        },
        Policy::SmallBusinessGrant => PolicyTradeoff {
            policy,
            category: PolicyCategory::Economy,
            benefits: &[("Commercial demand +10%", 10.0)],
            drawbacks: &[("Monthly cost $25", -25.0)],
        },
        Policy::RecyclingProgram => PolicyTradeoff {
            policy,
            category: PolicyCategory::Environment,
            benefits: &[("Garbage reduced 30%", 30.0)],
            drawbacks: &[
                ("Garbage budget +10%", -10.0),
                ("Monthly cost $20", -20.0),
            ],
        },
        Policy::IndustrialAirFilters => PolicyTradeoff {
            policy,
            category: PolicyCategory::Environment,
            benefits: &[("Industrial pollution -40%", 40.0)],
            drawbacks: &[("Monthly cost $35", -35.0)],
        },
        Policy::WaterConservation => PolicyTradeoff {
            policy,
            category: PolicyCategory::Environment,
            benefits: &[("Water consumption -20%", 20.0)],
            drawbacks: &[("Monthly cost $10", -10.0)],
        },
        Policy::GreenSpaceInitiative => PolicyTradeoff {
            policy,
            category: PolicyCategory::Environment,
            benefits: &[("Park effectiveness +50%", 50.0)],
            drawbacks: &[("Monthly cost $15", -15.0)],
        },
        Policy::EducationPush => PolicyTradeoff {
            policy,
            category: PolicyCategory::Social,
            benefits: &[("Education speed +50%", 50.0)],
            drawbacks: &[
                ("Education spending +50%", -50.0),
                ("Monthly cost $40", -40.0),
            ],
        },
        Policy::HealthcareForAll => PolicyTradeoff {
            policy,
            category: PolicyCategory::Social,
            benefits: &[
                ("Health coverage +100%", 100.0),
                ("Happiness +2", 2.0),
                ("Poverty reduction +10%", 10.0),
            ],
            drawbacks: &[("Monthly cost $45", -45.0)],
        },
        Policy::SmokeDetectorMandate => PolicyTradeoff {
            policy,
            category: PolicyCategory::PublicSafety,
            benefits: &[("Fire risk -30%", 30.0)],
            drawbacks: &[("Monthly cost $10", -10.0)],
        },
        Policy::NeighborhoodWatch => PolicyTradeoff {
            policy,
            category: PolicyCategory::PublicSafety,
            benefits: &[
                ("Crime reduction", 15.0),
                ("Happiness +2", 2.0),
            ],
            drawbacks: &[("Monthly cost $15", -15.0)],
        },
        Policy::HighRiseBan => PolicyTradeoff {
            policy,
            category: PolicyCategory::Zoning,
            benefits: &[
                ("Preserves neighborhood character", 10.0),
                ("Reduced density pressure", 5.0),
            ],
            drawbacks: &[
                ("Prevents level 4-5 buildings", -40.0),
                ("Limits housing capacity", -30.0),
            ],
        },
        Policy::NightShiftBan => PolicyTradeoff {
            policy,
            category: PolicyCategory::Social,
            benefits: &[("Happiness +3", 3.0)],
            drawbacks: &[("Commercial output reduced", -15.0)],
        },
        Policy::IndustrialZoningRestriction => PolicyTradeoff {
            policy,
            category: PolicyCategory::Zoning,
            benefits: &[("Less industrial near residential", 20.0)],
            drawbacks: &[("Limits industrial expansion", -15.0)],
        },
        Policy::EminentDomain => PolicyTradeoff {
            policy,
            category: PolicyCategory::Zoning,
            benefits: &[("Override NIMBY opposition", 30.0)],
            drawbacks: &[
                ("Happiness penalty", -5.0),
                ("Monthly cost $20", -20.0),
            ],
        },
        Policy::CumulativeZoning => PolicyTradeoff {
            policy,
            category: PolicyCategory::Zoning,
            benefits: &[("Flexible zone usage (Euclidean hierarchy)", 20.0)],
            drawbacks: &[("May increase mixed-use conflicts", -5.0)],
        },
        Policy::EncourageBiking => PolicyTradeoff {
            policy,
            category: PolicyCategory::Transport,
            benefits: &[
                ("Cycling rate +15%", 15.0),
                ("Car trips -10%", 10.0),
            ],
            drawbacks: &[("Monthly cost $15", -15.0)],
        },
        Policy::CombustionEngineBan => PolicyTradeoff {
            policy,
            category: PolicyCategory::Transport,
            benefits: &[
                ("Pollution -30%", 30.0),
                ("Noise -30%", 30.0),
                ("Transit ridership +40%", 40.0),
                ("Cycling +20%", 20.0),
            ],
            drawbacks: &[
                ("No private cars", -100.0),
                ("Monthly cost $30", -30.0),
                ("Reduced accessibility for elderly/disabled", -10.0),
            ],
        },
        Policy::SmallBusinessEnthusiast => PolicyTradeoff {
            policy,
            category: PolicyCategory::Economy,
            benefits: &[
                ("Small business growth +20%", 20.0),
                ("Commercial demand +20%", 20.0),
            ],
            drawbacks: &[
                ("Caps commercial at level 2", -60.0),
                ("Monthly cost $20", -20.0),
            ],
        },
        Policy::HeavyTrafficBan => PolicyTradeoff {
            policy,
            category: PolicyCategory::Transport,
            benefits: &[
                ("Road noise -40%", 40.0),
                ("Road wear reduced", 20.0),
            ],
            drawbacks: &[
                ("Industrial output -15%", -15.0),
                ("Freight logistics impaired", -10.0),
                ("Monthly cost $10", -10.0),
            ],
        },
        Policy::SmokeDetectorDistribution => PolicyTradeoff {
            policy,
            category: PolicyCategory::PublicSafety,
            benefits: &[("Fire hazard -50%", 50.0)],
            drawbacks: &[("Costs $0.5/citizen/month", -0.5)],
        },
        Policy::OldTownHistoric => PolicyTradeoff {
            policy,
            category: PolicyCategory::Zoning,
            benefits: &[
                ("Tourism boost +15%", 15.0),
                ("Preserves historic character", 20.0),
            ],
            drawbacks: &[
                ("No building changes allowed", -100.0),
                ("Growth -20%", -20.0),
                ("Monthly cost $5", -5.0),
            ],
        },
        Policy::IndustrialSpacePlanning => PolicyTradeoff {
            policy,
            category: PolicyCategory::Economy,
            benefits: &[
                ("Industrial output +50%", 50.0),
                ("Industrial demand +50%", 50.0),
            ],
            drawbacks: &[
                ("Pollution +10%", -10.0),
                ("Monthly cost $25", -25.0),
            ],
        },
        Policy::RentControl => PolicyTradeoff {
            policy,
            category: PolicyCategory::Social,
            benefits: &[
                ("Prevents rent increases", 30.0),
                ("Happiness +3", 3.0),
                ("Reduces displacement", 15.0),
            ],
            drawbacks: &[
                ("New construction -25%", -25.0),
                ("Monthly cost $10", -10.0),
            ],
        },
        Policy::MinimumWage => PolicyTradeoff {
            policy,
            category: PolicyCategory::Social,
            benefits: &[
                ("Poverty -20%", 20.0),
                ("Happiness +2", 2.0),
            ],
            drawbacks: &[
                ("Business costs +10%", -10.0),
                ("Monthly cost $20", -20.0),
            ],
        },
        Policy::TaxIncentiveZone => PolicyTradeoff {
            policy,
            category: PolicyCategory::Economy,
            benefits: &[("Construction rate +25%", 25.0)],
            drawbacks: &[("Property tax -50%", -50.0)],
        },
        Policy::PetBan => PolicyTradeoff {
            policy,
            category: PolicyCategory::Environment,
            benefits: &[("Garbage -10%", 10.0)],
            drawbacks: &[
                ("Happiness -5", -5.0),
                ("Monthly cost $5", -5.0),
            ],
        },
        Policy::ParksAndRec => PolicyTradeoff {
            policy,
            category: PolicyCategory::Environment,
            benefits: &[
                ("Park land value +10%", 10.0),
                ("Happiness +2", 2.0),
            ],
            drawbacks: &[
                ("Parks budget +10%", -10.0),
                ("Monthly cost $20", -20.0),
            ],
        },
    }
}

// =============================================================================
// Computed effects resource
// =============================================================================

/// Aggregated effects of all active policies, recomputed each slow tick.
///
/// Other simulation systems read this resource to apply policy tradeoff
/// effects without needing to check individual policy toggles.
#[derive(Resource, Debug, Clone)]
pub struct PolicyTradeoffEffects {
    /// Pollution multiplier (1.0 = normal, lower = less pollution).
    pub pollution_multiplier: f32,
    /// Garbage multiplier (1.0 = normal, lower = less garbage).
    pub garbage_multiplier: f32,
    /// Park effectiveness multiplier.
    pub park_multiplier: f32,
    /// Max building level allowed (all zone types).
    pub max_building_level: u8,
    /// Max commercial building level.
    pub max_commercial_level: u8,
    /// Industrial tax multiplier (1.0 = normal).
    pub industrial_tax_multiplier: f32,
    /// Property tax multiplier (1.0 = normal).
    pub property_tax_multiplier: f32,
    /// Commercial demand bonus (additive).
    pub commercial_demand_bonus: f32,
    /// Happiness bonus (additive).
    pub happiness_bonus: f32,
    /// Education speed multiplier.
    pub education_multiplier: f32,
    /// Industrial demand bonus (additive).
    pub industrial_demand_bonus: f32,
    /// Industrial output multiplier.
    pub industrial_output_multiplier: f32,
    /// Noise multiplier (1.0 = normal, lower = less noise).
    pub noise_multiplier: f32,
    /// Fire hazard multiplier (1.0 = normal, lower = safer).
    pub fire_hazard_multiplier: f32,
    /// Construction rate multiplier.
    pub construction_rate_multiplier: f32,
    /// Whether building changes are blocked (OldTownHistoric).
    pub building_changes_blocked: bool,
    /// Whether private cars are banned.
    pub private_cars_banned: bool,
    /// Whether heavy trucks are banned.
    pub heavy_trucks_banned: bool,
    /// Transit ridership bonus (additive fraction).
    pub transit_ridership_bonus: f32,
    /// Cycling rate bonus (additive fraction).
    pub cycling_rate_bonus: f32,
    /// Car trip multiplier (1.0 = normal, lower = fewer).
    pub car_trip_multiplier: f32,
    /// Poverty reduction factor (additive fraction).
    pub poverty_reduction: f32,
    /// Business cost multiplier (1.0 = normal, higher = more expensive).
    pub business_cost_multiplier: f32,
    /// Total monthly cost of all active policies.
    pub total_monthly_cost: f64,
    /// Number of active policies.
    pub active_policy_count: u32,
}

impl Default for PolicyTradeoffEffects {
    fn default() -> Self {
        Self {
            pollution_multiplier: 1.0,
            garbage_multiplier: 1.0,
            park_multiplier: 1.0,
            max_building_level: 3,
            max_commercial_level: 3,
            industrial_tax_multiplier: 1.0,
            property_tax_multiplier: 1.0,
            commercial_demand_bonus: 0.0,
            happiness_bonus: 0.0,
            education_multiplier: 1.0,
            industrial_demand_bonus: 0.0,
            industrial_output_multiplier: 1.0,
            noise_multiplier: 1.0,
            fire_hazard_multiplier: 1.0,
            construction_rate_multiplier: 1.0,
            building_changes_blocked: false,
            private_cars_banned: false,
            heavy_trucks_banned: false,
            transit_ridership_bonus: 0.0,
            cycling_rate_bonus: 0.0,
            car_trip_multiplier: 1.0,
            poverty_reduction: 0.0,
            business_cost_multiplier: 1.0,
            total_monthly_cost: 0.0,
            active_policy_count: 0,
        }
    }
}

/// Recompute all policy tradeoff effects from the active policy set.
pub fn compute_effects(policies: &crate::policies::Policies) -> PolicyTradeoffEffects {
    PolicyTradeoffEffects {
        pollution_multiplier: policies.pollution_multiplier(),
        garbage_multiplier: policies.garbage_multiplier(),
        park_multiplier: policies.park_multiplier(),
        max_building_level: policies.max_building_level(),
        max_commercial_level: policies.max_commercial_level(),
        industrial_tax_multiplier: policies.industrial_tax_multiplier(),
        property_tax_multiplier: policies.property_tax_multiplier(),
        commercial_demand_bonus: policies.commercial_demand_bonus(),
        happiness_bonus: policies.happiness_bonus(),
        education_multiplier: policies.education_multiplier(),
        industrial_demand_bonus: policies.industrial_demand_bonus(),
        industrial_output_multiplier: policies.industrial_output_multiplier(),
        noise_multiplier: policies.noise_multiplier(),
        fire_hazard_multiplier: policies.fire_hazard_multiplier(),
        construction_rate_multiplier: policies.construction_rate_multiplier(),
        building_changes_blocked: policies.building_changes_blocked(),
        private_cars_banned: policies.private_cars_banned(),
        heavy_trucks_banned: policies.heavy_trucks_banned(),
        transit_ridership_bonus: policies.transit_ridership_bonus(),
        cycling_rate_bonus: policies.cycling_rate_bonus(),
        car_trip_multiplier: policies.car_trip_multiplier(),
        poverty_reduction: policies.poverty_reduction(),
        business_cost_multiplier: policies.business_cost_multiplier(),
        total_monthly_cost: policies.total_monthly_cost(),
        active_policy_count: policies.active.len() as u32,
    }
}
