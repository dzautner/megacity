use crate::save_codec::*;
use crate::save_types::*;

use simulation::life_simulation::LifeSimTimer;
use simulation::lifecycle::LifecycleTimer;
use simulation::policies::Policies;
use simulation::recycling::{RecyclingEconomics, RecyclingState};
use simulation::unlocks::UnlockState;
use simulation::virtual_population::VirtualPopulation;

/// Policy / progression state: policies, unlocks, recycling, composting,
/// lifecycle timer, life sim timer, virtual population.
pub struct PolicyStageOutput {
    pub policies: Option<SavePolicies>,
    pub unlock_state: Option<SaveUnlockState>,
    pub recycling_state: Option<SaveRecyclingState>,
    pub composting_state: Option<SaveCompostingState>,
    pub lifecycle_timer: Option<SaveLifecycleTimer>,
    pub life_sim_timer: Option<SaveLifeSimTimer>,
    pub virtual_population: Option<SaveVirtualPopulation>,
}

/// Collect policy / progression state.
pub fn collect_policy_stage(
    policies: Option<&Policies>,
    unlock_state: Option<&UnlockState>,
    recycling_state: Option<(&RecyclingState, &RecyclingEconomics)>,
    composting_state: Option<&simulation::composting::CompostingState>,
    lifecycle_timer: Option<&LifecycleTimer>,
    life_sim_timer: Option<&LifeSimTimer>,
    virtual_population: Option<&VirtualPopulation>,
) -> PolicyStageOutput {
    PolicyStageOutput {
        policies: policies.map(|p| SavePolicies {
            active: p.active.iter().map(|&pol| policy_to_u8(pol)).collect(),
        }),
        unlock_state: unlock_state.map(|u| SaveUnlockState {
            development_points: u.development_points,
            spent_points: u.spent_points,
            unlocked_nodes: u
                .unlocked_nodes
                .iter()
                .map(|&n| unlock_node_to_u8(n))
                .collect(),
            last_milestone_pop: u.last_milestone_pop,
        }),
        recycling_state: recycling_state.map(|(rs, re)| SaveRecyclingState {
            tier: recycling_tier_to_u8(rs.tier),
            daily_tons_diverted: rs.daily_tons_diverted,
            daily_tons_contaminated: rs.daily_tons_contaminated,
            daily_revenue: rs.daily_revenue,
            daily_cost: rs.daily_cost,
            total_revenue: rs.total_revenue,
            total_cost: rs.total_cost,
            participating_households: rs.participating_households,
            price_paper: re.price_paper,
            price_plastic: re.price_plastic,
            price_glass: re.price_glass,
            price_metal: re.price_metal,
            price_organic: re.price_organic,
            market_cycle_position: re.market_cycle_position,
            economics_last_update_day: re.last_update_day,
        }),
        composting_state: composting_state.map(|cs| SaveCompostingState {
            facilities: cs
                .facilities
                .iter()
                .map(|f| SaveCompostFacility {
                    method: compost_method_to_u8(f.method),
                    capacity_tons_per_day: f.capacity_tons_per_day,
                    cost_per_ton: f.cost_per_ton,
                    tons_processed_today: f.tons_processed_today,
                })
                .collect(),
            participation_rate: cs.participation_rate,
            organic_fraction: cs.organic_fraction,
            total_diverted_tons: cs.total_diverted_tons,
            daily_diversion_tons: cs.daily_diversion_tons,
            compost_revenue_per_ton: cs.compost_revenue_per_ton,
            daily_revenue: cs.daily_revenue,
            biogas_mwh_per_ton: cs.biogas_mwh_per_ton,
            daily_biogas_mwh: cs.daily_biogas_mwh,
        }),
        lifecycle_timer: lifecycle_timer.map(|lt| SaveLifecycleTimer {
            last_aging_day: lt.last_aging_day,
            last_emigration_tick: lt.last_emigration_tick,
        }),
        life_sim_timer: life_sim_timer.map(|lst| SaveLifeSimTimer {
            needs_tick: lst.needs_tick,
            life_event_tick: lst.life_event_tick,
            salary_tick: lst.salary_tick,
            education_tick: lst.education_tick,
            job_seek_tick: lst.job_seek_tick,
            personality_tick: lst.personality_tick,
            health_tick: lst.health_tick,
        }),
        virtual_population: virtual_population.map(|vp| SaveVirtualPopulation {
            total_virtual: vp.total_virtual,
            virtual_employed: vp.virtual_employed,
            district_stats: vp
                .district_stats
                .iter()
                .map(|ds| SaveDistrictStats {
                    population: ds.population,
                    employed: ds.employed,
                    avg_happiness: ds.avg_happiness,
                    avg_age: ds.avg_age,
                    age_brackets: ds.age_brackets,
                    commuters_out: ds.commuters_out,
                    tax_contribution: ds.tax_contribution,
                    service_demand: ds.service_demand,
                })
                .collect(),
            max_real_citizens: vp.max_real_citizens,
        }),
    }
}
