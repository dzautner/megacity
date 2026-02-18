# INFRA-004: Particle-Based Hydraulic Erosion Simulation
**Priority:** T2
**Complexity:** L (3-5 days)
**Dependencies:** INFRA-001
**Source:** procedural_terrain.md, Section 2.1-2.4

## Description
Implement particle-based hydraulic erosion on the heightmap to create realistic river valleys, drainage basins, and natural-looking terrain. Drop N particles at random positions, let them flow downhill accumulating sediment, deposit sediment when slowing. Use parameters: `erosion_radius=3`, `inertia=0.05`, `sediment_capacity_factor=4.0`, `min_sediment_capacity=0.01`, `erode_speed=0.3`, `deposit_speed=0.3`, `evaporate_speed=0.01`, `gravity=4.0`. Run 50,000-200,000 droplets.

## Definition of Done
- [ ] `hydraulic_erosion()` function processes heightmap in-place
- [ ] Produces visible river valley features in flat terrain
- [ ] Droplet count and parameters are configurable
- [ ] Runs in < 2 seconds for 256x256 grid with 100K droplets
- [ ] Tests pass

## Test Plan
- Unit: Erode a synthetic cone heightmap, verify valley formation at base
- Unit: Verify total mass is approximately conserved (eroded ~= deposited)
- Integration: Visual inspection of generated terrain for natural drainage patterns

## Pitfalls
- Erosion at grid edges needs clamping to prevent out-of-bounds
- Too many droplets over-erodes terrain into flat plain
- Bilinear interpolation needed for sub-cell gradient calculation
- Performance: each droplet needs ~100 steps; 100K droplets = 10M iterations

## Relevant Code
- `crates/simulation/src/terrain.rs` -- new erosion pass after noise generation
- `crates/simulation/src/grid.rs` -- `WorldGrid` elevation data
