# INFRA-001: Multi-Octave fBm Terrain Generation
**Priority:** T0
**Complexity:** M (1-3 days)
**Dependencies:** none
**Source:** procedural_terrain.md, Section 1.2

## Description
Replace the single-pass OpenSimplex2 noise in `generate_terrain()` with 6-octave fractal Brownian motion (fBm). Use `fastnoise-lite`'s built-in `FractalType::FBm` with `persistence=0.45`, `lacunarity=2.0`, `base_frequency=0.008`, `octaves=6`. Currently `terrain.rs` uses a single noise call at frequency 0.008 with no fractal layering.

## Definition of Done
- [ ] `terrain.rs` `generate_terrain()` uses `FractalType::FBm` with 6 octaves
- [ ] Persistence, lacunarity, and frequency are configurable constants in `config.rs`
- [ ] Generated elevation distribution produces mostly buildable mid-range terrain
- [ ] Tests pass

## Test Plan
- Unit: Generate 256x256 heightmap, verify elevation range [0,1], verify standard deviation < 0.3
- Integration: Load game with new terrain, verify buildings can be placed on flat areas

## Pitfalls
- 6+ octaves at frequency 0.008 with lacunarity 2.0 means octave 6 has frequency 0.512, which at 256 cells gives wavelength ~2 cells (near Nyquist). Cap at 6 octaves.
- Existing Tel Aviv hardcoded terrain in `lib.rs` init may override `generate_terrain()` -- ensure both paths work.

## Relevant Code
- `crates/simulation/src/terrain.rs` -- main generation function
- `crates/simulation/src/config.rs` -- constants (GRID_WIDTH, GRID_HEIGHT, etc.)
- `crates/simulation/src/grid.rs` -- `Cell.elevation`, `WorldGrid`
