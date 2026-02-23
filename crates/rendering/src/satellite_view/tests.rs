//! Tests for satellite view rendering.

#[cfg(test)]
mod tests {
    use simulation::grid::{CellType, ZoneType};
    use simulation::weather::Weather;

    use crate::satellite_view::colors::{
        road_satellite_color, road_satellite_width, satellite_terrain_color, to_rgba8,
        zone_satellite_color,
    };
    use crate::satellite_view::image_gen::create_blank_image;
    use crate::satellite_view::painting::{paint_circle, paint_grid_cell};
    use crate::satellite_view::types::{TEX_SIZE, TRANSITION_END, TRANSITION_START};

    #[test]
    fn test_transition_constants_are_ordered() {
        assert!(TRANSITION_START > 0.0);
        assert!(TRANSITION_END > TRANSITION_START);
    }

    #[test]
    fn test_zone_satellite_color_produces_valid_rgba() {
        let zones = [
            ZoneType::ResidentialLow,
            ZoneType::ResidentialMedium,
            ZoneType::ResidentialHigh,
            ZoneType::CommercialLow,
            ZoneType::CommercialHigh,
            ZoneType::Industrial,
            ZoneType::Office,
            ZoneType::MixedUse,
        ];
        for zone in zones {
            for level in 1..=5 {
                let color = zone_satellite_color(zone, level);
                assert_eq!(color[3], 255, "Alpha should be fully opaque");
            }
        }
    }

    #[test]
    fn test_road_satellite_width_increases_with_road_type() {
        use simulation::grid::RoadType;
        let path_w = road_satellite_width(RoadType::Path);
        let local_w = road_satellite_width(RoadType::Local);
        let avenue_w = road_satellite_width(RoadType::Avenue);
        let highway_w = road_satellite_width(RoadType::Highway);
        assert!(path_w < local_w);
        assert!(local_w < avenue_w);
        assert!(avenue_w < highway_w);
    }

    #[test]
    fn test_create_blank_image_dimensions() {
        let img = create_blank_image();
        assert_eq!(img.width(), TEX_SIZE as u32);
        assert_eq!(img.height(), TEX_SIZE as u32);
    }

    #[test]
    fn test_to_rgba8_clamping() {
        let c = to_rgba8(1.5, -0.1, 0.5);
        assert_eq!(c[0], 255);
        assert_eq!(c[1], 0);
        assert_eq!(c[2], 127);
        assert_eq!(c[3], 255);
    }

    #[test]
    fn test_satellite_terrain_color_water() {
        let cell = simulation::grid::Cell {
            cell_type: CellType::Water,
            elevation: 0.2,
            zone: ZoneType::None,
            ..Default::default()
        };
        let weather = Weather::default();
        let color = satellite_terrain_color(&cell, &weather);
        assert!(color[2] > color[0], "Water blue channel should exceed red");
    }

    #[test]
    fn test_satellite_terrain_color_grass() {
        let cell = simulation::grid::Cell {
            cell_type: CellType::Grass,
            elevation: 0.5,
            zone: ZoneType::None,
            ..Default::default()
        };
        let weather = Weather::default();
        let color = satellite_terrain_color(&cell, &weather);
        assert!(color[1] > color[0], "Grass green channel should exceed red");
    }

    #[test]
    fn test_satellite_terrain_color_road() {
        let cell = simulation::grid::Cell {
            cell_type: CellType::Road,
            elevation: 0.5,
            zone: ZoneType::None,
            ..Default::default()
        };
        let weather = Weather::default();
        let color = satellite_terrain_color(&cell, &weather);
        assert!(color[0] < 128 && color[1] < 128 && color[2] < 128);
    }

    #[test]
    fn test_paint_grid_cell_within_bounds() {
        let size = 16;
        let mut pixels = vec![[0u8; 4]; size * size];
        let scale_x = 256.0 / size as f32;
        let scale_y = 256.0 / size as f32;
        paint_grid_cell(&mut pixels, size, scale_x, scale_y, 0, 0, [255, 0, 0, 255]);
        assert_eq!(pixels[0], [255, 0, 0, 255]);
    }

    #[test]
    fn test_paint_circle_center_pixel() {
        let size = 16;
        let mut pixels = vec![[0u8; 4]; size * size];
        paint_circle(&mut pixels, size, 8.0, 8.0, 1.0, [0, 255, 0, 255]);
        // Center pixel should be painted
        assert_eq!(pixels[8 * size + 8], [0, 255, 0, 255]);
    }

    #[test]
    fn test_road_colors_are_opaque() {
        use simulation::grid::RoadType;
        let types = [
            RoadType::Path,
            RoadType::OneWay,
            RoadType::Local,
            RoadType::Avenue,
            RoadType::Boulevard,
            RoadType::Highway,
        ];
        for rt in types {
            let c = road_satellite_color(rt);
            assert_eq!(c[3], 255);
        }
    }
}
