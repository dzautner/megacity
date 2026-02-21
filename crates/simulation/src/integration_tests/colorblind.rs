use crate::test_harness::TestCity;

#[test]
fn test_colorblind_mode_default_is_normal() {
    let city = TestCity::new();
    let settings = city.resource::<crate::colorblind::ColorblindSettings>();
    assert_eq!(
        settings.mode,
        crate::colorblind::ColorblindMode::Normal,
        "default colorblind mode should be Normal"
    );
}

#[test]
fn test_colorblind_mode_persists_across_ticks() {
    let mut city = TestCity::new();
    city.world_mut()
        .resource_mut::<crate::colorblind::ColorblindSettings>()
        .mode = crate::colorblind::ColorblindMode::Protanopia;
    city.tick(10);
    let settings = city.resource::<crate::colorblind::ColorblindSettings>();
    assert_eq!(
        settings.mode,
        crate::colorblind::ColorblindMode::Protanopia,
        "colorblind mode should persist across ticks"
    );
}

#[test]
fn test_colorblind_settings_saveable() {
    use crate::colorblind::{ColorblindMode, ColorblindSettings};
    use crate::Saveable;

    // Default should not save
    let default_settings = ColorblindSettings::default();
    assert!(
        default_settings.save_to_bytes().is_none(),
        "default settings should skip save"
    );

    // Non-default should save and restore
    let settings = ColorblindSettings {
        mode: ColorblindMode::Deuteranopia,
    };
    let bytes = settings.save_to_bytes().expect("should save non-default");
    let restored = ColorblindSettings::load_from_bytes(&bytes);
    assert_eq!(restored.mode, ColorblindMode::Deuteranopia);
}
