use crate::game_params::ZoneDemandParams;

use super::stats::ZoneStats;

// ---------------------------------------------------------------------------
// Vacancy rate helpers
// ---------------------------------------------------------------------------

/// Compute vacancy rate: fraction of capacity that is unoccupied.
/// Returns 0.0 when capacity is 0 (no buildings).
pub(crate) fn vacancy_rate(capacity: u32, occupants: u32) -> f32 {
    if capacity == 0 {
        return 0.0;
    }
    let occupied = occupants.min(capacity) as f32;
    (capacity as f32 - occupied) / capacity as f32
}

/// Compute a raw demand signal from a vacancy rate relative to a natural vacancy range.
/// Returns a value roughly in [-1.0, 1.0]:
///   - Large positive when vacancy is well below the natural range (market is tight).
///   - Large negative when vacancy is well above the natural range (market is oversupplied).
///   - Near zero when vacancy is within the natural range.
pub(crate) fn vacancy_demand_signal(vacancy: f32, natural: (f32, f32)) -> f32 {
    let midpoint = (natural.0 + natural.1) * 0.5;
    // Scale factor: how strongly the signal responds to deviation.
    // At the edges of the natural range the signal should be ~0.3,
    // so we want signal_strength * half_width ≈ 0.3.
    let half_width = (natural.1 - natural.0) * 0.5;
    let sensitivity = if half_width > 0.0 {
        0.3 / half_width
    } else {
        10.0
    };
    // Negative because high vacancy means low demand (invert).
    let raw = (midpoint - vacancy) * sensitivity;
    raw.clamp(-1.0, 1.0)
}

// ---------------------------------------------------------------------------
// Market factor helpers
// ---------------------------------------------------------------------------

/// Employment availability: fraction of total job slots that are unfilled.
/// High values mean lots of available jobs, pulling residential demand up.
fn employment_availability(zs: &ZoneStats) -> f32 {
    if zs.total_job_capacity == 0 {
        return 0.0;
    }
    let unfilled = zs.total_job_capacity.saturating_sub(zs.total_job_occupants);
    (unfilled as f32 / zs.total_job_capacity as f32).clamp(0.0, 1.0)
}

/// Population spending power: ratio of population to commercial capacity.
/// High ratio means lots of shoppers relative to commercial space, driving demand.
fn population_spending_pressure(zs: &ZoneStats) -> f32 {
    if zs.commercial_capacity == 0 {
        // No commercial at all: high pressure to build some.
        return if zs.population > 0 { 1.0 } else { 0.0 };
    }
    let ratio = zs.population as f32 / zs.commercial_capacity as f32;
    // Normalize: ratio of 1.0 is balanced, >1 means under-served.
    (ratio - 0.5).clamp(0.0, 1.0)
}

/// Labor supply factor for industrial demand: how many workers exist relative
/// to industrial capacity. High ratio means ample labor, good for industry.
fn labor_supply_factor(zs: &ZoneStats) -> f32 {
    if zs.industrial_capacity == 0 {
        return if zs.population > 0 { 0.8 } else { 0.0 };
    }
    let ratio = zs.population as f32 / (zs.industrial_capacity as f32 * 2.0);
    ratio.clamp(0.0, 1.0)
}

/// Educated workforce fraction for office demand.
/// Approximated by looking at office occupancy rate as a proxy.
fn office_workforce_factor(zs: &ZoneStats) -> f32 {
    if zs.office_capacity == 0 {
        // No offices yet: moderate pull if population exists.
        return if zs.population > 100 { 0.5 } else { 0.0 };
    }
    // How well-staffed are offices? High occupancy = educated workforce available.
    let occ_rate = zs.office_occupants as f32 / zs.office_capacity as f32;
    // Also consider population scale: bigger city, more demand for offices.
    let pop_scale = (zs.population as f32 / 5000.0).min(1.0);
    (occ_rate * 0.5 + pop_scale * 0.5).clamp(0.0, 1.0)
}

// ---------------------------------------------------------------------------
// Core demand computation (pure function, testable without ECS)
// ---------------------------------------------------------------------------

/// Compute raw (un-damped) demand targets given zone stats.
/// Returns (residential, commercial, industrial, office) demands in [0, 1].
/// Convenience wrapper that uses default `ZoneDemandParams` (matching the
/// original hardcoded constants). Used by unit tests and callers without
/// access to the ECS.
pub fn compute_market_demand(zs: &ZoneStats) -> (f32, f32, f32, f32) {
    compute_market_demand_with_params(zs, &ZoneDemandParams::default())
}

/// Compute raw (un-damped) demand targets given zone stats and configurable
/// zone demand parameters from [`GameParams`].
/// Returns (residential, commercial, industrial, office) demands in [0, 1].
pub fn compute_market_demand_with_params(
    zs: &ZoneStats,
    params: &ZoneDemandParams,
) -> (f32, f32, f32, f32) {
    // --- Bootstrap: no buildings at all ---
    if !zs.has_roads {
        return (0.0, 0.0, 0.0, 0.0);
    }

    let total_capacity = zs.residential_capacity
        + zs.commercial_capacity
        + zs.industrial_capacity
        + zs.office_capacity;

    if total_capacity == 0 {
        // Roads exist but no buildings: initial bootstrap demand.
        return (
            params.bootstrap_demand,
            params.bootstrap_demand * 0.4,
            params.bootstrap_demand * 0.6,
            params.bootstrap_demand * 0.2,
        );
    }

    // --- Vacancy rates ---
    let vr = vacancy_rate(zs.residential_capacity, zs.residential_occupants);
    let vc = vacancy_rate(zs.commercial_capacity, zs.commercial_occupants);
    let vi = vacancy_rate(zs.industrial_capacity, zs.industrial_occupants);
    let vo = vacancy_rate(zs.office_capacity, zs.office_occupants);

    // --- Vacancy signals: positive = need more, negative = oversupplied ---
    let r_vacancy_sig = vacancy_demand_signal(vr, params.natural_vacancy_residential);
    let c_vacancy_sig = vacancy_demand_signal(vc, params.natural_vacancy_commercial);
    let i_vacancy_sig = vacancy_demand_signal(vi, params.natural_vacancy_industrial);
    let o_vacancy_sig = vacancy_demand_signal(vo, params.natural_vacancy_office);

    // --- Market factor signals ---
    let emp_avail = employment_availability(zs);
    let spending = population_spending_pressure(zs);
    let labor = labor_supply_factor(zs);
    let office_wf = office_workforce_factor(zs);

    // --- Residential demand = f(employment availability, vacancy) ---
    // Weight: 50% vacancy signal, 35% employment availability, 15% base immigration pressure.
    let r_base_pressure = 0.15; // constant immigration pressure
    let r_raw = r_vacancy_sig * 0.50 + emp_avail * 0.35 + r_base_pressure;

    // --- Commercial demand = f(population spending power, vacancy) ---
    // Weight: 45% vacancy signal, 40% population spending, 15% base.
    let c_raw = c_vacancy_sig * 0.45 + spending * 0.40 + 0.05;

    // --- Industrial demand = f(labor supply, vacancy) ---
    // Weight: 50% vacancy signal, 35% labor supply, 15% base.
    let i_raw = i_vacancy_sig * 0.50 + labor * 0.35 + 0.05;

    // --- Office demand = f(educated workforce, vacancy) ---
    // Weight: 45% vacancy signal, 40% workforce factor, 15% base.
    let o_raw = o_vacancy_sig * 0.45 + office_wf * 0.40 + 0.02;

    (
        r_raw.clamp(0.0, 1.0),
        c_raw.clamp(0.0, 1.0),
        i_raw.clamp(0.0, 1.0),
        o_raw.clamp(0.0, 1.0),
    )
}
