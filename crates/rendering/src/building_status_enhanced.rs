use bevy::prelude::*;
use std::collections::HashMap;

use simulation::abandonment::Abandoned;
use simulation::buildings::{Building, UnderConstruction};
use simulation::config::CELL_SIZE;
use simulation::crime::CrimeGrid;
use simulation::fire::OnFire;
use simulation::grid::WorldGrid;

use crate::camera::OrbitCamera;

// =============================================================================
// Components
// =============================================================================

/// Marker component for enhanced status-icon entities floating above buildings.
#[derive(Component)]
pub struct EnhancedStatusIcon {
    /// The simulation `Building` entity this icon belongs to.
    pub tracked_entity: Entity,
}

/// Cached status so we only mutate the world when something changes.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct LastEnhancedStatus {
    pub kind: EnhancedIconKind,
}

// =============================================================================
// Icon classification
// =============================================================================

/// All the enhanced status icon types (priority order, highest first).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnhancedIconKind {
    /// Building is on fire (bright orange-red).
    Fire,
    /// Building is under construction (amber/orange).
    UnderConstruction,
    /// Building cell has high crime (dark red/maroon).
    HighCrime,
    /// Building is at full capacity (teal/green).
    CapacityFull,
    /// Building is abandoned (grey).
    AbandonedIcon,
}

/// The icon cube is 1.5x1.5x1.5 world units and floats at y=28
/// (slightly above the utility icons at y=25 to avoid overlap).
const ICON_Y: f32 = 28.0;
const ICON_HALF_SIZE: f32 = 0.75;

/// Camera distance beyond which enhanced icons are hidden (LOD).
/// At medium zoom (~800 units) icons are visible; beyond that they are too small.
const MAX_VISIBLE_DISTANCE: f32 = 800.0;

/// Crime level threshold (0-255) above which the high-crime icon is shown.
const HIGH_CRIME_THRESHOLD: u8 = 60;

fn icon_color(kind: EnhancedIconKind) -> Color {
    match kind {
        EnhancedIconKind::Fire => Color::srgb(1.0, 0.35, 0.0), // bright orange-red
        EnhancedIconKind::UnderConstruction => Color::srgb(1.0, 0.75, 0.0), // amber
        EnhancedIconKind::HighCrime => Color::srgb(0.6, 0.0, 0.1), // dark maroon
        EnhancedIconKind::CapacityFull => Color::srgb(0.0, 0.8, 0.6), // teal
        EnhancedIconKind::AbandonedIcon => Color::srgb(0.5, 0.5, 0.5), // grey
    }
}

/// Determines the highest-priority enhanced status for a building.
/// Returns `None` if the building has no noteworthy enhanced status.
fn classify_enhanced(
    building: &Building,
    grid: &WorldGrid,
    crime_grid: &CrimeGrid,
    on_fire: bool,
    under_construction: bool,
    abandoned: bool,
) -> Option<EnhancedIconKind> {
    // Priority: fire > under construction > abandoned > high crime > capacity full
    if on_fire {
        return Some(EnhancedIconKind::Fire);
    }

    if under_construction {
        return Some(EnhancedIconKind::UnderConstruction);
    }

    if abandoned {
        return Some(EnhancedIconKind::AbandonedIcon);
    }

    let gx = building.grid_x;
    let gy = building.grid_y;
    if gx < grid.width && gy < grid.height {
        let crime_level = crime_grid.get(gx, gy);
        if crime_level >= HIGH_CRIME_THRESHOLD {
            return Some(EnhancedIconKind::HighCrime);
        }
    }

    // Capacity full: building is operational and completely full
    if building.capacity > 0 && building.occupants >= building.capacity {
        return Some(EnhancedIconKind::CapacityFull);
    }

    None
}

// =============================================================================
// Systems
// =============================================================================

/// Toggles visibility of all enhanced status icons based on camera zoom distance.
///
/// Icons are only visible at close/medium zoom (distance < MAX_VISIBLE_DISTANCE).
pub fn lod_enhanced_status_icons(
    orbit: Res<OrbitCamera>,
    mut icons: Query<&mut Visibility, With<EnhancedStatusIcon>>,
) {
    if !orbit.is_changed() {
        return;
    }

    let visible = orbit.distance < MAX_VISIBLE_DISTANCE;
    let target = if visible {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };

    for mut vis in &mut icons {
        if *vis != target {
            *vis = target;
        }
    }
}

/// System that keeps floating enhanced-status cubes in sync with building state.
///
/// Designed to run on a timer (every 2 s) rather than every frame. It:
///   1. Reads every `Building` entity and checks for fire, construction, crime, capacity.
///   2. Compares against cached `LastEnhancedStatus` on existing icon entities.
///   3. Spawns / despawns / recolors icons only when the status has actually changed.
#[allow(clippy::too_many_arguments)]
pub fn update_enhanced_status_icons(
    mut commands: Commands,
    buildings: Query<(Entity, &Building)>,
    fire_query: Query<Entity, With<OnFire>>,
    construction_query: Query<Entity, With<UnderConstruction>>,
    abandoned_query: Query<Entity, With<Abandoned>>,
    grid: Res<WorldGrid>,
    crime_grid: Res<CrimeGrid>,
    orbit: Res<OrbitCamera>,
    existing_icons: Query<(Entity, &EnhancedStatusIcon, &LastEnhancedStatus)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Build lookup: building entity -> (icon entity, cached kind)
    let mut icon_map: HashMap<Entity, (Entity, EnhancedIconKind)> =
        HashMap::with_capacity(existing_icons.iter().len());
    for (icon_entity, icon, last) in &existing_icons {
        icon_map.insert(icon.tracked_entity, (icon_entity, last.kind));
    }

    let initial_visible = orbit.distance < MAX_VISIBLE_DISTANCE;

    for (building_entity, building) in &buildings {
        let on_fire = fire_query.get(building_entity).is_ok();
        let under_construction = construction_query.get(building_entity).is_ok();
        let abandoned = abandoned_query.get(building_entity).is_ok();

        let kind = classify_enhanced(
            building,
            &grid,
            &crime_grid,
            on_fire,
            under_construction,
            abandoned,
        );

        if let Some((icon_entity, prev_kind)) = icon_map.remove(&building_entity) {
            // Icon already exists
            match kind {
                Some(k) if k == prev_kind => {
                    // No change -- skip.
                    continue;
                }
                Some(k) => {
                    // Status changed: despawn old, spawn new.
                    commands.entity(icon_entity).despawn();
                    spawn_enhanced_icon(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        building_entity,
                        building.grid_x,
                        building.grid_y,
                        k,
                        initial_visible,
                    );
                }
                None => {
                    // Status cleared: despawn icon.
                    commands.entity(icon_entity).despawn();
                }
            }
        } else if let Some(k) = kind {
            // No existing icon; spawn one.
            spawn_enhanced_icon(
                &mut commands,
                &mut meshes,
                &mut materials,
                building_entity,
                building.grid_x,
                building.grid_y,
                k,
                initial_visible,
            );
        }
    }

    // Clean up icons for buildings that no longer exist.
    for (_building_entity, (icon_entity, _)) in icon_map {
        commands.entity(icon_entity).despawn();
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn_enhanced_icon(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    building_entity: Entity,
    gx: usize,
    gy: usize,
    kind: EnhancedIconKind,
    visible: bool,
) {
    let (wx, _wy) = WorldGrid::grid_to_world(gx, gy);
    let wz = gy as f32 * CELL_SIZE + CELL_SIZE * 0.5;

    let color = icon_color(kind);
    let mesh = meshes.add(Cuboid::new(
        ICON_HALF_SIZE * 2.0,
        ICON_HALF_SIZE * 2.0,
        ICON_HALF_SIZE * 2.0,
    ));
    let material = materials.add(StandardMaterial {
        base_color: color,
        emissive: color.to_linear() * 2.0,
        unlit: true,
        ..default()
    });

    let visibility = if visible {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };

    commands.spawn((
        EnhancedStatusIcon {
            tracked_entity: building_entity,
        },
        LastEnhancedStatus { kind },
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_xyz(wx, ICON_Y, wz),
        visibility,
    ));
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use simulation::grid::ZoneType;

    fn make_building(gx: usize, gy: usize, capacity: u32, occupants: u32) -> Building {
        Building {
            zone_type: ZoneType::ResidentialLow,
            level: 1,
            grid_x: gx,
            grid_y: gy,
            capacity,
            occupants,
        }
    }

    #[test]
    fn test_icon_colors_are_distinct() {
        let kinds = [
            EnhancedIconKind::Fire,
            EnhancedIconKind::UnderConstruction,
            EnhancedIconKind::HighCrime,
            EnhancedIconKind::CapacityFull,
            EnhancedIconKind::AbandonedIcon,
        ];

        // Each kind should have a unique color
        let colors: Vec<Color> = kinds.iter().map(|k| icon_color(*k)).collect();
        for i in 0..colors.len() {
            for j in (i + 1)..colors.len() {
                assert_ne!(
                    colors[i], colors[j],
                    "Icon colors for {:?} and {:?} should be distinct",
                    kinds[i], kinds[j]
                );
            }
        }
    }

    #[test]
    fn test_classify_fire_highest_priority() {
        let grid = WorldGrid::new(256, 256);
        let crime_grid = CrimeGrid::default();
        let building = make_building(10, 10, 100, 100);
        // Fire should take priority even when construction + full capacity
        let result = classify_enhanced(&building, &grid, &crime_grid, true, true, false);
        assert_eq!(result, Some(EnhancedIconKind::Fire));
    }

    #[test]
    fn test_classify_under_construction() {
        let grid = WorldGrid::new(256, 256);
        let crime_grid = CrimeGrid::default();
        let building = make_building(10, 10, 100, 0);
        let result = classify_enhanced(&building, &grid, &crime_grid, false, true, false);
        assert_eq!(result, Some(EnhancedIconKind::UnderConstruction));
    }

    #[test]
    fn test_classify_abandoned() {
        let grid = WorldGrid::new(256, 256);
        let crime_grid = CrimeGrid::default();
        let building = make_building(10, 10, 100, 0);
        let result = classify_enhanced(&building, &grid, &crime_grid, false, false, true);
        assert_eq!(result, Some(EnhancedIconKind::AbandonedIcon));
    }

    #[test]
    fn test_classify_high_crime() {
        let grid = WorldGrid::new(256, 256);
        let mut crime_grid = CrimeGrid::default();
        crime_grid.set(10, 10, HIGH_CRIME_THRESHOLD);
        let building = make_building(10, 10, 100, 50);
        let result = classify_enhanced(&building, &grid, &crime_grid, false, false, false);
        assert_eq!(result, Some(EnhancedIconKind::HighCrime));
    }

    #[test]
    fn test_classify_crime_below_threshold() {
        let grid = WorldGrid::new(256, 256);
        let mut crime_grid = CrimeGrid::default();
        crime_grid.set(10, 10, HIGH_CRIME_THRESHOLD - 1);
        let building = make_building(10, 10, 100, 50);
        let result = classify_enhanced(&building, &grid, &crime_grid, false, false, false);
        assert_eq!(result, None);
    }

    #[test]
    fn test_classify_capacity_full() {
        let grid = WorldGrid::new(256, 256);
        let crime_grid = CrimeGrid::default();
        let building = make_building(10, 10, 100, 100);
        let result = classify_enhanced(&building, &grid, &crime_grid, false, false, false);
        assert_eq!(result, Some(EnhancedIconKind::CapacityFull));
    }

    #[test]
    fn test_classify_capacity_over_full() {
        let grid = WorldGrid::new(256, 256);
        let crime_grid = CrimeGrid::default();
        let building = make_building(10, 10, 100, 150);
        let result = classify_enhanced(&building, &grid, &crime_grid, false, false, false);
        assert_eq!(result, Some(EnhancedIconKind::CapacityFull));
    }

    #[test]
    fn test_classify_no_status() {
        let grid = WorldGrid::new(256, 256);
        let crime_grid = CrimeGrid::default();
        let building = make_building(10, 10, 100, 50);
        let result = classify_enhanced(&building, &grid, &crime_grid, false, false, false);
        assert_eq!(result, None);
    }

    #[test]
    fn test_classify_zero_capacity_not_full() {
        let grid = WorldGrid::new(256, 256);
        let crime_grid = CrimeGrid::default();
        let building = make_building(10, 10, 0, 0);

        // Zero capacity should not show as full
        let result = classify_enhanced(&building, &grid, &crime_grid, false, false, false);
        assert_eq!(result, None);
    }

    #[test]
    fn test_classify_priority_order() {
        let grid = WorldGrid::new(256, 256);
        let mut crime_grid = CrimeGrid::default();
        crime_grid.set(10, 10, HIGH_CRIME_THRESHOLD + 10);
        let building = make_building(10, 10, 100, 100); // full capacity + high crime

        // High crime should take priority over capacity full
        let result = classify_enhanced(&building, &grid, &crime_grid, false, false, false);
        assert_eq!(result, Some(EnhancedIconKind::HighCrime));

        // Under construction should take priority over high crime
        let result = classify_enhanced(&building, &grid, &crime_grid, false, true, false);
        assert_eq!(result, Some(EnhancedIconKind::UnderConstruction));

        // Abandoned should be between construction and crime
        let result = classify_enhanced(&building, &grid, &crime_grid, false, false, true);
        assert_eq!(result, Some(EnhancedIconKind::AbandonedIcon));
    }

    #[test]
    fn test_lod_threshold() {
        // Verify the constant is reasonable
        assert!(MAX_VISIBLE_DISTANCE > 0.0);
        assert!(MAX_VISIBLE_DISTANCE < 4000.0); // less than max camera distance
    }
}
