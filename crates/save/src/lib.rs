#[cfg(not(target_arch = "wasm32"))]
mod atomic_write;
mod autosave_bridge;
mod crash_recovery;
mod despawn;
mod exclusive_load;
mod exclusive_new_game;
mod exclusive_save;
mod file_header;
mod reset_resources;
mod restore_resources;
mod save_codec;
pub mod save_error;
pub mod save_metadata;
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

#[cfg(test)]
mod save_fuzz_mutation_tests;
#[cfg(test)]
mod save_fuzz_tests;

pub use crash_recovery::CrashRecoveryState;
pub use file_header::read_metadata_only;
pub use save_error::SaveError;
pub use save_metadata::SaveMetadata;
pub use save_plugin::{LoadGameEvent, NewGameEvent, PendingSavePath, SaveGameEvent, SavePlugin};
pub use saveable_ext::SaveableAppExt;

#[cfg(not(target_arch = "wasm32"))]
pub use save_plugin::quicksave_file_path;
