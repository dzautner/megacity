use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDeposit {
    pub resource_type: ResourceType,
    pub amount: u32, // Remaining amount (finite resources deplete)
    pub max_amount: u32,
}
