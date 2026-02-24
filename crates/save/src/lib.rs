#[cfg(not(target_arch = "wasm32"))]
mod atomic_write;
mod autosave_bridge;
mod despawn;
mod exclusive_load;
mod exclusive_new_game;
mod exclusive_save;
mod file_header;
mod reset_resources;
mod restore_resources;
mod save_codec;
pub mod save_error;
mod save_migrate;
mod save_migrate_registry;
mod save_plugin;
mod save_restore;
pub mod save_stages;
mod save_types;
pub mod saveable_ext;
pub mod serialization;
mod spawn_entities;

#[cfg(target_arch = "wasm32")]
mod wasm_idb;

pub use save_error::SaveError;
pub use save_plugin::{LoadGameEvent, NewGameEvent, SaveGameEvent, SavePlugin};
pub use saveable_ext::SaveableAppExt;
