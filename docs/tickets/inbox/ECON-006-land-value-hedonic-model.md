# ECON-006: Hedonic Land Value Model
**Priority:** T1
**Complexity:** L
**Dependencies:** none
**Source:** economic_simulation.md, section 2.1-2.5; master_architecture.md, section 6.2

## Description
Replace the current simple land value calculation (base 50, add/subtract modifiers) with a hedonic pricing model that considers accessibility, environmental quality, service coverage, and building quality. Add temporal smoothing so values change gradually.

Current problem: Land value resets to base 50 each cycle, no persistence, no momentum.

- Hedonic formula: LV = beta_0 + sum(beta_i * factor_i)
- Factors: distance_to_CBD, service_coverage, park_access, transit_access, water_view, crime_inverse, pollution_inverse, noise_inverse, school_quality, building_quality_avg, employment_access
- Temporal smoothing: new_value = old_value + (computed_value - old_value) * adjustment_speed
- Values rise slowly (2% per tick) and fall faster (5% per tick) -- realistic asymmetry
- Track trend per cell (rising/falling) for overlay visualization
- Neighborhood spillover: high-value cells raise neighbors (diffusion radius 2-3 cells)

## Definition of Done
- [ ] Hedonic formula with 10+ input factors
- [ ] Temporal smoothing (no reset per cycle)
- [ ] Asymmetric adjustment speed (slow rise, fast fall)
- [ ] Trend tracking per cell
- [ ] Neighborhood spillover via diffusion
- [ ] Land value overlay shows trend direction

## Test Plan
- Unit: Cell near park + school + transit has higher LV than cell with none
- Unit: Temporal smoothing prevents > 5% change per tick
- Integration: Place services near low-value area, verify gradual value increase
- Integration: Place industrial near high-value area, verify gradual value decrease

## Pitfalls
- 65K cells * 10 factors = expensive computation -- use slow tick + chunk processing
- Beta weights need careful tuning to produce realistic value gradients
- Must handle edge case where city has no CBD (early game)
- Spillover diffusion can cause runaway positive feedback

## Relevant Code
- `crates/simulation/src/land_value.rs` -- rewrite from scratch
- `crates/simulation/src/happiness.rs:ServiceCoverageGrid` -- service coverage input
- `crates/simulation/src/pollution.rs` -- pollution input
- `crates/simulation/src/crime.rs` -- crime input
