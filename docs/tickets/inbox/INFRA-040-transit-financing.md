# INFRA-040: Transit Financing Model (Revenue, Costs, Subsidies)
**Priority:** T2
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-037
**Source:** transportation_simulation.md, Section 4.7

## Description
Implement transit financial model. Revenue = ridership * average_fare. Operating cost per vehicle-hour by bus type: Standard $120, Articulated $150, BiArticulated $180, Electric $100. Fare recovery ratio = fare_revenue / operating_cost. Typical: US 30-40%, Europe 40-60%. Gap covered by city budget (subsidy). Display per-route and system-wide financial performance. Gameplay tension: better transit loses more money but increases land values and reduces congestion.

## Definition of Done
- [ ] Per-route revenue and operating cost tracking
- [ ] Fare recovery ratio computed and displayed
- [ ] Transit subsidy deducted from city budget
- [ ] Per-route financial performance in transit info panel
- [ ] System-wide transit budget in budget panel
- [ ] Tests pass

## Test Plan
- Unit: 1000 riders/day at $2 fare = $2000 revenue
- Unit: 5 buses at $120/vehicle-hr * 16 hrs = $9600 cost
- Unit: Fare recovery = $2000/$9600 = 20.8%

## Pitfalls
- Transit often requires subsidy; players may abandon routes that lose money
- Need advisor guidance on acceptable subsidy levels
- Fare elasticity: higher fares reduce ridership

## Relevant Code
- `crates/simulation/src/economy.rs` -- transit budget line items
- `crates/simulation/src/budget.rs` -- expense categories
