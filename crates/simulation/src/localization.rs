use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::Saveable;

// =============================================================================
// Constants
// =============================================================================

/// Default locale used when no locale is explicitly set.
pub const DEFAULT_LOCALE: &str = "en";

/// All supported locale codes.
pub const SUPPORTED_LOCALES: &[&str] = &["en", "de", "es", "fr", "ja", "zh"];

/// Human-readable names for each supported locale (same order as SUPPORTED_LOCALES).
pub const LOCALE_NAMES: &[&str] = &[
    "English", "Deutsch", "Espanol", "Francais", "Japanese", "Chinese",
];

// =============================================================================
// String Table
// =============================================================================

/// A string table maps localization keys to their translated text for a single locale.
pub type StringTable = BTreeMap<String, String>;

// =============================================================================
// Resource
// =============================================================================

/// City-wide localization state.
///
/// Holds all string tables (one per locale) and the currently active locale.
/// UI systems read the active locale's string table via `get()` or `t()`.
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct LocalizationState {
    /// Currently active locale code (e.g. "en", "de", "ja").
    pub active_locale: String,
    /// String tables keyed by locale code. Each value is a key->translation map.
    pub tables: BTreeMap<String, StringTable>,
}

impl Default for LocalizationState {
    fn default() -> Self {
        let mut state = Self {
            active_locale: DEFAULT_LOCALE.to_string(),
            tables: BTreeMap::new(),
        };
        state.tables.insert("en".to_string(), build_english_table());
        state.tables.insert("de".to_string(), build_german_table());
        state.tables.insert("es".to_string(), build_spanish_table());
        state.tables.insert("fr".to_string(), build_french_table());
        state
            .tables
            .insert("ja".to_string(), build_japanese_table());
        state.tables.insert("zh".to_string(), build_chinese_table());
        state
    }
}

impl LocalizationState {
    /// Look up a localization key in the active locale's string table.
    /// Returns the translated string, or the key itself as a fallback.
    pub fn t<'a>(&'a self, key: &'a str) -> &'a str {
        self.get(key).unwrap_or(key)
    }

    /// Look up a localization key, returning `None` if not found.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.tables
            .get(&self.active_locale)
            .and_then(|table| table.get(key).map(|s| s.as_str()))
            .or_else(|| {
                // Fallback to English if key not found in active locale
                self.tables
                    .get(DEFAULT_LOCALE)
                    .and_then(|table| table.get(key).map(|s| s.as_str()))
            })
    }

    /// Set the active locale. If the locale is not supported, this is a no-op.
    pub fn set_locale(&mut self, locale: &str) {
        if self.tables.contains_key(locale) {
            self.active_locale = locale.to_string();
        }
    }

    /// Return the display name for the currently active locale.
    pub fn active_locale_name(&self) -> &str {
        for (i, code) in SUPPORTED_LOCALES.iter().enumerate() {
            if *code == self.active_locale {
                return LOCALE_NAMES[i];
            }
        }
        &self.active_locale
    }

    /// Format a number with locale-appropriate thousands separators.
    pub fn format_number(&self, n: i64) -> String {
        let separator = match self.active_locale.as_str() {
            "de" | "fr" | "es" => '.',
            _ => ',',
        };
        format_with_separator(n, separator)
    }

    /// Format a currency amount with locale-appropriate formatting.
    pub fn format_currency(&self, amount: f64) -> String {
        let prefix = match self.active_locale.as_str() {
            "ja" | "zh" => "\u{00a5}",
            "de" | "fr" | "es" => "\u{20ac}",
            _ => "$",
        };
        format!("{}{}", prefix, self.format_number(amount as i64))
    }

    /// Format a date (day number) with locale-appropriate format.
    pub fn format_date(&self, day: u32) -> String {
        // Simple day formatting; game uses day numbers
        let key = "ui.day";
        let day_label = self.t(key);
        format!("{} {}", day_label, self.format_number(day as i64))
    }

    /// Format a percentage with locale-appropriate decimal separator.
    pub fn format_percent(&self, value: f32) -> String {
        let decimal_sep = match self.active_locale.as_str() {
            "de" | "fr" | "es" => ',',
            _ => '.',
        };
        let int_part = value as i64;
        let frac_part = ((value - int_part as f32) * 10.0).abs() as u32;
        format!("{}{}{}", int_part, decimal_sep, frac_part)
    }

    /// Get the list of available locales as (code, display_name) pairs.
    pub fn available_locales(&self) -> Vec<(&'static str, &'static str)> {
        SUPPORTED_LOCALES
            .iter()
            .zip(LOCALE_NAMES.iter())
            .map(|(code, name)| (*code, *name))
            .collect()
    }
}

// =============================================================================
// Saveable implementation
// =============================================================================

impl Saveable for LocalizationState {
    const SAVE_KEY: &'static str = "localization";

    fn save_to_bytes(&self) -> Option<Vec<u8>> {
        // Only save the active locale (tables are always rebuilt from code).
        Some(self.active_locale.as_bytes().to_vec())
    }

    fn load_from_bytes(bytes: &[u8]) -> Self {
        let locale = std::str::from_utf8(bytes).unwrap_or(DEFAULT_LOCALE);
        let mut state = Self::default();
        state.set_locale(locale);
        state
    }
}

// =============================================================================
// Number formatting helper
// =============================================================================

fn format_with_separator(n: i64, sep: char) -> String {
    let negative = n < 0;
    let s = n.unsigned_abs().to_string();
    let bytes = s.as_bytes();
    let len = bytes.len();

    if len <= 3 {
        return if negative { format!("-{}", s) } else { s };
    }

    let mut result = String::with_capacity(len + len / 3);
    if negative {
        result.push('-');
    }

    let first_group = len % 3;
    if first_group > 0 {
        result.push_str(&s[..first_group]);
        if first_group < len {
            result.push(sep);
        }
    }

    let remaining = &s[first_group..];
    for (i, ch) in remaining.chars().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(sep);
        }
        result.push(ch);
    }

    result
}

// =============================================================================
// English string table (complete)
// =============================================================================

fn build_english_table() -> StringTable {
    let entries: &[(&str, &str)] = &[
        // General UI
        ("ui.day", "Day"),
        ("ui.population", "Pop"),
        ("ui.happiness", "Happy"),
        ("ui.treasury", "Treasury"),
        ("ui.save", "Save"),
        ("ui.load", "Load"),
        ("ui.new_game", "New"),
        ("ui.settings", "Settings"),
        ("ui.language", "Language"),
        ("ui.overlay", "Overlay"),
        ("ui.grid_snap", "GRID SNAP"),
        ("ui.demand", "demand"),
        ("ui.surplus", "surplus"),
        ("ui.balanced", "balanced"),
        // Speed controls
        ("ui.speed.pause", "Pause"),
        ("ui.speed.normal", "Normal"),
        ("ui.speed.fast", "Fast"),
        ("ui.speed.fastest", "Fastest"),
        // Tool categories
        ("category.roads", "Roads"),
        ("category.zones", "Zones"),
        ("category.utilities", "Utilities"),
        ("category.emergency", "Emergency"),
        ("category.education", "Education"),
        ("category.parks", "Parks"),
        ("category.landmarks", "Landmarks"),
        ("category.sanitation", "Sanitation"),
        ("category.transport", "Transport"),
        ("category.telecom", "Telecom"),
        ("category.views", "Views"),
        ("category.environment", "Environment"),
        ("category.terrain", "Terrain"),
        ("category.districts", "Districts"),
        ("category.tools", "Tools"),
        // Roads
        ("tool.local_road", "Local Road"),
        ("tool.avenue", "Avenue"),
        ("tool.boulevard", "Boulevard"),
        ("tool.highway", "Highway"),
        ("tool.one_way", "One-Way"),
        ("tool.path", "Path"),
        // Zones
        ("tool.res_low", "Res Low"),
        ("tool.res_medium", "Res Medium"),
        ("tool.res_high", "Res High"),
        ("tool.com_low", "Com Low"),
        ("tool.com_high", "Com High"),
        ("tool.industrial", "Industrial"),
        ("tool.office", "Office"),
        ("tool.mixed_use", "Mixed-Use"),
        // Utilities
        ("tool.power_plant", "Power Plant"),
        ("tool.solar_farm", "Solar Farm"),
        ("tool.wind_turbine", "Wind Turbine"),
        ("tool.nuclear_plant", "Nuclear Plant"),
        ("tool.geothermal", "Geothermal"),
        ("tool.water_tower", "Water Tower"),
        ("tool.sewage_plant", "Sewage Plant"),
        ("tool.pumping_station", "Pumping Station"),
        ("tool.water_treatment", "Water Treatment"),
        // Emergency
        ("tool.fire_house", "Fire House"),
        ("tool.fire_station", "Fire Station"),
        ("tool.fire_hq", "Fire HQ"),
        ("tool.police_kiosk", "Police Kiosk"),
        ("tool.police_station", "Police Station"),
        ("tool.police_hq", "Police HQ"),
        ("tool.prison", "Prison"),
        ("tool.medical_clinic", "Medical Clinic"),
        ("tool.hospital", "Hospital"),
        ("tool.medical_center", "Medical Center"),
        // Education
        ("tool.kindergarten", "Kindergarten"),
        ("tool.elementary", "Elementary"),
        ("tool.high_school", "High School"),
        ("tool.university", "University"),
        ("tool.library", "Library"),
        // Parks
        ("tool.small_park", "Small Park"),
        ("tool.large_park", "Large Park"),
        ("tool.playground", "Playground"),
        ("tool.plaza", "Plaza"),
        ("tool.sports_field", "Sports Field"),
        ("tool.stadium", "Stadium"),
        // Landmarks
        ("tool.city_hall", "City Hall"),
        ("tool.museum", "Museum"),
        ("tool.cathedral", "Cathedral"),
        ("tool.tv_station", "TV Station"),
        // Sanitation
        ("tool.landfill", "Landfill"),
        ("tool.recycling_center", "Recycling Center"),
        ("tool.incinerator", "Incinerator"),
        ("tool.transfer_station", "Transfer Station"),
        ("tool.cemetery", "Cemetery"),
        ("tool.crematorium", "Crematorium"),
        // Transport
        ("tool.bus_depot", "Bus Depot"),
        ("tool.train_station", "Train Station"),
        ("tool.subway", "Subway"),
        ("tool.tram_depot", "Tram Depot"),
        ("tool.ferry_pier", "Ferry Pier"),
        ("tool.small_airstrip", "Small Airstrip"),
        ("tool.regional_airport", "Regional Airport"),
        ("tool.intl_airport", "Int'l Airport"),
        // Telecom
        ("tool.cell_tower", "Cell Tower"),
        ("tool.data_center", "Data Center"),
        // Views/Overlays
        ("overlay.power", "Power"),
        ("overlay.water", "Water"),
        ("overlay.traffic", "Traffic"),
        ("overlay.pollution", "Pollution"),
        ("overlay.land_value", "Land Value"),
        ("overlay.education", "Education"),
        ("overlay.garbage", "Garbage"),
        ("overlay.noise", "Noise"),
        ("overlay.water_pollution", "Water Pollution"),
        ("overlay.gw_level", "GW Level"),
        ("overlay.gw_quality", "GW Quality"),
        // Environment
        ("tool.plant_tree", "Plant Tree"),
        ("tool.remove_tree", "Remove Tree"),
        // Terrain
        ("tool.raise", "Raise"),
        ("tool.lower", "Lower"),
        ("tool.flatten", "Flatten"),
        ("tool.water", "Water"),
        // Districts
        ("tool.downtown", "Downtown"),
        ("tool.suburbs", "Suburbs"),
        ("tool.waterfront", "Waterfront"),
        ("tool.historic", "Historic"),
        ("tool.arts", "Arts"),
        ("tool.tech_park", "Tech Park"),
        ("tool.erase_district", "Erase District"),
        // Tools
        ("tool.bulldoze", "Bulldoze"),
        ("tool.inspect", "Inspect"),
        // Milestone names
        ("milestone.settlement", "Settlement"),
        ("milestone.village", "Village"),
        ("milestone.hamlet", "Hamlet"),
        ("milestone.town", "Town"),
        ("milestone.small_city", "Small City"),
        ("milestone.city", "City"),
        ("milestone.large_city", "Large City"),
        ("milestone.metropolis", "Metropolis"),
        ("milestone.major_metropolis", "Major Metropolis"),
        ("milestone.megacity", "Megacity"),
        ("milestone.megalopolis", "Megalopolis"),
        ("milestone.world_capital", "World Capital"),
        // Info panel sections
        ("panel.statistics", "Statistics"),
        ("panel.budget", "Budget"),
        ("panel.policies", "Policies"),
        ("panel.charts", "Charts"),
        ("panel.journal", "Journal"),
        ("panel.advisor", "Advisor"),
        // Seasons
        ("season.spring", "Spring"),
        ("season.summer", "Summer"),
        ("season.autumn", "Autumn"),
        ("season.winter", "Winter"),
    ];

    entries
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

// =============================================================================
// German string table
// =============================================================================

fn build_german_table() -> StringTable {
    let entries: &[(&str, &str)] = &[
        ("ui.day", "Tag"),
        ("ui.population", "Bev"),
        ("ui.happiness", "Zufrieden"),
        ("ui.treasury", "Kasse"),
        ("ui.save", "Speichern"),
        ("ui.load", "Laden"),
        ("ui.new_game", "Neu"),
        ("ui.settings", "Einstellungen"),
        ("ui.language", "Sprache"),
        ("ui.overlay", "Overlay"),
        ("ui.grid_snap", "RASTER"),
        ("ui.demand", "Bedarf"),
        ("ui.surplus", "Ueberschuss"),
        ("ui.balanced", "Ausgeglichen"),
        ("ui.speed.pause", "Pause"),
        ("ui.speed.normal", "Normal"),
        ("ui.speed.fast", "Schnell"),
        ("ui.speed.fastest", "Sehr schnell"),
        ("category.roads", "Strassen"),
        ("category.zones", "Zonen"),
        ("category.utilities", "Versorgung"),
        ("category.emergency", "Notdienste"),
        ("category.education", "Bildung"),
        ("category.parks", "Parks"),
        ("category.landmarks", "Wahrzeichen"),
        ("category.sanitation", "Entsorgung"),
        ("category.transport", "Transport"),
        ("category.telecom", "Telekom"),
        ("category.views", "Ansichten"),
        ("category.environment", "Umwelt"),
        ("category.terrain", "Gelaende"),
        ("category.districts", "Bezirke"),
        ("category.tools", "Werkzeuge"),
        ("tool.local_road", "Nebenstrasse"),
        ("tool.avenue", "Allee"),
        ("tool.boulevard", "Boulevard"),
        ("tool.highway", "Autobahn"),
        ("tool.one_way", "Einbahnstrasse"),
        ("tool.path", "Weg"),
        ("tool.bulldoze", "Abreissen"),
        ("tool.inspect", "Inspizieren"),
        ("milestone.settlement", "Siedlung"),
        ("milestone.village", "Dorf"),
        ("milestone.hamlet", "Weiler"),
        ("milestone.town", "Kleinstadt"),
        ("milestone.small_city", "Stadt"),
        ("milestone.city", "Grossstadt"),
        ("milestone.large_city", "Grosse Stadt"),
        ("milestone.metropolis", "Metropole"),
        ("milestone.major_metropolis", "Grossmetropole"),
        ("milestone.megacity", "Megastadt"),
        ("milestone.megalopolis", "Megalopolis"),
        ("milestone.world_capital", "Welthauptstadt"),
        ("panel.statistics", "Statistiken"),
        ("panel.budget", "Haushalt"),
        ("panel.policies", "Politik"),
        ("panel.charts", "Diagramme"),
        ("panel.journal", "Journal"),
        ("panel.advisor", "Berater"),
        ("season.spring", "Fruehling"),
        ("season.summer", "Sommer"),
        ("season.autumn", "Herbst"),
        ("season.winter", "Winter"),
    ];

    entries
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

// =============================================================================
// Spanish string table
// =============================================================================

fn build_spanish_table() -> StringTable {
    let entries: &[(&str, &str)] = &[
        ("ui.day", "Dia"),
        ("ui.population", "Pob"),
        ("ui.happiness", "Felicidad"),
        ("ui.treasury", "Tesoro"),
        ("ui.save", "Guardar"),
        ("ui.load", "Cargar"),
        ("ui.new_game", "Nuevo"),
        ("ui.settings", "Ajustes"),
        ("ui.language", "Idioma"),
        ("ui.overlay", "Capa"),
        ("ui.grid_snap", "REJILLA"),
        ("ui.demand", "demanda"),
        ("ui.surplus", "excedente"),
        ("ui.balanced", "equilibrado"),
        ("ui.speed.pause", "Pausa"),
        ("ui.speed.normal", "Normal"),
        ("ui.speed.fast", "Rapido"),
        ("ui.speed.fastest", "Muy rapido"),
        ("category.roads", "Carreteras"),
        ("category.zones", "Zonas"),
        ("category.utilities", "Servicios"),
        ("category.emergency", "Emergencia"),
        ("category.education", "Educacion"),
        ("category.parks", "Parques"),
        ("category.landmarks", "Monumentos"),
        ("category.sanitation", "Saneamiento"),
        ("category.transport", "Transporte"),
        ("category.telecom", "Telecom"),
        ("category.views", "Vistas"),
        ("category.environment", "Medio Ambiente"),
        ("category.terrain", "Terreno"),
        ("category.districts", "Distritos"),
        ("category.tools", "Herramientas"),
        ("tool.bulldoze", "Demoler"),
        ("tool.inspect", "Inspeccionar"),
        ("milestone.settlement", "Asentamiento"),
        ("milestone.village", "Pueblo"),
        ("milestone.hamlet", "Aldea"),
        ("milestone.town", "Villa"),
        ("milestone.small_city", "Ciudad pequena"),
        ("milestone.city", "Ciudad"),
        ("milestone.large_city", "Gran ciudad"),
        ("milestone.metropolis", "Metropolis"),
        ("milestone.major_metropolis", "Gran metropolis"),
        ("milestone.megacity", "Megaciudad"),
        ("milestone.megalopolis", "Megalopolis"),
        ("milestone.world_capital", "Capital mundial"),
        ("panel.statistics", "Estadisticas"),
        ("panel.budget", "Presupuesto"),
        ("panel.policies", "Politicas"),
        ("panel.charts", "Graficos"),
        ("panel.journal", "Diario"),
        ("panel.advisor", "Asesor"),
        ("season.spring", "Primavera"),
        ("season.summer", "Verano"),
        ("season.autumn", "Otono"),
        ("season.winter", "Invierno"),
    ];

    entries
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

// =============================================================================
// French string table
// =============================================================================

fn build_french_table() -> StringTable {
    let entries: &[(&str, &str)] = &[
        ("ui.day", "Jour"),
        ("ui.population", "Pop"),
        ("ui.happiness", "Bonheur"),
        ("ui.treasury", "Tresor"),
        ("ui.save", "Sauvegarder"),
        ("ui.load", "Charger"),
        ("ui.new_game", "Nouveau"),
        ("ui.settings", "Parametres"),
        ("ui.language", "Langue"),
        ("ui.overlay", "Calque"),
        ("ui.grid_snap", "GRILLE"),
        ("ui.demand", "demande"),
        ("ui.surplus", "surplus"),
        ("ui.balanced", "equilibre"),
        ("ui.speed.pause", "Pause"),
        ("ui.speed.normal", "Normal"),
        ("ui.speed.fast", "Rapide"),
        ("ui.speed.fastest", "Tres rapide"),
        ("category.roads", "Routes"),
        ("category.zones", "Zones"),
        ("category.utilities", "Services"),
        ("category.emergency", "Urgences"),
        ("category.education", "Education"),
        ("category.parks", "Parcs"),
        ("category.landmarks", "Monuments"),
        ("category.sanitation", "Assainissement"),
        ("category.transport", "Transport"),
        ("category.telecom", "Telecom"),
        ("category.views", "Vues"),
        ("category.environment", "Environnement"),
        ("category.terrain", "Terrain"),
        ("category.districts", "Quartiers"),
        ("category.tools", "Outils"),
        ("tool.bulldoze", "Demolir"),
        ("tool.inspect", "Inspecter"),
        ("milestone.settlement", "Campement"),
        ("milestone.village", "Village"),
        ("milestone.hamlet", "Hameau"),
        ("milestone.town", "Bourg"),
        ("milestone.small_city", "Petite ville"),
        ("milestone.city", "Ville"),
        ("milestone.large_city", "Grande ville"),
        ("milestone.metropolis", "Metropole"),
        ("milestone.major_metropolis", "Grande metropole"),
        ("milestone.megacity", "Megapole"),
        ("milestone.megalopolis", "Megalopole"),
        ("milestone.world_capital", "Capitale mondiale"),
        ("panel.statistics", "Statistiques"),
        ("panel.budget", "Budget"),
        ("panel.policies", "Politiques"),
        ("panel.charts", "Graphiques"),
        ("panel.journal", "Journal"),
        ("panel.advisor", "Conseiller"),
        ("season.spring", "Printemps"),
        ("season.summer", "Ete"),
        ("season.autumn", "Automne"),
        ("season.winter", "Hiver"),
    ];

    entries
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

// =============================================================================
// Japanese string table
// =============================================================================

fn build_japanese_table() -> StringTable {
    let entries: &[(&str, &str)] = &[
        ("ui.day", "\u{65e5}"),
        ("ui.population", "\u{4eba}\u{53e3}"),
        ("ui.happiness", "\u{5e78}\u{798f}\u{5ea6}"),
        ("ui.treasury", "\u{8ca1}\u{653f}"),
        ("ui.save", "\u{4fdd}\u{5b58}"),
        ("ui.load", "\u{8aad}\u{8fbc}"),
        ("ui.new_game", "\u{65b0}\u{898f}"),
        ("ui.settings", "\u{8a2d}\u{5b9a}"),
        ("ui.language", "\u{8a00}\u{8a9e}"),
        (
            "ui.overlay",
            "\u{30aa}\u{30fc}\u{30d0}\u{30fc}\u{30ec}\u{30a4}",
        ),
        ("ui.grid_snap", "\u{30b0}\u{30ea}\u{30c3}\u{30c9}"),
        ("ui.demand", "\u{9700}\u{8981}"),
        ("ui.surplus", "\u{4f59}\u{5270}"),
        ("ui.balanced", "\u{5747}\u{8861}"),
        ("category.roads", "\u{9053}\u{8def}"),
        ("category.zones", "\u{30be}\u{30fc}\u{30f3}"),
        (
            "category.utilities",
            "\u{30e6}\u{30fc}\u{30c6}\u{30a3}\u{30ea}\u{30c6}\u{30a3}",
        ),
        ("category.emergency", "\u{7dca}\u{6025}"),
        ("category.education", "\u{6559}\u{80b2}"),
        ("category.parks", "\u{516c}\u{5712}"),
        ("category.tools", "\u{30c4}\u{30fc}\u{30eb}"),
        ("milestone.settlement", "\u{96c6}\u{843d}"),
        ("milestone.village", "\u{6751}"),
        ("milestone.town", "\u{753a}"),
        ("milestone.city", "\u{5e02}"),
        ("milestone.megacity", "\u{5de8}\u{5927}\u{90fd}\u{5e02}"),
        ("season.spring", "\u{6625}"),
        ("season.summer", "\u{590f}"),
        ("season.autumn", "\u{79cb}"),
        ("season.winter", "\u{51ac}"),
    ];

    entries
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

// =============================================================================
// Chinese string table
// =============================================================================

fn build_chinese_table() -> StringTable {
    let entries: &[(&str, &str)] = &[
        ("ui.day", "\u{5929}"),
        ("ui.population", "\u{4eba}\u{53e3}"),
        ("ui.happiness", "\u{5e78}\u{798f}"),
        ("ui.treasury", "\u{8d22}\u{653f}"),
        ("ui.save", "\u{4fdd}\u{5b58}"),
        ("ui.load", "\u{52a0}\u{8f7d}"),
        ("ui.new_game", "\u{65b0}\u{6e38}\u{620f}"),
        ("ui.settings", "\u{8bbe}\u{7f6e}"),
        ("ui.language", "\u{8bed}\u{8a00}"),
        ("ui.overlay", "\u{53e0}\u{52a0}\u{5c42}"),
        ("ui.grid_snap", "\u{7f51}\u{683c}"),
        ("ui.demand", "\u{9700}\u{6c42}"),
        ("ui.surplus", "\u{76c8}\u{4f59}"),
        ("ui.balanced", "\u{5e73}\u{8861}"),
        ("category.roads", "\u{9053}\u{8def}"),
        ("category.zones", "\u{533a}\u{57df}"),
        ("category.utilities", "\u{516c}\u{7528}\u{4e8b}\u{4e1a}"),
        ("category.emergency", "\u{7d27}\u{6025}"),
        ("category.education", "\u{6559}\u{80b2}"),
        ("category.parks", "\u{516c}\u{56ed}"),
        ("category.tools", "\u{5de5}\u{5177}"),
        ("milestone.settlement", "\u{5b9a}\u{5c45}\u{70b9}"),
        ("milestone.village", "\u{6751}\u{5e84}"),
        ("milestone.town", "\u{5c0f}\u{9547}"),
        ("milestone.city", "\u{57ce}\u{5e02}"),
        ("milestone.megacity", "\u{5de8}\u{578b}\u{57ce}\u{5e02}"),
        ("season.spring", "\u{6625}"),
        ("season.summer", "\u{590f}"),
        ("season.autumn", "\u{79cb}"),
        ("season.winter", "\u{51ac}"),
    ];

    entries
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

// =============================================================================
// Plugin
// =============================================================================

pub struct LocalizationPlugin;

impl Plugin for LocalizationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LocalizationState>();
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(format_with_separator(100, ','), "100");
    }

    #[test]
    fn test_format_with_separator_four_digits() {
        assert_eq!(format_with_separator(1000, ','), "1,000");
    }

    #[test]
    fn test_format_with_separator_seven_digits() {
        assert_eq!(format_with_separator(1234567, '.'), "1.234.567");
    }

    #[test]
    fn test_format_with_separator_negative_four_digits() {
        assert_eq!(format_with_separator(-1000, ','), "-1,000");
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
