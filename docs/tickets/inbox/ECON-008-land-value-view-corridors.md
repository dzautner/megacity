# ECON-008: Land Value View Corridor Bonus
**Priority:** T3
**Complexity:** M
**Dependencies:** ECON-006
**Source:** economic_simulation.md, section 2.3

## Description
Add view corridor bonuses to land value for cells with line-of-sight to water, parks, or landmarks. Real estate near water or with park views commands premium prices.

- Water view: raycast from cell toward water bodies, +15-25% LV if within 10 cells with unobstructed view
- Park view: proximity to park cells with LOS check, +5-10% LV
- Landmark view: proximity to landmark buildings, +10-15%
- View blocked by buildings taller than viewer (multi-cell buildings in path)
- Compute using simple grid raycast (no 3D; use building level as height proxy)

## Definition of Done
- [ ] View corridor calculation for water, parks, landmarks
- [ ] LOS raycast considers building heights
- [ ] View bonus applied to land value
- [ ] View bonus visible in land value overlay

## Test Plan
- Unit: Cell with clear water view gets +20% LV bonus
- Unit: Cell with building blocking water view gets no bonus
- Integration: Build tall building, verify it blocks water view for cells behind it

## Pitfalls
- Raycast from every cell in every direction is expensive -- limit to 4-8 directions
- Building "height" currently just a level number; need to define LOS height per level
- Water cells must be identified (CellType::Water exists)

## Relevant Code
- `crates/simulation/src/land_value.rs` -- add view corridor computation
- `crates/simulation/src/grid.rs:CellType::Water` -- water cell identification
- `crates/simulation/src/buildings.rs:Building` -- level as height proxy
