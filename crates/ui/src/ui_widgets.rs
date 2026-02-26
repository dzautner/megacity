//! Reusable themed widget helpers for Megacity UI.
//!
//! These helpers wrap common egui patterns (windows, buttons, headers,
//! stat rows, progress bars) with consistent styling from [`crate::theme`].
//! Use them in new UI panels to maintain visual consistency without
//! manually repeating color/spacing constants.

use bevy_egui::egui;

use crate::theme;

// =============================================================================
// Themed Window
// =============================================================================

/// Create a standard Megacity-styled window.
///
/// Applies consistent default width and styling. Returns the
/// `Option<InnerResponse<R>>` from `egui::Window::show` so callers can
/// check whether the window was rendered.
///
/// # Example
/// ```ignore
/// themed_window("Budget", ctx, |ui| {
///     ui.label("Hello");
/// });
/// ```
pub fn themed_window<R>(
    title: &str,
    ctx: &egui::Context,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> Option<egui::InnerResponse<Option<R>>> {
    egui::Window::new(title)
        .default_width(280.0)
        .resizable(true)
        .show(ctx, add_contents)
}

/// Create a standard Megacity-styled window with an open/close boolean.
///
/// Like [`themed_window`] but the window can be closed via the X button,
/// toggling `open` to `false`.
pub fn themed_window_closeable<R>(
    title: &str,
    ctx: &egui::Context,
    open: &mut bool,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> Option<egui::InnerResponse<Option<R>>> {
    egui::Window::new(title)
        .open(open)
        .default_width(280.0)
        .resizable(true)
        .show(ctx, add_contents)
}

// =============================================================================
// Themed Button
// =============================================================================

/// A standard button with consistent theme styling.
///
/// Returns the `egui::Response` so callers can check `.clicked()`.
pub fn themed_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let button = egui::Button::new(
        egui::RichText::new(text).size(theme::FONT_BODY),
    )
    .corner_radius(egui::CornerRadius::same(theme::WIDGET_CORNER_RADIUS));
    ui.add(button)
}

/// A primary (highlighted) button for the main action in a panel.
pub fn themed_button_primary(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let button = egui::Button::new(
        egui::RichText::new(text)
            .size(theme::FONT_BODY)
            .color(theme::TEXT_HEADING),
    )
    .fill(theme::PRIMARY)
    .corner_radius(egui::CornerRadius::same(theme::WIDGET_CORNER_RADIUS));
    ui.add(button)
}

// =============================================================================
// Themed Headers
// =============================================================================

/// Render a section heading with consistent font size and color.
pub fn themed_heading(ui: &mut egui::Ui, text: &str) {
    ui.label(
        egui::RichText::new(text)
            .size(theme::FONT_HEADING)
            .color(theme::TEXT_HEADING)
            .strong(),
    );
}

/// Render a sub-section heading.
pub fn themed_subheading(ui: &mut egui::Ui, text: &str) {
    ui.label(
        egui::RichText::new(text)
            .size(theme::FONT_SUBHEADING)
            .color(theme::TEXT_HEADING),
    );
}

// =============================================================================
// Stat Row
// =============================================================================

/// A label-value row commonly used in info panels.
///
/// Renders as `label:  value` in a horizontal layout with the label in
/// muted text and the value in normal text.
pub fn stat_row(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!("{}:", label))
                .size(theme::FONT_BODY)
                .color(theme::TEXT_MUTED),
        );
        ui.label(
            egui::RichText::new(value)
                .size(theme::FONT_BODY)
                .color(theme::TEXT),
        );
    });
}

/// A label-value row where the value is colored (e.g., green for positive,
/// red for negative).
pub fn stat_row_colored(
    ui: &mut egui::Ui,
    label: &str,
    value: &str,
    color: egui::Color32,
) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!("{}:", label))
                .size(theme::FONT_BODY)
                .color(theme::TEXT_MUTED),
        );
        ui.label(
            egui::RichText::new(value)
                .size(theme::FONT_BODY)
                .color(color),
        );
    });
}

// =============================================================================
// Progress Bar
// =============================================================================

/// A themed progress bar with an optional custom fill color.
///
/// `fraction` should be in the range `0.0..=1.0`. If `color` is `None`,
/// the primary accent color is used.
pub fn progress_bar(
    ui: &mut egui::Ui,
    fraction: f32,
    color: Option<egui::Color32>,
) -> egui::Response {
    let fill = color.unwrap_or(theme::PRIMARY);
    ui.add(
        egui::ProgressBar::new(fraction.clamp(0.0, 1.0))
            .fill(fill)
            .desired_width(ui.available_width().min(200.0)),
    )
}

/// A themed progress bar with a text overlay.
pub fn progress_bar_with_text(
    ui: &mut egui::Ui,
    fraction: f32,
    text: &str,
    color: Option<egui::Color32>,
) -> egui::Response {
    let fill = color.unwrap_or(theme::PRIMARY);
    ui.add(
        egui::ProgressBar::new(fraction.clamp(0.0, 1.0))
            .fill(fill)
            .text(text)
            .desired_width(ui.available_width().min(200.0)),
    )
}

// =============================================================================
// Section helpers
// =============================================================================

/// Add a separator with consistent spacing above and below.
pub fn section_separator(ui: &mut egui::Ui) {
    ui.add_space(theme::ITEM_SPACING);
    ui.separator();
    ui.add_space(theme::ITEM_SPACING);
}

/// Render a muted caption / small text line.
pub fn caption(ui: &mut egui::Ui, text: &str) {
    ui.label(
        egui::RichText::new(text)
            .size(theme::FONT_SMALL)
            .color(theme::TEXT_MUTED),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_bar_fraction_clamped() {
        // Verify the clamping logic works for out-of-range values.
        let clamped_low = (-0.5_f32).clamp(0.0, 1.0);
        assert!((clamped_low - 0.0).abs() < f32::EPSILON);

        let clamped_high = (1.5_f32).clamp(0.0, 1.0);
        assert!((clamped_high - 1.0).abs() < f32::EPSILON);

        let clamped_mid = (0.5_f32).clamp(0.0, 1.0);
        assert!((clamped_mid - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_theme_colors_are_distinct() {
        // Ensure key colors are not accidentally the same.
        assert_ne!(theme::PRIMARY, theme::SECONDARY);
        assert_ne!(theme::SUCCESS, theme::ERROR);
        assert_ne!(theme::BG_DARK, theme::BG_PANEL);
        assert_ne!(theme::TEXT, theme::TEXT_MUTED);
    }
}
