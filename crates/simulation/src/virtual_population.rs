use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Hard ceiling on real (ECS) citizens — never exceed this regardless of FPS.
pub const MAX_REAL_CITIZENS_HARD: u32 = 200_000;
/// Minimum real citizens — always keep at least this many.
pub const MIN_REAL_CITIZENS: u32 = 10_000;
/// Default cap that adjusts dynamically.
pub const DEFAULT_REAL_CITIZEN_CAP: u32 = 50_000;

/// Per-district virtual population statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DistrictStats {
    pub population: u32,
    pub employed: u32,
    pub avg_happiness: f32,
    pub avg_age: f32,
    /// Age brackets: [0-17, 18-34, 35-54, 55-64, 65+]
    pub age_brackets: [u32; 5],
    /// Approximate commuter flow out of this district
    pub commuters_out: u32,
    /// Tax contribution from virtual citizens
    pub tax_contribution: f32,
    /// Service demand pressure (0.0-1.0) from virtual citizens
    pub service_demand: f32,
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct VirtualPopulation {
    pub total_virtual: u32,
    pub virtual_employed: u32,
    /// Per-district statistics for virtual citizens.
    pub district_stats: Vec<DistrictStats>,
    /// Dynamic cap on real (ECS) citizens, adjusted based on frame time.
    pub max_real_citizens: u32,
    /// Smoothed frame time (seconds) used for cap adjustment.
    smoothed_frame_time: f32,
}

impl Default for VirtualPopulation {
    fn default() -> Self {
        Self {
            total_virtual: 0,
            virtual_employed: 0,
            district_stats: Vec::new(),
            max_real_citizens: DEFAULT_REAL_CITIZEN_CAP,
            smoothed_frame_time: 0.016,
        }
    }
}

impl VirtualPopulation {
    pub fn total_with_real(&self, real_count: u32) -> u32 {
        real_count + self.total_virtual
    }

    /// Absorb a new virtual citizen into district statistics.
    pub fn add_virtual_citizen(
        &mut self,
        district_idx: usize,
        age: u8,
        employed: bool,
        happiness: f32,
        salary: f32,
        tax_rate: f32,
    ) {
        self.total_virtual += 1;
        if employed {
            self.virtual_employed += 1;
        }

        // Ensure district vec is large enough
        if district_idx >= self.district_stats.len() {
            self.district_stats
                .resize_with(district_idx + 1, DistrictStats::default);
        }

        let ds = &mut self.district_stats[district_idx];
        // Update running averages
        let n = ds.population as f32;
        ds.avg_happiness = (ds.avg_happiness * n + happiness) / (n + 1.0);
        ds.avg_age = (ds.avg_age * n + age as f32) / (n + 1.0);
        ds.population += 1;
        if employed {
            ds.employed += 1;
            ds.tax_contribution += salary * tax_rate;
            ds.commuters_out += 1;
        }

        // Age bracket
        let bracket = match age {
            0..=17 => 0,
            18..=34 => 1,
            35..=54 => 2,
            55..=64 => 3,
            _ => 4,
        };
        ds.age_brackets[bracket] += 1;

        // Service demand grows with population density
        ds.service_demand = (ds.population as f32 / 5000.0).min(1.0);
    }

    /// Adjust the real citizen cap based on measured frame time.
    /// Called once per second from the update system.
    pub fn adjust_cap(&mut self, frame_time_secs: f32) {
        // Exponential moving average
        self.smoothed_frame_time =
            self.smoothed_frame_time * 0.9 + frame_time_secs * 0.1;

        let fps = 1.0 / self.smoothed_frame_time;

        let new_cap = if fps > 55.0 {
            // Running smooth — raise cap by 10%
            (self.max_real_citizens as f32 * 1.1) as u32
        } else if fps < 25.0 {
            // Struggling — lower cap by 20%
            (self.max_real_citizens as f32 * 0.8) as u32
        } else {
            self.max_real_citizens
        };

        self.max_real_citizens = new_cap.clamp(MIN_REAL_CITIZENS, MAX_REAL_CITIZENS_HARD);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_virtual_citizen_updates_stats() {
        let mut vp = VirtualPopulation::default();
        vp.add_virtual_citizen(0, 25, true, 75.0, 1000.0, 0.1);
        assert_eq!(vp.total_virtual, 1);
        assert_eq!(vp.virtual_employed, 1);
        assert_eq!(vp.district_stats.len(), 1);
        assert_eq!(vp.district_stats[0].population, 1);
        assert_eq!(vp.district_stats[0].employed, 1);
        assert!((vp.district_stats[0].avg_happiness - 75.0).abs() < 0.01);
        assert!((vp.district_stats[0].avg_age - 25.0).abs() < 0.01);
        assert_eq!(vp.district_stats[0].age_brackets[1], 1); // 18-34 bracket
        assert!((vp.district_stats[0].tax_contribution - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_add_virtual_citizen_running_averages() {
        let mut vp = VirtualPopulation::default();
        vp.add_virtual_citizen(0, 20, false, 50.0, 0.0, 0.1);
        vp.add_virtual_citizen(0, 40, false, 100.0, 0.0, 0.1);
        assert_eq!(vp.total_virtual, 2);
        assert_eq!(vp.virtual_employed, 0);
        assert!((vp.district_stats[0].avg_happiness - 75.0).abs() < 0.01);
        assert!((vp.district_stats[0].avg_age - 30.0).abs() < 0.01);
    }

    #[test]
    fn test_age_brackets() {
        let mut vp = VirtualPopulation::default();
        vp.add_virtual_citizen(0, 10, false, 50.0, 0.0, 0.0); // child
        vp.add_virtual_citizen(0, 25, true, 50.0, 500.0, 0.1); // young adult
        vp.add_virtual_citizen(0, 45, true, 50.0, 800.0, 0.1); // middle-aged
        vp.add_virtual_citizen(0, 60, true, 50.0, 900.0, 0.1); // pre-retirement
        vp.add_virtual_citizen(0, 70, false, 50.0, 0.0, 0.0);  // retired
        assert_eq!(vp.district_stats[0].age_brackets, [1, 1, 1, 1, 1]);
    }

    #[test]
    fn test_auto_expands_districts() {
        let mut vp = VirtualPopulation::default();
        vp.add_virtual_citizen(5, 30, false, 50.0, 0.0, 0.0);
        assert_eq!(vp.district_stats.len(), 6); // 0..=5
        assert_eq!(vp.district_stats[5].population, 1);
        assert_eq!(vp.district_stats[0].population, 0);
    }

    #[test]
    fn test_adjust_cap_raises_on_high_fps() {
        let mut vp = VirtualPopulation::default();
        assert_eq!(vp.max_real_citizens, DEFAULT_REAL_CITIZEN_CAP);
        // Simulate sustained high FPS (frame time ~8ms = 125 FPS)
        for _ in 0..100 {
            vp.adjust_cap(0.008);
        }
        assert!(vp.max_real_citizens > DEFAULT_REAL_CITIZEN_CAP);
    }

    #[test]
    fn test_adjust_cap_lowers_on_low_fps() {
        let mut vp = VirtualPopulation::default();
        // Simulate sustained low FPS (frame time ~50ms = 20 FPS)
        for _ in 0..100 {
            vp.adjust_cap(0.05);
        }
        assert!(vp.max_real_citizens < DEFAULT_REAL_CITIZEN_CAP);
    }

    #[test]
    fn test_adjust_cap_respects_bounds() {
        let mut vp = VirtualPopulation::default();
        // Drive cap very high
        for _ in 0..500 {
            vp.adjust_cap(0.001); // 1000 FPS
        }
        assert!(vp.max_real_citizens <= MAX_REAL_CITIZENS_HARD);

        // Drive cap very low
        for _ in 0..500 {
            vp.adjust_cap(0.1); // 10 FPS
        }
        assert!(vp.max_real_citizens >= MIN_REAL_CITIZENS);
    }

    #[test]
    fn test_adjust_cap_stable_in_target_range() {
        let mut vp = VirtualPopulation {
            smoothed_frame_time: 0.025, // start with 40 FPS baseline
            ..Default::default()
        };
        let initial = vp.max_real_citizens;
        // FPS in 25-55 range should not change cap
        vp.adjust_cap(0.025); // 40 FPS
        assert_eq!(vp.max_real_citizens, initial);
    }

    #[test]
    fn test_total_with_real() {
        let mut vp = VirtualPopulation::default();
        vp.total_virtual = 100;
        assert_eq!(vp.total_with_real(50), 150);
    }
}

/// System: adjust the real citizen cap once per second based on FPS.
pub fn adjust_real_citizen_cap(
    time: Res<Time>,
    mut virtual_pop: ResMut<VirtualPopulation>,
) {
    let dt = time.delta_secs();
    if dt > 0.0 {
        virtual_pop.adjust_cap(dt);
    }
}
