# WEATHER-013: Heat Wave Mitigation Measures

## Priority: T3 (Differentiation)

## Description
Implement heat wave mitigation options: cooling centers (public buildings open for shelter), green canopy (trees reduce temperature), light-colored roofs, misting stations, and emergency water distribution. Each reduces mortality and temperature effects.

## Current State
- No heat wave mitigation beyond the heating/cooling system.
- Trees reduce pollution but not temperature directly.

## Definition of Done
- [ ] Cooling centers: -50% heat mortality, public buildings open as shelters.
- [ ] Green canopy: -5F local temperature per 20% tree coverage in radius.
- [ ] Light-colored roofs: -3F roof temperature (building upgrade).
- [ ] Misting stations: -10F perceived temperature in public spaces (placeable).
- [ ] Emergency water distribution: prevents dehydration deaths (policy toggle).
- [ ] Each mitigation has a cost and activation condition (only during heat waves).

## Test Plan
- [ ] Unit test: cooling center halves mortality rate.
- [ ] Unit test: 40% tree coverage reduces temperature by 10F.
- [ ] Integration test: city with cooling centers has fewer heat-related deaths.

## Pitfalls
- Depends on WEATHER-006 (heat wave effects) being implemented first.
- Cooling centers require designating existing public buildings, not building new ones.
- Misting stations as a temporary/seasonal placeable is a new concept.

## Code References
- Research: `environment_climate.md` section 4.4.1 (Mitigation)
