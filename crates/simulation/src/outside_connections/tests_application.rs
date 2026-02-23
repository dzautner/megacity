#[cfg(test)]
mod tests {
    use crate::immigration::CityAttractiveness;
    use crate::natural_resources::ResourceBalance;
    use crate::outside_connections::effects::ConnectionEffects;
    use crate::outside_connections::*;
    use crate::tourism::Tourism;

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
    // 10. Import/export flow — effects on resource balance and trade
    // =========================================================================

    #[test]
    fn test_import_cost_reduction_applied_to_resource_balance() {
        // Simulate the effect application logic from update_outside_connections
        let mut resource_balance = ResourceBalance {
            fuel_consumption: 100.0,
            metal_consumption: 80.0,
            food_production: 50.0,
            food_consumption: 0.0,
            timber_production: 0.0,
            timber_consumption: 0.0,
            metal_production: 0.0,
            fuel_production: 0.0,
        };

        // Railway + SeaPort => import_cost_multiplier = 0.425
        let effects = ConnectionEffects {
            import_cost_multiplier: 0.425,
            tourism_bonus: 0.0,
            attractiveness_bonus: 0.0,
            immigration_multiplier: 1.0,
            industrial_production_bonus: 1.0,
            export_price_multiplier: 1.0,
        };

        // Apply the same logic as the system
        if effects.import_cost_multiplier < 1.0 {
            let reduction = 1.0 - effects.import_cost_multiplier;
            resource_balance.fuel_consumption *= 1.0 - reduction * 0.3;
            resource_balance.metal_consumption *= 1.0 - reduction * 0.3;
        }

        // reduction = 0.575, so multiplier = 1.0 - 0.575*0.3 = 0.8275
        let expected_fuel = 100.0 * (1.0 - 0.575 * 0.3);
        let expected_metal = 80.0 * (1.0 - 0.575 * 0.3);
        assert!(
            (resource_balance.fuel_consumption - expected_fuel).abs() < 0.01,
            "Fuel consumption should be reduced to {expected_fuel}, got {}",
            resource_balance.fuel_consumption
        );
        assert!(
            (resource_balance.metal_consumption - expected_metal).abs() < 0.01,
            "Metal consumption should be reduced to {expected_metal}, got {}",
            resource_balance.metal_consumption
        );
    }

    #[test]
    fn test_no_import_cost_reduction_when_multiplier_is_one() {
        let mut resource_balance = ResourceBalance {
            fuel_consumption: 100.0,
            metal_consumption: 80.0,
            food_production: 0.0,
            food_consumption: 0.0,
            timber_production: 0.0,
            timber_consumption: 0.0,
            metal_production: 0.0,
            fuel_production: 0.0,
        };

        let effects = ConnectionEffects {
            import_cost_multiplier: 1.0,
            tourism_bonus: 0.0,
            attractiveness_bonus: 0.0,
            immigration_multiplier: 1.0,
            industrial_production_bonus: 1.0,
            export_price_multiplier: 1.0,
        };

        // Should NOT modify consumption
        if effects.import_cost_multiplier < 1.0 {
            let reduction = 1.0 - effects.import_cost_multiplier;
            resource_balance.fuel_consumption *= 1.0 - reduction * 0.3;
            resource_balance.metal_consumption *= 1.0 - reduction * 0.3;
        }

        assert!(
            (resource_balance.fuel_consumption - 100.0).abs() < f32::EPSILON,
            "No reduction should be applied when multiplier is 1.0"
        );
        assert!(
            (resource_balance.metal_consumption - 80.0).abs() < f32::EPSILON,
            "No reduction should be applied when multiplier is 1.0"
        );
    }

    #[test]
    fn test_industrial_production_bonus_applied_to_resources() {
        let mut resource_balance = ResourceBalance {
            food_production: 100.0,
            timber_production: 80.0,
            metal_production: 60.0,
            fuel_production: 40.0,
            food_consumption: 0.0,
            timber_consumption: 0.0,
            metal_consumption: 0.0,
            fuel_consumption: 0.0,
        };

        // SeaPort: industrial_production_bonus = 1.15
        let effects = ConnectionEffects {
            import_cost_multiplier: 1.0,
            tourism_bonus: 0.0,
            attractiveness_bonus: 0.0,
            immigration_multiplier: 1.0,
            industrial_production_bonus: 1.15,
            export_price_multiplier: 1.0,
        };

        // Apply the same logic as the system
        if effects.industrial_production_bonus > 1.0 {
            let bonus = effects.industrial_production_bonus - 1.0;
            resource_balance.food_production *= 1.0 + bonus;
            resource_balance.timber_production *= 1.0 + bonus;
            resource_balance.metal_production *= 1.0 + bonus;
            resource_balance.fuel_production *= 1.0 + bonus;
        }

        // bonus = 0.15, multiplier = 1.15
        assert!(
            (resource_balance.food_production - 115.0).abs() < 0.01,
            "Food production should be boosted by 15%"
        );
        assert!(
            (resource_balance.timber_production - 92.0).abs() < 0.01,
            "Timber production should be boosted by 15%"
        );
        assert!(
            (resource_balance.metal_production - 69.0).abs() < 0.01,
            "Metal production should be boosted by 15%"
        );
        assert!(
            (resource_balance.fuel_production - 46.0).abs() < 0.01,
            "Fuel production should be boosted by 15%"
        );
    }

    #[test]
    fn test_no_industrial_bonus_when_multiplier_is_one() {
        let mut resource_balance = ResourceBalance {
            food_production: 100.0,
            timber_production: 80.0,
            metal_production: 60.0,
            fuel_production: 40.0,
            food_consumption: 0.0,
            timber_consumption: 0.0,
            metal_consumption: 0.0,
            fuel_consumption: 0.0,
        };

        let effects = ConnectionEffects {
            import_cost_multiplier: 1.0,
            tourism_bonus: 0.0,
            attractiveness_bonus: 0.0,
            immigration_multiplier: 1.0,
            industrial_production_bonus: 1.0,
            export_price_multiplier: 1.0,
        };

        if effects.industrial_production_bonus > 1.0 {
            let bonus = effects.industrial_production_bonus - 1.0;
            resource_balance.food_production *= 1.0 + bonus;
        }

        assert!(
            (resource_balance.food_production - 100.0).abs() < f32::EPSILON,
            "No bonus should be applied when multiplier is 1.0"
        );
    }

    // =========================================================================
    // 11. Trade balance calculation
    // =========================================================================

    #[test]
    fn test_export_price_multiplier_applied_to_trade_balance() {
        let mut trade_balance: f64 = 1000.0;

        // Airport: export_price_multiplier = 1.20
        let effects = ConnectionEffects {
            import_cost_multiplier: 1.0,
            tourism_bonus: 0.0,
            attractiveness_bonus: 0.0,
            immigration_multiplier: 1.0,
            industrial_production_bonus: 1.0,
            export_price_multiplier: 1.20,
        };

        // Apply the same logic as the system
        if effects.export_price_multiplier > 1.0 {
            let bonus_factor = effects.export_price_multiplier;
            trade_balance *= bonus_factor as f64;
        }

        assert!(
            (trade_balance - 1200.0).abs() < 0.01,
            "Trade balance should be boosted by 20% from airport"
        );
    }

    #[test]
    fn test_export_price_multiplier_on_negative_trade_balance() {
        let mut trade_balance: f64 = -500.0;

        let effects = ConnectionEffects {
            import_cost_multiplier: 1.0,
            tourism_bonus: 0.0,
            attractiveness_bonus: 0.0,
            immigration_multiplier: 1.0,
            industrial_production_bonus: 1.0,
            export_price_multiplier: 1.20,
        };

        if effects.export_price_multiplier > 1.0 {
            let bonus_factor = effects.export_price_multiplier;
            trade_balance *= bonus_factor as f64;
        }

        // Negative trade balance gets more negative (amplified deficit)
        assert!(
            (trade_balance - (-600.0)).abs() < 0.01,
            "Negative trade balance should also be multiplied"
        );
    }

    #[test]
    fn test_no_export_multiplier_when_at_one() {
        let mut trade_balance: f64 = 1000.0;

        let effects = ConnectionEffects {
            import_cost_multiplier: 1.0,
            tourism_bonus: 0.0,
            attractiveness_bonus: 0.0,
            immigration_multiplier: 1.0,
            industrial_production_bonus: 1.0,
            export_price_multiplier: 1.0,
        };

        if effects.export_price_multiplier > 1.0 {
            trade_balance *= effects.export_price_multiplier as f64;
        }

        assert!(
            (trade_balance - 1000.0).abs() < f64::EPSILON,
            "Trade balance should not change with multiplier = 1.0"
        );
    }

    #[test]
    fn test_tourism_bonus_applied_and_capped() {
        let mut tourism = Tourism::default();
        assert_eq!(tourism.attractiveness, 0.0);

        // Airport gives +30 tourism bonus
        let effects = ConnectionEffects {
            import_cost_multiplier: 1.0,
            tourism_bonus: 30.0,
            attractiveness_bonus: 0.0,
            immigration_multiplier: 1.0,
            industrial_production_bonus: 1.0,
            export_price_multiplier: 1.0,
        };

        // Apply same logic as system
        tourism.attractiveness = (tourism.attractiveness + effects.tourism_bonus).min(100.0);
        tourism.monthly_visitors = (tourism.attractiveness * 50.0) as u32;

        assert!((tourism.attractiveness - 30.0).abs() < 0.001);
        assert_eq!(tourism.monthly_visitors, 1500);
    }

    #[test]
    fn test_tourism_attractiveness_capped_at_100() {
        let mut tourism = Tourism {
            attractiveness: 85.0,
            ..Default::default()
        };

        // Railway(10) + Airport(30) = 40 tourism bonus, but 85 + 40 > 100
        let effects = ConnectionEffects {
            import_cost_multiplier: 1.0,
            tourism_bonus: 40.0,
            attractiveness_bonus: 0.0,
            immigration_multiplier: 1.0,
            industrial_production_bonus: 1.0,
            export_price_multiplier: 1.0,
        };

        tourism.attractiveness = (tourism.attractiveness + effects.tourism_bonus).min(100.0);
        assert!(
            (tourism.attractiveness - 100.0).abs() < 0.001,
            "Tourism attractiveness should be capped at 100"
        );
    }

    #[test]
    fn test_attractiveness_bonus_applied_and_clamped() {
        let mut attractiveness = CityAttractiveness::default();
        assert!((attractiveness.overall_score - 50.0).abs() < 0.001);

        // All four types: attractiveness bonus = 20
        let effects = ConnectionEffects {
            import_cost_multiplier: 1.0,
            tourism_bonus: 0.0,
            attractiveness_bonus: 20.0,
            immigration_multiplier: 1.0,
            industrial_production_bonus: 1.0,
            export_price_multiplier: 1.0,
        };

        attractiveness.overall_score =
            (attractiveness.overall_score + effects.attractiveness_bonus).clamp(0.0, 100.0);
        assert!(
            (attractiveness.overall_score - 70.0).abs() < 0.001,
            "Attractiveness should go from 50 to 70"
        );
    }

    #[test]
    fn test_attractiveness_clamped_at_100() {
        let mut attractiveness = CityAttractiveness {
            overall_score: 90.0,
            ..Default::default()
        };

        let effects = ConnectionEffects {
            import_cost_multiplier: 1.0,
            tourism_bonus: 0.0,
            attractiveness_bonus: 20.0,
            immigration_multiplier: 1.0,
            industrial_production_bonus: 1.0,
            export_price_multiplier: 1.0,
        };

        attractiveness.overall_score =
            (attractiveness.overall_score + effects.attractiveness_bonus).clamp(0.0, 100.0);
        assert!(
            (attractiveness.overall_score - 100.0).abs() < 0.001,
            "Attractiveness should be clamped at 100"
        );
    }

    // =========================================================================
    // 12. Connection stats / UI summary
    // =========================================================================

    #[test]
    fn test_connection_stats_summary() {
        let mut outside = OutsideConnections::default();
        outside
            .connections
            .push(make_conn(ConnectionType::Highway, 185, 0, 5000, 0.6));
        outside
            .connections
            .push(make_conn(ConnectionType::Highway, 185, 255, 5000, 0.4));

        let stats = outside.stats();
        assert_eq!(stats.len(), 4);

        let highway_stat = stats
            .iter()
            .find(|s| s.connection_type == ConnectionType::Highway)
            .unwrap();
        assert!(highway_stat.active);
        assert_eq!(highway_stat.count, 2);
        assert!((highway_stat.avg_utilization - 0.5).abs() < 0.001);

        let railway_stat = stats
            .iter()
            .find(|s| s.connection_type == ConnectionType::Railway)
            .unwrap();
        assert!(!railway_stat.active);
        assert_eq!(railway_stat.count, 0);
        assert_eq!(railway_stat.avg_utilization, 0.0);
    }

    #[test]
    fn test_stats_all_types_active() {
        let mut outside = OutsideConnections::default();
        outside
            .connections
            .push(make_conn(ConnectionType::Highway, 185, 0, 5000, 0.5));
        outside
            .connections
            .push(make_conn(ConnectionType::Railway, 1, 128, 2000, 0.3));
        outside
            .connections
            .push(make_conn(ConnectionType::SeaPort, 0, 0, 3000, 0.2));
        outside
            .connections
            .push(make_conn(ConnectionType::Airport, 100, 100, 5000, 0.8));

        let stats = outside.stats();
        for stat in &stats {
            assert!(stat.active, "{:?} should be active", stat.connection_type);
            assert_eq!(stat.count, 1);
            assert!(!stat.effect_description.is_empty());
        }
    }

    #[test]
    fn test_stats_no_connections_all_inactive() {
        let outside = OutsideConnections::default();
        let stats = outside.stats();
        assert_eq!(stats.len(), 4);
        for stat in &stats {
            assert!(
                !stat.active,
                "{:?} should be inactive",
                stat.connection_type
            );
            assert_eq!(stat.count, 0);
            assert_eq!(stat.avg_utilization, 0.0);
        }
    }
}
