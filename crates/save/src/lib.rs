#[cfg(not(target_arch = "wasm32"))]
mod atomic_write;
mod despawn;
mod exclusive_load;
mod exclusive_new_game;
mod exclusive_save;
mod reset_resources;
mod restore_resources;
mod save_codec;
mod save_migrate;
mod save_plugin;
mod save_restore;
pub mod save_stages;
mod save_types;
pub mod saveable_ext;
pub mod serialization;
mod spawn_entities;

#[cfg(target_arch = "wasm32")]
mod wasm_idb;

pub use save_plugin::{LoadGameEvent, NewGameEvent, SaveGameEvent, SavePlugin};
pub use saveable_ext::SaveableAppExt;
