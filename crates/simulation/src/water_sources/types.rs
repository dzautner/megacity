use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// =============================================================================
// Constants
// =============================================================================

/// 1 MGD = 1,000,000 gallons per day.
pub(crate) const MGD_TO_GPD: f32 = 1_000_000.0;

/// Groundwater well capacity in MGD.
pub(crate) const WELL_CAPACITY_MGD: f32 = 0.5;

/// Surface water intake capacity in MGD.
pub(crate) const SURFACE_INTAKE_CAPACITY_MGD: f32 = 5.0;

/// Reservoir capacity in MGD.
pub(crate) const RESERVOIR_CAPACITY_MGD: f32 = 20.0;

/// Desalination plant capacity in MGD.
pub(crate) const DESALINATION_CAPACITY_MGD: f32 = 10.0;

/// Reservoir storage buffer in days.
pub(crate) const RESERVOIR_BUFFER_DAYS: u32 = 90;

/// Reservoir footprint in grid cells (width x height).
pub(crate) const RESERVOIR_FOOTPRINT: (usize, usize) = (8, 8);

// =============================================================================
// Types
// =============================================================================

/// The type of water supply source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WaterSourceType {
    /// Groundwater well pump. Low cost, depends on groundwater level/quality.
    Well,
    /// Surface water intake. Must be placed on/near water cell.
    SurfaceIntake,
    /// Reservoir. Large footprint, stores 90-day buffer.
    Reservoir,
    /// Desalination plant. Placed on coast, very high cost, consistent quality.
    Desalination,
}

impl WaterSourceType {
    pub fn name(self) -> &'static str {
        match self {
            WaterSourceType::Well => "Groundwater Well",
            WaterSourceType::SurfaceIntake => "Surface Water Intake",
            WaterSourceType::Reservoir => "Reservoir",
            WaterSourceType::Desalination => "Desalination Plant",
        }
    }

    /// Base capacity in MGD (million gallons per day).
    pub fn capacity_mgd(self) -> f32 {
        match self {
            WaterSourceType::Well => WELL_CAPACITY_MGD,
            WaterSourceType::SurfaceIntake => SURFACE_INTAKE_CAPACITY_MGD,
            WaterSourceType::Reservoir => RESERVOIR_CAPACITY_MGD,
            WaterSourceType::Desalination => DESALINATION_CAPACITY_MGD,
        }
    }

    /// Construction cost.
    pub fn build_cost(self) -> f64 {
        match self {
            WaterSourceType::Well => 500.0,
            WaterSourceType::SurfaceIntake => 3_000.0,
            WaterSourceType::Reservoir => 15_000.0,
            WaterSourceType::Desalination => 20_000.0,
        }
    }

    /// Monthly operating cost (base, before quality adjustments).
    pub fn operating_cost(self) -> f64 {
        match self {
            WaterSourceType::Well => 15.0,
            WaterSourceType::SurfaceIntake => 80.0,
            WaterSourceType::Reservoir => 200.0,
            WaterSourceType::Desalination => 500.0,
        }
    }

    /// Footprint in grid cells (width, height).
    pub fn footprint(self) -> (usize, usize) {
        match self {
            WaterSourceType::Well => (1, 1),
            WaterSourceType::SurfaceIntake => (2, 2),
            WaterSourceType::Reservoir => RESERVOIR_FOOTPRINT,
            WaterSourceType::Desalination => (3, 3),
        }
    }

    /// Base water quality output (0.0 = contaminated, 1.0 = pure).
    pub fn base_quality(self) -> f32 {
        match self {
            WaterSourceType::Well => 0.7,
            WaterSourceType::SurfaceIntake => 0.6,
            WaterSourceType::Reservoir => 0.8,
            WaterSourceType::Desalination => 0.95,
        }
    }
}

/// Component attached to water source building entities.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct WaterSource {
    pub source_type: WaterSourceType,
    /// Effective capacity in MGD, may be reduced by environmental conditions.
    pub capacity_mgd: f32,
    /// Current water quality (0.0-1.0). Degrades when polluted.
    pub quality: f32,
    /// Current effective operating cost (increases with poor quality).
    pub operating_cost: f64,
    /// Grid position (top-left corner for multi-cell buildings).
    pub grid_x: usize,
    pub grid_y: usize,
    /// For reservoirs: current stored water in gallons.
    pub stored_gallons: f32,
    /// For reservoirs: maximum storage capacity in gallons.
    pub storage_capacity: f32,
}

impl WaterSource {
    /// Create a new water source with default values for the given type.
    pub fn new(source_type: WaterSourceType, grid_x: usize, grid_y: usize) -> Self {
        let storage_capacity = if source_type == WaterSourceType::Reservoir {
            source_type.capacity_mgd() * MGD_TO_GPD * RESERVOIR_BUFFER_DAYS as f32
        } else {
            0.0
        };

        Self {
            source_type,
            capacity_mgd: source_type.capacity_mgd(),
            quality: source_type.base_quality(),
            operating_cost: source_type.operating_cost(),
            grid_x,
            grid_y,
            stored_gallons: storage_capacity, // Start full
            storage_capacity,
        }
    }
}
