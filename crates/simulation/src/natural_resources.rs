use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{GRID_HEIGHT, GRID_WIDTH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    FertileLand,
    Forest,
    Ore,
    Oil,
}

impl ResourceType {
    pub fn is_renewable(self) -> bool {
        matches!(self, ResourceType::FertileLand | ResourceType::Forest)
    }
    pub fn name(self) -> &'static str {
        match self {
            ResourceType::FertileLand => "Fertile Land",
            ResourceType::Forest => "Forest",
            ResourceType::Ore => "Ore Deposit",
            ResourceType::Oil => "Oil Deposit",
        }
    }
}

/// Grid of natural resource deposits, generated alongside terrain
#[derive(Resource)]
pub struct ResourceGrid {
    pub deposits: Vec<Option<ResourceDeposit>>,
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDeposit {
    pub resource_type: ResourceType,
    pub amount: u32, // Remaining amount (finite resources deplete)
    pub max_amount: u32,
}

impl Default for ResourceGrid {
    fn default() -> Self {
        Self {
            deposits: vec![None; GRID_WIDTH * GRID_HEIGHT],
            width: GRID_WIDTH,
            height: GRID_HEIGHT,
        }
    }
}

impl ResourceGrid {
    pub fn get(&self, x: usize, y: usize) -> &Option<ResourceDeposit> {
        &self.deposits[y * self.width + x]
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> &mut Option<ResourceDeposit> {
        &mut self.deposits[y * self.width + x]
    }

    pub fn set(&mut self, x: usize, y: usize, deposit: ResourceDeposit) {
        self.deposits[y * self.width + x] = Some(deposit);
    }
}

/// Tracks city-wide resource production and consumption
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceBalance {
    pub food_production: f32,
    pub food_consumption: f32,
    pub timber_production: f32,
    pub timber_consumption: f32,
    pub metal_production: f32,
    pub metal_consumption: f32,
    pub fuel_production: f32,
    pub fuel_consumption: f32,
}

impl ResourceBalance {
    pub fn surplus(&self, resource: ResourceType) -> f32 {
        match resource {
            ResourceType::FertileLand => self.food_production - self.food_consumption,
            ResourceType::Forest => self.timber_production - self.timber_consumption,
            ResourceType::Ore => self.metal_production - self.metal_consumption,
            ResourceType::Oil => self.fuel_production - self.fuel_consumption,
        }
    }

    /// Trade income/cost from surplus/deficit. Surplus = export income, deficit = import cost
    pub fn trade_balance(&self) -> f64 {
        let mut balance = 0.0f64;
        for &rt in &[
            ResourceType::FertileLand,
            ResourceType::Forest,
            ResourceType::Ore,
            ResourceType::Oil,
        ] {
            let surplus = self.surplus(rt);
            if surplus > 0.0 {
                balance += surplus as f64 * 3.0; // Export income per unit
            } else {
                balance += surplus as f64 * 5.0; // Import cost per unit (more expensive)
            }
        }
        balance
    }
}

/// Generate resource deposits based on terrain elevation and noise
pub fn generate_resources(grid: &mut ResourceGrid, elevation: &[f32], seed: u32) {
    let width = grid.width;
    let height = grid.height;

    for y in 0..height {
        for x in 0..width {
            let elev = elevation[y * width + x];
            // Simple deterministic placement based on position hash + elevation
            let hash =
                (x.wrapping_mul(seed as usize + 7) ^ y.wrapping_mul(seed as usize + 13)) % 1000;

            if elev < 0.35 {
                continue; // Water - no resources
            }

            let deposit = if elev < 0.45 && hash < 30 {
                // Low elevation near water = fertile land
                Some(ResourceDeposit {
                    resource_type: ResourceType::FertileLand,
                    amount: 10000,
                    max_amount: 10000,
                })
            } else if elev > 0.45 && elev < 0.6 && hash < 25 {
                // Mid elevation = forest
                Some(ResourceDeposit {
                    resource_type: ResourceType::Forest,
                    amount: 8000,
                    max_amount: 8000,
                })
            } else if elev > 0.65 && hash < 15 {
                // High elevation = ore
                Some(ResourceDeposit {
                    resource_type: ResourceType::Ore,
                    amount: 5000,
                    max_amount: 5000,
                })
            } else if elev > 0.5 && elev < 0.65 && hash < 8 {
                // Mid-high = oil (rare)
                Some(ResourceDeposit {
                    resource_type: ResourceType::Oil,
                    amount: 3000,
                    max_amount: 3000,
                })
            } else {
                None
            };

            if let Some(d) = deposit {
                grid.set(x, y, d);
            }
        }
    }
}

/// System: update resource production from industrial buildings on resource deposits.
/// Finite resources (ore, oil) deplete over time. Renewable resources (forests, fertile land) regenerate.
pub fn update_resource_production(
    mut resource_grid: ResMut<ResourceGrid>,
    buildings: Query<&crate::buildings::Building>,
    mut balance: ResMut<ResourceBalance>,
    stats: Res<crate::stats::CityStats>,
) {
    // Reset production
    balance.food_production = 0.0;
    balance.timber_production = 0.0;
    balance.metal_production = 0.0;
    balance.fuel_production = 0.0;

    // Industrial buildings on resource deposits produce resources
    for building in &buildings {
        if building.zone_type != crate::grid::ZoneType::Industrial {
            continue;
        }
        if let Some(deposit) = resource_grid.get(building.grid_x, building.grid_y) {
            if deposit.amount == 0 {
                continue; // Depleted deposit produces nothing
            }
            let output = building.occupants as f32 * 0.5; // Per occupied worker
            match deposit.resource_type {
                ResourceType::FertileLand => balance.food_production += output,
                ResourceType::Forest => balance.timber_production += output,
                ResourceType::Ore => balance.metal_production += output,
                ResourceType::Oil => balance.fuel_production += output,
            }

            // Deplete finite resources; regenerate renewable ones
            let deposit = resource_grid.get_mut(building.grid_x, building.grid_y);
            if let Some(ref mut d) = deposit {
                if d.resource_type.is_renewable() {
                    // Renewable resources slowly regenerate (but extraction draws down)
                    let extraction = (output * 0.1) as u32;
                    d.amount = d.amount.saturating_sub(extraction);
                    // Regenerate a small amount each tick
                    d.amount = (d.amount + 1).min(d.max_amount);
                } else {
                    // Finite resources deplete permanently
                    let extraction = (output * 0.2) as u32;
                    d.amount = d.amount.saturating_sub(extraction.max(1));
                }
            }
        }
    }

    // Consumption based on population
    let pop = stats.population as f32;
    balance.food_consumption = pop * 0.02; // Each citizen needs food
    balance.timber_consumption = pop * 0.005; // Construction materials
    balance.metal_consumption = pop * 0.003; // Manufactured goods
    balance.fuel_consumption = pop * 0.004; // Energy supplement
}

pub struct NaturalResourcesPlugin;

impl Plugin for NaturalResourcesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ResourceGrid>()
            .init_resource::<ResourceBalance>()
            .add_systems(
                FixedUpdate,
                update_resource_production
                    .after(crate::imports_exports::process_trade)
                    .in_set(crate::SimulationSet::Simulation),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ====================================================================
    // 1. ResourceType unit tests
    // ====================================================================

    #[test]
    fn test_resource_type_renewable_classification() {
        assert!(
            ResourceType::FertileLand.is_renewable(),
            "FertileLand should be renewable"
        );
        assert!(
            ResourceType::Forest.is_renewable(),
            "Forest should be renewable"
        );
        assert!(
            !ResourceType::Ore.is_renewable(),
            "Ore should NOT be renewable"
        );
        assert!(
            !ResourceType::Oil.is_renewable(),
            "Oil should NOT be renewable"
        );
    }

    #[test]
    fn test_resource_type_names() {
        assert_eq!(ResourceType::FertileLand.name(), "Fertile Land");
        assert_eq!(ResourceType::Forest.name(), "Forest");
        assert_eq!(ResourceType::Ore.name(), "Ore Deposit");
        assert_eq!(ResourceType::Oil.name(), "Oil Deposit");
    }

    // ====================================================================
    // 2. ResourceGrid unit tests
    // ====================================================================

    #[test]
    fn test_resource_grid_default_is_empty() {
        let grid = ResourceGrid::default();
        assert_eq!(grid.width, GRID_WIDTH);
        assert_eq!(grid.height, GRID_HEIGHT);
        assert_eq!(grid.deposits.len(), GRID_WIDTH * GRID_HEIGHT);
        // All cells should be None by default
        for deposit in &grid.deposits {
            assert!(deposit.is_none(), "default grid should have no deposits");
        }
    }

    #[test]
    fn test_resource_grid_set_and_get() {
        let mut grid = ResourceGrid::default();
        let deposit = ResourceDeposit {
            resource_type: ResourceType::Ore,
            amount: 5000,
            max_amount: 5000,
        };
        grid.set(10, 20, deposit.clone());

        let retrieved = grid.get(10, 20);
        assert!(retrieved.is_some(), "should retrieve the set deposit");
        let retrieved = retrieved.as_ref().unwrap();
        assert_eq!(retrieved.resource_type, ResourceType::Ore);
        assert_eq!(retrieved.amount, 5000);
        assert_eq!(retrieved.max_amount, 5000);
    }

    #[test]
    fn test_resource_grid_get_mut_modifies_deposit() {
        let mut grid = ResourceGrid::default();
        grid.set(
            5,
            5,
            ResourceDeposit {
                resource_type: ResourceType::Oil,
                amount: 3000,
                max_amount: 3000,
            },
        );

        // Deplete via get_mut
        if let Some(ref mut d) = grid.get_mut(5, 5) {
            d.amount = 1000;
        }

        let deposit = grid.get(5, 5).as_ref().unwrap();
        assert_eq!(deposit.amount, 1000, "amount should be modified to 1000");
        assert_eq!(
            deposit.max_amount, 3000,
            "max_amount should remain unchanged"
        );
    }

    #[test]
    fn test_resource_grid_multiple_deposits_independent() {
        let mut grid = ResourceGrid::default();
        grid.set(
            0,
            0,
            ResourceDeposit {
                resource_type: ResourceType::FertileLand,
                amount: 10000,
                max_amount: 10000,
            },
        );
        grid.set(
            100,
            100,
            ResourceDeposit {
                resource_type: ResourceType::Forest,
                amount: 8000,
                max_amount: 8000,
            },
        );
        grid.set(
            200,
            200,
            ResourceDeposit {
                resource_type: ResourceType::Ore,
                amount: 5000,
                max_amount: 5000,
            },
        );

        assert_eq!(
            grid.get(0, 0).as_ref().unwrap().resource_type,
            ResourceType::FertileLand
        );
        assert_eq!(
            grid.get(100, 100).as_ref().unwrap().resource_type,
            ResourceType::Forest
        );
        assert_eq!(
            grid.get(200, 200).as_ref().unwrap().resource_type,
            ResourceType::Ore
        );
        // Other cells should still be None
        assert!(grid.get(50, 50).is_none());
    }

    #[test]
    fn test_resource_grid_overwrite_deposit() {
        let mut grid = ResourceGrid::default();
        grid.set(
            10,
            10,
            ResourceDeposit {
                resource_type: ResourceType::Ore,
                amount: 5000,
                max_amount: 5000,
            },
        );
        // Overwrite with a different resource type
        grid.set(
            10,
            10,
            ResourceDeposit {
                resource_type: ResourceType::Oil,
                amount: 3000,
                max_amount: 3000,
            },
        );

        let deposit = grid.get(10, 10).as_ref().unwrap();
        assert_eq!(
            deposit.resource_type,
            ResourceType::Oil,
            "deposit should be overwritten"
        );
        assert_eq!(deposit.amount, 3000);
    }

    // ====================================================================
    // 3. ResourceBalance unit tests
    // ====================================================================

    #[test]
    fn test_resource_balance_default_is_zero() {
        let balance = ResourceBalance::default();
        assert_eq!(balance.food_production, 0.0);
        assert_eq!(balance.food_consumption, 0.0);
        assert_eq!(balance.timber_production, 0.0);
        assert_eq!(balance.timber_consumption, 0.0);
        assert_eq!(balance.metal_production, 0.0);
        assert_eq!(balance.metal_consumption, 0.0);
        assert_eq!(balance.fuel_production, 0.0);
        assert_eq!(balance.fuel_consumption, 0.0);
    }

    #[test]
    fn test_surplus_positive_when_production_exceeds_consumption() {
        let balance = ResourceBalance {
            food_production: 100.0,
            food_consumption: 40.0,
            ..Default::default()
        };
        let surplus = balance.surplus(ResourceType::FertileLand);
        assert!(
            (surplus - 60.0).abs() < f32::EPSILON,
            "food surplus should be 60.0, got {surplus}"
        );
    }

    #[test]
    fn test_surplus_negative_when_consumption_exceeds_production() {
        let balance = ResourceBalance {
            metal_production: 10.0,
            metal_consumption: 50.0,
            ..Default::default()
        };
        let surplus = balance.surplus(ResourceType::Ore);
        assert!(
            (surplus - (-40.0)).abs() < f32::EPSILON,
            "metal surplus should be -40.0, got {surplus}"
        );
    }

    #[test]
    fn test_surplus_zero_when_balanced() {
        let balance = ResourceBalance {
            timber_production: 25.0,
            timber_consumption: 25.0,
            ..Default::default()
        };
        let surplus = balance.surplus(ResourceType::Forest);
        assert!(
            surplus.abs() < f32::EPSILON,
            "timber surplus should be 0.0, got {surplus}"
        );
    }

    #[test]
    fn test_surplus_maps_to_correct_resource_type() {
        let balance = ResourceBalance {
            food_production: 10.0,
            food_consumption: 5.0,
            timber_production: 20.0,
            timber_consumption: 8.0,
            metal_production: 30.0,
            metal_consumption: 15.0,
            fuel_production: 40.0,
            fuel_consumption: 25.0,
        };
        assert!((balance.surplus(ResourceType::FertileLand) - 5.0).abs() < f32::EPSILON);
        assert!((balance.surplus(ResourceType::Forest) - 12.0).abs() < f32::EPSILON);
        assert!((balance.surplus(ResourceType::Ore) - 15.0).abs() < f32::EPSILON);
        assert!((balance.surplus(ResourceType::Oil) - 15.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_trade_balance_all_surplus_gives_positive_income() {
        let balance = ResourceBalance {
            food_production: 20.0,
            food_consumption: 10.0, // surplus 10
            timber_production: 15.0,
            timber_consumption: 5.0, // surplus 10
            metal_production: 12.0,
            metal_consumption: 2.0, // surplus 10
            fuel_production: 18.0,
            fuel_consumption: 8.0, // surplus 10
        };
        // Each surplus unit exports at 3.0 per unit
        // Total = 4 resources * 10 surplus * 3.0 = 120.0
        let tb = balance.trade_balance();
        assert!(
            (tb - 120.0).abs() < 0.01,
            "all-surplus trade balance should be 120.0, got {tb}"
        );
    }

    #[test]
    fn test_trade_balance_all_deficit_gives_negative_cost() {
        let balance = ResourceBalance {
            food_production: 5.0,
            food_consumption: 15.0, // deficit -10
            timber_production: 3.0,
            timber_consumption: 13.0, // deficit -10
            metal_production: 0.0,
            metal_consumption: 10.0, // deficit -10
            fuel_production: 2.0,
            fuel_consumption: 12.0, // deficit -10
        };
        // Each deficit unit costs 5.0 per unit
        // Total = 4 resources * -10 deficit * 5.0 = -200.0
        let tb = balance.trade_balance();
        assert!(
            (tb - (-200.0)).abs() < 0.01,
            "all-deficit trade balance should be -200.0, got {tb}"
        );
    }

    #[test]
    fn test_trade_balance_mixed_surplus_and_deficit() {
        let balance = ResourceBalance {
            food_production: 30.0,
            food_consumption: 10.0, // surplus 20 => +60
            timber_production: 5.0,
            timber_consumption: 15.0, // deficit -10 => -50
            metal_production: 0.0,
            metal_consumption: 0.0, // balanced => 0
            fuel_production: 0.0,
            fuel_consumption: 0.0, // balanced => 0
        };
        let tb = balance.trade_balance();
        // 60 + (-50) = 10
        assert!(
            (tb - 10.0).abs() < 0.01,
            "mixed trade balance should be 10.0, got {tb}"
        );
    }

    #[test]
    fn test_trade_balance_zero_when_all_balanced() {
        let balance = ResourceBalance {
            food_production: 10.0,
            food_consumption: 10.0,
            timber_production: 5.0,
            timber_consumption: 5.0,
            metal_production: 8.0,
            metal_consumption: 8.0,
            fuel_production: 3.0,
            fuel_consumption: 3.0,
        };
        let tb = balance.trade_balance();
        assert!(tb.abs() < 0.01, "balanced trade should be 0.0, got {tb}");
    }

    // ====================================================================
    // 4. Resource generation tests
    // ====================================================================

    #[test]
    fn test_generate_resources_deterministic_with_same_seed() {
        let elevation = vec![0.5; GRID_WIDTH * GRID_HEIGHT];
        let mut grid1 = ResourceGrid::default();
        let mut grid2 = ResourceGrid::default();
        generate_resources(&mut grid1, &elevation, 42);
        generate_resources(&mut grid2, &elevation, 42);

        for i in 0..grid1.deposits.len() {
            match (&grid1.deposits[i], &grid2.deposits[i]) {
                (Some(d1), Some(d2)) => {
                    assert_eq!(d1.resource_type, d2.resource_type);
                    assert_eq!(d1.amount, d2.amount);
                    assert_eq!(d1.max_amount, d2.max_amount);
                }
                (None, None) => {}
                _ => panic!("Mismatch at index {i}: same seed should produce same deposits"),
            }
        }
    }

    #[test]
    fn test_generate_resources_different_seeds_produce_different_results() {
        let elevation = vec![0.5; GRID_WIDTH * GRID_HEIGHT];
        let mut grid1 = ResourceGrid::default();
        let mut grid2 = ResourceGrid::default();
        generate_resources(&mut grid1, &elevation, 42);
        generate_resources(&mut grid2, &elevation, 99);

        let count1: usize = grid1.deposits.iter().filter(|d| d.is_some()).count();
        let count2: usize = grid2.deposits.iter().filter(|d| d.is_some()).count();
        // Different seeds should produce at least slightly different layouts
        // (not strictly guaranteed but extremely likely with different hash seeds)
        let has_difference = count1 != count2
            || grid1
                .deposits
                .iter()
                .zip(grid2.deposits.iter())
                .any(|(d1, d2)| match (d1, d2) {
                    (Some(a), Some(b)) => a.resource_type != b.resource_type,
                    (Some(_), None) | (None, Some(_)) => true,
                    _ => false,
                });
        assert!(
            has_difference,
            "different seeds should produce different resource layouts"
        );
    }

    #[test]
    fn test_generate_resources_no_deposits_on_water() {
        // Elevation below 0.35 = water, should have no resources
        let elevation = vec![0.2; GRID_WIDTH * GRID_HEIGHT];
        let mut grid = ResourceGrid::default();
        generate_resources(&mut grid, &elevation, 42);

        let deposit_count: usize = grid.deposits.iter().filter(|d| d.is_some()).count();
        assert_eq!(
            deposit_count, 0,
            "water tiles (elev < 0.35) should have no deposits, got {deposit_count}"
        );
    }

    #[test]
    fn test_generate_resources_produces_deposits_on_land() {
        // Mid elevation — should produce at least some deposits
        let elevation = vec![0.5; GRID_WIDTH * GRID_HEIGHT];
        let mut grid = ResourceGrid::default();
        generate_resources(&mut grid, &elevation, 42);

        let deposit_count: usize = grid.deposits.iter().filter(|d| d.is_some()).count();
        assert!(
            deposit_count > 0,
            "land tiles should produce at least some deposits"
        );
    }

    #[test]
    fn test_generate_resources_fertile_land_at_low_elevation() {
        // Elevation 0.36-0.44 = low land near water => FertileLand
        let elevation = vec![0.40; GRID_WIDTH * GRID_HEIGHT];
        let mut grid = ResourceGrid::default();
        generate_resources(&mut grid, &elevation, 42);

        let has_fertile = grid
            .deposits
            .iter()
            .any(|d| matches!(d, Some(dep) if dep.resource_type == ResourceType::FertileLand));
        assert!(
            has_fertile,
            "low elevation (0.40) should produce fertile land deposits"
        );
    }

    #[test]
    fn test_generate_resources_ore_at_high_elevation() {
        // Elevation > 0.65 => Ore
        let elevation = vec![0.75; GRID_WIDTH * GRID_HEIGHT];
        let mut grid = ResourceGrid::default();
        generate_resources(&mut grid, &elevation, 42);

        let has_ore = grid
            .deposits
            .iter()
            .any(|d| matches!(d, Some(dep) if dep.resource_type == ResourceType::Ore));
        assert!(has_ore, "high elevation (0.75) should produce ore deposits");
    }

    #[test]
    fn test_generate_resources_initial_amounts_match_max() {
        let elevation = vec![0.5; GRID_WIDTH * GRID_HEIGHT];
        let mut grid = ResourceGrid::default();
        generate_resources(&mut grid, &elevation, 42);

        for deposit in grid.deposits.iter().flatten() {
            assert_eq!(
                deposit.amount, deposit.max_amount,
                "initial amount should equal max_amount for {:?}",
                deposit.resource_type
            );
        }
    }

    #[test]
    fn test_generate_resources_known_max_amounts() {
        let elevation = vec![0.5; GRID_WIDTH * GRID_HEIGHT];
        let mut grid = ResourceGrid::default();
        generate_resources(&mut grid, &elevation, 42);

        for deposit in grid.deposits.iter().flatten() {
            match deposit.resource_type {
                ResourceType::FertileLand => assert_eq!(deposit.max_amount, 10000),
                ResourceType::Forest => assert_eq!(deposit.max_amount, 8000),
                ResourceType::Ore => assert_eq!(deposit.max_amount, 5000),
                ResourceType::Oil => assert_eq!(deposit.max_amount, 3000),
            }
        }
    }

    // ====================================================================
    // 5. Resource depletion logic tests
    // ====================================================================

    #[test]
    fn test_finite_resource_depletion_reduces_amount() {
        let mut deposit = ResourceDeposit {
            resource_type: ResourceType::Ore,
            amount: 5000,
            max_amount: 5000,
        };
        // Simulate finite extraction: output * 0.2, at least 1
        let output = 10.0_f32;
        let extraction = (output * 0.2) as u32;
        deposit.amount = deposit.amount.saturating_sub(extraction.max(1));
        assert_eq!(deposit.amount, 4998, "Ore should deplete by extraction (2)");
    }

    #[test]
    fn test_finite_resource_fully_depletes_to_zero() {
        let mut deposit = ResourceDeposit {
            resource_type: ResourceType::Oil,
            amount: 1,
            max_amount: 3000,
        };
        let extraction = 5_u32;
        deposit.amount = deposit.amount.saturating_sub(extraction.max(1));
        assert_eq!(deposit.amount, 0, "depleted resource should saturate at 0");
    }

    #[test]
    fn test_renewable_resource_regenerates() {
        let mut deposit = ResourceDeposit {
            resource_type: ResourceType::Forest,
            amount: 7990,
            max_amount: 8000,
        };
        // Simulate renewable logic: extract then regen +1
        let output = 5.0_f32;
        let extraction = (output * 0.1) as u32; // 0
        deposit.amount = deposit.amount.saturating_sub(extraction);
        deposit.amount = (deposit.amount + 1).min(deposit.max_amount);
        assert_eq!(
            deposit.amount, 7991,
            "renewable resource should regenerate +1 when extraction is small"
        );
    }

    #[test]
    fn test_renewable_resource_capped_at_max() {
        let mut deposit = ResourceDeposit {
            resource_type: ResourceType::FertileLand,
            amount: 10000,
            max_amount: 10000,
        };
        // No extraction, just regen
        deposit.amount = (deposit.amount + 1).min(deposit.max_amount);
        assert_eq!(
            deposit.amount, 10000,
            "renewable resource should not exceed max_amount"
        );
    }

    #[test]
    fn test_depleted_finite_resource_stays_at_zero() {
        let mut deposit = ResourceDeposit {
            resource_type: ResourceType::Ore,
            amount: 0,
            max_amount: 5000,
        };
        // Attempt further extraction
        let extraction = 10_u32;
        deposit.amount = deposit.amount.saturating_sub(extraction.max(1));
        assert_eq!(
            deposit.amount, 0,
            "already depleted resource should stay at 0"
        );
    }

    // ====================================================================
    // 6. Import/export cost asymmetry tests
    // ====================================================================

    #[test]
    fn test_import_more_expensive_than_export() {
        // Export 10 units of food: income = 10 * 3.0 = 30.0
        let export_balance = ResourceBalance {
            food_production: 10.0,
            food_consumption: 0.0,
            ..Default::default()
        };
        // Import 10 units of food: cost = -10 * 5.0 = -50.0
        let import_balance = ResourceBalance {
            food_production: 0.0,
            food_consumption: 10.0,
            ..Default::default()
        };
        let export_income = export_balance.trade_balance();
        let import_cost = import_balance.trade_balance();
        assert!(
            export_income > 0.0,
            "exporting should yield positive income"
        );
        assert!(import_cost < 0.0, "importing should yield negative cost");
        assert!(
            import_cost.abs() > export_income,
            "importing should cost more than exporting earns"
        );
    }
}
