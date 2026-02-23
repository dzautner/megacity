// ---------------------------------------------------------------------------
// V2 save structs: Policies, Weather, UnlockState, ExtendedBudget, LoanBook,
// LifecycleTimer, LifeSimTimer, VirtualPopulation
// ---------------------------------------------------------------------------

use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Encode, Decode, Default)]
pub struct SavePolicies {
    /// Active policy discriminants
    pub active: Vec<u8>,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveWeather {
    pub season: u8,
    pub temperature: f32,
    pub current_event: u8,
    pub event_days_remaining: u32,
    pub last_update_day: u32,
    pub disasters_enabled: bool,
    #[serde(default = "default_save_humidity")]
    pub humidity: f32,
    #[serde(default)]
    pub cloud_cover: f32,
    #[serde(default)]
    pub precipitation_intensity: f32,
    #[serde(default)]
    pub last_update_hour: u32,
    /// Climate zone (0=Temperate default for backward compat).
    #[serde(default)]
    pub climate_zone: u8,
}

fn default_save_humidity() -> f32 {
    0.5
}

impl Default for SaveWeather {
    fn default() -> Self {
        Self {
            season: 0, // Spring
            temperature: 15.0,
            current_event: 0, // Sunny
            event_days_remaining: 0,
            last_update_day: 0,
            disasters_enabled: true,
            humidity: 0.5,
            cloud_cover: 0.0,
            precipitation_intensity: 0.0,
            last_update_hour: 0,
            climate_zone: 0, // Temperate
        }
    }
}

#[derive(Serialize, Deserialize, Encode, Decode, Default)]
pub struct SaveUnlockState {
    pub development_points: u32,
    pub spent_points: u32,
    pub unlocked_nodes: Vec<u8>,
    pub last_milestone_pop: u32,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveExtendedBudget {
    // Zone tax rates
    pub residential_tax: f32,
    pub commercial_tax: f32,
    pub industrial_tax: f32,
    pub office_tax: f32,
    // Service budgets
    pub fire_budget: f32,
    pub police_budget: f32,
    pub healthcare_budget: f32,
    pub education_budget: f32,
    pub sanitation_budget: f32,
    pub transport_budget: f32,
}

impl Default for SaveExtendedBudget {
    fn default() -> Self {
        Self {
            residential_tax: 0.10,
            commercial_tax: 0.10,
            industrial_tax: 0.10,
            office_tax: 0.10,
            fire_budget: 1.0,
            police_budget: 1.0,
            healthcare_budget: 1.0,
            education_budget: 1.0,
            sanitation_budget: 1.0,
            transport_budget: 1.0,
        }
    }
}

#[derive(Serialize, Deserialize, Encode, Decode, Default)]
pub struct SaveLifecycleTimer {
    pub last_aging_day: u32,
    pub last_emigration_tick: u32,
}

#[derive(Serialize, Deserialize, Encode, Decode, Default)]
pub struct SaveLifeSimTimer {
    pub needs_tick: u32,
    pub life_event_tick: u32,
    pub salary_tick: u32,
    pub education_tick: u32,
    pub job_seek_tick: u32,
    pub personality_tick: u32,
    pub health_tick: u32,
}

#[derive(Serialize, Deserialize, Encode, Decode, Default)]
pub struct SaveLoanBook {
    pub loans: Vec<SaveLoan>,
    pub max_loans: u32,
    pub credit_rating: f64,
    pub last_payment_day: u32,
    pub consecutive_solvent_days: u32,
}

#[derive(Serialize, Deserialize, Encode, Decode)]
pub struct SaveLoan {
    pub name: String,
    pub amount: f64,
    pub interest_rate: f64,
    pub monthly_payment: f64,
    pub remaining_balance: f64,
    pub term_months: u32,
    pub months_paid: u32,
}

#[derive(Serialize, Deserialize, Encode, Decode, Default)]
pub struct SaveDistrictStats {
    pub population: u32,
    pub employed: u32,
    pub avg_happiness: f32,
    pub avg_age: f32,
    pub age_brackets: [u32; 5],
    pub commuters_out: u32,
    pub tax_contribution: f32,
    pub service_demand: f32,
}

#[derive(Serialize, Deserialize, Encode, Decode, Default)]
pub struct SaveVirtualPopulation {
    pub total_virtual: u32,
    pub virtual_employed: u32,
    pub district_stats: Vec<SaveDistrictStats>,
    pub max_real_citizens: u32,
}

#[derive(Serialize, Deserialize, Encode, Decode, Default)]
pub struct SaveStormwaterGrid {
    pub runoff: Vec<f32>,
    pub total_runoff: f32,
    pub total_infiltration: f32,
    pub width: usize,
    pub height: usize,
}

#[derive(Serialize, Deserialize, Encode, Decode, Default)]
pub struct SaveDegreeDays {
    pub daily_hdd: f32,
    pub daily_cdd: f32,
    pub monthly_hdd: [f32; 12],
    pub monthly_cdd: [f32; 12],
    pub annual_hdd: f32,
    pub annual_cdd: f32,
    pub last_update_day: u32,
}

#[derive(Serialize, Deserialize, Encode, Decode, Default)]
pub struct SaveConstructionModifiers {
    pub speed_factor: f32,
    pub cost_factor: f32,
}
