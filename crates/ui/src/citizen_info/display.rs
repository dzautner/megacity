//! Display helpers for the citizen info panel: labels, colors, and UI bars.

use bevy_egui::egui;
use simulation::citizen::{CitizenState, Gender};

pub fn state_label(state: CitizenState) -> &'static str {
    match state {
        CitizenState::AtHome => "At Home",
        CitizenState::CommutingToWork => "Commuting to Work",
        CitizenState::Working => "Working",
        CitizenState::CommutingHome => "Commuting Home",
        CitizenState::CommutingToShop => "Going Shopping",
        CitizenState::Shopping => "Shopping",
        CitizenState::CommutingToLeisure => "Going to Leisure",
        CitizenState::AtLeisure => "At Leisure",
        CitizenState::CommutingToSchool => "Going to School",
        CitizenState::AtSchool => "At School",
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

pub fn gender_label(gender: Gender) -> &'static str {
    match gender {
        Gender::Male => "Male",
        Gender::Female => "Female",
    }
}

pub fn happiness_color(happiness: f32) -> egui::Color32 {
    if happiness >= 70.0 {
        egui::Color32::from_rgb(50, 200, 50)
    } else if happiness >= 40.0 {
        egui::Color32::from_rgb(220, 180, 50)
    } else {
        egui::Color32::from_rgb(220, 50, 50)
    }
}

pub fn need_color(value: f32) -> egui::Color32 {
    let pct = value / 100.0;
    if pct > 0.6 {
        egui::Color32::from_rgb(50, 200, 50)
    } else if pct > 0.3 {
        egui::Color32::from_rgb(220, 180, 50)
    } else {
        egui::Color32::from_rgb(220, 50, 50)
    }
}

pub fn needs_bar(ui: &mut egui::Ui, label: &str, value: f32) {
    ui.horizontal(|ui| {
        ui.label(format!("{:>7}", label));
        let (rect, _) = ui.allocate_exact_size(egui::vec2(80.0, 10.0), egui::Sense::hover());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 2.0, egui::Color32::from_gray(40));
        let pct = (value / 100.0).clamp(0.0, 1.0);
        let color = need_color(value);
        let fill_rect =
            egui::Rect::from_min_size(rect.min, egui::vec2(rect.width() * pct, rect.height()));
        painter.rect_filled(fill_rect, 2.0, color);
        ui.label(format!("{:.0}%", value));
    });
}
