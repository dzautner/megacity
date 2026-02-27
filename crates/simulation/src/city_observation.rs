//! Compact, typed, serializable snapshot of the city state.
//!
//! `CityObservation` is the "eyes" of the LLM agent — it captures the full
//! city state into a single struct each turn so the agent can reason about
//! what to do next.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Top-level observation
// ---------------------------------------------------------------------------

/// A point-in-time snapshot of the entire city state, designed to be sent to
/// an LLM agent each turn.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CityObservation {
    // -- Time ---------------------------------------------------------------
    pub tick: u64,
    pub day: u32,
    pub hour: f32,
    pub speed: f32,
    pub paused: bool,

    // -- Economy ------------------------------------------------------------
    pub treasury: f64,
    pub monthly_income: f64,
    pub monthly_expenses: f64,
    pub net_income: f64,

    // -- Population ---------------------------------------------------------
    pub population: PopulationSnapshot,

    // -- Zone demand --------------------------------------------------------
    pub zone_demand: ZoneDemandSnapshot,

    // -- Infrastructure coverage (0.0–1.0) ----------------------------------
    pub power_coverage: f32,
    pub water_coverage: f32,

    // -- Service coverage (0.0–1.0) -----------------------------------------
    pub services: ServiceCoverageSnapshot,

    // -- Happiness ----------------------------------------------------------
    pub happiness: HappinessSnapshot,

    // -- Warnings -----------------------------------------------------------
    pub warnings: Vec<CityWarning>,

    // -- Recent action results (from ActionResultLog when available) ---------
    pub recent_action_results: Vec<ActionResultEntry>,
}

// ---------------------------------------------------------------------------
// Sub-snapshots
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PopulationSnapshot {
    pub total: u32,
    pub employed: u32,
    pub unemployed: u32,
    pub homeless: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ZoneDemandSnapshot {
    pub residential: f32,
    pub commercial: f32,
    pub industrial: f32,
    pub office: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServiceCoverageSnapshot {
    pub fire: f32,
    pub police: f32,
    pub health: f32,
    pub education: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HappinessSnapshot {
    pub overall: f32,
    pub components: Vec<(String, f32)>,
}

// ---------------------------------------------------------------------------
// Warnings
// ---------------------------------------------------------------------------

/// High-level warning signals for the LLM agent.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CityWarning {
    PowerShortage,
    WaterShortage,
    HighCrime,
    HighPollution,
    HighUnemployment,
    NegativeBudget,
    HighHomelessness,
    TrafficCongestion,
}

// ---------------------------------------------------------------------------
// Action result entry
// ---------------------------------------------------------------------------

/// Compact summary of a recently executed game action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResultEntry {
    pub action_summary: String,
    pub success: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observation_default_is_empty() {
        let obs = CityObservation::default();
        assert_eq!(obs.tick, 0);
        assert!(obs.warnings.is_empty());
        assert!(obs.recent_action_results.is_empty());
    }

    #[test]
    fn observation_serializes_to_json() {
        let obs = CityObservation {
            tick: 42,
            day: 3,
            hour: 14.5,
            speed: 2.0,
            paused: false,
            treasury: 10_000.0,
            monthly_income: 500.0,
            monthly_expenses: 300.0,
            net_income: 200.0,
            population: PopulationSnapshot {
                total: 100,
                employed: 80,
                unemployed: 20,
                homeless: 5,
            },
            zone_demand: ZoneDemandSnapshot {
                residential: 0.6,
                commercial: 0.3,
                industrial: 0.1,
                office: 0.0,
            },
            power_coverage: 0.9,
            water_coverage: 0.85,
            services: ServiceCoverageSnapshot {
                fire: 0.7,
                police: 0.6,
                health: 0.5,
                education: 0.8,
            },
            happiness: HappinessSnapshot {
                overall: 65.0,
                components: vec![("employment".into(), 80.0), ("safety".into(), 50.0)],
            },
            warnings: vec![CityWarning::NegativeBudget],
            recent_action_results: vec![ActionResultEntry {
                action_summary: "Built road".into(),
                success: true,
            }],
        };
        let json = serde_json::to_string(&obs).unwrap();
        assert!(json.contains("\"tick\":42"));
        assert!(json.contains("NegativeBudget"));
    }
}
