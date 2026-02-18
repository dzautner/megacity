use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use std::collections::HashMap;

use simulation::grid::ZoneType;
use simulation::services::ServiceType;
use simulation::utilities::UtilityType;

use simulation::config::CELL_SIZE;

// ---------------------------------------------------------------------------
// GLB Model Cache - loads real 3D models from assets/models/
// ---------------------------------------------------------------------------

/// Pre-loaded scene handles for all building GLB models
#[derive(Resource)]
pub struct BuildingModelCache {
    /// Residential building scenes (suburban houses)
    pub residential: Vec<Handle<Scene>>,
    /// Commercial building scenes
    pub commercial: Vec<Handle<Scene>>,
    /// Commercial skyscraper scenes (tall buildings)
    pub skyscrapers: Vec<Handle<Scene>>,
    /// Industrial building scenes
    pub industrial: Vec<Handle<Scene>>,
    /// Vehicle scenes (sedan, SUV, van, truck, etc.)
    pub vehicles: Vec<Handle<Scene>>,
    /// Character scenes (male/female variants)
    pub characters: Vec<Handle<Scene>>,
    /// Tree/nature scenes
    pub trees: Vec<Handle<Scene>>,
    /// Urban prop scenes (benches, lights, etc.)
    pub props: Vec<Handle<Scene>>,

    /// Fallback procedural meshes for service/utility buildings that don't have GLB models
    pub service_meshes: HashMap<ServiceType, Handle<Mesh>>,
    pub utility_meshes: HashMap<UtilityType, Handle<Mesh>>,
    pub fallback_material: Handle<StandardMaterial>,
}

impl BuildingModelCache {
    /// Get a residential building scene handle based on a hash value for variation
    pub fn get_residential(&self, hash: usize) -> Handle<Scene> {
        if self.residential.is_empty() {
            // This shouldn't happen but return first skyscraper or any available model
            return self.skyscrapers.first().cloned().unwrap_or_default();
        }
        self.residential[hash % self.residential.len()].clone()
    }

    /// Get a commercial building scene based on level and hash
    pub fn get_commercial(&self, level: u8, hash: usize) -> Handle<Scene> {
        if level >= 4 && !self.skyscrapers.is_empty() {
            return self.skyscrapers[hash % self.skyscrapers.len()].clone();
        }
        if self.commercial.is_empty() {
            return self.skyscrapers.first().cloned().unwrap_or_default();
        }
        self.commercial[hash % self.commercial.len()].clone()
    }

    /// Get an industrial building scene
    pub fn get_industrial(&self, hash: usize) -> Handle<Scene> {
        if self.industrial.is_empty() {
            return self.commercial.first().cloned().unwrap_or_default();
        }
        self.industrial[hash % self.industrial.len()].clone()
    }

    /// Get a building scene for any zone type and level
    pub fn get_zone_scene(&self, zone: ZoneType, level: u8, hash: usize) -> Handle<Scene> {
        match zone {
            ZoneType::ResidentialLow => self.get_residential(hash),
            ZoneType::ResidentialHigh => {
                if level >= 3 && !self.skyscrapers.is_empty() {
                    self.skyscrapers[hash % self.skyscrapers.len()].clone()
                } else if !self.commercial.is_empty() {
                    // Use commercial buildings for high-density residential
                    self.commercial[hash % self.commercial.len()].clone()
                } else {
                    self.get_residential(hash)
                }
            }
            ZoneType::CommercialLow => self.get_commercial(level, hash),
            ZoneType::CommercialHigh => self.get_commercial(level, hash),
            ZoneType::Industrial => self.get_industrial(hash),
            ZoneType::Office => {
                if level >= 3 && !self.skyscrapers.is_empty() {
                    self.skyscrapers[hash % self.skyscrapers.len()].clone()
                } else {
                    self.get_commercial(level, hash)
                }
            }
            ZoneType::None => self.get_residential(hash),
        }
    }

    /// Get a vehicle scene
    pub fn get_vehicle(&self, hash: usize) -> Handle<Scene> {
        if self.vehicles.is_empty() {
            return Handle::default();
        }
        self.vehicles[hash % self.vehicles.len()].clone()
    }

    /// Get a character scene
    pub fn get_character(&self, hash: usize) -> Handle<Scene> {
        if self.characters.is_empty() {
            return Handle::default();
        }
        self.characters[hash % self.characters.len()].clone()
    }

    /// Get a tree scene
    pub fn get_tree(&self, hash: usize) -> Handle<Scene> {
        if self.trees.is_empty() {
            return Handle::default();
        }
        self.trees[hash % self.trees.len()].clone()
    }

    /// Get a prop scene
    pub fn get_prop(&self, hash: usize) -> Handle<Scene> {
        if self.props.is_empty() {
            return Handle::default();
        }
        self.props[hash % self.props.len()].clone()
    }

    pub fn get_or_create_service_mesh(
        &mut self,
        service_type: ServiceType,
        meshes: &mut Assets<Mesh>,
    ) -> Handle<Mesh> {
        self.service_meshes
            .entry(service_type)
            .or_insert_with(|| meshes.add(generate_service_mesh(service_type)))
            .clone()
    }

    pub fn get_or_create_utility_mesh(
        &mut self,
        utility_type: UtilityType,
        meshes: &mut Assets<Mesh>,
    ) -> Handle<Mesh> {
        self.utility_meshes
            .entry(utility_type)
            .or_insert_with(|| meshes.add(generate_utility_mesh(utility_type)))
            .clone()
    }
}

/// Startup system: load all GLB models from assets/models/ directory
pub fn load_building_models(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let load_scene = |path: String| -> Handle<Scene> {
        asset_server.load(
            bevy::gltf::GltfAssetLabel::Scene(0).from_asset(path),
        )
    };

    // Load residential buildings (suburban houses)
    let residential_files = [
        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k",
        "l", "m", "n", "o", "p", "q", "r", "s", "t", "u",
    ];
    let residential: Vec<Handle<Scene>> = residential_files
        .iter()
        .map(|letter| load_scene(format!("models/buildings/residential/building-type-{letter}.glb")))
        .collect();

    // Load commercial buildings
    let commercial_files = [
        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n",
    ];
    let mut commercial: Vec<Handle<Scene>> = commercial_files
        .iter()
        .map(|letter| load_scene(format!("models/buildings/commercial/building-{letter}.glb")))
        .collect();

    // Also load low-detail commercial buildings
    let low_detail_files = [
        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n",
    ];
    for letter in &low_detail_files {
        commercial.push(load_scene(format!("models/buildings/commercial/low-detail-building-{letter}.glb")));
    }

    // Load skyscrapers
    let skyscraper_files = ["a", "b", "c", "d", "e"];
    let skyscrapers: Vec<Handle<Scene>> = skyscraper_files
        .iter()
        .map(|letter| load_scene(format!("models/buildings/skyscrapers/building-skyscraper-{letter}.glb")))
        .collect();

    // Load industrial buildings
    let industrial_files = [
        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j",
        "k", "l", "m", "n", "o", "p", "q", "r", "s", "t",
    ];
    let industrial: Vec<Handle<Scene>> = industrial_files
        .iter()
        .map(|letter| load_scene(format!("models/buildings/industrial/building-{letter}.glb")))
        .collect();

    // Load vehicles
    let vehicle_files = [
        "sedan", "sedan-sports", "hatchback-sports", "suv", "suv-luxury",
        "van", "truck", "taxi", "police", "ambulance", "firetruck",
        "garbage-truck", "delivery", "delivery-flat", "truck-flat",
    ];
    let vehicles: Vec<Handle<Scene>> = vehicle_files
        .iter()
        .map(|name| load_scene(format!("models/vehicles/{name}.glb")))
        .collect();

    // Load characters
    let character_files = [
        "character-female-a", "character-female-b", "character-female-c",
        "character-female-d", "character-female-e", "character-female-f",
        "character-male-a", "character-male-b", "character-male-c",
        "character-male-d", "character-male-e", "character-male-f",
    ];
    let characters: Vec<Handle<Scene>> = character_files
        .iter()
        .map(|name| load_scene(format!("models/characters/{name}.glb")))
        .collect();

    // Load trees and nature
    let tree_files = [
        "tree-suburban", "tree-retro-large", "tree-park-large", "tree-park-pine-large",
    ];
    let mut trees: Vec<Handle<Scene>> = tree_files
        .iter()
        .map(|name| load_scene(format!("models/props/{name}.glb")))
        .collect();

    // Load additional trees from nature kit
    let nature_tree_files = [
        "tree_cone_fall", "tree_pineRoundA", "tree_pineRoundB", "tree_pineRoundC",
        "tree_tall_dark", "tree_palmTall", "tree_palmDetailedShort",
    ];
    for name in &nature_tree_files {
        trees.push(load_scene(format!("models/nature/{name}.glb")));
    }

    // Load urban props
    let prop_files = [
        "detail-bench", "detail-light-single", "detail-light-double",
        "detail-barrier-type-a", "detail-dumpster-closed", "planter",
    ];
    let props: Vec<Handle<Scene>> = prop_files
        .iter()
        .map(|name| load_scene(format!("models/props/{name}.glb")))
        .collect();

    let fallback_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.8,
        ..default()
    });

    commands.insert_resource(BuildingModelCache {
        residential,
        commercial,
        skyscrapers,
        industrial,
        vehicles,
        characters,
        trees,
        props,
        service_meshes: HashMap::new(),
        utility_meshes: HashMap::new(),
        fallback_material,
    });
}

/// Get the scale factor for a building based on zone type and level
/// Kenney city-kit models are ~1-3 units wide. CELL_SIZE = 16 units.
/// Buildings should fill their cell in dense areas for a Manhattan-like look.
pub fn building_scale(zone: ZoneType, level: u8) -> f32 {
    // Base scale: a ~2-unit-wide Kenney model * 7.5 = 15 units ≈ 94% of cell
    let base: f32 = 7.5;
    match (zone, level) {
        // Residential low: suburban houses (slightly smaller, with yard space)
        (ZoneType::ResidentialLow, 1) => base * 0.65,
        (ZoneType::ResidentialLow, 2) => base * 0.75,
        (ZoneType::ResidentialLow, _) => base * 0.85,

        // Residential high: apartments/towers (fill cell, grow taller with level)
        (ZoneType::ResidentialHigh, 1) => base * 0.85,
        (ZoneType::ResidentialHigh, 2) => base * 1.0,
        (ZoneType::ResidentialHigh, 3) => base * 1.15,
        (ZoneType::ResidentialHigh, 4) => base * 1.3,
        (ZoneType::ResidentialHigh, _) => base * 1.5,

        // Commercial low: shops (fill cell width)
        (ZoneType::CommercialLow, 1) => base * 0.75,
        (ZoneType::CommercialLow, 2) => base * 0.9,
        (ZoneType::CommercialLow, _) => base * 1.0,

        // Commercial high: towers/skyscrapers (full width, grow tall)
        (ZoneType::CommercialHigh, 1) => base * 0.9,
        (ZoneType::CommercialHigh, 2) => base * 1.1,
        (ZoneType::CommercialHigh, 3) => base * 1.3,
        (ZoneType::CommercialHigh, 4) => base * 1.5,
        (ZoneType::CommercialHigh, _) => base * 1.8,

        // Industrial: wide, low buildings
        (ZoneType::Industrial, 1) => base * 0.8,
        (ZoneType::Industrial, 2) => base * 0.95,
        (ZoneType::Industrial, 3) => base * 1.1,
        (ZoneType::Industrial, _) => base * 1.2,

        // Office: glass towers (full width, tall)
        (ZoneType::Office, 1) => base * 0.85,
        (ZoneType::Office, 2) => base * 1.05,
        (ZoneType::Office, 3) => base * 1.3,
        (ZoneType::Office, 4) => base * 1.5,
        (ZoneType::Office, _) => base * 1.8,

        _ => base,
    }
}

// ---------------------------------------------------------------------------
// Mesh Helpers (kept for service/utility procedural meshes)
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct MeshData {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

impl MeshData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn into_mesh(self) -> Mesh {
        let uvs: Vec<[f32; 2]> = vec![[0.0, 0.0]; self.positions.len()];
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.colors)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(self.indices))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_cuboid(&mut self, cx: f32, cy: f32, cz: f32, hw: f32, hh: f32, hd: f32, color: [f32; 4]) {
        let base = self.positions.len() as u32;
        let x0 = cx - hw;
        let x1 = cx + hw;
        let y0 = cy - hh;
        let y1 = cy + hh;
        let z0 = cz - hd;
        let z1 = cz + hd;

        let front_color = darken(color, 0.85);
        self.positions.extend_from_slice(&[[x0,y0,z1],[x1,y0,z1],[x1,y1,z1],[x0,y1,z1]]);
        self.normals.extend_from_slice(&[[0.0,0.0,1.0];4]);
        self.colors.extend_from_slice(&[front_color;4]);
        self.indices.extend_from_slice(&[base,base+1,base+2,base,base+2,base+3]);

        let b = base + 4;
        let back_color = darken(color, 0.75);
        self.positions.extend_from_slice(&[[x1,y0,z0],[x0,y0,z0],[x0,y1,z0],[x1,y1,z0]]);
        self.normals.extend_from_slice(&[[0.0,0.0,-1.0];4]);
        self.colors.extend_from_slice(&[back_color;4]);
        self.indices.extend_from_slice(&[b,b+1,b+2,b,b+2,b+3]);

        let b = base + 8;
        let top_color = lighten(color, 1.3);
        self.positions.extend_from_slice(&[[x0,y1,z1],[x1,y1,z1],[x1,y1,z0],[x0,y1,z0]]);
        self.normals.extend_from_slice(&[[0.0,1.0,0.0];4]);
        self.colors.extend_from_slice(&[top_color;4]);
        self.indices.extend_from_slice(&[b,b+1,b+2,b,b+2,b+3]);

        let b = base + 12;
        let bot_color = darken(color, 0.5);
        self.positions.extend_from_slice(&[[x0,y0,z0],[x1,y0,z0],[x1,y0,z1],[x0,y0,z1]]);
        self.normals.extend_from_slice(&[[0.0,-1.0,0.0];4]);
        self.colors.extend_from_slice(&[bot_color;4]);
        self.indices.extend_from_slice(&[b,b+1,b+2,b,b+2,b+3]);

        let b = base + 16;
        let right_color = darken(color, 0.7);
        self.positions.extend_from_slice(&[[x1,y0,z1],[x1,y0,z0],[x1,y1,z0],[x1,y1,z1]]);
        self.normals.extend_from_slice(&[[1.0,0.0,0.0];4]);
        self.colors.extend_from_slice(&[right_color;4]);
        self.indices.extend_from_slice(&[b,b+1,b+2,b,b+2,b+3]);

        let b = base + 20;
        let left_color = darken(color, 0.65);
        self.positions.extend_from_slice(&[[x0,y0,z0],[x0,y0,z1],[x0,y1,z1],[x0,y1,z0]]);
        self.normals.extend_from_slice(&[[-1.0,0.0,0.0];4]);
        self.colors.extend_from_slice(&[left_color;4]);
        self.indices.extend_from_slice(&[b,b+1,b+2,b,b+2,b+3]);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_cylinder(&mut self, cx: f32, cy: f32, cz: f32, radius: f32, height: f32, segments: u32, color: [f32; 4]) {
        let base = self.positions.len() as u32;
        let half_h = height * 0.5;

        for i in 0..=segments {
            let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let x = cx + angle.cos() * radius;
            let z = cz + angle.sin() * radius;
            let nx = angle.cos();
            let nz = angle.sin();

            self.positions.push([x, cy - half_h, z]);
            self.normals.push([nx, 0.0, nz]);
            self.colors.push(darken(color, 0.9));

            self.positions.push([x, cy + half_h, z]);
            self.normals.push([nx, 0.0, nz]);
            self.colors.push(color);
        }

        for i in 0..segments {
            let i0 = base + i * 2;
            let i1 = base + i * 2 + 1;
            let i2 = base + (i + 1) * 2;
            let i3 = base + (i + 1) * 2 + 1;
            self.indices.extend_from_slice(&[i0, i2, i1, i1, i2, i3]);
        }

        let top_center = self.positions.len() as u32;
        self.positions.push([cx, cy + half_h, cz]);
        self.normals.push([0.0, 1.0, 0.0]);
        self.colors.push(lighten(color, 1.1));

        for i in 0..segments {
            let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let x = cx + angle.cos() * radius;
            let z = cz + angle.sin() * radius;
            self.positions.push([x, cy + half_h, z]);
            self.normals.push([0.0, 1.0, 0.0]);
            self.colors.push(lighten(color, 1.1));
        }

        for i in 0..segments {
            let v1 = top_center + 1 + i;
            let v2 = top_center + 1 + (i + 1) % segments;
            self.indices.extend_from_slice(&[top_center, v1, v2]);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn add_roof_prism(&mut self, cx: f32, cy: f32, cz: f32, hw: f32, hh: f32, hd: f32, color: [f32; 4]) {
        let base = self.positions.len() as u32;
        let x0 = cx - hw;
        let x1 = cx + hw;
        let z0 = cz - hd;
        let z1 = cz + hd;
        let peak_y = cy + hh;

        self.positions.extend_from_slice(&[[x0,cy,z1],[x1,cy,z1],[cx,peak_y,z1]]);
        self.normals.extend_from_slice(&[[0.0,0.0,1.0];3]);
        self.colors.extend_from_slice(&[color;3]);
        self.indices.extend_from_slice(&[base,base+1,base+2]);

        let b = base + 3;
        self.positions.extend_from_slice(&[[x1,cy,z0],[x0,cy,z0],[cx,peak_y,z0]]);
        self.normals.extend_from_slice(&[[0.0,0.0,-1.0];3]);
        self.colors.extend_from_slice(&[color;3]);
        self.indices.extend_from_slice(&[b,b+1,b+2]);

        let b = base + 6;
        let n_left = Vec3::new(-hh, hw, 0.0).normalize();
        let nl = [n_left.x, n_left.y, n_left.z];
        self.positions.extend_from_slice(&[[x0,cy,z0],[x0,cy,z1],[cx,peak_y,z1],[cx,peak_y,z0]]);
        self.normals.extend_from_slice(&[nl;4]);
        self.colors.extend_from_slice(&[darken(color, 0.85);4]);
        self.indices.extend_from_slice(&[b,b+1,b+2,b,b+2,b+3]);

        let b = base + 10;
        let n_right = Vec3::new(hh, hw, 0.0).normalize();
        let nr = [n_right.x, n_right.y, n_right.z];
        self.positions.extend_from_slice(&[[x1,cy,z1],[x1,cy,z0],[cx,peak_y,z0],[cx,peak_y,z1]]);
        self.normals.extend_from_slice(&[nr;4]);
        self.colors.extend_from_slice(&[darken(color, 0.9);4]);
        self.indices.extend_from_slice(&[b,b+1,b+2,b,b+2,b+3]);
    }
}

fn lighten(c: [f32; 4], factor: f32) -> [f32; 4] {
    [(c[0] * factor).min(1.0), (c[1] * factor).min(1.0), (c[2] * factor).min(1.0), c[3]]
}

fn darken(c: [f32; 4], factor: f32) -> [f32; 4] {
    [c[0] * factor, c[1] * factor, c[2] * factor, c[3]]
}

// ---------------------------------------------------------------------------
// Service Building Meshes (procedural fallback for service/utility types)
// ---------------------------------------------------------------------------

fn generate_service_mesh(service_type: ServiceType) -> Mesh {
    let mut m = MeshData::new();
    let s = CELL_SIZE;
    let (fw, fh) = simulation::services::ServiceBuilding::footprint(service_type);
    let scale_x = fw as f32;
    let scale_z = fh as f32;

    match service_type {
        ServiceType::FireStation | ServiceType::FireHouse | ServiceType::FireHQ => {
            let color = [1.0, 0.50, 0.50, 1.0];
            let hw = s * 0.4 * scale_x;
            let hh = s * 0.3;
            let hd = s * 0.4 * scale_z;
            m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);
            // Garage doors
            m.add_cuboid(-hw * 0.3, hh * 0.45, hd + 0.05, hw * 0.25, hh * 0.45, 0.05, darken(color, 0.4));
            m.add_cuboid(hw * 0.3, hh * 0.45, hd + 0.05, hw * 0.25, hh * 0.45, 0.05, darken(color, 0.4));
            // Tower (hose drying tower)
            m.add_cuboid(hw * 0.7, hh * 2.5, hd * 0.5, s * 0.08, hh * 1.2, s * 0.08, darken(color, 0.8));
            // Flag pole
            m.add_cylinder(-hw * 0.7, hh * 2.5, hd * 0.7, s * 0.015, s * 0.6, 4, [0.6, 0.6, 0.6, 1.0]);
        }
        ServiceType::PoliceStation | ServiceType::PoliceKiosk | ServiceType::PoliceHQ => {
            let color = [0.41, 0.53, 0.66, 1.0];
            let hw = s * 0.4 * scale_x;
            let hh = s * 0.3;
            let hd = s * 0.4 * scale_z;
            m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);
            // Entrance columns
            m.add_cuboid(-hw * 0.3, hh, hd + s * 0.04, s * 0.03, hh, s * 0.03, [0.7, 0.7, 0.7, 1.0]);
            m.add_cuboid(hw * 0.3, hh, hd + s * 0.04, s * 0.03, hh, s * 0.03, [0.7, 0.7, 0.7, 1.0]);
            // Blue dome
            m.add_cylinder(0.0, hh * 2.0 + s * 0.05, 0.0, s * 0.06, s * 0.08, 6, [0.3, 0.5, 0.9, 1.0]);
            // Flag pole
            m.add_cylinder(hw * 0.8, hh * 2.0 + s * 0.2, hd * 0.8, s * 0.015, s * 0.5, 4, [0.6, 0.6, 0.6, 1.0]);
        }
        ServiceType::Prison => {
            let color = [0.45, 0.45, 0.45, 1.0];
            let hw = s * 0.45 * scale_x;
            let hh = s * 0.35;
            let hd = s * 0.45 * scale_z;
            m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);
            let wall = darken(color, 0.7);
            let wt = s * 0.03;
            m.add_cuboid(0.0, hh * 1.2, hd, hw, hh * 0.15, wt, wall);
            m.add_cuboid(0.0, hh * 1.2, -hd, hw, hh * 0.15, wt, wall);
            m.add_cuboid(hw, hh * 1.2, 0.0, wt, hh * 0.15, hd, wall);
            m.add_cuboid(-hw, hh * 1.2, 0.0, wt, hh * 0.15, hd, wall);
            // Guard towers at corners
            m.add_cuboid(hw, hh * 2.0, hd, s * 0.06, hh * 0.5, s * 0.06, darken(color, 0.6));
            m.add_cuboid(-hw, hh * 2.0, -hd, s * 0.06, hh * 0.5, s * 0.06, darken(color, 0.6));
        }
        ServiceType::Hospital | ServiceType::MedicalClinic | ServiceType::MedicalCenter => {
            let color = [0.94, 0.69, 0.75, 1.0];
            let hw = s * 0.4 * scale_x;
            let hh = s * 0.45;
            let hd = s * 0.4 * scale_z;
            // Multi-story building
            m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);
            // Windows on each floor
            let n_floors = 4;
            let floor_h = hh * 2.0 / n_floors as f32;
            for floor in 0..n_floors {
                let y = floor_h * 0.5 + floor as f32 * floor_h;
                // Front windows
                let spacing = (hw * 2.0) / 5.0;
                for i in 1..5 {
                    let wx = -hw + i as f32 * spacing;
                    m.add_cuboid(wx, y, hd - 0.05, s * 0.03, s * 0.04, 0.08, [0.2, 0.22, 0.3, 1.0]);
                }
            }
            // Red cross on facade
            let cross = [0.9, 0.1, 0.1, 1.0];
            m.add_cuboid(0.0, hh * 1.7, hd + 0.03, s * 0.15, s * 0.04, s * 0.02, cross);
            m.add_cuboid(0.0, hh * 1.7, hd + 0.03, s * 0.04, s * 0.15, s * 0.02, cross);
            // Entrance
            m.add_cuboid(0.0, hh * 0.3, hd + 0.05, hw * 0.25, hh * 0.3, 0.05, darken(color, 0.5));
        }
        ServiceType::ElementarySchool | ServiceType::HighSchool | ServiceType::Kindergarten => {
            let color = [0.94, 0.78, 0.63, 1.0];
            let hw = s * 0.4;
            let hh = s * 0.25;
            let hd = s * 0.4;
            // L-shaped building: main wing
            m.add_cuboid(0.0, hh, 0.0, hw, hh, hd * 0.6, color);
            // Side wing
            m.add_cuboid(hw * 0.5, hh, hd * 0.3, hw * 0.5, hh, hd * 0.4, darken(color, 0.95));
            // Peaked roof on main wing
            m.add_roof_prism(0.0, hh * 2.0, 0.0, hw, hh * 0.4, hd * 0.6, darken(color, 0.85));
            // Flagpole
            m.add_cylinder(hw * 0.8, hh * 2.0 + s * 0.15, hd * 0.8, s * 0.015, s * 0.4, 4, [0.6, 0.6, 0.6, 1.0]);
            // Windows
            let spacing = (hw * 2.0) / 5.0;
            for i in 1..5 {
                let wx = -hw + i as f32 * spacing;
                m.add_cuboid(wx, hh, hd * 0.6 - 0.05, s * 0.035, s * 0.04, 0.08, [0.2, 0.22, 0.3, 1.0]);
            }
        }
        ServiceType::University => {
            let color = [0.47, 0.47, 0.72, 1.0];
            let hw = s * 0.42;
            let hh = s * 0.45;
            let hd = s * 0.42;
            m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);
            // Clock tower
            m.add_cuboid(0.0, hh * 2.5, 0.0, s * 0.08, hh * 0.8, s * 0.08, darken(color, 0.85));
            // Dome on top
            m.add_cylinder(0.0, hh * 2.0 + hh * 1.6 + s * 0.06, 0.0, s * 0.12, s * 0.10, 8, lighten(color, 1.2));
            // Entrance columns
            for i in 0..4 {
                let x = -hw * 0.4 + i as f32 * hw * 0.27;
                m.add_cuboid(x, hh, hd + s * 0.03, s * 0.025, hh, s * 0.025, [0.75, 0.75, 0.78, 1.0]);
            }
        }
        ServiceType::Library => {
            let color = [0.85, 0.70, 0.50, 1.0];
            let hw = s * 0.38;
            let hh = s * 0.30;
            let hd = s * 0.38;
            m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);
            // Classical columns at entrance
            for i in 0..3 {
                let x = -hw * 0.3 + i as f32 * hw * 0.3;
                m.add_cuboid(x, hh, hd + s * 0.04, s * 0.03, hh, s * 0.03, [0.8, 0.78, 0.72, 1.0]);
            }
            // Wide steps
            m.add_cuboid(0.0, hh * 0.15, hd + s * 0.08, hw * 0.6, hh * 0.15, s * 0.06, darken(color, 0.8));
        }
        ServiceType::SmallPark | ServiceType::LargePark => {
            let color = [0.25, 0.65, 0.25, 1.0];
            let hw = s * 0.45 * scale_x;
            let hd = s * 0.45 * scale_z;
            // Flat green base
            m.add_cuboid(0.0, s * 0.02, 0.0, hw, s * 0.02, hd, color);
            // Paths (lighter strips)
            m.add_cuboid(0.0, s * 0.03, 0.0, hw * 0.08, s * 0.01, hd, [0.6, 0.55, 0.45, 1.0]);
            m.add_cuboid(0.0, s * 0.03, 0.0, hw, s * 0.01, hd * 0.08, [0.6, 0.55, 0.45, 1.0]);
            // Trees (trunk + canopy)
            m.add_cylinder(-hw * 0.4, s * 0.15, -hd * 0.4, s * 0.03, s * 0.2, 6, [0.45, 0.30, 0.15, 1.0]);
            m.add_cylinder(-hw * 0.4, s * 0.35, -hd * 0.4, s * 0.12, s * 0.15, 8, [0.15, 0.5, 0.12, 1.0]);
            m.add_cylinder(hw * 0.35, s * 0.18, hd * 0.3, s * 0.04, s * 0.25, 6, [0.45, 0.30, 0.15, 1.0]);
            m.add_cylinder(hw * 0.35, s * 0.40, hd * 0.3, s * 0.15, s * 0.18, 8, [0.12, 0.45, 0.10, 1.0]);
            // Bench
            m.add_cuboid(hw * 0.15, s * 0.06, hd * 0.5, s * 0.08, s * 0.02, s * 0.03, [0.50, 0.35, 0.20, 1.0]);
            // Fountain (for large park)
            if matches!(service_type, ServiceType::LargePark) {
                m.add_cylinder(0.0, s * 0.08, 0.0, s * 0.10, s * 0.08, 8, [0.6, 0.6, 0.65, 1.0]);
                m.add_cylinder(0.0, s * 0.15, 0.0, s * 0.04, s * 0.10, 6, [0.3, 0.5, 0.7, 1.0]);
            }
        }
        ServiceType::Playground => {
            let color = [0.25, 0.65, 0.25, 1.0];
            m.add_cuboid(0.0, s * 0.02, 0.0, s * 0.45, s * 0.02, s * 0.45, color);
            // Play structures (small colored cuboids)
            m.add_cuboid(-s * 0.15, s * 0.12, -s * 0.1, s * 0.06, s * 0.10, s * 0.06, [0.9, 0.3, 0.2, 1.0]);
            m.add_cuboid(s * 0.15, s * 0.08, s * 0.1, s * 0.08, s * 0.06, s * 0.04, [0.2, 0.5, 0.9, 1.0]);
            m.add_cuboid(0.0, s * 0.15, 0.0, s * 0.04, s * 0.13, s * 0.04, [0.9, 0.8, 0.2, 1.0]);
        }
        ServiceType::SportsField => {
            let color = [0.20, 0.60, 0.20, 1.0];
            m.add_cuboid(0.0, s * 0.02, 0.0, s * 0.45, s * 0.02, s * 0.45, color);
            // Goal posts
            m.add_cuboid(-s * 0.40, s * 0.10, 0.0, s * 0.02, s * 0.10, s * 0.02, [1.0, 1.0, 1.0, 1.0]);
            m.add_cuboid(s * 0.40, s * 0.10, 0.0, s * 0.02, s * 0.10, s * 0.02, [1.0, 1.0, 1.0, 1.0]);
            // Field lines
            m.add_cuboid(0.0, s * 0.025, 0.0, s * 0.01, s * 0.005, s * 0.35, [1.0, 1.0, 1.0, 0.8]);
        }
        ServiceType::Plaza => {
            let color = [0.60, 0.58, 0.52, 1.0];
            m.add_cuboid(0.0, s * 0.02, 0.0, s * 0.45, s * 0.02, s * 0.45, color);
            // Lamp posts
            m.add_cylinder(-s * 0.25, s * 0.20, -s * 0.25, s * 0.015, s * 0.35, 4, [0.4, 0.4, 0.42, 1.0]);
            m.add_cylinder(s * 0.25, s * 0.20, s * 0.25, s * 0.015, s * 0.35, 4, [0.4, 0.4, 0.42, 1.0]);
            // Fountain centerpiece
            m.add_cylinder(0.0, s * 0.08, 0.0, s * 0.12, s * 0.08, 8, [0.55, 0.55, 0.58, 1.0]);
            m.add_cylinder(0.0, s * 0.18, 0.0, s * 0.04, s * 0.12, 6, [0.3, 0.5, 0.7, 1.0]);
        }
        ServiceType::Stadium => {
            let _field_color = [0.55, 0.75, 0.55, 1.0];
            let hw = s * 0.45;
            let hh = s * 0.25;
            let hd = s * 0.45;
            // Field
            m.add_cuboid(0.0, s * 0.02, 0.0, hw * 0.7, s * 0.02, hd * 0.7, [0.2, 0.6, 0.2, 1.0]);
            // Stands (4 sides, stacked rings)
            let stand = [0.55, 0.55, 0.58, 1.0];
            m.add_cuboid(0.0, hh * 0.5, hd, hw, hh * 0.5, s * 0.08, stand);
            m.add_cuboid(0.0, hh, hd * 0.9, hw * 0.9, hh * 0.3, s * 0.06, darken(stand, 0.9));
            m.add_cuboid(0.0, hh * 0.5, -hd, hw, hh * 0.5, s * 0.08, stand);
            m.add_cuboid(0.0, hh, -hd * 0.9, hw * 0.9, hh * 0.3, s * 0.06, darken(stand, 0.9));
            m.add_cuboid(hw, hh * 0.5, 0.0, s * 0.08, hh * 0.5, hd, stand);
            m.add_cuboid(-hw, hh * 0.5, 0.0, s * 0.08, hh * 0.5, hd, stand);
            // Flag poles at corners
            m.add_cylinder(hw, hh * 2.0, hd, s * 0.015, s * 0.4, 4, [0.6, 0.6, 0.6, 1.0]);
            m.add_cylinder(-hw, hh * 2.0, -hd, s * 0.015, s * 0.4, 4, [0.6, 0.6, 0.6, 1.0]);
        }
        ServiceType::TrainStation => {
            let color = [0.50, 0.55, 0.62, 1.0];
            let hw = s * 0.45 * scale_x;
            let hh = s * 0.25;
            let hd = s * 0.4 * scale_z;
            // Main station building
            m.add_cuboid(0.0, hh, 0.0, hw * 0.5, hh, hd, color);
            // Platform canopy (flat roof on columns)
            m.add_cuboid(hw * 0.3, hh * 1.8, 0.0, hw * 0.5, s * 0.02, hd * 1.1, darken(color, 0.8));
            // Canopy columns
            for i in 0..4 {
                let z = -hd * 0.8 + i as f32 * hd * 0.53;
                m.add_cuboid(hw * 0.05, hh * 0.9, z, s * 0.02, hh * 0.9, s * 0.02, [0.5, 0.5, 0.52, 1.0]);
                m.add_cuboid(hw * 0.55, hh * 0.9, z, s * 0.02, hh * 0.9, s * 0.02, [0.5, 0.5, 0.52, 1.0]);
            }
            // Clock tower
            m.add_cuboid(0.0, hh * 2.5, 0.0, s * 0.06, hh * 0.8, s * 0.06, darken(color, 0.85));
            m.add_cuboid(0.0, hh * 3.2, 0.0, s * 0.04, s * 0.04, s * 0.04, [0.8, 0.78, 0.65, 1.0]);
        }
        ServiceType::BusDepot => {
            let color = [0.50, 0.58, 0.65, 1.0];
            let hw = s * 0.45 * scale_x;
            let hh = s * 0.30;
            let hd = s * 0.4 * scale_z;
            // Open-sided garage structure (roof on columns)
            m.add_cuboid(0.0, hh * 2.0, 0.0, hw, s * 0.03, hd, darken(color, 0.85));
            // Columns
            m.add_cuboid(-hw, hh, -hd, s * 0.04, hh, s * 0.04, [0.5, 0.5, 0.52, 1.0]);
            m.add_cuboid(hw, hh, -hd, s * 0.04, hh, s * 0.04, [0.5, 0.5, 0.52, 1.0]);
            m.add_cuboid(-hw, hh, hd, s * 0.04, hh, s * 0.04, [0.5, 0.5, 0.52, 1.0]);
            m.add_cuboid(hw, hh, hd, s * 0.04, hh, s * 0.04, [0.5, 0.5, 0.52, 1.0]);
            // Back wall
            m.add_cuboid(0.0, hh, -hd, hw, hh, s * 0.03, color);
            // Parked bus shape
            m.add_cuboid(0.0, s * 0.12, 0.0, s * 0.08, s * 0.10, s * 0.25, [0.2, 0.4, 0.7, 1.0]);
        }
        ServiceType::SubwayStation | ServiceType::TramDepot => {
            let color = [0.50, 0.60, 0.70, 1.0];
            let hw = s * 0.4 * scale_x;
            let hh = s * 0.20;
            let hd = s * 0.4 * scale_z;
            // Entrance building
            m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);
            // Stairs indicator (recessed darker section)
            m.add_cuboid(0.0, hh * 0.5, hd + 0.05, hw * 0.4, hh * 0.5, 0.08, darken(color, 0.45));
            // Subway sign post
            m.add_cylinder(hw * 0.6, hh * 2.5, hd * 0.6, s * 0.02, s * 0.3, 4, [0.5, 0.5, 0.55, 1.0]);
            m.add_cuboid(hw * 0.6, hh * 2.0 + s * 0.25, hd * 0.6, s * 0.06, s * 0.06, s * 0.02, lighten(color, 1.3));
        }
        ServiceType::SmallAirstrip => {
            let color = [0.65, 0.65, 0.70, 1.0];
            let hw = s * 0.45 * scale_x;
            let hh = s * 0.15;
            let hd = s * 0.45 * scale_z;
            // Small terminal building
            m.add_cuboid(0.0, hh, 0.0, hw * 0.4, hh, hd * 0.3, color);
            // Runway
            m.add_cuboid(0.0, s * 0.01, hd * 0.3, hw * 0.12, s * 0.01, hd * 0.7, [0.3, 0.3, 0.35, 1.0]);
            // Runway center stripe
            m.add_cuboid(0.0, s * 0.015, hd * 0.3, hw * 0.01, s * 0.005, hd * 0.6, [1.0, 1.0, 1.0, 0.8]);
            // Windsock pole
            m.add_cylinder(hw * 0.5, hh * 2.0, -hd * 0.3, s * 0.02, s * 0.3, 4, [0.5, 0.5, 0.55, 1.0]);
        }
        ServiceType::RegionalAirport => {
            let color = [0.60, 0.62, 0.68, 1.0];
            let hw = s * 0.45 * scale_x;
            let hh = s * 0.20;
            let hd = s * 0.45 * scale_z;
            // Terminal building
            m.add_cuboid(0.0, hh, -hd * 0.2, hw * 0.5, hh, hd * 0.35, color);
            // Terminal extension (gate concourse)
            m.add_cuboid(hw * 0.15, hh * 0.8, -hd * 0.55, hw * 0.2, hh * 0.6, hd * 0.1, darken(color, 0.9));
            // Runway
            m.add_cuboid(0.0, s * 0.01, hd * 0.25, hw * 0.15, s * 0.01, hd * 0.75, [0.3, 0.3, 0.35, 1.0]);
            // Runway center stripe
            m.add_cuboid(0.0, s * 0.015, hd * 0.25, hw * 0.01, s * 0.005, hd * 0.65, [1.0, 1.0, 1.0, 0.8]);
            // Control tower
            m.add_cylinder(hw * 0.55, hh * 2.5, -hd * 0.3, s * 0.05, s * 0.45, 8, [0.5, 0.5, 0.55, 1.0]);
            // Tower cab (observation deck)
            m.add_cuboid(hw * 0.55, hh * 2.5 + s * 0.25, -hd * 0.3, s * 0.08, s * 0.06, s * 0.08, [0.4, 0.6, 0.65, 1.0]);
        }
        ServiceType::InternationalAirport => {
            let color = [0.58, 0.60, 0.66, 1.0];
            let hw = s * 0.45 * scale_x;
            let hh = s * 0.22;
            let hd = s * 0.45 * scale_z;
            // Main terminal building (large)
            m.add_cuboid(0.0, hh, -hd * 0.15, hw * 0.6, hh, hd * 0.40, color);
            // Terminal wings (gate concourses on each side)
            m.add_cuboid(-hw * 0.35, hh * 0.7, -hd * 0.6, hw * 0.15, hh * 0.5, hd * 0.15, darken(color, 0.9));
            m.add_cuboid(hw * 0.35, hh * 0.7, -hd * 0.6, hw * 0.15, hh * 0.5, hd * 0.15, darken(color, 0.9));
            // Two parallel runways
            m.add_cuboid(-hw * 0.25, s * 0.01, hd * 0.3, hw * 0.12, s * 0.01, hd * 0.7, [0.3, 0.3, 0.35, 1.0]);
            m.add_cuboid(hw * 0.25, s * 0.01, hd * 0.3, hw * 0.12, s * 0.01, hd * 0.7, [0.3, 0.3, 0.35, 1.0]);
            // Runway center stripes
            m.add_cuboid(-hw * 0.25, s * 0.015, hd * 0.3, hw * 0.01, s * 0.005, hd * 0.6, [1.0, 1.0, 1.0, 0.8]);
            m.add_cuboid(hw * 0.25, s * 0.015, hd * 0.3, hw * 0.01, s * 0.005, hd * 0.6, [1.0, 1.0, 1.0, 0.8]);
            // Tall control tower
            m.add_cylinder(hw * 0.6, hh * 3.0, -hd * 0.25, s * 0.06, s * 0.65, 8, [0.5, 0.5, 0.55, 1.0]);
            // Tower cab
            m.add_cuboid(hw * 0.6, hh * 3.0 + s * 0.35, -hd * 0.25, s * 0.10, s * 0.08, s * 0.10, [0.3, 0.55, 0.6, 1.0]);
            // Parking structure
            m.add_cuboid(0.0, hh * 0.5, hd * 0.15, hw * 0.3, hh * 0.3, hd * 0.1, darken(color, 0.7));
        }
        ServiceType::FerryPier => {
            let color = [0.40, 0.55, 0.70, 1.0];
            m.add_cuboid(0.0, s * 0.08, 0.0, s * 0.4, s * 0.08, s * 0.15, color);
            m.add_cuboid(0.0, s * 0.04, s * 0.3, s * 0.1, s * 0.04, s * 0.2, darken(color, 0.7));
        }
        ServiceType::CellTower => {
            let color = [0.6, 0.6, 0.6, 1.0];
            m.add_cylinder(0.0, s * 0.5, 0.0, s * 0.03, s * 1.0, 6, color);
            m.add_cuboid(0.0, s * 0.85, 0.0, s * 0.15, s * 0.01, s * 0.01, [0.5, 0.5, 0.55, 1.0]);
            m.add_cuboid(0.0, s * 0.75, 0.0, s * 0.01, s * 0.01, s * 0.12, [0.5, 0.5, 0.55, 1.0]);
        }
        ServiceType::DataCenter => {
            let color = [0.35, 0.40, 0.50, 1.0];
            let hw = s * 0.4 * scale_x;
            let hh = s * 0.25;
            let hd = s * 0.4 * scale_z;
            m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);
            m.add_cuboid(hw * 0.5, hh * 2.0 + s * 0.05, hd * 0.5, s * 0.08, s * 0.05, s * 0.08, [0.4, 0.45, 0.5, 1.0]);
            m.add_cuboid(-hw * 0.5, hh * 2.0 + s * 0.05, -hd * 0.5, s * 0.08, s * 0.05, s * 0.08, [0.4, 0.45, 0.5, 1.0]);
        }
        ServiceType::TransferStation => {
            let color = [0.55, 0.50, 0.40, 1.0];
            m.add_cuboid(0.0, s * 0.2, 0.0, s * 0.4, s * 0.2, s * 0.35, color);
        }
        ServiceType::CityHall => {
            let color = [0.85, 0.80, 0.65, 1.0];
            let hw = s * 0.42;
            let hh = s * 0.40;
            let hd = s * 0.42;
            m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);
            // Columns at entrance
            for i in 0..5 {
                let x = -hw * 0.5 + i as f32 * hw * 0.25;
                m.add_cuboid(x, hh, hd + s * 0.04, s * 0.025, hh, s * 0.025, [0.8, 0.78, 0.72, 1.0]);
            }
            // Dome/cupola
            m.add_cylinder(0.0, hh * 2.0 + s * 0.1, 0.0, s * 0.12, s * 0.15, 8, darken(color, 0.85));
            // Flag
            m.add_cylinder(0.0, hh * 2.0 + s * 0.25, 0.0, s * 0.015, s * 0.3, 4, [0.6, 0.6, 0.6, 1.0]);
            // Steps
            m.add_cuboid(0.0, hh * 0.1, hd + s * 0.08, hw * 0.7, hh * 0.1, s * 0.06, darken(color, 0.8));
        }
        ServiceType::Cathedral => {
            let color = [0.78, 0.72, 0.60, 1.0];
            let hw = s * 0.40;
            let hh = s * 0.50;
            let hd = s * 0.42;
            m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);
            // Tall peaked roof
            m.add_roof_prism(0.0, hh * 2.0, 0.0, hw * 1.02, hh * 0.6, hd * 1.02, darken(color, 0.75));
            // Bell tower
            m.add_cuboid(hw * 0.35, hh * 2.8, -hd * 0.3, s * 0.08, hh * 0.8, s * 0.08, darken(color, 0.85));
            m.add_roof_prism(hw * 0.35, hh * 2.8 + hh * 0.8, -hd * 0.3, s * 0.10, s * 0.12, s * 0.10, darken(color, 0.7));
            // Rose window (circular approximation - just a colored disc)
            m.add_cuboid(0.0, hh * 1.5, hd + 0.03, s * 0.08, s * 0.08, s * 0.01, [0.5, 0.3, 0.6, 1.0]);
            // Entrance arch
            m.add_cuboid(0.0, hh * 0.35, hd + 0.05, hw * 0.2, hh * 0.35, 0.05, darken(color, 0.4));
        }
        ServiceType::Museum => {
            let color = [0.88, 0.85, 0.78, 1.0];
            let hw = s * 0.42;
            let hh = s * 0.35;
            let hd = s * 0.42;
            m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);
            // Classical columns
            for i in 0..6 {
                let x = -hw * 0.6 + i as f32 * hw * 0.24;
                m.add_cuboid(x, hh, hd + s * 0.05, s * 0.03, hh, s * 0.03, [0.82, 0.80, 0.75, 1.0]);
            }
            // Wide steps
            m.add_cuboid(0.0, hh * 0.1, hd + s * 0.10, hw * 0.8, hh * 0.1, s * 0.08, darken(color, 0.85));
            m.add_cuboid(0.0, hh * 0.2, hd + s * 0.06, hw * 0.8, hh * 0.1, s * 0.04, darken(color, 0.82));
            // Pediment (triangle above columns)
            m.add_roof_prism(0.0, hh * 2.0, hd + s * 0.02, hw * 0.65, hh * 0.3, s * 0.03, darken(color, 0.9));
        }
        ServiceType::Cemetery => {
            // Dark grey cemetery with headstones
            let color = [0.3, 0.35, 0.3, 1.0];
            let hw = s * 0.45;
            let hd = s * 0.45;
            // Flat ground base
            m.add_cuboid(0.0, s * 0.02, 0.0, hw, s * 0.02, hd, color);
            // Headstones scattered across the cemetery
            let stone_color = [0.6, 0.6, 0.6, 1.0];
            m.add_cuboid(-hw * 0.5, s * 0.08, -hd * 0.5, s * 0.03, s * 0.06, s * 0.015, stone_color);
            m.add_cuboid(-hw * 0.2, s * 0.08, -hd * 0.5, s * 0.03, s * 0.06, s * 0.015, stone_color);
            m.add_cuboid(hw * 0.1, s * 0.08, -hd * 0.5, s * 0.03, s * 0.06, s * 0.015, stone_color);
            m.add_cuboid(hw * 0.4, s * 0.08, -hd * 0.5, s * 0.03, s * 0.06, s * 0.015, stone_color);
            m.add_cuboid(-hw * 0.5, s * 0.08, 0.0, s * 0.03, s * 0.06, s * 0.015, stone_color);
            m.add_cuboid(-hw * 0.2, s * 0.08, 0.0, s * 0.03, s * 0.06, s * 0.015, stone_color);
            m.add_cuboid(hw * 0.1, s * 0.08, 0.0, s * 0.03, s * 0.06, s * 0.015, stone_color);
            m.add_cuboid(hw * 0.4, s * 0.08, 0.0, s * 0.03, s * 0.06, s * 0.015, stone_color);
            m.add_cuboid(-hw * 0.5, s * 0.08, hd * 0.5, s * 0.03, s * 0.06, s * 0.015, stone_color);
            m.add_cuboid(-hw * 0.2, s * 0.08, hd * 0.5, s * 0.03, s * 0.06, s * 0.015, stone_color);
            m.add_cuboid(hw * 0.1, s * 0.08, hd * 0.5, s * 0.03, s * 0.06, s * 0.015, stone_color);
            m.add_cuboid(hw * 0.4, s * 0.08, hd * 0.5, s * 0.03, s * 0.06, s * 0.015, stone_color);
            // Tree (cypress-style, tall and thin)
            m.add_cylinder(hw * 0.35, s * 0.15, hd * 0.35, s * 0.02, s * 0.2, 6, [0.35, 0.25, 0.15, 1.0]);
            m.add_cylinder(hw * 0.35, s * 0.35, hd * 0.35, s * 0.06, s * 0.25, 6, [0.15, 0.35, 0.12, 1.0]);
            // Gate/fence at entrance
            m.add_cuboid(0.0, s * 0.10, hd, hw * 0.15, s * 0.10, s * 0.02, darken(color, 0.6));
        }
        ServiceType::Crematorium => {
            // Dark red crematorium with chimney
            let color = [0.4, 0.25, 0.25, 1.0];
            let hw = s * 0.38;
            let hh = s * 0.30;
            let hd = s * 0.38;
            // Main building
            m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);
            // Tall chimney/smokestack
            m.add_cylinder(hw * 0.6, hh * 2.5, -hd * 0.4, s * 0.05, hh * 2.0, 8, [0.5, 0.45, 0.45, 1.0]);
            // Entrance
            m.add_cuboid(0.0, hh * 0.35, hd + 0.05, hw * 0.2, hh * 0.35, 0.05, darken(color, 0.4));
            // Peaked roof
            m.add_roof_prism(0.0, hh * 2.0, 0.0, hw * 1.02, hh * 0.3, hd * 1.02, darken(color, 0.8));
        }
        ServiceType::HomelessShelter => {
            // Warm-toned shelter building with beds visible
            let color = [0.65, 0.55, 0.45, 1.0];
            let hw = s * 0.40;
            let hh = s * 0.25;
            let hd = s * 0.40;
            // Main building
            m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);
            // Peaked roof
            m.add_roof_prism(0.0, hh * 2.0, 0.0, hw * 1.02, hh * 0.35, hd * 1.02, darken(color, 0.8));
            // Entrance (wide double door)
            m.add_cuboid(0.0, hh * 0.4, hd + 0.05, hw * 0.3, hh * 0.4, 0.05, darken(color, 0.4));
            // Windows
            m.add_cuboid(-hw * 0.5, hh, hd - 0.05, s * 0.04, s * 0.04, 0.08, [0.25, 0.25, 0.35, 1.0]);
            m.add_cuboid(hw * 0.5, hh, hd - 0.05, s * 0.04, s * 0.04, 0.08, [0.25, 0.25, 0.35, 1.0]);
            // Small sign
            m.add_cuboid(hw * 0.7, hh * 1.5, hd * 0.8, s * 0.06, s * 0.04, s * 0.01, [0.3, 0.6, 0.4, 1.0]);
        }
        ServiceType::WelfareOffice => {
            // Civic-style building in teal/green tones
            let color = [0.45, 0.60, 0.55, 1.0];
            let hw = s * 0.40;
            let hh = s * 0.30;
            let hd = s * 0.40;
            // Main building
            m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);
            // Entrance with columns
            m.add_cuboid(-hw * 0.25, hh, hd + s * 0.04, s * 0.025, hh, s * 0.025, [0.7, 0.7, 0.72, 1.0]);
            m.add_cuboid(hw * 0.25, hh, hd + s * 0.04, s * 0.025, hh, s * 0.025, [0.7, 0.7, 0.72, 1.0]);
            // Steps
            m.add_cuboid(0.0, hh * 0.12, hd + s * 0.07, hw * 0.5, hh * 0.12, s * 0.05, darken(color, 0.8));
            // Windows on each floor
            for i in 1..4 {
                let wx = -hw + i as f32 * hw * 0.5;
                m.add_cuboid(wx, hh, hd - 0.05, s * 0.035, s * 0.04, 0.08, [0.2, 0.22, 0.3, 1.0]);
            }
            // Flat roof with small sign
            m.add_cuboid(0.0, hh * 2.0 + s * 0.02, 0.0, hw * 0.3, s * 0.02, hd * 0.3, darken(color, 0.85));
            // Flag pole
            m.add_cylinder(hw * 0.7, hh * 2.5, hd * 0.7, s * 0.015, s * 0.5, 4, [0.5, 0.5, 0.55, 1.0]);
        }
        ServiceType::PostOffice => {
            // Classic post office: warm brick color with mail slot and flag
            let color = [0.72, 0.55, 0.38, 1.0];
            let hw = s * 0.38;
            let hh = s * 0.28;
            let hd = s * 0.38;
            // Main building
            m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);
            // Peaked roof
            m.add_roof_prism(0.0, hh * 2.0, 0.0, hw * 1.02, hh * 0.35, hd * 1.02, darken(color, 0.8));
            // Entrance door
            m.add_cuboid(0.0, hh * 0.4, hd + 0.05, hw * 0.2, hh * 0.4, 0.05, darken(color, 0.4));
            // Windows
            m.add_cuboid(-hw * 0.5, hh, hd - 0.05, s * 0.04, s * 0.04, 0.08, [0.2, 0.22, 0.3, 1.0]);
            m.add_cuboid(hw * 0.5, hh, hd - 0.05, s * 0.04, s * 0.04, 0.08, [0.2, 0.22, 0.3, 1.0]);
            // Mailbox (small blue box at entrance)
            m.add_cuboid(hw * 0.7, s * 0.08, hd + s * 0.06, s * 0.04, s * 0.08, s * 0.03, [0.2, 0.3, 0.7, 1.0]);
            // Flag pole
            m.add_cylinder(-hw * 0.7, hh * 2.5, hd * 0.7, s * 0.015, s * 0.5, 4, [0.6, 0.6, 0.6, 1.0]);
        }
        ServiceType::MailSortingCenter => {
            // Large industrial-style sorting center
            let color = [0.55, 0.50, 0.45, 1.0];
            let hw = s * 0.45;
            let hh = s * 0.30;
            let hd = s * 0.45;
            // Main warehouse building
            m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);
            // Flat roof
            m.add_cuboid(0.0, hh * 2.0 + s * 0.01, 0.0, hw * 1.02, s * 0.01, hd * 1.02, darken(color, 0.85));
            // Loading dock (raised platform at back)
            m.add_cuboid(0.0, hh * 0.3, -hd - s * 0.06, hw * 0.8, hh * 0.3, s * 0.06, darken(color, 0.7));
            // Loading bay doors
            m.add_cuboid(-hw * 0.4, hh * 0.5, hd + 0.05, hw * 0.2, hh * 0.5, 0.05, darken(color, 0.4));
            m.add_cuboid(hw * 0.4, hh * 0.5, hd + 0.05, hw * 0.2, hh * 0.5, 0.05, darken(color, 0.4));
            // Conveyor belt indicator on roof
            m.add_cuboid(0.0, hh * 2.0 + s * 0.06, 0.0, hw * 0.1, s * 0.04, hd * 0.6, [0.4, 0.4, 0.45, 1.0]);
            // Sorting center sign
            m.add_cuboid(0.0, hh * 1.8, hd + 0.03, hw * 0.3, s * 0.04, s * 0.01, [0.2, 0.3, 0.7, 1.0]);
        }
        ServiceType::HeatingBoiler => {
            // Small red-orange boiler building with chimney
            let color = [0.85, 0.40, 0.20, 1.0];
            let hw = s * 0.35;
            let hh = s * 0.25;
            let hd = s * 0.35;
            m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);
            // Chimney / smokestack
            m.add_cylinder(hw * 0.5, hh * 2.5, -hd * 0.3, s * 0.06, hh * 2.0, 8, [0.5, 0.5, 0.5, 1.0]);
            // Pipe network on side
            m.add_cuboid(-hw * 0.6, hh * 0.8, 0.0, s * 0.03, hh * 0.6, hd * 0.5, [0.6, 0.6, 0.65, 1.0]);
            // Door
            m.add_cuboid(0.0, hh * 0.35, hd + 0.05, hw * 0.2, hh * 0.35, 0.05, darken(color, 0.4));
        }
        ServiceType::DistrictHeatingPlant => {
            // Large industrial heating facility with two chimneys and pipes
            let color = [0.75, 0.35, 0.15, 1.0];
            let hw = s * 0.45 * scale_x;
            let hh = s * 0.35;
            let hd = s * 0.45 * scale_z;
            // Main building
            m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);
            // Two chimneys
            m.add_cylinder(-hw * 0.3, hh * 2.8, -hd * 0.3, s * 0.08, hh * 2.0, 8, [0.5, 0.5, 0.55, 1.0]);
            m.add_cylinder(hw * 0.3, hh * 2.5, -hd * 0.3, s * 0.06, hh * 1.8, 8, [0.55, 0.55, 0.6, 1.0]);
            // Pipe network along the front
            m.add_cuboid(0.0, hh * 0.6, hd + s * 0.04, hw * 0.8, s * 0.04, s * 0.04, [0.6, 0.6, 0.65, 1.0]);
            m.add_cuboid(0.0, hh * 1.0, hd + s * 0.04, hw * 0.8, s * 0.04, s * 0.04, [0.6, 0.6, 0.65, 1.0]);
            // Loading bay
            m.add_cuboid(hw * 0.5, hh * 0.4, hd + 0.05, hw * 0.3, hh * 0.4, 0.05, darken(color, 0.4));
        }
        ServiceType::GeothermalPlant => {
            // Geothermal facility with dome and pipes
            let color = [0.55, 0.40, 0.25, 1.0];
            let hw = s * 0.45 * scale_x;
            let hh = s * 0.30;
            let hd = s * 0.45 * scale_z;
            // Main building
            m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);
            // Geothermal dome (represents heat exchanger)
            m.add_cylinder(0.0, hh * 2.0 + s * 0.08, 0.0, s * 0.18, s * 0.14, 10, [0.65, 0.45, 0.30, 1.0]);
            // Steam vents (small cylinders)
            m.add_cylinder(-hw * 0.5, hh * 2.0, hd * 0.4, s * 0.04, s * 0.3, 6, [0.7, 0.7, 0.75, 1.0]);
            m.add_cylinder(hw * 0.5, hh * 2.0, -hd * 0.4, s * 0.04, s * 0.3, 6, [0.7, 0.7, 0.75, 1.0]);
            // Underground pipe indicators
            m.add_cuboid(0.0, s * 0.04, hd + s * 0.06, hw * 0.3, s * 0.04, s * 0.04, [0.5, 0.5, 0.55, 1.0]);
        }
        ServiceType::WaterTreatmentPlant => {
            // Large water treatment facility with settling tanks and processing building
            let color = [0.30, 0.55, 0.70, 1.0];
            let hw = s * 0.45 * scale_x;
            let hh = s * 0.25;
            let hd = s * 0.45 * scale_z;
            // Main processing building
            m.add_cuboid(-hw * 0.3, hh, 0.0, hw * 0.35, hh, hd * 0.6, color);
            // Circular settling tanks (two large cylinders)
            m.add_cylinder(hw * 0.25, s * 0.08, -hd * 0.35, s * 0.18, s * 0.08, 12, [0.35, 0.60, 0.75, 1.0]);
            m.add_cylinder(hw * 0.25, s * 0.08, hd * 0.35, s * 0.18, s * 0.08, 12, [0.35, 0.60, 0.75, 1.0]);
            // Tank rims (slightly darker)
            m.add_cylinder(hw * 0.25, s * 0.16, -hd * 0.35, s * 0.19, s * 0.01, 12, darken(color, 0.7));
            m.add_cylinder(hw * 0.25, s * 0.16, hd * 0.35, s * 0.19, s * 0.01, 12, darken(color, 0.7));
            // Pipe connecting tanks to building
            m.add_cuboid(0.0, hh * 0.5, 0.0, hw * 0.5, s * 0.03, s * 0.03, [0.5, 0.5, 0.55, 1.0]);
            // Outflow pipe
            m.add_cuboid(hw * 0.45, s * 0.06, hd * 0.6, s * 0.04, s * 0.04, s * 0.15, [0.4, 0.55, 0.65, 1.0]);
            // Small office/control room on top
            m.add_cuboid(-hw * 0.3, hh * 2.0 + s * 0.04, 0.0, hw * 0.15, s * 0.08, hd * 0.2, darken(color, 0.85));
        }
        ServiceType::WellPump => {
            // Small well pump station with pump housing and pipe
            let color = [0.40, 0.60, 0.55, 1.0];
            let hw = s * 0.30;
            let hh = s * 0.20;
            let hd = s * 0.30;
            // Concrete base/pad
            m.add_cuboid(0.0, s * 0.03, 0.0, hw, s * 0.03, hd, [0.55, 0.55, 0.55, 1.0]);
            // Pump housing (small building)
            m.add_cuboid(0.0, hh, 0.0, hw * 0.6, hh, hd * 0.6, color);
            // Pump motor on top
            m.add_cylinder(0.0, hh * 2.0 + s * 0.04, 0.0, s * 0.06, s * 0.06, 8, [0.5, 0.5, 0.55, 1.0]);
            // Pipe going into the ground
            m.add_cylinder(hw * 0.4, hh * 0.5, hd * 0.4, s * 0.03, hh * 1.5, 6, [0.45, 0.45, 0.50, 1.0]);
            // Horizontal output pipe
            m.add_cuboid(hw * 0.4, hh * 0.6, 0.0, s * 0.03, s * 0.03, hd * 0.6, [0.45, 0.45, 0.50, 1.0]);
            // Small access hatch on pump housing
            m.add_cuboid(0.0, hh * 0.35, hd * 0.6 + 0.05, hw * 0.15, hh * 0.35, 0.05, darken(color, 0.4));
        }
        _ => {
            let color = [0.91, 0.82, 0.38, 1.0];
            let hw = s * 0.4;
            let hh = s * 0.35;
            let hd = s * 0.4;
            m.add_cuboid(0.0, hh, 0.0, hw, hh, hd, color);
        }
    }

    m.into_mesh()
}

// ---------------------------------------------------------------------------
// Utility Building Meshes (procedural)
// ---------------------------------------------------------------------------

fn generate_utility_mesh(utility_type: UtilityType) -> Mesh {
    let mut m = MeshData::new();
    let s = CELL_SIZE;

    match utility_type {
        UtilityType::PowerPlant => {
            let color = [0.9, 0.5, 0.1, 1.0];
            m.add_cuboid(0.0, s * 0.25, 0.0, s * 0.4, s * 0.25, s * 0.35, color);
            m.add_cylinder(s * 0.2, s * 0.45, s * 0.15, s * 0.1, s * 0.4, 8, [0.6, 0.6, 0.6, 1.0]);
            m.add_cylinder(-s * 0.15, s * 0.45, -s * 0.1, s * 0.08, s * 0.35, 8, [0.6, 0.6, 0.6, 1.0]);
        }
        UtilityType::SolarFarm => {
            let color = [0.2, 0.25, 0.4, 1.0];
            for i in 0..3 {
                let z = (i as f32 - 1.0) * s * 0.25;
                m.add_cuboid(0.0, s * 0.1, z, s * 0.35, s * 0.01, s * 0.08, color);
            }
            m.add_cuboid(0.0, s * 0.05, 0.0, s * 0.02, s * 0.05, s * 0.02, [0.5, 0.5, 0.5, 1.0]);
        }
        UtilityType::WindTurbine => {
            let color = [0.85, 0.88, 0.9, 1.0];
            m.add_cylinder(0.0, s * 0.5, 0.0, s * 0.03, s * 1.0, 6, color);
            m.add_cuboid(0.0, s * 1.0, 0.0, s * 0.05, s * 0.04, s * 0.04, [0.7, 0.7, 0.7, 1.0]);
            m.add_cuboid(0.0, s * 1.0 + s * 0.2, s * 0.02, s * 0.015, s * 0.25, s * 0.015, color);
            m.add_cuboid(s * 0.17, s * 1.0 - s * 0.12, s * 0.02, s * 0.015, s * 0.12, s * 0.015, color);
            m.add_cuboid(-s * 0.17, s * 1.0 - s * 0.12, s * 0.02, s * 0.015, s * 0.12, s * 0.015, color);
        }
        UtilityType::WaterTower => {
            let color = [0.2, 0.7, 0.85, 1.0];
            for (dx, dz) in &[(0.08, 0.08), (-0.08, 0.08), (0.08, -0.08), (-0.08, -0.08)] {
                m.add_cylinder(dx * s, s * 0.2, dz * s, s * 0.02, s * 0.4, 4, [0.5, 0.5, 0.5, 1.0]);
            }
            m.add_cylinder(0.0, s * 0.5, 0.0, s * 0.15, s * 0.2, 8, color);
        }
        UtilityType::SewagePlant => {
            let color = [0.45, 0.55, 0.40, 1.0];
            m.add_cuboid(0.0, s * 0.15, 0.0, s * 0.4, s * 0.15, s * 0.35, color);
            m.add_cylinder(s * 0.15, s * 0.32, s * 0.12, s * 0.1, s * 0.02, 8, darken(color, 0.6));
        }
        UtilityType::NuclearPlant => {
            let color = [0.7, 0.7, 0.75, 1.0];
            m.add_cuboid(s * 0.15, s * 0.25, 0.0, s * 0.25, s * 0.25, s * 0.3, color);
            m.add_cylinder(-s * 0.15, s * 0.35, 0.0, s * 0.18, s * 0.3, 12, [0.75, 0.75, 0.8, 1.0]);
        }
        UtilityType::Geothermal => {
            let color = [0.65, 0.45, 0.30, 1.0];
            m.add_cuboid(0.0, s * 0.2, 0.0, s * 0.35, s * 0.2, s * 0.35, color);
            m.add_cylinder(s * 0.2, s * 0.4, s * 0.2, s * 0.04, s * 0.3, 6, [0.5, 0.5, 0.5, 1.0]);
        }
        UtilityType::PumpingStation => {
            let color = [0.3, 0.6, 0.8, 1.0];
            m.add_cuboid(0.0, s * 0.15, 0.0, s * 0.3, s * 0.15, s * 0.3, color);
        }
        UtilityType::WaterTreatment => {
            let color = [0.25, 0.55, 0.75, 1.0];
            m.add_cuboid(0.0, s * 0.2, 0.0, s * 0.4, s * 0.2, s * 0.35, color);
            m.add_cylinder(s * 0.15, s * 0.42, s * 0.1, s * 0.12, s * 0.02, 8, darken(color, 0.6));
        }
    }

    m.into_mesh()
}

// ---------------------------------------------------------------------------
// Color query functions for UI/minimap
// ---------------------------------------------------------------------------

pub fn zone_base_color(zone: ZoneType, _level: u8) -> Color {
    match zone {
        ZoneType::ResidentialLow | ZoneType::ResidentialHigh => Color::srgb(0.40, 0.72, 0.35),
        ZoneType::CommercialLow | ZoneType::CommercialHigh => Color::srgb(0.40, 0.52, 0.78),
        ZoneType::Industrial => Color::srgb(0.72, 0.65, 0.25),
        ZoneType::Office => Color::srgb(0.55, 0.50, 0.68),
        ZoneType::None => Color::srgb(0.7, 0.7, 0.7),
    }
}

pub fn service_base_color(service_type: ServiceType) -> Color {
    match service_type {
        ServiceType::FireStation | ServiceType::FireHouse | ServiceType::FireHQ =>
            Color::srgb(1.0, 0.50, 0.50),
        ServiceType::PoliceStation | ServiceType::PoliceKiosk | ServiceType::PoliceHQ =>
            Color::srgb(0.41, 0.53, 0.66),
        ServiceType::Prison => Color::srgb(0.45, 0.45, 0.45),
        ServiceType::Hospital | ServiceType::MedicalClinic | ServiceType::MedicalCenter =>
            Color::srgb(0.94, 0.69, 0.75),
        ServiceType::ElementarySchool | ServiceType::HighSchool | ServiceType::Library
        | ServiceType::Kindergarten => Color::srgb(0.94, 0.78, 0.63),
        ServiceType::University => Color::srgb(0.47, 0.47, 0.72),
        ServiceType::SmallPark | ServiceType::LargePark | ServiceType::Playground
        | ServiceType::SportsField => Color::srgb(0.44, 0.82, 0.44),
        ServiceType::Plaza | ServiceType::Stadium => Color::srgb(0.55, 0.75, 0.55),
        ServiceType::Landfill | ServiceType::RecyclingCenter | ServiceType::Incinerator
        | ServiceType::TransferStation => Color::srgb(0.60, 0.55, 0.45),
        ServiceType::Cemetery | ServiceType::Crematorium => Color::srgb(0.55, 0.55, 0.55),
        ServiceType::CityHall | ServiceType::Museum | ServiceType::Cathedral
        | ServiceType::TVStation => Color::srgb(0.91, 0.82, 0.38),
        ServiceType::BusDepot | ServiceType::TrainStation | ServiceType::SubwayStation
        | ServiceType::TramDepot | ServiceType::FerryPier => Color::srgb(0.50, 0.60, 0.70),
        ServiceType::SmallAirstrip | ServiceType::RegionalAirport | ServiceType::InternationalAirport =>
            Color::srgb(0.65, 0.65, 0.70),
        ServiceType::CellTower => Color::srgb(0.6, 0.6, 0.6),
        ServiceType::DataCenter => Color::srgb(0.35, 0.40, 0.50),
        ServiceType::HomelessShelter => Color::srgb(0.65, 0.55, 0.45),
        ServiceType::WelfareOffice => Color::srgb(0.45, 0.60, 0.55),
        ServiceType::PostOffice => Color::srgb(0.72, 0.55, 0.38),
        ServiceType::MailSortingCenter => Color::srgb(0.55, 0.50, 0.45),
        ServiceType::HeatingBoiler => Color::srgb(0.85, 0.40, 0.20),
        ServiceType::DistrictHeatingPlant => Color::srgb(0.75, 0.35, 0.15),
        ServiceType::GeothermalPlant => Color::srgb(0.55, 0.40, 0.25),
        ServiceType::WaterTreatmentPlant => Color::srgb(0.30, 0.55, 0.70),
        ServiceType::WellPump => Color::srgb(0.40, 0.60, 0.55),
    }
}

pub fn utility_base_color(utility_type: UtilityType) -> Color {
    match utility_type {
        UtilityType::PowerPlant => Color::srgb(0.9, 0.5, 0.1),
        UtilityType::SolarFarm => Color::srgb(0.95, 0.85, 0.2),
        UtilityType::WindTurbine => Color::srgb(0.7, 0.85, 0.95),
        UtilityType::WaterTower => Color::srgb(0.2, 0.7, 0.85),
        UtilityType::SewagePlant => Color::srgb(0.45, 0.55, 0.40),
        UtilityType::NuclearPlant => Color::srgb(0.7, 0.7, 0.75),
        UtilityType::Geothermal => Color::srgb(0.65, 0.45, 0.30),
        UtilityType::PumpingStation => Color::srgb(0.3, 0.6, 0.8),
        UtilityType::WaterTreatment => Color::srgb(0.25, 0.55, 0.75),
    }
}
