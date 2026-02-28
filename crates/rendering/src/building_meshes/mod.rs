//! Building mesh generation — model cache, procedural fallbacks, and color queries.
//!
//! Split into sub-modules by domain:
//! - `model_cache`: The `BuildingModelCache` resource and scale helpers
//! - `model_loading`: Startup system that loads all GLB models
//! - `mesh_data`: The `MeshData` helper for procedural mesh construction
//! - `service_scene_map`: ServiceType → GLB asset path mapping
//! - `utility_scene_map`: UtilityType → GLB asset path mapping
//! - `service_emergency`: Fire, police, prison, and hospital meshes
//! - `service_education`: School, university, and library meshes
//! - `service_recreation`: Park, playground, sports, plaza, and stadium meshes
//! - `service_transport`: Train, bus, subway, airport, and ferry meshes
//! - `service_civic`: City hall, cathedral, museum, cemetery, and infrastructure meshes
//! - `service_welfare`: Shelter, welfare, post office, and mail sorting meshes
//! - `service_infrastructure`: Heating, geothermal, water treatment, and well pump meshes
//! - `utility_meshes`: Power, solar, wind, water, and nuclear utility meshes
//! - `colors`: Color query functions for UI/minimap

mod colors;
mod mesh_data;
mod model_cache;
mod model_loading;
mod service_civic;
mod service_education;
mod service_emergency;
mod service_infrastructure;
mod service_recreation;
mod service_scene_map;
mod service_transport;
mod service_welfare;
mod utility_meshes;
mod utility_scene_map;

// Re-export everything so callers see the same public API as before.
pub use colors::{service_base_color, utility_base_color, zone_base_color};
pub use mesh_data::MeshData;
pub use model_cache::{building_scale, service_building_scale, BuildingModelCache};
pub use model_loading::load_building_models;
