//! Customizable keybindings resource (UX-035).
//!
//! Provides a `KeyBindings` resource containing all configurable keyboard
//! shortcuts. Systems read from this resource instead of hardcoding `KeyCode`
//! values. A settings UI allows rebinding, with conflict detection and
//! "Reset to Defaults".

mod actions;
mod bindings;
pub(crate) mod key_helpers;

pub use actions::BindableAction;
pub use bindings::{KeyBinding, KeyBindings, KeyBindingsPlugin, RebindState};
pub use key_helpers::keycode_label;
