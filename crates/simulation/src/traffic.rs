use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::citizen::{Citizen, CitizenStateComp, PathCache};
use crate::config::{GRID_HEIGHT, GRID_WIDTH};
use crate::grid::RoadType;

#[derive(Resource, Serialize, Deserialize)]
pub struct TrafficGrid {
    pub density: Vec<u16>,
    pub width: usize,
    pub height: usize,
}

impl Default for TrafficGrid {
    fn default() -> Self {
        Self {
            density: vec![0; GRID_WIDTH * GRID_HEIGHT],
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
        }
    }
}

impl TrafficGrid {
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> u16 {
        self.density[y * self.width + x]
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, val: u16) {
        self.density[y * self.width + x] = val;
    }

    pub fn congestion_level(&self, x: usize, y: usize) -> f32 {
        let d = self.get(x, y) as f32;
        // 0 = free, 1.0 = fully congested
        (d / 20.0).min(1.0)
    }

    pub fn path_cost(&self, x: usize, y: usize) -> u32 {
        let base = 1u32;
        let congestion_penalty = (self.congestion_level(x, y) * 5.0) as u32;
        base + congestion_penalty
    }

    /// Path cost factoring in road type speed
    pub fn path_cost_with_road(&self, x: usize, y: usize, road_type: RoadType) -> u32 {
        let speed = road_type.speed();
        // Higher speed = lower cost. Normalize: local(30)=base, highway(100)=0.3x
        let speed_factor = 30.0 / speed;
        let base = (speed_factor * 1.0) as u32 + 1;
        let congestion_penalty = (self.congestion_level(x, y) * 5.0) as u32;
        base + congestion_penalty
    }

    pub fn clear(&mut self) {
        self.density.fill(0);
    }
}

pub fn update_traffic_density(
    tick: Res<crate::TickCounter>,
    mut traffic: ResMut<TrafficGrid>,
    citizens: Query<(&CitizenStateComp, &PathCache), With<Citizen>>,
) {
    if !tick.0.is_multiple_of(5) {
        return;
    }
    traffic.clear();

    for (state, path) in &citizens {
        if !state.0.is_commuting() {
            continue;
        }

        // Mark current path segment on traffic grid
        if let Some(target) = path.current_target() {
            let x = target.0.min(GRID_WIDTH - 1);
            let y = target.1.min(GRID_HEIGHT - 1);
            let current = traffic.get(x, y);
            traffic.set(x, y, current.saturating_add(1));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_traffic_density_math() {
        let mut traffic = TrafficGrid::default();
        traffic.set(10, 10, 10);
        assert_eq!(traffic.get(10, 10), 10);
        assert!((traffic.congestion_level(10, 10) - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_congestion_bounds() {
        let mut traffic = TrafficGrid::default();
        traffic.set(5, 5, 100);
        assert!(traffic.congestion_level(5, 5) <= 1.0);

        traffic.set(5, 5, 0);
        assert_eq!(traffic.congestion_level(5, 5), 0.0);
    }

    #[test]
    fn test_path_cost_increases_with_congestion() {
        let mut traffic = TrafficGrid::default();
        let cost_empty = traffic.path_cost(10, 10);

        traffic.set(10, 10, 20);
        let cost_congested = traffic.path_cost(10, 10);

        assert!(cost_congested > cost_empty);
    }
}

pub struct TrafficPlugin;

impl Plugin for TrafficPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TrafficGrid>().add_systems(
            FixedUpdate,
            update_traffic_density
                .after(crate::movement::move_citizens)
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}
