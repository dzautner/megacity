use bevy::prelude::*;

use simulation::education::EducationGrid;
use simulation::garbage::GarbageGrid;
use simulation::groundwater::{GroundwaterGrid, WaterQualityGrid};
use simulation::land_value::LandValueGrid;
use simulation::noise::NoisePollutionGrid;
use simulation::pollution::PollutionGrid;
use simulation::snow::SnowGrid;
use simulation::traffic::TrafficGrid;
use simulation::water_pollution::WaterPollutionGrid;

use crate::overlay::{DualOverlayMode, OverlayMode};

pub struct OverlayGrids<'a> {
    pub pollution: Option<&'a PollutionGrid>,
    pub land_value: Option<&'a LandValueGrid>,
    pub education: Option<&'a EducationGrid>,
    pub garbage: Option<&'a GarbageGrid>,
    pub traffic: Option<&'a TrafficGrid>,
    pub noise: Option<&'a NoisePollutionGrid>,
    pub water_pollution: Option<&'a WaterPollutionGrid>,
    pub groundwater: Option<&'a GroundwaterGrid>,
    pub water_quality: Option<&'a WaterQualityGrid>,
    pub snow: Option<&'a SnowGrid>,
}

impl<'a> OverlayGrids<'a> {
    pub fn none() -> Self {
        Self {
            pollution: None,
            land_value: None,
            education: None,
            garbage: None,
            traffic: None,
            noise: None,
            water_pollution: None,
            groundwater: None,
            water_quality: None,
            snow: None,
        }
    }
}

/// Parameters for dual-overlay blending/split passed into mesh building.
pub struct DualOverlayInfo {
    /// The secondary overlay mode (None = single overlay).
    pub secondary: OverlayMode,
    /// Blend or Split mode.
    pub mode: DualOverlayMode,
    /// Blend factor: 0.0 = only primary, 1.0 = only secondary.
    pub blend_factor: f32,
}

impl Default for DualOverlayInfo {
    fn default() -> Self {
        Self {
            secondary: OverlayMode::None,
            mode: DualOverlayMode::Blend,
            blend_factor: 0.5,
        }
    }
}

impl DualOverlayInfo {
    /// Whether dual overlay is active (both primary and secondary are non-None).
    pub fn is_active(&self, primary: &OverlayMode) -> bool {
        *primary != OverlayMode::None && self.secondary != OverlayMode::None
    }
}

#[derive(Component)]
pub struct TerrainChunk {
    pub chunk_x: usize,
    pub chunk_y: usize,
}

#[derive(Component)]
pub struct ChunkDirty;
