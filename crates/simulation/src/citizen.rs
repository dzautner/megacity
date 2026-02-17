use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::roads::RoadNode;

// ---------------------------------------------------------------------------
// Identity
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Gender {
    Male,
    Female,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifeStage {
    Child,       // 0-5   (stays home)
    SchoolAge,   // 6-17  (goes to school)
    YoungAdult,  // 18-25 (first job, education)
    Adult,       // 26-54 (career, family)
    Senior,      // 55-64 (working but slowing)
    Retired,     // 65+   (no work)
}

impl LifeStage {
    pub fn from_age(age: u8) -> Self {
        match age {
            0..=5 => Self::Child,
            6..=17 => Self::SchoolAge,
            18..=25 => Self::YoungAdult,
            26..=54 => Self::Adult,
            55..=64 => Self::Senior,
            _ => Self::Retired,
        }
    }

    pub fn can_work(&self) -> bool {
        matches!(self, Self::YoungAdult | Self::Adult | Self::Senior)
    }

    pub fn should_attend_school(&self) -> bool {
        matches!(self, Self::SchoolAge)
    }
}

// ---------------------------------------------------------------------------
// Citizen state machine
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CitizenState {
    AtHome,
    CommutingToWork,
    Working,
    CommutingHome,
    CommutingToShop,
    Shopping,
    CommutingToLeisure,
    AtLeisure,
    CommutingToSchool,
    AtSchool,
}

impl CitizenState {
    pub fn is_commuting(self) -> bool {
        matches!(
            self,
            Self::CommutingToWork
                | Self::CommutingHome
                | Self::CommutingToShop
                | Self::CommutingToLeisure
                | Self::CommutingToSchool
        )
    }

    pub fn is_at_destination(self) -> bool {
        matches!(
            self,
            Self::AtHome | Self::Working | Self::Shopping | Self::AtLeisure | Self::AtSchool
        )
    }
}

// ---------------------------------------------------------------------------
// Core components
// ---------------------------------------------------------------------------

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct CitizenDetails {
    pub age: u8,
    pub gender: Gender,
    pub education: u8,  // 0=None, 1=Elementary, 2=HighSchool, 3=University
    pub happiness: f32, // 0.0-100.0
    pub health: f32,    // 0.0-100.0
    pub salary: f32,    // monthly income
    pub savings: f32,   // accumulated wealth
}

impl CitizenDetails {
    pub fn life_stage(&self) -> LifeStage {
        LifeStage::from_age(self.age)
    }

    /// Base salary for an education level (before job-match modifier).
    pub fn base_salary_for_education(education: u8) -> f32 {
        match education {
            0 => 1500.0,
            1 => 2200.0,
            2 => 3500.0,
            3 => 6000.0,
            _ => 8000.0,
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Component, Debug, Clone)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Component, Debug, Clone)]
pub struct HomeLocation {
    pub grid_x: usize,
    pub grid_y: usize,
    pub building: Entity,
}

#[derive(Component, Debug, Clone)]
pub struct WorkLocation {
    pub grid_x: usize,
    pub grid_y: usize,
    pub building: Entity,
}

#[derive(Component, Debug, Clone)]
pub struct CitizenStateComp(pub CitizenState);

// ---------------------------------------------------------------------------
// Path cache (unchanged)
// ---------------------------------------------------------------------------

#[derive(Component, Debug, Clone)]
pub struct PathCache {
    pub waypoints: Vec<RoadNode>,
    pub current_index: usize,
}

impl PathCache {
    pub fn new(waypoints: Vec<RoadNode>) -> Self {
        Self {
            waypoints,
            current_index: 0,
        }
    }

    pub fn current_target(&self) -> Option<&RoadNode> {
        self.waypoints.get(self.current_index)
    }

    pub fn advance(&mut self) -> bool {
        self.current_index += 1;
        self.current_index < self.waypoints.len()
    }

    pub fn is_complete(&self) -> bool {
        self.current_index >= self.waypoints.len()
    }

    /// Peek at the waypoint after the current one (for path smoothing).
    pub fn peek_next(&self) -> Option<&RoadNode> {
        self.waypoints.get(self.current_index + 1)
    }

    pub fn reset(&mut self) {
        self.current_index = 0;
    }
}

/// A queued pathfinding request. The batch system will compute the A* path
/// and write the result into `PathCache`, then remove this component.
#[derive(Component, Debug, Clone)]
pub struct PathRequest {
    pub from_gx: usize,
    pub from_gy: usize,
    pub to_gx: usize,
    pub to_gy: usize,
    /// The state to transition to once the path is ready.
    pub target_state: CitizenState,
}

// ---------------------------------------------------------------------------
// Personality (permanent traits, set at birth)
// ---------------------------------------------------------------------------

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Personality {
    pub ambition: f32,       // 0.0-1.0: career/education drive
    pub sociability: f32,    // 0.0-1.0: social need weight, family size
    pub materialism: f32,    // 0.0-1.0: money/housing quality importance
    pub resilience: f32,     // 0.0-1.0: stress resistance, handles bad conditions
}

impl Personality {
    pub fn random(rng: &mut impl rand::Rng) -> Self {
        Self {
            ambition: rng.gen_range(0.1..=1.0),
            sociability: rng.gen_range(0.1..=1.0),
            materialism: rng.gen_range(0.1..=1.0),
            resilience: rng.gen_range(0.1..=1.0),
        }
    }
}

// ---------------------------------------------------------------------------
// Needs (Sims-inspired, fluctuate over time)
// 100 = fully satisfied, 0 = critical
// ---------------------------------------------------------------------------

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Needs {
    pub hunger: f32,   // decays while awake, restored by eating at home / shopping
    pub energy: f32,   // decays while awake, restored by sleeping at home
    pub social: f32,   // decays when alone, restored by leisure / working
    pub fun: f32,      // decays during work, restored by leisure / entertainment
    pub comfort: f32,  // based on housing quality + utilities
}

impl Default for Needs {
    fn default() -> Self {
        Self {
            hunger: 80.0,
            energy: 80.0,
            social: 70.0,
            fun: 70.0,
            comfort: 60.0,
        }
    }
}

impl Needs {
    /// Overall satisfaction (0.0-1.0) -- weighted average of all needs.
    pub fn overall_satisfaction(&self) -> f32 {
        let raw = self.hunger * 0.25
            + self.energy * 0.25
            + self.social * 0.15
            + self.fun * 0.15
            + self.comfort * 0.20;
        (raw / 100.0).clamp(0.0, 1.0)
    }

    /// Returns the most critical need (lowest value).
    pub fn most_critical(&self) -> (&'static str, f32) {
        let mut worst = ("hunger", self.hunger);
        if self.energy < worst.1 {
            worst = ("energy", self.energy);
        }
        if self.social < worst.1 {
            worst = ("social", self.social);
        }
        if self.fun < worst.1 {
            worst = ("fun", self.fun);
        }
        if self.comfort < worst.1 {
            worst = ("comfort", self.comfort);
        }
        worst
    }
}

// ---------------------------------------------------------------------------
// Family relationships
// ---------------------------------------------------------------------------

#[derive(Component, Debug, Clone, Default)]
pub struct Family {
    pub partner: Option<Entity>,
    pub children: Vec<Entity>,
    pub parent: Option<Entity>, // if this citizen is a child
}

/// Bundle-like marker for citizens
#[derive(Component)]
pub struct Citizen;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_transitions() {
        let states = [
            CitizenState::AtHome,
            CitizenState::CommutingToWork,
            CitizenState::Working,
            CitizenState::CommutingHome,
            CitizenState::AtHome,
        ];
        for w in states.windows(2) {
            assert_ne!(w[0], w[1]);
        }
        assert_eq!(states[0], states[4]);
    }

    #[test]
    fn test_extended_states() {
        assert!(CitizenState::CommutingToShop.is_commuting());
        assert!(CitizenState::CommutingToLeisure.is_commuting());
        assert!(CitizenState::CommutingToSchool.is_commuting());
        assert!(!CitizenState::Shopping.is_commuting());
        assert!(!CitizenState::AtLeisure.is_commuting());

        assert!(CitizenState::Shopping.is_at_destination());
        assert!(CitizenState::AtLeisure.is_at_destination());
        assert!(CitizenState::AtSchool.is_at_destination());
    }

    #[test]
    fn test_life_stages() {
        assert_eq!(LifeStage::from_age(3), LifeStage::Child);
        assert_eq!(LifeStage::from_age(10), LifeStage::SchoolAge);
        assert_eq!(LifeStage::from_age(20), LifeStage::YoungAdult);
        assert_eq!(LifeStage::from_age(40), LifeStage::Adult);
        assert_eq!(LifeStage::from_age(60), LifeStage::Senior);
        assert_eq!(LifeStage::from_age(70), LifeStage::Retired);

        assert!(LifeStage::Adult.can_work());
        assert!(!LifeStage::Child.can_work());
        assert!(!LifeStage::Retired.can_work());
        assert!(LifeStage::SchoolAge.should_attend_school());
    }

    #[test]
    fn test_path_cache() {
        let mut path = PathCache::new(vec![
            RoadNode(0, 0),
            RoadNode(1, 0),
            RoadNode(2, 0),
        ]);
        assert_eq!(path.current_target(), Some(&RoadNode(0, 0)));
        assert!(!path.is_complete());

        path.advance();
        assert_eq!(path.current_target(), Some(&RoadNode(1, 0)));

        path.advance();
        assert_eq!(path.current_target(), Some(&RoadNode(2, 0)));

        path.advance();
        assert!(path.is_complete());
    }

    #[test]
    fn test_needs_satisfaction() {
        let needs = Needs {
            hunger: 100.0,
            energy: 100.0,
            social: 100.0,
            fun: 100.0,
            comfort: 100.0,
        };
        assert!((needs.overall_satisfaction() - 1.0).abs() < 0.01);

        let critical = Needs {
            hunger: 10.0,
            energy: 50.0,
            social: 80.0,
            fun: 60.0,
            comfort: 70.0,
        };
        assert_eq!(critical.most_critical().0, "hunger");
    }

    #[test]
    fn test_personality_random() {
        let mut rng = rand::thread_rng();
        let p = Personality::random(&mut rng);
        assert!(p.ambition >= 0.1 && p.ambition <= 1.0);
        assert!(p.sociability >= 0.1 && p.sociability <= 1.0);
    }

    #[test]
    fn test_salary_for_education() {
        assert!(CitizenDetails::base_salary_for_education(3) > CitizenDetails::base_salary_for_education(0));
    }
}
