mod tables_en;
mod tables_other;
#[cfg(test)]
mod tests;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::{Saveable, SaveableRegistry};

use tables_en::build_english_table;
use tables_other::{
    build_chinese_table, build_french_table, build_german_table, build_japanese_table,
    build_spanish_table,
};

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
        let locale = match std::str::from_utf8(bytes) {
            Ok(s) => s,
            Err(e) => {
                warn!(
                    "Saveable {}: failed to decode locale from {} bytes, using default: {}",
                    Self::SAVE_KEY,
                    bytes.len(),
                    e
                );
                DEFAULT_LOCALE
            }
        };
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
// Plugin
// =============================================================================

pub struct LocalizationPlugin;

impl Plugin for LocalizationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LocalizationState>();
        app.init_resource::<SaveableRegistry>();
        app.world_mut()
            .resource_mut::<SaveableRegistry>()
            .register::<LocalizationState>();
    }
}
