//! Display helpers and name generation for search results.

use bevy::prelude::*;
use simulation::citizen::Gender;
use simulation::grid::ZoneType;

// ---------------------------------------------------------------------------
// Name generation (matching citizen_info.rs)
// ---------------------------------------------------------------------------

const FIRST_NAMES_M: &[&str] = &[
    "James", "John", "Robert", "Michael", "David", "William", "Richard", "Joseph", "Thomas",
    "Daniel", "Matthew", "Anthony", "Mark", "Steven", "Paul", "Andrew", "Joshua", "Kenneth",
    "Kevin", "Brian", "George", "Timothy", "Ronald", "Edward", "Jason", "Jeffrey", "Ryan", "Jacob",
    "Gary", "Nicholas", "Eric", "Jonathan",
];

const FIRST_NAMES_F: &[&str] = &[
    "Mary",
    "Patricia",
    "Jennifer",
    "Linda",
    "Barbara",
    "Elizabeth",
    "Susan",
    "Jessica",
    "Sarah",
    "Karen",
    "Lisa",
    "Nancy",
    "Betty",
    "Margaret",
    "Sandra",
    "Ashley",
    "Dorothy",
    "Kimberly",
    "Emily",
    "Donna",
    "Michelle",
    "Carol",
    "Amanda",
    "Melissa",
    "Deborah",
    "Stephanie",
    "Rebecca",
    "Sharon",
    "Laura",
    "Cynthia",
    "Kathleen",
    "Amy",
];

const LAST_NAMES: &[&str] = &[
    "Smith",
    "Johnson",
    "Williams",
    "Brown",
    "Jones",
    "Garcia",
    "Miller",
    "Davis",
    "Rodriguez",
    "Martinez",
    "Hernandez",
    "Lopez",
    "Gonzalez",
    "Wilson",
    "Anderson",
    "Thomas",
    "Taylor",
    "Moore",
    "Jackson",
    "Martin",
    "Lee",
    "Thompson",
    "White",
    "Harris",
    "Clark",
    "Lewis",
    "Robinson",
    "Walker",
    "Young",
    "Allen",
    "King",
    "Wright",
    "Hill",
];

pub fn citizen_name(entity: Entity, gender: Gender) -> String {
    let idx = entity.index() as usize;
    let first = match gender {
        Gender::Male => FIRST_NAMES_M[idx % FIRST_NAMES_M.len()],
        Gender::Female => FIRST_NAMES_F[idx % FIRST_NAMES_F.len()],
    };
    let last = LAST_NAMES[(idx / 31) % LAST_NAMES.len()];
    format!("{} {}", first, last)
}

// ---------------------------------------------------------------------------
// Display helpers
// ---------------------------------------------------------------------------

pub fn zone_label(zone: ZoneType) -> &'static str {
    match zone {
        ZoneType::None => "None",
        ZoneType::ResidentialLow => "Residential (Low)",
        ZoneType::ResidentialMedium => "Residential (Med)",
        ZoneType::ResidentialHigh => "Residential (High)",
        ZoneType::CommercialLow => "Commercial (Low)",
        ZoneType::CommercialHigh => "Commercial (High)",
        ZoneType::Industrial => "Industrial",
        ZoneType::Office => "Office",
        ZoneType::MixedUse => "Mixed Use",
    }
}

pub fn education_label(education: u8) -> &'static str {
    match education {
        0 => "None",
        1 => "Elementary",
        2 => "High School",
        3 => "University",
        _ => "Advanced",
    }
}
