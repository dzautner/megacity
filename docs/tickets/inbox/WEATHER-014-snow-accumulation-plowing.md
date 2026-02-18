# WEATHER-014: Snow Accumulation and Road Plowing

## Priority: T3 (Differentiation)

## Description
Implement snow accumulation during winter precipitation events. Snow covers roads (reducing traffic speed), buildings (increasing heating demand), and parks (changing appearance). Snow plowing service clears roads at a cost.

## Current State
- Winter reduces travel speed by 15% (`travel_speed_multiplier` = 0.85).
- No snow accumulation grid.
- No plowing mechanics.
- Season-based grass color changes (white/grey in winter).

## Definition of Done
- [ ] `SnowGrid` resource tracking snow depth per cell (inches).
- [ ] Snow accumulates during winter rain/storm events when temperature < 32F.
- [ ] Snow melts when temperature > 32F at a rate proportional to temperature excess.
- [ ] Road snow: reduces traffic speed by depth * 5%, up to 80% reduction at 12+ inches.
- [ ] Snow plowing service: $500/cell/event, prioritizes highways > arterials > local roads.
- [ ] Heating demand increase: +10% per 6 inches of snow.
- [ ] Snowmelt contributes to spring flooding risk (DISASTER-005).
- [ ] Visual rendering: white terrain overlay based on snow depth.

## Test Plan
- [ ] Unit test: snow accumulates when T < 32F and precipitation is active.
- [ ] Unit test: snow melts at rate proportional to temperature above 32F.
- [ ] Integration test: heavy snowfall slows traffic dramatically.
- [ ] Integration test: plowing restores road speed.

## Pitfalls
- Snow grid needs save/load serialization.
- Snowmelt flooding in spring requires tracking accumulated snow over winter.
- Plowing priority system needs road type classification.

## Code References
- `crates/simulation/src/weather.rs`: temperature, precipitation
- Research: `environment_climate.md` section 4.4.2
