# POWER-014: Waste-to-Energy Power Plant

## Priority: T2 (Depth)

## Description
Implement waste-to-energy (WTE) plant that incinerates municipal waste to generate electricity. WTE reduces landfill volume by 90% but produces air emissions requiring scrubbers. Output depends on waste input tonnage.

## Current State
- No WTE facility exists.
- No waste-to-energy conversion.

## Definition of Done
- [ ] WTE plant: 200-1000 tons/day waste input, generates 0.5-1.0 MWh/ton electricity.
- [ ] Energy output formula: `waste_tons * BTU_per_lb * 2000 * boiler_eff * generator_eff / 3412`.
- [ ] Default: 500 tons/day = ~15.4 MW average output.
- [ ] Construction cost: $50M, build time: 10 game-days.
- [ ] Operating cost: $40-60/ton, revenue from tipping fees $50-80/ton.
- [ ] Air pollution: Q=45.0 (with scrubbers: Q=20.0).
- [ ] Ash residue: 10% of input mass, requires secure landfill.
- [ ] 4x4 building footprint.
- [ ] Competes with recycling (diverts waste from WTE feedstock).

## Test Plan
- [ ] Unit test: energy output matches formula at default parameters.
- [ ] Unit test: scrubbers reduce air pollution by 55%.
- [ ] Integration test: WTE reduces waste going to landfill.
- [ ] Integration test: WTE contributes to power grid.

## Pitfalls
- WTE competes with recycling for waste stream; needs balance.
- Ash disposal requirement adds a secondary waste stream.
- Public opposition (NIMBY) could be modeled as happiness penalty for nearby residents.

## Code References
- Research: `environment_climate.md` sections 6.5.1-6.5.3
