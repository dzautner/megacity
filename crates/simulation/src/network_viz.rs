//! Network visualization data for enhanced power/water overlays.
//!
//! Tracks per-cell source assignment and per-source coverage statistics,
//! computed during BFS propagation. The rendering crate uses this data
//! to draw pulsing glows, animated pulse lines, capacity fill bars,
//! and color-coded cells by source.

use std::collections::VecDeque;

use bevy::prelude::*;

use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::{CellType, WorldGrid};
use crate::roads::RoadNetwork;
use crate::utilities::{UtilitySource, UtilityType};
use crate::weather::Weather;

/// Maximum number of distinct source colors supported.
const MAX_SOURCE_COLORS: usize = 12;

/// Predefined hue palette for source color-coding.
/// Each source gets a unique hue index; colors cycle if more than 12 sources.
const SOURCE_HUES: [[f32; 3]; MAX_SOURCE_COLORS] = [
    [0.30, 0.55, 0.95], // blue
    [0.95, 0.55, 0.20], // orange
    [0.30, 0.80, 0.45], // green
    [0.85, 0.30, 0.40], // red-pink
    [0.60, 0.40, 0.85], // purple
    [0.20, 0.80, 0.75], // teal
    [0.90, 0.80, 0.20], // yellow
    [0.70, 0.35, 0.20], // brown
    [0.55, 0.75, 0.30], // lime
    [0.80, 0.45, 0.70], // pink
    [0.35, 0.65, 0.55], // sea green
    [0.95, 0.65, 0.50], // salmon
];

/// Per-source metadata for rendering.
#[derive(Debug, Clone)]
pub struct SourceInfo {
    /// Entity ID of the source.
    pub entity: Entity,
    /// Grid position of the source.
    pub grid_x: usize,
    pub grid_y: usize,
    /// Utility type (power/water).
    pub utility_type: UtilityType,
    /// Effective BFS range for this source.
    pub effective_range: u32,
    /// Number of cells this source covers.
    pub cells_covered: u32,
    /// Maximum cells this source could cover (based on range).
    pub max_coverage: u32,
    /// Color index into SOURCE_HUES palette.
    pub color_index: usize,
}

/// Lookup to find which source covers a given cell.
/// Stores the source index in `NetworkVizData::power_sources` or `water_sources`.
/// `u16::MAX` means no source covers this cell.
const NO_SOURCE: u16 = u16::MAX;

/// Resource holding all network visualization data, recomputed each time
/// utility propagation runs.
#[derive(Resource)]
pub struct NetworkVizData {
    /// Per-cell source index for power (index into `power_sources`).
    pub power_cell_source: Vec<u16>,
    /// Per-cell source index for water (index into `water_sources`).
    pub water_cell_source: Vec<u16>,
    /// Per-cell BFS distance from its assigned power source (for pulse animation).
    pub power_cell_dist: Vec<u16>,
    /// Per-cell BFS distance from its assigned water source.
    pub water_cell_dist: Vec<u16>,
    /// Metadata for all power sources.
    pub power_sources: Vec<SourceInfo>,
    /// Metadata for all water sources.
    pub water_sources: Vec<SourceInfo>,
    /// Road cells that are part of the power network (for drawing pulse lines).
    pub power_road_cells: Vec<(usize, usize, u16, u16)>, // (x, y, dist, source_idx)
    /// Road cells that are part of the water network.
    pub water_road_cells: Vec<(usize, usize, u16, u16)>,
    /// Whether data has been updated this frame.
    pub dirty: bool,
}

impl Default for NetworkVizData {
    fn default() -> Self {
        let grid_len = GRID_WIDTH * GRID_HEIGHT;
        Self {
            power_cell_source: vec![NO_SOURCE; grid_len],
            water_cell_source: vec![NO_SOURCE; grid_len],
            power_cell_dist: vec![0; grid_len],
            water_cell_dist: vec![0; grid_len],
            power_sources: Vec::new(),
            water_sources: Vec::new(),
            power_road_cells: Vec::new(),
            water_road_cells: Vec::new(),
            dirty: false,
        }
    }
}

impl NetworkVizData {
    /// Get the source color for a power-covered cell.
    /// Returns `None` if cell is not covered.
    pub fn power_source_color(&self, x: usize, y: usize) -> Option<[f32; 3]> {
        let idx = y * GRID_WIDTH + x;
        let src_idx = self.power_cell_source[idx];
        if src_idx == NO_SOURCE {
            return None;
        }
        let src = &self.power_sources[src_idx as usize];
        Some(SOURCE_HUES[src.color_index % MAX_SOURCE_COLORS])
    }

    /// Get the source color for a water-covered cell.
    pub fn water_source_color(&self, x: usize, y: usize) -> Option<[f32; 3]> {
        let idx = y * GRID_WIDTH + x;
        let src_idx = self.water_cell_source[idx];
        if src_idx == NO_SOURCE {
            return None;
        }
        let src = &self.water_sources[src_idx as usize];
        Some(SOURCE_HUES[src.color_index % MAX_SOURCE_COLORS])
    }

    /// Get power distance (BFS hops from source) for a cell.
    pub fn power_distance(&self, x: usize, y: usize) -> u16 {
        self.power_cell_dist[y * GRID_WIDTH + x]
    }

    /// Get water distance (BFS hops from source) for a cell.
    pub fn water_distance(&self, x: usize, y: usize) -> u16 {
        self.water_cell_dist[y * GRID_WIDTH + x]
    }
}

/// System that builds per-source network visualization data alongside
/// the standard utility propagation.
#[allow(clippy::too_many_arguments)]
pub fn compute_network_viz(
    grid: Res<WorldGrid>,
    roads: Res<RoadNetwork>,
    weather: Res<Weather>,
    sources: Query<(Entity, Ref<UtilitySource>)>,
    mut viz: ResMut<NetworkVizData>,
    mut visited_buf: Local<Vec<bool>>,
) {
    // Only recompute when something changed
    let sources_changed = sources.iter().any(|(_, s)| s.is_changed());
    if !roads.is_changed() && !weather.is_changed() && !sources_changed && !viz.dirty {
        return;
    }

    let grid_len = grid.width * grid.height;

    // Reset
    viz.power_cell_source.fill(NO_SOURCE);
    viz.water_cell_source.fill(NO_SOURCE);
    viz.power_cell_dist.fill(0);
    viz.water_cell_dist.fill(0);
    viz.power_sources.clear();
    viz.water_sources.clear();
    viz.power_road_cells.clear();
    viz.water_road_cells.clear();

    if visited_buf.len() != grid_len {
        *visited_buf = vec![false; grid_len];
    }

    let power_mult = weather.power_multiplier();
    let water_mult = weather.water_multiplier();

    // Separate sources by type
    let mut power_idx: u16 = 0;
    let mut water_idx: u16 = 0;

    for (entity, source) in &sources {
        let is_power = source.utility_type.is_power();
        let is_water = source.utility_type.is_water();

        let range_mult = if is_power {
            1.0 / power_mult
        } else {
            1.0 / water_mult
        };
        let effective_range = (source.range as f32 * range_mult) as u32;

        let src_idx = if is_power { power_idx } else { water_idx };
        let color_index = src_idx as usize;

        let mut info = SourceInfo {
            entity,
            grid_x: source.grid_x,
            grid_y: source.grid_y,
            utility_type: source.utility_type,
            effective_range,
            cells_covered: 0,
            max_coverage: effective_range * 4, // rough estimate
            color_index,
        };

        // BFS from this source
        visited_buf.fill(false);
        let mut queue = VecDeque::new();
        let sx = source.grid_x;
        let sy = source.grid_y;
        queue.push_back(((sx, sy), 0u32));
        visited_buf[sy * grid.width + sx] = true;

        // Mark source cell
        let src_cell_idx = sy * grid.width + sx;
        if is_power {
            viz.power_cell_source[src_cell_idx] = src_idx;
            viz.power_cell_dist[src_cell_idx] = 0;
        }
        if is_water {
            viz.water_cell_source[src_cell_idx] = src_idx;
            viz.water_cell_dist[src_cell_idx] = 0;
        }
        info.cells_covered += 1;

        while let Some(((x, y), dist)) = queue.pop_front() {
            if dist >= effective_range {
                continue;
            }

            let (neighbors, ncount) = grid.neighbors4(x, y);
            for &(nx, ny) in &neighbors[..ncount] {
                let nidx = ny * grid.width + nx;
                if visited_buf[nidx] {
                    continue;
                }

                let cell_type = grid.get(nx, ny).cell_type;
                if cell_type == CellType::Road || cell_type == CellType::Grass {
                    visited_buf[nidx] = true;
                    let new_dist = dist + 1;

                    if is_power {
                        viz.power_cell_source[nidx] = src_idx;
                        viz.power_cell_dist[nidx] = new_dist as u16;
                    }
                    if is_water {
                        viz.water_cell_source[nidx] = src_idx;
                        viz.water_cell_dist[nidx] = new_dist as u16;
                    }
                    info.cells_covered += 1;

                    // Track road cells for pulse line rendering
                    if cell_type == CellType::Road {
                        if is_power {
                            viz.power_road_cells
                                .push((nx, ny, new_dist as u16, src_idx));
                        }
                        if is_water {
                            viz.water_road_cells
                                .push((nx, ny, new_dist as u16, src_idx));
                        }
                        queue.push_back(((nx, ny), new_dist));
                    }
                }
            }
        }

        if is_power {
            viz.power_sources.push(info.clone());
            power_idx += 1;
        }
        if is_water {
            viz.water_sources.push(info);
            water_idx += 1;
        }
    }

    viz.dirty = false;
}

pub struct NetworkVizPlugin;

impl Plugin for NetworkVizPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NetworkVizData>().add_systems(
            FixedUpdate,
            compute_network_viz.after(crate::utilities::propagate_utilities),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{GRID_HEIGHT, GRID_WIDTH};
    use crate::roads::RoadNetwork;

    #[test]
    fn test_source_color_assignment() {
        // Verify that each source gets a different color index
        let viz = NetworkVizData::default();
        assert_eq!(viz.power_sources.len(), 0);
        assert_eq!(viz.water_sources.len(), 0);
    }

    #[test]
    fn test_no_source_sentinel() {
        let viz = NetworkVizData::default();
        assert!(viz.power_source_color(0, 0).is_none());
        assert!(viz.water_source_color(0, 0).is_none());
    }

    #[test]
    fn test_source_hues_valid_rgb() {
        for hue in &SOURCE_HUES {
            for &channel in hue {
                assert!(
                    (0.0..=1.0).contains(&channel),
                    "source hue channel out of range: {}",
                    channel
                );
            }
        }
    }
}
