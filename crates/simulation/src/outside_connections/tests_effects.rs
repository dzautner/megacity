#[cfg(test)]
mod tests {
    use crate::outside_connections::effects::ConnectionEffects;
    use crate::outside_connections::*;

    // =========================================================================
    // Helper: create an OutsideConnection with given type, position, capacity
    // =========================================================================
    fn make_conn(
        ct: ConnectionType,
        x: usize,
        y: usize,
        capacity: u32,
        utilization: f32,
    ) -> OutsideConnection {
        OutsideConnection {
            connection_type: ct,
            grid_x: x,
            grid_y: y,
            capacity,
            utilization,
        }
    }

    // =========================================================================
    // 9. ConnectionEffects computation
    // =========================================================================

    #[test]
    fn test_connection_effects_no_connections() {
        let empty = OutsideConnections::default();
        let effects = ConnectionEffects::compute(&empty);
        assert!((effects.import_cost_multiplier - 1.0).abs() < 0.001);
        assert!((effects.tourism_bonus - 0.0).abs() < 0.001);
        assert!((effects.attractiveness_bonus - 0.0).abs() < 0.001);
        assert!((effects.immigration_multiplier - 1.0).abs() < 0.001);
        assert!((effects.industrial_production_bonus - 1.0).abs() < 0.001);
        assert!((effects.export_price_multiplier - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_connection_effects_highway_only() {
        let mut conns = OutsideConnections::default();
        conns
            .connections
            .push(make_conn(ConnectionType::Highway, 185, 0, 5000, 0.5));
        let effects = ConnectionEffects::compute(&conns);
        assert!(
            (effects.immigration_multiplier - 1.20).abs() < 0.001,
            "Highway should boost immigration by 20%"
        );
        assert!(
            (effects.attractiveness_bonus - 5.0).abs() < 0.001,
            "Highway should add 5.0 attractiveness"
        );
        // Other effects should remain at default
        assert!((effects.import_cost_multiplier - 1.0).abs() < 0.001);
        assert!((effects.tourism_bonus - 0.0).abs() < 0.001);
        assert!((effects.industrial_production_bonus - 1.0).abs() < 0.001);
        assert!((effects.export_price_multiplier - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_connection_effects_railway_only() {
        let mut conns = OutsideConnections::default();
        conns
            .connections
            .push(make_conn(ConnectionType::Railway, 145, 0, 2000, 0.4));
        let effects = ConnectionEffects::compute(&conns);
        assert!(
            (effects.import_cost_multiplier - 0.85).abs() < 0.001,
            "Railway should reduce imports by 15%"
        );
        assert!(
            (effects.tourism_bonus - 10.0).abs() < 0.001,
            "Railway should add 10 tourism"
        );
        assert!(
            (effects.attractiveness_bonus - 3.0).abs() < 0.001,
            "Railway should add 3.0 attractiveness"
        );
    }

    #[test]
    fn test_connection_effects_seaport_only() {
        let mut conns = OutsideConnections::default();
        conns
            .connections
            .push(make_conn(ConnectionType::SeaPort, 55, 0, 3000, 0.2));
        let effects = ConnectionEffects::compute(&conns);
        assert!(
            (effects.import_cost_multiplier - 0.50).abs() < 0.001,
            "SeaPort should halve import costs"
        );
        assert!(
            (effects.industrial_production_bonus - 1.15).abs() < 0.001,
            "SeaPort should boost industrial production by 15%"
        );
        assert!(
            (effects.attractiveness_bonus - 4.0).abs() < 0.001,
            "SeaPort should add 4.0 attractiveness"
        );
    }

    #[test]
    fn test_connection_effects_airport_only() {
        let mut conns = OutsideConnections::default();
        conns
            .connections
            .push(make_conn(ConnectionType::Airport, 100, 100, 5000, 0.3));
        let effects = ConnectionEffects::compute(&conns);
        assert!(
            (effects.tourism_bonus - 30.0).abs() < 0.001,
            "Airport should add 30 tourism"
        );
        assert!(
            (effects.export_price_multiplier - 1.20).abs() < 0.001,
            "Airport should boost export prices by 20%"
        );
        assert!(
            (effects.attractiveness_bonus - 8.0).abs() < 0.001,
            "Airport should add 8.0 attractiveness"
        );
    }

    #[test]
    fn test_connection_effects_railway_plus_seaport_combined() {
        let mut conns = OutsideConnections::default();
        conns
            .connections
            .push(make_conn(ConnectionType::Railway, 145, 0, 2000, 0.4));
        conns
            .connections
            .push(make_conn(ConnectionType::SeaPort, 55, 50, 3000, 0.2));
        let effects = ConnectionEffects::compute(&conns);
        // Railway: 0.85 * SeaPort: 0.50 = 0.425
        assert!(
            (effects.import_cost_multiplier - 0.425).abs() < 0.001,
            "Combined import cost should be 0.85 * 0.50 = 0.425"
        );
        // Tourism: Railway 10 only (SeaPort adds none)
        assert!((effects.tourism_bonus - 10.0).abs() < 0.001);
        // Attractiveness: Railway 3 + SeaPort 4 = 7
        assert!((effects.attractiveness_bonus - 7.0).abs() < 0.001);
        // Industrial: SeaPort 1.15 only
        assert!((effects.industrial_production_bonus - 1.15).abs() < 0.001);
    }

    #[test]
    fn test_connection_effects_all_four_types_combined() {
        let mut conns = OutsideConnections::default();
        conns
            .connections
            .push(make_conn(ConnectionType::Highway, 185, 0, 5000, 0.5));
        conns
            .connections
            .push(make_conn(ConnectionType::Railway, 145, 0, 2000, 0.4));
        conns
            .connections
            .push(make_conn(ConnectionType::SeaPort, 55, 50, 3000, 0.2));
        conns
            .connections
            .push(make_conn(ConnectionType::Airport, 100, 100, 5000, 0.3));

        let effects = ConnectionEffects::compute(&conns);

        // Import cost: Railway 0.85 * SeaPort 0.50 = 0.425
        assert!(
            (effects.import_cost_multiplier - 0.425).abs() < 0.001,
            "All types: import cost should be 0.425"
        );
        // Tourism: Railway 10 + Airport 30 = 40
        assert!(
            (effects.tourism_bonus - 40.0).abs() < 0.001,
            "All types: tourism should be 40"
        );
        // Attractiveness: Highway 5 + Railway 3 + SeaPort 4 + Airport 8 = 20
        assert!(
            (effects.attractiveness_bonus - 20.0).abs() < 0.001,
            "All types: attractiveness should be 20"
        );
        // Immigration: 1.0 + Highway 0.20 = 1.20
        assert!(
            (effects.immigration_multiplier - 1.20).abs() < 0.001,
            "All types: immigration multiplier should be 1.20"
        );
        // Industrial: 1.0 + SeaPort 0.15 = 1.15
        assert!(
            (effects.industrial_production_bonus - 1.15).abs() < 0.001,
            "All types: industrial bonus should be 1.15"
        );
        // Export price: 1.0 + Airport 0.20 = 1.20
        assert!(
            (effects.export_price_multiplier - 1.20).abs() < 0.001,
            "All types: export price multiplier should be 1.20"
        );
    }

    #[test]
    fn test_connection_effects_duplicate_types_only_counted_once() {
        // Having two highways shouldn't double the effect (it's presence-based, not count-based)
        let mut conns = OutsideConnections::default();
        conns
            .connections
            .push(make_conn(ConnectionType::Highway, 50, 0, 5000, 0.3));
        conns
            .connections
            .push(make_conn(ConnectionType::Highway, 200, 0, 5000, 0.7));
        let effects = ConnectionEffects::compute(&conns);
        // Should still be 1.20 (not 1.40)
        assert!(
            (effects.immigration_multiplier - 1.20).abs() < 0.001,
            "Duplicate highways should not stack effects"
        );
    }
}
