# Environment, Climate, and Natural Systems

## Deep Feature Research for Megacity

This document provides detailed formulas, algorithms, grid-based pseudocode, and design
parameters for all environmental and climate systems in Megacity. Each section is designed
to translate real-world environmental engineering into a 256x256 grid-based city simulation
running on Bevy ECS.

---

## Table of Contents

1. [Pollution Systems](#1-pollution-systems)
   - 1.1 Air Pollution
   - 1.2 Water Pollution
   - 1.3 Noise Pollution
   - 1.4 Soil Contamination
2. [Water Systems](#2-water-systems)
   - 2.1 Water Supply and Demand
   - 2.2 Pressure Zones and Distribution
   - 2.3 Stormwater Management
   - 2.4 Flooding
   - 2.5 Wastewater and Treatment
3. [Energy Grid](#3-energy-grid)
   - 3.1 Energy Demand
   - 3.2 Generation Types
   - 3.3 Grid Balancing and Blackouts
   - 3.4 Energy Storage
   - 3.5 Peak Pricing and Economics
4. [Weather and Seasons](#4-weather-and-seasons)
   - 4.1 Seasonal Cycles
   - 4.2 Heating and Cooling Degree Days
   - 4.3 Seasonal Modifiers
   - 4.4 Extreme Weather Events
   - 4.5 Urban Heat Island Effect
5. [Natural Disasters](#5-natural-disasters)
   - 5.1 Earthquakes
   - 5.2 Floods
   - 5.3 Wildfires
   - 5.4 Tornadoes
   - 5.5 Volcanic Events
   - 5.6 Tsunamis
6. [Waste Management](#6-waste-management)
   - 6.1 Waste Generation
   - 6.2 Collection and Transport
   - 6.3 Landfill Systems
   - 6.4 Recycling
   - 6.5 Waste-to-Energy
   - 6.6 Composting and Organics
7. [Reference Games Analysis](#7-reference-games-analysis)
   - 7.1 Frostpunk
   - 7.2 Anno 2070
   - 7.3 Surviving Mars
   - 7.4 Eco
   - 7.5 Cities: Skylines
   - 7.6 SimCity (2013)
   - 7.7 Banished
8. [ECS Integration Architecture](#8-ecs-integration-architecture)
9. [Performance Considerations](#9-performance-considerations)

---

## 1. Pollution Systems

Pollution is one of the core negative externalities in city building. In Megacity, we model
four distinct pollution domains -- air, water, noise, and soil -- each with its own dispersion
model, source types, health effects, and mitigation strategies. All pollution values are stored
per-cell on the 256x256 grid and updated at configurable tick intervals.

### 1.1 Air Pollution

#### 1.1.1 Overview

Air pollution disperses from point sources (factories, power plants) and area sources (traffic,
residential heating) across the grid using a simplified Gaussian plume model adapted for
discrete cells. Concentration at each cell affects citizen health, land value, and happiness.

#### 1.1.2 Source Strengths by Building Type

Each building type emits a base emission rate Q (arbitrary pollution units per game-tick).
These values are scaled by building level/density and modified by technology upgrades.

| Building Type               | Base Q (units/tick) | Category      | Notes                              |
|-----------------------------|--------------------:|---------------|-------------------------------------|
| Coal Power Plant            |               100.0 | Point source  | Worst emitter, tall stack           |
| Oil Power Plant             |                70.0 | Point source  | Moderate, tall stack                |
| Gas Power Plant             |                35.0 | Point source  | Cleanest fossil fuel               |
| Heavy Industry              |                60.0 | Point source  | Manufacturing, smelting             |
| Light Industry              |                25.0 | Point source  | Assembly, light manufacturing       |
| Waste Incinerator           |                45.0 | Point source  | With scrubbers: 20.0               |
| Chemical Plant              |                55.0 | Point source  | Toxic emissions                     |
| Refinery                    |                80.0 | Point source  | VOCs, particulates                  |
| Dense Commercial (per cell) |                 8.0 | Area source   | HVAC, delivery trucks               |
| Light Commercial (per cell) |                 3.0 | Area source   | Minor HVAC                          |
| High-Density Residential    |                 5.0 | Area source   | Heating, cooking                    |
| Low-Density Residential     |                 2.0 | Area source   | Minimal                             |
| Major Road (per segment)    |                 6.0 | Line source   | Scales with traffic volume          |
| Highway (per segment)       |                12.0 | Line source   | High traffic throughput             |
| Airport                     |                50.0 | Point source  | Jet exhaust, ground vehicles        |
| Seaport                     |                40.0 | Point source  | Ship diesel, cargo handling         |
| Solar Farm                  |                 0.0 | None          | Zero emissions                      |
| Wind Farm                   |                 0.0 | None          | Zero emissions                      |
| Nuclear Plant               |                 0.0 | None          | Zero air emissions (thermal water)  |

**Traffic-based emissions** scale linearly with traffic volume on road segments:

```
Q_road(segment) = base_Q * (traffic_volume / road_capacity)
```

Where `traffic_volume` comes from the existing `RoadSegmentStore` traffic data.

#### 1.1.3 Gaussian Plume Model (Simplified for Grid)

The real Gaussian plume equation for ground-level concentration from a continuous point source is:

```
C(x, y) = Q / (2 * pi * u * sigma_y * sigma_z)
         * exp(-y^2 / (2 * sigma_y^2))
         * exp(-H^2 / (2 * sigma_z^2))
```

Where:
- Q = emission rate (units/s)
- u = wind speed (m/s)
- sigma_y, sigma_z = lateral and vertical dispersion coefficients
- H = effective stack height
- x = downwind distance, y = crosswind distance

**Simplification for Grid:** We reduce this to a 2D radial falloff with wind-directional bias:

```
C(dx, dy) = Q * dispersion_kernel(dx, dy, wind_dir, wind_speed, stack_height)
```

The dispersion kernel combines:
1. **Radial decay**: Inverse-square with distance
2. **Wind bias**: Elongation of the plume downwind
3. **Stack height bonus**: Taller stacks push concentration further from source

**Grid Kernel Function:**

```rust
fn dispersion_kernel(
    dx: i32, dy: i32,           // cell offset from source
    wind_dir: f32,               // radians, 0 = east
    wind_speed: f32,             // 0.0 to 1.0 normalized
    stack_height: f32,           // 0.0 (ground) to 1.0 (tall stack)
) -> f32 {
    let dist_sq = (dx * dx + dy * dy) as f32;
    if dist_sq == 0.0 { return 1.0; }
    let dist = dist_sq.sqrt();

    // Base radial decay (inverse square, clamped)
    let radial = 1.0 / (1.0 + dist_sq * 0.1);

    // Wind directional bias
    let angle_to_cell = (dy as f32).atan2(dx as f32);
    let angle_diff = (angle_to_cell - wind_dir).cos(); // -1 to 1
    let wind_factor = if wind_speed > 0.01 {
        // Upwind: rapid decay. Downwind: extended plume.
        let downwind_stretch = 1.0 + wind_speed * 2.0 * angle_diff.max(0.0);
        let upwind_suppress = 1.0 - wind_speed * 0.7 * (-angle_diff).max(0.0);
        downwind_stretch * upwind_suppress
    } else {
        1.0 // calm: symmetric dispersion
    };

    // Stack height effect: tall stacks reduce near-field concentration
    // but maintain concentration at distance
    let stack_factor = if dist < 3.0 {
        1.0 - stack_height * 0.6 * (1.0 - dist / 3.0)
    } else {
        1.0 + stack_height * 0.2
    };

    (radial * wind_factor * stack_factor).max(0.0)
}
```

#### 1.1.4 Plume Dispersion Algorithm (Grid-Based)

```
ALGORITHM: UpdateAirPollution
FREQUENCY: Every 4 game-ticks (configurable)
DATA: air_pollution_grid[256][256], source_list[], wind_dir, wind_speed

1. DECAY existing pollution:
   FOR each cell (x, y):
       air_pollution_grid[x][y] *= DECAY_RATE  // 0.85 per update cycle
       // Natural atmospheric dissipation

2. For each pollution SOURCE s in source_list:
   // Determine effective radius based on Q
   max_radius = ceil(sqrt(s.Q / MIN_CONCENTRATION_THRESHOLD))
   max_radius = clamp(max_radius, 1, 32)  // cap at 32 cells

   FOR dx = -max_radius to max_radius:
       FOR dy = -max_radius to max_radius:
           target_x = s.x + dx
           target_y = s.y + dy
           IF target_x, target_y out of bounds: CONTINUE

           kernel = dispersion_kernel(dx, dy, wind_dir, wind_speed, s.stack_height)
           contribution = s.Q * kernel

           // Terrain blocking: hills/mountains reduce transmission
           IF terrain_height[target_x][target_y] > terrain_height[s.x][s.y] + 2:
               contribution *= 0.3  // mountain blocking

           // Trees/parks absorb pollution
           IF cell_type[target_x][target_y] == PARK or FOREST:
               contribution *= 0.6  // vegetation filtering

           air_pollution_grid[target_x][target_y] += contribution

3. CLAMP all values to [0.0, MAX_POLLUTION]  // MAX_POLLUTION = 1000.0

4. UPDATE health effects (see 1.1.5)
```

**Performance note:** With potentially hundreds of sources, the naive O(sources * radius^2)
can be expensive. Optimization strategies:

- **Chunk-based updates**: Only recalculate chunks near sources that changed
- **LOD for distant sources**: Large sources use full kernel; small sources use simplified falloff
- **Temporal amortization**: Update 1/4 of the grid per tick (rotating quadrants)
- **Precomputed kernels**: Cache kernel tables for each wind direction octant

#### 1.1.5 Health Effects by Concentration

Air pollution concentration maps to health impacts on citizens within that cell:

| Concentration Range | AQI Equivalent | Label         | Health Effect                           | Modifier          |
|---------------------:|:--------------:|:-------------:|:----------------------------------------|:-------------------|
|           0 -- 50    | 0-50           | Good          | None                                    | health_rate: +0.01 |
|          51 -- 100   | 51-100         | Moderate      | Sensitive groups affected               | health_rate: 0.00  |
|         101 -- 200   | 101-150        | Unhealthy-SG  | Respiratory issues for sensitive        | health_rate: -0.02 |
|         201 -- 400   | 151-200        | Unhealthy     | All citizens affected                   | health_rate: -0.05 |
|         401 -- 700   | 201-300        | Very Unhealthy| Serious health effects                  | health_rate: -0.10 |
|         701 -- 1000  | 301-500        | Hazardous     | Emergency conditions                    | health_rate: -0.20 |

```rust
fn air_pollution_health_modifier(concentration: f32) -> f32 {
    match concentration as u32 {
        0..=50    =>  0.01,
        51..=100  =>  0.00,
        101..=200 => -0.02,
        201..=400 => -0.05,
        401..=700 => -0.10,
        _         => -0.20,
    }
}
```

**Additional effects:**
- **Land value**: Multiplied by `(1.0 - concentration / 2000.0).max(0.5)` -- up to 50% reduction
- **Happiness**: `-0.1` per 100 units of concentration
- **Immigration**: Cities with average AQI > 200 see 30% reduced immigration
- **Tourism**: Pollution > 150 in tourist areas reduces tourism by 40%

#### 1.1.6 Mitigation and Abatement

| Mitigation Measure         | Effect                                  | Cost Factor |
|:---------------------------|:----------------------------------------|:------------|
| Scrubbers on power plants  | -50% source Q                           | 1.5x        |
| Catalytic converters       | -30% road source Q                      | Policy cost  |
| Electric vehicle mandate   | -60% road source Q (phased over 5 yrs)  | Policy cost  |
| Green belt / tree planting | -40% transmission through cell          | Per cell     |
| Emissions cap policy       | -20% all industrial Q                   | -10% profit  |
| Air quality monitoring     | Unlocks per-cell AQI overlay            | $5,000       |
| Wind turbines near source  | No direct effect (but displaces fossil) | N/A          |

#### 1.1.7 Wind Model Integration

Wind direction and speed affect both pollution dispersion and other systems (energy, fire,
temperature). The wind model provides:

```rust
struct WindState {
    direction: f32,      // radians, 0 = east, pi/2 = north
    speed: f32,          // 0.0 (calm) to 1.0 (gale)
    gustiness: f32,      // 0.0 to 1.0, adds randomness
    // Updated every game-hour based on weather system
}
```

Wind changes gradually (max 15 degrees per game-hour in normal weather, up to 90 degrees
during storms). Prevailing wind direction is map-configurable with seasonal variation.

---

### 1.2 Water Pollution

#### 1.2.1 Overview

Water pollution flows through the drainage network following terrain slope. Unlike air
pollution which disperses radially, water pollution follows defined flow paths and accumulates
in low-lying areas, rivers, and lakes. Sources include point discharges (factories, sewage
outfalls) and non-point runoff (agriculture, roads, construction sites).

#### 1.2.2 Point vs Non-Point Sources

**Point Sources** (discrete, identifiable discharge locations):

| Source Type              | Base Pollution (units/tick) | Pollutant Type    |
|:-------------------------|---------------------------:|:------------------|
| Untreated sewage outfall |                       80.0 | Organic/Bacterial |
| Primary treatment plant  |                       32.0 | Organic/Bacterial |
| Secondary treatment      |                       12.0 | Organic/Bacterial |
| Tertiary treatment       |                        4.0 | Organic/Bacterial |
| Heavy industry discharge |                       50.0 | Chemical/Metal    |
| Light industry discharge |                       20.0 | Chemical          |
| Power plant cooling      |                       15.0 | Thermal           |
| Mining operation         |                       45.0 | Acid/Metal        |

**Non-Point Sources** (diffuse, runoff-based):

| Source Type            | Pollution per Cell (units/tick) | Activation          |
|:-----------------------|-------------------------------:|:--------------------|
| Agricultural land      |                            3.0 | During rain events  |
| Construction site      |                            5.0 | During rain events  |
| Paved roads            |                            1.5 | During rain events  |
| Parking lots           |                            2.0 | During rain events  |
| Lawns (fertilized)     |                            1.0 | During rain events  |
| Industrial yard        |                            4.0 | During rain events  |
| Landfill (unlined)     |                            6.0 | Continuous leaching |
| Landfill (lined)       |                            0.5 | Minimal leaching    |

Non-point source pollution activates primarily during rainfall events, with volume proportional
to rainfall intensity and the impervious surface percentage of the contributing area.

```
NPS_load(cell) = base_pollution * rainfall_intensity * (0.3 + 0.7 * imperviousness)
```

#### 1.2.3 Downstream Flow Model

Water pollution follows a simplified flow accumulation model. Each cell has a flow direction
determined by terrain slope (D8 algorithm -- flow to the steepest downhill neighbor).

```
ALGORITHM: PropagateWaterPollution
FREQUENCY: Every 8 game-ticks
DATA: water_pollution_grid[256][256], flow_dir[256][256], terrain[256][256]

PRECOMPUTE (once, on terrain change):
1. For each cell, compute flow_dir = direction of steepest descent among 8 neighbors
   flow_dir[x][y] = argmin over neighbors8(x,y) of terrain_height[nx][ny]
   If no neighbor is lower, cell is a sink (lake/depression)

2. Compute topological sort order (upstream to downstream)
   Use Kahn's algorithm on the flow directed graph
   Store as sorted_cells[] array

EACH UPDATE:
1. ADD source pollution:
   For each point source s:
       water_pollution_grid[s.x][s.y] += s.Q
   For each cell with non-point source (during rain):
       water_pollution_grid[x][y] += NPS_load(cell)

2. PROPAGATE downstream (in topological order):
   For each cell (x,y) in sorted_cells (upstream first):
       // Transfer pollution downstream
       (nx, ny) = neighbor in flow_dir[x][y]
       IF (nx, ny) is valid:
           transfer = water_pollution_grid[x][y] * FLOW_TRANSFER_RATE  // 0.7
           water_pollution_grid[nx][ny] += transfer
           water_pollution_grid[x][y] -= transfer
       ELSE:
           // Sink cell: pollution accumulates (lake, pond)
           // Slower natural decay
           water_pollution_grid[x][y] *= SINK_DECAY  // 0.98

3. NATURAL DECAY:
   For each cell:
       water_pollution_grid[x][y] *= STREAM_DECAY  // 0.90 (dilution)
       IF cell is RIVER or STREAM:
           water_pollution_grid[x][y] *= RIVER_DECAY  // extra 0.95 (oxygenation)

4. CLAMP to [0.0, MAX_WATER_POLLUTION]  // 500.0
```

#### 1.2.4 Treatment Effectiveness

Wastewater treatment removes pollution before discharge. Treatment levels represent
real-world technologies:

| Treatment Level | BOD Removal | TSS Removal | Nutrient Removal | Cost ($/MG)  | Notes                        |
|:----------------|:-----------:|:-----------:|:----------------:|-------------:|:-----------------------------|
| No treatment    | 0%          | 0%          | 0%               | $0           | Raw sewage discharge         |
| Primary         | 30-40%      | 50-65%      | 10%              | $800         | Settling, screening          |
| Secondary       | 85-95%      | 85-95%      | 30%              | $1,500       | Biological treatment (AS)    |
| Tertiary        | 95-99%      | 95-99%      | 80-95%           | $2,500       | Filtration, chemical P removal|
| Advanced (MBR)  | 99%+        | 99%+        | 95%+             | $4,000       | Membrane bioreactor          |

**Simplified game values:**

```rust
fn treatment_effectiveness(level: TreatmentLevel) -> f32 {
    match level {
        TreatmentLevel::None      => 0.00,  // 0% removal
        TreatmentLevel::Primary   => 0.60,  // 60% removal
        TreatmentLevel::Secondary => 0.85,  // 85% removal
        TreatmentLevel::Tertiary  => 0.95,  // 95% removal
        TreatmentLevel::Advanced  => 0.99,  // 99% removal
    }
}

fn treated_discharge(raw_pollution: f32, level: TreatmentLevel) -> f32 {
    raw_pollution * (1.0 - treatment_effectiveness(level))
}
```

**Treatment plant capacity** is measured in millions of gallons per day (MGD). If inflow
exceeds capacity, excess is discharged untreated (Combined Sewer Overflow -- see Section 2.5).

#### 1.2.5 Water Quality Effects

| Water Pollution Level | Label       | Effect on Citizens                    | Effect on Fisheries |
|----------------------:|:------------|:--------------------------------------|:--------------------|
|            0 -- 20    | Pristine    | Health bonus +0.02, tourism bonus     | Full yield          |
|           21 -- 50    | Clean       | No effect                             | Full yield          |
|           51 -- 100   | Moderate    | -0.01 health if drinking source       | -20% yield          |
|          101 -- 200   | Polluted    | -0.05 health, visible discoloration   | -60% yield          |
|          201 -- 350   | Heavily Pol.| -0.10 health, swimming banned         | -90% yield          |
|          351 -- 500   | Toxic       | -0.20 health, water unusable          | Dead zone           |

**Drinking water treatment** can clean water to safe levels, but at increasing cost:
- Source water pollution < 50: Standard treatment ($500/MG)
- Source water pollution 50-150: Enhanced treatment ($1,200/MG)
- Source water pollution 150-300: Advanced treatment ($2,500/MG)
- Source water pollution > 300: Requires desalination or import ($5,000/MG)

#### 1.2.6 Mitigation

| Measure                   | Effect                               | Cost              |
|:--------------------------|:-------------------------------------|:------------------|
| Upgrade treatment plant   | +1 treatment level                   | $50K-200K         |
| Riparian buffer zones     | -40% NPS entering waterway           | Per-cell cost     |
| Constructed wetlands      | -30% pollution passing through       | 4x4 cell facility |
| Stormwater detention      | Delays and filters runoff            | Per-basin         |
| Industrial pretreatment   | -50% industrial discharge            | Policy + cost     |
| Sewer separation          | Eliminates CSO events                | Very expensive    |
| Pervious pavement         | -60% road runoff pollution           | 2x road cost      |

---

### 1.3 Noise Pollution

#### 1.3.1 Overview

Noise pollution is modeled in decibels (dB), which is a logarithmic scale. Unlike chemical
pollution, noise does not accumulate over time -- it is instantaneous. The grid stores the
current noise level at each cell, recalculated periodically based on active sources.

Key acoustic principles for the simulation:
- **Inverse square law**: Sound intensity decreases with the square of distance
- **6 dB rule**: Sound level drops approximately 6 dB for each doubling of distance
- **Logarithmic addition**: Two 60 dB sources produce 63 dB, not 120 dB
- **Barrier attenuation**: Solid objects (buildings, walls) reduce transmission

#### 1.3.2 Source Levels

| Source Type                    | Level (dB) at Source | Radius (cells) | Time Pattern       |
|:-------------------------------|---------------------:|:--------------:|:-------------------|
| Highway / Freeway              |                   85 |             20 | 24h, peak at rush  |
| Major Road (high traffic)      |                   75 |             12 | Daytime peak       |
| Minor Road (low traffic)       |                   60 |              5 | Daytime only       |
| Rail line (train passing)      |                   90 |             25 | Intermittent       |
| Airport (takeoff/landing)      |                  105 |             40 | Daytime, scheduled |
| Heavy Industry                 |                   80 |             15 | Working hours      |
| Light Industry                 |                   65 |              8 | Working hours      |
| Construction Site              |                   85 |             15 | Daytime only       |
| Commercial District (dense)    |                   65 |              5 | Daytime            |
| Nightclub / Entertainment      |                   75 |              8 | Nighttime          |
| Power Plant                    |                   70 |             10 | 24h                |
| Emergency Siren                |                  100 |             30 | Occasional         |
| Park / Nature                  |                   35 |              0 | Ambient (positive) |
| Residential (quiet)            |                   40 |              0 | Ambient baseline   |

#### 1.3.3 Attenuation Model

Sound level at distance from a point source:

```
L(d) = L_source - 20 * log10(d / d_ref) - alpha * d
```

Where:
- L_source = source level in dB
- d = distance in cells
- d_ref = reference distance (1 cell)
- alpha = atmospheric absorption coefficient (0.5 dB/cell for outdoor propagation)

For the game grid, simplified:

```
L_at_cell(dx, dy) = L_source - 6.0 * log2(distance) - 0.5 * distance
```

Where `distance = sqrt(dx^2 + dy^2)`, and `log2(distance) = log10(distance)/log10(2)`.

This gives approximately 6 dB reduction per doubling of distance, plus atmospheric absorption.

#### 1.3.4 Barrier Attenuation

Buildings and terrain features between source and receiver reduce noise:

```
ALGORITHM: CalculateBarrierAttenuation
INPUT: source(sx, sy), receiver(rx, ry), grid

1. Cast a ray from source to receiver using Bresenham's line algorithm
2. For each cell along the ray:
   IF cell contains a building:
       IF building is solid (concrete, brick):
           attenuation += 15.0 dB  // solid wall
       ELSE IF building is lightweight:
           attenuation += 8.0 dB   // wood frame, glass
   IF cell contains a noise barrier (wall):
       attenuation += 12.0 dB      // purpose-built barrier
   IF cell contains dense trees:
       attenuation += 3.0 dB       // vegetation (limited)
   IF terrain elevation difference > 2:
       attenuation += 10.0 dB      // terrain berm/hill

3. RETURN min(attenuation, 40.0)  // cap total barrier effect at 40 dB
```

#### 1.3.5 Logarithmic Addition of Multiple Sources

When multiple noise sources affect a cell, they combine logarithmically:

```
L_total = 10 * log10( sum over all sources i of 10^(L_i / 10) )
```

Practical shortcuts:
- Two equal sources: +3 dB
- Source 10 dB louder dominates (other contributes +0.4 dB)
- Source 20 dB louder: other is negligible

```rust
fn combine_noise_levels(levels: &[f32]) -> f32 {
    let sum: f64 = levels.iter()
        .map(|&l| 10.0_f64.powf(l as f64 / 10.0))
        .sum();
    if sum > 0.0 {
        (10.0 * sum.log10()) as f32
    } else {
        0.0
    }
}
```

#### 1.3.6 Full Noise Grid Algorithm

```
ALGORITHM: UpdateNoiseGrid
FREQUENCY: Every 8 game-ticks (noise is relatively static)
DATA: noise_grid[256][256], noise_sources[]

1. CLEAR noise_grid to ambient baseline (35 dB everywhere)

2. For each noise SOURCE s:
   // Skip inactive sources (nighttime for daytime-only, etc.)
   IF NOT s.is_active(current_hour): CONTINUE

   // Scale source level by activity
   effective_level = s.level * s.activity_factor(current_hour)

   // Determine max propagation radius (where level drops below ambient)
   max_radius = s.radius  // precomputed from source level

   FOR dx = -max_radius to max_radius:
       FOR dy = -max_radius to max_radius:
           rx = s.x + dx
           ry = s.y + dy
           IF out of bounds: CONTINUE

           dist = sqrt(dx*dx + dy*dy) as f32
           IF dist < 0.5: dist = 0.5  // prevent division issues

           // Distance attenuation
           level_at_cell = effective_level - 6.0 * log2(dist) - 0.5 * dist

           // Barrier attenuation (simplified: check 3 points along path)
           barrier_atten = estimate_barrier(s.x, s.y, rx, ry)
           level_at_cell -= barrier_atten

           IF level_at_cell > noise_grid[rx][ry]:
               // For performance, use max instead of logarithmic addition
               // (dominant source approximation -- error < 3 dB)
               noise_grid[rx][ry] = level_at_cell

3. // Optional: proper logarithmic combination for high-accuracy cells
   // (only in cells where multiple sources are within 6 dB of each other)
```

**Performance optimization**: The "dominant source" approximation (using max instead of
logarithmic addition) is accurate to within 3 dB and avoids expensive log/pow operations.
For gameplay purposes, this is acceptable. Only cells near multiple major sources would
benefit from exact logarithmic combination.

#### 1.3.7 Land Value and Health Effects

| Noise Level (dB) | Label         | Land Value Modifier | Health Effect      | Sleep Effect      |
|------------------:|:--------------|:-------------------:|:-------------------|:------------------|
|        35 -- 45   | Quiet         | +10%                | None               | None              |
|        46 -- 55   | Normal        | 0%                  | None               | None              |
|        56 -- 65   | Noticeable    | -10%                | None               | Mild disruption   |
|        66 -- 75   | Loud          | -25%                | Stress +0.05       | Moderate          |
|        76 -- 85   | Very Loud     | -40%                | Stress +0.10, hearing risk | Severe   |
|        86 -- 95   | Painful       | -60%                | Health -0.05/tick  | Impossible        |
|         96+       | Dangerous     | -80%                | Health -0.15/tick  | Impossible        |

**Nighttime penalty**: Noise effects on residential cells are 50% worse between 22:00-06:00
(nighttime noise ordinance context). This means a 70 dB source at night has the same impact
as an 80 dB source during the day.

#### 1.3.8 Mitigation

| Measure                    | Effect                          | Cost              |
|:---------------------------|:--------------------------------|:------------------|
| Noise barrier walls        | -12 dB through barrier cell     | $2,000/cell       |
| Sound-insulated buildings  | -20 dB inside (no outdoor help) | +30% building cost|
| Speed limits               | -5 dB on affected roads         | Policy cost       |
| Truck route restrictions   | -8 dB on restricted roads       | Policy cost       |
| Curfew (nighttime noise)   | Sources off 22:00-06:00         | Policy cost       |
| Tree buffer                | -3 dB per tree row              | $500/cell         |
| Road surface (quiet asphalt)| -3 dB on road                  | +50% road cost    |
| Depressed highway          | -15 dB (below grade)            | 3x highway cost   |
| Zoning buffer              | Separation distance             | Opportunity cost  |

---

### 1.4 Soil Contamination

#### 1.4.1 Overview

Soil contamination is a long-term, persistent form of pollution that accumulates from
industrial activity, waste disposal, chemical spills, and long-term air/water deposition.
Unlike air and noise pollution, soil contamination persists long after the source is removed
(brownfield sites) and requires active remediation.

#### 1.4.2 Contamination Sources

| Source Type                | Rate (units/tick) | Persistence | Spread Rate |
|:---------------------------|------------------:|:-----------:|:------------|
| Heavy industry (active)    |              3.0  | Very high   | 1 cell/year |
| Chemical plant (active)    |              5.0  | Very high   | 2 cells/yr  |
| Gas station (underground)  |              1.0  | High        | 0.5 cell/yr |
| Landfill (unlined)         |              4.0  | Very high   | 1 cell/yr   |
| Landfill (lined)           |              0.2  | Low         | Negligible  |
| Mining operation           |              6.0  | Very high   | 2 cells/yr  |
| Agricultural (pesticides)  |              0.5  | Moderate    | Negligible  |
| Road salt (winter)         |              0.3  | Low         | 0.5 cell/yr |
| Air pollution deposition   |       0.01 * AQI  | Low         | N/A         |

#### 1.4.3 Accumulation and Spread Model

```
ALGORITHM: UpdateSoilContamination
FREQUENCY: Every 30 game-ticks (soil changes slowly)
DATA: soil_contamination[256][256]

1. ADD contamination from active sources:
   For each source s:
       soil_contamination[s.x][s.y] += s.rate

2. SLOW SPREAD to adjacent cells:
   For each cell (x, y) where soil_contamination > SPREAD_THRESHOLD (50.0):
       excess = soil_contamination[x][y] - SPREAD_THRESHOLD
       spread_amount = excess * SPREAD_RATE * 0.01  // very slow
       For each neighbor (nx, ny) in neighbors8(x, y):
           // Downhill spread is faster
           slope_factor = if terrain[x][y] > terrain[nx][ny] { 1.5 } else { 0.5 }
           soil_contamination[nx][ny] += spread_amount * slope_factor / 8.0

3. NATURAL DECAY (extremely slow):
   For each cell:
       soil_contamination[x][y] *= 0.9999  // half-life ~6930 ticks
       // Some pollutants persist for decades in real life

4. CLAMP to [0.0, 500.0]
```

#### 1.4.4 Brownfield Sites

When an industrial building is demolished, the soil contamination remains. These become
**brownfield sites** that:

- Cannot be rezoned for residential until remediated
- Reduce land value by 70-90%
- May contaminate groundwater (see Section 2)
- Require environmental assessment before redevelopment ($10,000)

#### 1.4.5 Remediation

| Method                   | Speed       | Cost/Cell    | Effectiveness | Notes                      |
|:-------------------------|:------------|:-------------|:--------------|:---------------------------|
| Natural attenuation      | 10-50 years | $0           | Slow          | Just waiting               |
| Excavation & disposal    | 1-2 years   | $50,000      | 95%           | Removes soil entirely      |
| Soil vapor extraction    | 2-5 years   | $25,000      | 80%           | For volatile compounds     |
| Bioremediation           | 3-7 years   | $15,000      | 70%           | Microbes break down toxins |
| Phytoremediation         | 5-15 years  | $5,000       | 50%           | Plants extract metals      |
| Containment (cap/barrier)| Immediate   | $20,000      | 90% (stops spread)| Does not remove         |

```rust
struct RemediationProject {
    cell_x: u16,
    cell_y: u16,
    method: RemediationMethod,
    progress: f32,          // 0.0 to 1.0
    duration_ticks: u32,    // total ticks to complete
    effectiveness: f32,     // fraction of contamination removed
    cost_per_tick: f32,
}
```

---

## 2. Water Systems

Water infrastructure is one of the most critical systems in a city. It encompasses supply
(getting clean water to buildings), stormwater (managing rainfall runoff), wastewater
(treating used water), and flood management. All quantities are tracked in gallons or
millions of gallons per day (MGD) for consistency with US engineering standards.

### 2.1 Water Supply and Demand

#### 2.1.1 Per Capita Demand

Average US municipal water demand is approximately 80-100 gallons per capita per day (GPCD)
for residential use, and 150 GPCD when including commercial, industrial, and public uses.
In the game, we use simplified per-building demand:

| Building Type               | Demand (gal/day/unit) | Unit Definition       |
|:----------------------------|----------------------:|:----------------------|
| Low-density residential     |                   200 | Per household (2.5 ppl)|
| Medium-density residential  |                   400 | Per building (8 units) |
| High-density residential    |                 2,000 | Per building (40 units)|
| Small commercial            |                   500 | Per building           |
| Large commercial            |                 3,000 | Per building           |
| Office building             |                 1,500 | Per building           |
| Light industry              |                 5,000 | Per building           |
| Heavy industry              |                20,000 | Per building           |
| Hospital                    |                15,000 | Per facility           |
| School                      |                 3,000 | Per facility           |
| Park (irrigated)            |                 1,000 | Per cell               |
| Fire station                |                 2,000 | Per facility (+ surge) |

**Seasonal adjustment:**
- Summer: x1.3 (irrigation, cooling)
- Winter: x0.8 (no irrigation)
- Heat wave: x1.6 (emergency cooling demand)

```rust
fn water_demand(building: &Building, season: Season, weather: &WeatherState) -> f32 {
    let base = match building.building_type {
        BuildingType::ResidentialLow  => 200.0,
        BuildingType::ResidentialMed  => 400.0,
        BuildingType::ResidentialHigh => 2000.0,
        BuildingType::CommercialSmall => 500.0,
        BuildingType::CommercialLarge => 3000.0,
        BuildingType::Office          => 1500.0,
        BuildingType::IndustryLight   => 5000.0,
        BuildingType::IndustryHeavy   => 20000.0,
        BuildingType::Hospital        => 15000.0,
        BuildingType::School          => 3000.0,
        // ... etc
        _ => 500.0,
    };

    let seasonal = match season {
        Season::Summer => 1.3,
        Season::Winter => 0.8,
        _ => 1.0,
    };

    let weather_mod = if weather.is_heat_wave { 1.6 } else { 1.0 };

    base * seasonal * weather_mod * building.occupancy_fraction()
}
```

#### 2.1.2 Water Sources

| Source Type         | Capacity (MGD) | Cost/MG | Quality  | Vulnerability          |
|:--------------------|:--------------:|--------:|:---------|:-----------------------|
| River intake        |     5 -- 50    |   $200  | Variable | Drought, pollution     |
| Reservoir           |    10 -- 100   |   $150  | Good     | Drought (slow)         |
| Groundwater well    |     1 -- 10    |   $300  | Good     | Depletion, contamination|
| Desalination plant  |     5 -- 30    | $1,500  | Excellent| Energy-intensive       |
| Water import (pipe) |     Variable   | $1,000  | Good     | Political, expensive   |
| Rainwater harvesting|     0.1 -- 1   |    $50  | Fair     | Seasonal               |

**Groundwater depletion model:**

```
groundwater_level[tick] = groundwater_level[tick-1]
    - total_well_extraction
    + natural_recharge_rate * rainfall_factor
    + pervious_surface_recharge

IF groundwater_level < CRITICAL_LEVEL:
    well_capacity *= (groundwater_level / CRITICAL_LEVEL)  // reduced output
IF groundwater_level <= 0:
    wells_dry = true  // emergency
```

Natural recharge rate depends on:
- Rainfall (primary driver)
- Pervious surface area (parks, unpaved areas recharge aquifer)
- Soil type (sandy soils recharge 3x faster than clay)

#### 2.1.3 Water Treatment Plant

Raw water must be treated before distribution. Treatment cost depends on source water quality
(see Section 1.2.5):

```rust
struct WaterTreatmentPlant {
    capacity_mgd: f32,           // max throughput
    current_load_mgd: f32,       // current demand served
    treatment_level: WaterTreatmentLevel,
    source_quality: f32,         // pollution level of source water
    operating_cost_per_mg: f32,  // varies with source quality
    position: (u16, u16),
}

impl WaterTreatmentPlant {
    fn operating_cost(&self) -> f32 {
        let base = match self.treatment_level {
            WaterTreatmentLevel::Standard => 500.0,
            WaterTreatmentLevel::Enhanced => 1200.0,
            WaterTreatmentLevel::Advanced => 2500.0,
        };
        // Dirtier source water costs more to treat
        base * (1.0 + self.source_quality / 200.0)
    }

    fn can_serve(&self) -> bool {
        self.current_load_mgd < self.capacity_mgd
    }
}
```

### 2.2 Pressure Zones and Distribution

#### 2.2.1 Overview

Water distribution requires pressure to push water through pipes. The city is divided into
pressure zones based on elevation. Water flows downhill naturally; uphill delivery requires
pumping stations.

#### 2.2.2 Pressure Zone Model

```
ALGORITHM: CalculatePressureZones
DATA: terrain[256][256], water_supply_points[], pump_stations[]

1. DEFINE pressure zones by elevation bands:
   Zone 0: elevation 0-3   (sea level / lowest)
   Zone 1: elevation 4-7
   Zone 2: elevation 8-11
   Zone 3: elevation 12-15
   Zone 4: elevation 16+   (hilltop)

2. Water towers and treatment plants have a SERVICE_ELEVATION:
   service_elevation = tower_elevation + tower_height
   // Can serve all zones with elevation <= service_elevation

3. For each zone that cannot be served by gravity:
   REQUIRE pump station at zone boundary
   pump_cost = elevation_difference * flow_rate * PUMP_ENERGY_FACTOR
   // Each pump station consumes electricity proportional to lift height

4. Buildings in unserved zones have NO WATER SERVICE
   Effect: Cannot develop beyond basic level, fire risk extreme
```

#### 2.2.3 Pipe Network (Simplified)

Rather than modeling individual pipes, we use a coverage-based approach:

```
water_service_coverage(cell) =
    1.0 IF cell is within SERVICE_RADIUS of a water main AND
        cell elevation is servable by available pressure AND
        total demand in service area < plant capacity
    0.0 OTHERWISE

SERVICE_RADIUS = cells connected to road network with water main
// Water mains follow roads (similar to how power lines follow roads)
```

Buildings without water service:
- Cannot exceed building level 1
- Fire danger increased 3x
- Health penalty: -0.10/tick
- Happiness: -30%
- Land value: -50%

### 2.3 Stormwater Management

#### 2.3.1 The Rational Method

The Rational Method is the standard engineering formula for peak stormwater runoff:

```
Q = C * i * A
```

Where:
- **Q** = peak runoff rate (cubic feet per second, cfs)
- **C** = runoff coefficient (dimensionless, 0.0 to 1.0)
- **i** = rainfall intensity (inches per hour)
- **A** = drainage area (acres)

For our grid, each cell is `CELL_SIZE = 16.0` units, representing approximately 0.1 acres.

#### 2.3.2 Runoff Coefficients by Land Use

| Land Use                    | C (Runoff Coefficient) | Impervious % |
|:----------------------------|:----------------------:|:------------:|
| Natural forest/woods        |                   0.15 |            5 |
| Open parkland               |                   0.20 |           10 |
| Low-density residential     |                   0.40 |           35 |
| Medium-density residential  |                   0.55 |           55 |
| High-density residential    |                   0.70 |           75 |
| Commercial                  |                   0.85 |           90 |
| Industrial                  |                   0.80 |           85 |
| Downtown/CBD                |                   0.95 |           98 |
| Roads (paved)               |                   0.90 |           95 |
| Parking lot                 |                   0.95 |           98 |
| Green roof building         |                   0.40 |           40 |
| Pervious pavement           |                   0.30 |           30 |
| Rain garden / bioswale      |                   0.10 |            5 |
| Water body                  |                   1.00 |          100 |

#### 2.3.3 Stormwater Runoff Algorithm

```
ALGORITHM: CalculateStormwaterRunoff
TRIGGER: Each rainfall event
DATA: rainfall_intensity, cell_land_use[256][256], terrain[256][256]

1. For each cell (x, y):
   C = runoff_coefficient(cell_land_use[x][y])
   A = CELL_AREA_ACRES  // 0.1 acres per cell
   Q_cell = C * rainfall_intensity * A

   // Store runoff volume for this cell
   runoff_volume[x][y] = Q_cell * storm_duration_hours

2. ROUTE runoff downhill (same D8 flow direction as water pollution):
   For each cell in topological order (upstream first):
       (nx, ny) = downstream neighbor
       IF (nx, ny) valid:
           runoff_volume[nx][ny] += runoff_volume[x][y]
       // Accumulation: downstream cells receive all upstream runoff

3. CHECK for flooding (see 2.4):
   For each cell:
       IF runoff_volume[x][y] > drainage_capacity[x][y]:
           flood_depth[x][y] = (runoff_volume[x][y] - drainage_capacity[x][y])
                               / CELL_AREA
```

#### 2.3.4 Drainage Infrastructure

| Infrastructure Type     | Capacity (cfs) | Cost/Cell  | Notes                       |
|:------------------------|:--------------:|-----------:|:----------------------------|
| No drainage (natural)   |            0.5 |         $0 | Natural soil absorption     |
| Basic storm drain       |            5.0 |     $2,000 | Small pipes, curb inlets    |
| Standard storm sewer    |           20.0 |     $8,000 | Full underground system     |
| Large storm sewer       |           50.0 |    $20,000 | Major trunk lines           |
| Retention pond (4x4)    |          200.0 |    $50,000 | Holds water, slow release   |
| Detention basin (6x6)   |          500.0 |   $100,000 | Larger, flood control       |
| Underground cistern     |          100.0 |    $40,000 | Under buildings, reuse      |

#### 2.3.5 Green Infrastructure

Green infrastructure reduces runoff at the source rather than piping it away:

| Type                    | C Reduction | Additional Benefits              | Cost/Cell  |
|:------------------------|:-----------:|:---------------------------------|-----------:|
| Rain garden             |       -0.30 | Pollutant filtering, aesthetics  |     $3,000 |
| Bioswale (along road)   |       -0.25 | Road drainage, linear feature    |     $2,500 |
| Green roof              |       -0.35 | Insulation, habitat, aesthetics  |    $15,000 |
| Pervious pavement       |       -0.50 | Parking/roads, groundwater rech. |     $5,000 |
| Tree canopy             |       -0.15 | Shade, air quality, land value   |       $500 |
| Constructed wetland     |       -0.40 | Water treatment, habitat, tourism|    $25,000 |
| Rainwater harvesting    |       -0.20 | Water supply supplement          |     $4,000 |

```rust
fn effective_runoff_coefficient(base_c: f32, green_infra: &[GreenInfra]) -> f32 {
    let reduction: f32 = green_infra.iter()
        .map(|gi| gi.c_reduction)
        .sum();
    (base_c - reduction).max(0.05)  // minimum 5% always runs off
}
```

#### 2.3.6 Retention/Detention Sizing

Real engineering rule of thumb for detention pond sizing:

```
Volume_required = Q_peak * storm_duration * SAFETY_FACTOR

Where:
  Q_peak = sum of (C * i * A) for all contributing cells
  storm_duration = design storm duration (typ. 1-hour for 10-year storm)
  SAFETY_FACTOR = 1.25

For a 10-year, 1-hour design storm:
  rainfall_intensity = 2.5 inches/hour (typical mid-latitude US)

Example: 100-cell contributing area, average C = 0.7
  Q_peak = 0.7 * 2.5 * 100 * 0.1 = 17.5 cfs
  Volume = 17.5 cfs * 3600 sec * 1.25 = 78,750 cubic feet
  = ~590,000 gallons = ~1.8 acre-feet
```

Game simplification: Each retention cell provides `RETENTION_CAPACITY = 50,000 gallons`
of storage. A 4x4 retention pond provides `16 * 50,000 = 800,000 gallons`.

### 2.4 Flooding

#### 2.4.1 Flood Triggers

Flooding occurs when:
1. Rainfall exceeds drainage capacity (pluvial/surface flooding)
2. River/stream flow exceeds channel capacity (fluvial flooding)
3. Storm surge from coast (coastal flooding)
4. Dam/levee failure (catastrophic flooding)

#### 2.4.2 Flood Depth and Damage

Flood damage follows a well-established depth-damage curve. The US Army Corps of Engineers
publishes depth-damage functions used in real flood risk assessment:

| Flood Depth (ft) | Residential Damage % | Commercial Damage % | Infrastructure Damage % |
|------------------:|:--------------------:|:-------------------:|:-----------------------:|
|              0.0  |                   0% |                  0% |                      0% |
|              0.5  |                  10% |                  8% |                      3% |
|              1.0  |                  20% |                 15% |                      5% |
|              2.0  |                  35% |                 28% |                     12% |
|              3.0  |                  48% |                 40% |                     20% |
|              4.0  |                  58% |                 50% |                     30% |
|              6.0  |                  72% |                 65% |                     45% |
|              8.0  |                  82% |                 75% |                     60% |
|             12.0  |                  92% |                 88% |                     80% |
|             16.0+ |                  98% |                 95% |                     95% |

```rust
fn flood_damage_fraction(depth_ft: f32, building_type: BuildingCategory) -> f32 {
    let curve = match building_type {
        BuildingCategory::Residential => &RESIDENTIAL_DAMAGE_CURVE,
        BuildingCategory::Commercial  => &COMMERCIAL_DAMAGE_CURVE,
        BuildingCategory::Industrial  => &INDUSTRIAL_DAMAGE_CURVE,
        BuildingCategory::Infrastructure => &INFRA_DAMAGE_CURVE,
    };
    // Linear interpolation on the depth-damage curve
    interpolate_curve(curve, depth_ft)
}

// Damage cost = building_value * flood_damage_fraction
// Citizens in flooded cells: evacuate, health risk, possible death at depth > 4ft
```

#### 2.4.3 Flood Simulation Algorithm

```
ALGORITHM: SimulateFlooding
TRIGGER: When rainfall exceeds drainage OR river overflows
DATA: terrain[256][256], water_level[256][256], drainage[256][256]

1. INITIALIZE:
   For each cell: water_level[x][y] = terrain[x][y]
   // Water level starts at terrain elevation

2. ADD rainfall excess:
   For each cell:
       excess = runoff_volume[x][y] - drainage_capacity[x][y]
       IF excess > 0:
           water_level[x][y] += excess / CELL_AREA  // depth increase

3. FLOW EQUALIZATION (simplified shallow water):
   REPEAT for N_ITERATIONS (5-10):
       For each cell (x, y):
           avg_water_level = average of water_level[neighbors]
           IF water_level[x][y] > avg_water_level:
               transfer = (water_level[x][y] - avg_water_level) * FLOW_RATE
               // Distribute to lower neighbors proportional to slope
               For each neighbor (nx, ny) where water_level[nx][ny] < water_level[x][y]:
                   fraction = slope_to_neighbor / total_slope
                   water_level[nx][ny] += transfer * fraction
                   water_level[x][y] -= transfer * fraction

4. COMPUTE flood depth:
   For each cell:
       flood_depth[x][y] = max(0, water_level[x][y] - terrain[x][y])

5. APPLY damage (see 2.4.2)

6. DRAIN over time:
   For each cell:
       water_level[x][y] -= DRAIN_RATE * drainage_capacity[x][y]
       water_level[x][y] = max(water_level[x][y], terrain[x][y])
```

#### 2.4.4 Flood Mitigation

| Measure                  | Effect                            | Cost           |
|:-------------------------|:----------------------------------|:---------------|
| Levees/flood walls       | Block flooding up to design height| $15,000/cell   |
| Floodplain zoning        | Prevent building in flood zones   | Opportunity cost|
| Channel improvements     | +200% river capacity              | $25,000/cell   |
| Retention basins         | Store excess water                | $50-100K       |
| Flood warning system     | 6-hour warning, allows evacuation | $50,000        |
| Building elevation       | Raise first floor above flood level| +40% build cost|
| Flood insurance program  | Does not prevent but reduces cost | Policy         |
| Wetland preservation     | Natural flood storage             | $2,000/cell    |

### 2.5 Wastewater and Treatment

#### 2.5.1 Wastewater Generation

Wastewater generation is approximately **80% of water consumption** (the other 20% is
consumed by irrigation, evaporation, and industrial processes):

```
wastewater_flow(building) = water_demand(building) * 0.80
```

Total city wastewater flow:

```
total_wastewater_mgd = sum over all buildings of wastewater_flow(building) / 1_000_000
```

#### 2.5.2 Treatment Plant Sizing

Treatment plants are sized in MGD capacity. Rule of thumb: design capacity should be
2x average dry weather flow to handle peak flows and wet weather:

```
design_capacity = average_dry_weather_flow * PEAKING_FACTOR  // 2.0-2.5

// For a city of 100,000 people at 120 GPCD effective wastewater:
// 100,000 * 120 / 1,000,000 = 12 MGD average
// Design: 12 * 2.5 = 30 MGD plant
```

| Plant Size (MGD) | Footprint (cells) | Construction Cost | Operating Cost/Day |
|------------------:|:-----------------:|------------------:|-------------------:|
|               1.0 |               2x2 |          $500,000 |             $1,500 |
|               5.0 |               3x3 |        $2,000,000 |             $6,000 |
|              10.0 |               4x4 |        $4,000,000 |            $10,000 |
|              25.0 |               5x5 |        $8,000,000 |            $20,000 |
|              50.0 |               6x6 |       $15,000,000 |            $35,000 |

#### 2.5.3 Combined Sewer Overflow (CSO)

Many older cities have **combined sewers** that carry both wastewater and stormwater in the
same pipes. During heavy rain, the combined flow exceeds treatment plant capacity, and the
excess (a mix of sewage and stormwater) is discharged directly to waterways untreated.

```
ALGORITHM: CombinedSewerOverflow
DATA: wastewater_flow, stormwater_flow, plant_capacity

combined_flow = wastewater_flow + stormwater_flow

IF combined_flow <= plant_capacity:
    treated_flow = combined_flow
    overflow = 0
    water_pollution_discharge = treated_flow * (1.0 - treatment_effectiveness)
ELSE:
    treated_flow = plant_capacity
    overflow = combined_flow - plant_capacity
    // Overflow is partially diluted stormwater + raw sewage
    overflow_concentration = 0.3  // 30% of raw sewage concentration
    water_pollution_discharge = treated_flow * (1.0 - treatment_effectiveness)
                              + overflow * overflow_concentration
    // CSO EVENT: notification to player, environmental penalty
```

**Sewer separation** (building separate storm and sanitary sewers) eliminates CSO but is
extremely expensive -- typically $50,000-100,000 per cell of served area.

#### 2.5.4 Sewer Service Coverage

Similar to water supply, sewer service follows roads:

```
sewer_service(cell) =
    1.0 IF cell is connected to road network with sewer main AND
        downstream treatment plant has capacity
    0.0 OTHERWISE

// Buildings without sewer:
// - Require septic system (only for low-density, fails in high water table)
// - Septic failure: groundwater contamination
// - Cannot develop beyond low density
```

#### 2.5.5 Resource Recovery

Modern wastewater treatment can recover valuable resources:

| Resource                | Recovery Method      | Value           | Requires       |
|:------------------------|:---------------------|:----------------|:---------------|
| Biogas (methane)        | Anaerobic digestion  | Energy for plant| Secondary+     |
| Reclaimed water         | Tertiary treatment   | Irrigation use  | Tertiary       |
| Biosolids (fertilizer)  | Dewatering + drying  | Agricultural use| Secondary+     |
| Phosphorus              | Chemical precipitation| Fertilizer     | Advanced       |
| Heat energy             | Heat exchangers      | District heating| Any level      |

A treatment plant with resource recovery can offset 30-60% of its operating costs.

---

## 3. Energy Grid

The energy system models electricity generation, distribution, demand balancing, and the
economic/environmental trade-offs of different power sources. Supply must match demand at
all times; failure causes rolling blackouts that cascade through the city.

### 3.1 Energy Demand

#### 3.1.1 Base Demand by Building Type

Energy demand is measured in kilowatt-hours (kWh) per game-day. Each building type has a
base demand that varies by season (heating/cooling) and time of day.

| Building Type               | Base Demand (kWh/day) | Peak Hour Factor | Notes                    |
|:----------------------------|----------------------:|-----------------:|:-------------------------|
| Low-density residential     |                    30 |              1.8 | Per household             |
| Medium-density residential  |                   200 |              1.6 | Per building (8 units)    |
| High-density residential    |                   800 |              1.5 | Per building (40 units)   |
| Small commercial            |                   150 |              2.0 | AC heavy in summer        |
| Large commercial            |                 1,500 |              2.2 | Shopping center           |
| Office building             |                   800 |              2.5 | Daytime peak              |
| Light industry              |                 2,000 |              1.3 | Fairly constant           |
| Heavy industry              |                 8,000 |              1.2 | Very constant, high base  |
| Hospital                    |                 5,000 |              1.4 | Critical: cannot lose     |
| School                      |                   400 |              1.5 | Daytime only              |
| Water treatment plant       |                 3,000 |              1.1 | Constant, critical        |
| Wastewater treatment plant  |                 4,000 |              1.1 | Constant, critical        |
| Street lighting (per cell)  |                     5 |              1.0 | Nighttime only            |
| Electric rail (per segment) |                   200 |              2.0 | Rush hour peak            |
| EV charging station         |                   500 |              1.8 | Evening peak              |
| Data center                 |                15,000 |              1.1 | Very constant, cooling    |
| Stadium/arena               |                 2,000 |              5.0 | Event-driven spikes       |

#### 3.1.2 Time-of-Day Demand Profile

Electricity demand follows a characteristic daily curve:

```
Hour    Residential  Commercial  Industrial  Overall
00:00      0.5          0.2         0.8        0.5
04:00      0.4          0.1         0.8        0.4
06:00      0.8          0.3         0.9        0.6
08:00      0.9          0.8         1.0        0.9
10:00      0.7          1.0         1.0        0.9
12:00      0.8          1.0         1.0        0.9
14:00      0.7          1.0         1.0        0.9
16:00      0.8          0.9         1.0        0.9
18:00      1.0          0.6         0.9        0.8   // residential peak
20:00      1.0          0.3         0.8        0.7
22:00      0.8          0.2         0.8        0.6
```

```rust
fn demand_time_factor(hour: u8, sector: Sector) -> f32 {
    // Lookup from precomputed curves
    let curve = match sector {
        Sector::Residential => &RESIDENTIAL_DEMAND_CURVE,
        Sector::Commercial  => &COMMERCIAL_DEMAND_CURVE,
        Sector::Industrial  => &INDUSTRIAL_DEMAND_CURVE,
    };
    interpolate_hourly(curve, hour as f32)
}

fn total_demand(buildings: &[Building], hour: u8, season: Season, temp: f32) -> f32 {
    buildings.iter().map(|b| {
        let base = b.base_energy_demand();
        let time = demand_time_factor(hour, b.sector());
        let seasonal = seasonal_energy_factor(season, temp, b.sector());
        base * time * seasonal * b.occupancy_fraction()
    }).sum()
}
```

#### 3.1.3 Seasonal Energy Factors

Temperature drives heating and cooling demand:

```rust
fn seasonal_energy_factor(season: Season, temperature_f: f32, sector: Sector) -> f32 {
    let base = 1.0;

    // Heating demand (below 65F)
    let heating = if temperature_f < 65.0 {
        (65.0 - temperature_f) * 0.02  // 2% per degree below 65F
    } else { 0.0 };

    // Cooling demand (above 75F)
    let cooling = if temperature_f > 75.0 {
        (temperature_f - 75.0) * 0.03  // 3% per degree above 75F (AC expensive)
    } else { 0.0 };

    let hvac_factor = match sector {
        Sector::Residential => 1.0,
        Sector::Commercial  => 1.3,  // more AC-dependent
        Sector::Industrial  => 0.3,  // less temperature-sensitive
    };

    base + (heating + cooling) * hvac_factor
}
```

### 3.2 Generation Types

#### 3.2.1 Power Plant Specifications

| Plant Type         | Capacity (MW) | Cost ($M) | O&M ($/MWh) | CO2 (t/MWh) | Uptime % | Build Time | Footprint |
|:-------------------|:-------------:|:---------:|:----------:|:-----------:|:--------:|:----------:|:---------:|
| Coal               |     200-600   |   2.0-4.0 |       30   |       0.95  |    85    |  4 years   |    4x4    |
| Natural Gas (CC)   |     100-400   |   1.0-2.0 |       25   |       0.40  |    90    |  2 years   |    3x3    |
| Natural Gas (peak) |      50-150   |   0.3-0.8 |       50   |       0.55  |    95    |  1 year    |    2x2    |
| Nuclear             |    800-1200   |   8.0-12  |       12   |       0.00  |    92    |  8 years   |    6x6    |
| Solar Farm          |      10-100   |   0.8-2.0 |        5   |       0.00  |    25    |  1 year    |    4x8    |
| Wind Farm           |      20-200   |   1.5-3.0 |        8   |       0.00  |    35    |  2 years   |    6x6    |
| Hydroelectric       |     50-500    |   3.0-8.0 |        4   |       0.00  |    50    |  5 years   |    4x4    |
| Biomass             |      10-50    |   0.5-1.5 |       40   |       0.10  |    80    |  2 years   |    3x3    |
| Geothermal          |      20-100   |   2.0-5.0 |        6   |       0.00  |    95    |  3 years   |    3x3    |
| Tidal               |      5-50     |   2.0-4.0 |       10   |       0.00  |    30    |  3 years   |    2x4    |
| Waste-to-Energy     |      10-50    |   0.5-2.0 |       45   |       0.30  |    85    |  3 years   |    3x3    |
| Oil                 |    100-300    |   1.0-2.0 |       45   |       0.75  |    88    |  2 years   |    3x3    |
| Diesel Generator    |       1-10    |   0.01-0.1|       80   |       0.80  |    95    | Immediate  |    1x1    |

#### 3.2.2 Variable Generation (Renewables)

Solar and wind output varies with time and weather:

**Solar generation:**
```
solar_output(hour, season, weather) =
    capacity * solar_irradiance(hour, season) * cloud_factor(weather) * panel_efficiency

solar_irradiance(hour, season):
    // Bell curve peaking at solar noon
    peak_hour = 12.0 + seasonal_offset  // earlier in summer
    daylight_hours = season.daylight_hours()  // 9-15 hours
    if abs(hour - peak_hour) > daylight_hours / 2: return 0.0
    return cos((hour - peak_hour) * PI / daylight_hours)^2

cloud_factor:
    Clear:          1.0
    Partly cloudy:  0.7
    Overcast:       0.3
    Heavy rain:     0.15
    Snow:           0.1 (but reflection can help: 0.2 with snow on ground)

Seasonal peak capacity factor:
    Summer: 0.28 (6-7 peak sun hours)
    Spring: 0.22
    Autumn: 0.18
    Winter: 0.12 (4-5 peak sun hours, low angle)
```

**Wind generation:**
```
wind_output(wind_speed) =
    0                           if wind_speed < CUT_IN  (0.15)
    capacity * cubic_ramp       if CUT_IN <= wind_speed < RATED  (0.15-0.55)
    capacity * 1.0              if RATED <= wind_speed < CUT_OUT  (0.55-0.90)
    0                           if wind_speed >= CUT_OUT  (0.90)

cubic_ramp = ((wind_speed - CUT_IN) / (RATED - CUT_IN))^3
// Wind power proportional to cube of wind speed

Average capacity factor by season:
    Spring: 0.38 (windiest)
    Winter: 0.35
    Autumn: 0.32
    Summer: 0.28 (calmest)
```

#### 3.2.3 Fuel Costs and Economics

| Fuel Type     | Cost per MWh (fuel only) | Price Volatility | Supply Risk    |
|:--------------|-------------------------:|-----------------:|:---------------|
| Coal          |                      $20 | Low              | Moderate       |
| Natural Gas   |                      $15 | High (2x swings) | Import-dependent|
| Nuclear (U)   |                       $5 | Very low         | Low            |
| Oil           |                      $35 | Very high        | Import-dependent|
| Biomass       |                      $25 | Moderate         | Local supply   |
| Solar/Wind    |                       $0 | None             | None           |
| Hydro         |                       $0 | None             | Drought risk   |

**Levelized Cost of Energy (LCOE)** -- total cost including capital, O&M, fuel:

```
LCOE = (Capital_Cost * CRF + Annual_O&M + Annual_Fuel) / Annual_Generation

Where CRF (Capital Recovery Factor) = r(1+r)^n / ((1+r)^n - 1)
r = discount rate (8%), n = plant lifetime (20-60 years)
```

| Plant Type      | LCOE ($/MWh) | Lifetime (years) |
|:----------------|:------------:|:-----------------:|
| Coal            |        65-80 |                40 |
| Natural Gas CC  |        45-65 |                30 |
| Nuclear         |        70-100|                60 |
| Solar           |        30-50 |                25 |
| Wind            |        35-55 |                25 |
| Hydro           |        40-60 |                80 |

### 3.3 Grid Balancing and Blackouts

#### 3.3.1 Supply-Demand Balance

The fundamental constraint: **supply must equal demand at all times**.

```
ALGORITHM: GridBalance
FREQUENCY: Every game-hour
DATA: generators[], total_demand, grid_storage

1. COMPUTE total demand (sum of all building demands with modifiers)
   demand = total_demand(buildings, hour, season, temperature)

2. DISPATCH generation (merit order: cheapest first):
   Sort generators by marginal cost (fuel + variable O&M)
   supply = 0
   For each generator g in merit order:
       IF g.is_available():  // not under maintenance, weather permits
           dispatch = min(g.available_capacity(), demand - supply)
           g.current_output = dispatch
           supply += dispatch
       IF supply >= demand: BREAK

3. CHECK battery storage:
   IF supply > demand:
       excess = supply - demand
       charge_amount = min(excess, storage.charge_rate, storage.remaining_capacity)
       storage.charge(charge_amount)
       curtailment = excess - charge_amount  // wasted renewable energy
   ELSE IF supply < demand:
       deficit = demand - supply
       discharge = min(deficit, storage.discharge_rate, storage.stored_energy)
       storage.discharge(discharge)
       supply += discharge

4. CHECK final balance:
   IF supply >= demand:
       grid_frequency = 60.0  // stable
       reserve_margin = (supply - demand) / demand
   ELSE:
       deficit_fraction = (demand - supply) / demand
       TRIGGER blackout_cascade(deficit_fraction)
```

#### 3.3.2 Blackout Cascade Model

When supply falls short, load must be shed. This cascades through the grid:

```
ALGORITHM: BlackoutCascade
INPUT: deficit_fraction (0.0 to 1.0)

1. CLASSIFY loads by priority:
   Priority 1 (CRITICAL): Hospitals, fire stations, water/wastewater plants, emergency
   Priority 2 (ESSENTIAL): Government, police, schools
   Priority 3 (IMPORTANT): Residential
   Priority 4 (STANDARD): Commercial
   Priority 5 (DEFERRABLE): Industrial, entertainment, stadiums

2. SHED load from lowest priority first:
   remaining_deficit = deficit_fraction * total_demand
   For priority = 5 downto 1:
       loads_in_priority = sum of demand in this priority
       IF remaining_deficit > 0:
           shed_fraction = min(1.0, remaining_deficit / loads_in_priority)
           // Randomly select buildings in this priority to shed
           For each building b in priority:
               IF random() < shed_fraction:
                   b.has_power = false
                   remaining_deficit -= b.demand
       IF remaining_deficit <= 0: BREAK

3. EFFECTS of no power:
   - Residential: happiness -40%, no heating/cooling, food spoilage
   - Commercial: revenue = 0, workers sent home
   - Industrial: production = 0, restart cost after restoration
   - Hospital without power: death rate +500% (emergency generators last 8 hours)
   - Water plant without power: water service interrupted in 4 hours
   - Traffic signals: accident rate +300%
   - Street lights: crime rate +200%

4. CASCADING EFFECTS:
   // Power loss can cause further failures
   IF water_plant.has_power == false for > 4 hours:
       water_service_fails -> fire suppression fails -> fire risk extreme
   IF wastewater_plant.has_power == false for > 8 hours:
       raw_sewage_discharge -> water_pollution_spike
```

#### 3.3.3 Grid Reliability Metrics

| Metric                  | Definition                                | Target      |
|:------------------------|:------------------------------------------|:------------|
| Reserve margin          | (capacity - peak demand) / peak demand    | > 15%       |
| SAIDI                   | Avg. minutes of outage per customer/year  | < 120 min   |
| SAIFI                   | Avg. number of outages per customer/year  | < 1.5       |
| LOLP                    | Probability of insufficient supply        | < 0.1%      |
| Capacity factor         | Actual generation / max possible          | Varies      |

**Player feedback:**
- Reserve margin < 15%: Yellow warning "Power grid strained"
- Reserve margin < 5%: Red warning "Rolling blackouts imminent"
- Reserve margin < 0%: Blackouts active
- SAIDI > 200 min/year: Happiness penalty, business flight

### 3.4 Energy Storage

#### 3.4.1 Storage Technologies

| Storage Type          | Capacity (MWh) | Power (MW) | Efficiency | Cost ($M) | Lifetime  | Response |
|:----------------------|:--------------:|:----------:|:----------:|:---------:|:---------:|:--------:|
| Lithium-ion battery   |       4-100    |    1-25    |     90%    |  0.3-5.0  | 15 years  | Instant  |
| Pumped hydro          |    1000-5000   |  100-500   |     80%    | 50-200    | 80 years  | Minutes  |
| Compressed air        |     100-500    |   10-50    |     70%    |  5-20     | 40 years  | Minutes  |
| Flywheel              |       1-10     |   1-20     |     85%    |  0.5-2.0  | 20 years  | Instant  |
| Hydrogen (electrolyzer)|    50-500     |   5-50     |     40%    | 10-50     | 20 years  | Hours    |

```rust
struct EnergyStorage {
    technology: StorageTech,
    max_capacity_mwh: f32,     // total energy storage
    current_stored_mwh: f32,   // current charge level
    max_charge_rate_mw: f32,   // max input power
    max_discharge_rate_mw: f32,// max output power
    round_trip_efficiency: f32,// energy in vs energy out
    degradation_rate: f32,     // capacity loss per cycle
    cycles: u32,               // total charge/discharge cycles
}

impl EnergyStorage {
    fn charge(&mut self, mwh: f32) -> f32 {
        let actual = mwh.min(self.max_charge_rate_mw)
                       .min(self.max_capacity_mwh - self.current_stored_mwh);
        self.current_stored_mwh += actual * self.round_trip_efficiency.sqrt();
        actual
    }

    fn discharge(&mut self, mwh: f32) -> f32 {
        let actual = mwh.min(self.max_discharge_rate_mw)
                       .min(self.current_stored_mwh);
        self.current_stored_mwh -= actual;
        self.cycles += 1;
        actual * self.round_trip_efficiency.sqrt()
    }

    fn effective_capacity(&self) -> f32 {
        // Battery degradation over time
        let degradation = 1.0 - (self.cycles as f32 * self.degradation_rate);
        self.max_capacity_mwh * degradation.max(0.5)
    }
}
```

#### 3.4.2 Storage Strategy

```
ALGORITHM: StorageStrategy
FREQUENCY: Hourly

1. FORECAST demand for next 24 hours (based on day-of-week, season, weather)
2. FORECAST renewable generation for next 24 hours (solar/wind)

3. CHARGE storage when:
   - Renewable generation exceeds demand (free energy)
   - Off-peak hours (electricity price low)
   - Storage level < 30% and no deficit expected

4. DISCHARGE storage when:
   - Demand exceeds generation capacity
   - Peak pricing hours (sell at premium)
   - Emergency (any deficit)

5. RESERVE threshold:
   - Always keep 20% stored for emergencies
   - Hospital/critical facilities have dedicated backup batteries
```

### 3.5 Peak Pricing and Economics

#### 3.5.1 Electricity Pricing Model

The game uses time-of-use pricing that affects city revenue and citizen costs:

```
electricity_price(hour, season, reserve_margin) =
    base_rate * time_of_use_multiplier * scarcity_multiplier

base_rate = $0.12 / kWh  (adjustable by player)

time_of_use_multiplier:
    Off-peak (22:00-06:00):     0.6
    Mid-peak (06:00-14:00):     1.0
    On-peak  (14:00-22:00):     1.5

scarcity_multiplier:
    reserve_margin > 20%:      1.0
    reserve_margin 10-20%:     1.2
    reserve_margin 5-10%:      1.5
    reserve_margin 0-5%:       2.0
    reserve_margin < 0%:       3.0  (rolling blackouts + emergency pricing)
```

#### 3.5.2 Revenue and Cost Summary

```rust
struct EnergyEconomics {
    // Revenue
    residential_revenue: f32,    // kWh sold * rate
    commercial_revenue: f32,
    industrial_revenue: f32,

    // Costs
    fuel_costs: f32,             // sum of generator fuel
    maintenance_costs: f32,      // plant O&M
    capital_payments: f32,       // loan payments on plant construction
    transmission_costs: f32,     // grid maintenance

    // Derived
    fn net_energy_income(&self) -> f32 {
        self.total_revenue() - self.total_costs()
    }

    fn average_cost_per_kwh(&self) -> f32 {
        self.total_costs() / self.total_generation_kwh
    }
}
```

#### 3.5.3 Demand Response Programs

Players can implement demand response to reduce peak load:

| Program                    | Peak Reduction | Cost to City | Citizen Impact    |
|:---------------------------|:--------------:|:-------------|:------------------|
| Smart thermostat program   |            8%  | $1M          | Minor comfort     |
| Industrial load shifting   |           12%  | $500K        | Production timing |
| EV managed charging        |            5%  | $300K        | Convenience       |
| Peak pricing signals       |           10%  | $0           | Higher bills      |
| Interruptible service      |           15%  | $2M rebates  | Occasional outage |
| Critical peak rebates      |            7%  | $1M rebates  | Behavior change   |

#### 3.5.4 Transmission and Distribution

Power flows from generators to consumers through the grid. In the game, we simplify this:

```
power_service(cell) =
    1.0 IF cell is within POWER_RANGE of a power line AND
        power line is connected to a generator with available capacity
    0.0 OTHERWISE

POWER_RANGE = 6 cells from a power line (similar to zone influence)

// Power lines follow roads automatically (underground in dense areas)
// High-voltage transmission lines: needed for distant generators
// Transformer substations: required every 20 cells of transmission
```

**Transmission losses**: 2% per 10 cells of distance from generator to consumer.
This incentivizes distributed generation closer to demand centers.

---

## 4. Weather and Seasons

Weather is the most visible environmental system. It drives energy demand, affects
construction, tourism, agriculture, disaster risk, and citizen mood. The weather system
generates realistic patterns with seasonal variation, random events, and climate feedback
from urbanization (heat island effect).

### 4.1 Seasonal Cycles

#### 4.1.1 Season Definition

The game year is divided into four seasons, each lasting a configurable number of game-days
(default: 7 game-days per season = 28 game-days per year).

```rust
#[derive(Clone, Copy, PartialEq)]
enum Season {
    Spring,  // Days 0-6:   Warming, rain, growth
    Summer,  // Days 7-13:  Hot, dry, tourism peak
    Autumn,  // Days 14-20: Cooling, harvest, foliage
    Winter,  // Days 21-27: Cold, snow, heating demand
}

impl Season {
    fn from_day(day: u32) -> Self {
        match (day % 28) / 7 {
            0 => Season::Spring,
            1 => Season::Summer,
            2 => Season::Autumn,
            _ => Season::Winter,
        }
    }

    fn base_temperature_f(&self) -> (f32, f32) { // (min, max)
        match self {
            Season::Spring => (45.0, 68.0),
            Season::Summer => (65.0, 92.0),
            Season::Autumn => (40.0, 65.0),
            Season::Winter => (20.0, 42.0),
        }
    }

    fn daylight_hours(&self) -> f32 {
        match self {
            Season::Spring => 12.5,
            Season::Summer => 15.0,
            Season::Autumn => 11.0,
            Season::Winter =>  9.0,
        }
    }

    fn precipitation_chance(&self) -> f32 {
        match self {
            Season::Spring => 0.40,  // rainy season
            Season::Summer => 0.15,  // dry, but thunderstorms
            Season::Autumn => 0.30,  // moderate
            Season::Winter => 0.35,  // snow/rain
        }
    }
}
```

#### 4.1.2 Climate Zones (Map Presets)

The map's climate zone shifts all seasonal parameters:

| Climate Zone   | Winter Low | Summer High | Rain Pattern    | Snow?  | Notes              |
|:---------------|:----------:|:----------:|:----------------|:------:|:-------------------|
| Temperate      |    20 F    |    90 F    | Even            | Yes    | Default, balanced  |
| Tropical       |    65 F    |   100 F    | Monsoon (summer)| No     | Year-round warm    |
| Arid/Desert    |    35 F    |   115 F    | Very rare       | No     | Extreme heat       |
| Mediterranean  |    40 F    |    95 F    | Winter rain only| Rare   | Dry summers        |
| Continental    |   -10 F    |    85 F    | Even            | Heavy  | Extreme cold       |
| Subarctic      |   -30 F    |    65 F    | Light           | Heavy  | Short summers      |
| Oceanic        |    35 F    |    72 F    | Frequent        | Light  | Mild, cloudy       |

### 4.2 Heating and Cooling Degree Days

#### 4.2.1 Definition

Heating Degree Days (HDD) and Cooling Degree Days (CDD) are standard measures of how
much heating or cooling is needed:

```
HDD(day) = max(0, 65 - T_avg)    // degrees below 65F baseline
CDD(day) = max(0, T_avg - 65)    // degrees above 65F baseline

Where T_avg = (T_max + T_min) / 2 for the day
```

Annual totals drive energy demand:
- US average: ~4,500 HDD, ~1,200 CDD per year
- Minneapolis: ~7,800 HDD, ~700 CDD (heating-dominated)
- Phoenix: ~1,000 HDD, ~4,200 CDD (cooling-dominated)

#### 4.2.2 Energy Impact

```rust
fn daily_hvac_energy_modifier(hdd: f32, cdd: f32) -> f32 {
    // Each HDD adds ~2% heating load to residential base
    let heating_load = hdd * 0.02;
    // Each CDD adds ~3% cooling load (AC less efficient than furnace)
    let cooling_load = cdd * 0.03;
    1.0 + heating_load + cooling_load
}

// Example: 30 HDD day (35F avg, deep winter)
// heating_load = 30 * 0.02 = 0.60 -> 60% more energy
// Example: 20 CDD day (85F avg, hot summer)
// cooling_load = 20 * 0.03 = 0.60 -> 60% more energy
```

#### 4.2.3 Cost to Citizens

```
monthly_heating_cost = HDD_month * floor_area_sqft * fuel_efficiency * fuel_price
                     / 100_000  // BTU normalization

Typical values:
  floor_area = 1,500 sqft (average home)
  fuel_efficiency = 0.80 (80% efficient furnace)
  fuel_price_per_therm = $1.20 (natural gas)

For 900 HDD month (cold January):
  cost = 900 * 1500 * 1.20 / (0.80 * 100_000) = ~$20/day = ~$600/month

For 500 CDD month (hot August):
  electricity: 500 * 1500 * 0.12 / (3.0 * 100_000) = ~$3/day electricity
  But AC coefficient of performance (COP) = 3.0, so actual: ~$9/day = ~$270/month
```

High energy costs reduce citizen disposable income, which affects:
- Happiness: -0.05 per $100/month above baseline
- Commerce: less consumer spending
- Immigration: high utility costs deter newcomers

### 4.3 Seasonal Modifiers

#### 4.3.1 Solar Energy Production

| Season  | Peak Sun Hours | Capacity Factor | Notes                            |
|:--------|:--------------:|:---------------:|:---------------------------------|
| Spring  |            5.5 |           0.22  | Increasing day length            |
| Summer  |            7.0 |           0.28  | Maximum production               |
| Autumn  |            4.5 |           0.18  | Decreasing, but clear skies      |
| Winter  |            3.5 |           0.12  | Short days, low sun angle        |

#### 4.3.2 Tourism

Tourism has strong seasonal patterns:

```rust
fn tourism_seasonal_modifier(season: Season, weather: &WeatherState) -> f32 {
    let base = match season {
        Season::Summer => 1.5,   // peak tourism
        Season::Spring => 1.2,   // good weather, flowers
        Season::Autumn => 1.1,   // foliage tourism
        Season::Winter => 0.6,   // low (unless ski resort)
    };

    // Weather modifiers
    let weather_mod = match weather.condition {
        WeatherCondition::Sunny       => 1.2,
        WeatherCondition::PartlyCloudy => 1.0,
        WeatherCondition::Overcast     => 0.8,
        WeatherCondition::Rain         => 0.5,
        WeatherCondition::Storm        => 0.2,
        WeatherCondition::Snow         => 0.7,  // can be positive for winter tourism
        WeatherCondition::Extreme      => 0.1,
    };

    base * weather_mod
}
```

#### 4.3.3 Construction

Construction is weather-dependent:

| Season    | Construction Speed | Cost Modifier | Notes                         |
|:----------|:------------------:|:-------------:|:------------------------------|
| Spring    |               100% |          1.0  | Optimal                       |
| Summer    |               110% |          1.0  | Long days, fast work          |
| Autumn    |                90% |          1.05 | Shorter days, rain delays     |
| Winter    |                60% |          1.25 | Cold, snow, short days        |

```
construction_progress_per_tick =
    base_rate * season_speed_factor * weather_factor

weather_factor:
    Clear/Sunny:  1.0
    Rain:         0.5  (outdoor work delayed)
    Snow:         0.3  (severe delays)
    Extreme cold: 0.2  (concrete won't cure, equipment fails)
    Storm:        0.0  (work stopped)
```

#### 4.3.4 Agriculture and Growing Season

For cities with agricultural zones:

```
growing_season_active =
    average_temperature > 50F AND
    frost_risk < 10% AND
    NOT Season::Winter

crop_yield_modifier =
    rainfall_adequacy * temperature_suitability * soil_quality * fertilizer_bonus

rainfall_adequacy:
    adequate (20-40 in/year): 1.0
    excess (>40 in/year): 0.8 (flooding, disease)
    deficit (<20 in/year): 0.6 (drought)
    irrigation available: min(1.0, water_supply / crop_demand)
```

### 4.4 Extreme Weather Events

#### 4.4.1 Heat Waves

A heat wave is defined as 3+ consecutive days with temperature > 100F (or 15F above
seasonal average).

**Mortality curve:**
Heat-related mortality follows an exponential curve above a threshold:

```
excess_deaths_per_100k = 0

IF temperature > HEAT_THRESHOLD (95F for acclimatized, 85F for non-acclimatized):
    excess = temperature - HEAT_THRESHOLD
    excess_deaths_per_100k = 0.5 * exp(0.15 * excess)

// At 100F:  0.5 * exp(0.75) = 1.1 per 100K/day
// At 105F:  0.5 * exp(1.50) = 2.2 per 100K/day
// At 110F:  0.5 * exp(2.25) = 4.7 per 100K/day
// At 115F:  0.5 * exp(3.00) = 10.0 per 100K/day

// Vulnerability factors:
IF citizen.age > 65: risk *= 3.0
IF citizen.age < 5:  risk *= 2.0
IF citizen.has_ac == false: risk *= 5.0
IF citizen.is_homeless: risk *= 8.0
IF citizen.works_outdoors: risk *= 2.5
```

**Heat wave effects:**

| Effect                    | Modifier                                    |
|:--------------------------|:--------------------------------------------|
| Energy demand             | +40-80% (AC load)                           |
| Water demand              | +60% (cooling, irrigation)                  |
| Worker productivity       | -20% (outdoor), -5% (indoor with AC)        |
| Road damage               | Pavement buckles at sustained >110F         |
| Rail delays               | Track expansion, speed restrictions          |
| Wildfire risk             | +300% if combined with drought              |
| Mortality (elderly/sick)  | See curve above                             |
| Blackout risk             | Extreme AC demand may exceed grid capacity  |
| Water main breaks         | Thermal expansion stress                    |

**Mitigation:**
- Cooling centers (public buildings open for shelter): -50% mortality
- Green canopy (trees): -5F local temperature per 20% tree coverage
- Light-colored roofs: -3F roof temperature
- Misting stations: -10F perceived temperature in public spaces
- Emergency water distribution: prevents dehydration deaths

#### 4.4.2 Cold Snaps and Pipe Bursts

Cold snaps: 3+ days with temperature < 10F (or 20F below seasonal average).

**Pipe burst probability:**

```
burst_probability_per_mile_per_day(temp_f) =
    IF temp_f > 32: 0.0001  // baseline, very rare
    IF temp_f > 20: 0.001   // freezing, 10x baseline
    IF temp_f > 0:  0.01    // severe, 100x baseline
    IF temp_f > -10: 0.05   // extreme, pipes bursting
    ELSE:           0.10    // catastrophic, mass failures

// For a city with 500 miles of water mains:
// At 15F: 500 * 0.001 = 0.5 breaks/day (manageable)
// At -5F: 500 * 0.01 = 5 breaks/day (strained crews)
// At -15F: 500 * 0.05 = 25 breaks/day (emergency)
// At -25F: 500 * 0.10 = 50 breaks/day (system failure)
```

**Cold snap effects:**

| Effect                    | Modifier                                    |
|:--------------------------|:--------------------------------------------|
| Heating demand            | +80-150% above normal                       |
| Pipe burst rate           | See formula above                           |
| Road damage               | Frost heave, potholes                       |
| Homeless mortality        | Exponential below 0F without shelter        |
| Vehicle failures          | -20% traffic, dead batteries                |
| School closures           | Below -20F, schools close                   |
| Construction              | Halted below 15F                            |
| Natural gas demand        | May exceed pipeline capacity                |

#### 4.4.3 Storm Damage

Storms combine wind, rain, and sometimes hail or lightning:

```rust
struct StormEvent {
    wind_speed: f32,       // 0.0-1.0 normalized (0.5 = 50mph, 1.0 = 100mph)
    rainfall_intensity: f32, // inches/hour
    duration_hours: f32,
    hail: bool,
    lightning: bool,
    tornado_risk: f32,      // 0.0-1.0 probability
}

fn storm_damage_per_cell(storm: &StormEvent, cell: &Cell) -> f32 {
    let wind_damage = if storm.wind_speed > 0.4 {
        // Damage scales with cube of wind speed above threshold
        let excess = storm.wind_speed - 0.4;
        excess * excess * excess * 1000.0
    } else { 0.0 };

    let flood_damage = flood_from_rainfall(storm.rainfall_intensity, storm.duration_hours);

    let hail_damage = if storm.hail {
        match cell.building_type {
            Some(b) if b.has_glass_facade() => 0.15,  // 15% damage to glass
            Some(_) => 0.05,                            // 5% general
            None => 0.0,
        }
    } else { 0.0 };

    let fire_from_lightning = if storm.lightning {
        0.002 * cell.fire_vulnerability()  // 0.2% chance per cell
    } else { 0.0 };

    wind_damage + flood_damage + hail_damage + fire_from_lightning
}
```

**Wind damage thresholds (Beaufort-inspired):**

| Wind Speed (mph) | Category        | Effects                                   |
|------------------:|:---------------|:------------------------------------------|
|          0 -- 30  | Breezy         | No damage                                 |
|         31 -- 45  | Strong         | Minor: signs, small branches              |
|         46 -- 60  | Gale           | Moderate: roof shingles, fences, trees    |
|         61 -- 75  | Storm          | Significant: structural, power outages    |
|         76 -- 95  | Severe Storm   | Major: roof removal, widespread outages   |
|        96 -- 110  | Hurricane-force| Devastating: building failure, flooding   |
|          111+     | Extreme        | Catastrophic: complete destruction         |

#### 4.4.4 Drought

Drought develops slowly over weeks/months of below-average precipitation:

```
drought_index = running_average_rainfall(30_days) / expected_rainfall

IF drought_index > 0.8: Normal
IF drought_index > 0.5: Moderate drought
    - Water restrictions (lawn watering banned): -20% demand
    - Agricultural yield: -30%
    - Fire risk: +100%
IF drought_index > 0.25: Severe drought
    - Mandatory rationing: -40% demand, happiness -20%
    - Agricultural yield: -60%
    - Fire risk: +300%
    - Reservoir levels dropping
IF drought_index <= 0.25: Extreme drought
    - Emergency water imports required
    - Agricultural failure
    - Fire risk: +500%
    - Reservoir < 20%, wells drying up
    - Possible water system failure
```

### 4.5 Urban Heat Island Effect

#### 4.5.1 Overview

The Urban Heat Island (UHI) effect causes cities to be 2-8 degrees C (3.6-14.4 degrees F)
warmer than surrounding rural areas. This is caused by:

1. **Dark surfaces** (asphalt, roofs) absorb solar radiation
2. **Reduced vegetation** (less evapotranspiration cooling)
3. **Waste heat** from buildings, vehicles, industry
4. **Canyon effect** (tall buildings trap heat, reduce wind)
5. **Reduced sky view** (buildings block radiative cooling at night)

#### 4.5.2 UHI Calculation per Cell

```
UHI_increment(cell) =
    surface_heat_factor(cell)
    + vegetation_deficit(cell)
    + waste_heat(cell)
    + canyon_factor(cell)

surface_heat_factor:
    Asphalt/dark roof:    +2.0 F
    Concrete:             +1.5 F
    Light-colored roof:   +0.5 F
    Green roof:           -1.0 F
    Water body:           -2.0 F
    Vegetation:           -1.5 F

vegetation_deficit:
    // Compare cell's green fraction to rural baseline (0.6)
    deficit = max(0, 0.6 - cell.green_fraction)
    UHI_contribution = deficit * 8.0 F  // up to +4.8F at 0% green

waste_heat:
    // Proportional to energy consumption density
    UHI_contribution = energy_demand_density * 0.001  // F per kWh/cell

canyon_factor:
    // Height-to-width ratio of street canyons
    IF avg_building_height > 4 stories:
        H_W_ratio = avg_building_height / street_width
        UHI_contribution = H_W_ratio * 1.5 F  // up to +3F in dense canyons
    ELSE: 0.0
```

#### 4.5.3 Full UHI Grid Algorithm

```
ALGORITHM: CalculateUrbanHeatIsland
FREQUENCY: Every game-day (or when significant building change)
DATA: terrain[256][256], buildings[256][256], uhi_grid[256][256]

1. For each cell (x, y):
   uhi_grid[x][y] = 0.0

   // Surface albedo contribution
   uhi_grid[x][y] += surface_heat_factor(cell_type[x][y])

   // Vegetation deficit
   green = count_green_cells_in_radius(x, y, 3) / total_cells_in_radius
   uhi_grid[x][y] += max(0, 0.6 - green) * 8.0

   // Waste heat
   uhi_grid[x][y] += energy_demand[x][y] * 0.001

   // Canyon effect
   IF building_height[x][y] > 4:
       uhi_grid[x][y] += building_height[x][y] / 3.0

2. SMOOTH the grid (3x3 average) to prevent sharp boundaries

3. APPLY nighttime amplification:
   // UHI effect is 2-3x stronger at night (buildings release stored heat)
   IF is_nighttime:
       uhi_grid *= 2.0

4. FINAL temperature at cell:
   cell_temperature = base_temperature + uhi_grid[x][y]
```

#### 4.5.4 UHI Mitigation

| Measure                   | UHI Reduction | Co-Benefits                    | Cost/Cell   |
|:--------------------------|:-------------:|:-------------------------------|:------------|
| Tree planting             |     -1.5 F    | Air quality, beauty, shade     | $500        |
| Green roofs               |     -2.0 F    | Stormwater, insulation         | $15,000     |
| Cool (white) roofs        |     -1.5 F    | Reduced AC cost                | $3,000      |
| Cool pavement             |     -1.0 F    | Reduced road heat              | $5,000      |
| Parks and open space       |     -3.0 F    | Recreation, drainage, value    | $10,000     |
| Water features (fountains)|     -2.0 F    | Aesthetics, tourism            | $8,000      |
| Permeable surfaces        |     -0.5 F    | Stormwater, groundwater        | $4,000      |
| Building energy efficiency|     -0.5 F    | Lower waste heat               | Retrofit    |
| District cooling          |     -1.0 F    | Centralized efficiency         | $50,000     |

#### 4.5.5 Weather Generation Algorithm

```
ALGORITHM: GenerateWeather
FREQUENCY: Every game-hour (with daily planning)

STATE:
    temperature: f32,
    humidity: f32,           // 0.0-1.0
    cloud_cover: f32,        // 0.0-1.0
    precipitation: f32,      // inches/hour
    wind_speed: f32,         // 0.0-1.0
    wind_direction: f32,     // radians
    condition: WeatherCondition,

DAILY PLANNING (at midnight):
    1. Base temperature from season + random variation (+/- 10F)
    2. Precipitation chance from season
    3. Roll for weather events:
       IF random() < precipitation_chance:
           plan rain/snow event with duration and intensity
       IF random() < 0.02:  // 2% daily chance
           plan extreme event (storm, heat wave start, cold snap start)

HOURLY UPDATE:
    1. Temperature follows diurnal curve:
       T(hour) = T_min + (T_max - T_min) * diurnal_factor(hour)
       diurnal_factor peaks at 15:00, minimum at 06:00

    2. Smoothly transition between conditions:
       temperature += (target_temperature - temperature) * 0.3
       humidity += (target_humidity - humidity) * 0.2
       wind transitions over 2-4 hours

    3. Apply UHI:
       // Per-cell temperature = weather_temperature + uhi_grid[x][y]

    4. Update precipitation:
       IF rain_event_active:
           precipitation = event_intensity * random_variation(0.5, 1.5)
       ELSE:
           precipitation = 0

    5. Determine visual condition:
       IF precipitation > 0.5 AND temperature > 32: HEAVY_RAIN
       IF precipitation > 0 AND temperature > 32: RAIN
       IF precipitation > 0 AND temperature <= 32: SNOW
       IF cloud_cover > 0.8: OVERCAST
       IF cloud_cover > 0.4: PARTLY_CLOUDY
       ELSE: SUNNY
```

---

## 5. Natural Disasters

Natural disasters are high-impact, low-frequency events that test the player's preparedness.
Each disaster type has distinct mechanics for generation, propagation, damage assessment,
and recovery. Disasters can be toggled on/off or frequency-adjusted in game settings.

### 5.1 Earthquakes

#### 5.1.1 Earthquake Generation

Earthquakes originate at a random epicenter with a magnitude drawn from a
Gutenberg-Richter-inspired distribution (more small quakes, fewer large ones):

```
ALGORITHM: GenerateEarthquake
TRIGGER: Random event (configurable frequency: ~1 per 20 game-years)

1. SELECT epicenter:
   - Random cell, biased toward map edges (tectonic boundary)
   - Or fixed fault line defined by map preset

2. DETERMINE magnitude (Richter-like scale):
   // Inverse power law: P(M) ~ 10^(-bM), b ~ 1.0
   uniform_random = random()  // 0.0-1.0
   magnitude = -log10(uniform_random) + 3.0
   // Results: mostly 3.0-5.0, rarely 6.0-7.0, extremely rarely 8.0+
   magnitude = clamp(magnitude, 3.0, 9.0)

3. DETERMINE depth (affects surface intensity):
   depth_km = random_range(5.0, 50.0)
   // Shallow earthquakes (< 10km) cause more surface damage
```

#### 5.1.2 Modified Mercalli Intensity Scale

The Modified Mercalli Intensity (MMI) scale describes shaking intensity at each location,
which determines damage. Intensity decreases with distance from the epicenter:

```
MMI(distance, magnitude, depth) =
    base_intensity(magnitude) - distance_attenuation(distance, depth)
    + site_amplification(soil_type)

base_intensity(M):
    M 3.0: MMI III   (felt indoors)
    M 4.0: MMI V     (felt by all, some damage)
    M 5.0: MMI VI    (slight damage)
    M 6.0: MMI VIII  (moderate to heavy damage)
    M 7.0: MMI IX    (heavy damage)
    M 8.0: MMI X     (extreme damage)
    M 9.0: MMI XII   (total destruction)

distance_attenuation:
    // MMI drops ~1 intensity unit per doubling of distance
    attenuation = 2.5 * log10(distance_km / depth_km)
    // Minimum MMI is I (not felt)

site_amplification:
    Bedrock:          0 (no amplification)
    Firm soil:       +0.5 MMI
    Soft soil:       +1.0 MMI
    Fill/reclaimed:  +1.5 MMI
    Liquefaction zone: +2.0 MMI (see below)
```

For the game grid, convert cell distance to km:

```
distance_km = sqrt(dx^2 + dy^2) * CELL_SIZE_METERS / 1000.0
// With CELL_SIZE = 16.0 representing ~50 meters:
distance_km = sqrt(dx^2 + dy^2) * 0.05
```

#### 5.1.3 Damage Probability by Construction Type

At each MMI level, different construction types have different probabilities of damage states:

| MMI   | Wood Frame   | URM (Brick)  | Reinforced Concrete | Steel Frame  | Seismic-Designed |
|:------|:------------:|:------------:|:-------------------:|:------------:|:----------------:|
|       | %None/%Mod/%Sev/%Col | %N/%M/%S/%C | %N/%M/%S/%C | %N/%M/%S/%C | %N/%M/%S/%C |
| V     | 95/5/0/0     | 90/9/1/0     | 98/2/0/0            | 99/1/0/0     | 100/0/0/0        |
| VI    | 85/13/2/0    | 75/20/5/0    | 95/4/1/0            | 97/3/0/0     | 99/1/0/0         |
| VII   | 70/22/7/1    | 50/30/15/5   | 85/12/3/0           | 90/8/2/0     | 96/3/1/0         |
| VIII  | 45/35/15/5   | 25/30/30/15  | 65/25/8/2           | 75/18/6/1    | 90/8/2/0         |
| IX    | 20/35/30/15  | 5/15/40/40   | 35/35/22/8          | 50/30/15/5   | 75/18/6/1        |
| X     | 5/20/40/35   | 0/5/25/70    | 10/25/40/25         | 25/30/30/15  | 50/30/15/5       |
| XI    | 0/5/30/65    | 0/0/10/90    | 2/10/38/50          | 5/20/40/35   | 25/30/30/15      |
| XII   | 0/0/10/90    | 0/0/0/100    | 0/2/18/80           | 0/5/25/70    | 5/15/40/40       |

Legend: None / Moderate / Severe / Collapse (% probability)

```rust
#[derive(Clone, Copy)]
enum DamageState {
    None,
    Moderate,    // Repairable: 10-30% of building value
    Severe,      // Major repair: 50-80% of building value
    Collapse,    // Total loss, casualties possible
}

fn earthquake_damage(mmi: f32, construction: ConstructionType) -> DamageState {
    let probs = DAMAGE_PROBABILITY_TABLE[mmi as usize][construction as usize];
    let roll = random();
    if roll < probs.none      { DamageState::None }
    else if roll < probs.none + probs.moderate { DamageState::Moderate }
    else if roll < probs.none + probs.moderate + probs.severe { DamageState::Severe }
    else { DamageState::Collapse }
}
```

#### 5.1.4 Secondary Earthquake Effects

- **Fires**: Gas line breaks cause fires (probability increases with MMI)
  - At MMI VII: 5% chance per cell of fire ignition
  - At MMI IX: 20% chance per cell
  - Post-earthquake fires are devastating (broken water mains impede firefighting)

- **Liquefaction**: In soft soil / fill areas, ground becomes fluid-like
  - Buildings sink, tilt, or collapse regardless of construction quality
  - Identified by soil type on the map (configurable per map)

- **Landslides**: On steep terrain (slope > 30 degrees), shaking triggers landslides
  - Cells on steep slopes within 10 MMI units of epicenter roll for landslide
  - Landslide buries downhill cells, destroying buildings

- **Tsunami**: Offshore earthquake M > 7.0 can trigger tsunami (see 5.6)

- **Aftershocks**: After main shock, aftershocks occur for 3-7 game-days
  - Magnitude = main_magnitude - 1.2 (average)
  - Frequency decreases exponentially (Omori's law)

#### 5.1.5 Earthquake Preparedness

| Measure                    | Effect                               | Cost            |
|:---------------------------|:-------------------------------------|:----------------|
| Building codes (seismic)   | All new buildings seismic-designed    | +20% build cost |
| Retrofit program           | Upgrade existing to seismic standard | $10K/building   |
| Early warning system       | 10-60 sec warning, auto gas shutoff  | $500K           |
| Emergency supplies cache   | Faster recovery, fewer casualties    | $200K           |
| Earthquake drills          | -30% casualties                      | $50K/year       |
| Flexible gas lines         | -80% post-quake fire                 | $5K/building    |
| Base isolation (critical)  | Hospitals/fire: immune below MMI IX  | $1M/facility    |

#### 5.1.6 Earthquake Grid Algorithm

```
ALGORITHM: SimulateEarthquake
INPUT: epicenter(ex, ey), magnitude, depth_km

1. For each cell (x, y):
   dx = x - ex
   dy = y - ey
   distance_cells = sqrt(dx*dx + dy*dy)
   distance_km = distance_cells * 0.05  // cell to km conversion

   // Calculate local MMI
   base_mmi = 1.5 * magnitude - 1.0  // rough magnitude-to-MMI
   attenuation = 2.5 * log10(max(1.0, distance_km / depth_km))
   site_amp = soil_amplification(soil_type[x][y])
   local_mmi = clamp(base_mmi - attenuation + site_amp, 1.0, 12.0)
   mmi_grid[x][y] = local_mmi

2. For each cell with a building:
   damage = earthquake_damage(mmi_grid[x][y], building.construction_type)
   APPLY damage to building
   IF damage == Collapse:
       casualties += building.occupants * CASUALTY_RATE  // 5-15%
       building.destroyed = true
   IF damage == Severe:
       casualties += building.occupants * 0.01  // 1%
       building.needs_major_repair = true
   IF damage == Moderate:
       building.needs_repair = true

3. SECONDARY effects:
   For each cell where mmi > 6.0:
       IF random() < fire_probability(mmi):
           ignite_fire(x, y)
   For each cell on soft soil where mmi > 7.0:
       IF random() < 0.3:
           liquefaction(x, y)  // building sinks/collapses
   For each steep cell where mmi > 6.0:
       IF random() < slope_factor * 0.1:
           landslide(x, y)

4. DISRUPT infrastructure:
   // Roads: 10% chance of closure per cell at MMI > VII
   // Water mains: 20% chance of break per cell at MMI > VII
   // Power lines: 15% chance of failure per cell at MMI > VII
   // Bridges: 30% chance of damage at MMI > VIII
```

---

### 5.2 Floods

#### 5.2.1 Flood Event Types

Building on the stormwater system (Section 2.3-2.4), major flood events are distinguished
from routine drainage issues by their scale and source:

| Flood Type      | Source                    | Warning Time | Duration    | Max Depth |
|:----------------|:--------------------------|:-------------|:------------|:----------|
| Flash flood     | Intense localized rain    | 0-1 hours    | 2-6 hours   | 2-6 ft    |
| River flood     | Sustained upstream rain   | 12-48 hours  | Days-weeks  | 4-20 ft   |
| Coastal surge   | Hurricane/storm           | 6-24 hours   | 6-12 hours  | 5-25 ft   |
| Dam break       | Structural failure        | 0-2 hours    | 2-8 hours   | 10-40 ft  |
| Snowmelt flood  | Spring warming            | Days         | Weeks       | 2-8 ft    |
| Urban flood     | Overwhelmed drainage      | 1-3 hours    | 3-12 hours  | 1-4 ft    |

#### 5.2.2 River Flood Model

```
ALGORITHM: RiverFlood
TRIGGER: Upstream rainfall accumulation > flood stage threshold

1. ACCUMULATE upstream rainfall:
   upstream_flow = sum of rainfall * area for all upstream cells
   // Uses the same D8 flow accumulation as stormwater

2. CHECK river stage:
   river_stage = upstream_flow / channel_capacity
   IF river_stage > 1.0:  // overbank
       overflow_volume = (river_stage - 1.0) * channel_flow
       // Spread overflow to floodplain cells

3. FLOODPLAIN inundation:
   Use flood simulation algorithm from 2.4.3
   // Water spreads to cells with elevation < river_stage + bank_elevation

4. DAMAGE assessment:
   Apply depth-damage curves from 2.4.2
   Duration matters: damage increases 20% per day of sustained flooding
   (mold, structural degradation, inventory loss)
```

#### 5.2.3 Flood Zone Mapping

Pre-computed flood risk zones help players plan:

```
ALGORITHM: ComputeFloodZones
RUN: Once at map generation, updated when terrain/levees change

1. For each river/stream cell:
   Simulate 100-year flood (1% annual chance):
       Assume rainfall = 6 inches in 24 hours
       Route all flow to stream network
       Expand water to fill terrain below flood stage

2. Mark cells:
   flood_zone[x][y] = HIGH_RISK  if flooded by 100-year event
   flood_zone[x][y] = MODERATE   if flooded by 500-year event
   flood_zone[x][y] = LOW_RISK   otherwise

3. Policy options:
   - Prohibit building in HIGH_RISK zones (floodplain regulation)
   - Require flood insurance for MODERATE zones
   - Allow development with elevation requirements
```

---

### 5.3 Wildfires

#### 5.3.1 Fire Spread Model

Wildfires spread based on three factors: **fuel**, **wind**, and **slope**. The classic
Rothermel fire spread model is adapted for the grid:

```
spread_rate(cell) = R_0 * phi_w * phi_s * phi_m

Where:
    R_0  = base spread rate from fuel type (cells per tick)
    phi_w = wind factor
    phi_s = slope factor
    phi_m = moisture factor
```

#### 5.3.2 Fuel Types and Base Spread Rates

| Fuel Type            | R_0 (cells/tick) | Intensity | Duration | Notes              |
|:---------------------|:----------------:|:---------:|:--------:|:-------------------|
| Grass (short)        |            0.50  | Low       | Short    | Fast but brief     |
| Grass (tall)         |            0.70  | Medium    | Short    | Prairie fire       |
| Brush/shrub          |            0.35  | High      | Medium   | Intense            |
| Light forest         |            0.20  | Medium    | Long     | Understory fire    |
| Dense forest         |            0.10  | Very high | Very long| Crown fire if dry  |
| Urban (wood frame)   |            0.15  | Very high | Long     | Structure fire     |
| Urban (concrete)     |            0.02  | Low       | Long     | Contents only      |
| Park/maintained      |            0.05  | Low       | Short    | Managed vegetation |
| Bare ground/rock     |            0.00  | None      | N/A      | Firebreak          |
| Water                |            0.00  | None      | N/A      | Firebreak          |
| Road (paved)         |            0.00  | None      | N/A      | Firebreak          |

#### 5.3.3 Wind Factor

Wind dramatically increases fire spread in the downwind direction:

```
phi_w(wind_speed, angle_to_wind) =
    1.0 + wind_speed * 3.0 * max(0, cos(angle_to_wind))

// Directly downwind (cos=1.0):
//   Calm wind (0.1): phi_w = 1.3 (30% faster)
//   Moderate (0.4):  phi_w = 2.2 (120% faster)
//   Strong (0.7):    phi_w = 3.1 (210% faster)
//   Gale (1.0):      phi_w = 4.0 (300% faster)

// Directly upwind (cos=-1.0):
//   phi_w = 1.0 (no speed increase; fire burns upwind slowly)

// Crosswind (cos=0):
//   phi_w = 1.0 (no directional advantage)
```

#### 5.3.4 Slope Factor

Fire spreads faster uphill:

```
phi_s(slope) =
    IF slope > 0 (uphill):
        1.0 + slope * 5.0   // 100% faster per 20% slope
    ELSE (downhill):
        1.0 / (1.0 + abs(slope) * 3.0)  // slower downhill

// slope = (terrain[target] - terrain[source]) / CELL_SIZE
```

#### 5.3.5 Moisture Factor

Fuel moisture reduces fire spread:

```
phi_m(fuel_moisture) =
    IF fuel_moisture > 0.30:  0.0  // too wet to burn (after rain)
    IF fuel_moisture > 0.20:  0.3  // slow
    IF fuel_moisture > 0.10:  0.7  // moderate
    IF fuel_moisture > 0.05:  1.0  // normal
    ELSE:                     1.5  // bone dry, extreme fire behavior

fuel_moisture depends on:
    - Days since last rain (decreases 0.05/day without rain)
    - Humidity (high humidity slows drying)
    - Season (winter = higher base moisture)
    - Temperature (heat dries fuel faster)
```

#### 5.3.6 Wildfire Simulation Algorithm

```
ALGORITHM: SimulateWildfire
DATA: fire_state[256][256], fuel_moisture[256][256], terrain[256][256]
STATES: UNBURNED, BURNING, BURNED_OUT, FIREBREAK

1. IGNITION:
   fire_state[ignition_x][ignition_y] = BURNING
   fire_time[ignition_x][ignition_y] = 0

2. EACH FIRE TICK:
   For each cell (x, y) where fire_state == BURNING:
       fire_time[x][y] += 1

       // Check if fire burns out
       IF fire_time[x][y] > burn_duration(fuel_type[x][y]):
           fire_state[x][y] = BURNED_OUT
           CONTINUE

       // Try to spread to each neighbor
       For each neighbor (nx, ny) in neighbors8(x, y):
           IF fire_state[nx][ny] != UNBURNED: CONTINUE

           // Calculate spread probability
           dx = nx - x
           dy = ny - y
           angle_to_wind = atan2(dy, dx) - wind_direction
           slope = (terrain[nx][ny] - terrain[x][y]) / CELL_SIZE

           R = fuel_spread_rate[nx][ny]    // R_0
           R *= phi_w(wind_speed, angle_to_wind)
           R *= phi_s(slope)
           R *= phi_m(fuel_moisture[nx][ny])

           // Stochastic spread: probability = R (clamped to 0-1)
           IF random() < min(R, 0.95):
               fire_state[nx][ny] = BURNING
               fire_time[nx][ny] = 0

3. APPLY damage:
   For each BURNING or BURNED_OUT cell:
       IF building exists:
           IF building.is_wood_frame:
               building.damage = 1.0  // total loss
           ELSE:
               building.damage = 0.3  // contents/interior
       casualties += building.occupants * (1.0 - evacuation_fraction) * 0.01

4. FIREFIGHTING:
   For each fire station with available units:
       Deploy to nearest BURNING cell
       IF firefighter at cell:
           fire_intensity[x][y] *= 0.5  // slows fire
           spread_probability to neighbors *= 0.3  // containment
       Water supply required: 500 gal/min per fire cell
       IF water_supply_available < required:
           firefighting effectiveness reduced proportionally

5. SMOKE:
   // Generate air pollution from burning cells
   For each BURNING cell:
       air_pollution_source += fire_intensity * 50.0
       // Smoke plume follows wind direction (uses air pollution model)
```

#### 5.3.7 Firebreak Effectiveness

| Firebreak Type        | Width Needed | Stop Probability | Notes                 |
|:----------------------|:------------:|:----------------:|:----------------------|
| Paved road (2 lanes)  |     1 cell   |              70% | Embers can jump       |
| Highway (4+ lanes)    |     2 cells  |              90% | Wide gap              |
| River/water body      |     1 cell   |              95% | Very effective         |
| Cleared/bare ground   |     2 cells  |              85% | No fuel               |
| Irrigated green       |     1 cell   |              75% | Wet vegetation resists |
| Concrete buildings    |     1 cell   |              60% | Stops grass/brush fire |

#### 5.3.8 Wildfire Mitigation

| Measure                    | Effect                          | Cost              |
|:---------------------------|:--------------------------------|:------------------|
| Firebreaks (maintained)    | Stops fire spread               | $1,000/cell/year  |
| Prescribed burns           | Reduces fuel load 80%           | $500/cell         |
| Fire-resistant building    | Reduces structure damage 60%    | +15% build cost   |
| Defensible space           | 100ft cleared around structures | $2,000/building   |
| Fire stations              | Active firefighting             | $100K + staff     |
| Water infrastructure       | Hydrant coverage for fighting   | $5,000/cell       |
| Aerial firefighting        | Drops water/retardant on fire   | $50K/deployment   |
| Evacuation routes          | Reduces casualties 80%          | Road planning     |
| Fire weather monitoring    | Early warning for red flag days | $20K              |

---

### 5.4 Tornadoes

#### 5.4.1 Tornado Generation

Tornadoes are associated with severe thunderstorms. They follow a path across the grid
with a width and intensity determined by the Enhanced Fujita (EF) scale.

```
ALGORITHM: GenerateTornado
TRIGGER: During severe thunderstorm events (spring/summer)

1. DETERMINE EF rating (weighted random):
   EF0: 40% chance (65-85 mph winds)
   EF1: 25% chance (86-110 mph)
   EF2: 20% chance (111-135 mph)
   EF3: 10% chance (136-165 mph)
   EF4:  4% chance (166-200 mph)
   EF5:  1% chance (200+ mph)

2. DETERMINE path:
   start_cell = random edge cell (typically west/southwest)
   direction = prevailing_wind + random(-30, 30) degrees
   path_length = 5 + EF_rating * 10 + random(0, 20) cells  // 5-70 cells
   path_width = 1 + EF_rating cells  // 1-6 cells wide

3. DETERMINE forward speed:
   speed = 2 + random(0, 3) cells per tick  // 2-5 cells per update

4. Path can wobble:
   direction += random(-10, 10) degrees per cell traveled
```

#### 5.4.2 Enhanced Fujita Scale Damage

| EF Rating | Wind (mph)  | Path Width | Damage Description              | Damage % |
|:---------:|:-----------:|:----------:|:--------------------------------|---------:|
| EF0       |   65-85     |   1 cell   | Light: branches, signs           |      5% |
| EF1       |   86-110    |   2 cells  | Moderate: roofs, mobile homes    |     20% |
| EF2       |  111-135    |   3 cells  | Significant: roofs off, trees    |     45% |
| EF3       |  136-165    |   4 cells  | Severe: stories destroyed        |     70% |
| EF4       |  166-200    |   5 cells  | Devastating: well-built homes    |     90% |
| EF5       |   200+      |   6 cells  | Incredible: foundations swept    |     99% |

```rust
fn tornado_damage(ef_rating: u8, building: &Building) -> f32 {
    let base_damage = match ef_rating {
        0 => 0.05,
        1 => 0.20,
        2 => 0.45,
        3 => 0.70,
        4 => 0.90,
        _ => 0.99,
    };

    // Building resilience factor
    let resilience = match building.construction_type {
        ConstructionType::MobileHome    => 0.2,  // very vulnerable
        ConstructionType::WoodFrame     => 0.5,
        ConstructionType::Masonry       => 0.7,
        ConstructionType::ReinforcedConc => 0.85,
        ConstructionType::SteelFrame    => 0.90,
        ConstructionType::Underground   => 0.99,
    };

    // Damage reduced by building quality
    base_damage * (1.0 - resilience * (1.0 - base_damage))
}
```

#### 5.4.3 Tornado Path Simulation

```
ALGORITHM: SimulateTornado
DATA: tornado path parameters from generation

1. INITIALIZE:
   current_x, current_y = start_cell
   remaining_length = path_length

2. For each step:
   // Move tornado forward
   current_x += cos(direction) * speed
   current_y += sin(direction) * speed
   remaining_length -= speed

   IF remaining_length <= 0 OR out_of_bounds: STOP

   // Damage cells in path width
   For wx = -path_width/2 to path_width/2:
       For wy = -path_width/2 to path_width/2:
           tx = round(current_x + wx)
           ty = round(current_y + wy)
           IF out of bounds: CONTINUE

           // Edge of path is weaker
           dist_from_center = sqrt(wx*wx + wy*wy)
           intensity = 1.0 - (dist_from_center / (path_width / 2.0)) * 0.5

           // Apply damage
           IF building at (tx, ty):
               damage = tornado_damage(ef_rating, building) * intensity
               building.apply_damage(damage)
               IF damage > 0.8:
                   debris_generation(tx, ty)  // flying debris hits neighbors
           // Destroy trees, signs, infrastructure
           IF tree at (tx, ty): tree.destroyed = random() < intensity
           IF power_line at (tx, ty): power_outage(tx, ty)

   // Wobble direction
   direction += random(-0.15, 0.15)
```

#### 5.4.4 Tornado Mitigation

| Measure                     | Effect                         | Cost            |
|:----------------------------|:-------------------------------|:----------------|
| Tornado sirens              | 15-min warning, reduces deaths | $200K           |
| Storm shelters (public)     | Saves lives in EF3+            | $50K each       |
| Safe rooms (residential)    | Saves occupants                | $5K/home        |
| Building codes (wind)       | +1 EF level resistance         | +10% build cost |
| Doppler radar               | Better warning (30 min)        | $1M             |
| Underground utilities       | Reduces power outage duration  | 2x utility cost |
| Mobile home restrictions    | Ban in tornado-prone zones     | Policy          |

---

### 5.5 Volcanic Events

#### 5.5.1 Overview

Volcanic events are map-specific (only available on maps with volcanic terrain). They range
from minor ash fall to catastrophic eruptions.

| Event Type       | Frequency     | Effects                                | Range       |
|:-----------------|:--------------|:---------------------------------------|:------------|
| Fumaroles        | Constant      | Sulfur smell, minor tourism            | 2 cells     |
| Minor eruption   | Every 5-10 yr | Ash fall, road closures                | 20 cells    |
| Moderate eruption| Every 20-50 yr| Lava flow, evacuations, ash damage     | 40 cells    |
| Major eruption   | Every 100+ yr | Pyroclastic flow, widespread destruction| 80+ cells  |

#### 5.5.2 Ash Fall

```
ash_depth(distance, eruption_magnitude) =
    magnitude * 10.0 / (1.0 + distance^1.5)  // cm of ash

// Wind carries ash downwind:
effective_distance = distance - wind_bias * cos(angle_to_wind)

Effects by ash depth:
    0-1 cm:   Nuisance, roof cleaning needed
    1-5 cm:   Road closures, crop damage, AQI hazardous
    5-15 cm:  Structural risk (flat roofs collapse at ~10cm wet ash)
    15-30 cm: Most buildings at risk, total crop loss
    30+ cm:   Destruction zone
```

#### 5.5.3 Lava Flow

Lava follows terrain downhill (same D8 flow algorithm as water), but much slower:

```
lava_flow_speed = 0.1 cells/tick (slow, viscous)
// Faster on steep terrain: speed *= (1.0 + slope * 5.0)
// Everything in lava path is destroyed (100% damage)
// Lava solidifies after 10-20 ticks, creating new terrain
```

---

### 5.6 Tsunamis

#### 5.6.1 Tsunami Generation

Triggered by offshore earthquakes (M > 7.0) or underwater landslides.

```
ALGORITHM: SimulateTsunami
TRIGGER: Offshore earthquake M > 7.0

1. DETERMINE wave height:
   base_height_ft = (magnitude - 6.0) * 8.0  // 8ft for M7, 24ft for M9
   // Coastal amplification (shoaling): height increases in shallow water
   amplified_height = base_height_ft * sqrt(offshore_depth / coastal_depth)

2. INUNDATION:
   // Wave penetrates inland based on terrain
   For each coastal cell, moving inland:
       wave_height -= terrain_slope * 2.0
       IF wave_height > 0:
           flood_depth[x][y] = wave_height
           APPLY flood damage with depth-damage curve (Section 2.4.2)
           // Tsunami adds momentum damage: 50% more than static flooding
           damage *= 1.5
       ELSE: STOP

3. WARNING:
   // Depending on earthquake distance:
   // Near-field: 10-30 minutes warning
   // Far-field: 2-8 hours warning
   // Warning system reduces casualties by 80-95%
```

---

### 5.7 Disaster Recovery Framework

All disasters share a common recovery process:

```rust
struct DisasterRecovery {
    disaster_type: DisasterType,
    affected_cells: Vec<(u16, u16)>,
    total_damage_cost: f64,
    casualties: u32,
    displaced_citizens: u32,

    // Recovery phases
    phase: RecoveryPhase,
    emergency_duration_ticks: u32,    // 1-3 days
    repair_duration_ticks: u32,        // weeks to months
    rebuild_duration_ticks: u32,       // months to years
}

enum RecoveryPhase {
    Emergency,      // Search and rescue, shelter, medical
    Assessment,     // Damage inspection, cost estimation
    Repair,         // Fix damaged (not destroyed) buildings
    Rebuild,        // Reconstruct destroyed buildings
    Recovered,      // All damage addressed
}

// Recovery speed depends on:
// - City budget (money available for repairs)
// - Emergency service capacity (fire, police, medical)
// - Construction workforce availability
// - Insurance coverage (reduces city cost burden)
// - Federal/state aid (unlocked by declaring emergency)
```

**Insurance and aid:**
- City disaster fund: Player can pre-fund emergency reserve
- Insurance: Costs 2% of property value/year, covers 80% of damage
- Federal aid: Available for damages > 10% of city budget, covers 75%
- Without any coverage: Full cost on city budget

---

## 6. Waste Management

Waste is an unavoidable byproduct of a functioning city. The waste system tracks generation,
collection, processing, and disposal. Poorly managed waste causes pollution, health issues,
and citizen unhappiness. Well-managed waste can become a revenue source through recycling
and energy recovery.

### 6.1 Waste Generation

#### 6.1.1 Per Capita Generation Rates

The US average is approximately **4.5 pounds (2.04 kg) per person per day** of municipal
solid waste (MSW). This varies by wealth level and building type:

| Source Category         | Waste Rate              | Unit           | Notes                    |
|:------------------------|:-----------------------:|:---------------|:-------------------------|
| Low-income residential  |     3.0 lbs/person/day  | Per citizen    | Less consumption         |
| Middle-income residential|    4.5 lbs/person/day  | Per citizen    | US average               |
| High-income residential |     6.0 lbs/person/day  | Per citizen    | More packaging, goods    |
| Small commercial        |    50 lbs/day           | Per building   | Office waste, packaging  |
| Large commercial        |   300 lbs/day           | Per building   | Retail, food service     |
| Restaurant              |   200 lbs/day           | Per building   | High organic content     |
| Light industry          |   500 lbs/day           | Per building   | Process waste            |
| Heavy industry          | 2,000 lbs/day           | Per building   | Bulk industrial waste    |
| Hospital                | 1,500 lbs/day           | Per facility   | Includes medical waste   |
| School                  |   100 lbs/day           | Per facility   | Cafeteria, paper         |
| Construction site       | 5,000 lbs/day           | Per active site| C&D debris               |
| Demolition              |50,000 lbs total         | Per building   | One-time C&D debris      |

#### 6.1.2 Waste Composition

Understanding composition is critical for recycling and disposal planning:

| Material          | % of MSW | Recyclable? | Compostable? | Energy Content (BTU/lb) |
|:------------------|:--------:|:-----------:|:------------:|:-----------------------:|
| Paper/Cardboard   |    25%   |     Yes     |    Partial   |                   7,000 |
| Food waste        |    22%   |      No     |      Yes     |                   2,500 |
| Yard waste        |    12%   |      No     |      Yes     |                   3,000 |
| Plastics          |    13%   | Some (1,2,5)|      No      |                  14,000 |
| Metals            |     9%   |     Yes     |      No      |                     300 |
| Glass             |     4%   |     Yes     |      No      |                      60 |
| Wood              |     6%   |    Partial  |    Partial   |                   8,000 |
| Textiles          |     6%   |    Some     |      No      |                   7,500 |
| Other             |     3%   |      No     |      No      |                   5,000 |

**Average energy content of MSW: ~4,500 BTU/lb** (important for waste-to-energy).

```rust
struct WasteComposition {
    paper_cardboard: f32,   // 0.25
    food_waste: f32,        // 0.22
    yard_waste: f32,        // 0.12
    plastics: f32,          // 0.13
    metals: f32,            // 0.09
    glass: f32,             // 0.04
    wood: f32,              // 0.06
    textiles: f32,          // 0.06
    other: f32,             // 0.03
}

impl WasteComposition {
    fn recyclable_fraction(&self) -> f32 {
        self.paper_cardboard * 0.80    // 80% of paper is recyclable
        + self.plastics * 0.30          // 30% of plastics
        + self.metals * 0.95            // 95% of metals
        + self.glass * 0.90             // 90% of glass
        + self.wood * 0.20              // 20% of wood
        + self.textiles * 0.15          // 15% of textiles
    }

    fn compostable_fraction(&self) -> f32 {
        self.food_waste * 0.95
        + self.yard_waste * 0.98
        + self.paper_cardboard * 0.10   // shredded paper can be composted
        + self.wood * 0.30              // chips/sawdust
    }

    fn energy_content_btu_per_lb(&self) -> f32 {
        self.paper_cardboard * 7000.0
        + self.food_waste * 2500.0
        + self.yard_waste * 3000.0
        + self.plastics * 14000.0
        + self.metals * 300.0
        + self.glass * 60.0
        + self.wood * 8000.0
        + self.textiles * 7500.0
        + self.other * 5000.0
    }
}
```

### 6.2 Collection and Transport

#### 6.2.1 Collection System

Waste collection is service-area based. Each collection facility (transfer station or
processing facility) has a service radius and capacity:

```
ALGORITHM: WasteCollection
FREQUENCY: Every game-day

1. For each waste collection zone:
   total_waste = sum of waste_generated by all buildings in zone
   collection_capacity = trucks * truck_capacity * trips_per_day

   IF total_waste <= collection_capacity:
       collection_rate = 1.0  // all waste collected
   ELSE:
       collection_rate = collection_capacity / total_waste
       // Uncollected waste:
       uncollected = total_waste * (1.0 - collection_rate)
       // Accumulates at buildings: health/happiness penalty

2. Collected waste is routed to:
   - Recycling facility (if recycling program active)
   - Compost facility (if composting program active)
   - Waste-to-energy plant (if available)
   - Landfill (default destination for remainder)

3. Transport cost:
   cost = total_waste * COST_PER_TON_MILE * avg_distance_to_facility
   // Closer facilities save money (incentivizes distributed waste infra)
```

#### 6.2.2 Collection Infrastructure

| Facility Type          | Capacity (tons/day) | Footprint | Cost        | Operating Cost/Day |
|:-----------------------|:-------------------:|:---------:|:-----------:|:------------------:|
| Transfer station       |              200    |    2x2    |   $500K     |           $2,000   |
| Recycling center       |              100    |    3x3    | $2,000K     |           $4,000   |
| Composting facility    |               50    |    4x4    |   $800K     |           $1,500   |
| Waste-to-energy plant  |              500    |    4x4    |$50,000K     |          $15,000   |
| Landfill               |            1,000    |    8x8    | $5,000K     |           $3,000   |
| Hazardous waste site   |               20    |    2x2    | $3,000K     |           $5,000   |

**Collection trucks:**
- Capacity: 10 tons per truck
- Cost: $150,000 per truck
- Trips per day: 3-4
- Each truck serves approximately 500-800 households
- Route optimization: closer buildings served first

### 6.3 Landfill Systems

#### 6.3.1 Landfill Capacity Model

```rust
struct Landfill {
    total_capacity_tons: f64,
    current_fill_tons: f64,
    daily_input_tons: f32,
    has_liner: bool,
    has_leachate_collection: bool,
    has_gas_collection: bool,
    cells: Vec<(u16, u16)>,     // grid cells occupied
}

impl Landfill {
    fn remaining_capacity(&self) -> f64 {
        self.total_capacity_tons - self.current_fill_tons
    }

    fn years_remaining(&self) -> f32 {
        if self.daily_input_tons <= 0.0 { return f32::INFINITY; }
        (self.remaining_capacity() as f32) / (self.daily_input_tons * 365.0)
    }

    fn fill_fraction(&self) -> f32 {
        (self.current_fill_tons / self.total_capacity_tons) as f32
    }
}
```

**Landfill sizing:**
```
capacity_per_cell = 50,000 tons  (approximate, with compaction)
8x8 landfill = 64 cells * 50,000 = 3,200,000 tons capacity

For a city of 100,000 people at 4.5 lbs/person/day:
  Daily waste: 100,000 * 4.5 / 2000 = 225 tons/day
  With 50% diversion (recycling): 112.5 tons/day
  Annual: 41,000 tons/year
  Landfill life: 3,200,000 / 41,000 = ~78 years

For 500,000 people, same diversion:
  562 tons/day -> 205,000 tons/year
  Landfill life: ~16 years (need expansion or new site!)
```

#### 6.3.2 Landfill Environmental Effects

| Effect                  | Unlined Landfill | Lined Landfill | Lined + Collection |
|:------------------------|:----------------:|:--------------:|:------------------:|
| Groundwater pollution   |           High   |          Low   |        Minimal     |
| Soil contamination      |           High   |       Medium   |           Low      |
| Air pollution (methane) |         Medium   |       Medium   |   Low (w/capture)  |
| Odor radius (cells)     |              15  |            10  |               5    |
| Land value effect (%)   |           -40%   |          -25%  |            -15%    |
| Noise                   |        65 dB     |        65 dB   |          65 dB     |
| Vermin/pests            |           High   |       Medium   |           Low      |

**Landfill gas (LFG):**
- Decomposing organic waste generates methane (CH4)
- ~100 cubic feet of LFG per ton of waste per year
- LFG is 50% methane, 50% CO2
- Can be captured for energy: ~1 MW per 1,000 tons/day landfill
- Without capture: greenhouse gas + fire/explosion risk

#### 6.3.3 Post-Closure

When a landfill reaches capacity:
- Must be capped (clay/synthetic cover): $10,000/cell
- 30-year post-closure monitoring required: $50K/year
- Site can eventually become park (after 30+ years)
- Gas collection continues for 15-30 years
- Cannot build structures on closed landfill (settlement risk)

### 6.4 Recycling

#### 6.4.1 Recycling Program Tiers

| Program Level    | Diversion Rate | Participation | Cost/Household/Year | Revenue        |
|:-----------------|:--------------:|:-------------:|:-------------------:|:---------------|
| No program       |            5%  |          N/A  |                  $0 | None           |
| Voluntary drop-off|          15%  |          30%  |                 $20 | Minimal        |
| Curbside (basic) |           30%  |          60%  |                 $80 | Moderate       |
| Curbside (sort)  |           45%  |          70%  |                $120 | Good           |
| Single-stream    |           40%  |          80%  |                $100 | Moderate       |
| Pay-as-you-throw |           50%  |          85%  |                $60* | Good           |
| Zero waste goal  |           60%  |          90%  |                $150 | Best           |

*Pay-as-you-throw: variable fees based on waste volume reduce total costs

#### 6.4.2 Recycling Economics

```rust
struct RecyclingEconomics {
    // Revenue from selling recyclables
    paper_price_per_ton: f32,      // $50-150 (volatile)
    plastic_price_per_ton: f32,    // $100-400 (very volatile)
    metal_price_per_ton: f32,      // $200-600 (aluminum valuable)
    glass_price_per_ton: f32,      // $10-40 (low value)

    // Costs
    collection_cost_per_ton: f32,  // $50-100 (sorting adds cost)
    processing_cost_per_ton: f32,  // $30-80
    contamination_rate: f32,       // 0.15-0.30 (waste in recycling stream)
}

impl RecyclingEconomics {
    fn net_value_per_ton(&self, composition: &WasteComposition) -> f32 {
        let revenue =
            composition.paper_cardboard * self.paper_price_per_ton
            + composition.plastics * self.plastic_price_per_ton * 0.30  // only some recyclable
            + composition.metals * self.metal_price_per_ton
            + composition.glass * self.glass_price_per_ton;

        let cost = self.collection_cost_per_ton + self.processing_cost_per_ton;
        let contamination_loss = revenue * self.contamination_rate;

        revenue - contamination_loss - cost
        // Can be negative! Recycling doesn't always pay for itself.
        // Environmental benefits justify the cost.
    }
}
```

**Market volatility:** Recycling commodity prices fluctuate based on global markets.
In the game, prices cycle with a ~5 game-year period:
- Boom: prices at 1.5x average (recycling very profitable)
- Normal: baseline prices
- Bust: prices at 0.3x average (recycling costs money, may need subsidies)

#### 6.4.3 Recycling Algorithm

```
ALGORITHM: ProcessRecycling
FREQUENCY: Every game-day
DATA: waste_generated[], recycling_program, recycling_facilities[]

1. For each building producing waste:
   total_waste = building.waste_generated()
   recyclable = total_waste * waste_composition.recyclable_fraction()
   participation = recycling_program.participation_rate

   actually_recycled = recyclable * participation
   remaining_waste = total_waste - actually_recycled

2. Route recycled material to recycling facility:
   IF recycling_facility.has_capacity(actually_recycled):
       facility.process(actually_recycled)
       contaminated = actually_recycled * contamination_rate
       actually_recycled -= contaminated  // contaminated goes to landfill
       revenue += actually_recycled * market_price_per_ton
   ELSE:
       // Overflow goes to landfill
       remaining_waste += actually_recycled

3. Route remaining waste to landfill/WTE:
   // Standard disposal path
```

### 6.5 Waste-to-Energy

#### 6.5.1 WTE Plant Design

Waste-to-energy plants incinerate waste to generate electricity and steam:

```
energy_output(waste_tons) =
    waste_tons * avg_btu_per_lb * 2000  // total BTU
    * boiler_efficiency                  // 0.65-0.85
    * generator_efficiency               // 0.30-0.40
    / 3412                               // BTU per kWh

Example: 500 tons/day, 4500 BTU/lb, 80% boiler, 35% generator
  = 500 * 4500 * 2000 * 0.80 * 0.35 / 3412
  = 370,000 kWh/day = 15.4 MW average
```

| Parameter                | Value              |
|:-------------------------|:-------------------|
| Waste input              | 200-1000 tons/day  |
| Electricity output       | 0.5-1.0 MWh/ton   |
| Steam output (optional)  | District heating   |
| Ash residue              | 10% of input mass  |
| Air emissions            | Controlled (scrubbers required) |
| Ash disposal             | Secure landfill    |
| Construction cost        | $50-150M           |
| Operating cost           | $40-60/ton         |
| Revenue: electricity     | $30-50/ton         |
| Revenue: tipping fees    | $50-80/ton         |
| Net cost                 | -$10 to +$30/ton   |

#### 6.5.2 WTE Environmental Trade-offs

**Advantages:**
- Reduces landfill volume by 90% (ash residue only)
- Generates renewable energy
- Destroys pathogens and hazardous organics
- Smaller footprint than equivalent landfill
- Can provide district heating

**Disadvantages:**
- Air emissions (even with controls): particulates, dioxins, heavy metals
- Source Q for air pollution: 45.0 (with scrubbers: 20.0)
- Competes with recycling (diversion reduces WTE feedstock)
- High capital cost
- Ash requires secure disposal
- Public opposition (NIMBY)

#### 6.5.3 Waste-to-Energy vs Recycling Hierarchy

The waste management hierarchy in order of preference:
1. **Reduce** (less consumption)
2. **Reuse** (extend product life)
3. **Recycle** (material recovery)
4. **Energy recovery** (WTE)
5. **Landfill** (last resort)

In the game, policies can push the hierarchy:
- Plastic bag ban: -5% overall waste
- Deposit/return program: +10% recycling
- Composting mandate: +15% diversion
- WTE plant: -90% landfill volume but requires recycling balance

### 6.6 Composting and Organics

#### 6.6.1 Organic Waste Diversion

Organic waste (food + yard waste) comprises ~34% of the waste stream. Composting diverts
this from landfill and produces useful soil amendment:

```
compostable_waste = population * waste_per_capita
                  * (food_fraction + yard_fraction)
                  * participation_rate

// For 100,000 people:
// 100,000 * 4.5 lbs * 0.34 * 0.70 participation
// = 107,100 lbs/day = 53.5 tons/day
```

| Composting Method    | Capacity (tons/day) | Time to Compost | Footprint | Cost/Ton |
|:---------------------|:-------------------:|:---------------:|:---------:|:--------:|
| Windrow              |              50     | 3-6 months      |    6x6    |     $30  |
| Aerated static pile  |             100     | 2-3 months      |    4x4    |     $45  |
| In-vessel            |             200     | 2-4 weeks       |    3x3    |     $60  |
| Anaerobic digestion  |             100     | 3-4 weeks       |    3x3    |     $50  |

**Anaerobic digestion** has the additional benefit of producing biogas (methane) that can
generate electricity -- similar to landfill gas recovery but faster and more controlled.
Typical output: 0.1-0.2 MWh per ton of organic waste.

#### 6.6.2 Compost Benefits

Compost produced can be:
- Sold to citizens/farms: $20-40/ton revenue
- Used in city parks: reduces irrigation need by 25%, reduces fertilizer cost
- Applied to agricultural zones: +15% crop yield
- Used in green infrastructure: improves rain garden performance

### 6.7 Hazardous Waste

#### 6.7.1 Sources and Handling

Industrial and medical facilities generate hazardous waste requiring special treatment:

| Source               | Hazardous Waste (lbs/day) | Type              |
|:---------------------|:-------------------------:|:------------------|
| Chemical plant       |                       200 | Chemical          |
| Hospital             |                       100 | Medical/biohazard |
| Auto repair shop     |                        20 | Oil, solvents     |
| Electronics factory  |                        50 | Heavy metals      |
| University/lab       |                        30 | Chemical, radioactive|
| Nuclear plant        |                         5 | Radioactive (low) |
| Dry cleaner          |                        10 | Solvents          |

Hazardous waste must go to a licensed hazardous waste facility. If no facility exists:
- Illegal dumping: soil + groundwater contamination
- Health crisis if near residential areas
- EPA fines (game equivalent: federal penalties)

#### 6.7.2 Hazardous Waste Facility

```rust
struct HazardousWasteFacility {
    capacity_tons_per_day: f32,
    treatment_types: Vec<HazWasteType>,
    operating_cost_per_ton: f32,      // $500-2000
    pollution_output: f32,             // very low with proper management
    required_buffer_zone_cells: u8,    // 5 cells minimum from residential
}
```

---

## 7. Reference Games Analysis

This section analyzes how other games have implemented environmental and climate systems,
extracting lessons for Megacity's design.

### 7.1 Frostpunk

#### 7.1.1 Temperature as Central Mechanic

Frostpunk (11 bit studios, 2018) is built entirely around surviving extreme cold. Temperature
is the game's primary antagonist.

**Key mechanics:**
- **Global temperature**: A single value affecting the entire city, dropping over time
- **Temperature tiers**: Chilly (-20C) -> Cold (-40C) -> Very Cold (-60C) -> Freezing (-80C)
- **Heating zones**: Steam hubs and generators create heated zones with radius
- **Insulation**: Building upgrades improve heat retention
- **Hope/Discontent**: Dual morale system driven by temperature and survival

**What Megacity can learn:**
- Temperature as a clear, visible, dramatic threat works well for engagement
- Concentric heating zones (generator -> steam hubs -> buildings) create spatial planning
- The "temperature drops over time" creates escalating tension
- Insulation upgrades give players a meaningful tech tree

**Specific numbers from Frostpunk:**
- Generator radius: 4 tiles warm, 2 tiles comfortable
- Steam hub radius: 3 tiles warm
- Each temperature tier increases coal consumption by ~50%
- Citizens at "freezing" risk: 20% daily chance of frostbite -> 10% chance of death
- Buildings below "cold" threshold: 50% worker efficiency

**Adaptation for Megacity:**
- Rather than a single global temperature, use per-cell temperature with UHI
- Heating/cooling zones from HVAC systems (similar to steam hubs)
- Winter/cold snap events as periodic challenges, not constant escalation
- Energy grid stress during extreme temperatures (more interesting than single coal meter)
- Citizen health effects from temperature (already modeled in Section 4.4)

### 7.2 Anno 2070

#### 7.2.1 Eco vs Industrial Factions

Anno 2070 (Ubisoft Blue Byte, 2011) features two main factions with contrasting
environmental philosophies:

**Eco faction (Eden Initiative):**
- Buildings are green/organic aesthetic
- Lower production output but no pollution
- Renewable energy (wind, solar, tidal)
- Bonus from high "ecobalance"
- Citizens want parks, clean air, nature

**Industrial faction (Global Trust):**
- High production output but heavy pollution
- Fossil fuel and nuclear energy
- Bonus from high production efficiency
- Citizens want goods, technology, comfort

**Ecobalance mechanic:**
- Global counter: +/- based on building types
- Positive ecobalance: bonus fertility, fish yield, citizen happiness
- Negative ecobalance: reduced yields, pollution, citizen health issues
- Neutral: no bonuses or penalties

**What Megacity can learn:**
- The faction system creates replayability and distinct strategies
- Ecobalance as a single visible metric helps players understand environmental impact
- Trade-offs between production and environment are compelling gameplay
- Visual feedback (green vs brown landscapes) reinforces environmental state

**Adaptation for Megacity:**
- No faction lock: players can mix eco and industrial freely
- "Environmental score" aggregate metric visible on dashboard
- Green building variants available as upgrades (more expensive, less pollution)
- Policy system allowing eco regulations vs industrial deregulation
- Visual degradation: polluted areas show brown/hazy, clean areas show lush green
- Achievement system: "Green City" (avg pollution < 20), "Industrial Powerhouse" (GDP > X)

### 7.3 Surviving Mars

#### 7.3.1 Dome-Based Environmental Control

Surviving Mars (Haemimont Games, 2018) uses domes to create habitable zones on Mars:

**Key mechanics:**
- **Domes** are enclosed habitable zones with controlled atmosphere
- **Life support**: Oxygen, water, food must be produced/imported
- **Power grid**: Critical -- outages kill colonists
- **Dust storms**: Periodic events that damage solar panels, bury buildings
- **Cold waves**: Increase heating demand, risk of pipe freezing
- **Meteors**: Random destruction events

**Environmental management:**
- Oxygen: Produced by MOXIE plants, stored in tanks, consumed per colonist
- Water: Extracted from subsurface, recycled via water reclamation
- Each resource has production, consumption, and storage
- Buffer systems (tanks, batteries) smooth supply fluctuations

**What Megacity can learn:**
- Resource chains (production -> storage -> consumption) create planning depth
- Environmental hazards that damage specific infrastructure types
- The importance of redundancy (backup power, water reserves)
- Storage as a key mechanic: cities with storage survive crises
- Gradual difficulty (early Mars is forgiving, late game gets harder)

**Adaptation for Megacity:**
- Apply resource chain thinking to water and energy
- Storm events that specifically target infrastructure (solar panels, power lines)
- Emergency reserves for water, power, food as player-managed systems
- Dust/pollution events that reduce solar efficiency (already in wind/solar model)
- The "dome" concept parallels district management with local infrastructure

### 7.4 Eco

#### 7.4.1 Ecosystem Simulation

Eco (Strange Loop Games, 2018) is a multiplayer survival/civilization game with a
sophisticated ecosystem simulation:

**Key mechanics:**
- **Full ecosystem**: Plants, animals, soil nutrients, water cycle all simulated
- **Pollution is persistent**: CO2, tailings, sewage permanently degrade environment
- **Extinction events**: Overhunting/overharvesting can eliminate species
- **Government system**: Players vote on laws (pollution limits, hunting quotas)
- **Meteor threat**: Must build technology to destroy approaching meteor
- **Skill system**: Players specialize (farmer, miner, engineer)

**Environmental modeling depth:**
- Soil has nutrients (nitrogen, phosphorus) that deplete with farming
- Plants grow based on temperature, rainfall, soil quality, biome
- Animals have population dynamics (birth rate, death rate, migration)
- Pollution diffuses through air and water realistically
- Climate change: CO2 emissions raise global temperature over time

**What Megacity can learn:**
- Long-term consequences: pollution that never fully goes away creates tension
- Ecosystem collapse as a real failure state (not just a penalty)
- Government/policy as a mechanic for managing commons
- The urgency of a timer (meteor) driving environmental cooperation
- Nutrient cycles add depth to agricultural/food systems

**Adaptation for Megacity:**
- Persistent soil contamination (already designed in 1.4)
- Climate change as a long-term consequence of fossil fuel use
  - Track cumulative CO2 emissions
  - After threshold: sea level rise (+1 cell flooding), more extreme weather
- Policy panel for environmental regulations
- Ecosystem services: parks/nature provide measurable economic benefits
- "Tipping points" where environmental damage becomes self-reinforcing

### 7.5 Cities: Skylines

#### 7.5.1 Environmental Systems

Cities: Skylines (Colossal Order, 2015) has the most directly comparable environmental systems:

**Pollution:**
- Ground pollution: visual brown overlay, affects health and land value
- Noise pollution: separate overlay, from roads and industry
- Water pollution: flows downstream, must place intake upstream of sewage outfall
- No air pollution as distinct system (merged with ground)

**Water:**
- Realistic water flow simulation (Unity-based fluid dynamics)
- Water intake and sewage outfall placement matters
- Flooding from rain (in DLC)
- Dam construction for hydroelectric

**Power:**
- Multiple generation types with different costs/pollution
- Power line placement required
- Wind turbines affected by terrain-generated wind patterns

**Strengths:**
- Water flow is visually impressive and intuitive
- Zoning + infrastructure creates organic city growth
- The pollution overlay is clear and actionable

**Weaknesses:**
- Pollution is binary (polluted/clean), lacks nuance
- No seasonal variation in base game
- Energy system is simple (no time-of-day demand)
- No climate change or long-term environmental consequences

**What Megacity should do differently:**
- More granular pollution (continuous values, not binary)
- Seasonal effects on all systems
- Dynamic energy market with peak pricing
- Long-term environmental scoring affecting city development
- Weather as gameplay driver, not just visual

### 7.6 SimCity (2013)

#### 7.6.1 GlassBox Simulation

SimCity 2013 (Maxis) used the "GlassBox" agent-based simulation engine:

**Environmental features:**
- Air pollution: Wind-driven, industrial/traffic sources
- Ground pollution: From industry, landfills
- Water table: Underground water model
- Sewage: Underground flow toward treatment
- Resource extraction: Oil, ore, coal as finite map resources

**Resource depletion:**
- Oil wells, mines, and quarries extract finite resources
- Creates boom-bust cycles for resource-dependent cities
- Forces economic diversification or city death

**Interesting mechanics:**
- Waste management chain: collection -> processing -> disposal
- Recycling as a profitable industry
- Regional play: share services between cities (water, power, garbage)
- Specialization: cities can focus on industry, tourism, education, etc.

**Lessons:**
- Finite resources create interesting long-term strategy
- Regional cooperation could be adapted for district management
- Waste as a chain (not just "disappears") is more interesting
- Resource extraction provides economic narratives

### 7.7 Banished

#### 7.7.1 Survival Resource Management

Banished (Shining Rock Software, 2014) is a medieval city builder focused on survival:

**Environmental relevance:**
- **Seasonal cycles**: Growing seasons, winter survival
- **Food system**: Multiple crops, livestock, fishing, gathering
- **Forestry**: Trees as renewable resource, deforestation consequences
- **Health**: Clean water, varied diet, herbal medicine
- **Education**: Increases worker efficiency

**Temperature model:**
- Winter: Citizens need firewood, warm clothing, heated homes
- Blizzards: Increase firewood consumption 200%
- Citizens outside in winter: health penalty, possible death
- Food spoilage: hot weather spoils stored food

**Resource chains:**
- Log -> Firewood (for heating)
- Ore -> Tools (for all work)
- Crops -> Food (seasonal, must store for winter)
- Herbs -> Medicine (for health)
- Stone/Wood/Iron -> Buildings

**What Megacity can learn:**
- Seasonal food/resource cycles create natural rhythm
- Winter preparation as gameplay (stockpiling resources)
- The satisfaction of surviving a harsh winter
- Education improving efficiency is a compelling upgrade path
- Deforestation consequences (erosion, wildlife loss)

---

## 8. ECS Integration Architecture

This section describes how the environmental systems integrate with Megacity's Bevy ECS
architecture.

### 8.1 Resource Types

All environmental data stored as Bevy resources:

```rust
// Pollution grids (256x256 flat arrays for cache efficiency)
#[derive(Resource)]
struct AirPollutionGrid {
    data: Vec<f32>,  // 65,536 cells
    dirty_chunks: BitVec,  // which 8x8 chunks need recalculation
}

#[derive(Resource)]
struct WaterPollutionGrid {
    data: Vec<f32>,
    flow_direction: Vec<u8>,  // D8 direction per cell
    topo_order: Vec<u16>,     // precomputed topological sort
}

#[derive(Resource)]
struct NoisePollutionGrid {
    data: Vec<f32>,           // current dB level per cell
}

#[derive(Resource)]
struct SoilContaminationGrid {
    data: Vec<f32>,
}

#[derive(Resource)]
struct UhiGrid {
    data: Vec<f32>,           // temperature increment per cell
}

// Weather state (global)
#[derive(Resource)]
struct WeatherState {
    temperature: f32,
    humidity: f32,
    cloud_cover: f32,
    precipitation: f32,
    wind: WindState,
    condition: WeatherCondition,
    season: Season,
    day_of_year: u32,
    hour: u8,
}

// Energy system
#[derive(Resource)]
struct EnergyGrid {
    total_demand_mwh: f32,
    total_supply_mwh: f32,
    reserve_margin: f32,
    electricity_price: f32,
    blackout_active: bool,
    blackout_cells: BitVec,
}

// Waste system
#[derive(Resource)]
struct WasteSystem {
    total_generated_tons: f32,
    total_collected_tons: f32,
    recycling_diversion_rate: f32,
    landfill_remaining_capacity: f64,
    uncollected_waste_cells: Vec<(u16, u16)>,
}

// Water supply
#[derive(Resource)]
struct WaterSupply {
    total_demand_mgd: f32,
    total_supply_mgd: f32,
    groundwater_level: f32,
    reservoir_level: f32,
    service_coverage: f32,  // fraction of buildings served
}

// Disaster state
#[derive(Resource)]
struct ActiveDisasters {
    events: Vec<DisasterEvent>,
    recovery_projects: Vec<DisasterRecovery>,
}
```

### 8.2 System Schedule

Systems are organized by update frequency to optimize performance:

```rust
impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        app
            // EVERY TICK (core simulation)
            .add_systems(Update, (
                weather_hourly_update
                    .run_if(on_game_hour),
                fire_spread_system
                    .run_if(any_fire_active),
                disaster_tick_system
                    .run_if(any_disaster_active),
            ))

            // EVERY 4 TICKS (frequent updates)
            .add_systems(Update, (
                air_pollution_update,
                energy_demand_calculation,
                energy_dispatch,
            ).run_if(every_n_ticks(4)))

            // EVERY 8 TICKS (moderate updates)
            .add_systems(Update, (
                water_pollution_propagation,
                noise_grid_update,
                stormwater_calculation
                    .run_if(is_raining),
            ).run_if(every_n_ticks(8)))

            // EVERY 30 TICKS (slow updates)
            .add_systems(Update, (
                soil_contamination_update,
                uhi_calculation,
                landfill_capacity_update,
            ).run_if(every_n_ticks(30)))

            // DAILY (game day boundary)
            .add_systems(Update, (
                waste_collection_system,
                water_demand_calculation,
                seasonal_modifier_update,
                disaster_recovery_progress,
                weather_daily_planning,
            ).run_if(on_game_day))

            // YEARLY
            .add_systems(Update, (
                recycling_market_update,
                climate_change_assessment,
                environmental_score_calculation,
            ).run_if(on_game_year))

            // EVENT-DRIVEN
            .add_systems(Update, (
                earthquake_system.run_if(earthquake_triggered),
                tornado_system.run_if(tornado_triggered),
                flood_event_system.run_if(flood_triggered),
                wildfire_ignition_check.run_if(fire_risk_high),
            ));
    }
}
```

### 8.3 Component Types

Buildings and citizens get environment-related components:

```rust
// Building environment components
#[derive(Component)]
struct PollutionSource {
    air_q: f32,
    water_q: f32,
    noise_db: f32,
    soil_rate: f32,
    stack_height: f32,
}

#[derive(Component)]
struct EnergyConsumer {
    base_demand_kwh: f32,
    priority: LoadPriority,
    has_power: bool,
}

#[derive(Component)]
struct WaterConsumer {
    demand_gpd: f32,
    has_water: bool,
    has_sewer: bool,
}

#[derive(Component)]
struct WasteProducer {
    waste_lbs_per_day: f32,
    recycling_participation: bool,
}

#[derive(Component)]
struct FloodVulnerability {
    elevation_offset: f32,    // raised foundation = less vulnerable
    building_category: BuildingCategory,
}

#[derive(Component)]
struct SeismicRating {
    construction_type: ConstructionType,
    has_retrofit: bool,
}

#[derive(Component)]
struct FireResistance {
    material: BuildingMaterial,
    has_sprinklers: bool,
    defensible_space: bool,
}

// Citizen environment components
#[derive(Component)]
struct EnvironmentalHealth {
    air_quality_exposure: f32,
    noise_exposure: f32,
    water_quality: f32,
    temperature_stress: f32,
}
```

### 8.4 Event System

Environmental events communicate between systems:

```rust
// Events that other systems can listen to
#[derive(Event)]
struct PowerOutageEvent {
    affected_cells: Vec<(u16, u16)>,
    duration_estimate_hours: f32,
}

#[derive(Event)]
struct FloodingEvent {
    flooded_cells: Vec<(u16, u16, f32)>,  // x, y, depth
}

#[derive(Event)]
struct DisasterStartEvent {
    disaster_type: DisasterType,
    severity: f32,
    epicenter: Option<(u16, u16)>,
}

#[derive(Event)]
struct PollutionAlertEvent {
    pollution_type: PollutionType,
    severity: AlertLevel,
    affected_area: Vec<(u16, u16)>,
}

#[derive(Event)]
struct WeatherChangeEvent {
    old_condition: WeatherCondition,
    new_condition: WeatherCondition,
    is_extreme: bool,
}

#[derive(Event)]
struct WasteCapacityWarning {
    landfill_years_remaining: f32,
    urgency: AlertLevel,
}
```

---

## 9. Performance Considerations

### 9.1 Grid Operation Costs

For a 256x256 grid (65,536 cells), the cost of common operations:

| Operation                    | Cells Touched | Frequency   | Est. Time (us) | Notes          |
|:-----------------------------|:-------------:|:------------|:---------------:|:---------------|
| Full grid decay (multiply)   |        65,536 | 4 ticks     |              50 | SIMD-friendly  |
| Source dispersion (r=10)     |    ~300/source| 4 ticks     |         5/source| Per-source     |
| D8 flow propagation          |        65,536 | 8 ticks     |             200 | Topo sort      |
| Noise grid (max over sources)|       ~10,000 | 8 ticks     |             150 | Dominant approx|
| Soil update                  |        65,536 | 30 ticks    |              80 | Very simple ops|
| UHI calculation              |        65,536 | 30 ticks    |             300 | Neighborhood   |
| Flood simulation (5 iters)   |        65,536 | Event-driven|           2,000 | Expensive      |
| Fire spread                  |   ~100-10,000 | Every tick  |       50-1,000 | Scales with fire|
| Earthquake damage            |        65,536 | Event       |             500 | One-time       |

### 9.2 Optimization Strategies

#### 9.2.1 Spatial Partitioning

Use the existing CHUNK_SIZE = 8 system. Each chunk is 8x8 = 64 cells.
Grid has 32x32 = 1,024 chunks.

```
Dirty chunk tracking:
- Mark chunks near pollution sources as dirty
- Only update dirty chunks for dispersion
- Chunks with no sources and low pollution: skip (fast decay to 0)

Expected savings: 60-80% of cells skipped in typical city
```

#### 9.2.2 SIMD Operations

Grid-wide operations (decay, clamping) are perfect for SIMD:

```rust
// Decay all pollution values using SIMD (conceptual)
fn decay_grid_simd(grid: &mut [f32; 65536], factor: f32) {
    // Process 8 floats at a time with AVX2
    for chunk in grid.chunks_exact_mut(8) {
        // In practice, use std::simd or packed_simd
        for v in chunk.iter_mut() {
            *v *= factor;
        }
    }
}
```

#### 9.2.3 Temporal Amortization

Spread expensive updates across multiple ticks:

```
Quadrant rotation for air pollution:
  Tick 0: Update quadrant (0,0)-(127,127)
  Tick 1: Update quadrant (128,0)-(255,127)
  Tick 2: Update quadrant (0,128)-(127,255)
  Tick 3: Update quadrant (128,128)-(255,255)

Each tick processes 1/4 of the grid = 16,384 cells
Full grid updated every 4 ticks (still responsive)
```

#### 9.2.4 Level of Detail

For sources far from the camera or in non-critical areas:

```
LOD for pollution sources:
  High detail (within 20 cells of camera): Full kernel, exact values
  Medium detail (20-60 cells): Simplified kernel (4 samples, not full radius)
  Low detail (60+ cells): Point contribution only (no spread)

LOD for weather effects:
  Visible area: Full per-cell temperature/precipitation
  Off-screen: Aggregate per-chunk values
```

#### 9.2.5 Precomputation

Many environmental calculations can be precomputed:

```
Precomputed on map load / terrain change:
  - D8 flow directions (water pollution, stormwater)
  - Topological sort order
  - Flood zones (100-year, 500-year)
  - Wind exposure map (for wind turbine placement)
  - Slope map (for fire spread, landslide risk)

Precomputed on building change:
  - Noise source list with positions
  - Pollution source list with Q values
  - Energy demand/supply totals
  - Water demand totals
  - Waste generation totals

Precomputed per weather change:
  - Solar irradiance by hour
  - Wind power output
  - Heating/cooling demand multiplier
```

### 9.3 Memory Budget

| Data Structure                | Size (bytes) | Notes                        |
|:------------------------------|:------------:|:-----------------------------|
| Air pollution grid (f32)      |      262,144 | 256*256*4                    |
| Water pollution grid (f32)    |      262,144 |                              |
| Noise grid (f32)              |      262,144 |                              |
| Soil contamination grid (f32) |      262,144 |                              |
| UHI grid (f32)                |      262,144 |                              |
| Flow direction grid (u8)      |       65,536 |                              |
| Flood depth grid (f32)        |      262,144 | Only during flood events     |
| Fire state grid (u8)          |       65,536 | Only during fire events      |
| Topo sort order (u32)         |      262,144 | Precomputed                  |
| Chunk dirty flags (bits)      |          128 | 1024 chunks / 8              |
| **Total (permanent)**         |  **~1.7 MB** |                              |
| **Total (with events)**       |  **~2.2 MB** |                              |

This is well within budget. The grids are the most memory-efficient representation
possible for per-cell data.

### 9.4 Parallelism

Many environmental systems are independent and can run in parallel:

```
Parallel group A (pollution, no dependencies):
  - Air pollution update
  - Water pollution update
  - Noise update
  - Soil update
  (All read from source lists and write to separate grids)

Parallel group B (infrastructure, reads pollution):
  - Energy dispatch (reads demand, weather)
  - Water supply (reads demand, weather)
  - Waste collection (reads generation)

Sequential (depends on A and B):
  - Health effects (reads all pollution grids)
  - Happiness calculation (reads health, noise, services)
  - Building development (reads services, pollution, land value)
```

Using Bevy's built-in parallel system scheduling, these can be automatically parallelized
as long as resource access is correctly annotated.

---

## Appendix A: Constants and Tuning Parameters

All environment system constants collected in one place for easy tuning:

```rust
// === POLLUTION ===
const AIR_POLLUTION_DECAY: f32 = 0.85;
const AIR_POLLUTION_MAX: f32 = 1000.0;
const AIR_MIN_CONCENTRATION: f32 = 0.1;
const AIR_MAX_RADIUS: usize = 32;

const WATER_POLLUTION_FLOW_TRANSFER: f32 = 0.70;
const WATER_POLLUTION_STREAM_DECAY: f32 = 0.90;
const WATER_POLLUTION_RIVER_DECAY: f32 = 0.95;
const WATER_POLLUTION_SINK_DECAY: f32 = 0.98;
const WATER_POLLUTION_MAX: f32 = 500.0;

const NOISE_AMBIENT_DB: f32 = 35.0;
const NOISE_ATTENUATION_PER_CELL: f32 = 0.5;
const NOISE_BARRIER_MAX_DB: f32 = 40.0;

const SOIL_SPREAD_THRESHOLD: f32 = 50.0;
const SOIL_SPREAD_RATE: f32 = 0.01;
const SOIL_NATURAL_DECAY: f32 = 0.9999;
const SOIL_MAX: f32 = 500.0;

// === WATER ===
const WATER_DEMAND_BASE_GPCD: f32 = 150.0;
const WASTEWATER_FRACTION: f32 = 0.80;
const GROUNDWATER_RECHARGE_BASE: f32 = 0.01;
const GROUNDWATER_CRITICAL_LEVEL: f32 = 0.20;

// === ENERGY ===
const ELECTRICITY_BASE_RATE: f32 = 0.12;
const RESERVE_MARGIN_WARNING: f32 = 0.15;
const RESERVE_MARGIN_CRITICAL: f32 = 0.05;
const TRANSMISSION_LOSS_PER_10_CELLS: f32 = 0.02;
const POWER_SERVICE_RADIUS: u8 = 6;

// === WEATHER ===
const HDD_BASE_TEMP: f32 = 65.0;
const CDD_BASE_TEMP: f32 = 65.0;
const HEATING_ENERGY_PER_HDD: f32 = 0.02;
const COOLING_ENERGY_PER_CDD: f32 = 0.03;
const UHI_MAX_INCREMENT: f32 = 15.0;
const UHI_NIGHTTIME_AMPLIFICATION: f32 = 2.0;

// === WASTE ===
const WASTE_PER_CAPITA_LBS: f32 = 4.5;
const LANDFILL_CAPACITY_PER_CELL: f64 = 50_000.0;
const RECYCLING_CONTAMINATION_RATE: f32 = 0.20;
const WTE_BTU_PER_LB: f32 = 4500.0;
const WTE_BOILER_EFFICIENCY: f32 = 0.80;
const WTE_GENERATOR_EFFICIENCY: f32 = 0.35;

// === DISASTERS ===
const EARTHQUAKE_FREQUENCY_YEARS: f32 = 20.0;
const TORNADO_SEASON_MONTHS: [u8; 4] = [3, 4, 5, 6]; // Mar-Jun
const WILDFIRE_FUEL_DRY_THRESHOLD: f32 = 0.10;
const FLOOD_100YR_RAINFALL_IN: f32 = 6.0;

// === UPDATE FREQUENCIES (in ticks) ===
const AIR_POLLUTION_UPDATE_INTERVAL: u32 = 4;
const WATER_POLLUTION_UPDATE_INTERVAL: u32 = 8;
const NOISE_UPDATE_INTERVAL: u32 = 8;
const SOIL_UPDATE_INTERVAL: u32 = 30;
const UHI_UPDATE_INTERVAL: u32 = 30;
```

---

## Appendix B: Overlay Visualization Guide

Each environmental system should have a corresponding map overlay:

| Overlay Name         | Color Scheme                                    | Data Source              |
|:---------------------|:------------------------------------------------|:-------------------------|
| Air Quality          | Green (good) -> Yellow -> Orange -> Red (bad)   | air_pollution_grid       |
| Water Quality        | Blue (clean) -> Green -> Yellow -> Brown (dirty) | water_pollution_grid    |
| Noise Level          | White (quiet) -> Yellow -> Orange -> Red (loud)  | noise_grid              |
| Soil Contamination   | Green (clean) -> Yellow -> Orange -> Brown       | soil_contamination_grid |
| Temperature / UHI    | Blue (cool) -> White -> Yellow -> Red (hot)      | uhi_grid + weather     |
| Flood Risk           | Transparent -> Light blue -> Blue -> Dark blue   | flood_zones + depth    |
| Fire Risk            | Green (low) -> Yellow -> Orange -> Red (extreme) | fuel + moisture + wind |
| Power Coverage       | Green (served) -> Red (no power)                 | energy_grid.blackout   |
| Water Service        | Blue (served) -> Gray (no water)                 | water_supply.coverage  |
| Waste Collection     | Green (collected) -> Red (uncollected)            | waste_system           |
| Wind                 | Animated arrows showing direction and speed      | weather.wind           |
| Land Value Impact    | Combined pollution effects on property values    | Computed from all grids |

Each overlay should support:
- Toggle on/off from toolbar
- Opacity slider (0.3 to 0.8 default)
- Legend showing value ranges
- Tooltip on hover showing exact value
- Layer combination (e.g., show air + noise simultaneously)

---

*Document version: 1.0*
*Total systems: 6 major systems, 35+ subsystems*
*Total formulas/algorithms: 25+*
*Estimated implementation effort: 8-12 weeks for core systems, 4-6 weeks for disasters*
