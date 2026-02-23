#[cfg(test)]
mod tests {
    use crate::localization::*;
    use crate::Saveable;

    // -------------------------------------------------------------------------
    // Default state tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_default_locale_is_english() {
        let state = LocalizationState::default();
        assert_eq!(state.active_locale, "en");
    }

    #[test]
    fn test_default_has_all_supported_locales() {
        let state = LocalizationState::default();
        for locale in SUPPORTED_LOCALES {
            assert!(
                state.tables.contains_key(*locale),
                "Missing locale: {}",
                locale
            );
        }
    }

    #[test]
    fn test_english_table_not_empty() {
        let state = LocalizationState::default();
        let en_table = state.tables.get("en").unwrap();
        assert!(!en_table.is_empty());
    }

    // -------------------------------------------------------------------------
    // Translation lookup tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_t_returns_english_translation() {
        let state = LocalizationState::default();
        assert_eq!(state.t("ui.save"), "Save");
    }

    #[test]
    fn test_t_returns_key_for_missing() {
        let state = LocalizationState::default();
        assert_eq!(state.t("nonexistent.key"), "nonexistent.key");
    }

    #[test]
    fn test_get_returns_none_for_missing() {
        let state = LocalizationState::default();
        assert!(state.get("nonexistent.key").is_none());
    }

    #[test]
    fn test_get_returns_some_for_existing() {
        let state = LocalizationState::default();
        assert_eq!(state.get("ui.save"), Some("Save"));
    }

    #[test]
    fn test_german_translation() {
        let mut state = LocalizationState::default();
        state.set_locale("de");
        assert_eq!(state.t("ui.save"), "Speichern");
    }

    #[test]
    fn test_spanish_translation() {
        let mut state = LocalizationState::default();
        state.set_locale("es");
        assert_eq!(state.t("ui.save"), "Guardar");
    }

    #[test]
    fn test_french_translation() {
        let mut state = LocalizationState::default();
        state.set_locale("fr");
        assert_eq!(state.t("ui.save"), "Sauvegarder");
    }

    #[test]
    fn test_fallback_to_english_for_missing_key_in_other_locale() {
        let mut state = LocalizationState::default();
        state.set_locale("de");
        // "tool.power_plant" may not be in the German table; should fall back to English.
        let result = state.t("tool.power_plant");
        assert_eq!(result, "Power Plant");
    }

    // -------------------------------------------------------------------------
    // Locale switching tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_set_locale_valid() {
        let mut state = LocalizationState::default();
        state.set_locale("de");
        assert_eq!(state.active_locale, "de");
    }

    #[test]
    fn test_set_locale_invalid_is_noop() {
        let mut state = LocalizationState::default();
        state.set_locale("xx");
        assert_eq!(state.active_locale, "en");
    }

    #[test]
    fn test_active_locale_name_english() {
        let state = LocalizationState::default();
        assert_eq!(state.active_locale_name(), "English");
    }

    #[test]
    fn test_active_locale_name_german() {
        let mut state = LocalizationState::default();
        state.set_locale("de");
        assert_eq!(state.active_locale_name(), "Deutsch");
    }

    // -------------------------------------------------------------------------
    // Number formatting tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_format_number_english_small() {
        let state = LocalizationState::default();
        assert_eq!(state.format_number(42), "42");
    }

    #[test]
    fn test_format_number_english_thousands() {
        let state = LocalizationState::default();
        assert_eq!(state.format_number(1234), "1,234");
    }

    #[test]
    fn test_format_number_english_millions() {
        let state = LocalizationState::default();
        assert_eq!(state.format_number(1234567), "1,234,567");
    }

    #[test]
    fn test_format_number_german_dot_separator() {
        let mut state = LocalizationState::default();
        state.set_locale("de");
        assert_eq!(state.format_number(1234567), "1.234.567");
    }

    #[test]
    fn test_format_number_negative() {
        let state = LocalizationState::default();
        assert_eq!(state.format_number(-5000), "-5,000");
    }

    #[test]
    fn test_format_number_zero() {
        let state = LocalizationState::default();
        assert_eq!(state.format_number(0), "0");
    }

    // -------------------------------------------------------------------------
    // Currency formatting tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_format_currency_english() {
        let state = LocalizationState::default();
        assert_eq!(state.format_currency(50000.0), "$50,000");
    }

    #[test]
    fn test_format_currency_german_euro() {
        let mut state = LocalizationState::default();
        state.set_locale("de");
        let result = state.format_currency(50000.0);
        assert!(result.starts_with('\u{20ac}'));
    }

    #[test]
    fn test_format_currency_japanese_yen() {
        let mut state = LocalizationState::default();
        state.set_locale("ja");
        let result = state.format_currency(50000.0);
        assert!(result.starts_with('\u{00a5}'));
    }

    // -------------------------------------------------------------------------
    // Percentage formatting tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_format_percent_english() {
        let state = LocalizationState::default();
        assert_eq!(state.format_percent(85.5), "85.5");
    }

    #[test]
    fn test_format_percent_german_comma() {
        let mut state = LocalizationState::default();
        state.set_locale("de");
        assert_eq!(state.format_percent(85.5), "85,5");
    }

    // -------------------------------------------------------------------------
    // Date formatting tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_format_date_english() {
        let state = LocalizationState::default();
        assert_eq!(state.format_date(42), "Day 42");
    }

    #[test]
    fn test_format_date_german() {
        let mut state = LocalizationState::default();
        state.set_locale("de");
        assert_eq!(state.format_date(42), "Tag 42");
    }

    // -------------------------------------------------------------------------
    // Available locales test
    // -------------------------------------------------------------------------

    #[test]
    fn test_available_locales() {
        let state = LocalizationState::default();
        let locales = state.available_locales();
        assert_eq!(locales.len(), SUPPORTED_LOCALES.len());
        assert_eq!(locales[0], ("en", "English"));
    }

    // -------------------------------------------------------------------------
    // Saveable tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_save_key() {
        assert_eq!(LocalizationState::SAVE_KEY, "localization");
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let mut state = LocalizationState::default();
        state.set_locale("de");

        let bytes = state.save_to_bytes().expect("should serialize");
        let restored = LocalizationState::load_from_bytes(&bytes);

        assert_eq!(restored.active_locale, "de");
        // Tables should be rebuilt from code, so English keys must still exist.
        assert!(restored.tables.contains_key("en"));
        assert!(restored.tables.contains_key("de"));
    }

    #[test]
    fn test_load_invalid_locale_falls_back_to_english() {
        let restored = LocalizationState::load_from_bytes(b"xx");
        assert_eq!(restored.active_locale, "en");
    }

    // -------------------------------------------------------------------------
    // Number formatter edge cases
    // -------------------------------------------------------------------------

    #[test]
    fn test_format_with_separator_exact_three_digits() {
        assert_eq!(crate::localization::format_with_separator(100, ','), "100");
    }

    #[test]
    fn test_format_with_separator_four_digits() {
        assert_eq!(
            crate::localization::format_with_separator(1000, ','),
            "1,000"
        );
    }

    #[test]
    fn test_format_with_separator_seven_digits() {
        assert_eq!(
            crate::localization::format_with_separator(1234567, '.'),
            "1.234.567"
        );
    }

    #[test]
    fn test_format_with_separator_negative_four_digits() {
        assert_eq!(
            crate::localization::format_with_separator(-1000, ','),
            "-1,000"
        );
    }

    // -------------------------------------------------------------------------
    // String table coverage tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_all_category_keys_exist_in_english() {
        let state = LocalizationState::default();
        let en = state.tables.get("en").unwrap();

        let expected_keys = &[
            "category.roads",
            "category.zones",
            "category.utilities",
            "category.emergency",
            "category.education",
            "category.parks",
            "category.landmarks",
            "category.sanitation",
            "category.transport",
            "category.telecom",
            "category.views",
            "category.environment",
            "category.terrain",
            "category.districts",
            "category.tools",
        ];

        for key in expected_keys {
            assert!(en.contains_key(*key), "Missing English key: {}", key);
        }
    }

    #[test]
    fn test_all_milestone_keys_exist_in_english() {
        let state = LocalizationState::default();
        let en = state.tables.get("en").unwrap();

        let expected_keys = &[
            "milestone.settlement",
            "milestone.village",
            "milestone.hamlet",
            "milestone.town",
            "milestone.small_city",
            "milestone.city",
            "milestone.large_city",
            "milestone.metropolis",
            "milestone.major_metropolis",
            "milestone.megacity",
            "milestone.megalopolis",
            "milestone.world_capital",
        ];

        for key in expected_keys {
            assert!(en.contains_key(*key), "Missing English key: {}", key);
        }
    }

    #[test]
    fn test_all_season_keys_exist_in_english() {
        let state = LocalizationState::default();
        let en = state.tables.get("en").unwrap();

        for season in &[
            "season.spring",
            "season.summer",
            "season.autumn",
            "season.winter",
        ] {
            assert!(en.contains_key(*season), "Missing English key: {}", season);
        }
    }

    #[test]
    fn test_supported_locales_and_names_same_length() {
        assert_eq!(SUPPORTED_LOCALES.len(), LOCALE_NAMES.len());
    }

    #[test]
    fn test_english_table_has_ui_keys() {
        let state = LocalizationState::default();
        let en = state.tables.get("en").unwrap();

        for key in &[
            "ui.save",
            "ui.load",
            "ui.new_game",
            "ui.population",
            "ui.happiness",
            "ui.treasury",
            "ui.settings",
            "ui.language",
        ] {
            assert!(en.contains_key(*key), "Missing English key: {}", key);
        }
    }
}
