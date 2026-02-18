# DISASTER-007: Wildfire Spread Using Rothermel Model (Fuel, Wind, Slope, Moisture)

## Priority: T2 (Depth)

## Description
Enhance the existing forest fire system with the Rothermel-inspired spread model: `spread_rate = R_0 * phi_w * phi_s * phi_m`. Currently fire spread uses a simple hash-based probability. The research doc specifies fuel type spread rates, wind factor (cubic), slope factor, and moisture factor.

## Current State
- `forest_fire.rs` has wind-influenced spread with alignment dot product.
- Fuel types are binary (tree/no-tree); no grass/brush/dense-forest distinction.
- No fuel moisture tracking.
- No slope factor in spread probability.
- Wind influence exists but is linear, not directional cubic.

## Definition of Done
- [ ] Fuel types: grass_short(R=0.50), grass_tall(R=0.70), brush(R=0.35), light_forest(R=0.20), dense_forest(R=0.10), urban_wood(R=0.15), urban_concrete(R=0.02), park(R=0.05), bare/water/road(R=0.00).
- [ ] Wind factor: `phi_w = 1.0 + wind_speed * 3.0 * max(0, cos(angle_to_wind))`.
- [ ] Slope factor: uphill `phi_s = 1.0 + slope * 5.0`, downhill `phi_s = 1.0 / (1.0 + abs(slope) * 3.0)`.
- [ ] Moisture factor: derived from days since rain, humidity, season.
- [ ] `spread_probability = min(R_total, 0.95)` per neighbor per tick.
- [ ] Fire states: UNBURNED, BURNING, BURNED_OUT, FIREBREAK.

## Test Plan
- [ ] Unit test: grass spreads faster than dense forest.
- [ ] Unit test: downwind spread is 3x faster at strong wind.
- [ ] Unit test: uphill spread is faster than downhill.
- [ ] Unit test: wet fuel (moisture > 0.30) does not burn.
- [ ] Integration test: fire spreads rapidly downwind through grass, slowly through forest.

## Pitfalls
- Fuel type assignment requires mapping from grid cell data (zone, building, tree).
- Moisture tracking requires days-since-rain counter (not currently tracked).
- May need to rework `ForestFireGrid` to support the fire state enum.

## Code References
- `crates/simulation/src/forest_fire.rs`: current fire spread system
- `crates/simulation/src/wind.rs`: `WindState`
- `crates/simulation/src/grid.rs`: `Cell.elevation` for slope calculation
- Research: `environment_climate.md` sections 5.3.1-5.3.6
