//! MILE-001: Milestone and Tech Tree System Overhaul.
//!
//! Defines 12 milestone tiers gated by population. Each tier automatically
//! unlocks a set of `UnlockNode`s when the city reaches the required
//! population. Progress toward the next milestone and notification events
//! are provided for UI display.

use bevy::prelude::*;
use bitcode::{Decode, Encode};

use crate::notifications::{NotificationEvent, NotificationPriority};
use crate::stats::CityStats;
use crate::unlocks::{UnlockNode, UnlockState};
use crate::SlowTickTimer;

// =============================================================================
// Milestone Tier Definition
// =============================================================================

/// The 12 milestone tiers a city progresses through.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub enum MilestoneTier {
    /// Tier 0: Starting (0 pop) - Basic roads, R/C/I zones, water, power
    Hamlet,
    /// Tier 1: 240 pop - Healthcare, deathcare, garbage
    SmallSettlement,
    /// Tier 2: 1,200 pop - Fire, police, elementary school
    Village,
    /// Tier 3: 2,600 pop - High school, parks, policies
    LargeVillage,
    /// Tier 4: 5,000 pop - Bus lines, unique buildings
    Town,
    /// Tier 5: 7,500 pop - High density zones, metro, office zones
    LargeTown,
    /// Tier 6: 12,000 pop - University, train, cargo
    SmallCity,
    /// Tier 7: 20,000 pop - Airport, ferry
    City,
    /// Tier 8: 36,000 pop - Tax office, more unique buildings
    LargeCity,
    /// Tier 9: 50,000 pop - Stock exchange, monument unlocks
    Metropolis,
    /// Tier 10: 65,000 pop - Advanced monuments
    LargeMetropolis,
    /// Tier 11: 80,000 pop - All monuments, all tiles
    Megalopolis,
}

impl MilestoneTier {
    /// All tiers in order.
    pub const ALL: &'static [MilestoneTier] = &[
        MilestoneTier::Hamlet,
        MilestoneTier::SmallSettlement,
        MilestoneTier::Village,
        MilestoneTier::LargeVillage,
        MilestoneTier::Town,
        MilestoneTier::LargeTown,
        MilestoneTier::SmallCity,
        MilestoneTier::City,
        MilestoneTier::LargeCity,
        MilestoneTier::Metropolis,
        MilestoneTier::LargeMetropolis,
        MilestoneTier::Megalopolis,
    ];

    /// Population required to reach this tier.
    pub fn required_population(self) -> u32 {
        match self {
            MilestoneTier::Hamlet => 0,
            MilestoneTier::SmallSettlement => 240,
            MilestoneTier::Village => 1_200,
            MilestoneTier::LargeVillage => 2_600,
            MilestoneTier::Town => 5_000,
            MilestoneTier::LargeTown => 7_500,
            MilestoneTier::SmallCity => 12_000,
            MilestoneTier::City => 20_000,
            MilestoneTier::LargeCity => 36_000,
            MilestoneTier::Metropolis => 50_000,
            MilestoneTier::LargeMetropolis => 65_000,
            MilestoneTier::Megalopolis => 80_000,
        }
    }

    /// Human-readable name for this tier.
    pub fn name(self) -> &'static str {
        match self {
            MilestoneTier::Hamlet => "Hamlet",
            MilestoneTier::SmallSettlement => "Small Settlement",
            MilestoneTier::Village => "Village",
            MilestoneTier::LargeVillage => "Large Village",
            MilestoneTier::Town => "Town",
            MilestoneTier::LargeTown => "Large Town",
            MilestoneTier::SmallCity => "Small City",
            MilestoneTier::City => "City",
            MilestoneTier::LargeCity => "Large City",
            MilestoneTier::Metropolis => "Metropolis",
            MilestoneTier::LargeMetropolis => "Large Metropolis",
            MilestoneTier::Megalopolis => "Megalopolis",
        }
    }

    /// Development points awarded when reaching this tier.
    pub fn dp_reward(self) -> u32 {
        match self {
            MilestoneTier::Hamlet => 0,
            MilestoneTier::SmallSettlement => 2,
            MilestoneTier::Village => 2,
            MilestoneTier::LargeVillage => 3,
            MilestoneTier::Town => 3,
            MilestoneTier::LargeTown => 3,
            MilestoneTier::SmallCity => 4,
            MilestoneTier::City => 4,
            MilestoneTier::LargeCity => 4,
            MilestoneTier::Metropolis => 5,
            MilestoneTier::LargeMetropolis => 5,
            MilestoneTier::Megalopolis => 6,
        }
    }

    /// The `UnlockNode`s that become available at this tier.
    pub fn unlocks(self) -> &'static [UnlockNode] {
        match self {
            MilestoneTier::Hamlet => &[
                UnlockNode::BasicRoads,
                UnlockNode::ResidentialZoning,
                UnlockNode::CommercialZoning,
                UnlockNode::IndustrialZoning,
                UnlockNode::BasicPower,
                UnlockNode::BasicWater,
            ],
            MilestoneTier::SmallSettlement => &[
                UnlockNode::HealthCare,
                UnlockNode::DeathCare,
                UnlockNode::BasicSanitation,
            ],
            MilestoneTier::Village => &[
                UnlockNode::FireService,
                UnlockNode::PoliceService,
                UnlockNode::ElementaryEducation,
            ],
            MilestoneTier::LargeVillage => &[
                UnlockNode::HighSchoolEducation,
                UnlockNode::SmallParks,
                UnlockNode::PolicySystem,
            ],
            MilestoneTier::Town => &[
                UnlockNode::PublicTransport,
                UnlockNode::Landmarks,
            ],
            MilestoneTier::LargeTown => &[
                UnlockNode::HighDensityResidential,
                UnlockNode::HighDensityCommercial,
                UnlockNode::AdvancedTransport,
                UnlockNode::OfficeZoning,
            ],
            MilestoneTier::SmallCity => &[
                UnlockNode::UniversityEducation,
                UnlockNode::AdvancedSanitation,
                UnlockNode::PostalService,
            ],
            MilestoneTier::City => &[
                UnlockNode::SmallAirstrips,
                UnlockNode::AdvancedParks,
                UnlockNode::WaterInfrastructure,
            ],
            MilestoneTier::LargeCity => &[
                UnlockNode::Telecom,
                UnlockNode::Entertainment,
                UnlockNode::BasicHeating,
            ],
            MilestoneTier::Metropolis => &[
                UnlockNode::RegionalAirports,
                UnlockNode::SolarPower,
                UnlockNode::WindPower,
                UnlockNode::SewagePlant,
            ],
            MilestoneTier::LargeMetropolis => &[
                UnlockNode::AdvancedEmergency,
                UnlockNode::DistrictHeatingNetwork,
                UnlockNode::NuclearPower,
            ],
            MilestoneTier::Megalopolis => &[
                UnlockNode::InternationalAirports,
            ],
        }
    }

    /// Return the next tier, or `None` if this is the final tier.
    pub fn next(self) -> Option<MilestoneTier> {
        let all = Self::ALL;
        let idx = all.iter().position(|&t| t == self)?;
        all.get(idx + 1).copied()
    }

    /// Return the tier index (0-11).
    pub fn index(self) -> usize {
        Self::ALL.iter().position(|&t| t == self).unwrap_or(0)
    }
}

// =============================================================================
// Milestone Progress Resource
// =============================================================================

/// Tracks the player's current milestone tier and progress toward the next.
#[derive(Resource, Debug, Clone, Encode, Decode)]
pub struct MilestoneProgress {
    /// The highest tier the city has reached.
    pub current_tier: MilestoneTier,
    /// Population when the current tier was reached.
    pub tier_reached_pop: u32,
    /// All tiers that have been reached (for notification dedup).
    pub reached_tiers: Vec<MilestoneTier>,
}

impl Default for MilestoneProgress {
    fn default() -> Self {
        Self {
            current_tier: MilestoneTier::Hamlet,
            tier_reached_pop: 0,
            reached_tiers: vec![MilestoneTier::Hamlet],
        }
    }
}

impl MilestoneProgress {
    /// The next tier the city is working toward, or `None` if at max.
    pub fn next_tier(&self) -> Option<MilestoneTier> {
        self.current_tier.next()
    }

    /// Population required for the next tier, or `None` if at max.
    pub fn next_tier_population(&self) -> Option<u32> {
        self.next_tier().map(|t| t.required_population())
    }

    /// Progress toward the next tier as a fraction (0.0 to 1.0).
    /// Returns 1.0 if at the final tier.
    pub fn progress_fraction(&self, current_pop: u32) -> f32 {
        let Some(next) = self.next_tier() else {
            return 1.0;
        };
        let current_threshold = self.current_tier.required_population();
        let next_threshold = next.required_population();
        let range = next_threshold.saturating_sub(current_threshold);
        if range == 0 {
            return 1.0;
        }
        let progress = current_pop.saturating_sub(current_threshold);
        (progress as f32 / range as f32).clamp(0.0, 1.0)
    }

    /// Check if a specific tier has been reached.
    pub fn has_reached(&self, tier: MilestoneTier) -> bool {
        self.reached_tiers.contains(&tier)
    }
}

// =============================================================================
// Milestone Progression System
// =============================================================================

/// Checks population against milestone tiers and auto-unlocks features.
/// Emits notification events when new tiers are reached.
#[allow(clippy::too_many_arguments)]
pub fn check_milestone_progression(
    slow_tick: Res<SlowTickTimer>,
    stats: Res<CityStats>,
    mut progress: ResMut<MilestoneProgress>,
    mut unlocks: ResMut<UnlockState>,
    mut notifications: EventWriter<NotificationEvent>,
) {
    if !slow_tick.should_run() {
        return;
    }

    let pop = stats.population;

    for &tier in MilestoneTier::ALL {
        if tier == MilestoneTier::Hamlet {
            continue; // Hamlet is always reached
        }
        if progress.has_reached(tier) {
            continue;
        }
        if pop < tier.required_population() {
            break; // Tiers are ordered; no need to check further
        }

        // New tier reached!
        progress.reached_tiers.push(tier);
        progress.current_tier = tier;
        progress.tier_reached_pop = pop;

        // Award development points
        let dp = tier.dp_reward();
        if dp > 0 {
            unlocks.development_points += dp;
        }

        // Auto-unlock all nodes for this tier
        for &node in tier.unlocks() {
            if !unlocks.is_unlocked(node) {
                unlocks.unlocked_nodes.push(node);
            }
        }

        // Update last_milestone_pop for backward compatibility
        unlocks.last_milestone_pop = tier.required_population();

        // Emit notification
        notifications.send(NotificationEvent {
            text: format!(
                "Milestone reached: {} ({} population)! {} unlocked.",
                tier.name(),
                tier.required_population(),
                tier.unlocks()
                    .iter()
                    .map(|n| n.name())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            priority: NotificationPriority::Positive,
            location: None,
        });
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct MilestonesPlugin;

impl Plugin for MilestonesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MilestoneProgress>().add_systems(
            FixedUpdate,
            check_milestone_progression
                .after(crate::stats::update_stats)
                .in_set(crate::SimulationSet::PostSim),
        );

        // Register saveable
        app.init_resource::<crate::SaveableRegistry>();
        let mut registry = app.world_mut().resource_mut::<crate::SaveableRegistry>();
        registry.register::<MilestoneProgress>();
    }
}

// =============================================================================
// Saveable Implementation
// =============================================================================

impl crate::Saveable for MilestoneProgress {
    const SAVE_KEY: &'static str = "milestone_progress";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.reached_tiers.len() <= 1 {
            return None; // Only Hamlet reached, skip saving
        }
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}
