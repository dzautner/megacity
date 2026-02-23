//! Shared helper functions, color utilities, and name-generation data
//! used by the Building Inspector tabs.

use bevy::prelude::*;
use bevy_egui::egui;

use simulation::citizen::{CitizenState, Gender};
use simulation::grid::ZoneType;
use simulation::trees::TreeGrid;

use super::types::BuildingTab;

// =============================================================================
// Label / color helpers
// =============================================================================

pub(crate) fn zone_type_label(zone: ZoneType) -> &'static str {
    match zone {
        ZoneType::None => "Unzoned",
        ZoneType::ResidentialLow => "Low-Density Residential",
        ZoneType::ResidentialMedium => "Medium-Density Residential",
        ZoneType::ResidentialHigh => "High-Density Residential",
        ZoneType::CommercialLow => "Low-Density Commercial",
        ZoneType::CommercialHigh => "High-Density Commercial",
        ZoneType::Industrial => "Industrial",
        ZoneType::Office => "Office",
        ZoneType::MixedUse => "Mixed-Use",
    }
}

pub(crate) fn happiness_color(happiness: f32) -> egui::Color32 {
    if happiness >= 70.0 {
        egui::Color32::from_rgb(50, 200, 50)
    } else if happiness >= 40.0 {
        egui::Color32::from_rgb(220, 180, 50)
    } else {
        egui::Color32::from_rgb(220, 50, 50)
    }
}

pub(crate) fn occupancy_color(pct: f32) -> egui::Color32 {
    if pct >= 90.0 {
        egui::Color32::from_rgb(220, 50, 50)
    } else if pct >= 70.0 {
        egui::Color32::from_rgb(220, 180, 50)
    } else {
        egui::Color32::from_rgb(50, 200, 50)
    }
}

pub(crate) fn pollution_color(level: u8) -> egui::Color32 {
    if level > 50 {
        egui::Color32::from_rgb(200, 50, 50)
    } else if level > 20 {
        egui::Color32::from_rgb(200, 150, 50)
    } else {
        egui::Color32::from_rgb(50, 200, 50)
    }
}

pub(crate) fn noise_color(level: u8) -> egui::Color32 {
    if level > 60 {
        egui::Color32::from_rgb(200, 50, 50)
    } else if level > 30 {
        egui::Color32::from_rgb(200, 150, 50)
    } else {
        egui::Color32::from_rgb(50, 200, 50)
    }
}

pub(crate) fn education_short(education: u8) -> &'static str {
    match education {
        0 => "-",
        1 => "Elem",
        2 => "HS",
        3 => "Uni",
        _ => "Adv",
    }
}

pub(crate) fn citizen_state_label(state: CitizenState) -> &'static str {
    match state {
        CitizenState::AtHome => "Home",
        CitizenState::CommutingToWork => "To Work",
        CitizenState::Working => "Working",
        CitizenState::CommutingHome => "Going Home",
        CitizenState::CommutingToShop => "To Shop",
        CitizenState::Shopping => "Shopping",
        CitizenState::CommutingToLeisure => "To Leisure",
        CitizenState::AtLeisure => "Leisure",
        CitizenState::CommutingToSchool => "To School",
        CitizenState::AtSchool => "At School",
    }
}

// =============================================================================
// Name generation data
// =============================================================================

pub(crate) const FIRST_NAMES_M: &[&str] = &[
    "James", "John", "Robert", "Michael", "David", "William", "Richard", "Joseph", "Thomas",
    "Daniel", "Matthew", "Anthony", "Mark", "Steven", "Paul", "Andrew", "Joshua", "Kenneth",
    "Kevin", "Brian", "George", "Timothy", "Ronald", "Edward", "Jason", "Jeffrey", "Ryan", "Jacob",
    "Gary", "Nicholas", "Eric", "Jonathan",
];
pub(crate) const FIRST_NAMES_F: &[&str] = &[
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
    "Emily",
    "Donna",
    "Michelle",
    "Carol",
    "Amanda",
    "Dorothy",
    "Melissa",
    "Deborah",
    "Stephanie",
    "Rebecca",
    "Sharon",
    "Laura",
    "Cynthia",
    "Kathleen",
    "Amy",
    "Angela",
];
pub(crate) const LAST_NAMES: &[&str] = &[
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

pub(crate) fn gen_citizen_name(entity: Entity, gender: Gender) -> String {
    let idx = entity.index() as usize;
    let first = match gender {
        Gender::Male => FIRST_NAMES_M[idx % FIRST_NAMES_M.len()],
        Gender::Female => FIRST_NAMES_F[idx % FIRST_NAMES_F.len()],
    };
    let last = LAST_NAMES[(idx / 31) % LAST_NAMES.len()];
    format!("{} {}", first, last)
}

// =============================================================================
// UI widgets
// =============================================================================

pub(crate) fn needs_bar(ui: &mut egui::Ui, label: &str, value: f32) {
    ui.horizontal(|ui| {
        ui.label(format!("{:>7}", label));
        let (rect, _) = ui.allocate_exact_size(egui::vec2(80.0, 10.0), egui::Sense::hover());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 2.0, egui::Color32::from_gray(40));
        let pct = (value / 100.0).clamp(0.0, 1.0);
        let color = if pct > 0.6 {
            egui::Color32::from_rgb(50, 200, 50)
        } else if pct > 0.3 {
            egui::Color32::from_rgb(220, 180, 50)
        } else {
            egui::Color32::from_rgb(220, 50, 50)
        };
        let fill_rect =
            egui::Rect::from_min_size(rect.min, egui::vec2(rect.width() * pct, rect.height()));
        painter.rect_filled(fill_rect, 2.0, color);
        ui.label(format!("{:.0}%", value));
    });
}

/// Renders a horizontal tab bar, returning the updated active tab.
pub(crate) fn tab_bar(ui: &mut egui::Ui, active: &mut BuildingTab) {
    ui.horizontal(|ui| {
        for tab in BuildingTab::ALL {
            let is_selected = *active == tab;
            let text = egui::RichText::new(tab.label());
            let text = if is_selected {
                text.strong().color(egui::Color32::from_rgb(220, 220, 255))
            } else {
                text.color(egui::Color32::from_rgb(160, 160, 180))
            };
            if ui.add(egui::Button::new(text).frame(is_selected)).clicked() {
                *active = tab;
            }
        }
    });
    ui.separator();
}

/// Count nearby trees (green space) within a radius around a grid position.
pub(crate) fn count_nearby_trees(
    tree_grid: &TreeGrid,
    gx: usize,
    gy: usize,
    radius: usize,
) -> usize {
    let mut count = 0;
    let min_x = gx.saturating_sub(radius);
    let max_x = (gx + radius).min(tree_grid.width.saturating_sub(1));
    let min_y = gy.saturating_sub(radius);
    let max_y = (gy + radius).min(tree_grid.height.saturating_sub(1));
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            if tree_grid.has_tree(x, y) {
                count += 1;
            }
        }
    }
    count
}

pub(crate) fn green_space_label(count: usize) -> (&'static str, egui::Color32) {
    if count >= 20 {
        ("Excellent", egui::Color32::from_rgb(50, 200, 50))
    } else if count >= 10 {
        ("Good", egui::Color32::from_rgb(80, 200, 80))
    } else if count >= 4 {
        ("Moderate", egui::Color32::from_rgb(220, 180, 50))
    } else if count >= 1 {
        ("Low", egui::Color32::from_rgb(220, 120, 50))
    } else {
        ("None", egui::Color32::from_rgb(180, 80, 80))
    }
}
