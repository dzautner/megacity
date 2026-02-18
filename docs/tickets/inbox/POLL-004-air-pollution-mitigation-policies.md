# POLL-004: Air Pollution Mitigation Policies and Technology Upgrades

## Priority: T2 (Depth)

## Description
Implement pollution mitigation measures as player-activated policies and building upgrades. The research doc defines 7 mitigation options ranging from scrubbers (-50% source Q) to emissions caps (-20% industrial Q at -10% profit). Currently `policies.rs` has a `pollution_multiplier()` but no granular per-measure controls.

## Current State
- `Policies` resource has a generic `pollution_multiplier()` method.
- No per-building upgrade system for scrubbers or converters.
- No EV mandate or phased emission reduction.
- Green belt / tree planting exists (trees reduce pollution) but is not a formal policy toggle.

## Definition of Done
- [ ] Policy: "Scrubbers on Power Plants" -- reduces power plant source Q by 50%, costs 1.5x O&M.
- [ ] Policy: "Catalytic Converters" -- reduces road source Q by 30%.
- [ ] Policy: "Electric Vehicle Mandate" -- reduces road source Q by 60%, phased over 5 game-years.
- [ ] Policy: "Emissions Cap" -- reduces all industrial Q by 20%, reduces industrial profit by 10%.
- [ ] Policy: "Air Quality Monitoring" -- unlocks per-cell AQI overlay with labels (cost $5,000).
- [ ] Each policy has a toggle in the policy panel with cost/benefit description.
- [ ] Policies affect the emission rate calculation in the pollution system.

## Test Plan
- [ ] Unit test: scrubber policy halves power plant emissions.
- [ ] Unit test: EV mandate reduces road pollution progressively over time.
- [ ] Integration test: enabling emissions cap reduces average city pollution.
- [ ] UI test: all policies appear in the policies panel with correct descriptions.

## Pitfalls
- Phased EV mandate requires tracking years since policy activation (needs a timer or activation date).
- "Air Quality Monitoring" as an unlock gate for overlay detail is a UX decision that may frustrate players.
- Must not double-count with existing `pollution_multiplier()`.

## Code References
- `crates/simulation/src/policies.rs`: `Policies` resource
- `crates/simulation/src/pollution.rs`: `update_pollution`
- `crates/ui/src/info_panel.rs`: policy panel UI
- Research: `environment_climate.md` section 1.1.6
