# Procedural Terrain Generation

## Document Status and Scope

This document covers the complete procedural terrain generation pipeline for Megacity, from raw noise generation through hydraulic erosion, water body placement, biome classification, resource distribution, and integration with all game systems. The existing codebase uses `fastnoise-lite` with a single-octave OpenSimplex2 pass at frequency 0.008 on a 256x256 grid (CELL_SIZE=16.0, CHUNK_SIZE=8). This document describes how to evolve that into a full multi-layered terrain system.

**Current state of the codebase:**
- `crates/simulation/src/terrain.rs`: Single-pass OpenSimplex2, normalize to [0,1], threshold at 0.35 for water
- `crates/simulation/src/grid.rs`: `Cell { elevation: f32, cell_type: CellType, ... }`, `WorldGrid` with 256x256 cells
- `crates/rendering/src/terrain_render.rs`: Chunk-based flat mesh (y=0.0 for all vertices), vertex-colored
- `crates/simulation/src/natural_resources.rs`: Hash-based resource placement keyed on elevation bands
- Grid constants: GRID_WIDTH=256, GRID_HEIGHT=256, CELL_SIZE=16.0, CHUNK_SIZE=8, WATER_THRESHOLD=0.35

**Physical scale:** 256 cells * 16m = 4,096m = ~4.1km per axis. Total map area: ~16.8 km^2.

---

## Table of Contents

1. [Heightmap Generation](#1-heightmap-generation)
   - 1.1 Noise Algorithm Selection
   - 1.2 Fractal Brownian Motion (fBm)
   - 1.3 Domain Warping
   - 1.4 Height Distribution Shaping
   - 1.5 Noise Scale for 256x256 City Terrain
2. [Hydraulic Erosion Simulation](#2-hydraulic-erosion-simulation)
   - 2.1 Particle-Based Erosion Algorithm
   - 2.2 River Valley and Drainage Basin Formation
   - 2.3 Thermal Erosion
   - 2.4 Iteration Counts and Performance
3. [Water Body Generation](#3-water-body-generation)
   - 3.1 River Placement
   - 3.2 Lake Detection
   - 3.3 Ocean and Coastline
   - 3.4 River Width and Meanders
   - 3.5 Deltas and Estuaries
   - 3.6 Mapping to CellType::Water
4. [Biome and Vegetation](#4-biome-and-vegetation)
   - 4.1 Temperature and Moisture Mapping
   - 4.2 Whittaker Diagram Classification
   - 4.3 Vegetation Density
   - 4.4 Starting Biome Selection
5. [Resource Distribution](#5-resource-distribution)
   - 5.1 Ore and Mineral Deposits
   - 5.2 Fertile Soil Zones
   - 5.3 Oil and Gas Reserves
   - 5.4 Forest Density
   - 5.5 Resource Discovery Mechanics
6. [Starting Map Design](#6-starting-map-design)
   - 6.1 Good Starting Locations
   - 6.2 Playability Guarantees
   - 6.3 Seed-Based Generation
   - 6.4 Landmark Templates
   - 6.5 Real-World Scale Comparisons
7. [Terrain Interaction with Game Systems](#7-terrain-interaction-with-game-systems)
   - 7.1 Slope Effects
   - 7.2 Elevation Effects
   - 7.3 Soil Type Grid
   - 7.4 Flood Plains
   - 7.5 Earthquake Fault Lines
8. [Technical Implementation](#8-technical-implementation)
   - 8.1 Chunk-Based Generation
   - 8.2 LOD for Terrain Rendering
   - 8.3 Terrain Modification
   - 8.4 Heightmap Serialization
   - 8.5 Bevy Integration
9. [Reference Games](#9-reference-games)
   - 9.1 SimCity 4
   - 9.2 Cities: Skylines
   - 9.3 Banished
   - 9.4 Dwarf Fortress
   - 9.5 Minecraft

---

## 1. Heightmap Generation

The heightmap is the foundation of all terrain. Every other system -- water, biomes, resources, building placement, road costs -- derives from the elevation grid. Getting the heightmap right is the single most important step in procedural terrain.

### 1.1 Noise Algorithm Selection

Three noise algorithms are commonly used for terrain heightmaps. All three produce coherent gradient noise (smooth, continuous values without sharp discontinuities), but they differ in performance, visual quality, and artifact characteristics.

#### Perlin Noise (Classic/Improved)

Ken Perlin's original algorithm (1983, improved 2002) works by:
1. Defining a regular grid of gradient vectors at integer lattice points
2. For each sample point, identifying the surrounding grid cell (4 corners in 2D)
3. Computing dot products between each corner's gradient and the vector from that corner to the sample point
4. Interpolating these dot products using a smooth fade curve (the improved version uses `6t^5 - 15t^4 + 10t^3` instead of the original `3t^2 - 2t^3`)

**Artifacts:** Perlin noise has a well-known axis-alignment problem. Because the gradient grid is rectilinear, the noise tends to produce features that align with the X and Y axes. At certain frequencies this creates a subtle "quilted" or grid-like pattern, especially visible when the noise is used raw (without fractal layering). The improved version reduces but does not eliminate this.

**Performance:** Perlin noise requires 4 gradient lookups and 4 dot products in 2D (8 in 3D). Each dot product is `dx*gx + dy*gy`. The fade curve is 3 multiplications. Total: roughly 20 floating-point operations per 2D sample.

**Visual quality rating for terrain:** 6/10. Adequate when used as one layer in fBm, but axis alignment can be visible in flat areas. Not recommended as the sole noise source.

```
// Pseudocode: Classic Perlin 2D
fn perlin_2d(x: f32, y: f32, perm: &[u8; 512]) -> f32 {
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;
    let xf = x - x.floor();  // fractional part
    let yf = y - y.floor();

    // Fade curves (improved Perlin)
    let u = xf * xf * xf * (xf * (xf * 6.0 - 15.0) + 10.0);
    let v = yf * yf * yf * (yf * (yf * 6.0 - 15.0) + 10.0);

    // Hash corner coordinates to gradient indices
    let aa = perm[(perm[xi & 255] + yi) & 255];
    let ab = perm[(perm[xi & 255] + yi + 1) & 255];
    let ba = perm[(perm[(xi + 1) & 255] + yi) & 255];
    let bb = perm[(perm[(xi + 1) & 255] + yi + 1) & 255];

    // Gradient dot products
    let g00 = grad(aa, xf, yf);
    let g10 = grad(ba, xf - 1.0, yf);
    let g01 = grad(ab, xf, yf - 1.0);
    let g11 = grad(bb, xf - 1.0, yf - 1.0);

    // Bilinear interpolation
    let x0 = lerp(g00, g10, u);
    let x1 = lerp(g01, g11, u);
    lerp(x0, x1, v)
}
```

#### Simplex Noise

Ken Perlin's successor algorithm (2001) addresses the axis-alignment problem by using a simplex grid (equilateral triangles in 2D, tetrahedra in 3D) instead of a rectangular grid.

Key differences from Perlin:
1. The input coordinate is skewed onto a simplex grid using the factor `F = (sqrt(N+1) - 1) / N` (where N=2 for 2D, so F = (sqrt(3)-1)/2 = 0.366)
2. Only N+1 corners contribute (3 in 2D vs 4 for Perlin; 4 in 3D vs 8 for Perlin)
3. Each corner uses a radial falloff kernel `(0.5 - dx^2 - dy^2)^4` instead of an interpolated fade curve
4. No interpolation step needed -- contributions are summed directly

**Artifacts:** Simplex noise has no axis-aligned artifacts. However, the triangular grid can sometimes produce faint hexagonal patterns at certain scales. These are much less objectionable than Perlin's grid alignment and are effectively invisible once fBm layering is applied.

**Performance:** In 2D, simplex noise requires only 3 gradient lookups and 3 dot products (vs Perlin's 4+4). The radial kernel is slightly more expensive per corner (`(0.5 - r^2)^4 * dot`), but fewer corners are evaluated. Net result: roughly 15-20% faster than Perlin in 2D, and the advantage grows dramatically in 3D (4 corners vs 8 = nearly 2x faster).

**Patent issue:** Simplex noise was patented by Ken Perlin (US Patent 6,867,776, expired 2022). The patent covered the specific simplex grid + gradient approach. While this patent has now expired, the open-source community developed alternatives during its validity period, leading to OpenSimplex and OpenSimplex2.

**Visual quality rating for terrain:** 8/10. Excellent isotropy, smooth gradients, good feature distribution.

```
// Pseudocode: Simplex 2D (core structure)
fn simplex_2d(x: f32, y: f32) -> f32 {
    let F2: f32 = 0.5 * (3.0_f32.sqrt() - 1.0); // skew factor
    let G2: f32 = (3.0 - 3.0_f32.sqrt()) / 6.0;  // unskew factor

    // Skew input space to simplex grid
    let s = (x + y) * F2;
    let i = (x + s).floor() as i32;
    let j = (y + s).floor() as i32;

    // Unskew back to find first simplex corner in input space
    let t = (i + j) as f32 * G2;
    let x0 = x - (i as f32 - t);
    let y0 = y - (j as f32 - t);

    // Determine which simplex triangle we're in
    let (i1, j1) = if x0 > y0 { (1, 0) } else { (0, 1) };

    let x1 = x0 - i1 as f32 + G2;
    let y1 = y0 - j1 as f32 + G2;
    let x2 = x0 - 1.0 + 2.0 * G2;
    let y2 = y0 - 1.0 + 2.0 * G2;

    // Sum contributions from each corner
    let mut result = 0.0;
    for &(dx, dy, corner_hash) in &[(x0, y0, hash(i, j)),
                                     (x1, y1, hash(i+i1, j+j1)),
                                     (x2, y2, hash(i+1, j+1))] {
        let t = 0.5 - dx*dx - dy*dy;
        if t > 0.0 {
            let t2 = t * t;
            result += t2 * t2 * gradient_dot(corner_hash, dx, dy);
        }
    }
    result * 70.0  // Scale to approximately [-1, 1]
}
```

#### OpenSimplex2 (What We Use)

OpenSimplex (2014, Kurt Spencer) was created as a patent-free alternative to simplex noise. OpenSimplex2 (2019) is the refined second generation, which is what `fastnoise-lite` provides via `NoiseType::OpenSimplex2`.

OpenSimplex2 uses a different lattice geometry than simplex noise:
- In 2D: Uses a honeycomb-like lattice derived from the A2 lattice (which gives equilateral triangles, similar to simplex but with different coordinate mapping)
- The key innovation is using a different skewing approach that avoids the simplex noise patent while achieving comparable or better isotropy
- Two variants exist: OpenSimplex2 (standard) and OpenSimplex2S (smoother, with a larger kernel radius)

**Artifacts:** OpenSimplex2 has the best isotropy of the three algorithms. No axis alignment, no visible hexagonal patterns. The noise field is highly uniform in all directions. The standard variant has slightly more "bubbly" features compared to simplex noise; the S variant smooths these out at a small performance cost.

**Performance:** Comparable to simplex noise -- roughly 3-4 gradient evaluations in 2D. The `fastnoise-lite` implementation is highly optimized with SIMD-friendly code paths. In practice, generating 256x256 samples (65,536 evaluations) takes under 1ms on modern hardware.

**Visual quality rating for terrain:** 9/10. The best general-purpose noise for terrain. Highly isotropic, no patents, excellent implementations available.

**Why it is the right choice for Megacity:** The codebase already uses `fastnoise-lite` with OpenSimplex2. This is the correct choice. The library also provides built-in fBm (fractal) modes and domain warping, which means we can extend the terrain generator without adding new dependencies.

#### Algorithm Comparison Summary

```
Algorithm     | 2D Corners | Artifacts        | Patent  | Speed (relative) | Quality
--------------+------------+------------------+---------+------------------+--------
Perlin        | 4          | Axis-aligned     | No      | 1.0x baseline    | 6/10
Simplex       | 3          | Faint hexagonal  | Expired | 1.15-1.20x       | 8/10
OpenSimplex2  | 3-4        | None significant | No      | 1.10-1.15x       | 9/10
OpenSimplex2S | 4          | None             | No      | 0.95-1.0x        | 9/10
```

### 1.2 Fractal Brownian Motion (fBm)

A single noise evaluation produces smooth, blobby features at a single scale. Real terrain has detail at every scale -- large mountain ranges, medium hills, small ridges, tiny bumps. Fractal Brownian Motion (fBm) achieves this by layering multiple noise evaluations ("octaves") at increasing frequencies and decreasing amplitudes.

#### The fBm Formula

```
fn fbm(x: f32, y: f32, octaves: u32, lacunarity: f32, persistence: f32) -> f32 {
    let mut total = 0.0;
    let mut frequency = 1.0;
    let mut amplitude = 1.0;
    let mut max_amplitude = 0.0;

    for _ in 0..octaves {
        total += noise(x * frequency, y * frequency) * amplitude;
        max_amplitude += amplitude;
        frequency *= lacunarity;
        amplitude *= persistence;
    }

    total / max_amplitude  // Normalize to [-1, 1] (or [0, 1] after remapping)
}
```

**Parameters explained:**

- **Octaves:** The number of noise layers summed together. Each octave adds detail at a finer scale. More octaves = more detail but more computation (linear cost). For a 256x256 grid, frequencies above 128 produce sub-pixel noise that wastes computation.

- **Lacunarity:** The frequency multiplier between octaves. Standard value is 2.0 (each octave is double the frequency). Values above 2.0 create "gaps" between detail scales, producing a more artificial look. Values below 2.0 create overlapping scales, producing a muddier but smoother result.

- **Persistence:** The amplitude multiplier between octaves (sometimes called "gain"). Standard value is 0.5 (each octave is half the amplitude). Higher persistence (0.6-0.7) makes small details more prominent, creating rougher terrain. Lower persistence (0.3-0.4) makes large features dominate, creating smoother terrain.

#### Recommended Values for City-Building Terrain

The goal for a city builder is terrain that is **mostly flat with interesting features** -- players need large buildable areas, but completely flat terrain is boring. This calls for specific fBm tuning:

```
// Terrain that works well for city building
const TERRAIN_OCTAVES: u32 = 6;
const TERRAIN_LACUNARITY: f32 = 2.0;
const TERRAIN_PERSISTENCE: f32 = 0.45;  // Slightly below 0.5 for smoother terrain
const TERRAIN_BASE_FREQUENCY: f32 = 0.008;  // Already used in our codebase
```

**Why 6 octaves?** With base frequency 0.008 and lacunarity 2.0:
- Octave 0: freq = 0.008, wavelength = 125 cells (2km) -- continent-scale hills
- Octave 1: freq = 0.016, wavelength = 62.5 cells (1km) -- large hills
- Octave 2: freq = 0.032, wavelength = 31.25 cells (500m) -- hill clusters
- Octave 3: freq = 0.064, wavelength = 15.6 cells (250m) -- individual hills
- Octave 4: freq = 0.128, wavelength = 7.8 cells (125m) -- terrain bumps
- Octave 5: freq = 0.256, wavelength = 3.9 cells (62m) -- fine detail

Octaves 6+ would have wavelengths below 2 cells, which is below the Nyquist limit for our grid and would alias. Six octaves is the sweet spot.

**Why persistence 0.45?** The amplitude ratio of the finest octave to the coarsest is `0.45^5 = 0.0185`. This means the large-scale hills (octave 0) are 54x stronger than the finest detail (octave 5). The terrain reads as gentle rolling hills with subtle surface variation, not jagged mountains.

**Amplitude breakdown at persistence 0.45:**
```
Octave 0: amplitude = 1.000  (100.0%)  -- broad landforms
Octave 1: amplitude = 0.450  ( 45.0%)  -- major hills
Octave 2: amplitude = 0.203  ( 20.3%)  -- medium features
Octave 3: amplitude = 0.091  (  9.1%)  -- small mounds
Octave 4: amplitude = 0.041  (  4.1%)  -- surface variation
Octave 5: amplitude = 0.018  (  1.8%)  -- fine texture
Total max amplitude: 1.803 (normalize by dividing)
```

**Comparison of persistence values:**

| Persistence | Character               | Good For                           |
|-------------|-------------------------|------------------------------------|
| 0.30        | Very smooth, rolling    | Plains, deserts, gentle coastlines |
| 0.40        | Smooth with texture     | Farmland, suburbs                  |
| 0.45        | Moderate, city-friendly | General city-building terrain      |
| 0.50        | Standard fractal        | Varied landscapes                  |
| 0.60        | Rough, detailed         | Mountainous regions                |
| 0.70        | Very rough              | Alpine/rocky terrain               |

#### Using fastnoise-lite's Built-In fBm

The `fastnoise-lite` crate provides built-in fractal modes, which avoids manual octave looping:

```rust
use fastnoise_lite::{FastNoiseLite, NoiseType, FractalType};

let mut noise = FastNoiseLite::with_seed(seed);
noise.set_noise_type(Some(NoiseType::OpenSimplex2));
noise.set_fractal_type(Some(FractalType::FBm));
noise.set_fractal_octaves(Some(6));
noise.set_fractal_lacunarity(Some(2.0));
noise.set_fractal_gain(Some(0.45));  // "gain" = persistence
noise.set_frequency(Some(0.008));

// Now each call to get_noise_2d automatically computes 6-octave fBm
let value = noise.get_noise_2d(x, y);  // Returns roughly [-1, 1]
```

This is the recommended approach for upgrading the existing `generate_terrain()` function. The library's internal loop is optimized and avoids allocations.

#### Ridged Multifractal Noise

Standard fBm produces smooth, rounded features. For mountain ridges and sharp peaks, **ridged noise** is more appropriate. The key modification is taking the absolute value of each octave and inverting it:

```
fn ridged_fbm(x: f32, y: f32, octaves: u32, lacunarity: f32, persistence: f32) -> f32 {
    let mut total = 0.0;
    let mut frequency = 1.0;
    let mut amplitude = 1.0;
    let mut weight = 1.0;

    for _ in 0..octaves {
        let signal = noise(x * frequency, y * frequency);
        let signal = 1.0 - signal.abs();  // Invert absolute value: valleys become ridges
        let signal = signal * signal;      // Sharpen the ridges
        let signal = signal * weight;      // Weight by previous octave

        total += signal * amplitude;
        weight = (signal * 2.0).clamp(0.0, 1.0);  // Higher signal = more detail in next octave

        frequency *= lacunarity;
        amplitude *= persistence;
    }
    total
}
```

The `weight` feedback loop is what creates the characteristic "ridges meeting at peaks" pattern. High points in one octave cause more detail to appear in the next octave at that location, creating convergent ridge lines.

`fastnoise-lite` supports this directly:
```rust
noise.set_fractal_type(Some(FractalType::Ridged));
```

**Recommended usage:** Do not use ridged noise for the entire map. Instead, blend it with standard fBm based on a mask:

```rust
// Mountain regions use ridged noise, plains use standard fBm
let continent_noise = continent_fbm.get_noise_2d(x, y);  // smooth, large-scale
let mountain_mask = ((continent_noise - 0.3) / 0.4).clamp(0.0, 1.0);  // 1.0 in high areas

let smooth = standard_fbm.get_noise_2d(x, y);
let ridged = ridged_fbm.get_noise_2d(x, y);
let elevation = lerp(smooth, ridged, mountain_mask);
```

### 1.3 Domain Warping

Domain warping is one of the most powerful techniques for making procedurally generated terrain look natural. The idea is simple: instead of sampling `noise(x, y)`, you first distort the coordinates using another noise function:

```
fn warped_noise(x: f32, y: f32) -> f32 {
    let warp_x = warp_noise_x.get_noise_2d(x, y);
    let warp_y = warp_noise_y.get_noise_2d(x, y);
    let warp_strength = 40.0;  // In grid cells

    terrain_noise.get_noise_2d(
        x + warp_x * warp_strength,
        y + warp_y * warp_strength
    )
}
```

This transforms the regular noise patterns into flowing, organic shapes that resemble real geological formations.

#### Why Domain Warping Works

Real terrain is shaped by geological processes (tectonic folding, glacial carving, differential erosion) that create systematic distortions in what would otherwise be random topography. Domain warping simulates these distortions: the warp field acts like a displacement map that pushes terrain features around as if they had been folded and compressed.

The visual result is terrain with:
- Elongated ridges instead of round blobs
- Winding valleys instead of simple depressions
- Features that seem to flow and connect naturally
- An overall "geological" character that pure noise lacks

#### Single-Layer Warping

The simplest form uses one warp layer:

```rust
// Setup: two independent noise generators for X and Y displacement
let mut warp_x_noise = FastNoiseLite::with_seed(seed + 1000);
warp_x_noise.set_noise_type(Some(NoiseType::OpenSimplex2));
warp_x_noise.set_fractal_type(Some(FractalType::FBm));
warp_x_noise.set_fractal_octaves(Some(4));
warp_x_noise.set_frequency(Some(0.006));  // Slightly lower freq than terrain

let mut warp_y_noise = FastNoiseLite::with_seed(seed + 2000);
warp_y_noise.set_noise_type(Some(NoiseType::OpenSimplex2));
warp_y_noise.set_fractal_type(Some(FractalType::FBm));
warp_y_noise.set_fractal_octaves(Some(4));
warp_y_noise.set_frequency(Some(0.006));

// Application
let warp_strength = 30.0;  // cells of displacement
let wx = warp_x_noise.get_noise_2d(x as f32, y as f32) * warp_strength;
let wy = warp_y_noise.get_noise_2d(x as f32, y as f32) * warp_strength;
let elevation = terrain_noise.get_noise_2d(x as f32 + wx, y as f32 + wy);
```

**Warp strength guidelines for our 256x256 grid:**
- 10-20 cells: Subtle warping, barely noticeable but removes "noise regularity"
- 30-50 cells: Moderate warping, creates flowing terrain features. Best for general city terrain.
- 60-80 cells: Heavy warping, creates dramatic folds and elongated ridges
- 100+: Extreme warping, terrain becomes surreal and loses natural appearance

#### Multi-Layer Warping (Warping the Warp)

Inigo Quilez popularized the technique of recursive domain warping, where the warp coordinates are themselves warped:

```
fn double_warped(x: f32, y: f32) -> f32 {
    // First warp layer
    let w1x = noise_a(x, y);
    let w1y = noise_b(x, y);

    // Second warp layer (warps the first)
    let w2x = noise_c(x + w1x * 20.0, y + w1y * 20.0);
    let w2y = noise_d(x + w1x * 20.0, y + w1y * 20.0);

    // Final terrain sample with double-warped coordinates
    terrain_noise(
        x + w2x * 40.0,
        y + w2y * 40.0
    )
}
```

Each layer of warping adds more organic character. Two layers is usually sufficient -- three layers starts to look like marble textures rather than terrain.

#### fastnoise-lite Domain Warp Support

The library has built-in domain warping via `FractalType::DomainWarpProgressive` and `FractalType::DomainWarpIndependent`:

```rust
let mut noise = FastNoiseLite::with_seed(seed);
noise.set_noise_type(Some(NoiseType::OpenSimplex2));
noise.set_fractal_type(Some(FractalType::DomainWarpProgressive));
noise.set_domain_warp_amp(Some(30.0));  // warp strength in input-space units
noise.set_fractal_octaves(Some(4));
noise.set_frequency(Some(0.008));
```

The difference between the two warp modes:
- **DomainWarpIndependent:** Each octave's warp is calculated independently. Produces more uniform warping.
- **DomainWarpProgressive:** Each octave's warp is applied to the input of the next octave (recursive/progressive). Produces more complex, flowing patterns. Generally preferred for terrain.

### 1.4 Height Distribution Shaping

Raw fBm noise produces values that follow an approximately Gaussian distribution centered around 0. When normalized to [0,1], the histogram is bell-shaped: lots of mid-elevation values, few extreme highs or lows. This is not ideal for city terrain, which typically wants:

- Large flat areas (for building) centered around a specific elevation band
- Some water areas (low elevation)
- Occasional hills or ridges (high elevation) for visual interest
- Not too much extreme terrain

Several techniques reshape the height distribution to achieve this.

#### Transfer Functions (Remapping Curves)

The simplest approach is to apply a transfer function after generating the noise. This remaps the elevation values through a curve that concentrates values where you want them.

**Plateau function (flattens mid-range, preserves extremes):**
```rust
fn plateau_remap(h: f32) -> f32 {
    // h is in [0, 1]
    // This creates a flat plateau in the 0.4-0.6 range
    // while preserving valleys and peaks
    if h < 0.3 {
        h * 0.7  // Compress low values (deeper water)
    } else if h < 0.7 {
        // Flatten the middle range: map [0.3, 0.7] -> [0.21, 0.51]
        // This creates a 30-cell-unit plateau
        0.21 + (h - 0.3) * 0.75
    } else {
        // Stretch high values (taller hills)
        0.51 + (h - 0.7) * 1.63
    }
}
```

**Power curve (adjustable flatness):**
```rust
fn power_remap(h: f32, exponent: f32) -> f32 {
    // exponent < 1.0: pushes values toward 1.0 (more land, less water)
    // exponent > 1.0: pushes values toward 0.0 (more water, less land)
    // exponent = 1.0: no change
    h.powf(exponent)
}

// For city terrain: exponent = 0.7 gives ~75% land, ~25% water
// (assuming WATER_THRESHOLD = 0.35)
```

**S-curve (concentrates mid-values, like real coastal plains):**
```rust
fn smoothstep_remap(h: f32) -> f32 {
    // Hermite interpolation: concentrates values in the middle
    // 3h^2 - 2h^3, but applied after shifting to center the interesting range
    let t = h.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
```

#### Histogram Equalization

For maximum control, compute the actual height distribution and equalize it:

```rust
fn histogram_equalize(heights: &mut [f32], width: usize, height: usize) {
    let total = (width * height) as f32;

    // Build cumulative distribution function
    let mut sorted: Vec<f32> = heights.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // Create a lookup: for each height value, what percentile is it?
    // Use this to remap heights to a uniform distribution
    for h in heights.iter_mut() {
        // Binary search for rank in sorted array
        let rank = sorted.partition_point(|v| *v < *h) as f32;
        *h = rank / total;  // Now uniformly distributed in [0, 1]
    }
}
```

After equalization, the height values are uniformly distributed. You can then apply a target distribution by mapping through the inverse CDF of your desired distribution.

**Recommended approach for Megacity:** Use a two-stage process:
1. Generate raw fBm values
2. Apply a custom transfer function that creates ~15-20% water, ~60-70% flat buildable land, ~10-15% hills

```rust
fn city_terrain_remap(h: f32) -> f32 {
    // Input: h in [0, 1] from normalized fBm
    // Output: elevation in [0, 1] where < 0.35 = water

    // Target: 18% water, 65% flat land (0.35-0.55), 17% hills/mountains

    if h < 0.18 {
        // Bottom 18% -> water (elevation 0.0 to 0.35)
        h / 0.18 * 0.35
    } else if h < 0.83 {
        // Middle 65% -> flat buildable land (elevation 0.35 to 0.55)
        // Map [0.18, 0.83] -> [0.35, 0.55]: a wide input range into a narrow output range
        0.35 + (h - 0.18) / 0.65 * 0.20
    } else {
        // Top 17% -> hills and mountains (elevation 0.55 to 1.0)
        0.55 + (h - 0.83) / 0.17 * 0.45
    }
}
```

This produces terrain where:
- Water bodies are well-defined depressions (not just "barely below threshold")
- Most of the map is gently undulating flat-ish terrain suitable for building
- Hills are real hills with meaningful height differences, not just slightly elevated noise

#### Terracing (Optional, Style-Dependent)

For a more stylized look, you can quantize elevation into discrete levels:

```rust
fn terrace(h: f32, levels: u32) -> f32 {
    let step = 1.0 / levels as f32;
    let level = (h / step).floor();
    let frac = (h / step) - level;

    // Smooth terrace: blend between flat level and slope
    let blend = smoothstep(frac);  // 0 at level start, 1 at level end
    (level + blend) * step
}
```

This is not recommended for Megacity's realistic style, but is included for completeness. Games like Townscaper use heavy terracing.

### 1.5 Noise Scale for 256x256 City Terrain

The base frequency parameter controls how many features appear across the map. For city building, the right frequency depends on the physical scale and gameplay goals.

**Physical scale reminder:** Each cell is 16m x 16m. The full map is 4,096m x 4,096m. This is a small-to-medium city -- roughly the size of central Manhattan (which is about 3.7km long by 2km wide).

#### Frequency and Feature Size

The relationship between frequency and feature wavelength:

```
wavelength (in cells) = 1.0 / frequency
wavelength (in meters) = wavelength_cells * CELL_SIZE

Frequency  | Wavelength (cells) | Wavelength (meters) | Feature type
-----------+--------------------+---------------------+-----------------------------
0.002      | 500 cells          | 8,000m              | Continent-scale (larger than map)
0.004      | 250 cells          | 4,000m              | Island/peninsula (fills map)
0.008      | 125 cells          | 2,000m              | Large hills, river valleys
0.016      | 62.5 cells         | 1,000m              | City districts, ridgelines
0.032      | 31.25 cells        | 500m                | Neighborhoods, individual hills
0.064      | 15.6 cells         | 250m                | Small bumps, terrain texture
0.128      | 7.8 cells          | 125m                | Near-cell-level variation
```

#### Our Current Frequency (0.008)

The existing code uses `frequency = 0.008`, which produces features with a wavelength of about 125 cells or 2km. For a 256-cell-wide map, this means roughly 2 major features across the map. With single-octave noise, this creates 1-2 large blobs of water/land, which is decent but monotonous.

**With 6-octave fBm**, the same base frequency will produce:
- 2 major landforms (from octave 0)
- 4 medium hills (from octave 1)
- 8 small hills (from octave 2)
- 16 terrain bumps (from octave 3)
- And progressively finer detail from octaves 4-5

This is a good balance. The base frequency of 0.008 is appropriate and should be kept.

#### Multi-Scale Continent Approach

For maximum control, use separate noise generators for large-scale vs small-scale features:

```rust
// Layer 1: Continental shape (determines land vs water at the broadest scale)
let mut continent = FastNoiseLite::with_seed(seed);
continent.set_noise_type(Some(NoiseType::OpenSimplex2));
continent.set_frequency(Some(0.003));  // Very large features
// NO fractal -- just one octave for the broad shape

// Layer 2: Terrain detail (adds hills, valleys, texture)
let mut detail = FastNoiseLite::with_seed(seed + 100);
detail.set_noise_type(Some(NoiseType::OpenSimplex2));
detail.set_fractal_type(Some(FractalType::FBm));
detail.set_fractal_octaves(Some(5));
detail.set_fractal_gain(Some(0.45));
detail.set_frequency(Some(0.015));  // Medium-scale features

// Combine: continent sets the broad shape, detail adds texture
let continent_val = (continent.get_noise_2d(x, y) + 1.0) * 0.5; // [0, 1]
let detail_val = detail.get_noise_2d(x, y) * 0.15;  // +/- 0.15
let elevation = (continent_val + detail_val).clamp(0.0, 1.0);
```

This two-layer approach gives the map designer control over the overall land/water distribution (via the continent layer) independently of the terrain texture (via the detail layer). It is particularly useful for ensuring playable starting conditions (see Section 6).

#### Edge Falloff (Island Maps)

To create island-style maps surrounded by water, multiply the elevation by a radial falloff:

```rust
fn edge_falloff(x: usize, y: usize, width: usize, height: usize) -> f32 {
    let cx = width as f32 / 2.0;
    let cy = height as f32 / 2.0;
    let dx = (x as f32 - cx) / cx;  // [-1, 1]
    let dy = (y as f32 - cy) / cy;  // [-1, 1]

    let dist = (dx * dx + dy * dy).sqrt();  // 0 at center, ~1.41 at corners

    // Smooth falloff: 1.0 in center, 0.0 at edges
    let falloff_start = 0.5;   // Start fading at 50% distance from center
    let falloff_end = 0.95;    // Fully submerged at 95% distance

    if dist < falloff_start {
        1.0
    } else if dist < falloff_end {
        let t = (dist - falloff_start) / (falloff_end - falloff_start);
        1.0 - t * t  // Quadratic falloff
    } else {
        0.0
    }
}

// Usage: elevation *= edge_falloff(x, y, GRID_WIDTH, GRID_HEIGHT);
```

This is optional but useful for scenarios where the player wants a bounded island to build on, rather than terrain that extends to the map edges.

---

## 2. Hydraulic Erosion Simulation

Raw noise-generated terrain, even with fBm and domain warping, has a characteristic "fluffy" or "pillowy" look that does not resemble real terrain. Real landscapes are carved by water flowing downhill over millions of years. Hydraulic erosion simulation applies this process to the heightmap, creating realistic river valleys, drainage networks, alluvial fans, and carved hillsides.

Erosion is the single most impactful post-processing step for terrain quality. A simple heightmap with good erosion looks far more realistic than a complex multi-noise heightmap without erosion.

### 2.1 Particle-Based Erosion Algorithm

The most common approach (used by Sebastian Lague, Hans Theobald Beyer, and many game terrain tools) simulates individual water droplets rolling downhill across the heightmap.

#### The Droplet Lifecycle

Each simulated water particle follows this lifecycle:
1. **Spawn** at a random position on the map
2. **Flow downhill** following the steepest gradient
3. **Erode** material from the ground beneath it (proportional to speed and slope)
4. **Carry sediment** in suspension (limited by carrying capacity)
5. **Deposit sediment** when carrying capacity decreases (on flat areas, in depressions)
6. **Evaporate** gradually, losing water volume
7. **Die** when water volume reaches zero, depositing remaining sediment

#### The Particle Data Structure

```rust
struct ErosionParticle {
    x: f32,                // Position (continuous, not grid-snapped)
    y: f32,
    dx: f32,               // Direction/velocity X
    dy: f32,               // Direction/velocity Y
    speed: f32,            // Current speed (magnitude of velocity)
    water: f32,            // Current water volume
    sediment: f32,         // Currently carried sediment
}

impl Default for ErosionParticle {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            dx: 0.0,
            dy: 0.0,
            speed: 0.0,
            water: 1.0,       // Start with full water
            sediment: 0.0,    // Start with no sediment
        }
    }
}
```

#### Erosion Parameters

```rust
struct ErosionParams {
    // Erosion behavior
    inertia: f32,           // 0.0-1.0, how much previous direction influences new direction
                            // Higher = smoother paths, Lower = more responsive to terrain
    capacity_factor: f32,   // Sediment carrying capacity multiplier
    min_slope: f32,         // Minimum slope for capacity calculation (prevents division issues)
    erosion_rate: f32,      // How fast material is removed
    deposition_rate: f32,   // How fast excess sediment is deposited
    erosion_radius: i32,    // Radius of erosion brush (in cells)

    // Particle lifecycle
    evaporation_rate: f32,  // Water loss per step (0.01 = 1% per step)
    gravity: f32,           // Acceleration due to slope
    max_lifetime: u32,      // Maximum steps before particle dies
    initial_water: f32,     // Starting water volume
    initial_speed: f32,     // Starting speed
}

impl Default for ErosionParams {
    fn default() -> Self {
        Self {
            inertia: 0.3,
            capacity_factor: 8.0,
            min_slope: 0.01,
            erosion_rate: 0.3,
            deposition_rate: 0.3,
            erosion_radius: 3,
            evaporation_rate: 0.01,
            gravity: 10.0,
            max_lifetime: 100,
            initial_water: 1.0,
            initial_speed: 1.0,
        }
    }
}
```

#### The Core Simulation Loop

```rust
fn simulate_erosion(
    heightmap: &mut [f32],
    width: usize,
    height: usize,
    num_particles: u32,
    params: &ErosionParams,
    rng: &mut impl Rng,
) {
    // Precompute erosion brush weights (Gaussian kernel)
    let brush = compute_erosion_brush(params.erosion_radius);

    for _ in 0..num_particles {
        // Spawn particle at random position
        let mut p = ErosionParticle {
            x: rng.gen_range(1.0..(width - 2) as f32),
            y: rng.gen_range(1.0..(height - 2) as f32),
            speed: params.initial_speed,
            water: params.initial_water,
            ..Default::default()
        };

        for _ in 0..params.max_lifetime {
            let ix = p.x as usize;
            let iy = p.y as usize;

            if ix < 1 || ix >= width - 1 || iy < 1 || iy >= height - 1 {
                break;  // Particle left the map
            }

            // 1. Calculate height and gradient at current position using bilinear interpolation
            let (h, grad_x, grad_y) = height_and_gradient(heightmap, width, p.x, p.y);

            // 2. Update direction (blend old direction with gradient using inertia)
            p.dx = p.dx * params.inertia - grad_x * (1.0 - params.inertia);
            p.dy = p.dy * params.inertia - grad_y * (1.0 - params.inertia);

            // Normalize direction
            let len = (p.dx * p.dx + p.dy * p.dy).sqrt();
            if len < 1e-6 {
                // Random direction if on a flat spot
                let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                p.dx = angle.cos();
                p.dy = angle.sin();
            } else {
                p.dx /= len;
                p.dy /= len;
            }

            // 3. Move particle
            let new_x = p.x + p.dx;
            let new_y = p.y + p.dy;

            // Check bounds
            if new_x < 1.0 || new_x >= (width - 2) as f32
                || new_y < 1.0 || new_y >= (height - 2) as f32 {
                break;
            }

            // 4. Calculate height difference
            let new_h = interpolate_height(heightmap, width, new_x, new_y);
            let delta_h = new_h - h;

            // 5. Calculate sediment carrying capacity
            // Capacity increases with speed, water volume, and slope
            let capacity = (-delta_h).max(params.min_slope)
                * p.speed
                * p.water
                * params.capacity_factor;

            // 6. Erode or deposit
            if p.sediment > capacity || delta_h > 0.0 {
                // DEPOSIT: carrying too much sediment, or going uphill
                let deposit_amount = if delta_h > 0.0 {
                    // Going uphill: deposit enough to fill the gap (but not more than we carry)
                    delta_h.min(p.sediment)
                } else {
                    // On flat/downhill but over capacity: deposit excess
                    (p.sediment - capacity) * params.deposition_rate
                };

                p.sediment -= deposit_amount;
                // Deposit at current position (bilinear distribution)
                deposit_sediment(heightmap, width, p.x, p.y, deposit_amount);

            } else {
                // ERODE: under capacity, pick up material
                let erode_amount = ((capacity - p.sediment) * params.erosion_rate)
                    .min(-delta_h);  // Don't erode more than the height difference

                // Apply erosion using brush (spreads erosion over nearby cells)
                apply_erosion_brush(heightmap, width, height, ix, iy, erode_amount, &brush);
                p.sediment += erode_amount;
            }

            // 7. Update speed (accelerate on steep slopes, decelerate on flat)
            p.speed = (p.speed * p.speed + delta_h * params.gravity).abs().sqrt();

            // 8. Evaporate water
            p.water *= 1.0 - params.evaporation_rate;

            // 9. Move to new position
            p.x = new_x;
            p.y = new_y;

            // Terminate if water is gone
            if p.water < 0.001 {
                break;
            }
        }

        // Particle dies: deposit remaining sediment
        if p.sediment > 0.001 {
            let ix = p.x as usize;
            let iy = p.y as usize;
            if ix < width && iy < height {
                deposit_sediment(heightmap, width, p.x, p.y, p.sediment);
            }
        }
    }
}
```

#### Bilinear Height and Gradient Calculation

The particle position is continuous (floating point), not grid-snapped. Heights and gradients are computed using bilinear interpolation of the four surrounding grid cells:

```rust
fn height_and_gradient(
    heightmap: &[f32],
    width: usize,
    x: f32,
    y: f32,
) -> (f32, f32, f32) {
    let ix = x as usize;
    let iy = y as usize;
    let fx = x - ix as f32;  // Fractional X (0..1 within cell)
    let fy = y - iy as f32;  // Fractional Y

    // Four corner heights
    let h00 = heightmap[iy * width + ix];
    let h10 = heightmap[iy * width + ix + 1];
    let h01 = heightmap[(iy + 1) * width + ix];
    let h11 = heightmap[(iy + 1) * width + ix + 1];

    // Interpolated height at (x, y)
    let h = h00 * (1.0 - fx) * (1.0 - fy)
          + h10 * fx * (1.0 - fy)
          + h01 * (1.0 - fx) * fy
          + h11 * fx * fy;

    // Gradient (partial derivatives of the bilinear interpolation)
    let grad_x = (h10 - h00) * (1.0 - fy) + (h11 - h01) * fy;
    let grad_y = (h01 - h00) * (1.0 - fx) + (h11 - h10) * fx;

    (h, grad_x, grad_y)
}
```

#### The Erosion Brush

Instead of eroding only the single cell under the particle, erosion is spread over a circular area using a weighted brush. This prevents sharp single-cell pits and creates smoother, more realistic channels.

```rust
struct ErosionBrush {
    offsets: Vec<(i32, i32)>,   // Relative cell offsets
    weights: Vec<f32>,          // Normalized weights (sum to 1.0)
}

fn compute_erosion_brush(radius: i32) -> ErosionBrush {
    let mut offsets = Vec::new();
    let mut weights = Vec::new();
    let mut total_weight = 0.0;

    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let dist_sq = (dx * dx + dy * dy) as f32;
            let r_sq = (radius * radius) as f32;

            if dist_sq <= r_sq {
                let weight = 1.0 - (dist_sq / r_sq).sqrt();  // Linear falloff
                offsets.push((dx, dy));
                weights.push(weight);
                total_weight += weight;
            }
        }
    }

    // Normalize weights
    for w in &mut weights {
        *w /= total_weight;
    }

    ErosionBrush { offsets, weights }
}

fn apply_erosion_brush(
    heightmap: &mut [f32],
    width: usize,
    height: usize,
    cx: usize,
    cy: usize,
    amount: f32,
    brush: &ErosionBrush,
) {
    for (i, &(dx, dy)) in brush.offsets.iter().enumerate() {
        let nx = cx as i32 + dx;
        let ny = cy as i32 + dy;

        if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
            let idx = ny as usize * width + nx as usize;
            heightmap[idx] -= amount * brush.weights[i];
        }
    }
}
```

### 2.2 River Valley and Drainage Basin Formation

The emergent property of particle erosion is the formation of realistic drainage networks. Here is why it works:

1. **Convergent flow:** When multiple particles flow downhill from different starting points, they tend to converge into the same low channels. Each particle deepens its channel slightly, making future particles more likely to follow the same path. This positive feedback creates river valleys.

2. **Branching networks:** Tributaries form naturally because particles starting at different locations on a ridge flow down different sides of the ridge but converge at the base. The result is a tree-like drainage pattern that closely resembles real river systems.

3. **V-shaped valleys:** Erosion carves V-shaped cross-sections because particles flow fastest (and erode most) at the bottom of the channel where the slope is steepest. The sides of the valley erode more slowly.

4. **Alluvial fans and deltas:** When a fast-flowing particle reaches a flat area, its speed drops and carrying capacity decreases. It deposits sediment, creating a fan-shaped deposit. This naturally creates alluvial fans at the base of mountains and deltas at river mouths.

5. **Drainage basins:** The entire map naturally divides into drainage basins -- regions where all water flows to a common low point. Basin boundaries follow ridgelines. This is identical to real hydrology.

#### Enhancing Drainage Networks

To make drainage networks more pronounced for gameplay purposes (rivers are important for city building), you can bias particle spawning:

```rust
// Instead of spawning particles uniformly, spawn more particles at higher elevations
// This concentrates erosion along the natural flow paths
fn spawn_position_biased(
    heightmap: &[f32],
    width: usize,
    height: usize,
    rng: &mut impl Rng,
) -> (f32, f32) {
    loop {
        let x = rng.gen_range(1.0..(width - 2) as f32);
        let y = rng.gen_range(1.0..(height - 2) as f32);
        let h = heightmap[y as usize * width + x as usize];

        // Accept with probability proportional to elevation^2
        // High points get more particles = stronger drainage carving
        if rng.gen::<f32>() < h * h {
            return (x, y);
        }
    }
}
```

#### Flow Accumulation Map

After erosion, you can compute a flow accumulation map to identify where rivers would form. This is useful for water body placement (Section 3):

```rust
fn compute_flow_accumulation(
    heightmap: &[f32],
    width: usize,
    height: usize,
) -> Vec<u32> {
    let total = width * height;
    let mut flow = vec![1u32; total];  // Each cell starts with 1 unit of "rain"

    // Sort cells by elevation (highest first)
    let mut indices: Vec<usize> = (0..total).collect();
    indices.sort_by(|&a, &b|
        heightmap[b].partial_cmp(&heightmap[a]).unwrap()
    );

    // Process cells from highest to lowest
    for &idx in &indices {
        let x = idx % width;
        let y = idx / width;

        // Find lowest neighbor (steepest descent)
        let mut lowest_idx = idx;
        let mut lowest_h = heightmap[idx];

        for &(dx, dy) in &[(-1i32, 0), (1, 0), (0, -1), (0, 1),
                            (-1, -1), (1, -1), (-1, 1), (1, 1)] {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                let ni = ny as usize * width + nx as usize;
                if heightmap[ni] < lowest_h {
                    lowest_h = heightmap[ni];
                    lowest_idx = ni;
                }
            }
        }

        // Flow to lowest neighbor
        if lowest_idx != idx {
            flow[lowest_idx] += flow[idx];
        }
    }

    flow
}

// Cells with flow accumulation > threshold are river cells
// Typical threshold for 256x256: flow > 50-100
```

### 2.3 Thermal Erosion

Hydraulic erosion creates water-carved features. **Thermal erosion** (also called weathering or talus erosion) simulates the breakdown of rock on steep slopes. When a slope exceeds the "angle of repose" (the steepest angle at which loose material can rest), material slides downhill.

This creates:
- Smooth, uniform slopes at the angle of repose
- Talus (scree) deposits at the base of cliffs
- Rounded ridgelines and hilltops
- More realistic cliff faces (stepped rather than razor-sharp)

#### Thermal Erosion Algorithm

```rust
fn thermal_erosion(
    heightmap: &mut [f32],
    width: usize,
    height: usize,
    iterations: u32,
    talus_angle: f32,   // Maximum stable slope (in height units per cell)
    erosion_rate: f32,   // Fraction of excess material moved per iteration (0.0-0.5)
) {
    // talus_angle is the maximum height difference between adjacent cells
    // that is considered stable. For our 16m cells:
    // talus_angle = 0.05 means max stable slope = 0.05/1 * (1/16m) = 0.3% grade
    // In practice, use values like 0.02-0.08 for the normalized [0,1] heightmap

    for _ in 0..iterations {
        // Process each cell
        for y in 1..(height - 1) {
            for x in 1..(width - 1) {
                let idx = y * width + x;
                let h = heightmap[idx];

                // Find the maximum height difference to any neighbor
                let mut max_diff = 0.0f32;
                let mut total_diff = 0.0f32;
                let mut diff_count = 0u32;

                let neighbors = [
                    (x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1),
                    (x - 1, y - 1), (x + 1, y - 1), (x - 1, y + 1), (x + 1, y + 1),
                ];

                let mut diffs: [(usize, f32); 8] = [(0, 0.0); 8];
                let mut n = 0;

                for &(nx, ny) in &neighbors {
                    let ni = ny * width + nx;
                    let diff = h - heightmap[ni];
                    // For diagonal neighbors, divide by sqrt(2) to get true slope
                    let is_diagonal = (nx != x) && (ny != y);
                    let effective_diff = if is_diagonal { diff / 1.414 } else { diff };

                    if effective_diff > talus_angle {
                        diffs[n] = (ni, effective_diff - talus_angle);
                        total_diff += effective_diff - talus_angle;
                        max_diff = max_diff.max(effective_diff);
                        n += 1;
                    }
                }

                // Distribute excess material to lower neighbors proportionally
                if n > 0 && total_diff > 0.0 {
                    let move_amount = (max_diff - talus_angle) * erosion_rate * 0.5;

                    for i in 0..n {
                        let (ni, diff) = diffs[i];
                        let fraction = diff / total_diff;
                        let transfer = move_amount * fraction;
                        heightmap[idx] -= transfer;
                        heightmap[ni] += transfer;
                    }
                }
            }
        }
    }
}
```

**Recommended parameters for city terrain:**
- `talus_angle = 0.04` (roughly a 40m height difference over 1km horizontal, or ~2.3-degree slope for instability)
- `erosion_rate = 0.4`
- `iterations = 20-40`

Thermal erosion is much cheaper per iteration than hydraulic erosion, so more iterations are practical.

### 2.4 Iteration Counts and Performance

#### How Many Particles Are Needed?

The right number of particles depends on map size and desired erosion depth. Rules of thumb:

| Map Size    | Cells  | Light Erosion | Medium Erosion | Heavy Erosion |
|-------------|--------|---------------|----------------|---------------|
| 128x128     | 16K    | 30K particles | 60K particles  | 120K particles|
| 256x256     | 64K    | 80K particles | 150K particles | 300K particles|
| 512x512     | 256K   | 200K particles| 500K particles | 1M particles  |
| 1024x1024   | 1M     | 500K particles| 1.5M particles | 4M particles  |

**For our 256x256 grid:** 100K-200K particles produces good results. This takes approximately 50-200ms on modern hardware (single-threaded), which is fast enough to run at map generation time without a loading screen.

The ratio of about 1.5-3 particles per grid cell is a good guideline. Below 1:1, erosion is spotty and inconsistent. Above 5:1, additional particles provide diminishing returns.

#### Performance Optimization Strategies

**1. Batch processing:** Process particles in batches of 1000-10000. Between batches, the heightmap reflects all previous erosion, so later particles flow into already-carved channels (desirable).

**2. Early termination:** Particles that reach the map edge, enter deep water, or slow to near-zero speed should terminate early. The `max_lifetime = 100` parameter ensures no particle runs forever.

**3. Precomputed brush:** The erosion brush (offsets and weights) is computed once and reused for all particles. For radius=3, this is a 37-element brush.

**4. Parallelism considerations:** Particle simulation is inherently sequential within each particle (each step depends on the previous step). However, multiple particles can be simulated in parallel if they operate on separate regions of the heightmap. A practical approach:
- Divide the map into 4 quadrants
- Run 4 threads, each spawning particles only in its quadrant
- Use atomic operations for cells near quadrant boundaries
- This gives roughly 3x speedup (not 4x due to boundary contention)

Alternatively, process particles in parallel with a read-only heightmap snapshot, collecting all erosion deltas, then apply deltas in a single pass. This is less physically accurate (particles do not see each other's erosion within a batch) but highly parallelizable.

**5. No-allocation loop:** The inner particle loop should allocate nothing. All state is in the particle struct and the heightmap array. The brush is precomputed. This makes the hot loop cache-friendly.

#### Performance Budget for Megacity

Map generation is a one-time cost (when starting a new game or loading a seed). Target budget:

```
Step                    | Time (256x256)  | Notes
------------------------+-----------------+----------------------------
Noise generation (fBm)  | 2-5ms           | fastnoise-lite is fast
Domain warping          | 2-5ms           | Same noise evaluation cost
Height remapping        | <1ms            | Simple per-cell math
Hydraulic erosion 150K  | 80-150ms        | Main cost, single-threaded
Thermal erosion 30 iter | 10-20ms         | Cheap per iteration
Flow accumulation       | 5-10ms          | Sort + single pass
Water body detection    | 2-5ms           | Flood fill
Resource placement      | <1ms            | Simple per-cell logic
Total                   | ~120-200ms      | Well under 1 second
```

This is fast enough to generate terrain interactively when the player clicks "New Game" and adjusts sliders.

---

## 3. Water Body Generation

Water is one of the most important terrain features for a city builder. Rivers provide water supply, sewage disposal, shipping routes, and aesthetic value. Lakes create natural landmarks and recreation areas. Coastlines define the boundary of buildable land and enable ports. The existing codebase uses a simple threshold (`elevation < 0.35 = CellType::Water`), which creates blob-shaped water bodies with no flow direction, no rivers, and no distinction between lakes, rivers, and oceans.

### 3.1 River Placement

Rivers should follow physically plausible paths: starting at high elevations, flowing downhill, and merging into larger rivers that eventually reach the sea or a terminal lake.

#### Method 1: Flow-Based Rivers (From Erosion Data)

If hydraulic erosion has already been run (Section 2), the flow accumulation map provides a natural river network:

```rust
fn extract_rivers_from_flow(
    flow_accumulation: &[u32],
    heightmap: &[f32],
    width: usize,
    height: usize,
    river_threshold: u32,   // Cells with flow > this are river cells
) -> Vec<Vec<(usize, usize)>> {
    // Mark all cells above threshold as river candidates
    let mut is_river = vec![false; width * height];
    for (i, &flow) in flow_accumulation.iter().enumerate() {
        if flow >= river_threshold {
            is_river[i] = true;
        }
    }

    // Trace connected river segments from headwaters to mouths
    // Find headwater cells (river cells whose uphill neighbors are not river cells)
    let mut headwaters: Vec<usize> = Vec::new();
    for y in 1..(height - 1) {
        for x in 1..(width - 1) {
            let idx = y * width + x;
            if !is_river[idx] { continue; }

            // Check if any higher neighbor is also a river cell
            let has_upstream = [(-1i32, 0), (1, 0), (0, -1), (0, 1)]
                .iter()
                .any(|&(dx, dy)| {
                    let nx = (x as i32 + dx) as usize;
                    let ny = (y as i32 + dy) as usize;
                    let ni = ny * width + nx;
                    is_river[ni] && heightmap[ni] > heightmap[idx]
                });

            if !has_upstream {
                headwaters.push(idx);
            }
        }
    }

    // Trace each river downstream
    let mut rivers = Vec::new();
    let mut visited = vec![false; width * height];

    for &start in &headwaters {
        let mut path = Vec::new();
        let mut current = start;

        loop {
            if visited[current] { break; }
            visited[current] = true;

            let x = current % width;
            let y = current / width;
            path.push((x, y));

            // Find lowest river neighbor (downstream)
            let mut next = None;
            let mut lowest_h = heightmap[current];

            for &(dx, dy) in &[(-1i32, 0), (1, 0), (0, -1), (0, 1)] {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx < 0 || nx >= width as i32 || ny < 0 || ny >= height as i32 {
                    continue;
                }
                let ni = ny as usize * width + nx as usize;
                if heightmap[ni] < lowest_h {
                    lowest_h = heightmap[ni];
                    next = Some(ni);
                }
            }

            match next {
                Some(ni) if is_river[ni] || heightmap[ni] < WATER_THRESHOLD => {
                    current = ni;
                }
                _ => break,  // Reached a depression or map edge
            }
        }

        if path.len() >= 5 {  // Minimum river length
            rivers.push(path);
        }
    }

    rivers
}
```

**River threshold for 256x256:** A threshold of 50-100 typically produces 3-8 significant rivers. Lower thresholds create denser creek networks; higher thresholds produce only the major rivers.

#### Method 2: Procedural River Carving (Without Erosion)

If erosion simulation is too expensive or not desired, rivers can be placed procedurally by selecting source points and tracing downhill paths:

```rust
fn carve_river(
    heightmap: &mut [f32],
    width: usize,
    height: usize,
    source_x: usize,
    source_y: usize,
    carve_depth: f32,     // How deep to carve the channel
    carve_width: f32,     // Width of the carved channel (in cells)
) -> Vec<(usize, usize)> {
    let mut path = Vec::new();
    let mut x = source_x as f32;
    let mut y = source_y as f32;
    let mut prev_dir = (0.0f32, 0.0f32);

    for _ in 0..1000 {  // max steps
        let ix = x as usize;
        let iy = y as usize;

        if ix < 1 || ix >= width - 1 || iy < 1 || iy >= height - 1 {
            break;
        }

        path.push((ix, iy));

        // Get gradient
        let (_, gx, gy) = height_and_gradient(heightmap, width, x, y);

        // Blend with previous direction for smooth curves (inertia)
        let inertia = 0.6;
        let dir_x = prev_dir.0 * inertia - gx * (1.0 - inertia);
        let dir_y = prev_dir.1 * inertia - gy * (1.0 - inertia);
        let len = (dir_x * dir_x + dir_y * dir_y).sqrt().max(0.001);

        prev_dir = (dir_x / len, dir_y / len);

        // Carve the channel at current position
        let carve_r = carve_width as i32;
        for dy in -carve_r..=carve_r {
            for dx in -carve_r..=carve_r {
                let dist = ((dx * dx + dy * dy) as f32).sqrt();
                if dist <= carve_width {
                    let nx = ix as i32 + dx;
                    let ny = iy as i32 + dy;
                    if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                        let ni = ny as usize * width + nx as usize;
                        let falloff = 1.0 - dist / carve_width;
                        heightmap[ni] -= carve_depth * falloff;
                    }
                }
            }
        }

        // Move
        x += prev_dir.0;
        y += prev_dir.1;

        // Stop at water level
        if heightmap[iy * width + ix] < WATER_THRESHOLD * 0.8 {
            break;
        }
    }

    path
}
```

### 3.2 Lake Detection

Lakes form in terrain depressions -- areas where water cannot flow out to the map edge or to a lower basin. Detecting these depressions is equivalent to finding closed basins in the heightmap.

#### Depression Filling (Pour Point Detection)

The standard algorithm for hydrological depression filling is the Planchon-Darboux algorithm or the more efficient Priority-Flood algorithm by Barnes et al.:

```rust
use std::collections::BinaryHeap;
use std::cmp::Reverse;

/// Priority-Flood algorithm: fills depressions to find the "filled" heightmap
/// where all cells can drain to the edge. The difference between the filled
/// heightmap and the original heightmap gives lake depths.
fn priority_flood_fill(
    heightmap: &[f32],
    width: usize,
    height: usize,
) -> Vec<f32> {
    let total = width * height;
    let mut filled = vec![f32::MAX; total];

    // Priority queue: (height, index) -- process lowest cells first
    // Reverse for min-heap behavior
    let mut queue: BinaryHeap<Reverse<(OrderedFloat<f32>, usize)>> = BinaryHeap::new();

    // Initialize: edge cells are known outlets (water can flow off the map)
    for x in 0..width {
        for &y in &[0, height - 1] {
            let idx = y * width + x;
            filled[idx] = heightmap[idx];
            queue.push(Reverse((OrderedFloat(heightmap[idx]), idx)));
        }
    }
    for y in 1..(height - 1) {
        for &x in &[0, width - 1] {
            let idx = y * width + x;
            filled[idx] = heightmap[idx];
            queue.push(Reverse((OrderedFloat(heightmap[idx]), idx)));
        }
    }

    // Process cells from lowest to highest
    while let Some(Reverse((OrderedFloat(h), idx))) = queue.pop() {
        let x = idx % width;
        let y = idx / width;

        for &(dx, dy) in &[(-1i32, 0), (1, 0), (0, -1), (0, 1)] {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx < 0 || nx >= width as i32 || ny < 0 || ny >= height as i32 {
                continue;
            }
            let ni = ny as usize * width + nx as usize;

            if filled[ni] == f32::MAX {
                // This neighbor hasn't been processed yet
                // Its water level is max(its own height, the current cell's water level)
                filled[ni] = heightmap[ni].max(h);
                queue.push(Reverse((OrderedFloat(filled[ni]), ni)));
            }
        }
    }

    filled
}

/// Extract lake cells: where filled height > original height
fn detect_lakes(
    heightmap: &[f32],
    filled: &[f32],
    width: usize,
    height: usize,
    min_depth: f32,  // Minimum depth to count as a lake (filters tiny puddles)
) -> Vec<bool> {
    let total = width * height;
    let mut is_lake = vec![false; total];

    for i in 0..total {
        let depth = filled[i] - heightmap[i];
        if depth > min_depth {
            is_lake[i] = true;
        }
    }

    is_lake
}
```

**Pour points** are the cells on a depression's rim where water would overflow. These are the natural locations for river outlets from lakes:

```rust
fn find_pour_points(
    heightmap: &[f32],
    filled: &[f32],
    is_lake: &[bool],
    width: usize,
    height: usize,
) -> Vec<(usize, usize)> {
    let mut pour_points = Vec::new();

    for y in 1..(height - 1) {
        for x in 1..(width - 1) {
            let idx = y * width + x;
            if !is_lake[idx] { continue; }

            // A pour point is a lake cell adjacent to a non-lake cell
            // at the filled water level
            for &(dx, dy) in &[(-1i32, 0), (1, 0), (0, -1), (0, 1)] {
                let nx = (x as i32 + dx) as usize;
                let ny = (y as i32 + dy) as usize;
                let ni = ny * width + nx;

                if !is_lake[ni] && (filled[idx] - heightmap[ni]).abs() < 0.001 {
                    pour_points.push((x, y));
                    break;
                }
            }
        }
    }

    pour_points
}
```

### 3.3 Ocean and Coastline

For maps that include ocean, the simplest approach is the existing threshold method, but enhanced with proper coastline processing.

#### Sea Level and Beach Zones

```rust
const SEA_LEVEL: f32 = 0.35;        // Same as current WATER_THRESHOLD
const BEACH_UPPER: f32 = 0.38;      // Upper edge of beach zone
const SHALLOW_WATER: f32 = 0.30;    // Shallow water (lighter blue, wade-able)
const DEEP_WATER: f32 = 0.15;       // Deep water (dark blue, needs bridges)

fn classify_water_depth(elevation: f32) -> WaterDepth {
    if elevation >= SEA_LEVEL {
        WaterDepth::NotWater
    } else if elevation >= SHALLOW_WATER {
        WaterDepth::Shallow    // Passable by foot (slowly), no boats
    } else if elevation >= DEEP_WATER {
        WaterDepth::Medium     // Boats only, needs bridges for crossing
    } else {
        WaterDepth::Deep       // Open water, large ships
    }
}

fn classify_shore(elevation: f32, has_water_neighbor: bool) -> ShoreType {
    if elevation < SEA_LEVEL {
        ShoreType::NotShore
    } else if elevation < BEACH_UPPER && has_water_neighbor {
        ShoreType::Beach       // Sandy beach zone
    } else if has_water_neighbor {
        ShoreType::Cliff       // Elevated land meeting water
    } else {
        ShoreType::NotShore
    }
}
```

#### Coastline Smoothing

Raw threshold-based coastlines create jagged staircase edges because the grid is rectilinear. Two techniques improve this:

**1. Marching squares for smooth coastline rendering (visual only):**
```
For each 2x2 cell block, determine which corners are above/below sea level.
This gives 16 possible configurations. For each configuration, place an
isoline segment that smoothly traces the water boundary. Use this for
rendering the coastline mesh, even though the simulation still uses
per-cell water/land classification.
```

**2. Coastline erosion pass (modifies heightmap):**
```rust
fn smooth_coastline(heightmap: &mut [f32], width: usize, height: usize, iterations: u32) {
    for _ in 0..iterations {
        let snapshot = heightmap.to_vec();
        for y in 1..(height - 1) {
            for x in 1..(width - 1) {
                let idx = y * width + x;
                let h = snapshot[idx];

                // Only smooth cells near the coastline
                if (h - SEA_LEVEL).abs() > 0.05 { continue; }

                // Average with neighbors (Gaussian blur restricted to shore zone)
                let mut sum = h * 4.0;  // Center weight
                let mut count = 4.0;
                for &(dx, dy) in &[(-1i32, 0), (1, 0), (0, -1), (0, 1)] {
                    let ni = (y as i32 + dy) as usize * width + (x as i32 + dx) as usize;
                    sum += snapshot[ni];
                    count += 1.0;
                }
                heightmap[idx] = sum / count;
            }
        }
    }
}
```

### 3.4 River Width and Meanders

In real geography, rivers widen downstream as tributaries add water volume. They also meander (form S-curves) on flat terrain.

#### Width Variation

River width should be proportional to flow accumulation (a proxy for water volume):

```rust
fn river_width_at_cell(flow: u32, min_width: f32, max_width: f32) -> f32 {
    // Logarithmic scaling: width grows slowly with flow
    // flow=50 (headwater) -> min_width (1 cell)
    // flow=5000 (major river) -> max_width (3-4 cells)
    let t = ((flow as f32).ln() - 50.0_f32.ln())
        / (5000.0_f32.ln() - 50.0_f32.ln());
    let t = t.clamp(0.0, 1.0);
    min_width + t * (max_width - min_width)
}

// For our 256x256 grid (16m cells):
// min_width = 0.5 cells (8m creek)
// max_width = 3.0 cells (48m major river)
// Real-world reference: the Thames through London is ~250m wide,
// but our map is only 4km wide, so 48m is proportionally appropriate
```

#### Meander Simulation

Real river meanders form because water flows faster on the outside of a curve (eroding the bank) and slower on the inside (depositing sediment). Simulating this precisely is expensive, but an approximation works well:

```rust
fn add_meanders(
    river_path: &[(usize, usize)],
    heightmap: &[f32],
    width: usize,
    amplitude: f32,    // Max meander offset in cells
    wavelength: f32,   // Meander wavelength in cells
) -> Vec<(f32, f32)> {
    // Convert grid path to smooth centerline using Catmull-Rom spline
    let smooth = catmull_rom_interpolate(river_path, 4);  // 4 subdivisions per segment

    let mut meandered = Vec::with_capacity(smooth.len());
    let total_length = path_length(&smooth);

    for (i, &(x, y)) in smooth.iter().enumerate() {
        // Calculate tangent direction at this point
        let prev = if i > 0 { smooth[i - 1] } else { smooth[i] };
        let next = if i + 1 < smooth.len() { smooth[i + 1] } else { smooth[i] };
        let tx = next.0 - prev.0;
        let ty = next.1 - prev.1;
        let tlen = (tx * tx + ty * ty).sqrt().max(0.001);

        // Normal direction (perpendicular to tangent)
        let nx = -ty / tlen;
        let ny = tx / tlen;

        // Sinusoidal offset along normal
        let t = arc_length_at(i, &smooth) / total_length;
        let offset = (t * total_length / wavelength * std::f32::consts::TAU).sin()
            * amplitude;

        // Reduce meander amplitude on steep terrain (rivers meander on flat terrain)
        let slope = local_slope(heightmap, width, x as usize, y as usize);
        let slope_factor = (1.0 - slope * 20.0).clamp(0.0, 1.0);

        meandered.push((
            x + nx * offset * slope_factor,
            y + ny * offset * slope_factor,
        ));
    }

    meandered
}
```

**Meander parameters for our scale:**
- `amplitude = 3.0` cells (48m) -- enough for visible curves
- `wavelength = 20.0` cells (320m) -- produces 2-3 visible meander bends per river

### 3.5 Deltas and Estuaries

Where a river meets a body of standing water (ocean or lake), sediment deposition creates a delta or estuary.

#### Delta Formation

Deltas form when a river's carrying capacity drops sharply upon entering standing water. The sediment fans out in a branching pattern:

```rust
fn generate_delta(
    heightmap: &mut [f32],
    width: usize,
    height: usize,
    river_mouth: (usize, usize),
    river_direction: (f32, f32),  // Normalized direction of flow at mouth
    delta_size: f32,              // Radius in cells
    num_channels: u32,            // Number of distributary channels (3-7)
    rng: &mut impl Rng,
) {
    let (mx, my) = river_mouth;
    let (rdx, rdy) = river_direction;

    // Create distributary channels fanning out from the mouth
    let spread_angle = std::f32::consts::PI * 0.4;  // 72-degree fan
    let base_angle = rdy.atan2(rdx);

    for i in 0..num_channels {
        let angle = base_angle
            + spread_angle * (i as f32 / (num_channels - 1) as f32 - 0.5)
            + rng.gen_range(-0.1..0.1);  // Slight randomization

        let dx = angle.cos();
        let dy = angle.sin();

        // Trace each channel, depositing sediment (lowering nearby terrain
        // while carving the channel slightly)
        let channel_length = delta_size * rng.gen_range(0.6..1.0);
        let channel_width = 0.5 + rng.gen_range(0.0..0.5);

        for step in 0..(channel_length as u32) {
            let t = step as f32;
            let cx = mx as f32 + dx * t;
            let cy = my as f32 + dy * t;

            let ix = cx as usize;
            let iy = cy as usize;
            if ix >= width || iy >= height { break; }

            // Deposit sediment: raise the land slightly around the channel
            // This creates the characteristic delta "fingers" above water
            let deposit_radius = channel_width + t * 0.1;  // Widens slightly
            for ody in -(deposit_radius as i32)..=(deposit_radius as i32) {
                for odx in -(deposit_radius as i32)..=(deposit_radius as i32) {
                    let nx = ix as i32 + odx;
                    let ny = iy as i32 + ody;
                    if nx < 0 || nx >= width as i32 || ny < 0 || ny >= height as i32 {
                        continue;
                    }
                    let ni = ny as usize * width + nx as usize;
                    let dist = ((odx * odx + ody * ody) as f32).sqrt();
                    if dist <= deposit_radius {
                        let falloff = 1.0 - dist / deposit_radius;
                        let deposit = 0.02 * falloff * (1.0 - t / channel_length);
                        // Only deposit in water areas (building up the delta)
                        if heightmap[ni] < SEA_LEVEL {
                            heightmap[ni] += deposit;
                        }
                    }
                }
            }
        }
    }
}
```

#### Estuary Formation

Estuaries are wider, funnel-shaped river mouths where tidal action prevents delta formation. For gameplay, an estuary is simpler: just widen the river near its mouth:

```rust
fn widen_river_near_mouth(
    river_path: &[(usize, usize)],
    water_width: &mut [f32],       // Width at each river cell
    mouth_index: usize,            // Index in path where river meets ocean
    estuary_length: usize,         // How far upstream the widening extends
    max_extra_width: f32,          // Maximum additional width at mouth
) {
    for i in 0..estuary_length.min(mouth_index) {
        let path_idx = mouth_index - i;
        let t = 1.0 - (i as f32 / estuary_length as f32);  // 1.0 at mouth, 0.0 upstream
        let extra = max_extra_width * t * t;  // Quadratic widening
        water_width[path_idx] += extra;
    }
}
```

### 3.6 Mapping to CellType::Water

After all water body generation is complete, the results need to be written back to the `WorldGrid`:

```rust
fn apply_water_to_grid(
    grid: &mut WorldGrid,
    heightmap: &[f32],
    is_river: &[bool],
    is_lake: &[bool],
    river_width_map: &[f32],  // Width at each cell (0 if not a river)
) {
    for y in 0..grid.height {
        for x in 0..grid.width {
            let idx = y * grid.width + x;
            let cell = grid.get_mut(x, y);
            cell.elevation = heightmap[idx];

            let is_water = heightmap[idx] < WATER_THRESHOLD  // Ocean
                || is_lake[idx]                               // Lake
                || is_river[idx];                             // River centerline

            // For rivers, also mark nearby cells based on river width
            let near_river = if !is_river[idx] {
                // Check if this cell is within the river width of any river cell
                // (This is an O(1) check using the precomputed river width map)
                river_width_map[idx] > 0.0
            } else {
                true
            };

            if is_water || near_river {
                cell.cell_type = CellType::Water;
            }
        }
    }
}
```

**Extended CellType for water differentiation:**

The current `CellType::Water` does not distinguish between rivers, lakes, and ocean. For gameplay systems that need this distinction (ports only on ocean, water pumps best on rivers, fishing on lakes), consider extending the type:

```rust
// Option A: Extended enum
enum CellType {
    Grass,
    Water,       // Keep for backward compat, means "ocean"
    River,       // Flowing water -- direction matters
    Lake,        // Still water in a depression
    Road,
}

// Option B: Separate water metadata (preserves existing code better)
struct Cell {
    // ... existing fields ...
    water_type: WaterType,  // None, Ocean, River, Lake
    flow_direction: Option<(f32, f32)>,  // For rivers: which way water flows
    water_depth: f32,       // 0 = shore, 1.0 = deepest
}
```

Option B is recommended because it avoids touching the many match statements on `CellType` throughout the codebase.

---

## 4. Biome and Vegetation

Biomes determine the visual character of terrain and have significant gameplay implications: vegetation density affects land value and happiness, temperature affects heating costs, moisture affects agriculture and fire risk.

### 4.1 Temperature and Moisture Mapping

Real-world biome classification depends primarily on two variables: mean annual temperature and mean annual precipitation. For a game, we can derive these from terrain features.

#### Temperature Model

Temperature in the real world depends on latitude and elevation. Our 4km map is too small for latitude to vary meaningfully (at most ~0.04 degrees), so we use elevation as the primary driver, with a configurable base temperature determined by the biome preset.

```rust
/// Environmental lapse rate: temperature decreases ~6.5C per 1000m of elevation gain
/// Our elevation range [0, 1] maps to a configurable real-world height range.
const LAPSE_RATE: f32 = 6.5;  // Celsius per 1000m

struct ClimateParams {
    base_temperature: f32,     // Temperature at sea level (C)
    max_elevation_meters: f32, // What elevation=1.0 corresponds to in meters
    base_moisture: f32,        // Base precipitation (0-1 scale)
    moisture_from_water: f32,  // How much nearby water increases moisture
    wind_direction: (f32, f32),// Prevailing wind direction (affects rain shadow)
}

// Presets
impl ClimateParams {
    fn temperate() -> Self {
        Self {
            base_temperature: 15.0,      // Average ~15C (London, Seattle, Tokyo)
            max_elevation_meters: 800.0,  // Hills up to 800m
            base_moisture: 0.6,
            moisture_from_water: 0.3,
            wind_direction: (1.0, 0.0),   // Westerly winds
        }
    }

    fn arid() -> Self {
        Self {
            base_temperature: 28.0,
            max_elevation_meters: 1200.0,
            base_moisture: 0.15,
            moisture_from_water: 0.4,     // Water matters more in dry climates
            wind_direction: (0.0, 1.0),
        }
    }

    fn tropical() -> Self {
        Self {
            base_temperature: 27.0,
            max_elevation_meters: 600.0,
            base_moisture: 0.85,
            moisture_from_water: 0.15,
            wind_direction: (-1.0, 0.5),
        }
    }

    fn boreal() -> Self {
        Self {
            base_temperature: 2.0,
            max_elevation_meters: 1500.0,
            base_moisture: 0.5,
            moisture_from_water: 0.2,
            wind_direction: (0.5, -1.0),
        }
    }
}

fn compute_temperature(elevation: f32, params: &ClimateParams) -> f32 {
    let height_meters = elevation * params.max_elevation_meters;
    params.base_temperature - (height_meters / 1000.0) * LAPSE_RATE
}
```

#### Moisture Model

Moisture depends on proximity to water bodies, prevailing wind direction (rain shadow effect), and elevation:

```rust
fn compute_moisture_grid(
    heightmap: &[f32],
    water_mask: &[bool],   // true for water cells
    width: usize,
    height: usize,
    params: &ClimateParams,
) -> Vec<f32> {
    let total = width * height;
    let mut moisture = vec![params.base_moisture; total];

    // Step 1: Distance-to-water contribution
    // Compute distance transform from water cells
    let water_dist = distance_transform(water_mask, width, height);

    for i in 0..total {
        // Cells near water get more moisture (exponential falloff)
        let dist = water_dist[i];
        let water_bonus = params.moisture_from_water * (-dist / 30.0).exp();
        // 30.0 = decay distance in cells (480m); moisture halves every ~20 cells
        moisture[i] += water_bonus;
    }

    // Step 2: Rain shadow effect
    // Simulate wind carrying moisture, losing it when hitting high terrain
    // Sweep in wind direction: cells downwind of mountains get less moisture
    let (wdx, wdy) = params.wind_direction;
    let wind_len = (wdx * wdx + wdy * wdy).sqrt();
    let (nwx, nwy) = (wdx / wind_len, wdy / wind_len);

    // Process columns along wind direction
    // Simplified: for each cell, trace upwind and check for high terrain
    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            let mut shadow = 0.0f32;

            // Trace upwind for 40 cells
            for step in 1..40 {
                let ux = x as f32 - nwx * step as f32;
                let uy = y as f32 - nwy * step as f32;
                let uix = ux as i32;
                let uiy = uy as i32;

                if uix < 0 || uix >= width as i32 || uiy < 0 || uiy >= height as i32 {
                    break;
                }

                let upwind_elev = heightmap[uiy as usize * width + uix as usize];
                let our_elev = heightmap[idx];

                // If upwind terrain is significantly higher, it blocks moisture
                if upwind_elev > our_elev + 0.05 {
                    let blocking = (upwind_elev - our_elev - 0.05) * 2.0;
                    shadow = shadow.max(blocking * (1.0 - step as f32 / 40.0));
                }
            }

            moisture[idx] = (moisture[idx] - shadow).clamp(0.0, 1.0);
        }
    }

    // Step 3: Elevation effect (higher = less moisture, in general)
    for i in 0..total {
        let elev_factor = 1.0 - heightmap[i] * 0.3;  // 30% reduction at max elevation
        moisture[i] *= elev_factor;
        moisture[i] = moisture[i].clamp(0.0, 1.0);
    }

    moisture
}

/// Manhattan distance transform -- fast approximation of nearest water distance
fn distance_transform(water_mask: &[bool], width: usize, height: usize) -> Vec<f32> {
    let total = width * height;
    let mut dist = vec![f32::MAX; total];

    // Initialize water cells with distance 0
    for i in 0..total {
        if water_mask[i] {
            dist[i] = 0.0;
        }
    }

    // Forward pass (top-left to bottom-right)
    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            if x > 0 { dist[idx] = dist[idx].min(dist[idx - 1] + 1.0); }
            if y > 0 { dist[idx] = dist[idx].min(dist[(y-1) * width + x] + 1.0); }
        }
    }

    // Backward pass (bottom-right to top-left)
    for y in (0..height).rev() {
        for x in (0..width).rev() {
            let idx = y * width + x;
            if x + 1 < width { dist[idx] = dist[idx].min(dist[idx + 1] + 1.0); }
            if y + 1 < height { dist[idx] = dist[idx].min(dist[(y+1) * width + x] + 1.0); }
        }
    }

    dist
}
```

### 4.2 Whittaker Diagram Classification

The Whittaker biome diagram classifies biomes based on mean annual temperature (x-axis) and mean annual precipitation/moisture (y-axis). We can implement this as a lookup function:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum Biome {
    // Cold biomes
    Tundra,             // Cold + low moisture
    BorealForest,       // Cold + moderate moisture (taiga)
    // Temperate biomes
    TemperateGrassland, // Moderate temp + low moisture (steppe, prairie)
    TemperateForest,    // Moderate temp + moderate moisture (deciduous)
    TemperateRainforest,// Moderate temp + high moisture (Pacific NW)
    // Warm biomes
    Desert,             // Hot + very low moisture
    Savanna,            // Hot + low-moderate moisture
    TropicalDryForest,  // Hot + moderate moisture
    TropicalRainforest, // Hot + high moisture
    // Special
    Alpine,             // Very high elevation (above treeline)
    Wetland,            // Any temp + very high moisture + low elevation
}

fn classify_biome(temperature: f32, moisture: f32, elevation: f32) -> Biome {
    // Alpine override: above treeline regardless of temp/moisture
    // Treeline is roughly where temp < -5C year-round
    if temperature < -5.0 {
        return Biome::Tundra;
    }
    if elevation > 0.8 && temperature < 5.0 {
        return Biome::Alpine;
    }

    // Wetland override: low-lying, very wet areas
    if elevation < 0.40 && moisture > 0.8 {
        return Biome::Wetland;
    }

    // Main Whittaker classification
    match (temperature, moisture) {
        (t, m) if t < 3.0 && m < 0.3 => Biome::Tundra,
        (t, _) if t < 3.0 => Biome::BorealForest,

        (t, m) if t < 15.0 && m < 0.3 => Biome::TemperateGrassland,
        (t, m) if t < 15.0 && m < 0.7 => Biome::TemperateForest,
        (t, _) if t < 15.0 => Biome::TemperateRainforest,

        (_, m) if m < 0.15 => Biome::Desert,
        (_, m) if m < 0.35 => Biome::Savanna,
        (_, m) if m < 0.65 => Biome::TropicalDryForest,
        _ => Biome::TropicalRainforest,
    }
}
```

#### Biome Transition Blending

Sharp biome boundaries look unnatural. Use a blending zone where two biomes overlap:

```rust
fn biome_blend(
    temperature: f32,
    moisture: f32,
    elevation: f32,
    noise_value: f32,  // Per-cell noise for stochastic blending
) -> (Biome, Biome, f32) {
    // Returns (primary_biome, secondary_biome, blend_factor)
    // blend_factor: 0.0 = fully primary, 1.0 = fully secondary

    let primary = classify_biome(temperature, moisture, elevation);

    // Check nearby classification by perturbing temperature and moisture
    let perturb = noise_value * 3.0;  // +/- 3 degrees or moisture units
    let alt = classify_biome(temperature + perturb, moisture + perturb * 0.1, elevation);

    if alt == primary {
        (primary, primary, 0.0)
    } else {
        // Blend factor based on how close we are to the boundary
        // (This is a simplification; a proper implementation would
        //  compute distance-to-boundary in temp/moisture space)
        let blend = (noise_value.abs() * 2.0).clamp(0.0, 0.5);
        (primary, alt, blend)
    }
}
```

### 4.3 Vegetation Density

Each biome has a characteristic vegetation density and type distribution:

```rust
struct BiomeVegetation {
    tree_density: f32,          // 0-1, probability of tree per cell
    grass_density: f32,         // 0-1, grass coverage
    shrub_density: f32,         // 0-1, shrub coverage
    tree_types: &'static [TreeType],
    grass_color_base: (f32, f32, f32),  // Base grass RGB
}

#[derive(Debug, Clone, Copy)]
enum TreeType {
    Oak,
    Pine,
    Birch,
    Spruce,
    Palm,
    Cactus,
    Mangrove,
    Willow,
}

fn biome_vegetation(biome: Biome) -> BiomeVegetation {
    match biome {
        Biome::Tundra => BiomeVegetation {
            tree_density: 0.0,
            grass_density: 0.3,
            shrub_density: 0.1,
            tree_types: &[],
            grass_color_base: (0.45, 0.50, 0.35),  // Grey-green
        },
        Biome::BorealForest => BiomeVegetation {
            tree_density: 0.6,
            grass_density: 0.2,
            shrub_density: 0.15,
            tree_types: &[TreeType::Spruce, TreeType::Pine, TreeType::Birch],
            grass_color_base: (0.30, 0.42, 0.25),  // Dark green
        },
        Biome::TemperateGrassland => BiomeVegetation {
            tree_density: 0.05,
            grass_density: 0.9,
            shrub_density: 0.1,
            tree_types: &[TreeType::Oak],
            grass_color_base: (0.55, 0.62, 0.30),  // Golden-green
        },
        Biome::TemperateForest => BiomeVegetation {
            tree_density: 0.55,
            grass_density: 0.4,
            shrub_density: 0.2,
            tree_types: &[TreeType::Oak, TreeType::Birch, TreeType::Pine],
            grass_color_base: (0.28, 0.50, 0.20),  // Rich green
        },
        Biome::TemperateRainforest => BiomeVegetation {
            tree_density: 0.75,
            grass_density: 0.3,
            shrub_density: 0.4,
            tree_types: &[TreeType::Oak, TreeType::Willow, TreeType::Birch],
            grass_color_base: (0.18, 0.45, 0.15),  // Deep lush green
        },
        Biome::Desert => BiomeVegetation {
            tree_density: 0.01,
            grass_density: 0.05,
            shrub_density: 0.08,
            tree_types: &[TreeType::Cactus],
            grass_color_base: (0.72, 0.65, 0.45),  // Sandy tan
        },
        Biome::Savanna => BiomeVegetation {
            tree_density: 0.08,
            grass_density: 0.7,
            shrub_density: 0.15,
            tree_types: &[TreeType::Oak],  // Acacia-like
            grass_color_base: (0.62, 0.58, 0.32),  // Dry golden
        },
        Biome::TropicalDryForest => BiomeVegetation {
            tree_density: 0.45,
            grass_density: 0.3,
            shrub_density: 0.25,
            tree_types: &[TreeType::Palm, TreeType::Oak],
            grass_color_base: (0.35, 0.52, 0.22),
        },
        Biome::TropicalRainforest => BiomeVegetation {
            tree_density: 0.85,
            grass_density: 0.2,
            shrub_density: 0.5,
            tree_types: &[TreeType::Palm, TreeType::Mangrove],
            grass_color_base: (0.12, 0.42, 0.10),  // Dense jungle green
        },
        Biome::Alpine => BiomeVegetation {
            tree_density: 0.0,
            grass_density: 0.15,
            shrub_density: 0.05,
            tree_types: &[],
            grass_color_base: (0.50, 0.50, 0.42),  // Rocky grey-brown
        },
        Biome::Wetland => BiomeVegetation {
            tree_density: 0.15,
            grass_density: 0.6,
            shrub_density: 0.3,
            tree_types: &[TreeType::Willow, TreeType::Mangrove],
            grass_color_base: (0.25, 0.45, 0.28),  // Marshy green
        },
    }
}
```

#### Tree Placement Algorithm

Trees should not be placed in a regular grid (looks artificial) or purely randomly (creates unrealistic clumps and gaps). Poisson disk sampling produces a natural-looking distribution:

```rust
fn place_trees_poisson(
    width: usize,
    height: usize,
    density: f32,        // 0-1
    min_spacing: f32,    // Minimum distance between trees (in cells)
    heightmap: &[f32],
    water_mask: &[bool],
    rng: &mut impl Rng,
) -> Vec<(usize, usize, TreeType)> {
    let mut trees = Vec::new();
    let mut occupied = vec![false; width * height];

    // Number of attempts proportional to density
    let attempts = (width * height) as f32 * density * 3.0;

    for _ in 0..(attempts as u32) {
        let x = rng.gen_range(0..width);
        let y = rng.gen_range(0..height);
        let idx = y * width + x;

        // Skip water, steep slopes, already-occupied cells
        if water_mask[idx] { continue; }
        if occupied[idx] { continue; }

        // Check minimum spacing
        let r = min_spacing.ceil() as i32;
        let too_close = (-r..=r).any(|dy| {
            (-r..=r).any(|dx| {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx < 0 || nx >= width as i32 || ny < 0 || ny >= height as i32 {
                    return false;
                }
                let dist_sq = (dx * dx + dy * dy) as f32;
                if dist_sq > min_spacing * min_spacing { return false; }
                occupied[ny as usize * width + nx as usize]
            })
        });

        if too_close { continue; }

        // Elevation-based tree probability falloff
        // (fewer trees at very high and very low elevations)
        let elev = heightmap[idx];
        let elev_factor = if elev < 0.38 { (elev - 0.35) / 0.03 }  // Near water: sparse
                          else if elev > 0.75 { 1.0 - (elev - 0.75) / 0.25 }  // High: treeline
                          else { 1.0 };

        if rng.gen::<f32>() > density * elev_factor.clamp(0.0, 1.0) {
            continue;
        }

        occupied[idx] = true;
        trees.push((x, y, TreeType::Oak));  // Tree type selected by biome
    }

    trees
}
```

### 4.4 Starting Biome Selection

The player chooses a biome preset when starting a new game. This sets the climate parameters and affects every subsequent system:

```
Biome Preset      | Temp  | Moisture | Gameplay Effects
------------------+-------+----------+--------------------------------------------------
Temperate         | 15C   | 0.6      | Balanced. Moderate heating/cooling costs.
                  |       |          | Good agriculture. Deciduous trees.
                  |       |          | Four distinct seasons. Standard fire risk.
------------------+-------+----------+--------------------------------------------------
Arid/Desert       | 28C   | 0.15     | High cooling costs. Poor agriculture (needs
                  |       |          | irrigation). Very low tree density. High fire
                  |       |          | risk. Water supply is critical. Solar power bonus.
------------------+-------+----------+--------------------------------------------------
Tropical          | 27C   | 0.85     | No heating costs. Dense vegetation (clearing
                  |       |          | costs). Flooding risk. Disease risk higher.
                  |       |          | Tourism bonus. Fast tree regrowth.
------------------+-------+----------+--------------------------------------------------
Boreal/Subarctic  | 2C    | 0.5      | High heating costs. Short growing season.
                  |       |          | Conifer forests. Snow cover 6+ months.
                  |       |          | Low fire risk. High road maintenance (frost).
                  |       |          | Low population density preference.
------------------+-------+----------+--------------------------------------------------
```

**How biome affects terrain generation:**

The biome preset does not change the heightmap algorithm, but it changes:
1. Water threshold: arid maps have less water, tropical maps have more
2. Erosion intensity: arid terrain has more thermal erosion (exposed rock), tropical has more hydraulic erosion (rain carving)
3. Vegetation density: applied after terrain generation
4. Ground texture/color: the `terrain_color()` function in `terrain_render.rs` should use biome grass colors instead of hardcoded values
5. Resource distribution: fertile land is more common in temperate/tropical, ore is more common in arid/boreal (less vegetation covering deposits)

```rust
fn biome_terrain_adjustments(params: &ClimateParams) -> TerrainGenConfig {
    TerrainGenConfig {
        water_threshold: match params.base_moisture {
            m if m < 0.3 => 0.25,   // Less water in arid climates
            m if m > 0.7 => 0.40,   // More water in wet climates
            _ => 0.35,              // Standard
        },
        erosion_particles: match params.base_moisture {
            m if m < 0.3 => 80_000,  // Less rainfall = less erosion
            m if m > 0.7 => 200_000, // More rainfall = more erosion
            _ => 150_000,
        },
        thermal_erosion_iterations: match params.base_temperature {
            t if t > 25.0 => 50,     // Hot = more thermal weathering
            t if t < 5.0 => 40,      // Cold = freeze-thaw weathering
            _ => 25,
        },
        // ... additional parameters
    }
}
```

---

## 5. Resource Distribution

The existing `natural_resources.rs` uses a simple hash-based approach keyed on elevation bands. This produces uniformly scattered individual cells of resources, which does not create the clustered deposits that make resource extraction interesting for gameplay. Resources should form coherent regions that the player can discover, plan around, and exploit.

### 5.1 Ore and Mineral Deposits

Real ore deposits are geological features -- they form in specific contexts (volcanic intrusions, hydrothermal vents, sedimentary layers) and occur in localized clusters. For gameplay, we want 3-8 distinct ore zones per map, each containing 20-80 contiguous cells.

#### Clustered Poisson Disk Sampling

Place ore deposit centers using Poisson disk sampling (guarantees minimum spacing), then grow each center into a cluster:

```rust
fn generate_ore_deposits(
    heightmap: &[f32],
    width: usize,
    height: usize,
    resource_grid: &mut ResourceGrid,
    num_deposits: u32,        // Target number of deposits (3-8)
    min_spacing: f32,         // Minimum distance between deposit centers (30-50 cells)
    cluster_radius: f32,      // Radius of each deposit (5-12 cells)
    min_elevation: f32,       // Only place on elevated terrain (0.5+)
    rng: &mut impl Rng,
) {
    let mut centers: Vec<(usize, usize)> = Vec::new();

    // Phase 1: Place deposit centers using rejection sampling
    let max_attempts = num_deposits * 100;
    for _ in 0..max_attempts {
        if centers.len() >= num_deposits as usize { break; }

        let x = rng.gen_range(5..(width - 5));
        let y = rng.gen_range(5..(height - 5));

        // Must be on elevated terrain
        if heightmap[y * width + x] < min_elevation { continue; }

        // Must be far enough from existing deposits
        let too_close = centers.iter().any(|&(cx, cy)| {
            let dx = x as f32 - cx as f32;
            let dy = y as f32 - cy as f32;
            (dx * dx + dy * dy).sqrt() < min_spacing
        });
        if too_close { continue; }

        centers.push((x, y));
    }

    // Phase 2: Grow each center into a cluster
    for &(cx, cy) in &centers {
        let radius = cluster_radius * rng.gen_range(0.7..1.3);  // Vary radius
        let richness = rng.gen_range(3000..8000u32);

        // Use a noise function to create irregular cluster shape
        let cluster_noise_seed = rng.gen::<i32>();
        let mut cluster_noise = FastNoiseLite::with_seed(cluster_noise_seed);
        cluster_noise.set_noise_type(Some(NoiseType::OpenSimplex2));
        cluster_noise.set_frequency(Some(0.15));  // High frequency for ragged edges

        let r = radius.ceil() as i32;
        for dy in -r..=r {
            for dx in -r..=r {
                let nx = cx as i32 + dx;
                let ny = cy as i32 + dy;
                if nx < 0 || nx >= width as i32 || ny < 0 || ny >= height as i32 {
                    continue;
                }

                let dist = ((dx * dx + dy * dy) as f32).sqrt();
                if dist > radius { continue; }

                // Irregular shape using noise
                let noise_val = cluster_noise.get_noise_2d(nx as f32, ny as f32);
                let effective_radius = radius * (0.7 + noise_val * 0.3);
                if dist > effective_radius { continue; }

                // Skip water cells
                let idx = ny as usize * width + nx as usize;
                if heightmap[idx] < WATER_THRESHOLD { continue; }

                // Richness decreases toward edges (richest at center)
                let edge_factor = 1.0 - dist / radius;
                let cell_amount = (richness as f32 * edge_factor) as u32;

                resource_grid.set(nx as usize, ny as usize, ResourceDeposit {
                    resource_type: ResourceType::Ore,
                    amount: cell_amount,
                    max_amount: cell_amount,
                });
            }
        }
    }
}
```

**Design considerations:**
- 3-8 deposits per 256x256 map, with 30-50 cell minimum spacing, ensures deposits are spread across the map but not everywhere
- Cluster radius of 5-12 cells (80-192m) creates deposits large enough to build a mining district around
- Elevation requirement (> 0.5) ensures ore is found in hilly/mountainous areas, which creates interesting tradeoffs (harder to build roads to, but valuable resources)
- Irregular cluster shape via noise prevents perfectly circular deposits

### 5.2 Fertile Soil Zones

Fertile land concentrates in river valleys, flood plains, and low-lying areas with good drainage. Unlike ore (which is point-like clusters), fertile land forms broad contiguous regions.

```rust
fn generate_fertile_zones(
    heightmap: &[f32],
    moisture: &[f32],
    water_dist: &[f32],    // Distance to nearest water
    flow_accumulation: &[u32],
    width: usize,
    height: usize,
    resource_grid: &mut ResourceGrid,
) {
    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            let elev = heightmap[idx];

            // Skip water
            if elev < WATER_THRESHOLD { continue; }

            // Fertility score based on multiple factors
            let mut fertility = 0.0f32;

            // Factor 1: Low-lying land (valleys, plains)
            // Elevation 0.35-0.45 is ideal; decreases above that
            let elev_score = if elev < 0.45 {
                (elev - 0.35) / 0.10  // 0 at water's edge, 1.0 at ideal height
            } else {
                1.0 - (elev - 0.45) / 0.35  // Decreases linearly above 0.45
            };
            fertility += elev_score.clamp(0.0, 1.0) * 0.3;

            // Factor 2: Proximity to water (alluvial soil)
            let water_proximity = (1.0 - water_dist[idx] / 15.0).clamp(0.0, 1.0);
            fertility += water_proximity * 0.3;

            // Factor 3: Moisture level
            fertility += moisture[idx] * 0.2;

            // Factor 4: Flow accumulation (floodplain = nutrient-rich)
            let flow_score = (flow_accumulation[idx] as f32 / 100.0).clamp(0.0, 1.0);
            fertility += flow_score * 0.2;

            // Threshold: only mark as fertile if score is high enough
            if fertility > 0.55 {
                let amount = (fertility * 15000.0) as u32;
                resource_grid.set(x, y, ResourceDeposit {
                    resource_type: ResourceType::FertileLand,
                    amount,
                    max_amount: amount,
                });
            }
        }
    }
}
```

**Result:** Fertile land naturally concentrates along river banks, around lakes, and in low-lying plains -- exactly where real-world agriculture thrives. This creates a gameplay incentive to settle near water, which also provides water supply and transportation.

### 5.3 Oil and Gas Reserves

Oil and gas form in sedimentary basins -- broad, deep geological structures. For gameplay, oil should be:
- Rare (1-3 deposits per map)
- Located at medium-to-low elevations (sedimentary, not mountainous terrain)
- In broad, amorphous zones (larger than ore deposits)
- Not immediately visible (requires surveying to discover)

```rust
fn generate_oil_reserves(
    heightmap: &[f32],
    width: usize,
    height: usize,
    resource_grid: &mut ResourceGrid,
    seed: i32,
    rng: &mut impl Rng,
) {
    // Oil is placed using a separate low-frequency noise pass
    // This creates broad geological zones independent of surface terrain
    let mut oil_noise = FastNoiseLite::with_seed(seed + 5000);
    oil_noise.set_noise_type(Some(NoiseType::OpenSimplex2));
    oil_noise.set_frequency(Some(0.005));  // Very large features

    // Secondary noise for richness variation within zones
    let mut richness_noise = FastNoiseLite::with_seed(seed + 5001);
    richness_noise.set_noise_type(Some(NoiseType::OpenSimplex2));
    richness_noise.set_frequency(Some(0.02));

    let oil_threshold = 0.65;  // Top ~17% of noise values contain oil
    // Adjust threshold to control rarity

    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            let elev = heightmap[idx];

            // Oil only in medium-low elevation (sedimentary terrain)
            if elev < WATER_THRESHOLD || elev > 0.6 { continue; }

            // Don't place oil where fertile land already exists
            if resource_grid.get(x, y).is_some() { continue; }

            let oil_val = oil_noise.get_noise_2d(x as f32, y as f32);
            let normalized = (oil_val + 1.0) * 0.5;  // [0, 1]

            if normalized > oil_threshold {
                let richness = richness_noise.get_noise_2d(x as f32, y as f32);
                let amount = (2000.0 + richness * 2000.0).max(500.0) as u32;

                resource_grid.set(x, y, ResourceDeposit {
                    resource_type: ResourceType::Oil,
                    amount,
                    max_amount: amount,
                });
            }
        }
    }
}
```

### 5.4 Forest Density

Forests are the most visible resource. Unlike the point-based resource grid, forests should be represented as a density value per cell that affects tree rendering, lumber production, and fire fuel load.

```rust
/// Forest density grid: 0.0 = no trees, 1.0 = old-growth dense forest
#[derive(Resource)]
struct ForestDensity {
    density: Vec<f32>,
    width: usize,
    height: usize,
}

fn generate_forest_density(
    heightmap: &[f32],
    moisture: &[f32],
    biome_map: &[Biome],
    width: usize,
    height: usize,
    seed: i32,
) -> ForestDensity {
    let total = width * height;
    let mut density = vec![0.0f32; total];

    // Forest noise: creates patchy, natural-looking forest distribution
    let mut forest_noise = FastNoiseLite::with_seed(seed + 3000);
    forest_noise.set_noise_type(Some(NoiseType::OpenSimplex2));
    forest_noise.set_fractal_type(Some(FractalType::FBm));
    forest_noise.set_fractal_octaves(Some(4));
    forest_noise.set_frequency(Some(0.025));

    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;

            // Skip water
            if heightmap[idx] < WATER_THRESHOLD { continue; }

            // Base density from biome
            let biome_density = biome_vegetation(biome_map[idx]).tree_density;

            // Modulate with noise (patchy distribution)
            let noise_val = (forest_noise.get_noise_2d(x as f32, y as f32) + 1.0) * 0.5;
            let patchy = noise_val * noise_val;  // Square to create sparser patches with dense clusters

            // Moisture bonus (more trees near water)
            let moisture_factor = moisture[idx];

            // Slope penalty (fewer trees on steep slopes)
            let slope = compute_slope(heightmap, width, height, x, y);
            let slope_factor = (1.0 - slope * 5.0).clamp(0.0, 1.0);

            density[idx] = (biome_density * patchy * moisture_factor * slope_factor)
                .clamp(0.0, 1.0);
        }
    }

    // Old-growth regions: areas far from map edges and in deep forest get a bonus
    // This creates ancient forest cores that provide special resources
    let edge_dist = compute_edge_distance(width, height);
    for i in 0..total {
        if density[i] > 0.5 {
            let dist_factor = (edge_dist[i] as f32 / 30.0).clamp(0.0, 1.0);
            // Mark as old-growth if dense and far from edges
            density[i] = density[i] * (0.8 + dist_factor * 0.2);
        }
    }

    ForestDensity { density, width, height }
}

fn compute_slope(heightmap: &[f32], width: usize, height: usize, x: usize, y: usize) -> f32 {
    if x == 0 || x >= width - 1 || y == 0 || y >= height - 1 { return 0.0; }
    let idx = y * width + x;
    let dx = heightmap[idx + 1] - heightmap[idx - 1];
    let dy = heightmap[idx + width] - heightmap[idx - width];
    (dx * dx + dy * dy).sqrt()
}
```

**Forest density thresholds for gameplay:**

| Density    | Visual            | Resource Yield | Fire Risk | Land Value Impact |
|------------|-------------------|----------------|-----------|-------------------|
| 0.0 - 0.1 | Open grassland    | None           | Low       | Neutral           |
| 0.1 - 0.3 | Scattered trees   | Low            | Low       | +5% (scenic)      |
| 0.3 - 0.5 | Light woodland    | Medium         | Medium    | +10% (parkland)   |
| 0.5 - 0.7 | Forest            | High           | High      | +5% (proximity)   |
| 0.7 - 0.9 | Dense forest      | Very high      | Very high | -5% (dark, remote)|
| 0.9 - 1.0 | Old-growth        | Special lumber | Extreme   | +15% (rare nature)|

### 5.5 Resource Discovery Mechanics

Not all resources should be visible at game start. Geological resources (ore, oil) should require surveying to discover, creating an exploration dynamic.

#### Fog of Resources

```rust
#[derive(Resource)]
struct ResourceVisibility {
    discovered: Vec<bool>,
    width: usize,
    height: usize,
}

impl ResourceVisibility {
    fn new(width: usize, height: usize) -> Self {
        Self {
            discovered: vec![false; width * height],
            width,
            height,
        }
    }

    fn is_discovered(&self, x: usize, y: usize) -> bool {
        self.discovered[y * self.width + x]
    }

    /// Reveal resources in a radius around a survey point
    fn survey(&mut self, center_x: usize, center_y: usize, radius: usize) {
        let r = radius as i32;
        for dy in -r..=r {
            for dx in -r..=r {
                let nx = center_x as i32 + dx;
                let ny = center_y as i32 + dy;
                if nx < 0 || nx >= self.width as i32 || ny < 0 || ny >= self.height as i32 {
                    continue;
                }
                let dist = ((dx * dx + dy * dy) as f32).sqrt();
                if dist <= radius as f32 {
                    self.discovered[ny as usize * self.width + nx as usize] = true;
                }
            }
        }
    }
}
```

**Discovery mechanics:**
1. **Surface resources** (fertile land, forest) are always visible -- you can see them by looking at the terrain
2. **Subsurface resources** (ore, oil) start hidden and require a "Geological Survey" action
3. Surveying costs money ($5,000-$20,000 per survey) and reveals a 20-30 cell radius
4. The player can build a "Geological Survey Office" building that passively reveals resources in its service radius over time
5. Some hint of resources can be shown before surveying: slightly different terrain coloring, "Potential mineral deposits" tooltip on hover

```rust
// Hint system: show partial information before full survey
fn resource_hint(
    resource_grid: &ResourceGrid,
    visibility: &ResourceVisibility,
    x: usize,
    y: usize,
) -> ResourceHint {
    if visibility.is_discovered(x, y) {
        if let Some(deposit) = resource_grid.get(x, y) {
            ResourceHint::Known(deposit.resource_type, deposit.amount)
        } else {
            ResourceHint::KnownEmpty
        }
    } else {
        // Check if any neighbors in a 5-cell radius have resources
        // If so, show a vague hint
        let has_nearby = (-5i32..=5).any(|dy| {
            (-5i32..=5).any(|dx| {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx < 0 || ny < 0 { return false; }
                resource_grid.get(nx as usize, ny as usize).is_some()
            })
        });

        if has_nearby {
            ResourceHint::Suspected  // "Possible deposits in this area"
        } else {
            ResourceHint::Unknown
        }
    }
}
```

---

## 6. Starting Map Design

A city builder map must be fun to play. Random terrain generation can produce maps that are unplayable (all mountains, no water access, no flat land) or boring (completely flat). The generation pipeline must include validation and adjustment steps to ensure every generated map is both playable and interesting.

### 6.1 What Makes a Good Starting Location

Through analysis of successful city builder games, good starting maps share these properties:

**Essential (hard requirements):**
- At least 30% of the map is flat enough to build on (slope < 10% grade)
- At least one water body accessible from the flat area (for water supply)
- A contiguous flat region of at least 2,000 cells (for the initial city core)
- Water coverage between 10-35% of the map (not landlocked, not mostly ocean)

**Desirable (soft requirements that make maps interesting):**
- Elevation variation visible from the starting area (hills in the background)
- Multiple distinct flat areas connected by passable terrain (encourages multi-district cities)
- At least one river (for water supply diversity and aesthetic interest)
- At least 2 different resource types within 50 cells of the flat starting area
- A natural harbor or river bend for port placement
- A ridge or hill that creates natural district boundaries

**Anti-patterns (maps to reject and regenerate):**
- "Bowl" maps where the entire playable area is a single depression surrounded by impassable mountains
- "Pancake" maps where elevation varies by less than 5% across the entire map
- "Archipelago" maps where no single island is large enough for a viable city (unless specifically selected)
- Maps where water completely bisects the buildable area with no narrow crossing point

### 6.2 Playability Guarantees

After generating terrain, run a validation pass that rejects or adjusts maps that fail playability criteria:

```rust
struct MapQualityReport {
    total_land_cells: u32,
    total_water_cells: u32,
    flat_cells: u32,                    // Slope < 10% grade
    largest_flat_region: u32,           // Contiguous flat area (flood fill)
    water_adjacent_flat_cells: u32,     // Flat cells within 5 cells of water
    elevation_range: f32,               // Max elevation - min elevation
    num_water_bodies: u32,              // Distinct water regions
    num_rivers: u32,                    // Elongated water features
    resource_types_accessible: u32,     // Resource types within 50 cells of largest flat region
}

fn validate_map(heightmap: &[f32], width: usize, height: usize) -> MapQualityReport {
    let total = width * height;

    // Compute slope for every cell
    let mut slopes = vec![0.0f32; total];
    for y in 1..(height - 1) {
        for x in 1..(width - 1) {
            slopes[y * width + x] = compute_slope(heightmap, width, height, x, y);
        }
    }

    let flat_threshold = 0.015;  // ~10% grade in our normalized height space
    let water_threshold = WATER_THRESHOLD;

    let mut total_land = 0u32;
    let mut total_water = 0u32;
    let mut flat_cells = 0u32;

    for i in 0..total {
        if heightmap[i] < water_threshold {
            total_water += 1;
        } else {
            total_land += 1;
            if slopes[i] < flat_threshold {
                flat_cells += 1;
            }
        }
    }

    // Find largest contiguous flat region using flood fill
    let mut visited = vec![false; total];
    let mut largest_flat = 0u32;
    let mut largest_flat_cells: Vec<usize> = Vec::new();

    for start in 0..total {
        if visited[start] || heightmap[start] < water_threshold || slopes[start] >= flat_threshold {
            continue;
        }

        // Flood fill from this cell
        let mut region = Vec::new();
        let mut stack = vec![start];

        while let Some(idx) = stack.pop() {
            if visited[idx] { continue; }
            visited[idx] = true;

            if heightmap[idx] >= water_threshold && slopes[idx] < flat_threshold {
                region.push(idx);
                let x = idx % width;
                let y = idx / width;

                if x > 0 { stack.push(idx - 1); }
                if x + 1 < width { stack.push(idx + 1); }
                if y > 0 { stack.push(idx - width); }
                if y + 1 < height { stack.push(idx + width); }
            }
        }

        if region.len() as u32 > largest_flat {
            largest_flat = region.len() as u32;
            largest_flat_cells = region;
        }
    }

    // Count flat cells near water
    let water_dist = distance_transform(
        &(0..total).map(|i| heightmap[i] < water_threshold).collect::<Vec<_>>(),
        width, height
    );
    let water_adjacent = (0..total)
        .filter(|&i| heightmap[i] >= water_threshold
                && slopes[i] < flat_threshold
                && water_dist[i] <= 5.0)
        .count() as u32;

    let elev_min = heightmap.iter().copied().fold(f32::MAX, f32::min);
    let elev_max = heightmap.iter().copied().fold(f32::MIN, f32::max);

    MapQualityReport {
        total_land_cells: total_land,
        total_water_cells: total_water,
        flat_cells,
        largest_flat_region: largest_flat,
        water_adjacent_flat_cells: water_adjacent,
        elevation_range: elev_max - elev_min,
        num_water_bodies: count_water_bodies(heightmap, width, height),
        num_rivers: 0,  // Computed separately after river extraction
        resource_types_accessible: 0,  // Computed after resource placement
    }
}

fn is_map_playable(report: &MapQualityReport) -> bool {
    let total = report.total_land_cells + report.total_water_cells;
    let water_fraction = report.total_water_cells as f32 / total as f32;
    let flat_fraction = report.flat_cells as f32 / total as f32;

    // Hard requirements
    flat_fraction >= 0.30               // At least 30% flat
        && water_fraction >= 0.10       // At least 10% water
        && water_fraction <= 0.40       // No more than 40% water
        && report.largest_flat_region >= 2000  // Large enough starting area
        && report.water_adjacent_flat_cells >= 200  // Water-accessible flat land
        && report.elevation_range >= 0.15  // Some terrain variation
}
```

#### Rejection Sampling

The simplest approach: if a generated map fails validation, increment the seed and try again:

```rust
fn generate_valid_map(base_seed: i32, max_attempts: u32) -> (WorldGrid, i32) {
    for attempt in 0..max_attempts {
        let seed = base_seed + attempt as i32;
        let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
        generate_terrain(&mut grid, seed);  // Full pipeline

        let report = validate_map(
            &grid.cells.iter().map(|c| c.elevation).collect::<Vec<_>>(),
            GRID_WIDTH, GRID_HEIGHT
        );

        if is_map_playable(&report) {
            return (grid, seed);
        }
    }

    // Fallback: use the base seed with terrain adjustments
    let mut grid = WorldGrid::new(GRID_WIDTH, GRID_HEIGHT);
    generate_terrain(&mut grid, base_seed);
    force_playable(&mut grid);  // Modify terrain to meet minimum requirements
    (grid, base_seed)
}
```

In practice, with well-tuned noise parameters and the height distribution shaping from Section 1.4, the rejection rate is low (typically < 20% of seeds fail), so generating 2-3 maps to find a good one is fast.

#### Terrain Surgery (Force Playable)

If no valid seed is found within the attempt limit, modify the terrain:

```rust
fn force_playable(grid: &mut WorldGrid) {
    // Guarantee 1: Ensure there is water access
    // Find the center of the largest flat region and carve a river to it
    // if no water is within 20 cells

    // Guarantee 2: Flatten a 40x40 area near the center of the map
    // if no flat region exceeds 1000 cells
    let center_x = GRID_WIDTH / 2;
    let center_y = GRID_HEIGHT / 2;

    // Gaussian smoothing in a 40-cell radius around center
    let radius = 20;
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let x = (center_x as i32 + dx) as usize;
            let y = (center_y as i32 + dy) as usize;
            if x >= GRID_WIDTH || y >= GRID_HEIGHT { continue; }

            let dist = ((dx * dx + dy * dy) as f32).sqrt();
            if dist > radius as f32 { continue; }

            let blend = (1.0 - dist / radius as f32).powi(2);
            let target_elevation = 0.45;  // Flat, above water, not too high
            let cell = grid.get_mut(x, y);
            cell.elevation = cell.elevation * (1.0 - blend) + target_elevation * blend;

            // Reclassify water/land
            if cell.elevation < WATER_THRESHOLD {
                cell.cell_type = CellType::Water;
            } else if cell.cell_type == CellType::Water {
                cell.cell_type = CellType::Grass;
            }
        }
    }
}
```

### 6.3 Seed-Based Generation

Deterministic seed-based generation enables map sharing. Given the same seed, the same map is always produced. This is important for:
- Community map sharing ("try seed 42, great river spot!")
- Competitive scenarios (same map for all players)
- Bug reproduction
- Save/load (only need to store the seed, not the full terrain)

**Implementation requirements:**
1. All random number generation must use the seed. No `SystemTime`, no `thread_rng()`, no uninitialized memory.
2. The noise library must be deterministic. `fastnoise-lite` is deterministic by design (it uses a permutation table derived from the seed).
3. Erosion particle spawning must use a seeded PRNG (e.g., `rand::rngs::StdRng::seed_from_u64(seed as u64)`).
4. The order of operations must be fixed. Do not use `HashMap` iteration order for anything that affects terrain (it is nondeterministic in Rust by default due to random hash seeds; use `BTreeMap` or sorted iteration).

```rust
use rand::SeedableRng;
use rand::rngs::StdRng;

fn generate_terrain_full(grid: &mut WorldGrid, seed: i32) {
    let mut rng = StdRng::seed_from_u64(seed as u64);

    // 1. Base heightmap (noise is deterministic from seed)
    generate_heightmap_fbm(grid, seed);

    // 2. Domain warping (deterministic from seed + offset)
    apply_domain_warp(grid, seed);

    // 3. Height distribution shaping (pure math, no randomness)
    apply_height_remap(grid);

    // 4. Hydraulic erosion (uses seeded RNG)
    hydraulic_erosion(&mut grid_elevations(grid), GRID_WIDTH, GRID_HEIGHT,
                      150_000, &ErosionParams::default(), &mut rng);

    // 5. Thermal erosion (deterministic iteration)
    thermal_erosion(&mut grid_elevations(grid), GRID_WIDTH, GRID_HEIGHT,
                    30, 0.04, 0.4);

    // 6. Water body detection and river extraction (deterministic)
    detect_and_apply_water(grid);

    // 7. Resource placement (uses seeded RNG)
    place_resources(grid, seed as u32, &mut rng);

    // 8. Validation
    // (reject and regenerate if needed, using seed+1)
}
```

#### Seed Display and Sharing

```rust
// Display seed in the UI as a human-friendly code
fn seed_to_display(seed: i32) -> String {
    // Convert to alphanumeric for easy sharing
    // e.g., seed 1234567 -> "AX7-K2M"
    let words = ["Alpha", "Beta", "Delta", "Echo", "Fox", "Gulf", "Hotel", "Iris"];
    let w1 = words[(seed.abs() % 8) as usize];
    let num = (seed.abs() / 8) % 10000;
    format!("{}-{:04}", w1, num)
}
```

### 6.4 Landmark Templates

In addition to fully procedural generation, pre-designed landmark templates can be blended into the generated terrain to create recognizable features. Templates are small heightmap patches (32x32 or 64x64) that are stamped onto the procedural terrain.

#### Template Types

**Volcano:**
```rust
fn stamp_volcano(
    heightmap: &mut [f32],
    width: usize,
    center_x: usize,
    center_y: usize,
    radius: f32,       // 15-25 cells
    rim_height: f32,   // 0.85-0.95
    crater_depth: f32, // 0.1-0.2 below rim
) {
    let r = radius as i32;
    for dy in -r..=r {
        for dx in -r..=r {
            let nx = center_x as i32 + dx;
            let ny = center_y as i32 + dy;
            if nx < 0 || nx >= width as i32 || ny < 0 || ny >= (heightmap.len() / width) as i32 {
                continue;
            }
            let dist = ((dx * dx + dy * dy) as f32).sqrt();
            if dist > radius * 1.5 { continue; }

            let idx = ny as usize * width + nx as usize;
            let t = dist / radius;

            let volcano_h = if t < 0.3 {
                // Crater: bowl shape
                rim_height - crater_depth * (1.0 - t / 0.3)
            } else if t < 1.0 {
                // Cone: rises to rim then descends
                let cone_t = (t - 0.3) / 0.7;
                rim_height * (1.0 - cone_t * cone_t)
            } else {
                // Gradual falloff beyond the base
                let outer_t = (t - 1.0) / 0.5;
                let base_h = 0.0;
                base_h + (rim_height * 0.1) * (1.0 - outer_t).max(0.0)
            };

            // Blend with existing terrain (max blend: template wins where it is higher)
            let blend = (1.0 - (t / 1.3).powi(4)).clamp(0.0, 1.0);
            heightmap[idx] = heightmap[idx] * (1.0 - blend) + volcano_h * blend;
        }
    }
}
```

**Mountain Range:**
```rust
fn stamp_mountain_range(
    heightmap: &mut [f32],
    width: usize,
    height: usize,
    // Spine defined as a polyline
    spine: &[(f32, f32)],
    range_width: f32,     // Half-width of the range (8-15 cells)
    peak_height: f32,     // 0.75-0.95
    seed: i32,
) {
    // For each cell, find distance to the nearest spine segment
    let mut spine_noise = FastNoiseLite::with_seed(seed + 7000);
    spine_noise.set_noise_type(Some(NoiseType::OpenSimplex2));
    spine_noise.set_frequency(Some(0.08));

    for y in 0..height {
        for x in 0..width {
            let min_dist = spine.windows(2).map(|seg| {
                point_to_segment_distance(x as f32, y as f32, seg[0], seg[1])
            }).fold(f32::MAX, f32::min);

            if min_dist > range_width * 2.0 { continue; }

            let t = min_dist / range_width;
            let noise_val = spine_noise.get_noise_2d(x as f32, y as f32);

            // Mountain profile: sharp peak at spine, gradual descent
            let mountain_h = if t < 1.0 {
                peak_height * (1.0 - t * t) * (0.7 + noise_val * 0.3)
            } else {
                let outer = (t - 1.0) / 1.0;
                peak_height * 0.15 * (1.0 - outer).max(0.0) * (0.8 + noise_val * 0.2)
            };

            let idx = y * width + x;
            heightmap[idx] = heightmap[idx].max(mountain_h);
        }
    }
}
```

**Archipelago:** Generate multiple island-template stamps with random sizes and positions, each with its own edge falloff.

**River Valley:** Carve a wide U-shaped or V-shaped valley between two points, with the river at the bottom.

#### Template Selection

Templates can be:
1. **Player-selected:** UI presents options like "Coastal", "River Valley", "Mountain", "Volcanic Island", "Plains"
2. **Seed-determined:** The seed determines which template (if any) is applied, and where
3. **None:** Fully procedural with no templates (default)

### 6.5 Real-World Scale Comparisons

Our map is 256 cells x 256 cells at 16m/cell = 4,096m x 4,096m = ~4.1km x 4.1km. Total area: ~16.8 km^2.

**What fits in this space (real cities):**

| City/Area                        | Real Size          | Fit in Our Map?    |
|----------------------------------|--------------------|--------------------|
| Manhattan (tip to Central Park)  | 3.7km x 2.0km     | Yes, comfortably   |
| City of London (historic core)   | 2.6km x 1.6km     | Yes, with suburbs  |
| Venice (historic center)         | 1.8km x 1.2km     | Yes, plus lagoon   |
| Monaco                           | 2.0km x 0.7km     | Yes, with coast    |
| San Francisco downtown           | 3.0km x 2.5km     | Yes, tight fit     |
| Paris (within peripherique)      | 10km x 8km        | No (1/5th scale)   |
| Central Tokyo (Yamanote loop)    | 8km x 6km         | No (1/4th scale)   |
| Singapore                        | 50km x 27km       | No (far too large) |

**Interpretation:** Our map represents a small city or a single district of a large city. This is comparable to:
- A European old town with surrounding neighborhoods
- A small American downtown with immediate suburbs
- An island city-state like a mini-Monaco

**Implications for terrain:**
- At this scale, a single river is a major feature (spanning the whole map)
- Mountains occupy a significant fraction of buildable space
- 2-3 distinct elevation zones is enough variety
- Players should not expect sprawling suburbs -- every cell matters

**Scale considerations for terrain features:**

| Feature            | Real Scale  | Our Scale (cells)  | Notes                          |
|--------------------|-------------|--------------------|--------------------------------|
| Creek              | 2-5m wide   | 0.1-0.3 cells      | Below cell resolution, skip    |
| Small river        | 10-30m wide | 0.6-2 cells        | Minimum visible river          |
| Major river        | 50-200m     | 3-12 cells          | Dominant feature               |
| Pond               | 50m diam    | 3 cells diameter   | Smallest useful water body     |
| Small lake         | 200-500m    | 12-30 cells        | Significant landmark           |
| Hill               | 200m wide   | 12 cells           | Visible terrain bump           |
| Mountain           | 1-2km base  | 60-125 cells       | Dominates large map area       |
| Beach              | 10-30m deep | 1-2 cells          | Narrow fringe                  |
| Cliff              | Vertical    | 1 cell transition  | Sharp elevation change         |

---

## 7. Terrain Interaction with Game Systems

Terrain is not just visual -- it should deeply affect every game system. This section catalogs how elevation, slope, soil type, and water proximity interact with construction, simulation, and economy.

### 7.1 Slope Effects

Slope is the rate of elevation change between adjacent cells. It affects construction feasibility, cost, and several simulation systems.

#### Slope Calculation

```rust
/// Compute slope at a cell as the maximum elevation difference to any neighbor,
/// normalized by cell distance. Returns a value in range [0, ~0.5] for our terrain.
/// Slope of 0.0 = perfectly flat, 0.1 = 10% grade, etc.
fn cell_slope(heightmap: &[f32], width: usize, height: usize, x: usize, y: usize) -> f32 {
    let idx = y * width + x;
    let h = heightmap[idx];
    let mut max_slope = 0.0f32;

    // Check 8 neighbors
    for &(dx, dy) in &[(-1i32, 0), (1, 0), (0, -1), (0, 1),
                        (-1, -1), (1, -1), (-1, 1), (1, 1)] {
        let nx = x as i32 + dx;
        let ny = y as i32 + dy;
        if nx < 0 || nx >= width as i32 || ny < 0 || ny >= height as i32 {
            continue;
        }
        let ni = ny as usize * width + nx as usize;
        let dh = (h - heightmap[ni]).abs();
        // Diagonal distance is sqrt(2) cells, cardinal is 1 cell
        let dist = if dx != 0 && dy != 0 { 1.414 } else { 1.0 };
        let slope = dh / dist;
        max_slope = max_slope.max(slope);
    }

    max_slope
}

/// Convert internal slope to real-world percentage grade
fn slope_to_grade_percent(slope: f32) -> f32 {
    // Our normalized heights span [0, 1] over max_elevation_meters
    // Slope is the normalized height difference per cell (16m)
    // Grade = (rise / run) * 100
    // rise = slope * max_elevation_meters, run = CELL_SIZE
    let max_elev = 800.0;  // Default temperate setting
    slope * max_elev / CELL_SIZE * 100.0
}
```

#### Slope Impact Table

| System                  | Slope Effect                                                    | Implementation                              |
|-------------------------|-----------------------------------------------------------------|---------------------------------------------|
| **Road construction**   | Cost multiplier: 1.0x flat, 1.5x moderate, 3.0x steep, impossible > 25% grade | Multiply `RoadType::cost()` by slope factor |
| **Building placement**  | Residential: max 15% grade. Commercial: max 10%. Industrial: max 8%. High-rise: max 5%. | Check slope before allowing building spawn |
| **Water runoff speed**  | Steeper slope = faster runoff = less infiltration = more flooding downstream | Used in flood simulation system |
| **Fire spread rate**    | Fire travels uphill 2-4x faster than downhill (convective preheating) | Multiply fire spread probability by slope factor in uphill direction |
| **Road vehicle speed**  | Uphill: speed * (1.0 - grade * 0.02). Downhill: speed * (1.0 + grade * 0.01). Max reduction: 50% | Applied in pathfinding edge weights |
| **Erosion risk**        | Steep exposed slopes erode over time if deforested, increasing maintenance costs | Gradual terrain modification system |
| **Construction time**   | Steep sites require grading/terracing, adding 20-100% build time | Applied as building construction duration modifier |
| **Accessibility**       | Pedestrians avoid steep paths (walking speed penalty above 8% grade) | Affects path-based citizen movement |

```rust
fn road_construction_cost_modifier(slope: f32) -> f32 {
    let grade = slope_to_grade_percent(slope);
    if grade < 5.0 {
        1.0  // Flat to gentle slope, no extra cost
    } else if grade < 10.0 {
        1.0 + (grade - 5.0) / 5.0 * 0.5  // 1.0x to 1.5x
    } else if grade < 20.0 {
        1.5 + (grade - 10.0) / 10.0 * 1.5  // 1.5x to 3.0x
    } else if grade < 25.0 {
        3.0 + (grade - 20.0) / 5.0 * 2.0   // 3.0x to 5.0x
    } else {
        f32::MAX  // Cannot build road
    }
}

fn building_max_slope(zone: ZoneType) -> f32 {
    // Returns maximum slope (in normalized units) for building placement
    match zone {
        ZoneType::ResidentialLow => 0.025,   // ~15% grade -- houses on hills
        ZoneType::ResidentialHigh => 0.015,  // ~10% grade -- apartments need flat foundations
        ZoneType::CommercialLow => 0.018,    // ~12% grade -- strip malls
        ZoneType::CommercialHigh => 0.010,   // ~6% grade -- office towers
        ZoneType::Industrial => 0.012,       // ~8% grade -- factories need flat floors
        ZoneType::Office => 0.010,           // ~6% grade -- similar to commercial high
        ZoneType::None => 1.0,               // No restriction
    }
}
```

### 7.2 Elevation Effects

Absolute elevation (not just slope) affects temperature, wind exposure, and aesthetic value.

#### Temperature Lapse Rate

Real-world temperature decreases approximately 6.5 degrees Celsius per 1,000 meters of elevation gain (the environmental lapse rate). For our terrain:

```rust
fn temperature_at_cell(base_temp: f32, elevation: f32, max_elevation_m: f32) -> f32 {
    let height_m = elevation * max_elevation_m;
    base_temp - (height_m / 1000.0) * 6.5
}

// Example with temperate preset (base_temp=15C, max_elev=800m):
// Elevation 0.0 (sea level): 15.0C
// Elevation 0.35 (water threshold, ~280m): 13.2C
// Elevation 0.5 (mid hills, ~400m): 12.4C
// Elevation 0.75 (high hills, ~600m): 11.1C
// Elevation 1.0 (peak, ~800m): 9.8C
```

**Gameplay impact of elevation-based temperature:**
- Higher buildings need more heating in winter (increased utility costs)
- Lower-elevation industrial areas may be warmer and more uncomfortable
- Snow line appears at high elevations in winter (visual + affects road maintenance)
- Temperature affects crop growth (agricultural zones at high elevation are less productive)

#### Wind Exposure

High, exposed terrain receives more wind. This affects:
- Wind power generation (hilltop wind turbines are 30-50% more efficient)
- Building heating costs (wind chill increases energy consumption)
- Noise propagation (sound carries farther in windy areas)
- Pollution dispersal (high-wind areas disperse pollution faster)

```rust
fn wind_exposure(
    heightmap: &[f32],
    width: usize,
    height: usize,
    x: usize,
    y: usize,
    wind_dir: (f32, f32),
) -> f32 {
    let idx = y * width + x;
    let h = heightmap[idx];

    // Exposure = how much higher this cell is than upwind terrain
    // Check 10 cells upwind
    let mut upwind_max = 0.0f32;
    let (wdx, wdy) = wind_dir;

    for step in 1..=10 {
        let ux = x as f32 - wdx * step as f32;
        let uy = y as f32 - wdy * step as f32;
        let uix = ux as usize;
        let uiy = uy as usize;

        if uix >= width || uiy >= height { break; }
        upwind_max = upwind_max.max(heightmap[uiy * width + uix]);
    }

    // Exposure is how much we stick above the upwind terrain
    // Sheltered cells (in the lee of hills) have low exposure
    (h - upwind_max).max(0.0) * 10.0  // Scale to useful range
}
```

#### View Value and Land Price

Elevated positions with views of water, parks, or distant terrain command higher land values:

```rust
fn view_value(
    heightmap: &[f32],
    water_mask: &[bool],
    width: usize,
    height: usize,
    x: usize,
    y: usize,
) -> f32 {
    let idx = y * width + x;
    let h = heightmap[idx];
    let mut value = 0.0f32;

    // Cast rays in 8 directions, count visible water/landscape cells
    let directions: [(f32, f32); 8] = [
        (1.0, 0.0), (-1.0, 0.0), (0.0, 1.0), (0.0, -1.0),
        (0.707, 0.707), (-0.707, 0.707), (0.707, -0.707), (-0.707, -0.707),
    ];

    for &(dx, dy) in &directions {
        let mut max_angle = f32::MIN;  // Tracks the highest "angle" seen so far
        let mut visible_water = 0;
        let mut visible_distance = 0;

        for step in 1..30 {
            let tx = x as f32 + dx * step as f32;
            let ty = y as f32 + dy * step as f32;
            let tix = tx as usize;
            let tiy = ty as usize;

            if tix >= width || tiy >= height { break; }

            let ti = tiy * width + tix;
            let th = heightmap[ti];

            // Angle from viewer to this cell (simplified)
            let angle = (th - h) / step as f32;

            if angle > max_angle {
                // This cell is visible (not blocked by closer terrain)
                max_angle = angle;
                visible_distance += 1;

                if water_mask[ti] {
                    visible_water += 1;
                }
            }
        }

        // Water views are especially valuable
        value += visible_water as f32 * 2.0;
        // Long sight lines are valuable (panoramic views)
        value += visible_distance as f32 * 0.5;
    }

    // Normalize to [0, 1]
    (value / 200.0).clamp(0.0, 1.0)
}
```

**Land value modifiers from elevation/view:**
- Water view: +15-30% land value
- Hilltop panoramic view: +10-20% land value
- Valley floor (no view, potential flooding): -5-10% land value
- Sheltered from wind (lee of hill): +5% residential land value

### 7.3 Soil Type Grid

Soil composition affects construction, agriculture, drainage, and groundwater. A soil type grid adds geological depth to the terrain.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum SoilType {
    Clay,       // Poor drainage, cheap foundations, good agriculture
    Sand,       // Good drainage, moderate foundations, poor agriculture
    Rock,       // No drainage issues, expensive foundations, no agriculture
    Loam,       // Best agriculture, good drainage, moderate foundations
    Peat,       // Wetland soil, very poor foundations, moderate agriculture
    Gravel,     // Excellent drainage, good foundations, no agriculture
}

impl SoilType {
    fn foundation_cost_modifier(self) -> f32 {
        match self {
            SoilType::Clay => 1.0,     // Standard cost
            SoilType::Sand => 1.1,     // Needs compaction
            SoilType::Rock => 1.8,     // Needs blasting/drilling
            SoilType::Loam => 1.0,     // Standard
            SoilType::Peat => 2.5,     // Needs deep piles
            SoilType::Gravel => 0.9,   // Easiest to build on
        }
    }

    fn agriculture_factor(self) -> f32 {
        match self {
            SoilType::Clay => 0.6,     // Decent but heavy
            SoilType::Sand => 0.3,     // Poor water retention
            SoilType::Rock => 0.0,     // Cannot farm on rock
            SoilType::Loam => 1.0,     // Ideal
            SoilType::Peat => 0.7,     // Good nutrients but waterlogged
            SoilType::Gravel => 0.1,   // Almost useless for farming
        }
    }

    fn drainage_rate(self) -> f32 {
        // How fast water infiltrates (higher = less surface flooding)
        match self {
            SoilType::Clay => 0.2,     // Very slow drainage
            SoilType::Sand => 0.9,     // Very fast drainage
            SoilType::Rock => 0.05,    // Almost impermeable
            SoilType::Loam => 0.6,     // Good drainage
            SoilType::Peat => 0.3,     // Slow, waterlogged
            SoilType::Gravel => 1.0,   // Near-instant drainage
        }
    }

    fn groundwater_recharge(self) -> f32 {
        // How much rain infiltrates to groundwater
        match self {
            SoilType::Clay => 0.1,
            SoilType::Sand => 0.8,
            SoilType::Rock => 0.02,
            SoilType::Loam => 0.5,
            SoilType::Peat => 0.2,
            SoilType::Gravel => 0.9,
        }
    }
}
```

#### Soil Type Generation

Soil type is derived from elevation, slope, and proximity to water:

```rust
fn generate_soil_grid(
    heightmap: &[f32],
    water_dist: &[f32],
    slopes: &[f32],
    width: usize,
    height: usize,
    seed: i32,
) -> Vec<SoilType> {
    let total = width * height;
    let mut soil = vec![SoilType::Loam; total];

    // Geological noise: creates irregular soil zones
    let mut geo_noise = FastNoiseLite::with_seed(seed + 4000);
    geo_noise.set_noise_type(Some(NoiseType::OpenSimplex2));
    geo_noise.set_frequency(Some(0.012));

    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            let elev = heightmap[idx];
            let slope = slopes[idx];
            let wd = water_dist[idx];
            let noise = (geo_noise.get_noise_2d(x as f32, y as f32) + 1.0) * 0.5;

            // Rock: high elevation + steep slopes
            if elev > 0.7 || slope > 0.03 {
                soil[idx] = if noise > 0.6 { SoilType::Rock } else { SoilType::Gravel };
                continue;
            }

            // Peat: low elevation, close to water, wet
            if elev < 0.42 && wd < 5.0 {
                soil[idx] = SoilType::Peat;
                continue;
            }

            // Sand: near water, moderate elevation (beach/floodplain)
            if wd < 3.0 && elev < 0.45 {
                soil[idx] = SoilType::Sand;
                continue;
            }

            // Clay vs Loam based on noise (geological variation)
            if noise < 0.35 {
                soil[idx] = SoilType::Clay;
            } else if noise < 0.75 {
                soil[idx] = SoilType::Loam;
            } else {
                // Sandy patches in upland areas
                soil[idx] = if elev > 0.55 { SoilType::Gravel } else { SoilType::Sand };
            }
        }
    }

    soil
}
```

### 7.4 Flood Plains

Cells near rivers at low elevation are at risk of periodic flooding. Flood risk should be calculated during terrain generation and stored for the flood disaster system.

```rust
#[derive(Resource)]
struct FloodRiskGrid {
    risk: Vec<f32>,      // 0.0 = no risk, 1.0 = floods every rain event
    width: usize,
    height: usize,
}

fn compute_flood_risk(
    heightmap: &[f32],
    flow_accumulation: &[u32],
    water_dist: &[f32],
    slopes: &[f32],
    soil_types: &[SoilType],
    width: usize,
    height: usize,
) -> FloodRiskGrid {
    let total = width * height;
    let mut risk = vec![0.0f32; total];

    for i in 0..total {
        // Skip water cells (they are already water, not "at risk")
        if heightmap[i] < WATER_THRESHOLD { continue; }

        // Factor 1: Proximity to water (exponential increase near water)
        let proximity_risk = (-water_dist[i] / 5.0).exp();  // halves every ~3.5 cells

        // Factor 2: Low-lying terrain (cells barely above water level)
        let elevation_risk = ((WATER_THRESHOLD + 0.05 - heightmap[i]) / 0.05).clamp(0.0, 1.0);

        // Factor 3: Upstream flow accumulation (more water flowing through = more risk)
        let flow_risk = (flow_accumulation[i] as f32 / 200.0).clamp(0.0, 1.0);

        // Factor 4: Flat terrain (water pools on flat land)
        let flat_risk = (1.0 - slopes[i] * 20.0).clamp(0.0, 1.0);

        // Factor 5: Poor drainage soil increases flood risk
        let soil_risk = 1.0 - soil_types[i].drainage_rate();

        // Combined risk (weighted geometric mean)
        risk[i] = (proximity_risk * 0.35
            + elevation_risk * 0.25
            + flow_risk * 0.15
            + flat_risk * 0.15
            + soil_risk * 0.10)
            .clamp(0.0, 1.0);
    }

    FloodRiskGrid { risk, width, height }
}
```

**Gameplay integration:**
- **Overlay:** Flood risk overlay shows areas colored green (safe) to red (high risk)
- **Warnings:** When placing buildings in flood-risk zones, display a warning
- **Insurance:** Buildings in flood zones pay higher insurance (maintenance costs)
- **Flood events:** During heavy rain, cells with risk > 0.5 may flood, damaging buildings
- **Mitigation:** Levees, drainage canals, and retention ponds reduce flood risk
- **Land value:** Flood risk reduces land value by up to 20%

### 7.5 Earthquake Fault Lines

Fault lines are linear geological features that increase earthquake probability. They add long-term strategic considerations to city placement.

```rust
struct FaultLine {
    start: (f32, f32),    // Grid coordinates
    end: (f32, f32),
    activity: f32,        // 0.0-1.0, how active the fault is
    depth: f32,           // Shallow faults are more dangerous
}

fn generate_fault_lines(
    width: usize,
    height: usize,
    num_faults: u32,     // 0-3 per map
    rng: &mut impl Rng,
) -> Vec<FaultLine> {
    let mut faults = Vec::new();

    for _ in 0..num_faults {
        // Faults tend to run in consistent directions (tectonic forces)
        let base_angle = rng.gen_range(0.0..std::f32::consts::PI);

        let mid_x = rng.gen_range(width as f32 * 0.2..width as f32 * 0.8);
        let mid_y = rng.gen_range(height as f32 * 0.2..height as f32 * 0.8);
        let half_length = rng.gen_range(40.0..120.0);  // 640m to 1920m

        let dx = base_angle.cos() * half_length;
        let dy = base_angle.sin() * half_length;

        faults.push(FaultLine {
            start: (mid_x - dx, mid_y - dy),
            end: (mid_x + dx, mid_y + dy),
            activity: rng.gen_range(0.3..0.9),
            depth: rng.gen_range(0.5..2.0),  // km
        });
    }

    faults
}

/// Compute earthquake risk at each cell based on distance to faults
fn earthquake_risk_at(x: f32, y: f32, faults: &[FaultLine]) -> f32 {
    let mut max_risk = 0.0f32;

    for fault in faults {
        let dist = point_to_segment_distance(x, y, fault.start, fault.end);

        // Risk decreases with distance from fault (inverse square)
        // and increases with fault activity
        let distance_factor = 1.0 / (1.0 + dist * dist * 0.01);
        let risk = distance_factor * fault.activity;

        max_risk = max_risk.max(risk);
    }

    max_risk
}

fn point_to_segment_distance(
    px: f32, py: f32,
    a: (f32, f32), b: (f32, f32),
) -> f32 {
    let (ax, ay) = a;
    let (bx, by) = b;
    let dx = bx - ax;
    let dy = by - ay;
    let len_sq = dx * dx + dy * dy;

    if len_sq < 0.001 {
        return ((px - ax).powi(2) + (py - ay).powi(2)).sqrt();
    }

    let t = ((px - ax) * dx + (py - ay) * dy) / len_sq;
    let t = t.clamp(0.0, 1.0);

    let closest_x = ax + t * dx;
    let closest_y = ay + t * dy;

    ((px - closest_x).powi(2) + (py - closest_y).powi(2)).sqrt()
}
```

**Gameplay integration:**
- Fault lines are visible in the terrain overlay (subtle visual marker)
- Buildings on/near faults have increased earthquake damage
- Earthquake-resistant building codes (research/policy) reduce damage
- Fault lines can be discovered through geological survey (like resources)
- Emergency services need faster response times near faults

---

## 8. Technical Implementation

This section covers the engineering details of integrating procedural terrain into the Megacity codebase: chunk-based generation, LOD rendering, player terrain modification, serialization, and Bevy ECS integration.

### 8.1 Chunk-Based Generation

The existing codebase uses 8x8 cell chunks (32 chunks per axis = 1,024 total chunks). Terrain generation can leverage this chunk structure for:
1. **Streaming generation:** Generate chunks on demand as the camera moves (relevant if map size increases beyond 256x256)
2. **Parallel generation:** Generate independent chunks on separate threads
3. **Dirty tracking:** Only regenerate mesh for modified chunks (already implemented via `ChunkDirty`)

However, for a 256x256 grid, the full generation pipeline runs in under 200ms, so streaming is not necessary. The primary benefit of chunk awareness is in the rendering pipeline.

#### Chunk-Aware Noise Generation

If chunk-based streaming is ever needed (e.g., for larger maps), noise generation is naturally chunk-compatible because noise is evaluated per-cell independently:

```rust
fn generate_chunk_heightmap(
    chunk_x: usize,
    chunk_y: usize,
    chunk_size: usize,
    noise: &FastNoiseLite,
) -> Vec<f32> {
    let mut heights = vec![0.0; chunk_size * chunk_size];

    let base_gx = chunk_x * chunk_size;
    let base_gy = chunk_y * chunk_size;

    for ly in 0..chunk_size {
        for lx in 0..chunk_size {
            let gx = base_gx + lx;
            let gy = base_gy + ly;
            let raw = noise.get_noise_2d(gx as f32, gy as f32);
            heights[ly * chunk_size + lx] = (raw + 1.0) * 0.5;
        }
    }

    heights
}
```

**Caveat for erosion:** Hydraulic erosion is NOT chunk-compatible because particles flow across chunk boundaries. Erosion must run on the full heightmap. If streaming generation is needed, erosion can be run in a post-processing pass once all chunks have been generated, or approximated with a chunked approach where particles are "handed off" at boundaries.

#### Parallel Chunk Mesh Generation

Mesh generation (the expensive part of rendering) CAN be parallelized per chunk using Bevy's `ComputeTaskPool`:

```rust
use bevy::tasks::ComputeTaskPool;

fn rebuild_dirty_chunks_parallel(
    grid: &WorldGrid,
    roads: &RoadNetwork,
    segments: &RoadSegmentStore,
    overlay: &OverlayMode,
    overlay_grids: &OverlayGrids,
    season: Season,
    dirty_chunks: &[(usize, usize)],  // (chunk_x, chunk_y)
) -> Vec<((usize, usize), Mesh)> {
    let pool = ComputeTaskPool::get();

    // Each chunk mesh build is independent -- safe to parallelize
    pool.scope(|s| {
        for &(cx, cy) in dirty_chunks {
            s.spawn(async move {
                let mesh = build_chunk_mesh(
                    grid, roads, segments, cx, cy,
                    overlay, overlay_grids, season,
                );
                ((cx, cy), mesh)
            });
        }
    })
}
```

In the current codebase, `rebuild_dirty_chunks` processes chunks sequentially. For initial terrain spawn (all 1,024 chunks), parallelization would reduce mesh generation from ~50ms to ~15ms on a 4-core machine.

### 8.2 LOD for Terrain Rendering

The existing terrain renderer creates per-cell quads (4 vertices, 2 triangles per cell). For a full 256x256 grid, this is:
- 256 * 256 * 4 = 262,144 vertices
- 256 * 256 * 2 = 131,072 triangles

This is modest by modern standards, but if the map grows or vertex effects are added, LOD becomes important.

#### Per-Chunk LOD Levels

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TerrainLOD {
    Full,        // 1 quad per cell (8x8 = 64 quads per chunk)
    Half,        // 1 quad per 2x2 cells (4x4 = 16 quads per chunk)
    Quarter,     // 1 quad per 4x4 cells (2x2 = 4 quads per chunk)
    Single,      // 1 quad for entire chunk (1 quad per chunk)
}

fn select_terrain_lod(
    chunk_world_pos: Vec3,
    camera_pos: Vec3,
) -> TerrainLOD {
    let dist = chunk_world_pos.distance(camera_pos);

    // Distance thresholds in world units (CELL_SIZE = 16m)
    if dist < 400.0 {        // < 25 cells away
        TerrainLOD::Full
    } else if dist < 800.0 { // 25-50 cells away
        TerrainLOD::Half
    } else if dist < 1600.0 { // 50-100 cells away
        TerrainLOD::Quarter
    } else {
        TerrainLOD::Single
    }
}
```

#### Building LOD Meshes

```rust
fn build_chunk_mesh_lod(
    grid: &WorldGrid,
    cx: usize,
    cy: usize,
    lod: TerrainLOD,
    season: Season,
) -> Mesh {
    let step = match lod {
        TerrainLOD::Full => 1,
        TerrainLOD::Half => 2,
        TerrainLOD::Quarter => 4,
        TerrainLOD::Single => CHUNK_SIZE,
    };

    let cells_per_side = CHUNK_SIZE / step;
    let base_gx = cx * CHUNK_SIZE;
    let base_gy = cy * CHUNK_SIZE;

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();

    for ly in (0..CHUNK_SIZE).step_by(step) {
        for lx in (0..CHUNK_SIZE).step_by(step) {
            let gx = base_gx + lx;
            let gy = base_gy + ly;

            if gx >= GRID_WIDTH || gy >= GRID_HEIGHT { continue; }

            // For reduced LOD, average the elevation and color of covered cells
            let (avg_elevation, avg_color) = if step == 1 {
                let cell = grid.get(gx, gy);
                (cell.elevation, terrain_color_simple(cell, season))
            } else {
                let mut sum_elev = 0.0f32;
                let mut sum_r = 0.0f32;
                let mut sum_g = 0.0f32;
                let mut sum_b = 0.0f32;
                let mut count = 0u32;

                for dy in 0..step {
                    for dx in 0..step {
                        let sx = gx + dx;
                        let sy = gy + dy;
                        if sx < GRID_WIDTH && sy < GRID_HEIGHT {
                            let cell = grid.get(sx, sy);
                            sum_elev += cell.elevation;
                            let c = terrain_color_simple(cell, season);
                            sum_r += c[0];
                            sum_g += c[1];
                            sum_b += c[2];
                            count += 1;
                        }
                    }
                }

                let n = count as f32;
                (sum_elev / n, [sum_r / n, sum_g / n, sum_b / n, 1.0])
            };

            let x0 = lx as f32 * CELL_SIZE;
            let z0 = ly as f32 * CELL_SIZE;
            let x1 = (lx + step) as f32 * CELL_SIZE;
            let z1 = (ly + step) as f32 * CELL_SIZE;

            // Use elevation for Y coordinate (3D terrain mesh)
            let y = avg_elevation * MAX_TERRAIN_HEIGHT;  // e.g., 50.0 world units max height

            let vi = positions.len() as u32;
            positions.push([x0, y, z0]);
            positions.push([x1, y, z0]);
            positions.push([x1, y, z1]);
            positions.push([x0, y, z1]);
            normals.extend_from_slice(&[[0.0, 1.0, 0.0]; 4]);
            colors.extend_from_slice(&[avg_color; 4]);

            indices.push(vi);
            indices.push(vi + 2);
            indices.push(vi + 1);
            indices.push(vi);
            indices.push(vi + 3);
            indices.push(vi + 2);
        }
    }

    // ... build mesh from attributes (same as existing code)
    build_mesh_from_attributes(positions, normals, colors, indices)
}
```

#### LOD Transition Seams

When adjacent chunks have different LOD levels, their edges do not match, creating visible cracks. Solutions:

1. **Skirt geometry:** Add vertical "skirt" polygons hanging below each chunk edge. These fill any gaps between LOD levels. Cost: 4 * (cells_per_edge * 2) triangles per chunk.

2. **Edge stitching:** Generate special edge meshes that transition between LOD levels. More complex but produces cleaner results.

3. **No visible height variation (current approach):** Since the current renderer uses y=0.0 for all vertices (flat terrain), LOD seams are not visible. If 3D terrain is implemented, skirt geometry is the simplest fix.

### 8.3 Terrain Modification

Players should be able to modify terrain: flatten areas for buildings, raise ground to prevent flooding, dig canals for water flow, create hills for parks.

#### Modification Operations

```rust
enum TerrainModification {
    Flatten {
        center: (usize, usize),
        radius: usize,
        target_height: f32,     // Target elevation
    },
    Raise {
        center: (usize, usize),
        radius: usize,
        amount: f32,            // How much to raise
    },
    Lower {
        center: (usize, usize),
        radius: usize,
        amount: f32,
    },
    Smooth {
        center: (usize, usize),
        radius: usize,
        strength: f32,          // 0-1, blend with average of neighbors
    },
    Canal {
        path: Vec<(usize, usize)>,
        width: usize,
        depth: f32,
    },
}

fn apply_terrain_modification(
    grid: &mut WorldGrid,
    modification: &TerrainModification,
) -> Vec<(usize, usize)> {
    let mut modified_cells = Vec::new();

    match modification {
        TerrainModification::Flatten { center, radius, target_height } => {
            let r = *radius as i32;
            for dy in -r..=r {
                for dx in -r..=r {
                    let nx = center.0 as i32 + dx;
                    let ny = center.1 as i32 + dy;
                    if nx < 0 || nx >= GRID_WIDTH as i32 || ny < 0 || ny >= GRID_HEIGHT as i32 {
                        continue;
                    }

                    let dist = ((dx * dx + dy * dy) as f32).sqrt();
                    if dist > *radius as f32 { continue; }

                    // Smooth falloff: full effect at center, no effect at edge
                    let blend = 1.0 - (dist / *radius as f32).powi(2);

                    let cell = grid.get_mut(nx as usize, ny as usize);
                    cell.elevation = cell.elevation * (1.0 - blend) + target_height * blend;

                    // Update cell type based on new elevation
                    if cell.elevation < WATER_THRESHOLD {
                        cell.cell_type = CellType::Water;
                    } else if cell.cell_type == CellType::Water {
                        cell.cell_type = CellType::Grass;
                    }

                    modified_cells.push((nx as usize, ny as usize));
                }
            }
        }

        TerrainModification::Canal { path, width, depth } => {
            for &(px, py) in path {
                let w = *width as i32;
                for dy in -w..=w {
                    for dx in -w..=w {
                        let nx = px as i32 + dx;
                        let ny = py as i32 + dy;
                        if nx < 0 || nx >= GRID_WIDTH as i32
                            || ny < 0 || ny >= GRID_HEIGHT as i32 {
                            continue;
                        }

                        let dist = ((dx * dx + dy * dy) as f32).sqrt();
                        if dist > *width as f32 { continue; }

                        let falloff = 1.0 - dist / *width as f32;
                        let cell = grid.get_mut(nx as usize, ny as usize);
                        cell.elevation -= depth * falloff;

                        if cell.elevation < WATER_THRESHOLD {
                            cell.cell_type = CellType::Water;
                        }

                        modified_cells.push((nx as usize, ny as usize));
                    }
                }
            }
        }

        // Raise and Lower are similar to Flatten but add/subtract instead of target
        // Smooth averages with neighbors
        _ => { /* similar implementations */ }
    }

    modified_cells
}
```

#### Undo Stack for Terrain Modifications

Terrain changes should be undoable. Store the elevation snapshot of affected cells before each operation:

```rust
struct TerrainUndoEntry {
    operation: TerrainModification,
    previous_elevations: Vec<(usize, usize, f32)>,  // (x, y, old_elevation)
    previous_cell_types: Vec<(usize, usize, CellType)>,
    cost: f64,  // Money spent on this operation
}

struct TerrainUndoStack {
    entries: Vec<TerrainUndoEntry>,
    max_entries: usize,  // Limit memory usage (e.g., 100 operations)
}

impl TerrainUndoStack {
    fn push(&mut self, entry: TerrainUndoEntry) {
        if self.entries.len() >= self.max_entries {
            self.entries.remove(0);  // Drop oldest
        }
        self.entries.push(entry);
    }

    fn undo(&mut self, grid: &mut WorldGrid) -> Option<f64> {
        if let Some(entry) = self.entries.pop() {
            // Restore previous elevations
            for &(x, y, elev) in &entry.previous_elevations {
                grid.get_mut(x, y).elevation = elev;
            }
            for &(x, y, ct) in &entry.previous_cell_types {
                grid.get_mut(x, y).cell_type = ct;
            }
            Some(entry.cost * 0.75)  // Refund 75% of cost
        } else {
            None
        }
    }
}
```

#### Terrain Modification Costs

Terraforming should cost money, proportional to the volume of earth moved:

```rust
fn terrain_modification_cost(
    grid: &WorldGrid,
    modification: &TerrainModification,
) -> f64 {
    let cost_per_cell_per_unit = 50.0;  // $50 per cell per 0.01 elevation change

    match modification {
        TerrainModification::Flatten { center, radius, target_height } => {
            let r = *radius as i32;
            let mut total_work = 0.0;

            for dy in -r..=r {
                for dx in -r..=r {
                    let nx = center.0 as i32 + dx;
                    let ny = center.1 as i32 + dy;
                    if nx < 0 || nx >= GRID_WIDTH as i32
                        || ny < 0 || ny >= GRID_HEIGHT as i32 {
                        continue;
                    }

                    let dist = ((dx * dx + dy * dy) as f32).sqrt();
                    if dist > *radius as f32 { continue; }

                    let cell = grid.get(nx as usize, ny as usize);
                    let diff = (cell.elevation - target_height).abs();
                    let blend = 1.0 - (dist / *radius as f32).powi(2);
                    total_work += diff as f64 * blend as f64;
                }
            }

            total_work * cost_per_cell_per_unit
        }
        // Similar for other operations
        _ => 0.0,
    }
}
```

### 8.4 Heightmap Serialization

The heightmap needs to be saved and loaded efficiently. Several options exist.

#### Option A: Float32 Per Cell (Current)

Each `Cell.elevation` is an `f32` (4 bytes). For 256x256:
- Size: 256 * 256 * 4 = 262,144 bytes = 256 KB
- Pros: Full precision, no conversion needed
- Cons: Largest option, mostly unnecessary precision

#### Option B: u16 Per Cell (Recommended)

Quantize to 16-bit unsigned integer (65,536 height levels):

```rust
fn compress_heightmap(cells: &[Cell]) -> Vec<u16> {
    cells.iter().map(|c| {
        (c.elevation.clamp(0.0, 1.0) * 65535.0) as u16
    }).collect()
}

fn decompress_heightmap(compressed: &[u16]) -> Vec<f32> {
    compressed.iter().map(|&v| {
        v as f32 / 65535.0
    }).collect()
}
```

- Size: 256 * 256 * 2 = 131,072 bytes = 128 KB
- Precision: 1/65536 = 0.0000153 elevation units. For 800m max height, this is 0.012m = 1.2cm resolution. More than sufficient.

#### Option C: u8 Per Cell (Compact)

Quantize to 8-bit unsigned integer (256 height levels):

```rust
fn compress_heightmap_u8(cells: &[Cell]) -> Vec<u8> {
    cells.iter().map(|c| {
        (c.elevation.clamp(0.0, 1.0) * 255.0) as u8
    }).collect()
}
```

- Size: 256 * 256 * 1 = 65,536 bytes = 64 KB
- Precision: 1/256 = 0.0039 elevation units. For 800m max height, this is 3.1m resolution. Adequate for gameplay (buildings are placed per-cell, not per-centimeter), but may produce visible terracing in the 3D mesh if heights are used for vertex Y positions.

#### Option D: Delta Encoding + Compression

Store the difference between adjacent cells (which is usually small), then compress with zlib/zstd:

```rust
fn delta_encode(heights: &[u16], width: usize) -> Vec<i16> {
    let mut deltas = Vec::with_capacity(heights.len());
    deltas.push(heights[0] as i16);  // First value is absolute

    for i in 1..heights.len() {
        let x = i % width;
        let prev = if x > 0 { heights[i - 1] } else { heights[i - width] };
        deltas.push(heights[i] as i16 - prev as i16);
    }

    deltas
}

// After delta encoding, most values are near zero.
// zstd compression at level 3 typically achieves 3-5x compression
// on delta-encoded heightmaps.
// Final size: roughly 25-40 KB for 256x256
```

**Recommendation:** Use u16 per cell (Option B) for the heightmap portion of save files. It is a good balance of precision and size. Apply zstd compression to the entire save file (not just the heightmap) for additional savings.

#### Integration with Existing Save System

The existing `crates/save/src/serialization.rs` serializes the full `WorldGrid` including all cell fields. The heightmap is already stored implicitly in the `Cell.elevation` field. No special handling is needed unless save file size becomes an issue.

If the save system moves to a custom binary format (rather than serde/JSON), the heightmap should be separated from per-cell gameplay data and stored in its compressed form:

```rust
struct SaveFile {
    // Terrain (regenerable from seed, but stored for modified terrain)
    heightmap: Vec<u16>,           // 128 KB
    soil_types: Vec<u8>,           // 64 KB (enum fits in u8)
    water_metadata: Vec<u8>,       // 64 KB (WaterType enum)

    // Per-cell gameplay state (must be saved)
    cell_types: Vec<u8>,           // 64 KB
    zone_types: Vec<u8>,           // 64 KB
    road_types: Vec<u8>,           // 64 KB
    building_ids: Vec<Option<u32>>,// 256 KB (sparse, could compress well)
    power_water: Vec<u8>,          // 64 KB (bitflags)

    // Resources
    resource_deposits: Vec<...>,   // Sparse, only cells with resources

    // Seed (for reference, not for regeneration if terrain was modified)
    terrain_seed: i32,
    terrain_was_modified: bool,
}
```

### 8.5 Bevy Integration

#### Terrain as a Resource

The `WorldGrid` is already a Bevy `Resource`, which is the correct approach for globally shared grid data. Additional terrain-related data should also be resources:

```rust
// Already exists:
// #[derive(Resource)] pub struct WorldGrid { ... }

// New terrain resources:
#[derive(Resource)]
pub struct TerrainMetadata {
    pub seed: i32,
    pub biome: BiomePreset,
    pub climate_params: ClimateParams,
    pub was_modified: bool,
}

#[derive(Resource)]
pub struct SoilGrid {
    pub types: Vec<SoilType>,
    pub width: usize,
    pub height: usize,
}

#[derive(Resource)]
pub struct MoistureGrid {
    pub moisture: Vec<f32>,
    pub width: usize,
    pub height: usize,
}

#[derive(Resource)]
pub struct FloodRiskGrid {
    pub risk: Vec<f32>,
    pub width: usize,
    pub height: usize,
}

#[derive(Resource)]
pub struct ForestDensityGrid {
    pub density: Vec<f32>,
    pub width: usize,
    pub height: usize,
}

// These grids are generated once at map creation and only modified by
// player terraforming or seasonal events. They should not be regenerated
// every frame.
```

#### System Registration

Terrain generation systems should run during the `Startup` schedule (one-time initialization) or in response to a "New Game" event:

```rust
pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<TerrainMetadata>()
            .init_resource::<SoilGrid>()
            .init_resource::<MoistureGrid>()
            .init_resource::<FloodRiskGrid>()
            .init_resource::<ForestDensityGrid>()
            .add_systems(Startup, generate_all_terrain)
            .add_systems(Update, (
                handle_terrain_modification
                    .run_if(resource_changed::<TerrainModificationRequest>()),
                update_derived_grids_after_modification
                    .after(handle_terrain_modification),
            ));
    }
}

fn generate_all_terrain(
    mut commands: Commands,
    mut grid: ResMut<WorldGrid>,
    mut resource_grid: ResMut<ResourceGrid>,
    metadata: Res<TerrainMetadata>,
) {
    let seed = metadata.seed;
    let climate = &metadata.climate_params;

    // 1. Heightmap
    generate_heightmap_fbm(&mut grid, seed);

    // 2. Erosion
    let mut heights: Vec<f32> = grid.cells.iter().map(|c| c.elevation).collect();
    let mut rng = StdRng::seed_from_u64(seed as u64);
    simulate_erosion(&mut heights, GRID_WIDTH, GRID_HEIGHT, 150_000,
                     &ErosionParams::default(), &mut rng);
    thermal_erosion(&mut heights, GRID_WIDTH, GRID_HEIGHT, 30, 0.04, 0.4);

    // Write back to grid
    for (i, h) in heights.iter().enumerate() {
        grid.cells[i].elevation = *h;
    }

    // 3. Water classification
    for cell in grid.cells.iter_mut() {
        if cell.elevation < WATER_THRESHOLD {
            cell.cell_type = CellType::Water;
        }
    }

    // 4. Derived grids
    let water_mask: Vec<bool> = grid.cells.iter()
        .map(|c| c.cell_type == CellType::Water)
        .collect();
    let water_dist = distance_transform(&water_mask, GRID_WIDTH, GRID_HEIGHT);
    let slopes: Vec<f32> = (0..GRID_WIDTH * GRID_HEIGHT).map(|i| {
        let x = i % GRID_WIDTH;
        let y = i / GRID_WIDTH;
        compute_slope(&heights, GRID_WIDTH, GRID_HEIGHT, x, y)
    }).collect();

    let moisture = compute_moisture_grid(&heights, &water_mask,
                                          GRID_WIDTH, GRID_HEIGHT, climate);
    let soil = generate_soil_grid(&heights, &water_dist, &slopes,
                                   GRID_WIDTH, GRID_HEIGHT, seed);
    let flood_risk = compute_flood_risk(&heights, &vec![0; heights.len()],
                                         &water_dist, &slopes, &soil,
                                         GRID_WIDTH, GRID_HEIGHT);

    // 5. Resources
    generate_resources(&mut resource_grid,
                       &grid.cells.iter().map(|c| c.elevation).collect::<Vec<_>>(),
                       seed as u32);

    // 6. Insert derived grids as resources
    commands.insert_resource(MoistureGrid {
        moisture, width: GRID_WIDTH, height: GRID_HEIGHT,
    });
    commands.insert_resource(SoilGrid {
        types: soil, width: GRID_WIDTH, height: GRID_HEIGHT,
    });
    commands.insert_resource(flood_risk);
}
```

#### 3D Terrain Mesh (Elevation as Y Coordinate)

The current renderer uses `y = 0.0` for all terrain vertices (flat plane with vertex colors indicating terrain type). To render actual 3D terrain, modify the mesh builder to use elevation:

```rust
// In build_chunk_mesh, change:
// let y = 0.0;
// To:
let elevation = cell.elevation;
let y = elevation * MAX_TERRAIN_HEIGHT;  // MAX_TERRAIN_HEIGHT = e.g., 50.0 world units

// For proper 3D terrain, each vertex needs its own elevation.
// The 4 corners of each cell quad should use the elevations of the
// cells at those corners (or interpolated elevations):
let h00 = grid.get(gx, gy).elevation;
let h10 = if gx + 1 < GRID_WIDTH { grid.get(gx + 1, gy).elevation } else { h00 };
let h01 = if gy + 1 < GRID_HEIGHT { grid.get(gx, gy + 1).elevation } else { h00 };
let h11 = if gx + 1 < GRID_WIDTH && gy + 1 < GRID_HEIGHT {
    grid.get(gx + 1, gy + 1).elevation
} else { h00 };

let scale = 50.0;  // World-space max height
positions.push([x0, h00 * scale, z0]);
positions.push([x1, h10 * scale, z0]);
positions.push([x1, h11 * scale, z1]);
positions.push([x0, h01 * scale, z1]);

// Normals now need to be computed from the actual triangle geometry:
let v0 = Vec3::new(x0, h00 * scale, z0);
let v1 = Vec3::new(x1, h10 * scale, z0);
let v2 = Vec3::new(x1, h11 * scale, z1);
let v3 = Vec3::new(x0, h01 * scale, z1);

let normal = ((v1 - v0).cross(v3 - v0)).normalize();
normals.extend_from_slice(&[normal.to_array(); 4]);
```

**Important consideration:** Moving to 3D terrain affects many systems:
- Building placement needs to check that the building footprint is not too sloped
- Road rendering needs to follow the terrain surface
- Citizens walk along the terrain surface (y-position depends on ground height)
- Camera system needs to account for terrain height
- Water rendering needs to handle water at a specific Y level

This is a significant architectural change. The recommendation is to implement it incrementally:
1. First, use elevation only for vertex colors (current approach, enhanced)
2. Then, add subtle Y displacement (multiply elevation by a small factor like 2.0 for gentle hills)
3. Finally, full 3D terrain with all dependent systems updated

#### Texture Splatting (Future Enhancement)

Instead of vertex colors, real terrain renderers use texture splatting: blending between multiple terrain textures (grass, rock, sand, snow) based on elevation, slope, and biome. This requires:

1. A set of tileable terrain textures (albedo, normal, roughness)
2. A splat map (per-cell weights for each texture layer)
3. A custom terrain shader that samples and blends the textures

```rust
// Splat map generation (4 channels = 4 texture layers)
struct SplatMap {
    weights: Vec<[f32; 4]>,  // [grass, rock, sand, snow] per cell
    width: usize,
    height: usize,
}

fn generate_splat_map(
    heightmap: &[f32],
    slopes: &[f32],
    moisture: &[f32],
    biome: &[Biome],
    width: usize,
    height: usize,
) -> SplatMap {
    let total = width * height;
    let mut weights = vec![[0.0f32; 4]; total];

    for i in 0..total {
        let elev = heightmap[i];
        let slope = slopes[i];
        let moist = moisture[i];

        // Channel 0: Grass (dominant on flat, moderate-elevation land)
        weights[i][0] = (1.0 - slope * 15.0).clamp(0.0, 1.0)
            * (1.0 - (elev - 0.5).abs() * 3.0).clamp(0.0, 1.0);

        // Channel 1: Rock (steep slopes, high elevation)
        weights[i][1] = (slope * 20.0).clamp(0.0, 1.0)
            + ((elev - 0.7) * 3.0).clamp(0.0, 0.5);

        // Channel 2: Sand (near water, low elevation)
        weights[i][2] = ((WATER_THRESHOLD + 0.03 - elev) / 0.03).clamp(0.0, 1.0)
            * (1.0 - slope * 10.0).clamp(0.0, 1.0);

        // Channel 3: Snow (very high elevation in cold biomes)
        weights[i][3] = ((elev - 0.85) * 5.0).clamp(0.0, 1.0);

        // Normalize so weights sum to 1.0
        let sum: f32 = weights[i].iter().sum();
        if sum > 0.0 {
            for w in &mut weights[i] {
                *w /= sum;
            }
        }
    }

    SplatMap { weights, width, height }
}
```

This is a future enhancement. The current vertex-color approach is adequate for the game's art style and avoids shader complexity.

---

## 9. Reference Games

### 9.1 SimCity 4 (2003)

SimCity 4's terrain system was remarkably advanced for its era and remains a useful reference.

**Region-based terrain:** SC4 used a region map divided into city tiles (small, medium, large). Terrain was continuous across the region, with players editing terrain before starting a city. The region heightmap was a low-resolution grid (~257x257 vertices for a region) that individual city tiles inherited and could modify locally.

**Terrain editor:** Players had access to terrain tools (raise, lower, smooth, flatten, create mountain, create valley) BEFORE placing any buildings. This is a key design insight: terrain editing is a creative act separate from city building. Many players spent hours sculpting terrain before building anything.

**God Mode terraforming:** Available before city founding but disabled (or very expensive) after. This prevented players from trivially flattening terrain to avoid construction challenges.

**Slope restrictions:** Buildings required mostly flat terrain. Roads could handle moderate slopes but not steep ones. The game would automatically level terrain slightly when placing buildings (auto-grade), which reduced frustration but made terrain feel less impactful.

**What to learn from SC4:**
- Pre-game terrain editing is beloved by players -- consider offering it
- Auto-grading buildings reduces frustration but also reduces terrain's impact on gameplay
- Region-scale terrain continuity makes the world feel larger than any single city
- Terrain editing tools should feel like sculpting, not programming

**What to avoid:**
- SC4's terrain was essentially flat during gameplay (buildings flattened everything). Megacity should preserve terrain character after building.
- The terraform tools were mode-based (enter God Mode, edit terrain, exit). Consider allowing incremental terraforming during gameplay at cost.

### 9.2 Cities: Skylines (2015)

**Heightmap import system:** CS1 allowed importing real-world heightmaps (grayscale PNG images) as terrain. This was enormously popular -- players could recreate their home cities on actual terrain. The heightmap resolution was 1081x1081 for a 17.28km x 17.28km map (16m per pixel, similar to Megacity's cell size).

**Terrain data sources:** Players typically sourced heightmaps from:
- terrain.party (now defunct, used USGS SRTM data at ~30m resolution)
- tangrams.github.io/heightmapper
- Custom QGIS/GDAL exports from public DEM datasets

**Water system:** CS1 had a flow-based water simulation. Water was a fluid that actually flowed downhill, pooled in depressions, and could flood low areas. Rivers were not static terrain features -- they were simulated water particles. This made water feel alive but was computationally expensive and sometimes buggy (rivers could dry up if the source was blocked, floods could be hard to control).

**Procedural generation:** CS1 did NOT have procedural terrain generation at launch. Maps were either handcrafted or imported. The Cities: Skylines 2 (2023) added limited procedural generation but it was widely considered inferior to hand-crafted maps.

**What to learn from CS1:**
- Heightmap import is a must-have feature for enthusiast players
- Real-world terrain at 16m/pixel looks excellent in-game
- Flow-based water is impressive but difficult to get right. Static water with visual flow direction is a reasonable compromise.
- Player-created content (workshop maps) was a huge driver of longevity. Make the map format exportable/importable.

**What to avoid:**
- CS1's terrain was unmodifiable during gameplay. Players wanted terraforming. CS2 added it.
- Water simulation bugs (rivers flowing backwards, infinite water sources, cascading floods) were common complaints

**Heightmap import implementation for Megacity:**

```rust
fn import_heightmap_from_image(
    image_path: &str,
    grid: &mut WorldGrid,
) -> Result<(), String> {
    // Load grayscale PNG
    let img = image::open(image_path)
        .map_err(|e| format!("Failed to load image: {}", e))?;
    let gray = img.to_luma8();

    let (img_w, img_h) = gray.dimensions();

    // Resample to grid size using bilinear interpolation
    for gy in 0..GRID_HEIGHT {
        for gx in 0..GRID_WIDTH {
            // Map grid coordinates to image coordinates
            let ix = gx as f32 / GRID_WIDTH as f32 * img_w as f32;
            let iy = gy as f32 / GRID_HEIGHT as f32 * img_h as f32;

            // Bilinear sample
            let x0 = ix.floor() as u32;
            let y0 = iy.floor() as u32;
            let x1 = (x0 + 1).min(img_w - 1);
            let y1 = (y0 + 1).min(img_h - 1);
            let fx = ix - x0 as f32;
            let fy = iy - y0 as f32;

            let p00 = gray.get_pixel(x0, y0).0[0] as f32 / 255.0;
            let p10 = gray.get_pixel(x1, y0).0[0] as f32 / 255.0;
            let p01 = gray.get_pixel(x0, y1).0[0] as f32 / 255.0;
            let p11 = gray.get_pixel(x1, y1).0[0] as f32 / 255.0;

            let elevation = p00 * (1.0 - fx) * (1.0 - fy)
                          + p10 * fx * (1.0 - fy)
                          + p01 * (1.0 - fx) * fy
                          + p11 * fx * fy;

            let cell = grid.get_mut(gx, gy);
            cell.elevation = elevation;
            cell.cell_type = if elevation < WATER_THRESHOLD {
                CellType::Water
            } else {
                CellType::Grass
            };
        }
    }

    Ok(())
}
```

### 9.3 Banished (2014)

Banished is a small-scale medieval colony survival game with excellent procedural terrain generation, particularly notable for its river placement.

**River quality:** Banished generates rivers that look remarkably natural. They meander through the landscape, widen at low elevations, and create realistic floodplains. The rivers are functional: watermills must be built on rivers, fishing docks require river access, and bridges enable crossing.

**Banished's approach:**
1. Generate base terrain using noise (similar to standard fBm)
2. Select 1-2 river entry points on map edges
3. Trace a path from entry to exit, following a combination of lowest terrain and random perturbation
4. Carve the river channel into the terrain
5. Add tributaries by branching from the main river at random points
6. Place resources (trees, minerals, herbs) based on distance from water and elevation

**What Banished gets right:**
- Rivers are the map's primary feature. They define where the settlement grows, where food comes from, and where trade routes go.
- The river is carved into the terrain AFTER noise generation, not defined by a threshold. This means the river always exists and always has a coherent flow path.
- River width varies naturally (wider in flat areas, narrower in valleys)
- The surrounding terrain is adjusted to create realistic riverbanks

**What to learn from Banished:**
- Carving rivers (Section 3.1 Method 2) produces better results than threshold-based water for small maps
- Rivers should be guaranteed features, not random noise artifacts
- Resources should concentrate near rivers, creating natural settlement locations
- At the 256x256 scale, 1-2 rivers with tributaries is the right density

### 9.4 Dwarf Fortress (2006-present)

Dwarf Fortress has the most complex terrain generation of any game, simulating geological processes over thousands of years.

**Multi-layer geology:** DF generates a full 3D geological column for each map cell:
1. Base rock type (determined by tectonic plate simulation)
2. Sedimentary layers (deposited over time in basins)
3. Igneous intrusions (volcanic activity creates dykes and sills)
4. Metamorphic zones (where heat and pressure transform rock)
5. Soil layers (topsoil, subsoil, clay, sand -- based on parent rock and weathering)
6. Aquifer layers (water-bearing rock formations)

Each layer affects mining output, structural stability, and water flow. DF's fortress maps are 3D voxel grids where you dig through these layers.

**Relevant lessons for Megacity:**

While Megacity does not need DF's full geological simulation, several concepts are valuable:

1. **Soil types matter.** DF's distinction between soil (easy to dig) and rock (hard, requires mining) creates interesting gameplay. Our `SoilType` enum (Section 7.3) captures this at a simpler level.

2. **Aquifers.** Underground water layers affect both resource access (groundwater pumping for water supply) and construction (basements in wet areas are expensive). Our `groundwater.rs` system could be enhanced with terrain-derived aquifer data.

3. **Mineral distribution follows geology.** Iron ore in DF is found in specific rock types, not randomly scattered. Linking resource deposits to soil/rock types creates logical consistency.

4. **Multi-Z-level terrain is compelling but complex.** DF's fully 3D world enables caves, cliffs, and waterfalls. Megacity's 2D grid with an elevation value is a reasonable simplification, but the option to dig underground (basements, subways, tunnels) is a natural extension.

**What to take from DF:**
- The soil type grid is a lightweight version of DF's geological simulation
- Resource placement should follow geological logic (ore in mountains, oil in sedimentary basins)
- Underground layers (aquifers affecting groundwater, rock type affecting construction) add depth without full 3D voxels

**What not to take from DF:**
- Full geological simulation is overkill for a city builder
- DF's world generation takes minutes; we target sub-second
- Z-level complexity conflicts with the overhead city builder perspective

### 9.5 Minecraft (2011-present)

Minecraft pioneered chunk-based procedural terrain generation in games. While the aesthetic is different (voxel vs continuous), many technical lessons apply.

**Chunk-based streaming generation:** Minecraft generates terrain in 16x16 chunks as the player explores. Each chunk is generated independently using the world seed + chunk coordinates, enabling infinite worlds. Key implementation details:
- Chunk generation is done on worker threads, not the main thread
- Chunks near the player are generated at high priority; distant chunks can wait
- Generated chunks are cached (not regenerated when revisiting an area)
- Decoration (trees, ores, structures) is done in a second pass after all neighboring chunks are generated (to handle cross-chunk features)

**Biome selection:** Minecraft uses a multi-noise approach where different noise functions control temperature, humidity, and "weirdness" (terrain shape). The biome at each position is determined by looking up these noise values in a biome parameter table. This is similar to our Whittaker diagram approach (Section 4.2).

**Cave carving:** Minecraft generates caves by running 3D noise (cheese caves) and worm-path carving (spaghetti caves). While we do not need caves, the worm-path technique is analogous to our river carving (Section 3.1 Method 2) -- both trace a path through terrain and remove material.

**What to learn from Minecraft:**
1. **Seed determinism is non-negotiable.** Minecraft's entire community revolves around sharing seeds. Every aspect of generation must be deterministic from the seed.
2. **Decoration pass separation.** Generate base terrain first, then place features (trees, resources, structures) in a separate pass. This matches our pipeline.
3. **Multi-noise biome selection** is more flexible than hardcoded thresholds. Use continuous noise parameters to select biomes, with blending at boundaries.
4. **Chunk boundaries need care.** Features that cross chunk boundaries (rivers, large trees) must be handled correctly. For Megacity, generate on the full grid rather than per-chunk to avoid this issue.
5. **Structure templates** (Minecraft's villages, dungeons) are analogous to our landmark templates (Section 6.4). They can be pre-designed and stamped into generated terrain.

**What not to take from Minecraft:**
- Infinite worlds do not apply (we have a fixed 256x256 grid)
- Voxel representation is not needed
- Minecraft's terrain generation has accumulated 15 years of complexity; our system should be simpler

---

## Appendix A: Complete Generation Pipeline

Summary of the full terrain generation pipeline in execution order:

```
Step  | Function                      | Input                  | Output                  | Time (est.)
------+-------------------------------+------------------------+-------------------------+-----------
1     | generate_heightmap_fbm()      | seed                   | elevation[256*256]      | 3ms
2     | apply_domain_warp()           | elevation, seed        | elevation (warped)      | 3ms
3     | city_terrain_remap()          | elevation              | elevation (redistributed)| <1ms
4     | simulate_erosion()            | elevation, seed        | elevation (eroded)      | 100ms
5     | thermal_erosion()             | elevation              | elevation (smoothed)    | 15ms
6     | smooth_coastline()            | elevation              | elevation (smooth coast)| 2ms
7     | classify_water()              | elevation              | CellType (water/land)   | <1ms
8     | compute_flow_accumulation()   | elevation              | flow_acc[256*256]       | 8ms
9     | extract_rivers()              | flow_acc, elevation    | river paths             | 5ms
10    | detect_lakes()                | elevation              | lake mask               | 10ms
11    | apply_water_to_grid()         | rivers, lakes, elev    | WorldGrid updated       | <1ms
12    | compute_moisture_grid()       | elevation, water_mask  | moisture[256*256]       | 5ms
13    | classify_biomes()             | elevation, moisture    | biome[256*256]          | <1ms
14    | generate_soil_grid()          | elevation, water_dist  | soil_type[256*256]      | 2ms
15    | compute_flood_risk()          | elevation, flow, soil  | flood_risk[256*256]     | 2ms
16    | generate_forest_density()     | elevation, biome       | forest[256*256]         | 3ms
17    | generate_ore_deposits()       | elevation, seed        | ResourceGrid (ore)      | <1ms
18    | generate_fertile_zones()      | elevation, moisture    | ResourceGrid (fertile)  | <1ms
19    | generate_oil_reserves()       | elevation, seed        | ResourceGrid (oil)      | <1ms
20    | generate_fault_lines()        | seed                   | Vec<FaultLine>          | <1ms
21    | validate_map()                | all grids              | MapQualityReport        | 5ms
------+-------------------------------+------------------------+-------------------------+-----------
TOTAL |                               |                        |                         | ~165ms
```

All steps are deterministic given the same seed. The entire pipeline runs in under 200ms, fast enough for interactive "New Game" generation with real-time seed preview.

---

## Appendix B: Configuration Constants

```rust
// Grid dimensions (existing)
pub const GRID_WIDTH: usize = 256;
pub const GRID_HEIGHT: usize = 256;
pub const CELL_SIZE: f32 = 16.0;           // meters per cell
pub const CHUNK_SIZE: usize = 8;            // cells per chunk side

// Terrain generation
pub const TERRAIN_BASE_FREQUENCY: f32 = 0.008;
pub const TERRAIN_OCTAVES: u32 = 6;
pub const TERRAIN_LACUNARITY: f32 = 2.0;
pub const TERRAIN_PERSISTENCE: f32 = 0.45;
pub const DOMAIN_WARP_STRENGTH: f32 = 30.0;
pub const DOMAIN_WARP_FREQUENCY: f32 = 0.006;
pub const DOMAIN_WARP_OCTAVES: u32 = 4;

// Water
pub const WATER_THRESHOLD: f32 = 0.35;     // existing
pub const BEACH_UPPER: f32 = 0.38;
pub const SHALLOW_WATER: f32 = 0.30;
pub const DEEP_WATER: f32 = 0.15;
pub const RIVER_FLOW_THRESHOLD: u32 = 75;  // flow accumulation for river classification

// Erosion
pub const EROSION_PARTICLES: u32 = 150_000;
pub const EROSION_INERTIA: f32 = 0.3;
pub const EROSION_CAPACITY: f32 = 8.0;
pub const EROSION_RATE: f32 = 0.3;
pub const EROSION_DEPOSITION: f32 = 0.3;
pub const EROSION_RADIUS: i32 = 3;
pub const EROSION_EVAPORATION: f32 = 0.01;
pub const EROSION_MAX_LIFETIME: u32 = 100;
pub const THERMAL_ITERATIONS: u32 = 30;
pub const THERMAL_TALUS: f32 = 0.04;
pub const THERMAL_RATE: f32 = 0.4;

// 3D terrain (when enabled)
pub const MAX_TERRAIN_HEIGHT: f32 = 50.0;   // world units (Y-axis)
// At max_elevation_meters = 800m, this means 1 world unit = 16m vertically
// (same as CELL_SIZE horizontally, keeping proportions natural)

// Resources
pub const ORE_DEPOSITS_TARGET: u32 = 5;
pub const ORE_MIN_SPACING: f32 = 40.0;     // cells between deposits
pub const ORE_CLUSTER_RADIUS: f32 = 8.0;
pub const OIL_THRESHOLD: f32 = 0.65;
pub const FERTILE_THRESHOLD: f32 = 0.55;
pub const FOREST_MIN_SPACING: f32 = 1.5;   // cells between trees

// Map validation
pub const MIN_FLAT_FRACTION: f32 = 0.30;
pub const MIN_WATER_FRACTION: f32 = 0.10;
pub const MAX_WATER_FRACTION: f32 = 0.40;
pub const MIN_FLAT_REGION: u32 = 2000;      // contiguous cells
pub const MIN_WATER_ADJACENT: u32 = 200;
pub const MIN_ELEVATION_RANGE: f32 = 0.15;
```

---

## Appendix C: Migration Path from Current Code

The existing `terrain.rs` uses a single-octave OpenSimplex2 pass. Here is the recommended incremental migration path:

**Phase 1: Enhanced Noise (Low Risk)**
- Add fBm parameters to the existing `generate_terrain()` function
- Use `fastnoise-lite`'s built-in `FractalType::FBm`
- Add height distribution remapping
- No new files, no new dependencies
- Result: Better terrain shape, same flat rendering

**Phase 2: Erosion (Moderate Effort)**
- Add `erosion.rs` in `crates/simulation/src/`
- Implement particle-based hydraulic erosion + thermal erosion
- Call erosion from `generate_terrain()` after noise generation
- Result: Natural-looking river valleys and carved terrain

**Phase 3: Water Bodies (Moderate Effort)**
- Enhance water classification beyond simple threshold
- Add flow accumulation and river extraction
- Add `WaterType` metadata to `Cell` struct
- Result: Defined rivers vs lakes vs ocean

**Phase 4: Derived Grids (Moderate Effort)**
- Add soil type, moisture, flood risk, forest density grids
- Add corresponding `Resource` types in Bevy
- Wire up to existing systems (building cost, land value, fire)
- Result: Terrain affects gameplay deeply

**Phase 5: 3D Terrain Rendering (High Effort)**
- Modify `build_chunk_mesh` to use elevation for vertex Y
- Compute proper normals
- Handle LOD seams
- Update building/road placement for terrain surface
- Result: Visual 3D terrain

**Phase 6: Player Terraforming (Moderate Effort)**
- Add terrain modification tools to UI
- Implement undo stack
- Wire up costs to budget system
- Result: Interactive terrain editing during gameplay

Each phase is independently useful and can be shipped separately. Phases 1-3 provide the most visual improvement for the least effort.
