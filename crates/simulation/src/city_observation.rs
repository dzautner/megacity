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

    /// Projected monthly income based on current buildings, tax rates, and
    /// tourism — updated every slow-tick, never stale like `monthly_income`
    /// which only refreshes on the 30-day tax collection cycle.
    #[serde(default)]
    pub estimated_monthly_income: f64,
    /// Projected monthly expenses (roads + services + policies + fuel).
    #[serde(default)]
    pub estimated_monthly_expenses: f64,

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

    // -- Attractiveness (immigration driver, 0-100) -------------------------
    #[serde(default)]
    pub attractiveness_score: f32,
    #[serde(default)]
    pub attractiveness: AttractivenessSnapshot,

    // -- Building counts ----------------------------------------------------
    #[serde(default)]
    pub building_count: u32,

    // -- Warnings -----------------------------------------------------------
    pub warnings: Vec<CityWarning>,

    // -- Recent action results (from ActionResultLog when available) ---------
    pub recent_action_results: Vec<ActionResultEntry>,

    // -- Maps (always included) ---------------------------------------------
    #[serde(default)]
    pub overview_map: String,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AttractivenessSnapshot {
    pub employment: f32,
    pub happiness: f32,
    pub services: f32,
    pub housing: f32,
    pub tax: f32,
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
    TradeDeficit,
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
        assert!(obs.overview_map.is_empty());
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
            estimated_monthly_income: 500.0,
            estimated_monthly_expenses: 300.0,
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
            attractiveness_score: 65.0,
            attractiveness: AttractivenessSnapshot {
                employment: 0.8,
                happiness: 0.65,
                services: 0.5,
                housing: 0.6,
                tax: 0.55,
            },
            building_count: 42,
            warnings: vec![CityWarning::NegativeBudget],
            recent_action_results: vec![ActionResultEntry {
                action_summary: "Built road".into(),
                success: true,
            }],
            overview_map: String::new(),
        };
        let json = serde_json::to_string(&obs).unwrap();
        assert!(json.contains("\"tick\":42"));
        assert!(json.contains("NegativeBudget"));
    }

    #[test]
    fn observation_deserializes_without_overview_map() {
        // Simulate an old observation JSON without the overview_map field
        let json = r#"{"tick":10,"day":1,"hour":6.0,"speed":1.0,"paused":false,"treasury":0.0,"monthly_income":0.0,"monthly_expenses":0.0,"net_income":0.0,"population":{"total":0,"employed":0,"unemployed":0,"homeless":0},"zone_demand":{"residential":0.0,"commercial":0.0,"industrial":0.0,"office":0.0},"power_coverage":0.0,"water_coverage":0.0,"services":{"fire":0.0,"police":0.0,"health":0.0,"education":0.0},"happiness":{"overall":0.0,"components":[]},"warnings":[],"recent_action_results":[]}"#;
        let obs: CityObservation = serde_json::from_str(json).unwrap();
        assert_eq!(obs.tick, 10);
        assert!(obs.overview_map.is_empty());
    }
}
