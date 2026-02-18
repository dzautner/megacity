# ECON-022: Commercial Specializations (Leisure/Tourism/Organic/IT)
**Priority:** T2
**Complexity:** M
**Dependencies:** ZONE-015
**Source:** cities_skylines_analysis.md, section 14.1, 14.5; master_architecture.md, section 2

## Description
Implement commercial district specializations modeled after CS1's After Dark and Green Cities DLCs. Each specialization changes building behavior and revenue patterns.

- Leisure: bars, clubs, restaurants. Higher nighttime revenue (+30% at night), higher noise
- Tourism: hotels, souvenir shops. Revenue from tourists, requires tourist attractions nearby
- Organic/Local: health food, farmers markets. Fewer delivery trucks, higher land value
- IT Cluster (office): tech companies, high tax revenue, requires university-educated workers
- Self-Sufficient (residential): solar panels, green roofs. -50% power, -30% water, +10% happiness

## Definition of Done
- [ ] 4+ commercial/office specializations available per district
- [ ] Each specialization changes revenue, costs, and requirements
- [ ] Distinct building visuals per specialization
- [ ] Specialization effects properly integrated

## Test Plan
- Integration: Specialize district as Leisure, verify nighttime revenue boost
- Integration: Specialize as IT Cluster, verify high-education worker requirement

## Pitfalls
- specialization.rs already handles industrial specializations -- extend for commercial
- Nighttime revenue requires time-of-day integration (time_of_day.rs exists)
- Tourism specialization requires tourism system (tourism.rs exists)

## Relevant Code
- `crates/simulation/src/specialization.rs` -- add commercial specializations
- `crates/simulation/src/districts.rs` -- specialization per district
- `crates/simulation/src/economy.rs` -- specialization revenue modifiers
