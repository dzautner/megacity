use super::grid::ResourceGrid;
use super::types::{ResourceDeposit, ResourceType};

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
