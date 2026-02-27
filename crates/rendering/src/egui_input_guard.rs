//! Egui input guard: prevents click-through from UI elements to the world.
//!
//! When egui (toolbar, panels, menus) is handling pointer input, world-level
//! input systems should skip processing to avoid unintended road placement,
//! zone painting, or building placement underneath the UI.

use bevy_egui::EguiContexts;

/// Returns `true` when egui wants the pointer — i.e. the cursor is over an
/// egui panel or egui is actively handling a drag/click. Input systems should
/// early-return when this is `true`.
#[inline]
pub fn egui_wants_pointer(contexts: &mut EguiContexts) -> bool {
    let ctx = contexts.ctx_mut();
    ctx.wants_pointer_input() || ctx.is_pointer_over_area()
}
