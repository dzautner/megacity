# INFRA-043: Multinomial Logit Mode Choice Model
**Priority:** T2
**Complexity:** L (3-5 days)
**Dependencies:** INFRA-037, INFRA-042
**Source:** transportation_simulation.md, Section 4.1 (Step 3); master_architecture.md, M3

## Description
Implement multinomial logit mode choice: `P(mode m) = exp(V_m) / sum(exp(V_k))`. Utility functions: `V_auto = beta_time * time_auto + beta_cost * cost_auto + ASC_auto`, similar for transit (with wait and walk penalties), walk, and bike. Coefficients: beta_time=-0.03 to -0.05/min, beta_wait=-0.06 to -0.10/min (2-3x in-vehicle), beta_cost=-0.005 to -0.02/cent, ASC_auto=+0.5 to +2.0. Each citizen makes probabilistic mode choice per trip.

## Definition of Done
- [ ] `ModeChoice` enum: Auto, Transit, Walk, Bike
- [ ] `mode_choice()` function with utility computation
- [ ] Coefficient values configurable
- [ ] Auto mode includes parking cost component
- [ ] Transit mode includes walk-to-stop and wait time
- [ ] Walk mode capped at 45 minutes max
- [ ] Citizens choose mode per trip based on probabilities
- [ ] Mode split statistics displayed
- [ ] Tests pass

## Test Plan
- Unit: 10km downtown trip with $5 parking: transit ~61%, auto ~24%, walk ~15%
- Unit: Same trip with free parking: auto ~65%
- Unit: Walk probability near zero for trips > 45 min walking

## Pitfalls
- Need transit travel time computation (walk to stop + wait + ride + walk from stop)
- Auto travel time must include parking search time
- Mode choice per-citizen per-trip is expensive; batch by zone pairs
- exp() overflow for very negative utilities; use log-sum-exp trick

## Relevant Code
- `crates/simulation/src/movement.rs` -- citizen movement, mode selection
- `crates/simulation/src/citizen.rs` -- citizen trip state
