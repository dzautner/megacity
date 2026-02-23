#[cfg(test)]
mod tests {
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
    // 1. Default / basic state tests
    // =========================================================================

    #[test]
    fn test_default_has_no_connections() {
        let outside = OutsideConnections::default();
        assert!(outside.connections.is_empty());
        assert!(!outside.has_connection(ConnectionType::Highway));
        assert!(!outside.has_connection(ConnectionType::Railway));
        assert!(!outside.has_connection(ConnectionType::SeaPort));
        assert!(!outside.has_connection(ConnectionType::Airport));
        assert_eq!(outside.count(ConnectionType::Highway), 0);
        assert_eq!(outside.avg_utilization(ConnectionType::Highway), 0.0);
    }

    #[test]
    fn test_all_connection_types() {
        let all = ConnectionType::all();
        assert_eq!(all.len(), 4);
        assert!(all.contains(&ConnectionType::Highway));
        assert!(all.contains(&ConnectionType::Railway));
        assert!(all.contains(&ConnectionType::SeaPort));
        assert!(all.contains(&ConnectionType::Airport));
    }

    #[test]
    fn test_connection_type_names() {
        assert_eq!(ConnectionType::Highway.name(), "Highway");
        assert_eq!(ConnectionType::Railway.name(), "Railway");
        assert_eq!(ConnectionType::SeaPort.name(), "Sea Port");
        assert_eq!(ConnectionType::Airport.name(), "Airport");
    }

    #[test]
    fn test_effect_descriptions_exist() {
        for &ct in ConnectionType::all() {
            let desc = OutsideConnections::effect_description(ct);
            assert!(!desc.is_empty());
        }
    }

    // =========================================================================
    // 2. has_connection / count / avg_utilization queries
    // =========================================================================

    #[test]
    fn test_has_connection_returns_true_when_present() {
        let mut outside = OutsideConnections::default();
        outside
            .connections
            .push(make_conn(ConnectionType::Railway, 10, 0, 2000, 0.3));
        assert!(outside.has_connection(ConnectionType::Railway));
        assert!(!outside.has_connection(ConnectionType::Highway));
    }

    #[test]
    fn test_count_multiple_connections_of_same_type() {
        let mut outside = OutsideConnections::default();
        outside
            .connections
            .push(make_conn(ConnectionType::Highway, 50, 0, 5000, 0.2));
        outside
            .connections
            .push(make_conn(ConnectionType::Highway, 200, 0, 5000, 0.8));
        outside
            .connections
            .push(make_conn(ConnectionType::Railway, 10, 0, 2000, 0.5));
        assert_eq!(outside.count(ConnectionType::Highway), 2);
        assert_eq!(outside.count(ConnectionType::Railway), 1);
        assert_eq!(outside.count(ConnectionType::Airport), 0);
    }

    #[test]
    fn test_avg_utilization_single_connection() {
        let mut outside = OutsideConnections::default();
        outside
            .connections
            .push(make_conn(ConnectionType::Airport, 100, 100, 5000, 0.75));
        assert!((outside.avg_utilization(ConnectionType::Airport) - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_avg_utilization_multiple_connections() {
        let mut outside = OutsideConnections::default();
        outside
            .connections
            .push(make_conn(ConnectionType::Highway, 50, 0, 5000, 0.2));
        outside
            .connections
            .push(make_conn(ConnectionType::Highway, 200, 0, 5000, 0.8));
        // Average of 0.2 and 0.8 = 0.5
        assert!((outside.avg_utilization(ConnectionType::Highway) - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_avg_utilization_no_connections_returns_zero() {
        let outside = OutsideConnections::default();
        assert_eq!(outside.avg_utilization(ConnectionType::SeaPort), 0.0);
    }
}
