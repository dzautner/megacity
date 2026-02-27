use bevy::prelude::*;
use bitcode::{Decode, Encode};

use crate::buildings::Building;
use crate::grid::{CellType, WorldGrid};
use crate::stats::CityStats;
use crate::time_of_day::GameClock;
use crate::utilities::{UtilitySource, UtilityType};
use crate::Saveable;

// =============================================================================
// Tutorial Step Definition
// =============================================================================

/// The sequential steps of the onboarding tutorial.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode)]
pub enum TutorialStep {
    /// Welcome message and overview.
    Welcome,
    /// Place your first road.
    PlaceRoad,
    /// Zone residential area next to the road.
    ZoneResidential,
    /// Zone commercial area.
    ZoneCommercial,
    /// Place a power plant to supply electricity.
    PlacePowerPlant,
    /// Place a water tower to supply water.
    PlaceWaterTower,
    /// Observe city growth (wait for population).
    ObserveGrowth,
    /// Review and manage the budget.
    ManageBudget,
    /// Tutorial complete.
    Completed,
}

impl TutorialStep {
    /// All steps in order for iteration.
    pub const ALL: &'static [TutorialStep] = &[
        TutorialStep::Welcome,
        TutorialStep::PlaceRoad,
        TutorialStep::ZoneResidential,
        TutorialStep::ZoneCommercial,
        TutorialStep::PlacePowerPlant,
        TutorialStep::PlaceWaterTower,
        TutorialStep::ObserveGrowth,
        TutorialStep::ManageBudget,
        TutorialStep::Completed,
    ];

    /// Human-readable title for this step.
    pub fn title(self) -> &'static str {
        match self {
            TutorialStep::Welcome => "Welcome to Megacity!",
            TutorialStep::PlaceRoad => "Step 1: Place a Road",
            TutorialStep::ZoneResidential => "Step 2: Zone Residential",
            TutorialStep::ZoneCommercial => "Step 3: Zone Commercial",
            TutorialStep::PlacePowerPlant => "Step 4: Place a Power Plant",
            TutorialStep::PlaceWaterTower => "Step 5: Place a Water Tower",
            TutorialStep::ObserveGrowth => "Step 6: Watch Your City Grow",
            TutorialStep::ManageBudget => "Step 7: Manage Your Budget",
            TutorialStep::Completed => "Tutorial Complete!",
        }
    }

    /// Detailed instruction text for this step.
    pub fn description(self) -> &'static str {
        match self {
            TutorialStep::Welcome => {
                "Welcome, Mayor! In this tutorial you will learn the basics of \
                 building a thriving city. We will guide you through placing \
                 roads, zoning areas, providing utilities, and managing your \
                 budget. Click 'Next' to begin, or 'Skip Tutorial' if you are \
                 already experienced."
            }
            TutorialStep::PlaceRoad => {
                "Roads are the foundation of your city. Open the 'Roads' category \
                 in the bottom toolbar and select 'Local Road'. Click and drag \
                 on the map to place a road segment of at least 5 cells. Roads \
                 allow buildings to grow along them."
            }
            TutorialStep::ZoneResidential => {
                "Now let's create homes for your citizens. Open the 'Zones' \
                 category and select 'Res Low' (low-density residential). Paint \
                 at least 10 zone cells adjacent to your road. Buildings will \
                 appear once power and water are available."
            }
            TutorialStep::ZoneCommercial => {
                "Citizens need places to work and shop. Open the 'Zones' category \
                 and select 'Com Low' (low-density commercial). Zone at least 5 \
                 cells near your road, ideally close to the residential area."
            }
            TutorialStep::PlacePowerPlant => {
                "Buildings need electricity to function. Open the 'Utilities' \
                 category and place a 'Power Plant' ($800) near your zones. It \
                 will supply power to nearby buildings within its range."
            }
            TutorialStep::PlaceWaterTower => {
                "Buildings also need water. Open the 'Utilities' category and \
                 place a 'Water Tower' ($600) near your zones. With both power \
                 and water supplied, buildings will begin to develop."
            }
            TutorialStep::ObserveGrowth => {
                "Excellent! Your city now has the basics: roads, zones, power, \
                 and water. Unpause the simulation and watch as buildings grow \
                 and citizens move in. Wait until at least one building appears \
                 and your population reaches 5 to continue."
            }
            TutorialStep::ManageBudget => {
                "As your city grows, you will earn tax revenue and incur \
                 expenses. Press 'B' or check the info panel to review your \
                 budget. You can adjust the tax rate to balance income and \
                 spending. Your treasury is shown in the top bar."
            }
            TutorialStep::Completed => {
                "Congratulations! You have completed the tutorial. You now know \
                 how to build roads, zone areas, provide utilities, and manage \
                 your budget. Continue building your city to unlock milestones \
                 and achievements. Good luck, Mayor!"
            }
        }
    }

    /// Hint text shown below the description to guide the player.
    pub fn hint(self) -> &'static str {
        match self {
            TutorialStep::Welcome => "Click 'Next' to start the tutorial.",
            TutorialStep::PlaceRoad => "Hint: Select Roads > Local Road and drag to place at least 5 cells.",
            TutorialStep::ZoneResidential => {
                "Hint: Select Zones > Res Low and paint at least 10 cells next to your road."
            }
            TutorialStep::ZoneCommercial => {
                "Hint: Select Zones > Com Low and paint at least 5 cells near your residential zone."
            }
            TutorialStep::PlacePowerPlant => {
                "Hint: Select Utilities > Power Plant and place it near zones."
            }
            TutorialStep::PlaceWaterTower => {
                "Hint: Select Utilities > Water Tower and place it near zones."
            }
            TutorialStep::ObserveGrowth => {
                "Hint: Press Space to unpause. Wait for buildings and population to reach 5."
            }
            TutorialStep::ManageBudget => "Hint: Press 'B' to open the budget panel.",
            TutorialStep::Completed => "You can now close this window.",
        }
    }

    /// Returns the index of this step in the ALL array.
    pub fn index(self) -> usize {
        TutorialStep::ALL
            .iter()
            .position(|&s| s == self)
            .unwrap_or(0)
    }

    /// Returns the next step, or None if this is the last step.
    pub fn next(self) -> Option<TutorialStep> {
        let idx = self.index();
        TutorialStep::ALL.get(idx + 1).copied()
    }

    /// Returns the previous step, or None if this is the first step.
    pub fn previous(self) -> Option<TutorialStep> {
        let idx = self.index();
        if idx > 0 {
            TutorialStep::ALL.get(idx - 1).copied()
        } else {
            None
        }
    }

    /// Total number of steps (excluding Completed).
    pub fn total_steps() -> usize {
        TutorialStep::ALL.len() - 1 // exclude Completed from count
    }
}

// =============================================================================
// Tutorial State Resource
// =============================================================================

/// Tracks the player's progress through the onboarding tutorial.
#[derive(Resource, Debug, Clone, Encode, Decode)]
pub struct TutorialState {
    /// Current step in the tutorial.
    pub current_step: TutorialStep,
    /// Whether the tutorial has been completed or skipped.
    pub completed: bool,
    /// Whether the tutorial is actively being shown.
    pub active: bool,
    /// Whether the simulation was paused by the tutorial (to restore on skip/complete).
    pub paused_by_tutorial: bool,
}

impl Default for TutorialState {
    fn default() -> Self {
        Self {
            current_step: TutorialStep::Welcome,
            completed: false,
            active: false, // Only activated explicitly on New Game
            paused_by_tutorial: false,
        }
    }
}

impl TutorialState {
    /// Skip the tutorial entirely.
    pub fn skip(&mut self) {
        self.current_step = TutorialStep::Completed;
        self.completed = true;
        self.active = false;
        self.paused_by_tutorial = false;
    }

    /// Advance to the next step. Returns true if advanced, false if already completed.
    pub fn advance(&mut self) -> bool {
        if self.completed {
            return false;
        }
        if let Some(next) = self.current_step.next() {
            self.current_step = next;
            if next == TutorialStep::Completed {
                self.completed = true;
                self.active = false;
                self.paused_by_tutorial = false;
            }
            true
        } else {
            self.completed = true;
            self.active = false;
            self.paused_by_tutorial = false;
            false
        }
    }

    /// Go back to the previous step. Returns true if moved back.
    pub fn go_back(&mut self) -> bool {
        if self.completed {
            return false;
        }
        if let Some(prev) = self.current_step.previous() {
            self.current_step = prev;
            true
        } else {
            false
        }
    }

    /// Whether the current step requires manual advancement (Next button).
    pub fn is_manual_step(&self) -> bool {
        matches!(
            self.current_step,
            TutorialStep::Welcome | TutorialStep::ManageBudget | TutorialStep::Completed
        )
    }
}

// =============================================================================
// Saveable Implementation
// =============================================================================

impl Saveable for TutorialState {
    const SAVE_KEY: &'static str = "tutorial";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        if self.completed && !self.active {
            // Only save if completed (to remember skip/completion)
            Some(bitcode::encode(self))
        } else if self.current_step != TutorialStep::Welcome {
            // Save in-progress state
            Some(bitcode::encode(self))
        } else {
            None // Default state, no need to save
        }
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// =============================================================================
// Tutorial Progress Thresholds
// =============================================================================

/// Minimum number of road cells required to complete the PlaceRoad step.
pub const MIN_ROAD_CELLS: usize = 5;
/// Minimum number of residential zone cells required to complete the ZoneResidential step.
pub const MIN_RESIDENTIAL_ZONES: usize = 10;
/// Minimum number of commercial zone cells required to complete the ZoneCommercial step.
pub const MIN_COMMERCIAL_ZONES: usize = 5;

// =============================================================================
// Tutorial Progress Detection System
// =============================================================================

/// System that checks whether the player has completed the current tutorial step's
/// objective and automatically advances to the next step.
#[allow(clippy::too_many_arguments)]
pub fn check_tutorial_progress(
    mut tutorial: ResMut<TutorialState>,
    grid: Res<WorldGrid>,
    stats: Res<CityStats>,
    utility_sources: Query<&UtilitySource>,
    buildings: Query<&Building>,
    mut clock: ResMut<GameClock>,
) {
    if !tutorial.active || tutorial.completed {
        return;
    }

    // Pause during instruction steps (not during ObserveGrowth or ManageBudget)
    let should_pause = matches!(
        tutorial.current_step,
        TutorialStep::Welcome
            | TutorialStep::PlaceRoad
            | TutorialStep::ZoneResidential
            | TutorialStep::ZoneCommercial
            | TutorialStep::PlacePowerPlant
            | TutorialStep::PlaceWaterTower
    );

    if should_pause && !clock.paused {
        clock.paused = true;
        tutorial.paused_by_tutorial = true;
    }

    // Manual steps (Welcome, ManageBudget, Completed) advance via UI button only
    if tutorial.is_manual_step() {
        return;
    }

    let step = tutorial.current_step;
    let completed = match step {
        TutorialStep::PlaceRoad => {
            // Require at least MIN_ROAD_CELLS road cells so players learn to
            // drag a meaningful road segment, not just click once.
            let count = grid
                .cells
                .iter()
                .filter(|cell| cell.cell_type == CellType::Road)
                .count();
            count >= MIN_ROAD_CELLS
        }
        TutorialStep::ZoneResidential => {
            // Require at least MIN_RESIDENTIAL_ZONES residential zone cells
            // so players learn to paint a proper residential area.
            let count = grid
                .cells
                .iter()
                .filter(|cell| cell.zone.is_residential())
                .count();
            count >= MIN_RESIDENTIAL_ZONES
        }
        TutorialStep::ZoneCommercial => {
            // Require at least MIN_COMMERCIAL_ZONES commercial zone cells
            // so players learn to paint a proper commercial area.
            let count = grid
                .cells
                .iter()
                .filter(|cell| cell.zone.is_commercial())
                .count();
            count >= MIN_COMMERCIAL_ZONES
        }
        TutorialStep::PlacePowerPlant => {
            // Check if there's at least one power utility
            utility_sources.iter().any(|u| u.utility_type.is_power())
        }
        TutorialStep::PlaceWaterTower => {
            // Check if there's at least one water utility
            utility_sources
                .iter()
                .any(|u| u.utility_type == UtilityType::WaterTower)
        }
        TutorialStep::ObserveGrowth => {
            // Require at least one residential or commercial building to have
            // actually spawned (not just zoned), AND population >= 5.
            let has_building = buildings.iter().any(|b| {
                b.zone_type.is_residential() || b.zone_type.is_commercial()
            });
            has_building && stats.population >= 5
        }
        _ => false,
    };

    if completed {
        // Unpause if we paused for this step
        if tutorial.paused_by_tutorial {
            clock.paused = false;
            tutorial.paused_by_tutorial = false;
        }
        tutorial.advance();
    }
}
// =============================================================================
// Plugin
// =============================================================================

pub struct TutorialPlugin;

impl Plugin for TutorialPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TutorialState>().add_systems(
            Update,
            check_tutorial_progress.in_set(crate::SimulationUpdateSet::Visual),
        );

        // Register for save/load via the SaveableRegistry
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<TutorialState>();
    }
}
