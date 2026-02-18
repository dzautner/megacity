# ECON-025: Land Value Neighborhood Spillover (Diffusion)
**Priority:** T1
**Complexity:** M
**Dependencies:** ECON-006
**Source:** economic_simulation.md, section 2.4; master_architecture.md, section 6.2

## Description
Implement land value diffusion where high-value cells raise neighbor values. Currently there is no spillover. Real estate values are heavily influenced by surrounding property values. A luxury building raises land value for adjacent cells.

- Diffusion: each cell's value influenced by average of neighbors in 2-3 cell radius
- Diffusion weight: 0.1-0.3 per neighbor cell
- Negative spillover: abandoned buildings, pollution sources drag down neighbor values
- Compute on slow tick (not every frame)
- Use iterative Gauss-Seidel smoothing (2-3 iterations per tick)

## Definition of Done
- [ ] Land value diffuses to neighboring cells
- [ ] High-value areas create gradients (not sharp borders)
- [ ] Negative spillover from blight
- [ ] Diffusion smooth and gradual
- [ ] Land value overlay shows smooth gradients

## Test Plan
- Unit: Cell adjacent to high-value cell has higher value than isolated cell
- Integration: Place landmark, verify land value gradient radiates outward

## Pitfalls
- Too much diffusion = all values converge to average (bland)
- Too little diffusion = no visible effect
- Negative spillover can create death spiral (abandoned -> low value -> more abandonment)

## Relevant Code
- `crates/simulation/src/land_value.rs` -- add diffusion pass
