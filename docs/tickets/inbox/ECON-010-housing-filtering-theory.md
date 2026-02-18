# ECON-010: Housing Filtering and Income Bracket Assignment
**Priority:** T3
**Complexity:** L
**Dependencies:** BLDG-007, ECON-012
**Source:** economic_simulation.md, section 3.1

## Description
Implement housing filtering theory where buildings transition from higher-income to lower-income occupants as they age. New luxury construction -> upper income moves in -> their previous (older) homes become available to middle income -> chain continues.

- HousingUnit struct: quality (0-1.0), target_income_bracket, rent
- Quality = 1.0 at construction, depreciates 1.5%/year without renovation
- Bracket assignment: Luxury (q>=0.85), UpperMiddle (>=0.65), Middle (>=0.45), LowerMiddle (>=0.25), Low (<0.25)
- Rent tracks quality + land value: base_rent * quality * land_value_multiplier
- Renovation resets quality (gentrification = reverse filtering)
- Creates natural affordable housing pipeline without intervention

## Definition of Done
- [ ] Housing quality degrades over time
- [ ] Target income bracket assigned based on quality
- [ ] Rent scales with quality and land value
- [ ] Older buildings naturally house lower-income citizens
- [ ] Renovation mechanic resets quality

## Test Plan
- Unit: New building quality = 1.0, maps to Luxury bracket
- Unit: 40-year-old building quality ~0.4, maps to LowerMiddle
- Integration: Over time, old neighborhoods become lower-income areas

## Pitfalls
- Filtering is slow (decades in real time) -- game time compression needs to make it visible
- Without enough new construction, filtering chain stalls (no displacement)
- Land value can override filtering (high-value land + old building = gentrification pressure)

## Relevant Code
- `crates/simulation/src/buildings.rs:Building` -- add quality/rent fields or separate component
- `crates/simulation/src/citizen.rs` -- income bracket matching
- `crates/simulation/src/citizen_spawner.rs` -- match income to building bracket
