# POL-001: Policy Tradeoff System
**Priority:** T2
**Complexity:** M
**Dependencies:** ZONE-015
**Source:** cities_skylines_analysis.md, section 12.2; master_architecture.md, section 5.1

## Description
Expand the policy system from simple toggles to tradeoff-based decisions. Each policy should have clear positive and negative effects. Based on CS1's ~55 policies with DLCs.

Initial policies (15 minimum):
- Free Public Transport: transit ridership +30%, all transit revenue lost
- High-Rise Ban: prevents level 4-5 buildings, preserves neighborhood character
- Heavy Traffic Ban: bans trucks from district, reduces noise, slight industry penalty
- Small Business Enthusiast: caps commercial at level 2, preserves neighborhood
- Smoke Detector Distribution: -50% fire hazard, costs $0.5/citizen/month
- Encourage Biking: +15% cycling, -10% car trips, requires bike infrastructure
- Combustion Engine Ban: bans private cars in district, forces transit/walking
- Recycling: reduces garbage 20%, costs +10% garbage budget
- Parks & Rec: +10% park land value boost, +10% parks budget
- Pet Ban: -10% garbage, -5% happiness
- Old Town / Historic: prevents building changes, preserves aesthetic
- Industrial Space Planning: +50% industrial output, slight pollution increase
- Rent Control: prevents rent increases above inflation, reduces developer construction
- Minimum Wage: sets wage floor, increases business costs, reduces poverty
- Tax Incentive Zone: -50% property tax, +25% construction rate for 5 years

## Definition of Done
- [ ] 15+ policies implemented with clear tradeoffs
- [ ] Policies toggleable city-wide or per-district
- [ ] Effects correctly applied to relevant systems
- [ ] Policy costs tracked in budget
- [ ] Policy UI shows effects before enabling

## Test Plan
- Integration: Enable High-Rise Ban, verify buildings cap at level 3
- Integration: Enable Free Public Transport, verify transit revenue = 0

## Pitfalls
- policies.rs already has partial implementation
- Each policy needs integration with specific game systems (not just flags)
- Too many policies overwhelm new players -- group by category

## Relevant Code
- `crates/simulation/src/policies.rs` -- policy definitions and effects
- `crates/simulation/src/districts.rs:DistrictPolicies` -- per-district toggle
- `crates/ui/src/info_panel.rs` -- policy UI panel
