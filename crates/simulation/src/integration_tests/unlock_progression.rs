use crate::services::ServiceType;
use crate::test_harness::TestCity;
use crate::utilities::UtilityType;

// ====================================================================
// TEST-064: Unlock / Progression System Tests (MILE-001 12-tier overhaul)
// ====================================================================

#[test]
fn test_unlock_state_default_has_starter_unlocks() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let state = UnlockState::default();
    assert!(state.is_unlocked(UnlockNode::BasicRoads));
    assert!(state.is_unlocked(UnlockNode::ResidentialZoning));
    assert!(state.is_unlocked(UnlockNode::CommercialZoning));
    assert!(state.is_unlocked(UnlockNode::IndustrialZoning));
    assert!(state.is_unlocked(UnlockNode::BasicPower));
    assert!(state.is_unlocked(UnlockNode::BasicWater));
    assert!(!state.is_unlocked(UnlockNode::FireService));
    assert!(!state.is_unlocked(UnlockNode::PoliceService));
}

#[test]
fn test_unlock_state_default_has_three_development_points() {
    use crate::unlocks::UnlockState;
    let state = UnlockState::default();
    assert_eq!(state.development_points, 3);
    assert_eq!(state.spent_points, 0);
    assert_eq!(state.available_points(), 3);
}

#[test]
fn test_unlock_node_cost_tiers() {
    use crate::unlocks::UnlockNode;
    // Tier 0 — free starters
    assert_eq!(UnlockNode::BasicRoads.cost(), 0);
    assert_eq!(UnlockNode::BasicWater.cost(), 0);
    // Tier 1 — 240 pop
    assert_eq!(UnlockNode::HealthCare.cost(), 1);
    assert_eq!(UnlockNode::DeathCare.cost(), 1);
    assert_eq!(UnlockNode::BasicSanitation.cost(), 1);
    // Tier 2 — 1,200 pop
    assert_eq!(UnlockNode::FireService.cost(), 1);
    assert_eq!(UnlockNode::PoliceService.cost(), 1);
    assert_eq!(UnlockNode::ElementaryEducation.cost(), 1);
    // Tier 3 — 2,600 pop
    assert_eq!(UnlockNode::HighSchoolEducation.cost(), 2);
    assert_eq!(UnlockNode::SmallParks.cost(), 2);
    assert_eq!(UnlockNode::PolicySystem.cost(), 2);
    // Tier 5 — 7,500 pop
    assert_eq!(UnlockNode::HighDensityResidential.cost(), 3);
    assert_eq!(UnlockNode::OfficeZoning.cost(), 3);
    // Tier 6 — 12,000 pop
    assert_eq!(UnlockNode::UniversityEducation.cost(), 3);
    // Tier 8 — 36,000 pop
    assert_eq!(UnlockNode::Telecom.cost(), 4);
    // Tier 9 — 50,000 pop
    assert_eq!(UnlockNode::SolarPower.cost(), 4);
    assert_eq!(UnlockNode::WindPower.cost(), 4);
    // Tier 10 — 65,000 pop
    assert_eq!(UnlockNode::NuclearPower.cost(), 5);
    // Tier 11 — 80,000 pop
    assert_eq!(UnlockNode::InternationalAirports.cost(), 7);
}

#[test]
fn test_unlock_node_required_population_tiers() {
    use crate::unlocks::UnlockNode;
    // 12-tier milestone thresholds
    assert_eq!(UnlockNode::BasicRoads.required_population(), 0);
    assert_eq!(UnlockNode::HealthCare.required_population(), 240);
    assert_eq!(UnlockNode::DeathCare.required_population(), 240);
    assert_eq!(UnlockNode::BasicSanitation.required_population(), 240);
    assert_eq!(UnlockNode::FireService.required_population(), 1_200);
    assert_eq!(UnlockNode::PoliceService.required_population(), 1_200);
    assert_eq!(UnlockNode::ElementaryEducation.required_population(), 1_200);
    assert_eq!(UnlockNode::HighSchoolEducation.required_population(), 2_600);
    assert_eq!(UnlockNode::SmallParks.required_population(), 2_600);
    assert_eq!(UnlockNode::PolicySystem.required_population(), 2_600);
    assert_eq!(UnlockNode::PublicTransport.required_population(), 5_000);
    assert_eq!(UnlockNode::Landmarks.required_population(), 5_000);
    assert_eq!(
        UnlockNode::HighDensityResidential.required_population(),
        7_500
    );
    assert_eq!(UnlockNode::OfficeZoning.required_population(), 7_500);
    assert_eq!(UnlockNode::AdvancedTransport.required_population(), 7_500);
    assert_eq!(
        UnlockNode::UniversityEducation.required_population(),
        12_000
    );
    assert_eq!(UnlockNode::SmallAirstrips.required_population(), 20_000);
    assert_eq!(UnlockNode::Telecom.required_population(), 36_000);
    assert_eq!(UnlockNode::RegionalAirports.required_population(), 50_000);
    assert_eq!(UnlockNode::AdvancedEmergency.required_population(), 65_000);
    assert_eq!(UnlockNode::NuclearPower.required_population(), 65_000);
    assert_eq!(
        UnlockNode::InternationalAirports.required_population(),
        80_000
    );
}

#[test]
fn test_unlock_purchase_deducts_points() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let mut state = UnlockState::default();
    assert_eq!(state.available_points(), 3);
    assert!(state.purchase(UnlockNode::HealthCare)); // cost 1
    assert_eq!(state.available_points(), 2);
    assert!(state.is_unlocked(UnlockNode::HealthCare));
    assert!(state.purchase(UnlockNode::DeathCare)); // cost 1
    assert_eq!(state.available_points(), 1);
    assert!(state.purchase(UnlockNode::BasicSanitation)); // cost 1
    assert_eq!(state.available_points(), 0);
}

#[test]
fn test_unlock_purchase_fails_insufficient_points() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let mut state = UnlockState::default();
    // Start with 3 DP, buy two tier-1 unlocks (cost 1 each) = 1 DP left
    assert!(state.purchase(UnlockNode::HealthCare));
    assert!(state.purchase(UnlockNode::DeathCare));
    assert_eq!(state.available_points(), 1);
    // HighSchoolEducation costs 2, should fail
    assert!(!state.purchase(UnlockNode::HighSchoolEducation));
    assert!(!state.is_unlocked(UnlockNode::HighSchoolEducation));
    assert_eq!(state.available_points(), 1);
}

#[test]
fn test_unlock_purchase_fails_for_already_unlocked() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let mut state = UnlockState::default();
    assert!(!state.purchase(UnlockNode::BasicRoads));
    assert_eq!(state.available_points(), 3);
}

#[test]
fn test_unlock_can_purchase_checks_population_threshold() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let mut state = UnlockState::default();
    state.development_points = 10;
    // HealthCare requires 240 pop
    assert!(!state.can_purchase(UnlockNode::HealthCare, 239));
    assert!(state.can_purchase(UnlockNode::HealthCare, 240));
    assert!(state.can_purchase(UnlockNode::HealthCare, 1000));
    // FireService requires 1,200 pop
    assert!(!state.can_purchase(UnlockNode::FireService, 1_199));
    assert!(state.can_purchase(UnlockNode::FireService, 1_200));
}

#[test]
fn test_unlock_can_purchase_false_when_already_unlocked() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let state = UnlockState::default();
    assert!(!state.can_purchase(UnlockNode::BasicRoads, 0));
}

#[test]
fn test_unlock_can_purchase_false_when_insufficient_points() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let mut state = UnlockState::default();
    state.spent_points = 3;
    assert_eq!(state.available_points(), 0);
    assert!(!state.can_purchase(UnlockNode::HealthCare, 1000));
}

#[test]
fn test_unlock_all_nodes_have_names() {
    use crate::unlocks::UnlockNode;
    for &node in UnlockNode::all() {
        assert!(!node.name().is_empty(), "Node {:?} has no name", node);
    }
}

#[test]
fn test_unlock_all_returns_all_variants() {
    use crate::unlocks::UnlockNode;
    let all = UnlockNode::all();
    assert!(all.contains(&UnlockNode::BasicRoads));
    assert!(all.contains(&UnlockNode::FireService));
    assert!(all.contains(&UnlockNode::HealthCare));
    assert!(all.contains(&UnlockNode::OfficeZoning));
    assert!(all.contains(&UnlockNode::Landmarks));
    assert!(all.contains(&UnlockNode::InternationalAirports));
}

#[test]
fn test_unlock_service_mapping_fire_service() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let mut state = UnlockState::default();
    assert!(!state.is_service_unlocked(ServiceType::FireStation));
    assert!(!state.is_service_unlocked(ServiceType::FireHouse));
    state.development_points = 10;
    state.purchase(UnlockNode::FireService);
    assert!(state.is_service_unlocked(ServiceType::FireStation));
    assert!(state.is_service_unlocked(ServiceType::FireHouse));
    assert!(!state.is_service_unlocked(ServiceType::FireHQ));
}

#[test]
fn test_unlock_service_mapping_police_service() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let mut state = UnlockState::default();
    assert!(!state.is_service_unlocked(ServiceType::PoliceStation));
    state.development_points = 10;
    state.purchase(UnlockNode::PoliceService);
    assert!(state.is_service_unlocked(ServiceType::PoliceStation));
    assert!(state.is_service_unlocked(ServiceType::PoliceKiosk));
    assert!(!state.is_service_unlocked(ServiceType::PoliceHQ));
    assert!(!state.is_service_unlocked(ServiceType::Prison));
}

#[test]
fn test_unlock_service_mapping_advanced_emergency() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let mut state = UnlockState::default();
    state.development_points = 50;
    state.purchase(UnlockNode::AdvancedEmergency);
    assert!(state.is_service_unlocked(ServiceType::FireHQ));
    assert!(state.is_service_unlocked(ServiceType::PoliceHQ));
    assert!(state.is_service_unlocked(ServiceType::Prison));
    assert!(state.is_service_unlocked(ServiceType::MedicalCenter));
}

#[test]
fn test_unlock_service_mapping_education() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let mut state = UnlockState::default();
    state.development_points = 50;
    state.purchase(UnlockNode::ElementaryEducation);
    assert!(state.is_service_unlocked(ServiceType::ElementarySchool));
    assert!(state.is_service_unlocked(ServiceType::Library));
    assert!(state.is_service_unlocked(ServiceType::Kindergarten));
    assert!(!state.is_service_unlocked(ServiceType::HighSchool));
    state.purchase(UnlockNode::HighSchoolEducation);
    assert!(state.is_service_unlocked(ServiceType::HighSchool));
    assert!(!state.is_service_unlocked(ServiceType::University));
    state.purchase(UnlockNode::UniversityEducation);
    assert!(state.is_service_unlocked(ServiceType::University));
}

#[test]
fn test_unlock_service_mapping_sanitation() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let mut state = UnlockState::default();
    state.development_points = 50;
    state.purchase(UnlockNode::BasicSanitation);
    assert!(state.is_service_unlocked(ServiceType::Landfill));
    assert!(state.is_service_unlocked(ServiceType::TransferStation));
    assert!(!state.is_service_unlocked(ServiceType::RecyclingCenter));
    state.purchase(UnlockNode::AdvancedSanitation);
    assert!(state.is_service_unlocked(ServiceType::RecyclingCenter));
    assert!(state.is_service_unlocked(ServiceType::Incinerator));
}

#[test]
fn test_unlock_service_mapping_transport() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let mut state = UnlockState::default();
    state.development_points = 50;
    state.purchase(UnlockNode::PublicTransport);
    assert!(state.is_service_unlocked(ServiceType::BusDepot));
    assert!(state.is_service_unlocked(ServiceType::TrainStation));
    assert!(!state.is_service_unlocked(ServiceType::SubwayStation));
    state.purchase(UnlockNode::AdvancedTransport);
    assert!(state.is_service_unlocked(ServiceType::SubwayStation));
    assert!(state.is_service_unlocked(ServiceType::TramDepot));
    assert!(state.is_service_unlocked(ServiceType::FerryPier));
}

#[test]
fn test_unlock_service_mapping_airports() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let mut state = UnlockState::default();
    state.development_points = 50;
    assert!(!state.is_service_unlocked(ServiceType::SmallAirstrip));
    state.purchase(UnlockNode::SmallAirstrips);
    assert!(state.is_service_unlocked(ServiceType::SmallAirstrip));
    assert!(!state.is_service_unlocked(ServiceType::RegionalAirport));
    state.purchase(UnlockNode::RegionalAirports);
    assert!(state.is_service_unlocked(ServiceType::RegionalAirport));
    assert!(!state.is_service_unlocked(ServiceType::InternationalAirport));
    state.purchase(UnlockNode::InternationalAirports);
    assert!(state.is_service_unlocked(ServiceType::InternationalAirport));
}

#[test]
fn test_unlock_service_mapping_telecom() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let mut state = UnlockState::default();
    state.development_points = 50;
    assert!(!state.is_service_unlocked(ServiceType::CellTower));
    assert!(!state.is_service_unlocked(ServiceType::DataCenter));
    state.purchase(UnlockNode::Telecom);
    assert!(state.is_service_unlocked(ServiceType::CellTower));
    assert!(state.is_service_unlocked(ServiceType::DataCenter));
}

#[test]
fn test_unlock_service_mapping_landmarks() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let mut state = UnlockState::default();
    state.development_points = 50;
    assert!(!state.is_service_unlocked(ServiceType::CityHall));
    state.purchase(UnlockNode::Landmarks);
    assert!(state.is_service_unlocked(ServiceType::CityHall));
    assert!(state.is_service_unlocked(ServiceType::Museum));
    assert!(state.is_service_unlocked(ServiceType::Cathedral));
    assert!(state.is_service_unlocked(ServiceType::TVStation));
}

#[test]
fn test_unlock_service_mapping_death_care() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let mut state = UnlockState::default();
    state.development_points = 50;
    assert!(!state.is_service_unlocked(ServiceType::Cemetery));
    assert!(!state.is_service_unlocked(ServiceType::Crematorium));
    state.purchase(UnlockNode::DeathCare);
    assert!(state.is_service_unlocked(ServiceType::Cemetery));
    assert!(state.is_service_unlocked(ServiceType::Crematorium));
}

#[test]
fn test_unlock_service_mapping_postal() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let mut state = UnlockState::default();
    state.development_points = 50;
    assert!(!state.is_service_unlocked(ServiceType::PostOffice));
    assert!(!state.is_service_unlocked(ServiceType::MailSortingCenter));
    state.purchase(UnlockNode::PostalService);
    assert!(state.is_service_unlocked(ServiceType::PostOffice));
    assert!(state.is_service_unlocked(ServiceType::MailSortingCenter));
}

#[test]
fn test_unlock_service_mapping_heating() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let mut state = UnlockState::default();
    state.development_points = 50;
    assert!(!state.is_service_unlocked(ServiceType::HeatingBoiler));
    state.purchase(UnlockNode::BasicHeating);
    assert!(state.is_service_unlocked(ServiceType::HeatingBoiler));
    assert!(!state.is_service_unlocked(ServiceType::DistrictHeatingPlant));
    state.purchase(UnlockNode::DistrictHeatingNetwork);
    assert!(state.is_service_unlocked(ServiceType::DistrictHeatingPlant));
    assert!(state.is_service_unlocked(ServiceType::GeothermalPlant));
}

#[test]
fn test_unlock_service_mapping_water_infrastructure() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let mut state = UnlockState::default();
    state.development_points = 50;
    assert!(!state.is_service_unlocked(ServiceType::WaterTreatmentPlant));
    assert!(!state.is_service_unlocked(ServiceType::WellPump));
    state.purchase(UnlockNode::WaterInfrastructure);
    assert!(state.is_service_unlocked(ServiceType::WaterTreatmentPlant));
    assert!(state.is_service_unlocked(ServiceType::WellPump));
}

#[test]
fn test_unlock_utility_mapping_basic_power_and_water() {
    use crate::unlocks::UnlockState;
    let state = UnlockState::default();
    assert!(state.is_utility_unlocked(UtilityType::PowerPlant));
    assert!(state.is_utility_unlocked(UtilityType::WaterTower));
    assert!(state.is_utility_unlocked(UtilityType::PumpingStation));
    assert!(!state.is_utility_unlocked(UtilityType::SolarFarm));
    assert!(!state.is_utility_unlocked(UtilityType::WindTurbine));
    assert!(!state.is_utility_unlocked(UtilityType::NuclearPlant));
}

#[test]
fn test_unlock_utility_mapping_advanced_power() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let mut state = UnlockState::default();
    state.development_points = 50;
    state.purchase(UnlockNode::SolarPower);
    assert!(state.is_utility_unlocked(UtilityType::SolarFarm));
    state.purchase(UnlockNode::WindPower);
    assert!(state.is_utility_unlocked(UtilityType::WindTurbine));
    assert!(state.is_utility_unlocked(UtilityType::Geothermal));
    state.purchase(UnlockNode::NuclearPower);
    assert!(state.is_utility_unlocked(UtilityType::NuclearPlant));
}

#[test]
fn test_unlock_utility_mapping_sewage() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let mut state = UnlockState::default();
    state.development_points = 50;
    assert!(!state.is_utility_unlocked(UtilityType::SewagePlant));
    assert!(!state.is_utility_unlocked(UtilityType::WaterTreatment));
    state.purchase(UnlockNode::SewagePlant);
    assert!(state.is_utility_unlocked(UtilityType::SewagePlant));
    assert!(state.is_utility_unlocked(UtilityType::WaterTreatment));
}

#[test]
fn test_unlock_state_resource_exists_in_test_city() {
    use crate::unlocks::UnlockState;
    let city = TestCity::new();
    city.assert_resource_exists::<UnlockState>();
    let state = city.resource::<UnlockState>();
    assert_eq!(state.unlocked_nodes.len(), 6);
}

#[test]
fn test_achievement_tracker_resource_exists_in_test_city() {
    use crate::achievements::AchievementTracker;
    let city = TestCity::new();
    city.assert_resource_exists::<AchievementTracker>();
    let tracker = city.resource::<AchievementTracker>();
    assert_eq!(tracker.unlocked_count(), 0);
}

#[test]
fn test_achievement_dp_reward_increases_unlock_points() {
    use crate::achievements::{Achievement, AchievementReward};
    let reward = Achievement::Millionaire.reward();
    match reward {
        AchievementReward::DevelopmentPoints(pts) => {
            assert_eq!(pts, 5, "Millionaire should give 5 DP");
        }
        _ => panic!("Millionaire should reward DevelopmentPoints"),
    }
    let reward = Achievement::FullPowerCoverage.reward();
    match reward {
        AchievementReward::DevelopmentPoints(pts) => {
            assert_eq!(pts, 3, "FullPowerCoverage should give 3 DP");
        }
        _ => panic!("FullPowerCoverage should reward DevelopmentPoints"),
    }
}

#[test]
fn test_achievement_treasury_reward_values() {
    use crate::achievements::{Achievement, AchievementReward};
    let reward = Achievement::Population1K.reward();
    match reward {
        AchievementReward::TreasuryBonus(amount) => {
            assert!((amount - 5_000.0).abs() < 0.01, "Pop 1K should give $5K");
        }
        _ => panic!("Population1K should reward TreasuryBonus"),
    }
    let reward = Achievement::Population1M.reward();
    match reward {
        AchievementReward::TreasuryBonus(amount) => {
            assert!(
                (amount - 1_000_000.0).abs() < 0.01,
                "Pop 1M should give $1M"
            );
        }
        _ => panic!("Population1M should reward TreasuryBonus"),
    }
}

#[test]
fn test_achievement_tracker_no_double_unlock() {
    use crate::achievements::{Achievement, AchievementTracker};
    let mut tracker = AchievementTracker::default();
    assert!(!tracker.is_unlocked(Achievement::Population1K));
    tracker.unlocked.insert(Achievement::Population1K, 100);
    assert!(tracker.is_unlocked(Achievement::Population1K));
    assert_eq!(tracker.unlocked_count(), 1);
    tracker.unlocked.insert(Achievement::Population1K, 200);
    assert_eq!(tracker.unlocked_count(), 1);
    assert!(tracker.is_unlocked(Achievement::Population1K));
}

#[test]
fn test_achievement_all_have_metadata() {
    use crate::achievements::Achievement;
    for &a in Achievement::ALL {
        assert!(!a.name().is_empty(), "Achievement {:?} has no name", a);
        assert!(
            !a.description().is_empty(),
            "Achievement {:?} has no description",
            a
        );
        let reward_desc = a.reward().description();
        assert!(
            !reward_desc.is_empty(),
            "Achievement {:?} reward has no description",
            a
        );
    }
}

#[test]
fn test_unlock_purchase_sequence_exhausts_points() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let mut state = UnlockState::default();
    // 3 DP, buy three cost-1 nodes
    assert!(state.purchase(UnlockNode::HealthCare)); // 1
    assert!(state.purchase(UnlockNode::DeathCare)); // 1
    assert!(state.purchase(UnlockNode::BasicSanitation)); // 1
    assert_eq!(state.available_points(), 0);
    assert!(!state.purchase(UnlockNode::FireService));
    assert!(!state.is_unlocked(UnlockNode::FireService));
}

#[test]
fn test_unlock_available_points_uses_saturating_sub() {
    use crate::unlocks::UnlockState;
    let mut state = UnlockState::default();
    state.spent_points = 100;
    assert_eq!(state.available_points(), 0);
}

#[test]
fn test_unlock_service_healthcare_chain() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let mut state = UnlockState::default();
    state.development_points = 50;
    assert!(!state.is_service_unlocked(ServiceType::Hospital));
    assert!(!state.is_service_unlocked(ServiceType::MedicalClinic));
    assert!(!state.is_service_unlocked(ServiceType::HomelessShelter));
    assert!(!state.is_service_unlocked(ServiceType::WelfareOffice));
    state.purchase(UnlockNode::HealthCare);
    assert!(state.is_service_unlocked(ServiceType::Hospital));
    assert!(state.is_service_unlocked(ServiceType::MedicalClinic));
    assert!(state.is_service_unlocked(ServiceType::HomelessShelter));
    assert!(state.is_service_unlocked(ServiceType::WelfareOffice));
}

#[test]
fn test_unlock_service_parks_and_entertainment() {
    use crate::unlocks::{UnlockNode, UnlockState};
    let mut state = UnlockState::default();
    state.development_points = 50;
    state.purchase(UnlockNode::SmallParks);
    assert!(state.is_service_unlocked(ServiceType::SmallPark));
    assert!(state.is_service_unlocked(ServiceType::Playground));
    assert!(!state.is_service_unlocked(ServiceType::LargePark));
    state.purchase(UnlockNode::AdvancedParks);
    assert!(state.is_service_unlocked(ServiceType::LargePark));
    assert!(state.is_service_unlocked(ServiceType::SportsField));
    assert!(!state.is_service_unlocked(ServiceType::Plaza));
    state.purchase(UnlockNode::Entertainment);
    assert!(state.is_service_unlocked(ServiceType::Plaza));
    assert!(state.is_service_unlocked(ServiceType::Stadium));
}
