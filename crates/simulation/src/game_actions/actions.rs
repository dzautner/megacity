use serde::{Deserialize, Serialize};

use crate::grid::{RoadType, ZoneType};
use crate::utilities::UtilityType;
use crate::services::ServiceType;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GameAction {
    NewGame { 
        seed: u64, 
        map_size: Option<u32> 
    },
    SetPaused { 
        paused: bool 
    },
    SetSpeed { 
        speed: u32 
    },
    PlaceRoadLine { 
        start: (u32, u32), 
        end: (u32, u32), 
        road_type: RoadType 
    },
    ZoneRect { 
        min: (u32, u32), 
        max: (u32, u32), 
        zone_type: ZoneType 
    },
    PlaceUtility { 
        pos: (u32, u32), 
        utility_type: UtilityType 
    },
    PlaceService { 
        pos: (u32, u32), 
        service_type: ServiceType 
    },
    BulldozeRect { 
        min: (u32, u32), 
        max: (u32, u32) 
    },
    SetTaxRates { 
        residential: f32, 
        commercial: f32, 
        industrial: f32, 
        office: f32 
    },
    Save { 
        path: String 
    },
    Load { 
        path: String 
    },
}
