use super::types::{ConnectionType, OutsideConnections};

// =============================================================================
// Connection effects
// =============================================================================

/// Summary of effects applied by active outside connections this tick.
#[derive(Debug, Clone, Default)]
pub struct ConnectionEffects {
    /// Multiplier for import costs (1.0 = normal, <1.0 = cheaper).
    pub import_cost_multiplier: f32,
    /// Additive tourism bonus.
    pub tourism_bonus: f32,
    /// Additive attractiveness bonus (applied to overall score).
    pub attractiveness_bonus: f32,
    /// Multiplier for immigration rate (1.0 = normal).
    pub immigration_multiplier: f32,
    /// Multiplier for industrial production (1.0 = normal).
    pub industrial_production_bonus: f32,
    /// Multiplier for export prices (1.0 = normal, >1.0 = higher value).
    pub export_price_multiplier: f32,
}

impl ConnectionEffects {
    pub fn compute(connections: &OutsideConnections) -> Self {
        let mut effects = ConnectionEffects {
            import_cost_multiplier: 1.0,
            tourism_bonus: 0.0,
            attractiveness_bonus: 0.0,
            immigration_multiplier: 1.0,
            industrial_production_bonus: 1.0,
            export_price_multiplier: 1.0,
        };

        // Highway: enables trade import/export, boosts immigration by 20%, adds traffic
        if connections.has_connection(ConnectionType::Highway) {
            effects.immigration_multiplier += 0.20;
            effects.attractiveness_bonus += 5.0;
        }

        // Railway: enables bulk goods transport (cheaper imports), boosts tourism +10
        if connections.has_connection(ConnectionType::Railway) {
            effects.import_cost_multiplier *= 0.85; // 15% cheaper imports
            effects.tourism_bonus += 10.0;
            effects.attractiveness_bonus += 3.0;
        }

        // SeaPort: enables heavy cargo (oil, ore imports at 50% cost), boosts industrial
        if connections.has_connection(ConnectionType::SeaPort) {
            effects.import_cost_multiplier *= 0.50; // 50% cheaper for heavy goods
            effects.industrial_production_bonus += 0.15;
            effects.attractiveness_bonus += 4.0;
        }

        // Airport: massive tourism boost (+30), high-value exports, boosts office/tech
        if connections.has_connection(ConnectionType::Airport) {
            effects.tourism_bonus += 30.0;
            effects.export_price_multiplier += 0.20; // 20% higher export prices
            effects.attractiveness_bonus += 8.0;
        }

        effects
    }
}
