//! Tests for citizen info panel helpers.

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy_egui::egui;
    use simulation::citizen::{CitizenState, Gender};

    use crate::citizen_info::display::{
        education_label, gender_label, happiness_color, need_color, state_label,
    };
    use crate::citizen_info::names::citizen_name;
    use crate::citizen_info::resources::{FollowCitizen, SelectedCitizen};

    #[test]
    fn test_state_label_all_states() {
        assert_eq!(state_label(CitizenState::AtHome), "At Home");
        assert_eq!(
            state_label(CitizenState::CommutingToWork),
            "Commuting to Work"
        );
        assert_eq!(state_label(CitizenState::Working), "Working");
        assert_eq!(state_label(CitizenState::CommutingHome), "Commuting Home");
        assert_eq!(state_label(CitizenState::CommutingToShop), "Going Shopping");
        assert_eq!(state_label(CitizenState::Shopping), "Shopping");
        assert_eq!(
            state_label(CitizenState::CommutingToLeisure),
            "Going to Leisure"
        );
        assert_eq!(state_label(CitizenState::AtLeisure), "At Leisure");
        assert_eq!(
            state_label(CitizenState::CommutingToSchool),
            "Going to School"
        );
        assert_eq!(state_label(CitizenState::AtSchool), "At School");
    }

    #[test]
    fn test_education_label() {
        assert_eq!(education_label(0), "None");
        assert_eq!(education_label(1), "Elementary");
        assert_eq!(education_label(2), "High School");
        assert_eq!(education_label(3), "University");
        assert_eq!(education_label(4), "Advanced");
    }

    #[test]
    fn test_gender_label() {
        assert_eq!(gender_label(Gender::Male), "Male");
        assert_eq!(gender_label(Gender::Female), "Female");
    }

    #[test]
    fn test_happiness_color_green() {
        let color = happiness_color(80.0);
        assert_eq!(color, egui::Color32::from_rgb(50, 200, 50));
    }

    #[test]
    fn test_happiness_color_yellow() {
        let color = happiness_color(50.0);
        assert_eq!(color, egui::Color32::from_rgb(220, 180, 50));
    }

    #[test]
    fn test_happiness_color_red() {
        let color = happiness_color(20.0);
        assert_eq!(color, egui::Color32::from_rgb(220, 50, 50));
    }

    #[test]
    fn test_need_color_green() {
        let color = need_color(80.0);
        assert_eq!(color, egui::Color32::from_rgb(50, 200, 50));
    }

    #[test]
    fn test_need_color_yellow() {
        let color = need_color(45.0);
        assert_eq!(color, egui::Color32::from_rgb(220, 180, 50));
    }

    #[test]
    fn test_need_color_red() {
        let color = need_color(10.0);
        assert_eq!(color, egui::Color32::from_rgb(220, 50, 50));
    }

    #[test]
    fn test_citizen_name_male() {
        let entity = Entity::from_raw(0);
        let name = citizen_name(entity, Gender::Male);
        assert_eq!(name, "James Smith");
    }

    #[test]
    fn test_citizen_name_female() {
        let entity = Entity::from_raw(0);
        let name = citizen_name(entity, Gender::Female);
        assert_eq!(name, "Mary Smith");
    }

    #[test]
    fn test_citizen_name_different_indices() {
        let e1 = Entity::from_raw(1);
        let e2 = Entity::from_raw(2);
        let n1 = citizen_name(e1, Gender::Male);
        let n2 = citizen_name(e2, Gender::Male);
        assert_ne!(n1, n2);
    }

    #[test]
    fn test_citizen_name_wraps_around() {
        // With 32 first names and 32 last names, index 32 should wrap first name
        let entity = Entity::from_raw(32);
        let name = citizen_name(entity, Gender::Male);
        // Index 32 % 32 = 0 -> "James", (32/31) % 32 = 1 -> "Johnson"
        assert_eq!(name, "James Johnson");
    }

    #[test]
    fn test_selected_citizen_default() {
        let selected = SelectedCitizen::default();
        assert!(selected.0.is_none());
    }

    #[test]
    fn test_follow_citizen_default() {
        let follow = FollowCitizen::default();
        assert!(follow.0.is_none());
    }
}
