//! NIMBY/YIMBY citizen mechanics (ZONE-007).
//!
//! When the player rezones land or places a high-impact building, nearby
//! citizens generate opinions (opposition or support) based on personality,
//! property values, and the type of development.
//!
//! **NIMBY factors** (increase opposition):
//! - Density increase (e.g. low-res to high-res or industrial)
//! - Industrial adjacency to residential
//! - Traffic increase from commercial/industrial zones
//! - Income mismatch (high-income residents oppose low-density changes)
//!
//! **YIMBY factors** (increase support):
//! - Amenity addition (parks, transit coverage)
//! - Job creation (commercial/office near unemployed residents)
//! - Housing need (when residential vacancy is very low)
//!
//! **Effects**:
//! - Net opposition reduces happiness of nearby citizens
//! - Opposition strength scales with land value (wealthy areas oppose more)
//! - High opposition slows construction (increases `UnderConstruction` ticks)
//! - High opposition reduces building upgrade speed
//! - Protests are logged to the `EventJournal` as visual events
//! - Eminent Domain policy overrides opposition at a global happiness cost

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::buildings::{Building, UnderConstruction};
use crate::citizen::{Citizen, CitizenDetails, HomeLocation, Personality};
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::events::{CityEvent, CityEventType, EventJournal};
use crate::grid::{WorldGrid, ZoneType};
use crate::happiness::ServiceCoverageGrid;
use crate::land_value::LandValueGrid;
use crate::policies::{Policies, Policy};
use crate::time_of_day::GameClock;
use crate::wealth::WealthTier;
use crate::zones::ZoneDemand;
use crate::SlowTickTimer;
use crate::{Saveable, SaveableRegistry};

// =============================================================================
// Constants
// =============================================================================

/// Radius (in grid cells) within which citizens react to zone changes.
const REACTION_RADIUS: i32 = 8;

/// Maximum number of zone change events tracked at once (ring buffer).
const MAX_ZONE_CHANGES: usize = 64;

/// Number of ticks that a zone change event remains active before decaying.
const OPINION_DURATION_TICKS: u32 = 200;

/// Happiness penalty per unit of net opposition (scaled by land value).
const HAPPINESS_PENALTY_PER_OPPOSITION: f32 = 0.3;

/// Maximum happiness penalty from NIMBY opposition per citizen.
const MAX_NIMBY_HAPPINESS_PENALTY: f32 = 15.0;

/// Opposition threshold above which a protest event is triggered.
const PROTEST_THRESHOLD: f32 = 50.0;

/// Additional construction ticks added per unit of net opposition.
const CONSTRUCTION_SLOWDOWN_PER_OPPOSITION: f32 = 0.5;

/// Maximum additional construction ticks from opposition.
const MAX_CONSTRUCTION_SLOWDOWN: u32 = 50;

/// Happiness penalty when Eminent Domain policy is active.
pub const EMINENT_DOMAIN_HAPPINESS_PENALTY: f32 = 5.0;

/// Minimum ticks between protest events for the same zone change.
const PROTEST_COOLDOWN_TICKS: u32 = 100;

// =============================================================================
// Zone Change Event
// =============================================================================

/// Represents a zone change event that nearby citizens react to.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneChangeEvent {
    /// Grid coordinates of the zone change.
    pub grid_x: usize,
    pub grid_y: usize,
    /// The previous zone type at this location.
    pub old_zone: ZoneType,
    /// The new zone type at this location.
    pub new_zone: ZoneType,
    /// Game tick when this change occurred.
    pub created_tick: u64,
    /// Remaining ticks before this event expires.
    pub remaining_ticks: u32,
    /// Whether a protest has been triggered for this event.
    pub protest_triggered: bool,
    /// Cooldown counter for re-triggering protests.
    pub protest_cooldown: u32,
}

// =============================================================================
// NIMBY State Resource
// =============================================================================

/// Resource tracking all active zone changes and aggregate NIMBY statistics.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct NimbyState {
    /// Active zone change events that citizens are reacting to.
    pub zone_changes: Vec<ZoneChangeEvent>,
    /// Per-cell opposition score grid (0.0 = neutral/support, positive = opposition).
    /// Stored flat, indexed by `y * GRID_WIDTH + x`.
    pub opposition_grid: Vec<f32>,
    /// Total active protests in the city.
    pub active_protests: u32,
    /// Total zone changes processed since game start.
    pub total_changes_processed: u64,
}

impl Default for NimbyState {
    fn default() -> Self {
        Self {
            zone_changes: Vec::new(),
            opposition_grid: vec![0.0; GRID_WIDTH * GRID_HEIGHT],
            active_protests: 0,
            total_changes_processed: 0,
        }
    }
}

impl NimbyState {
    /// Get the opposition score at a given cell.
    #[inline]
    pub fn opposition_at(&self, x: usize, y: usize) -> f32 {
        self.opposition_grid[y * GRID_WIDTH + x]
    }
}

impl Saveable for NimbyState {
    const SAVE_KEY: &'static str = "nimby_state";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        // Skip saving if no active zone changes
        if self.zone_changes.is_empty() {
            return None;
        }

        let mut buf = Vec::new();

        // total_changes_processed (8 bytes)
        buf.extend_from_slice(&self.total_changes_processed.to_le_bytes());

        // active_protests (4 bytes)
        buf.extend_from_slice(&self.active_protests.to_le_bytes());

        // zone_changes count (4 bytes)
        let count = self.zone_changes.len() as u32;
        buf.extend_from_slice(&count.to_le_bytes());

        // Each zone change event
        for event in &self.zone_changes {
            buf.extend_from_slice(&(event.grid_x as u32).to_le_bytes());
            buf.extend_from_slice(&(event.grid_y as u32).to_le_bytes());
            buf.push(zone_type_to_u8(event.old_zone));
            buf.push(zone_type_to_u8(event.new_zone));
            buf.extend_from_slice(&event.created_tick.to_le_bytes());
            buf.extend_from_slice(&event.remaining_ticks.to_le_bytes());
            buf.push(event.protest_triggered as u8);
            buf.extend_from_slice(&event.protest_cooldown.to_le_bytes());
        }

        Some(buf)
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        let mut state = NimbyState::default();

        if bytes.len() < 16 {
            return state;
        }

        let mut offset = 0;

        // total_changes_processed
        if let Some(slice) = bytes.get(offset..offset + 8) {
            state.total_changes_processed = u64::from_le_bytes(slice.try_into().unwrap_or([0; 8]));
            offset += 8;
        }

        // active_protests
        if let Some(slice) = bytes.get(offset..offset + 4) {
            state.active_protests = u32::from_le_bytes(slice.try_into().unwrap_or([0; 4]));
            offset += 4;
        }

        // zone_changes count
        let count = if let Some(slice) = bytes.get(offset..offset + 4) {
            u32::from_le_bytes(slice.try_into().unwrap_or([0; 4])) as usize
        } else {
            return state;
        };
        offset += 4;

        // Each zone change event (27 bytes each: 4+4+1+1+8+4+1+4)
        for _ in 0..count.min(MAX_ZONE_CHANGES) {
            if offset + 27 > bytes.len() {
                break;
            }
            let grid_x =
                u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap_or([0; 4])) as usize;
            offset += 4;
            let grid_y =
                u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap_or([0; 4])) as usize;
            offset += 4;
            let old_zone = zone_type_from_u8(bytes[offset]);
            offset += 1;
            let new_zone = zone_type_from_u8(bytes[offset]);
            offset += 1;
            let created_tick =
                u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap_or([0; 8]));
            offset += 8;
            let remaining_ticks =
                u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap_or([0; 4]));
            offset += 4;
            let protest_triggered = bytes[offset] != 0;
            offset += 1;
            let protest_cooldown =
                u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap_or([0; 4]));
            offset += 4;

            state.zone_changes.push(ZoneChangeEvent {
                grid_x,
                grid_y,
                old_zone,
                new_zone,
                created_tick,
                remaining_ticks,
                protest_triggered,
                protest_cooldown,
            });
        }

        state
    }
}

// =============================================================================
// Zone Change Snapshot (for detecting changes)
// =============================================================================

/// Snapshot of the zone grid from the previous tick, used to detect rezoning.
#[derive(Resource)]
pub struct ZoneSnapshot {
    pub zones: Vec<ZoneType>,
}

impl Default for ZoneSnapshot {
    fn default() -> Self {
        Self {
            zones: vec![ZoneType::None; GRID_WIDTH * GRID_HEIGHT],
        }
    }
}

// =============================================================================
// Pure helper functions (testable without ECS)
// =============================================================================

/// Calculate a density score for a zone type. Higher values = higher impact.
pub fn zone_density_score(zone: ZoneType) -> f32 {
    match zone {
        ZoneType::None => 0.0,
        ZoneType::ResidentialLow => 1.0,
        ZoneType::ResidentialMedium => 2.0,
        ZoneType::ResidentialHigh => 3.0,
        ZoneType::CommercialLow => 1.5,
        ZoneType::CommercialHigh => 2.5,
        ZoneType::Industrial => 3.5,
        ZoneType::Office => 2.0,
        ZoneType::MixedUse => 2.5,
    }
}

/// Determine if a zone type is residential.
pub fn is_residential(zone: ZoneType) -> bool {
    matches!(
        zone,
        ZoneType::ResidentialLow | ZoneType::ResidentialMedium | ZoneType::ResidentialHigh
    )
}

/// Calculate the NIMBY opposition score for a zone change, from the perspective
/// of a citizen at a given distance with certain characteristics.
///
/// Returns a value where positive = opposition, negative = support.
#[allow(clippy::too_many_arguments)]
pub fn calculate_opinion(
    old_zone: ZoneType,
    new_zone: ZoneType,
    distance: f32,
    land_value: u8,
    citizen_education: u8,
    personality: &Personality,
    has_park_coverage: bool,
    has_transit_coverage: bool,
    residential_vacancy: f32,
) -> f32 {
    let mut score = 0.0;

    // --- NIMBY factors (positive = opposition) ---

    // 1. Density increase: citizens oppose higher-density development
    let density_change = zone_density_score(new_zone) - zone_density_score(old_zone);
    if density_change > 0.0 {
        score += density_change * 5.0;
    }

    // 2. Industrial adjacency: residential citizens strongly oppose industrial
    if new_zone == ZoneType::Industrial && old_zone != ZoneType::Industrial {
        score += 15.0;
    }

    // 3. Income mismatch: high-income citizens oppose high-density residential
    let wealth = WealthTier::from_education(citizen_education);
    if wealth == WealthTier::HighIncome
        && matches!(new_zone, ZoneType::ResidentialHigh | ZoneType::Industrial)
    {
        score += 8.0;
    }

    // --- YIMBY factors (negative = support) ---

    // 4. Job creation: commercial/office zones create jobs, citizens support this
    if matches!(
        new_zone,
        ZoneType::CommercialLow | ZoneType::CommercialHigh | ZoneType::Office
    ) && is_residential(old_zone)
    {
        // Only oppose if replacing residential; if replacing None, it's pure support
    } else if matches!(
        new_zone,
        ZoneType::CommercialLow | ZoneType::CommercialHigh | ZoneType::Office
    ) {
        score -= 5.0;
    }

    // 5. Housing need: if residential vacancy is very low, citizens support new housing
    if is_residential(new_zone) && residential_vacancy < 0.05 {
        score -= 8.0;
    }

    // 6. Amenity proximity bonus: if near parks/transit, development is more welcome
    if has_park_coverage {
        score -= 3.0;
    }
    if has_transit_coverage {
        score -= 2.0;
    }

    // 7. Mixed use is moderately welcomed (creates local services)
    if new_zone == ZoneType::MixedUse && !is_residential(old_zone) {
        score -= 3.0;
    }

    // --- Personality modifiers ---

    // Materialistic citizens care more about property values (oppose more)
    score *= 0.7 + personality.materialism * 0.6;

    // Resilient citizens are less bothered
    score *= 1.3 - personality.resilience * 0.5;

    // --- Land value scaling: wealthier areas oppose more ---
    let land_value_factor = 0.5 + (land_value as f32 / 255.0) * 1.0;
    score *= land_value_factor;

    // --- Distance falloff: opposition weakens with distance ---
    let distance_factor = if distance <= 1.0 {
        1.0
    } else {
        (1.0 / distance).max(0.1)
    };
    score *= distance_factor;

    score
}

/// Calculate the construction slowdown (additional ticks) based on opposition.
pub fn construction_slowdown(opposition: f32) -> u32 {
    if opposition <= 0.0 {
        return 0;
    }
    let extra = (opposition * CONSTRUCTION_SLOWDOWN_PER_OPPOSITION) as u32;
    extra.min(MAX_CONSTRUCTION_SLOWDOWN)
}

/// Calculate the happiness penalty for a citizen based on local opposition.
pub fn nimby_happiness_penalty(opposition: f32) -> f32 {
    if opposition <= 0.0 {
        return 0.0;
    }
    (opposition * HAPPINESS_PENALTY_PER_OPPOSITION).min(MAX_NIMBY_HAPPINESS_PENALTY)
}

// =============================================================================
// Systems
// =============================================================================

/// Detect zone changes by comparing the current grid against the previous snapshot.
/// Any cell whose zone type changed is recorded as a `ZoneChangeEvent`.
pub fn detect_zone_changes(
    grid: Res<WorldGrid>,
    tick: Res<crate::TickCounter>,
    mut nimby: ResMut<NimbyState>,
    mut snapshot: ResMut<ZoneSnapshot>,
) {
    if !grid.is_changed() {
        return;
    }

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let idx = y * GRID_WIDTH + x;
            let current_zone = grid.get(x, y).zone;
            let old_zone = snapshot.zones[idx];

            if current_zone != old_zone && current_zone != ZoneType::None {
                // Record the zone change
                let event = ZoneChangeEvent {
                    grid_x: x,
                    grid_y: y,
                    old_zone,
                    new_zone: current_zone,
                    created_tick: tick.0,
                    remaining_ticks: OPINION_DURATION_TICKS,
                    protest_triggered: false,
                    protest_cooldown: 0,
                };

                nimby.zone_changes.push(event);
                nimby.total_changes_processed += 1;

                // Trim to max tracked events
                if nimby.zone_changes.len() > MAX_ZONE_CHANGES {
                    nimby.zone_changes.remove(0);
                }
            }

            // Update snapshot
            snapshot.zones[idx] = current_zone;
        }
    }
}

/// Update the opposition grid based on active zone changes.
/// Decays old events and computes per-cell opposition scores.
///
/// Runs on the slow tick timer to avoid per-frame cost.
#[allow(clippy::too_many_arguments)]
pub fn update_nimby_opinions(
    timer: Res<SlowTickTimer>,
    mut nimby: ResMut<NimbyState>,
    land_value: Res<LandValueGrid>,
    coverage: Res<ServiceCoverageGrid>,
    demand: Res<ZoneDemand>,
    policies: Res<Policies>,
    clock: Res<GameClock>,
    mut journal: ResMut<EventJournal>,
) {
    if !timer.should_run() {
        return;
    }

    // If Eminent Domain is active, suppress all opposition
    let eminent_domain_active = policies.is_active(Policy::EminentDomain);

    // Decay and remove expired zone change events
    nimby.zone_changes.retain_mut(|event| {
        event.remaining_ticks = event.remaining_ticks.saturating_sub(1);
        if event.protest_cooldown > 0 {
            event.protest_cooldown = event.protest_cooldown.saturating_sub(1);
        }
        event.remaining_ticks > 0
    });

    // Clear opposition grid
    nimby.opposition_grid.fill(0.0);

    if nimby.zone_changes.is_empty() {
        nimby.active_protests = 0;
        return;
    }

    let residential_vacancy = demand.vacancy_residential;

    // For each active zone change, compute opposition at surrounding cells
    // Clone to avoid borrow conflict with opposition_grid mutation
    let zone_changes_snapshot = nimby.zone_changes.to_vec();
    for event in &zone_changes_snapshot {
        let ex = event.grid_x as i32;
        let ey = event.grid_y as i32;

        // Time decay factor: opposition fades as the event ages
        let time_factor = event.remaining_ticks as f32 / OPINION_DURATION_TICKS as f32;

        for dy in -REACTION_RADIUS..=REACTION_RADIUS {
            for dx in -REACTION_RADIUS..=REACTION_RADIUS {
                let nx = ex + dx;
                let ny = ey + dy;
                if nx < 0 || ny < 0 || nx >= GRID_WIDTH as i32 || ny >= GRID_HEIGHT as i32 {
                    continue;
                }
                let ux = nx as usize;
                let uy = ny as usize;

                let distance = ((dx * dx + dy * dy) as f32).sqrt();
                if distance > REACTION_RADIUS as f32 {
                    continue;
                }

                let lv = land_value.get(ux, uy);
                let cov_idx = ServiceCoverageGrid::idx(ux, uy);
                let has_park = coverage.flags[cov_idx] & crate::happiness::COVERAGE_PARK != 0;
                let has_transit =
                    coverage.flags[cov_idx] & crate::happiness::COVERAGE_TRANSPORT != 0;

                // Use a representative citizen profile for grid-level calculation
                // (education 2 = middle income, neutral personality)
                let neutral_personality = Personality {
                    ambition: 0.5,
                    sociability: 0.5,
                    materialism: 0.5,
                    resilience: 0.5,
                };

                let opinion = calculate_opinion(
                    event.old_zone,
                    event.new_zone,
                    distance,
                    lv,
                    2,
                    &neutral_personality,
                    has_park,
                    has_transit,
                    residential_vacancy,
                );

                // Apply time decay and eminent domain override
                let effective_opinion = if eminent_domain_active {
                    opinion.min(0.0) // Only keep support (negative scores), suppress opposition
                } else {
                    opinion * time_factor
                };

                let idx = uy * GRID_WIDTH + ux;
                nimby.opposition_grid[idx] += effective_opinion;
            }
        }
    }

    // Check for protest triggers
    // Pre-compute local opposition per event to avoid borrow conflict
    let local_oppositions: Vec<f32> = nimby
        .zone_changes
        .iter()
        .map(|event| {
            if event.protest_triggered && event.protest_cooldown > 0 {
                return 0.0;
            }
            let ex = event.grid_x;
            let ey = event.grid_y;
            let mut local_opposition = 0.0_f32;
            let check_radius: i32 = 3;
            for dy in -check_radius..=check_radius {
                for dx in -check_radius..=check_radius {
                    let nx = ex as i32 + dx;
                    let ny = ey as i32 + dy;
                    if nx >= 0
                        && ny >= 0
                        && (nx as usize) < GRID_WIDTH
                        && (ny as usize) < GRID_HEIGHT
                    {
                        let opp = nimby.opposition_grid[ny as usize * GRID_WIDTH + nx as usize];
                        if opp > 0.0 {
                            local_opposition += opp;
                        }
                    }
                }
            }
            local_opposition
        })
        .collect();

    let mut protest_count = 0u32;
    for (i, event) in nimby.zone_changes.iter_mut().enumerate() {
        if event.protest_triggered && event.protest_cooldown > 0 {
            continue;
        }
        let local_opposition = local_oppositions[i];

        if local_opposition >= PROTEST_THRESHOLD && !eminent_domain_active {
            protest_count += 1;
            event.protest_triggered = true;
            event.protest_cooldown = PROTEST_COOLDOWN_TICKS;

            let zone_name = zone_type_name(event.new_zone);
            journal.push(CityEvent {
                event_type: CityEventType::NewPolicy(format!(
                    "Protest at ({}, {})",
                    event.grid_x, event.grid_y
                )),
                day: clock.day,
                hour: clock.hour,
                description: format!(
                    "Citizens are protesting {} development near ({}, {}). Opposition: {:.0}",
                    zone_name, event.grid_x, event.grid_y, local_opposition
                ),
            });
        }
    }

    nimby.active_protests = protest_count;
}

/// Apply NIMBY happiness penalties to citizens near opposed development.
/// Also applies the Eminent Domain global happiness penalty.
pub fn apply_nimby_happiness(
    timer: Res<SlowTickTimer>,
    nimby: Res<NimbyState>,
    policies: Res<Policies>,
    mut citizens: Query<(&HomeLocation, &mut CitizenDetails, &Personality), With<Citizen>>,
) {
    if !timer.should_run() {
        return;
    }

    // Skip if no active opposition and no eminent domain
    let eminent_domain_active = policies.is_active(Policy::EminentDomain);
    if nimby.zone_changes.is_empty() && !eminent_domain_active {
        return;
    }

    citizens
        .par_iter_mut()
        .for_each(|(home, mut details, personality)| {
            let opposition = nimby.opposition_at(home.grid_x, home.grid_y);

            // Per-citizen personality adjustment to opposition
            let personal_opposition = opposition
                * (0.7 + personality.materialism * 0.6)
                * (1.3 - personality.resilience * 0.5);

            let penalty = nimby_happiness_penalty(personal_opposition);
            if penalty > 0.0 {
                details.happiness = (details.happiness - penalty).max(0.0);
            }

            // Eminent Domain policy: global happiness penalty
            if eminent_domain_active {
                details.happiness = (details.happiness - EMINENT_DOMAIN_HAPPINESS_PENALTY).max(0.0);
            }
        });
}

/// Slow down construction of buildings in high-opposition areas.
pub fn apply_construction_slowdown(
    timer: Res<SlowTickTimer>,
    nimby: Res<NimbyState>,
    policies: Res<Policies>,
    mut buildings: Query<(&Building, &mut UnderConstruction)>,
) {
    if !timer.should_run() {
        return;
    }

    // Eminent Domain bypasses construction slowdown
    if policies.is_active(Policy::EminentDomain) {
        return;
    }

    if nimby.zone_changes.is_empty() {
        return;
    }

    for (building, mut construction) in &mut buildings {
        let opposition = nimby.opposition_at(building.grid_x, building.grid_y);
        let extra_ticks = construction_slowdown(opposition);
        if extra_ticks > 0 {
            construction.ticks_remaining += extra_ticks;
            construction.total_ticks += extra_ticks;
        }
    }
}

// =============================================================================
// Helper functions
// =============================================================================

/// Convert a ZoneType to a u8 for serialization.
fn zone_type_to_u8(zone: ZoneType) -> u8 {
    match zone {
        ZoneType::None => 0,
        ZoneType::ResidentialLow => 1,
        ZoneType::ResidentialMedium => 2,
        ZoneType::ResidentialHigh => 3,
        ZoneType::CommercialLow => 4,
        ZoneType::CommercialHigh => 5,
        ZoneType::Industrial => 6,
        ZoneType::Office => 7,
        ZoneType::MixedUse => 8,
    }
}

/// Convert a u8 back to a ZoneType for deserialization.
fn zone_type_from_u8(val: u8) -> ZoneType {
    match val {
        0 => ZoneType::None,
        1 => ZoneType::ResidentialLow,
        2 => ZoneType::ResidentialMedium,
        3 => ZoneType::ResidentialHigh,
        4 => ZoneType::CommercialLow,
        5 => ZoneType::CommercialHigh,
        6 => ZoneType::Industrial,
        7 => ZoneType::Office,
        8 => ZoneType::MixedUse,
        _ => ZoneType::None,
    }
}

/// Human-readable name for a zone type.
fn zone_type_name(zone: ZoneType) -> &'static str {
    match zone {
        ZoneType::None => "empty",
        ZoneType::ResidentialLow => "low-density residential",
        ZoneType::ResidentialMedium => "medium-density residential",
        ZoneType::ResidentialHigh => "high-density residential",
        ZoneType::CommercialLow => "low-density commercial",
        ZoneType::CommercialHigh => "high-density commercial",
        ZoneType::Industrial => "industrial",
        ZoneType::Office => "office",
        ZoneType::MixedUse => "mixed-use",
    }
}

// =============================================================================
// Plugin
// =============================================================================

pub struct NimbyPlugin;

impl Plugin for NimbyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NimbyState>()
            .init_resource::<ZoneSnapshot>()
            .add_systems(
                FixedUpdate,
                (
                    detect_zone_changes,
                    update_nimby_opinions,
                    apply_nimby_happiness,
                    apply_construction_slowdown,
                )
                    .chain()
                    .after(crate::zones::update_zone_demand),
            );
        app.init_resource::<SaveableRegistry>();
        app.world_mut()
            .resource_mut::<SaveableRegistry>()
            .register::<NimbyState>();
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Zone density score tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_zone_density_scores_increase_with_density() {
        assert!(
            zone_density_score(ZoneType::ResidentialLow)
                < zone_density_score(ZoneType::ResidentialMedium)
        );
        assert!(
            zone_density_score(ZoneType::ResidentialMedium)
                < zone_density_score(ZoneType::ResidentialHigh)
        );
        assert!(
            zone_density_score(ZoneType::CommercialLow)
                < zone_density_score(ZoneType::CommercialHigh)
        );
        assert!(zone_density_score(ZoneType::None) < zone_density_score(ZoneType::ResidentialLow));
    }

    #[test]
    fn test_industrial_has_highest_density_score() {
        assert!(
            zone_density_score(ZoneType::Industrial)
                > zone_density_score(ZoneType::ResidentialHigh)
        );
    }

    // -------------------------------------------------------------------------
    // is_residential tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_is_residential() {
        assert!(is_residential(ZoneType::ResidentialLow));
        assert!(is_residential(ZoneType::ResidentialMedium));
        assert!(is_residential(ZoneType::ResidentialHigh));
        assert!(!is_residential(ZoneType::CommercialLow));
        assert!(!is_residential(ZoneType::Industrial));
        assert!(!is_residential(ZoneType::None));
        assert!(!is_residential(ZoneType::Office));
        assert!(!is_residential(ZoneType::MixedUse));
    }

    // -------------------------------------------------------------------------
    // Opinion calculation tests
    // -------------------------------------------------------------------------

    fn default_personality() -> Personality {
        Personality {
            ambition: 0.5,
            sociability: 0.5,
            materialism: 0.5,
            resilience: 0.5,
        }
    }

    #[test]
    fn test_density_increase_causes_opposition() {
        let opinion = calculate_opinion(
            ZoneType::ResidentialLow,
            ZoneType::ResidentialHigh,
            1.0,
            128,
            2,
            &default_personality(),
            false,
            false,
            0.10,
        );
        assert!(
            opinion > 0.0,
            "density increase should cause opposition, got {}",
            opinion
        );
    }

    #[test]
    fn test_industrial_rezoning_causes_strong_opposition() {
        let opinion = calculate_opinion(
            ZoneType::ResidentialLow,
            ZoneType::Industrial,
            1.0,
            128,
            2,
            &default_personality(),
            false,
            false,
            0.10,
        );
        assert!(
            opinion > 10.0,
            "industrial rezoning should cause strong opposition, got {}",
            opinion
        );
    }

    #[test]
    fn test_job_creation_reduces_opposition() {
        // Commercial on empty land should be supportive
        let opinion = calculate_opinion(
            ZoneType::None,
            ZoneType::CommercialLow,
            1.0,
            128,
            2,
            &default_personality(),
            false,
            false,
            0.10,
        );
        // Should have job creation support (-5) plus some density cost
        // Net should be lower than purely density-driven
        let opinion_no_jobs = calculate_opinion(
            ZoneType::None,
            ZoneType::Industrial,
            1.0,
            128,
            2,
            &default_personality(),
            false,
            false,
            0.10,
        );
        assert!(
            opinion < opinion_no_jobs,
            "job creation should produce less opposition than industrial"
        );
    }

    #[test]
    fn test_housing_need_creates_support() {
        // Very low vacancy should generate support for new residential
        let opinion_low_vacancy = calculate_opinion(
            ZoneType::None,
            ZoneType::ResidentialHigh,
            1.0,
            128,
            2,
            &default_personality(),
            false,
            false,
            0.02, // very low vacancy
        );
        let opinion_high_vacancy = calculate_opinion(
            ZoneType::None,
            ZoneType::ResidentialHigh,
            1.0,
            128,
            2,
            &default_personality(),
            false,
            false,
            0.20, // plenty of vacancy
        );
        assert!(
            opinion_low_vacancy < opinion_high_vacancy,
            "low vacancy should produce more support: low={}, high={}",
            opinion_low_vacancy,
            opinion_high_vacancy
        );
    }

    #[test]
    fn test_park_coverage_reduces_opposition() {
        let opinion_no_park = calculate_opinion(
            ZoneType::None,
            ZoneType::ResidentialHigh,
            1.0,
            128,
            2,
            &default_personality(),
            false,
            false,
            0.10,
        );
        let opinion_with_park = calculate_opinion(
            ZoneType::None,
            ZoneType::ResidentialHigh,
            1.0,
            128,
            2,
            &default_personality(),
            true,
            false,
            0.10,
        );
        assert!(
            opinion_with_park < opinion_no_park,
            "park coverage should reduce opposition: park={}, no_park={}",
            opinion_with_park,
            opinion_no_park
        );
    }

    #[test]
    fn test_transit_coverage_reduces_opposition() {
        let opinion_no_transit = calculate_opinion(
            ZoneType::None,
            ZoneType::ResidentialHigh,
            1.0,
            128,
            2,
            &default_personality(),
            false,
            false,
            0.10,
        );
        let opinion_with_transit = calculate_opinion(
            ZoneType::None,
            ZoneType::ResidentialHigh,
            1.0,
            128,
            2,
            &default_personality(),
            false,
            true,
            0.10,
        );
        assert!(
            opinion_with_transit < opinion_no_transit,
            "transit should reduce opposition: transit={}, no_transit={}",
            opinion_with_transit,
            opinion_no_transit
        );
    }

    #[test]
    fn test_distance_reduces_opposition() {
        let close_opinion = calculate_opinion(
            ZoneType::None,
            ZoneType::Industrial,
            1.0,
            128,
            2,
            &default_personality(),
            false,
            false,
            0.10,
        );
        let far_opinion = calculate_opinion(
            ZoneType::None,
            ZoneType::Industrial,
            6.0,
            128,
            2,
            &default_personality(),
            false,
            false,
            0.10,
        );
        assert!(
            far_opinion < close_opinion,
            "distance should reduce opposition: close={}, far={}",
            close_opinion,
            far_opinion
        );
    }

    #[test]
    fn test_high_land_value_increases_opposition() {
        let low_lv_opinion = calculate_opinion(
            ZoneType::None,
            ZoneType::Industrial,
            1.0,
            50, // low land value
            2,
            &default_personality(),
            false,
            false,
            0.10,
        );
        let high_lv_opinion = calculate_opinion(
            ZoneType::None,
            ZoneType::Industrial,
            1.0,
            250, // high land value
            2,
            &default_personality(),
            false,
            false,
            0.10,
        );
        assert!(
            high_lv_opinion > low_lv_opinion,
            "high land value should increase opposition: high={}, low={}",
            high_lv_opinion,
            low_lv_opinion
        );
    }

    #[test]
    fn test_high_income_opposes_more() {
        let mid_income = calculate_opinion(
            ZoneType::None,
            ZoneType::ResidentialHigh,
            1.0,
            128,
            2, // middle income
            &default_personality(),
            false,
            false,
            0.10,
        );
        let high_income = calculate_opinion(
            ZoneType::None,
            ZoneType::ResidentialHigh,
            1.0,
            128,
            3, // high income
            &default_personality(),
            false,
            false,
            0.10,
        );
        assert!(
            high_income > mid_income,
            "high income should oppose more: high={}, mid={}",
            high_income,
            mid_income
        );
    }

    #[test]
    fn test_materialistic_personality_opposes_more() {
        let low_mat = Personality {
            ambition: 0.5,
            sociability: 0.5,
            materialism: 0.2,
            resilience: 0.5,
        };
        let high_mat = Personality {
            ambition: 0.5,
            sociability: 0.5,
            materialism: 0.9,
            resilience: 0.5,
        };
        let low_opinion = calculate_opinion(
            ZoneType::None,
            ZoneType::Industrial,
            1.0,
            128,
            2,
            &low_mat,
            false,
            false,
            0.10,
        );
        let high_opinion = calculate_opinion(
            ZoneType::None,
            ZoneType::Industrial,
            1.0,
            128,
            2,
            &high_mat,
            false,
            false,
            0.10,
        );
        assert!(
            high_opinion > low_opinion,
            "materialistic should oppose more: high={}, low={}",
            high_opinion,
            low_opinion
        );
    }

    #[test]
    fn test_resilient_personality_opposes_less() {
        let low_res = Personality {
            ambition: 0.5,
            sociability: 0.5,
            materialism: 0.5,
            resilience: 0.2,
        };
        let high_res = Personality {
            ambition: 0.5,
            sociability: 0.5,
            materialism: 0.5,
            resilience: 0.9,
        };
        let low_opinion = calculate_opinion(
            ZoneType::None,
            ZoneType::Industrial,
            1.0,
            128,
            2,
            &low_res,
            false,
            false,
            0.10,
        );
        let high_opinion = calculate_opinion(
            ZoneType::None,
            ZoneType::Industrial,
            1.0,
            128,
            2,
            &high_res,
            false,
            false,
            0.10,
        );
        assert!(
            high_opinion < low_opinion,
            "resilient should oppose less: high={}, low={}",
            high_opinion,
            low_opinion
        );
    }

    // -------------------------------------------------------------------------
    // Construction slowdown tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_construction_slowdown_zero_opposition() {
        assert_eq!(construction_slowdown(0.0), 0);
        assert_eq!(construction_slowdown(-5.0), 0);
    }

    #[test]
    fn test_construction_slowdown_moderate_opposition() {
        let slow = construction_slowdown(20.0);
        assert!(
            slow > 0 && slow <= MAX_CONSTRUCTION_SLOWDOWN,
            "moderate opposition should cause some slowdown: {}",
            slow
        );
    }

    #[test]
    fn test_construction_slowdown_capped() {
        let slow = construction_slowdown(1000.0);
        assert_eq!(
            slow, MAX_CONSTRUCTION_SLOWDOWN,
            "slowdown should be capped at max: {}",
            slow
        );
    }

    // -------------------------------------------------------------------------
    // Happiness penalty tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_happiness_penalty_zero() {
        assert_eq!(nimby_happiness_penalty(0.0), 0.0);
        assert_eq!(nimby_happiness_penalty(-5.0), 0.0);
    }

    #[test]
    fn test_happiness_penalty_moderate() {
        let penalty = nimby_happiness_penalty(10.0);
        assert!(
            penalty > 0.0 && penalty <= MAX_NIMBY_HAPPINESS_PENALTY,
            "moderate opposition should cause some penalty: {}",
            penalty
        );
    }

    #[test]
    fn test_happiness_penalty_capped() {
        let penalty = nimby_happiness_penalty(1000.0);
        assert!(
            (penalty - MAX_NIMBY_HAPPINESS_PENALTY).abs() < f32::EPSILON,
            "penalty should be capped: {}",
            penalty
        );
    }

    // -------------------------------------------------------------------------
    // NimbyState tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_nimby_state_default() {
        let state = NimbyState::default();
        assert!(state.zone_changes.is_empty());
        assert_eq!(state.opposition_grid.len(), GRID_WIDTH * GRID_HEIGHT);
        assert_eq!(state.active_protests, 0);
        assert_eq!(state.total_changes_processed, 0);
        assert_eq!(state.opposition_at(0, 0), 0.0);
        assert_eq!(state.opposition_at(128, 128), 0.0);
    }

    #[test]
    fn test_nimby_state_set_get_opposition() {
        let mut state = NimbyState::default();
        state.opposition_grid[10 * GRID_WIDTH + 10] = 25.0;
        assert_eq!(state.opposition_at(10, 10), 25.0);
        assert_eq!(state.opposition_at(0, 0), 0.0);
    }

    // -------------------------------------------------------------------------
    // ZoneSnapshot tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_zone_snapshot_default() {
        let snapshot = ZoneSnapshot::default();
        assert_eq!(snapshot.zones.len(), GRID_WIDTH * GRID_HEIGHT);
        assert_eq!(snapshot.zones[0], ZoneType::None);
    }

    // -------------------------------------------------------------------------
    // Saveable implementation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_saveable_skip_default() {
        let state = NimbyState::default();
        assert!(
            state.save_to_bytes().is_none(),
            "default state should skip saving"
        );
    }

    #[test]
    fn test_saveable_roundtrip() {
        let mut state = NimbyState::default();
        state.zone_changes.push(ZoneChangeEvent {
            grid_x: 10,
            grid_y: 20,
            old_zone: ZoneType::None,
            new_zone: ZoneType::Industrial,
            created_tick: 100,
            remaining_ticks: 150,
            protest_triggered: true,
            protest_cooldown: 50,
        });
        state.total_changes_processed = 5;

        let bytes = state
            .save_to_bytes()
            .expect("non-default state should save");
        let restored = NimbyState::load_from_bytes(&bytes);

        assert_eq!(restored.zone_changes.len(), 1);
        assert_eq!(restored.zone_changes[0].grid_x, 10);
        assert_eq!(restored.zone_changes[0].grid_y, 20);
        assert_eq!(restored.zone_changes[0].new_zone, ZoneType::Industrial);
        assert_eq!(restored.zone_changes[0].remaining_ticks, 150);
        assert!(restored.zone_changes[0].protest_triggered);
        assert_eq!(restored.zone_changes[0].protest_cooldown, 50);
        assert_eq!(restored.total_changes_processed, 5);
        // Opposition grid is recomputed each tick, not saved
        assert_eq!(restored.opposition_at(10, 20), 0.0);
    }

    // -------------------------------------------------------------------------
    // Zone type name tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_zone_type_name_coverage() {
        // Ensure all zone types have names
        assert!(!zone_type_name(ZoneType::None).is_empty());
        assert!(!zone_type_name(ZoneType::ResidentialLow).is_empty());
        assert!(!zone_type_name(ZoneType::ResidentialMedium).is_empty());
        assert!(!zone_type_name(ZoneType::ResidentialHigh).is_empty());
        assert!(!zone_type_name(ZoneType::CommercialLow).is_empty());
        assert!(!zone_type_name(ZoneType::CommercialHigh).is_empty());
        assert!(!zone_type_name(ZoneType::Industrial).is_empty());
        assert!(!zone_type_name(ZoneType::Office).is_empty());
        assert!(!zone_type_name(ZoneType::MixedUse).is_empty());
    }

    // -------------------------------------------------------------------------
    // Constants validation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_constants_are_reasonable() {
        assert!(REACTION_RADIUS > 0);
        assert!(MAX_ZONE_CHANGES > 0);
        assert!(OPINION_DURATION_TICKS > 0);
        assert!(HAPPINESS_PENALTY_PER_OPPOSITION > 0.0);
        assert!(MAX_NIMBY_HAPPINESS_PENALTY > 0.0);
        assert!(PROTEST_THRESHOLD > 0.0);
        assert!(CONSTRUCTION_SLOWDOWN_PER_OPPOSITION > 0.0);
        assert!(MAX_CONSTRUCTION_SLOWDOWN > 0);
        assert!(EMINENT_DOMAIN_HAPPINESS_PENALTY > 0.0);
        assert!(PROTEST_COOLDOWN_TICKS > 0);
    }

    // -------------------------------------------------------------------------
    // Mixed-use support tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_mixed_use_on_empty_land_is_welcomed() {
        let opinion = calculate_opinion(
            ZoneType::None,
            ZoneType::MixedUse,
            1.0,
            128,
            2,
            &default_personality(),
            false,
            false,
            0.10,
        );
        // MixedUse on empty land has density increase but also mixed-use support
        // It should be less opposed than pure high-density industrial
        let industrial_opinion = calculate_opinion(
            ZoneType::None,
            ZoneType::Industrial,
            1.0,
            128,
            2,
            &default_personality(),
            false,
            false,
            0.10,
        );
        assert!(
            opinion < industrial_opinion,
            "mixed-use should be less opposed than industrial: mixed={}, ind={}",
            opinion,
            industrial_opinion
        );
    }
}
