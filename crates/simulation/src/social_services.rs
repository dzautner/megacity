//! SVC-013: Social Services Building Types
//!
//! Adds four social service building types:
//! - CommunityCenter: +5 happiness, social need boost for all citizens in radius
//! - SubstanceAbuseTreatmentCenter: health & happiness boost in radius
//! - SeniorCenter: +10 happiness for retired citizens in radius
//! - YouthCenter: -15% juvenile crime in radius

use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::citizen::{Citizen, CitizenDetails, HomeLocation, LifeStage};
use crate::config::{CELL_SIZE, GRID_HEIGHT, GRID_WIDTH};
use crate::crime::CrimeGrid;
use crate::services::{ServiceBuilding, ServiceType};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Happiness bonus for citizens within CommunityCenter coverage.
pub const COMMUNITY_CENTER_HAPPINESS_BONUS: f32 = 5.0;

/// Happiness bonus for citizens within SubstanceAbuseTreatmentCenter coverage.
pub const SUBSTANCE_TREATMENT_HAPPINESS_BONUS: f32 = 3.0;

/// Health bonus per slow tick for citizens within SubstanceAbuseTreatmentCenter coverage.
pub const SUBSTANCE_TREATMENT_HEALTH_BONUS: f32 = 1.5;

/// Happiness bonus for retired citizens within SeniorCenter coverage.
pub const SENIOR_CENTER_HAPPINESS_BONUS: f32 = 10.0;

/// Fraction by which YouthCenter reduces crime in its radius (0.15 = 15%).
pub const YOUTH_CENTER_CRIME_REDUCTION: f32 = 0.15;

/// Happiness bonus for youth (SchoolAge/YoungAdult) within YouthCenter coverage.
pub const YOUTH_CENTER_HAPPINESS_BONUS: f32 = 4.0;

// ---------------------------------------------------------------------------
// Coverage Grid
// ---------------------------------------------------------------------------

/// Per-cell coverage grid for social service buildings.
/// Bit 0 = CommunityCenter, Bit 1 = SubstanceAbuseTreatmentCenter,
/// Bit 2 = SeniorCenter, Bit 3 = YouthCenter
#[derive(Resource)]
pub struct SocialServicesCoverage {
    pub flags: Vec<u8>,
    pub dirty: bool,
}

const COVERAGE_COMMUNITY: u8 = 0b0001;
const COVERAGE_SUBSTANCE: u8 = 0b0010;
const COVERAGE_SENIOR: u8 = 0b0100;
const COVERAGE_YOUTH: u8 = 0b1000;

impl Default for SocialServicesCoverage {
    fn default() -> Self {
        Self {
            flags: vec![0; GRID_WIDTH * GRID_HEIGHT],
            dirty: true,
        }
    }
}

impl SocialServicesCoverage {
    #[inline]
    pub fn idx(x: usize, y: usize) -> usize {
        y * GRID_WIDTH + x
    }

    pub fn clear(&mut self) {
        self.flags.fill(0);
    }

    #[inline]
    pub fn has_community_center(&self, x: usize, y: usize) -> bool {
        self.flags[Self::idx(x, y)] & COVERAGE_COMMUNITY != 0
    }

    #[inline]
    pub fn has_substance_treatment(&self, x: usize, y: usize) -> bool {
        self.flags[Self::idx(x, y)] & COVERAGE_SUBSTANCE != 0
    }

    #[inline]
    pub fn has_senior_center(&self, x: usize, y: usize) -> bool {
        self.flags[Self::idx(x, y)] & COVERAGE_SENIOR != 0
    }

    #[inline]
    pub fn has_youth_center(&self, x: usize, y: usize) -> bool {
        self.flags[Self::idx(x, y)] & COVERAGE_YOUTH != 0
    }

    /// Count of cells with community center coverage.
    pub fn community_covered_cells(&self) -> usize {
        self.flags
            .iter()
            .filter(|&&f| f & COVERAGE_COMMUNITY != 0)
            .count()
    }

    /// Count of cells with youth center coverage.
    pub fn youth_covered_cells(&self) -> usize {
        self.flags
            .iter()
            .filter(|&&f| f & COVERAGE_YOUTH != 0)
            .count()
    }
}

// ---------------------------------------------------------------------------
// Saveable State
// ---------------------------------------------------------------------------

#[derive(Resource, Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]
pub struct SocialServicesState {
    pub community_center_count: u32,
    pub substance_treatment_count: u32,
    pub senior_center_count: u32,
    pub youth_center_count: u32,
    pub citizens_covered_community: u32,
    pub citizens_covered_senior: u32,
    pub citizens_covered_youth: u32,
    pub monthly_maintenance: f64,
}

impl crate::Saveable for SocialServicesState {
    const SAVE_KEY: &'static str = "social_services";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        Some(bitcode::encode(self))
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        crate::decode_or_warn(Self::SAVE_KEY, bytes)
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Recompute social services coverage grid when service buildings change.
pub fn update_social_services_coverage(
    services: Query<&ServiceBuilding>,
    added_services: Query<Entity, Added<ServiceBuilding>>,
    mut coverage: ResMut<SocialServicesCoverage>,
) {
    if !added_services.is_empty() {
        coverage.dirty = true;
    }
    if !coverage.dirty {
        return;
    }
    coverage.dirty = false;
    coverage.clear();

    for service in &services {
        let bits = match service.service_type {
            ServiceType::CommunityCenter => COVERAGE_COMMUNITY,
            ServiceType::SubstanceAbuseTreatmentCenter => COVERAGE_SUBSTANCE,
            ServiceType::SeniorCenter => COVERAGE_SENIOR,
            ServiceType::YouthCenter => COVERAGE_YOUTH,
            _ => continue,
        };

        let radius_cells = (service.radius / CELL_SIZE).ceil() as i32;
        let sx = service.grid_x as i32;
        let sy = service.grid_y as i32;
        let r2 = service.radius * service.radius;

        for dy in -radius_cells..=radius_cells {
            for dx in -radius_cells..=radius_cells {
                let cx = sx + dx;
                let cy = sy + dy;
                if cx < 0 || cy < 0 || cx >= GRID_WIDTH as i32 || cy >= GRID_HEIGHT as i32 {
                    continue;
                }
                let wx_diff = dx as f32 * CELL_SIZE;
                let wy_diff = dy as f32 * CELL_SIZE;
                if wx_diff * wx_diff + wy_diff * wy_diff > r2 {
                    continue;
                }
                let idx = SocialServicesCoverage::idx(cx as usize, cy as usize);
                coverage.flags[idx] |= bits;
            }
        }
    }
}

/// Apply social services effects to citizens and update stats.
/// - CommunityCenter: +5 happiness for all citizens in radius
/// - SubstanceAbuseTreatmentCenter: +1.5 health, +3 happiness in radius
/// - SeniorCenter: +10 happiness for retired citizens in radius
/// - YouthCenter: +4 happiness for youth citizens in radius
#[allow(clippy::too_many_arguments)]
pub fn apply_social_services_effects(
    slow_timer: Res<crate::SlowTickTimer>,
    coverage: Res<SocialServicesCoverage>,
    services: Query<&ServiceBuilding>,
    mut state: ResMut<SocialServicesState>,
    mut citizens: Query<(&mut CitizenDetails, &HomeLocation), With<Citizen>>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // Count service buildings
    let mut community_count: u32 = 0;
    let mut substance_count: u32 = 0;
    let mut senior_count: u32 = 0;
    let mut youth_count: u32 = 0;
    let mut total_maintenance: f64 = 0.0;

    for service in &services {
        match service.service_type {
            ServiceType::CommunityCenter => {
                community_count += 1;
                total_maintenance +=
                    ServiceBuilding::monthly_maintenance(ServiceType::CommunityCenter);
            }
            ServiceType::SubstanceAbuseTreatmentCenter => {
                substance_count += 1;
                total_maintenance += ServiceBuilding::monthly_maintenance(
                    ServiceType::SubstanceAbuseTreatmentCenter,
                );
            }
            ServiceType::SeniorCenter => {
                senior_count += 1;
                total_maintenance +=
                    ServiceBuilding::monthly_maintenance(ServiceType::SeniorCenter);
            }
            ServiceType::YouthCenter => {
                youth_count += 1;
                total_maintenance +=
                    ServiceBuilding::monthly_maintenance(ServiceType::YouthCenter);
            }
            _ => {}
        }
    }

    state.community_center_count = community_count;
    state.substance_treatment_count = substance_count;
    state.senior_center_count = senior_count;
    state.youth_center_count = youth_count;
    state.monthly_maintenance = total_maintenance;

    // Apply effects to citizens
    let mut covered_community: u32 = 0;
    let mut covered_senior: u32 = 0;
    let mut covered_youth: u32 = 0;

    for (mut details, home) in &mut citizens {
        let hx = home.grid_x;
        let hy = home.grid_y;
        let life_stage = details.life_stage();

        // CommunityCenter: +5 happiness for everyone
        if coverage.has_community_center(hx, hy) {
            covered_community += 1;
            details.happiness =
                (details.happiness + COMMUNITY_CENTER_HAPPINESS_BONUS).min(100.0);
        }

        // SubstanceAbuseTreatmentCenter: +3 happiness, +1.5 health
        if coverage.has_substance_treatment(hx, hy) {
            details.happiness =
                (details.happiness + SUBSTANCE_TREATMENT_HAPPINESS_BONUS).min(100.0);
            details.health =
                (details.health + SUBSTANCE_TREATMENT_HEALTH_BONUS).min(100.0);
        }

        // SeniorCenter: +10 happiness for retired citizens only
        if coverage.has_senior_center(hx, hy)
            && life_stage == LifeStage::Retired
        {
            covered_senior += 1;
            details.happiness =
                (details.happiness + SENIOR_CENTER_HAPPINESS_BONUS).min(100.0);
        }

        // YouthCenter: +4 happiness for school-age and young adults
        if coverage.has_youth_center(hx, hy)
            && matches!(
                life_stage,
                LifeStage::SchoolAge | LifeStage::YoungAdult
            )
        {
            covered_youth += 1;
            details.happiness =
                (details.happiness + YOUTH_CENTER_HAPPINESS_BONUS).min(100.0);
        }
    }

    state.citizens_covered_community = covered_community;
    state.citizens_covered_senior = covered_senior;
    state.citizens_covered_youth = covered_youth;
}

/// Reduce crime in cells covered by YouthCenter buildings.
/// Applies a 15% reduction to crime levels within the coverage radius.
pub fn apply_youth_center_crime_reduction(
    slow_timer: Res<crate::SlowTickTimer>,
    coverage: Res<SocialServicesCoverage>,
    mut crime: ResMut<CrimeGrid>,
) {
    if !slow_timer.should_run() {
        return;
    }

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            if coverage.has_youth_center(x, y) {
                let current = crime.get(x, y);
                let reduction = (current as f32 * YOUTH_CENTER_CRIME_REDUCTION) as u8;
                crime.set(x, y, current.saturating_sub(reduction));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

pub struct SocialServicesPlugin;

impl Plugin for SocialServicesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SocialServicesCoverage>()
            .init_resource::<SocialServicesState>();

        // Register for save/load via the SaveableRegistry.
        app.init_resource::<crate::SaveableRegistry>();
        app.world_mut()
            .resource_mut::<crate::SaveableRegistry>()
            .register::<SocialServicesState>();

        app.add_systems(
            FixedUpdate,
            (
                update_social_services_coverage,
                apply_social_services_effects
                    .after(update_social_services_coverage),
                apply_youth_center_crime_reduction
                    .after(update_social_services_coverage),
            )
                .in_set(crate::SimulationSet::Simulation),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coverage_grid_default_is_clear() {
        let cov = SocialServicesCoverage::default();
        assert_eq!(cov.flags.len(), GRID_WIDTH * GRID_HEIGHT);
        assert!(cov.flags.iter().all(|&f| f == 0));
    }

    #[test]
    fn test_coverage_community_center() {
        let mut cov = SocialServicesCoverage::default();
        assert!(!cov.has_community_center(10, 10));
        let idx = SocialServicesCoverage::idx(10, 10);
        cov.flags[idx] |= COVERAGE_COMMUNITY;
        assert!(cov.has_community_center(10, 10));
        assert!(!cov.has_senior_center(10, 10));
    }

    #[test]
    fn test_coverage_senior_center() {
        let mut cov = SocialServicesCoverage::default();
        let idx = SocialServicesCoverage::idx(20, 20);
        cov.flags[idx] |= COVERAGE_SENIOR;
        assert!(cov.has_senior_center(20, 20));
        assert!(!cov.has_youth_center(20, 20));
    }

    #[test]
    fn test_coverage_youth_center() {
        let mut cov = SocialServicesCoverage::default();
        let idx = SocialServicesCoverage::idx(30, 30);
        cov.flags[idx] |= COVERAGE_YOUTH;
        assert!(cov.has_youth_center(30, 30));
        assert!(!cov.has_community_center(30, 30));
    }

    #[test]
    fn test_coverage_substance_treatment() {
        let mut cov = SocialServicesCoverage::default();
        let idx = SocialServicesCoverage::idx(15, 15);
        cov.flags[idx] |= COVERAGE_SUBSTANCE;
        assert!(cov.has_substance_treatment(15, 15));
    }

    #[test]
    fn test_coverage_clear() {
        let mut cov = SocialServicesCoverage::default();
        let idx = SocialServicesCoverage::idx(5, 5);
        cov.flags[idx] = COVERAGE_COMMUNITY | COVERAGE_YOUTH;
        cov.clear();
        assert!(!cov.has_community_center(5, 5));
        assert!(!cov.has_youth_center(5, 5));
    }

    #[test]
    fn test_state_default() {
        let state = SocialServicesState::default();
        assert_eq!(state.community_center_count, 0);
        assert_eq!(state.substance_treatment_count, 0);
        assert_eq!(state.senior_center_count, 0);
        assert_eq!(state.youth_center_count, 0);
        assert_eq!(state.monthly_maintenance, 0.0);
    }

    #[test]
    fn test_saveable_roundtrip() {
        use crate::Saveable;
        let mut state = SocialServicesState::default();
        state.community_center_count = 3;
        state.substance_treatment_count = 1;
        state.senior_center_count = 2;
        state.youth_center_count = 4;
        state.citizens_covered_community = 200;
        state.citizens_covered_senior = 50;
        state.citizens_covered_youth = 80;
        state.monthly_maintenance = 250.0;
        let bytes = state.save_to_bytes().expect("should serialize");
        let restored = SocialServicesState::load_from_bytes(&bytes);
        assert_eq!(restored.community_center_count, 3);
        assert_eq!(restored.substance_treatment_count, 1);
        assert_eq!(restored.senior_center_count, 2);
        assert_eq!(restored.youth_center_count, 4);
        assert_eq!(restored.citizens_covered_community, 200);
        assert_eq!(restored.citizens_covered_senior, 50);
        assert_eq!(restored.citizens_covered_youth, 80);
        assert!((restored.monthly_maintenance - 250.0).abs() < 0.001);
    }

    #[test]
    fn test_service_type_names() {
        assert_eq!(ServiceType::CommunityCenter.name(), "Community Center");
        assert_eq!(
            ServiceType::SubstanceAbuseTreatmentCenter.name(),
            "Substance Abuse Treatment Center"
        );
        assert_eq!(ServiceType::SeniorCenter.name(), "Senior Center");
        assert_eq!(ServiceType::YouthCenter.name(), "Youth Center");
    }

    #[test]
    fn test_coverage_radius_positive() {
        assert!(ServiceBuilding::coverage_radius(ServiceType::CommunityCenter) > 0.0);
        assert!(
            ServiceBuilding::coverage_radius(ServiceType::SubstanceAbuseTreatmentCenter) > 0.0
        );
        assert!(ServiceBuilding::coverage_radius(ServiceType::SeniorCenter) > 0.0);
        assert!(ServiceBuilding::coverage_radius(ServiceType::YouthCenter) > 0.0);
    }

    #[test]
    fn test_building_costs_positive() {
        assert!(ServiceBuilding::cost(ServiceType::CommunityCenter) > 0.0);
        assert!(ServiceBuilding::cost(ServiceType::SubstanceAbuseTreatmentCenter) > 0.0);
        assert!(ServiceBuilding::cost(ServiceType::SeniorCenter) > 0.0);
        assert!(ServiceBuilding::cost(ServiceType::YouthCenter) > 0.0);
    }

    #[test]
    fn test_monthly_maintenance_positive() {
        assert!(ServiceBuilding::monthly_maintenance(ServiceType::CommunityCenter) > 0.0);
        assert!(
            ServiceBuilding::monthly_maintenance(ServiceType::SubstanceAbuseTreatmentCenter)
                > 0.0
        );
        assert!(ServiceBuilding::monthly_maintenance(ServiceType::SeniorCenter) > 0.0);
        assert!(ServiceBuilding::monthly_maintenance(ServiceType::YouthCenter) > 0.0);
    }

    #[test]
    fn test_covered_cells_count() {
        let mut cov = SocialServicesCoverage::default();
        assert_eq!(cov.community_covered_cells(), 0);
        assert_eq!(cov.youth_covered_cells(), 0);

        let idx1 = SocialServicesCoverage::idx(1, 1);
        let idx2 = SocialServicesCoverage::idx(2, 2);
        cov.flags[idx1] = COVERAGE_COMMUNITY;
        cov.flags[idx2] = COVERAGE_COMMUNITY | COVERAGE_YOUTH;
        assert_eq!(cov.community_covered_cells(), 2);
        assert_eq!(cov.youth_covered_cells(), 1);
    }
}
