# Transportation Simulation: Deep Implementation Guide for Megacity

## Table of Contents

1. [Traffic Flow Mathematics](#1-traffic-flow-mathematics)
2. [Intersection Design and Capacity](#2-intersection-design-and-capacity)
3. [Road Network Hierarchy and Design](#3-road-network-hierarchy-and-design)
4. [Public Transit Simulation](#4-public-transit-simulation)
5. [Freight and Goods Movement](#5-freight-and-goods-movement)
6. [Parking Simulation](#6-parking-simulation)
7. [What Games Get Wrong](#7-what-games-get-wrong)
8. [Implementation Recommendations for Megacity](#8-implementation-recommendations-for-megacity)

---

## 1. Traffic Flow Mathematics

### 1.1 The Fundamental Diagram of Traffic Flow

The fundamental diagram is the cornerstone of all traffic engineering. It describes the relationship between three variables:

- **Flow (q)**: vehicles per hour per lane (veh/hr/lane)
- **Density (k)**: vehicles per kilometer per lane (veh/km/lane)
- **Speed (v)**: average speed in km/hr

The identity relationship:

```
q = k * v
```

This is not a model -- it is a definition. Flow equals density times speed, always.

The key insight is that these three quantities are constrained. When a road is empty (k near 0), vehicles travel at **free-flow speed** (v_f). As density increases, speed drops. At some critical density, flow reaches its maximum (**capacity**). Beyond that, adding more vehicles causes flow to *decrease* -- this is congestion.

**Key parameters:**

| Parameter | Symbol | Typical Values |
|-----------|--------|----------------|
| Free-flow speed | v_f | 50 km/h (local), 100 km/h (highway) |
| Jam density | k_j | 120-160 veh/km/lane |
| Capacity (max flow) | q_max | 1800-2200 veh/hr/lane (highway), 800-1200 (urban) |
| Critical density | k_c | ~k_j/3 to k_j/4, typically 30-50 veh/km/lane |
| Critical speed | v_c | ~v_f/2 to v_f/3 |

**Greenshields' Linear Model** (simplest):

```
v(k) = v_f * (1 - k/k_j)
```

Substituting into q = k * v:

```
q(k) = k * v_f * (1 - k/k_j)
```

This is a parabola. Maximum flow occurs at k_c = k_j/2, giving:

```
q_max = v_f * k_j / 4
```

Example: v_f = 100 km/h, k_j = 140 veh/km/lane:
- q_max = 100 * 140 / 4 = 3500 -- way too high
- Real highways max around 2200, so Greenshields overestimates

**Greenberg's Logarithmic Model** (better for congested regime):

```
v(k) = v_c * ln(k_j / k)
```

This blows up as k approaches 0 (log of infinity), so it only works in congested conditions.

**Underwood's Exponential Model** (better for free-flow regime):

```
v(k) = v_f * exp(-k / k_c)
```

**Drake's Model** (good compromise):

```
v(k) = v_f * exp(-0.5 * (k/k_c)^2)
```

**For game implementation**, Greenshields is fine. It gives a clean parabolic curve, is trivially invertible, and the overestimation can be corrected by tuning k_j downward. Use v_f from `RoadType::speed()` and derive k_j from lane count and road type.

### 1.2 The BPR Function (Bureau of Public Roads)

The BPR function is the workhorse of static traffic assignment. Rather than modeling density explicitly, it relates **travel time** to the **volume-to-capacity ratio** (V/C):

```
t(V) = t_0 * (1 + alpha * (V/C)^beta)
```

Where:
- `t_0` = free-flow travel time (seconds or minutes)
- `V` = volume of traffic (vehicles per hour)
- `C` = capacity (vehicles per hour)
- `alpha` = calibration parameter (sensitivity at low congestion)
- `beta` = calibration parameter (sensitivity at high congestion)

**Standard parameters (FHWA, 1964):**
- alpha = 0.15, beta = 4.0

These original values are widely criticized. At V/C = 1.0, travel time only increases by 15% -- far too optimistic. Real-world measurements show 50-100% increases at capacity.

**Updated BPR parameters (recommended for games):**

| Road Type | alpha | beta | Rationale |
|-----------|-------|------|-----------|
| Local (2-lane) | 0.80 | 4.0 | High friction from driveways, parking, pedestrians |
| Avenue (4-lane) | 0.50 | 4.0 | Moderate friction, signalized intersections |
| Boulevard (6-lane) | 0.40 | 5.0 | Better operations, but signals still matter |
| Highway (4-lane divided) | 0.20 | 6.0 | Low friction, grade-separated, high beta for sharp breakdown |
| OneWay (2-lane) | 0.60 | 4.0 | No opposing traffic, but still urban friction |

**Why these values work for games:**

1. Higher alpha on urban roads creates visible congestion earlier (more responsive to player actions)
2. Higher beta on highways creates the "cliff" effect -- highways work great until they suddenly don't, which creates dramatic gameplay moments
3. At V/C = 0.8 (LOS D), Local roads show t = t_0 * (1 + 0.80 * 0.41) = 1.33 * t_0, a 33% delay -- noticeable
4. At V/C = 1.0 (capacity), Highway shows t = t_0 * (1 + 0.20) = 1.20 * t_0 -- still manageable
5. At V/C = 1.2 (oversaturated), Highway shows t = t_0 * (1 + 0.20 * 2.99) = 1.60 * t_0 -- 60% slowdown
6. At V/C = 1.5 (heavily oversaturated), Highway shows t = t_0 * (1 + 0.20 * 11.39) = 3.28 * t_0 -- gridlock

**Pseudocode for BPR-weighted pathfinding (integration with Megacity's CSR graph):**

```rust
fn compute_edge_weight(
    segment: &RoadSegment,
    traffic_volume: &TrafficVolumes,    // volume on this segment
    capacity: &SegmentCapacity,         // derived from road_type + lane_count
) -> f32 {
    let t_0 = segment.arc_length / segment.road_type.speed();  // free-flow time
    let v_over_c = traffic_volume.current / capacity.max_flow;

    let (alpha, beta) = match segment.road_type {
        RoadType::Local     => (0.80, 4.0),
        RoadType::Avenue    => (0.50, 4.0),
        RoadType::Boulevard => (0.40, 5.0),
        RoadType::Highway   => (0.20, 6.0),
        RoadType::OneWay    => (0.60, 4.0),
        RoadType::Path      => (0.0,  1.0),  // always free-flow
    };

    t_0 * (1.0 + alpha * v_over_c.powf(beta))
}
```

### 1.3 Level of Service (LOS)

Level of Service categorizes traffic conditions from A (best) to F (worst). These are defined differently for different facility types, but for game purposes, a V/C-based definition is cleanest:

| LOS | V/C Ratio | Description | Travel Time Multiplier (alpha=0.50, beta=4) |
|-----|-----------|-------------|----------------------------------------------|
| A | 0.00 - 0.35 | Free flow, no delays | 1.00 - 1.01 |
| B | 0.35 - 0.55 | Stable flow, slight delays | 1.01 - 1.05 |
| C | 0.55 - 0.75 | Stable flow, acceptable delays | 1.05 - 1.16 |
| D | 0.75 - 0.90 | Approaching instability | 1.16 - 1.33 |
| E | 0.90 - 1.00 | At capacity, unstable | 1.33 - 1.50 |
| F | > 1.00 | Forced flow, breakdown | > 1.50 (rapidly increasing) |

**For game visualization:**

```rust
enum LevelOfService { A, B, C, D, E, F }

fn los_from_vc(v_over_c: f32) -> LevelOfService {
    match v_over_c {
        x if x <= 0.35 => LevelOfService::A,
        x if x <= 0.55 => LevelOfService::B,
        x if x <= 0.75 => LevelOfService::C,
        x if x <= 0.90 => LevelOfService::D,
        x if x <= 1.00 => LevelOfService::E,
        _              => LevelOfService::F,
    }
}

fn los_color(los: LevelOfService) -> Color {
    match los {
        LevelOfService::A => Color::rgb(0.0, 0.8, 0.0),   // green
        LevelOfService::B => Color::rgb(0.4, 0.9, 0.0),   // yellow-green
        LevelOfService::C => Color::rgb(0.9, 0.9, 0.0),   // yellow
        LevelOfService::D => Color::rgb(1.0, 0.6, 0.0),   // orange
        LevelOfService::E => Color::rgb(1.0, 0.2, 0.0),   // red-orange
        LevelOfService::F => Color::rgb(0.8, 0.0, 0.0),   // red
    }
}
```

This maps directly to Megacity's traffic overlay system. Each road segment stores its current V/C ratio, and the overlay renders segments colored by LOS.

### 1.4 Cell Transmission Model (CTM)

The Cell Transmission Model (Daganzo, 1994) discretizes a road into cells and propagates traffic in timesteps. It is a numerical scheme for solving the LWR (Lighthill-Whitham-Richards) partial differential equation.

**Setup:**
- Divide each road link into cells of length `delta_x`
- Each timestep is `delta_t`
- Constraint: `delta_x / delta_t >= v_f` (CFL condition -- information cannot travel faster than one cell per timestep)
- Each cell i has occupancy `n_i(t)` (number of vehicles in the cell)
- Each cell has maximum occupancy `N_i = k_j * delta_x` (jam density times cell length)

**Flow update rule:**

```
y_{i->i+1}(t) = min(
    n_i(t),                                    // sending flow (what upstream cell wants to send)
    Q_max * delta_t,                           // capacity constraint
    w/v_f * (N_{i+1} - n_{i+1}(t))            // receiving flow (what downstream cell can accept)
)
```

Where:
- `Q_max` = maximum flow rate (veh/hr) = capacity
- `w` = backward wave speed (typically v_f/3 to v_f/5, around 20 km/h for highways)
- The third term is critical: it models **backward-propagating congestion waves**

**Occupancy update:**

```
n_i(t + delta_t) = n_i(t) + y_{i-1->i}(t) - y_{i->i+1}(t)
```

**Why CTM matters for games:**

1. It naturally produces shockwaves -- when a highway backs up, the congestion propagates backward at a realistic speed
2. It handles merge/diverge points cleanly
3. It can run at coarse resolution (cells = road segments, delta_t = 1 game-minute)

**Simplified CTM for Megacity:**

Rather than subdividing each road segment into many cells, treat each segment as a single cell. The segment's length determines its capacity to hold vehicles. Flow between segments follows the CTM rules.

```rust
struct TrafficCell {
    occupancy: f32,           // current vehicles in this segment
    max_occupancy: f32,       // k_j * segment_length * lane_count
    max_flow: f32,            // capacity in veh per game-tick
    backward_wave_speed: f32, // w, controls congestion propagation
}

fn ctm_flow(upstream: &TrafficCell, downstream: &TrafficCell) -> f32 {
    let send = upstream.occupancy;
    let receive = (downstream.max_occupancy - downstream.occupancy)
                  * (downstream.backward_wave_speed / FREE_FLOW_SPEED);
    let capacity = upstream.max_flow;

    send.min(capacity).min(receive)
}
```

**Performance note:** With Megacity's 256x256 grid and typical road coverage of 15-25%, there might be 3000-6000 road segments. Iterating all segment pairs each tick is O(n) in edges -- very fast, under 100 microseconds even without SIMD.

### 1.5 Nagel-Schreckenberg Model

The Nagel-Schreckenberg (NaSch) model is a cellular automaton for traffic. Each road is divided into cells of ~7.5 meters (one car length). Each cell is either empty or contains one vehicle with integer speed 0..v_max.

**Update rules (applied to all vehicles simultaneously):**

1. **Acceleration:** `v = min(v + 1, v_max)` -- drivers want to go faster
2. **Deceleration:** `v = min(v, gap)` -- but cannot exceed gap to car ahead
3. **Randomization:** With probability `p`, `v = max(v - 1, 0)` -- random braking (models human reaction time variance)
4. **Movement:** advance by `v` cells

Where `gap` = number of empty cells between this vehicle and the next vehicle ahead.

**Key parameter: the randomization probability p**

| p value | Behavior | Use case |
|---------|----------|----------|
| 0.0 | Deterministic, no spontaneous jams | Unrealistic but good for testing |
| 0.1 | Very smooth flow, rare jams | Highways with cooperative driving |
| 0.2 | Moderate jam formation | Typical highway |
| 0.3 | Frequent spontaneous jams | Congested conditions |
| 0.5 | Very unstable, constant stop-and-go | Bad driving / poor road conditions |

**Mapping to game scale:**

NaSch at full resolution (7.5m cells) on a 256x256 grid with 16m cell size would require subdividing each game cell into ~2 NaSch cells. For a typical road network of 5000 game-cells, that is 10,000 NaSch cells. With 1000 vehicles, the model runs in microseconds.

However, NaSch is an **agent-based microscopic model** -- every vehicle is tracked individually. This conflicts with the desire for aggregate/statistical traffic. It is best used selectively:

- Use NaSch only for visible roads near the camera
- Use BPR/aggregate for everything else
- Transition agents between models at LOD boundaries

**NaSch pseudocode:**

```rust
struct NaSchVehicle {
    cell: u32,          // position along the road (in NaSch cells)
    speed: u32,         // current speed (NaSch cells per tick)
    v_max: u32,         // max speed for this vehicle
}

fn update_nasch(vehicles: &mut [NaSchVehicle], road_length: u32, p: f32) {
    // Sort by position (ascending)
    vehicles.sort_by_key(|v| v.cell);

    for i in 0..vehicles.len() {
        let gap = if i + 1 < vehicles.len() {
            vehicles[i + 1].cell - vehicles[i].cell - 1
        } else {
            road_length - vehicles[i].cell - 1  // wrap or road end
        };

        // 1. Acceleration
        let mut v = (vehicles[i].speed + 1).min(vehicles[i].v_max);

        // 2. Deceleration
        v = v.min(gap);

        // 3. Randomization
        if rand::random::<f32>() < p && v > 0 {
            v -= 1;
        }

        vehicles[i].speed = v;
    }

    // 4. Movement (separate pass to avoid interference)
    for v in vehicles.iter_mut() {
        v.cell += v.speed;
    }
}
```

### 1.6 Statistical/Aggregate Traffic Approaches

For a city of 100K+ citizens, simulating each vehicle individually is expensive. Statistical approaches model traffic as flows on links rather than individual agents.

**Approach 1: Volume Accumulation**

Instead of routing individual vehicles, accumulate trip counts on links:

```
For each origin-destination pair (O, D):
    path = shortest_path(O, D)  // using current BPR-weighted costs
    trip_count = demand(O, D)    // from trip generation model
    for each link in path:
        link.volume += trip_count
```

This gives you volume on every link. Apply BPR to get travel times. This is the basis of **static traffic assignment**.

**Approach 2: Stochastic Sampling**

Don't route every citizen. Instead:

1. Group citizens by origin zone and destination zone
2. Sample a representative subset (e.g., 10% of trips)
3. Route the sample, multiply volumes by 10
4. Apply BPR

Error decreases as sqrt(sample_size). With 10% sampling, relative error is ~3.2x what full enumeration would give -- acceptable for a game.

**Approach 3: Incremental Assignment**

Route trips in batches (e.g., 10% at a time), updating link costs between batches:

```
for iteration in 0..10:
    batch = total_demand / 10
    for each (O, D) pair:
        path = shortest_path(O, D)  // using CURRENT link costs
        for each link in path:
            link.volume += batch * demand(O, D) / total_demand
    update_link_costs()  // recalculate BPR with new volumes
```

This approximates user equilibrium (Wardrop's first principle: no traveler can reduce their travel time by unilaterally changing routes). It is not exact, but it is fast and good enough for games.

**Approach 4: Temporal Demand Profiles**

Traffic demand varies throughout the day. Apply a temporal profile to the daily demand:

```
Morning peak (7-9 AM):   1.8x average (home -> work/school)
Midday (9 AM - 4 PM):    0.7x average (shopping, errands)
Evening peak (4-7 PM):   1.6x average (work/school -> home)
Evening (7-11 PM):       0.5x average (entertainment, dining)
Night (11 PM - 6 AM):    0.1x average (minimal traffic)
```

These multipliers applied to the average hourly demand give realistic daily patterns. In-game, compress the 24-hour cycle as needed but preserve the relative ratios.

### 1.7 Frank-Wolfe Algorithm (Simplified)

The Frank-Wolfe algorithm solves the **user equilibrium traffic assignment problem**: find link flows such that no driver can reduce their travel time by switching routes.

The mathematical formulation minimizes:

```
Z(x) = sum over all links a of integral from 0 to x_a of t_a(w) dw
```

Where `x_a` is flow on link `a` and `t_a()` is the BPR function for that link. With BPR:

```
integral of t_0 * (1 + alpha * (w/C)^beta) dw from 0 to x
    = t_0 * x * (1 + alpha/(beta+1) * (x/C)^beta)
```

**Simplified Frank-Wolfe for games:**

```
INITIALIZE:
    x_a = 0 for all links a
    Compute t_a(0) = t_0_a for all links (free-flow times)
    Route ALL demand on shortest paths using free-flow times
    Set x_a = resulting flows

ITERATE (5-20 iterations is usually sufficient):
    1. Update costs: t_a = BPR(x_a) for all links
    2. Compute auxiliary flows y_a:
       Route ALL demand on shortest paths using current costs t_a
       Let y_a = resulting flows from this all-or-nothing assignment
    3. Line search: find lambda in [0,1] that minimizes Z(x + lambda*(y - x))
       Simple approximation: lambda = 2 / (iteration + 2)
    4. Update flows: x_a = x_a + lambda * (y_a - x_a)

CONVERGE when:
    gap = sum_a(t_a * (x_a - y_a)) / sum_a(t_a * x_a) < threshold
    For games, threshold = 0.01 (1% gap) is fine
```

**Performance for Megacity:**

With ~5000 road links and ~500 OD pairs (zone-to-zone), each iteration requires:
- BPR evaluation: 5000 multiplications -- trivial
- All-or-nothing assignment: 500 shortest-path computations on CSR graph
- Each shortest path on a 5000-node graph: ~0.5 ms with A*
- Total per iteration: ~250 ms
- 10 iterations: ~2.5 seconds

This is too slow for real-time but perfect for background computation. Run Frank-Wolfe every N game-minutes on a background thread, then interpolate link volumes between updates.

**Practical simplification:**

Skip Frank-Wolfe entirely and use incremental assignment (Section 1.6, Approach 3) with 4-5 iterations. The equilibrium will be approximate but visually indistinguishable. The key insight is that players care about *relative* congestion patterns (which roads are red vs green), not absolute travel times.

---

## 2. Intersection Design and Capacity

Intersections are the bottlenecks of any road network. A road segment can carry 1800 veh/hr/lane, but an intersection where two such roads cross might only pass 800 veh/hr through each approach. The capacity of a network is determined by its intersections, not its links. This is the single most important insight for traffic simulation.

### 2.1 Intersection Types and Capacities

**Unsignalized (Stop/Yield) Intersections:**

| Configuration | Capacity per approach | Typical delay | Use case |
|---------------|----------------------|---------------|----------|
| Two-way stop (minor road stops) | 300-600 veh/hr | 15-35 sec | Local-local, local-collector |
| All-way stop | 400-500 veh/hr (total) | 10-25 sec | Low-volume residential |
| Yield (roundabout-like priority) | 600-900 veh/hr | 5-15 sec | European-style, lower volume |

Gap acceptance governs unsignalized capacity. The minor road driver must find a gap in the major road traffic. The critical gap (minimum acceptable) is typically 5-7 seconds, and the follow-up headway (for queued vehicles accepting the same gap) is 2.5-3.5 seconds.

**Capacity formula for unsignalized (HCM method, simplified):**

```
c_x = v_c * exp(-v_c * t_c / 3600) / (1 - exp(-v_c * t_f / 3600))
```

Where:
- c_x = capacity of minor movement (veh/hr)
- v_c = conflicting traffic volume (veh/hr)
- t_c = critical gap (seconds)
- t_f = follow-up headway (seconds)

Example: v_c = 500 veh/hr, t_c = 6.0 sec, t_f = 3.0 sec:
- c_x = 500 * exp(-500*6/3600) / (1 - exp(-500*3/3600))
- c_x = 500 * exp(-0.833) / (1 - exp(-0.417))
- c_x = 500 * 0.435 / (1 - 0.659)
- c_x = 217.3 / 0.341
- c_x = 637 veh/hr

This drops sharply with increasing conflicting volume. At v_c = 1000, capacity falls to ~250 veh/hr. This is why unsignalized intersections fail at moderate volumes.

**Signalized Intersections:**

The saturation flow rate is the maximum throughput of an approach during a green phase. Standard value: **1800-1900 PCU/hr/lane** (PCU = passenger car units; trucks count as 1.5-2.5 PCU).

Actual capacity depends on the effective green time ratio:

```
c = s * (g/C)
```

Where:
- c = capacity (veh/hr/lane)
- s = saturation flow rate (1800 veh/hr/lane typical)
- g = effective green time for this phase (seconds)
- C = cycle length (seconds)

| Cycle Length | Phases | Green Ratio (g/C) | Capacity per lane | Total intersection throughput |
|-------------|--------|--------------------|--------------------|-------------------------------|
| 60 sec | 2-phase | 0.45 | 810 veh/hr | ~3240 (4 lanes, 2 approaches) |
| 90 sec | 3-phase | 0.35 | 630 veh/hr | ~5040 (8 lanes, 3 approaches) |
| 120 sec | 4-phase | 0.28 | 504 veh/hr | ~6050 (12 lanes, 4 approaches) |

**Lost time per phase change:** typically 4 seconds (2 sec all-red + 2 sec start-up lost time). With more phases, more time is wasted on transitions.

Total lost time per cycle:
```
L = n_phases * (l_start + l_clearance)
  = n_phases * (2 + 2)
  = 4 * n_phases seconds
```

Effective green ratio for balanced phases:
```
g_eff / C = (C - L) / (C * n_phases)
```

This is why traffic engineers prefer fewer phases -- each additional phase costs 4 seconds of capacity per cycle.

**Roundabouts:**

| Type | Diameter | Lanes | Capacity (total entering) | Best for |
|------|----------|-------|---------------------------|----------|
| Mini | 15-25m | 1 | 800-1200 veh/hr | Residential, traffic calming |
| Single-lane | 30-40m | 1 | 1200-1500 veh/hr | Moderate suburban |
| Multi-lane | 45-60m | 2 | 2000-2800 veh/hr | Arterial-arterial |
| Turbo | 35-50m | 2 (spiraling) | 2500-3500 veh/hr | High-volume, directional |

Roundabout capacity per entry (HCM 6th Edition):

```
c_entry = 1130 * exp(-0.0010 * v_c)    [single lane]
c_entry = 1130 * exp(-0.0007 * v_c)    [two lanes, entry]
```

Where v_c = conflicting circulating flow.

Roundabouts outperform signals when:
- Total intersection volume < 2500 veh/hr (single-lane) or < 4000 veh/hr (multi-lane)
- Volumes are roughly balanced across approaches
- Left turns (or right turns in left-driving countries) are a significant proportion

They underperform signals when:
- One approach dominates (> 60% of total volume)
- Very high pedestrian volumes (pedestrians block entries)
- Grade separation is needed

**Grade-Separated Interchanges:**

No conflicting movements means capacity is only limited by merge/diverge operations. Capacity per merge/diverge: ~1600-2000 veh/hr.

| Interchange Type | Footprint | Capacity | Cost ratio | Weaving? |
|------------------|-----------|----------|------------|----------|
| Diamond | Small (4 ramps) | Limited by surface signals | 1x | No |
| Parclo (partial clover) | Medium | Good for unbalanced flows | 2x | Some |
| Full cloverleaf | Large (4 loops) | ~7000 veh/hr total | 3x | Yes (problem!) |
| Stack (directional) | Very large | ~10,000+ veh/hr | 5-8x | No |
| Turbine / Windmill | Large | ~8,000-10,000 veh/hr | 4-6x | No |
| SPUI (single point) | Compact | Good urban | 2x | No |

The cloverleaf's Achilles heel is **weaving sections** -- where merging and diverging traffic must cross paths in a short distance. Weaving capacity is approximately:

```
c_weave = 1800 * (L_weave / 150)^0.5    [in veh/hr, for L_weave in meters]
```

For a typical cloverleaf with 200m weaving sections: c_weave = 1800 * 1.15 = 2077 veh/hr. This often becomes the bottleneck.

### 2.2 Signal Timing Calculation

**Webster's Formula for Optimal Cycle Length:**

```
C_opt = (1.5 * L + 5) / (1 - Y)
```

Where:
- C_opt = optimal cycle length (seconds)
- L = total lost time per cycle (seconds) = n_phases * 4
- Y = sum of critical flow ratios = sum of (q_i / s_i) for each phase i
- q_i = demand flow for critical movement in phase i
- s_i = saturation flow for that movement

Example: 2-phase intersection, each approach has 600 veh/hr demand, saturation flow 1800 veh/hr/lane:
- L = 2 * 4 = 8 seconds
- Y = 600/1800 + 600/1800 = 0.333 + 0.333 = 0.667
- C_opt = (1.5 * 8 + 5) / (1 - 0.667) = 17 / 0.333 = 51 seconds

Green splits proportional to demand:
- Phase 1 green = (C_opt - L) * (y_1 / Y) = (51 - 8) * 0.5 = 21.5 sec
- Phase 2 green = (C_opt - L) * (y_2 / Y) = (51 - 8) * 0.5 = 21.5 sec

**Practical limits:** C_opt should be clamped to [30, 120] seconds. Below 30 seconds, lost time dominates. Above 120 seconds, drivers become impatient and compliance drops.

**Adaptive signal timing for games:**

```rust
struct SignalController {
    phases: Vec<SignalPhase>,
    current_phase: usize,
    cycle_length: f32,       // seconds
    time_in_phase: f32,
    mode: SignalMode,
}

enum SignalMode {
    FixedTime,      // Webster's formula, recalculated every N minutes
    Actuated,       // extends green when vehicles detected, up to max
    Adaptive,       // fully responsive (SCOOT/SCATS-like)
}

struct SignalPhase {
    green_time: f32,         // seconds
    min_green: f32,          // minimum green (usually 7 sec for pedestrians)
    max_green: f32,          // maximum green extension
    movements: Vec<Movement>, // which turning movements are served
}

fn webster_timing(phases: &[SignalPhase], demands: &[f32], sat_flows: &[f32]) -> f32 {
    let n = phases.len();
    let total_lost = n as f32 * 4.0;  // 4 sec per phase change

    let y_sum: f32 = demands.iter()
        .zip(sat_flows.iter())
        .map(|(&d, &s)| d / s)
        .sum();

    if y_sum >= 0.95 {
        return 120.0;  // oversaturated, use max cycle
    }

    let c_opt = (1.5 * total_lost + 5.0) / (1.0 - y_sum);
    c_opt.clamp(30.0, 120.0)
}
```

### 2.3 Turn Lanes and Movement Capacities

Turning movements at intersections have different capacities:

| Movement | Capacity factor | Notes |
|----------|----------------|-------|
| Through | 1.00 | Baseline |
| Right turn (free, with merge lane) | 0.85 | Yield to pedestrians |
| Right turn (signalized, no overlap) | 0.85 | |
| Right turn on red (permitted) | adds ~100-200 veh/hr | |
| Left turn (protected, dedicated lane) | 0.95 | Own signal phase |
| Left turn (permitted, gap acceptance) | 0.40-0.80 | Depends on opposing volume |
| Left turn (no dedicated lane) | 0.30-0.50 | Blocks through traffic when waiting |
| U-turn | 0.60 | If permitted |

**Left turns are the enemy of capacity.** An unprotected left turn from a shared lane can reduce that lane's throughput by 50-70%. This is why:

1. Dedicated left turn lanes are essential on arterials
2. Protected left-turn phases are needed at high volumes (> 200 left-turners/hr)
3. Roundabouts eliminate the left-turn problem entirely (all turns are right-merges)
4. Michigan lefts (indirect lefts via U-turn downstream) recover capacity

**For game implementation:**

Track intersection approach geometry:
- Number of through lanes
- Presence/absence of dedicated turn lanes (left and right)
- Signal phasing (protected left vs permitted vs split)

The capacity of an intersection approach is approximately:

```
C_approach = sum over lanes of (s_lane * g_effective / C) * movement_factor
```

**Simplified intersection capacity for Megacity:**

```rust
fn intersection_capacity(
    road_a: RoadType,
    road_b: RoadType,
    intersection_type: IntersectionType,
) -> f32 {
    let base_saturation = 1800.0;  // veh/hr/lane

    match intersection_type {
        IntersectionType::Unsignalized => {
            // Minor road limited by gap acceptance
            let minor = road_a.lane_count().min(road_b.lane_count()) as f32;
            minor * 400.0  // very rough
        }
        IntersectionType::Signalized { phases } => {
            let total_lanes = (road_a.lane_count() + road_b.lane_count()) as f32;
            let lost_time = phases as f32 * 4.0;
            let cycle = 90.0;  // reasonable default
            let green_ratio = (cycle - lost_time) / cycle / phases as f32;
            total_lanes * base_saturation * green_ratio
        }
        IntersectionType::Roundabout { lanes } => {
            match lanes {
                1 => 1400.0,
                2 => 2600.0,
                _ => 3200.0,
            }
        }
        IntersectionType::GradeSeparated => {
            // Only merge/diverge limits, effectively unlimited for intersection
            let total = (road_a.lane_count() + road_b.lane_count()) as f32;
            total * 1800.0  // no intersection delay
        }
    }
}
```

### 2.4 Intersection Delay Models

**Webster's Delay Formula (uniform delay component):**

```
d_1 = C * (1 - g/C)^2 / (2 * (1 - min(1, x) * g/C))
```

Where x = V/C (degree of saturation).

**Random/overflow delay component (accounts for queue variability):**

```
d_2 = x^2 / (2 * q * (1 - x))    [for x < 1]
```

For x >= 1, the queue grows without bound. In practice, use:

```
d_2 = 900 * T * [(x - 1) + sqrt((x - 1)^2 + 8*k*I*x / (c*T))]
```

Where T = analysis period (typically 0.25 hr = 15 min), k = delay parameter (0.5 for pre-timed), I = upstream filtering adjustment (usually 1.0).

**Total average delay:**

```
d = d_1 + d_2
```

For game purposes, a simplified delay model suffices:

```rust
fn intersection_delay(v_over_c: f32, cycle_length: f32, green_ratio: f32) -> f32 {
    // Uniform delay (simplified Webster)
    let d1 = 0.5 * cycle_length * (1.0 - green_ratio).powi(2)
             / (1.0 - green_ratio * v_over_c.min(1.0));

    // Overflow delay (simplified HCM)
    let d2 = if v_over_c < 1.0 {
        4.0 * v_over_c.powi(2) / (1.0 - v_over_c).max(0.01)
    } else {
        30.0 * (v_over_c - 1.0) + 50.0  // linear growth above capacity
    };

    d1 + d2  // total delay in seconds per vehicle
}
```

### 2.5 Intersection Spacing and Coordination

**Signal coordination (green waves):**

When intersections along a corridor are spaced regularly and timed with offsets, vehicles traveling at the design speed can hit green lights consecutively. This is a "green wave."

Offset calculation:

```
offset_i = distance_i / v_design    [mod cycle_length]
```

Where distance_i = distance from the reference intersection to intersection i.

**Bandwidth** = the width of the green band (time window during which vehicles can pass through all signals without stopping). Maximum bandwidth = g_min - travel time variability.

For a corridor with 90-second cycles and 40% green, the maximum bandwidth is ~36 seconds. In practice, achieving 60-70% of theoretical maximum is good.

**Spacing and capacity interaction:**

| Spacing | Signals per km | Green wave feasible? | Capacity impact |
|---------|---------------|---------------------|-----------------|
| 100m | 10 | Very difficult | Severe degradation |
| 200m | 5 | Difficult | Significant degradation |
| 400m | 2.5 | Feasible, 2-phase only | Moderate |
| 800m | 1.25 | Good | Minor |
| 1600m+ | < 1 | Excellent | Minimal |

**For Megacity:** Given CELL_SIZE = 16m, intersection spacing on a grid is 16m * N where N = blocks between intersections. Encourage players to space arterials at 400-800m (25-50 cells) apart for good signal coordination. The game should auto-calculate signal offsets for corridors.

---

## 3. Road Network Hierarchy and Design

### 3.1 The Functional Classification Hierarchy

Road networks are organized into a strict hierarchy where each level serves a different function. The hierarchy exists because **mobility and access are competing goals**: a road that serves high-speed through traffic cannot simultaneously provide direct access to driveways and buildings.

| Level | Function | Mobility | Access | Megacity RoadType |
|-------|----------|----------|--------|-------------------|
| Local | Access to properties | Low | High | `RoadType::Local` |
| Collector | Gather local traffic to arterials | Medium | Medium | `RoadType::Avenue` |
| Minor Arterial | Distribute traffic within districts | Medium-High | Low | `RoadType::Boulevard` |
| Major Arterial / Expressway | Move traffic between districts | High | Very Low | (could add) |
| Freeway / Highway | Move traffic between regions | Very High | None (interchanges only) | `RoadType::Highway` |

**Capacity by classification (per direction):**

| Classification | Lanes (per dir) | Speed (km/h) | Capacity (veh/hr/dir) | Spacing | % of Network Length |
|---------------|-----------------|--------------|----------------------|---------|---------------------|
| Local | 1 | 30-40 | 400-800 | 50-150m | 65-80% |
| Collector | 1-2 | 40-50 | 800-1600 | 200-400m | 10-15% |
| Minor Arterial | 2-3 | 50-60 | 1600-3600 | 400-800m | 5-8% |
| Major Arterial | 3-4 | 60-80 | 3600-6000 | 800-1600m | 3-5% |
| Freeway | 2-4 | 80-120 | 3600-8000 | 3-10 km | 1-3% |

The percentages above are remarkably consistent across cities worldwide. A city that violates this hierarchy -- for example, by having too many arterials and too few locals -- will have problems.

**The hierarchy principle for games:**

Traffic should flow "up" the hierarchy to travel long distances, then "down" to reach destinations. A trip from home to work might use: Local -> Collector -> Arterial -> Highway -> Arterial -> Collector -> Local. The game's pathfinding (currently in `csr_find_path_with_traffic()`) should naturally produce this pattern if edge weights reflect the speed/capacity differences.

**The access management implication:**

Megacity already encodes this: `RoadType::allows_zoning()` returns false for Highway, meaning buildings cannot front directly onto highways. This is correct. Freeways should only be accessible via interchanges (ramps connecting to the surface network).

### 3.2 Capacity Numbers for Megacity's Road Types

Mapping Megacity's existing `RoadType` enum to engineering capacities:

```rust
impl RoadType {
    /// Capacity in vehicles per hour per direction
    pub fn directional_capacity(self) -> f32 {
        match self {
            // 2 lanes total = 1 per direction, reduced by parking/friction
            RoadType::Local => 600.0,

            // 4 lanes total = 2 per direction, signalized intersections
            RoadType::Avenue => 1400.0,

            // 6 lanes total = 3 per direction, coordinated signals
            RoadType::Boulevard => 2800.0,

            // 4 lanes divided = 2 per direction, grade-separated, high speed
            RoadType::Highway => 3600.0,

            // 2 lanes one-way = 2 in same direction, no opposing conflict
            RoadType::OneWay => 1200.0,

            // Pedestrian only
            RoadType::Path => 0.0,
        }
    }

    /// Vehicles that can physically be on this segment per km (jam density)
    pub fn jam_density_per_km(self) -> f32 {
        match self {
            RoadType::Local => 100.0,      // 1 lane, 10m spacing at jam
            RoadType::Avenue => 200.0,     // 2 lanes/dir
            RoadType::Boulevard => 300.0,  // 3 lanes/dir
            RoadType::Highway => 280.0,    // 2 lanes/dir, longer vehicles at speed
            RoadType::OneWay => 200.0,     // 2 lanes same dir
            RoadType::Path => 0.0,
        }
    }
}
```

**Why the Highway has fewer lanes per direction than Boulevard but more capacity:**

The Highway (4 lanes, 2/direction) has higher capacity per lane than the Boulevard (6 lanes, 3/direction) because:
1. No traffic signals: no lost time from red phases
2. No pedestrian conflicts
3. No driveway access friction
4. Higher speed means lower headways in time (even if spatial headways are larger)
5. Each highway lane can sustain ~1800 veh/hr vs ~900 effective veh/hr for a signalized boulevard lane

### 3.3 Why Grid Networks Work (and When They Don't)

Megacity uses a 256x256 grid, which naturally produces grid-pattern road networks. This is not a limitation -- grid networks have significant advantages.

**Advantages of grid networks:**

1. **Route redundancy:** Between any two points on a grid, there are many possible paths of equal or near-equal length. If one road is congested, traffic naturally distributes to parallel routes. On a tree/cul-de-sac network, there is exactly one path, creating bottlenecks.

2. **Load distribution:** On a perfect grid with N parallel routes, each route carries approximately 1/N of the traffic. This means capacity scales linearly with the number of parallel streets.

3. **Intersection simplicity:** Grid intersections are all 4-way (90-degree), which are the simplest to signal-time and the most efficient in terms of conflict points (32 conflict points for a 4-way vs. 0 for a T-intersection, but 4-way handles more volume).

4. **Walkability:** Grid networks have the shortest pedestrian distances because they minimize detours. The "pedestrian detour factor" for a grid is pi/4 = 0.785 (meaning a pedestrian travels on average 27% farther than the straight-line distance, vs. 50-100% farther on cul-de-sac networks).

5. **Transit efficiency:** Bus routes on grids can run on straight corridors. On curvilinear networks, buses must follow winding paths, adding 20-40% to route length.

**Quantifying grid efficiency:**

For a grid with block size B (meters) and a trip of straight-line distance D:

```
Manhattan distance = D * 4/pi   (on average, for random orientation)
                   = D * 1.273

Average detour factor for grid: 1.27
Average detour factor for cul-de-sac: 1.6 - 2.0
Average detour factor for radial: 1.1 - 1.3 (good for center-oriented trips)
```

**When grids fail:**

1. **Terrain:** Grids on steep hills create dangerously steep roads. San Francisco's grid on its hills was a planning mistake.
2. **Through traffic:** All streets on a grid are equally accessible, so cut-through traffic on residential streets is a constant problem. Hierarchical networks prevent this by making locals discontinuous.
3. **Speed:** Grid intersections every block force stops. A collector/arterial hierarchy with fewer intersections allows higher sustained speeds.

**Hybrid approach for Megacity:**

The game should reward players who build a hierarchical grid:
- Locals every 50-100m (3-6 cells)
- Collectors every 200-400m (12-25 cells), connecting locals to arterials
- Arterials every 400-800m (25-50 cells), with signal coordination
- Highways on the periphery or as ring roads, with interchanges every 1-2 km (60-125 cells)

The key rule: **locals should not connect directly to arterials.** Require collectors as intermediaries. This prevents the "everything connects to everything" problem that makes traffic unsolvable in most city builders.

### 3.4 Interchange Types and When to Use Them

In Megacity, where Highway meets Highway or Highway meets Boulevard, the player needs to build an interchange. Different types suit different situations.

**Diamond Interchange:**

```
         |
    -----+-----   <- surface intersection (signalized)
    |    |    |
====|====|====|====  <- highway (grade-separated)
    |    |    |
    -----+-----   <- surface intersection (signalized)
         |
```

- Footprint: ~4x4 cells (64m x 64m)
- Capacity: Limited by the two surface signals (~1200-1600 veh/hr per ramp)
- Cost: Low (4 ramps + 2 signals)
- Best for: Highway-arterial, low-to-moderate ramp volumes
- Game implementation: Player places highway crossing, game auto-generates ramps

**Parclo (Partial Cloverleaf):**

Two of the four turning movements use loops (no signal), two use ramps to surface signals.

- Footprint: ~6x6 cells
- Capacity: Better than diamond for the loop movements
- Cost: Medium
- Best for: When one pair of turning movements is much heavier than the other

**Full Cloverleaf:**

Four loop ramps eliminate all signals. Traffic weaves between ramps on the highway.

- Footprint: ~10x10 cells (160m x 160m) -- very large
- Capacity: Good, but limited by weaving sections (see Section 2.1)
- Cost: High
- Best for: Highway-highway outside urban areas
- Problem: Weaving sections can become bottlenecks at high volumes

**Stack Interchange (Directional):**

Multi-level flyover ramps for all turning movements. No weaving.

- Footprint: ~8x8 cells but multiple levels
- Capacity: Highest (10,000+ veh/hr)
- Cost: Very high (4-5x diamond)
- Best for: Major highway-highway junctions
- Game implementation: Most expensive but eliminates all bottlenecks

**Single-Point Urban Interchange (SPUI):**

All ramp terminals merge into one large signalized intersection with a single signal controller.

- Footprint: ~5x5 cells
- Capacity: Good for unbalanced flows (~6000-8000 veh/hr total)
- Cost: Medium
- Best for: Urban highway-arterial with limited right-of-way
- Advantage: Only one signal to coordinate, fewer stops than diamond

### 3.5 One-Way Streets: Tradeoffs

Megacity includes `RoadType::OneWay`. Here is why one-way streets exist and when they help.

**Capacity gains:**

A two-lane one-way street has higher capacity than a two-lane two-way street because:
1. No opposing traffic: no head-on conflict, wider effective lanes
2. Signal timing: all movements are same-direction, green time = nearly 100% minus cross-street time
3. Left turns: no opposing traffic to yield to -- left turns from one-way are just like right turns
4. Parking: can park on both sides without confusing traffic direction

Typical capacity comparison:
- 2-lane two-way: 600 veh/hr per direction (1 lane each way)
- 2-lane one-way: 1200 veh/hr (both lanes same direction)
- Net: 100% capacity increase in the favored direction

**One-way pair configuration:**

Converting a two-way arterial to a one-way pair (two parallel one-way streets, one block apart):
- Doubles capacity
- Enables signal progression at any speed (no need to time for both directions)
- Reduces pedestrian-vehicle conflicts by 50% (pedestrians only cross one direction)

**Disadvantages:**

1. **Out-of-direction travel:** Vehicles must travel past their destination and circle back. Average additional distance: 1/2 block length. For 200m blocks, this adds ~100m per trip.
2. **Speed increase:** One-way streets encourage faster driving, worse for safety
3. **Navigation confusion:** Visitors and delivery drivers get lost
4. **Local business impact:** Businesses on one-way streets have less drive-by visibility from one direction
5. **Transit penalty:** Bus routes must split onto two streets, confusing riders

**For Megacity gameplay:**

One-way streets should be a tool for experienced players. Benefits:
- Higher throughput on congested corridors
- Can create one-way pairs for downtown grids

Costs:
- Slight reduction in land value (less access)
- Increased travel distance for some trips
- Pathfinding must respect directionality (already handled by directed edges in CsrGraph)

### 3.6 Road Network Metrics

To evaluate network quality, the game can compute these metrics:

**Connectivity Index (gamma):**

```
gamma = e / (3 * (v - 2))
```

Where e = number of edges, v = number of nodes (intersections). Range: 0 to 1. A perfect grid scores ~0.67. Below 0.3 indicates a tree-like network with poor redundancy.

**Circuity (detour factor):**

```
circuity = average(network_distance / straight_line_distance)
```

For all OD pairs. A perfect grid scores ~1.27. Above 1.5 indicates poor network design.

**Lane-km per capita:**

```
lane_km = sum of (segment_length * lane_count) for all segments
lane_km_per_1000 = lane_km / (population / 1000)
```

Typical values:
- Dense urban: 2-4 lane-km per 1000 people
- Suburban: 6-10 lane-km per 1000 people
- Sprawl: 12-20 lane-km per 1000 people

**Intersection density:**

```
intersections_per_km2 = count(intersections) / city_area_km2
```

Typical values:
- Grid downtown: 100-200 per km2
- Suburban: 30-60 per km2
- Cul-de-sac: 10-30 per km2 (but most are dead ends)

---

## 4. Public Transit Simulation

### 4.1 The Four-Step Travel Demand Model

The four-step model is the standard framework for forecasting travel demand, including transit ridership. It runs in sequence:

**Step 1: Trip Generation**

How many trips originate from and are attracted to each zone?

```
Productions_i = a * Households_i + b * Workers_i
Attractions_j = c * Employment_j + d * Retail_sqft_j + e * School_enrollment_j
```

Typical production rates (trips per household per day):
- Single-family household: 9.5 trips/day
- Apartment household: 6.6 trips/day
- Average: 7.5-8.5 trips/day

Typical attraction rates (trips per employee per day):
- Office: 3.3 trips/employee/day
- Retail: 12.0 trips/employee/day (high because of customer trips)
- Industrial: 2.5 trips/employee/day

Balance productions and attractions so total productions = total attractions.

**For Megacity:** Each residential building generates trips proportional to its population. Each commercial/industrial/office building attracts trips proportional to its employment. The game already tracks population per building and jobs per zone.

**Step 2: Trip Distribution**

Which zones do trips go between? The gravity model:

```
T_ij = P_i * A_j * f(c_ij) / sum_j(A_j * f(c_ij))
```

Where:
- T_ij = trips from zone i to zone j
- P_i = productions in zone i
- A_j = attractions in zone j
- f(c_ij) = friction factor, a function of travel cost/time between i and j
- Common friction function: f(c) = c^(-beta) or f(c) = exp(-beta * c)

Beta (impedance parameter) values:
- Home-work trips: beta = 0.08 - 0.12 (willing to travel far)
- Home-shopping trips: beta = 0.15 - 0.25 (prefer nearby)
- Home-school trips: beta = 0.20 - 0.30 (very local)
- Home-recreation: beta = 0.10 - 0.15 (moderate)

**Pseudocode:**

```rust
fn gravity_model(
    productions: &[f32],       // trips produced by each zone
    attractions: &[f32],       // trips attracted to each zone
    travel_times: &[Vec<f32>], // travel time matrix [i][j]
    beta: f32,                 // impedance parameter
) -> Vec<Vec<f32>> {           // trip matrix T[i][j]
    let n = productions.len();
    let mut trips = vec![vec![0.0f32; n]; n];

    for i in 0..n {
        // Denominator: sum of A_j * f(c_ij)
        let denom: f32 = (0..n)
            .map(|j| attractions[j] * (-beta * travel_times[i][j]).exp())
            .sum();

        if denom < 1e-6 { continue; }

        for j in 0..n {
            let friction = (-beta * travel_times[i][j]).exp();
            trips[i][j] = productions[i] * attractions[j] * friction / denom;
        }
    }

    trips
}
```

**Step 3: Mode Choice**

What mode does each trip use? This is where transit ridership is determined.

The **multinomial logit model** is standard:

```
P(mode m) = exp(V_m) / sum_k(exp(V_k))
```

Where V_m is the "utility" of mode m, a linear function of attributes:

```
V_auto = beta_time * time_auto + beta_cost * cost_auto + ASC_auto
V_transit = beta_time * time_transit + beta_cost * cost_transit
          + beta_wait * wait_time + beta_walk * walk_time + ASC_transit
V_walk = beta_time * time_walk + ASC_walk
V_bike = beta_time * time_bike + ASC_bike
```

**Typical coefficient values (from empirical models):**

| Coefficient | Value | Units | Interpretation |
|-------------|-------|-------|----------------|
| beta_time (in-vehicle) | -0.03 to -0.05 | per minute | Each minute of in-vehicle time reduces utility by 0.03-0.05 |
| beta_time (wait) | -0.06 to -0.10 | per minute | Waiting is valued at 2-3x in-vehicle time |
| beta_time (walk) | -0.06 to -0.10 | per minute | Walking is valued at 2-3x in-vehicle time |
| beta_cost | -0.005 to -0.02 | per cent | Higher income = lower sensitivity to cost |
| ASC_auto | +0.5 to +2.0 | - | Auto mode preference (cultural, convenience) |
| ASC_transit | 0.0 (reference) | - | Transit as baseline |
| ASC_walk | -1.0 to +0.5 | - | Varies by city (negative in US, positive in Europe) |

**Example calculation:**

Trip: 10 km, downtown.
- Auto: 15 min driving + 5 min parking search = 20 min, cost $5 (gas + parking)
- Transit: 5 min walk + 3 min wait + 18 min ride = 26 min, cost $2.50
- Walk: 120 min, cost $0

```
V_auto   = -0.04*20 + -0.01*500 + 1.0 = -0.80 + -5.00 + 1.0 = -4.80
V_transit = -0.04*18 + -0.08*3 + -0.08*5 + -0.01*250 + 0.0 = -0.72 - 0.24 - 0.40 - 2.50 = -3.86
V_walk   = -0.04*120 + -0.5 = -4.80 - 0.50 = -5.30

P(auto) = exp(-4.80) / (exp(-4.80) + exp(-3.86) + exp(-5.30))
        = 0.00823 / (0.00823 + 0.02105 + 0.00499)
        = 0.00823 / 0.03427
        = 0.240 (24%)

P(transit) = 0.02105 / 0.03427 = 0.614 (61.4%)
P(walk) = 0.00499 / 0.03427 = 0.146 (14.6%)
```

Transit wins because of parking cost and the competitive travel time. Change parking to $0 and auto jumps to ~65%.

**Simplified mode choice for Megacity:**

```rust
fn mode_choice(
    auto_time: f32,       // minutes (including parking search)
    auto_cost: f32,       // dollars
    transit_time: f32,    // total minutes (walk + wait + ride)
    transit_wait: f32,    // wait time specifically
    transit_walk: f32,    // walk to/from stop
    transit_cost: f32,    // fare
    walk_time: f32,       // minutes to walk entire trip
) -> ModeChoice {
    let v_auto = -0.04 * auto_time - 0.01 * auto_cost * 100.0 + 1.0;
    let v_transit = -0.04 * (transit_time - transit_wait - transit_walk)
                    - 0.08 * transit_wait
                    - 0.08 * transit_walk
                    - 0.01 * transit_cost * 100.0;
    let v_walk = if walk_time < 45.0 {
        -0.04 * walk_time - 0.5
    } else {
        f32::NEG_INFINITY  // nobody walks 45+ minutes
    };

    let exp_auto = v_auto.exp();
    let exp_transit = v_transit.exp();
    let exp_walk = v_walk.exp();
    let total = exp_auto + exp_transit + exp_walk;

    // Probabilistic choice
    let r: f32 = rand::random();
    if r < exp_auto / total {
        ModeChoice::Auto
    } else if r < (exp_auto + exp_transit) / total {
        ModeChoice::Transit
    } else {
        ModeChoice::Walk
    }
}
```

**Step 4: Traffic Assignment**

Already covered in Section 1.6-1.7 for auto traffic. For transit, assignment means loading passengers onto routes:

```
For each transit trip (O, D):
    Find best transit path: walk to stop -> ride route -> transfer -> ride route -> walk
    Add passenger to each route segment
    Track boardings/alightings at each stop
```

Transit pathfinding requires a **multi-modal graph** that includes:
- Walking links (between stops and zone centroids, between transfer stops)
- Waiting links (at each stop, with cost = expected wait time = headway/2)
- In-vehicle links (along each route, with cost = ride time)

### 4.2 Bus vs Rail: Breakeven Analysis

When does a city need rail? This is a capacity and cost question.

**Vehicle capacities:**

| Mode | Vehicle capacity | Vehicles/hr | Peak capacity (pphpd) | Capital cost/km |
|------|-----------------|-------------|----------------------|-----------------|
| Standard bus (12m) | 60-80 pass | 30-40 | 2,000-3,200 | $0.5-2M |
| Articulated bus (18m) | 100-120 pass | 25-30 | 2,500-3,600 | $1-3M |
| BRT (dedicated lane) | 120-160 pass | 40-60 | 5,000-9,600 | $5-30M |
| Light rail (2-car) | 200-300 pass | 20-30 | 4,000-9,000 | $30-80M |
| Light rail (4-car) | 400-600 pass | 15-20 | 6,000-12,000 | $40-100M |
| Metro/subway (6-car) | 900-1500 pass | 30-40 | 27,000-60,000 | $100-500M |
| Commuter rail (8-car) | 1000-2000 pass | 6-12 | 6,000-24,000 | $20-80M |

(pphpd = passengers per hour per direction)

**Breakeven ridership for different modes:**

The key question: at what ridership level does the higher capital cost of rail pay off through lower operating costs per passenger?

Operating cost per revenue-km:
- Bus: $3-6/km
- BRT: $4-8/km (higher because of larger vehicles, dedicated infrastructure maintenance)
- Light rail: $6-12/km
- Metro: $8-15/km

Operating cost per passenger-km (at different loads):
- Bus at 20 pass/trip: $0.15-0.30/pass-km
- Bus at 40 pass/trip: $0.08-0.15/pass-km
- Light rail at 200 pass/trip: $0.03-0.06/pass-km
- Metro at 800 pass/trip: $0.01-0.02/pass-km

**The crossover points:**

```
Bus → BRT:     When corridor exceeds 2000-3000 pphpd
BRT → Light Rail: When corridor exceeds 5000-8000 pphpd
Light Rail → Metro: When corridor exceeds 15,000-25,000 pphpd
```

**For Megacity gameplay:**

Tier the transit unlocks:
1. Population < 10K: Buses only
2. Population 10K-50K: Unlock BRT (dedicated bus lanes)
3. Population 50K-150K: Unlock light rail/tram
4. Population 150K+: Unlock metro/subway

This follows real-world patterns and creates meaningful upgrade decisions.

### 4.3 Transit Operations: Headway, Capacity, and Stop Spacing

**Headway** is the time between consecutive vehicles on a route. It is the single most important determinant of transit quality.

**Impact of headway on ridership:**

| Headway | Wait time (avg) | Rider experience | Mode share impact |
|---------|----------------|------------------|-------------------|
| 3-5 min | 1.5-2.5 min | "Forget the schedule" -- show up and go | High ridership |
| 5-10 min | 2.5-5 min | Convenient, check schedule occasionally | Good ridership |
| 10-15 min | 5-7.5 min | Noticeable waits, plan around schedule | Moderate ridership |
| 15-30 min | 7.5-15 min | Must plan trips carefully | Low ridership |
| 30-60 min | 15-30 min | Transit of last resort | Very low ridership |

The wait time penalty in mode choice models (Section 4.1) means that every minute of headway improvement is worth 2-3 minutes of travel time improvement. **A bus that comes every 5 minutes beats a train that comes every 15 minutes**, even if the train is faster.

**Frequency-ridership spiral:**

```
More frequency → Less waiting → More riders → More fare revenue → More frequency → ...
Less frequency → More waiting → Fewer riders → Less revenue → Less frequency → ...
```

This feedback loop means transit systems tend toward either a virtuous or vicious cycle. The game should model this explicitly: routes that dip below a minimum ridership threshold should trigger advisor warnings.

**Stop spacing optimization:**

Closer stops = shorter walk to stop, but slower service (each stop adds 20-40 seconds for deceleration, dwell, acceleration).

| Mode | Optimal stop spacing | Walk catchment | Dwell time |
|------|---------------------|----------------|------------|
| Local bus | 200-400m | 400m (5 min walk) | 15-30 sec |
| Express bus | 800-1600m | 800m (10 min walk or feeder) | 15-20 sec |
| BRT | 400-800m | 600m (7 min walk) | 10-20 sec |
| Light rail | 400-800m | 800m (10 min walk) | 20-30 sec |
| Metro | 800-1600m | 800-1200m (walk or feeder) | 20-30 sec |
| Commuter rail | 2000-5000m | 1500m+ (feeder bus or park-and-ride) | 30-60 sec |

**Speed vs. stop spacing formula:**

```
v_avg = L / (L/v_cruise + n_stops * t_dwell)
```

Where:
- L = route length
- v_cruise = speed between stops
- n_stops = number of stops = L / stop_spacing
- t_dwell = average dwell time per stop (seconds)

Example: 10 km bus route, v_cruise = 30 km/h, t_dwell = 25 sec:
- 200m spacing: 50 stops, time = 10/30 hr + 50*25/3600 hr = 0.333 + 0.347 = 0.680 hr, v_avg = 14.7 km/h
- 400m spacing: 25 stops, time = 0.333 + 0.174 = 0.507 hr, v_avg = 19.7 km/h
- 800m spacing: 12.5 stops, time = 0.333 + 0.087 = 0.420 hr, v_avg = 23.8 km/h

Doubling stop spacing from 200m to 400m increases average speed by 34%. This is why express/limited-stop services are effective.

**Capacity of a transit line:**

```
line_capacity = vehicle_capacity * vehicles_per_hour
vehicles_per_hour = 3600 / headway_seconds
```

Example: Standard bus (70 passengers), 5-minute headway:
- vehicles_per_hour = 3600/300 = 12
- line_capacity = 70 * 12 = 840 passengers/hr/direction

At crush load (standing room only, 120 passengers per bus):
- line_capacity = 120 * 12 = 1440 passengers/hr/direction

### 4.4 Bus Rapid Transit (BRT) Design

BRT is the sweet spot for many cities and for Megacity gameplay. It provides rail-like performance at a fraction of the cost.

**BRT elements and their impact:**

| Element | Speed improvement | Capacity improvement | Cost |
|---------|-------------------|---------------------|------|
| Dedicated lanes | +25-40% | Consistent (no traffic delays) | High (lane repurposing) |
| Level boarding (raised platforms) | +10-15% (faster dwell) | Slightly higher (faster loading) | Medium |
| Off-board fare collection | +5-10% (no fare delay) | +5-10% (faster boarding) | Low |
| Signal priority (TSP) | +5-15% (fewer red lights) | Minor | Low-Medium |
| Passing lanes at stations | Minor | +50-100% (express services) | Medium |
| Larger vehicles (articulated/bi-articulated) | Minor | +50-100% | Medium |

**BRT performance spectrum:**

| BRT Tier | Elements | Avg speed | Capacity (pphpd) | Cost/km |
|----------|----------|-----------|-------------------|---------|
| BRT Lite | Signal priority + limited stops | 18-22 km/h | 2000-3000 | $2-5M |
| Full BRT | Dedicated lane + platforms + TSP | 22-28 km/h | 4000-6000 | $5-15M |
| Gold BRT | Full separation + stations + passing | 25-35 km/h | 8000-15000 | $10-30M |

**For Megacity implementation:**

BRT can be modeled as a RoadType variant or overlay. A boulevard with a dedicated bus lane loses 1 general-purpose lane (reducing auto capacity by ~33%) but gains a high-capacity transit corridor.

```rust
struct BusRoute {
    id: u32,
    stops: Vec<StopId>,           // ordered list of stops
    headway_seconds: f32,         // target headway
    vehicle_type: BusType,        // standard, articulated, bi-articulated
    has_dedicated_lane: bool,     // BRT vs mixed traffic
    has_signal_priority: bool,    // TSP
    ridership_daily: f32,         // tracked for performance
    fare: f32,                    // dollars per trip
    operating_cost_hourly: f32,   // dollars per vehicle-hour
}

enum BusType {
    Standard,       // 70-80 pass, $300K
    Articulated,    // 100-120 pass, $700K
    BiArticulated,  // 150-200 pass, $1.2M
    Electric,       // 60-70 pass, $600K, lower operating cost
}
```

### 4.5 Transit Network Design Patterns

**Grid network** (Los Angeles, Toronto):
- Bus routes run on a regular grid of streets
- Transfers between perpendicular routes
- Coverage: excellent (every block within walking distance)
- Frequency: spread thin (each route gets less service)
- Best for: cities with dispersed destinations

**Radial network** (London, Paris, Moscow):
- Routes converge on CBD
- Excellent for CBD-bound trips
- Terrible for suburb-to-suburb trips (must go through center)
- Ring routes can help but are less efficient

**Hub-and-spoke** (Singapore):
- Major stations/hubs connected by high-frequency trunk lines
- Feeder buses connect neighborhoods to hubs
- Efficient use of resources (high frequency on trunks)
- Forced transfers (negative for riders)

**Pulse/timed-transfer** (Zurich):
- All routes arrive at key hubs simultaneously
- Brief transfer window (2-5 min), then all depart
- Minimizes transfer wait time
- Requires very reliable operations

**For Megacity:** Start with a grid pattern for buses (simple, good coverage), allow players to upgrade key corridors to BRT or rail. The game should evaluate transit coverage: what percentage of buildings are within 400m (5 min walk) of a stop? Cities should target 80%+ coverage.

### 4.6 Transfer Penalties and Network Effects

Transfers kill ridership. Each required transfer typically reduces the probability of choosing transit by 10-20%.

**Transfer penalty values for mode choice models:**

| Transfer type | Equivalent minutes of in-vehicle time |
|---------------|--------------------------------------|
| Cross-platform (same station, <2 min) | 5-8 min |
| Same-stop (bus-to-bus, <5 min) | 8-12 min |
| Walk transfer (200-400m) | 12-18 min |
| Long walk transfer (400m+) | 15-25 min |

These are "perceived" penalties, not actual time. A 3-minute platform transfer that "costs" 8 minutes in the model captures the psychological burden: uncertainty, discomfort of waiting, risk of missing connection.

**Network connectivity multiplier:**

A transit network with N routes and good transfers gives access to N^2 origin-destination pairs (each route can connect to every other route). Without transfers, each route is isolated, giving only N lines of coverage.

This is why transfer-friendly design (hubs, timed transfers, fare integration) matters enormously. In Megacity, consider:
- Free transfers within 60-90 minutes of first boarding
- Hub stations with short platform-to-platform distances
- Timed transfers at key junctions (expensive but effective)

### 4.7 Transit Financing

**Revenue sources:**

```
Fare revenue = ridership * average_fare
Fare recovery ratio = fare_revenue / operating_cost
```

Typical fare recovery ratios:
- US average: 30-40% (heavily subsidized)
- Hong Kong MTR: 170%+ (rail + property development)
- Tokyo private railways: 100-130% (land value capture)
- European average: 40-60%

**Operating cost model for Megacity:**

```rust
fn transit_operating_cost(route: &BusRoute, game_hour: f32) -> f32 {
    let vehicles_needed = route.cycle_time() / route.headway_seconds * 3600.0;
    let cost_per_vehicle_hour = match route.vehicle_type {
        BusType::Standard => 120.0,      // $120/vehicle-hour
        BusType::Articulated => 150.0,   // $150/vehicle-hour
        BusType::BiArticulated => 180.0, // $180/vehicle-hour
        BusType::Electric => 100.0,      // $100/vehicle-hour (lower fuel)
    };

    vehicles_needed * cost_per_vehicle_hour * game_hour
}

fn transit_revenue(route: &BusRoute, game_hour: f32) -> f32 {
    route.ridership_per_hour() * route.fare
}
```

The gap between revenue and cost must be covered by the city budget (taxes). This creates a gameplay tension: better transit loses more money but increases land values and reduces road congestion.

---

## 5. Freight and Goods Movement

### 5.1 Truck Traffic Percentages

Freight traffic is the invisible monster of city traffic. Players rarely think about it, but trucks account for a significant share of road usage and a disproportionate share of congestion and road damage.

**Truck percentage of total traffic by road type:**

| Road Type | Truck % (by volume) | Truck % (by PCE) | PCE factor |
|-----------|--------------------|--------------------|------------|
| Local residential | 2-5% | 4-10% | 2.0 |
| Local commercial | 8-15% | 15-28% | 2.0 |
| Collector | 5-10% | 10-18% | 1.8 |
| Arterial | 5-12% | 9-22% | 1.8 |
| Urban highway | 8-15% | 14-26% | 1.7 |
| Rural highway | 15-25% | 25-40% | 1.7 |
| Interstate (US avg) | 10-20% | 18-34% | 1.7 |
| Industrial district access | 20-40% | 35-60% | 1.8 |

**Passenger Car Equivalents (PCE):**

Trucks consume more road capacity than cars due to their size, slower acceleration, and longer following distances.

| Vehicle Type | PCE (level terrain) | PCE (upgrade, 4% grade) | Length |
|-------------|---------------------|--------------------------|--------|
| Passenger car | 1.0 | 1.0 | 5m |
| Light truck/van | 1.2 | 1.5 | 6m |
| Single-unit truck (2 axle) | 1.5 | 3.0 | 8m |
| Semi-trailer (5 axle) | 2.0 | 5.0 | 20m |
| Double trailer | 2.5 | 6.0 | 30m |
| Bus | 1.5 | 3.0 | 12m |

**For Megacity:** When computing link volumes and V/C ratios, convert truck traffic to PCE. If 10% of vehicles on a highway are trucks with PCE 2.0, the effective volume is:

```
V_effective = V_cars + V_trucks * PCE_truck
            = 0.9 * V_total + 0.1 * V_total * 2.0
            = V_total * (0.9 + 0.2)
            = 1.1 * V_total
```

This 10% increase in effective volume can push a road from LOS D to LOS E.

### 5.2 Freight Trip Generation

**Trip generation rates by land use:**

| Land Use | Truck trips/day per 1000 sqm | Truck trips/day per employee |
|----------|-----------------------------|-----------------------------|
| Light industrial | 4-8 | 0.3-0.6 |
| Heavy industrial | 8-15 | 0.5-1.0 |
| Warehouse/distribution | 15-30 | 1.0-3.0 |
| Retail (general) | 3-6 | 0.2-0.4 |
| Retail (supermarket) | 6-12 | 0.5-0.8 |
| Office | 0.5-2 | 0.05-0.15 |
| Residential (per dwelling) | 0.1-0.3 | - |

**Truck trip distribution patterns:**

Unlike commute trips (home-to-work at peak hours), freight trips have different temporal patterns:

```
Early morning (5-7 AM):   Delivery vehicles dispatched, 1.5x average
Morning (7-10 AM):        Deliveries to retail/commercial, 1.3x average
Midday (10 AM - 2 PM):    Steady deliveries, 1.0x average
Afternoon (2-5 PM):       Some peak, 1.1x average
Evening (5-8 PM):         Trailing off, 0.6x average
Night (8 PM - 5 AM):      Long-haul through traffic only, 0.4x average
```

**Truck route restrictions:**

Many cities restrict trucks from residential areas and narrow streets. In Megacity:

```rust
impl RoadType {
    pub fn allows_heavy_trucks(self) -> bool {
        match self {
            RoadType::Local => false,      // residential streets, no trucks
            RoadType::Avenue => true,      // collector, trucks allowed
            RoadType::Boulevard => true,   // arterial, trucks allowed
            RoadType::Highway => true,     // designed for trucks
            RoadType::OneWay => true,      // depends on context, allow by default
            RoadType::Path => false,       // pedestrian
        }
    }
}
```

When trucks cannot use locals, they must route via collector/arterial network, adding distance but protecting neighborhoods. This creates gameplay: players must ensure industrial zones have direct access to the arterial network.

### 5.3 Ports and Rail Terminals (Simplified)

**Port/Cargo Terminal:**

A port or cargo terminal is an outside connection that generates freight trips. Simplified model:

```rust
struct CargoTerminal {
    position: (usize, usize),
    terminal_type: TerminalType,
    capacity_tons_per_day: f32,
    current_throughput: f32,
    truck_trips_per_day: f32,   // derived from throughput
}

enum TerminalType {
    SmallPort,       // 1000 tons/day, generates 50 truck trips/day
    LargePort,       // 10000 tons/day, generates 500 truck trips/day
    RailYard,        // 5000 tons/day, generates 250 truck trips/day
    AirCargo,        // 500 tons/day, generates 100 truck trips/day (high value)
    TruckTerminal,   // 2000 tons/day, generates 200 truck trips/day
}
```

**Conversion: tons to truck trips:**

```
truck_trips = tonnage / avg_payload
```

Average payloads:
- Semi-trailer: 20-25 tons (weight-limited loads)
- Semi-trailer: 80-100 cubic meters (volume-limited loads, lighter goods)
- Average effective payload: ~15-20 tons (because trucks are not always full)

So: 1000 tons/day / 20 tons/truck = 50 truck trips/day (loaded). Double for empties returning: ~100 truck movements/day.

**Rail-truck intermodal:**

Rail removes truck traffic from the road network. Each train replaces approximately 200-400 truck trips (a typical intermodal train carries 100-200 containers, each replacing 1-2 truck loads).

For Megacity gameplay:
- Building a rail connection to industrial zones removes ~50% of truck trips from those zones from the road network
- The rail connection has a fixed cost (track, terminal) and per-ton operating cost
- Breakeven: when road congestion costs (delays to all vehicles) exceed rail operating costs

### 5.4 Last-Mile Freight

In urban areas, the "last mile" of delivery is the most expensive and disruptive part of the freight chain.

**Delivery vehicle impacts:**

| Issue | Impact | Mitigation |
|-------|--------|------------|
| Double-parking | Blocks 1 lane, reduces capacity 30-50% | Loading zones, off-peak delivery |
| Curb time | 5-15 min per stop | Consolidation centers |
| Large vehicle turning | Blocks intersections | Restrict large trucks downtown |
| Residential delivery (e-commerce) | 0.5-1.5 truck trips/household/week growing | Lockers, consolidation |

**For Megacity:** Commercial zones in dense areas should have a "delivery congestion" factor. Without loading zones or off-peak delivery policies, commercial streets get a 15-25% capacity reduction during business hours from delivery vehicles blocking lanes.

```rust
fn delivery_congestion_factor(zone: &Zone, time_of_day: f32) -> f32 {
    if !zone.zone_type.is_commercial() { return 1.0; }

    let is_business_hours = time_of_day >= 8.0 && time_of_day <= 18.0;
    if !is_business_hours { return 1.0; }

    let base_reduction = match zone.zone_type {
        ZoneType::CommercialLow => 0.10,   // small shops, few deliveries
        ZoneType::CommercialHigh => 0.20,  // dense retail, many deliveries
        _ => 0.0,
    };

    let policy_mitigation = if zone.has_loading_zones { 0.5 } else { 1.0 }
                          * if zone.has_off_peak_delivery { 0.3 } else { 1.0 };

    1.0 - base_reduction * policy_mitigation
}
```

### 5.5 Freight and Industrial Zone Placement

**The fundamental freight access rule:**

Industrial zones should be:
1. Adjacent to arterials or highways (not on local streets)
2. Near outside connections (ports, rail, highway interchanges)
3. Separated from residential by buffer zones (noise, pollution)
4. Have direct truck routes that avoid residential streets

**Trip distribution for freight:**

Unlike commuter trips that go home-to-work, freight trips follow supply chain patterns:

```
Industrial → Warehouse:     30% of freight trips (raw materials to factory)
Industrial → Commercial:    25% (finished goods to retail)
Warehouse → Commercial:     20% (distribution to retail)
Port/Rail → Industrial:     10% (imports to factory)
Port/Rail → Warehouse:      10% (imports to distribution)
Other (construction, etc):   5%
```

These percentages define the OD matrix for freight. If a city has poor connections between its port and industrial zones, truck traffic must route through residential areas -- a major quality of life problem.

---

## 6. Parking Simulation

Parking is where transportation meets land use. In most cities, parking policy is the most powerful lever for shaping transportation behavior, yet city builder games almost universally ignore it.

### 6.1 Parking Requirements by Land Use

Conventional zoning codes mandate minimum parking requirements. These numbers dictate how much land is consumed by parking:

| Land Use | Typical Requirement (US suburban) | Actual Peak Utilization | Ratio |
|----------|----------------------------------|------------------------|-------|
| Single-family home | 2.0 spaces/unit | 1.4 spaces/unit | 70% |
| Apartment | 1.5 spaces/unit | 0.8-1.2 spaces/unit | 53-80% |
| Office | 3.0 spaces/1000 sqft | 2.0-2.5 spaces/1000 sqft | 67-83% |
| Retail (general) | 4.0 spaces/1000 sqft | 2.5-3.0 spaces/1000 sqft | 63-75% |
| Retail (shopping center) | 5.0 spaces/1000 sqft | 3.5 peak (holiday) | 70% |
| Restaurant | 10.0 spaces/1000 sqft | 7-8 spaces/1000 sqft | 70-80% |
| Hospital | 3.5 spaces/bed | 2.5-3.0 spaces/bed | 71-86% |
| Industrial | 1.0 spaces/1000 sqft | 0.5-0.8 spaces/1000 sqft | 50-80% |

**Key insight:** Requirements are almost always higher than actual demand. This overbuilding of parking has enormous consequences:

1. **Land waste:** A parking space occupies 30-35 sqm (including access lanes). A 100-space surface lot consumes 3,000-3,500 sqm -- the footprint of a 3-story apartment building housing 50+ families.

2. **Cost:** Surface parking costs $5,000-15,000 per space to build. Structured parking costs $25,000-50,000. Underground costs $35,000-75,000.

3. **Induced driving:** Free abundant parking subsidizes driving, increasing auto mode share and traffic.

**For Megacity:** Buildings should have a parking requirement that scales with zone type and density. High-density zones near transit should have reduced requirements (transit-oriented development bonus).

```rust
fn parking_requirement(zone: &Zone, near_transit: bool) -> f32 {
    let base = match zone.zone_type {
        ZoneType::ResidentialLow => 2.0,    // spaces per dwelling unit
        ZoneType::ResidentialHigh => 1.0,   // less car-dependent
        ZoneType::CommercialLow => 3.0,     // spaces per 100 sqm
        ZoneType::CommercialHigh => 2.0,    // denser, less parking
        ZoneType::Industrial => 1.0,
        ZoneType::Office => 2.5,
        ZoneType::None => 0.0,
    };

    let transit_reduction = if near_transit { 0.5 } else { 1.0 };
    let density_factor = match zone.level {
        1 => 1.0,
        2 => 0.9,
        3 => 0.8,
        4 => 0.6,
        5 => 0.4,  // highest density needs least parking
        _ => 1.0,
    };

    base * transit_reduction * density_factor
}
```

### 6.2 Cruising for Parking: The 30% Problem

When parking supply is tight (especially downtown), drivers circulate looking for available spaces. This "cruising" behavior adds significant traffic that serves no productive purpose.

**The classic finding (Shoup, 2006):**

In downtown areas, 30% of traffic can be vehicles cruising for parking. This number has been replicated in studies across many cities:

| City | Area | % Cruising | Avg search time |
|------|------|------------|-----------------|
| New York (Midtown) | CBD | 28-45% | 7.8 min |
| San Francisco (downtown) | CBD | 25-35% | 6.5 min |
| London (West End) | CBD | 20-30% | 5-8 min |
| Freiburg (Altstadt) | CBD | 30-40% | 6 min |
| Average across studies | CBD | 30% | 8 min |

**The cruising-congestion feedback loop:**

```
Underpriced parking → High demand → Few available spaces
    → Longer cruising time → More vehicles on road
    → More congestion → Slower speeds for everyone
    → Some drivers still find spots → Others keep circling
```

This is a tragedy of the commons. Each driver searching for a free/cheap spot imposes delay on all other drivers.

**Modeling cruising in Megacity:**

```rust
fn cruising_traffic(
    parking_demand: f32,      // vehicles wanting to park
    parking_supply: f32,      // available spaces
    avg_search_time: f32,     // minutes
) -> f32 {
    let occupancy = (parking_demand / parking_supply).min(1.0);

    // Below 85% occupancy, finding a space is easy
    // Above 85%, search time increases dramatically
    let effective_search_time = if occupancy < 0.85 {
        0.5  // quick, minimal cruising
    } else {
        // Exponential increase in search time above 85%
        0.5 + avg_search_time * ((occupancy - 0.85) / 0.15).powf(2.0)
    };

    // Cruising vehicles on the road = searchers * time / turnover
    let cruising_vehicles = parking_demand * effective_search_time / 60.0;
    cruising_vehicles  // add this to road network volumes near the destination
}
```

The 85% occupancy threshold is well-established in parking research. Below 85%, drivers can find a space quickly. Above 85%, the search becomes long and frustrating. Above 95%, it becomes nearly impossible and drivers may give up or double-park.

### 6.3 Parking Pricing and Its Effects

Parking pricing is the most effective single tool for managing congestion. The effects are large and well-documented:

**Demand elasticity of parking pricing:**

| Context | Price elasticity | Interpretation |
|---------|-----------------|----------------|
| Commuter (work) parking | -0.1 to -0.3 | 10% price increase → 1-3% fewer parkers |
| Shopping (short-term) | -0.3 to -0.6 | More price-sensitive |
| CBD aggregate | -0.2 to -0.4 | Moderate overall sensitivity |

**Impact of pricing changes:**

| Policy | Traffic reduction | Mode shift to transit | Revenue |
|--------|-------------------|-----------------------|---------|
| Free → $5/day | 10-15% reduction | 5-8% increase | Significant |
| Free → $15/day | 20-30% reduction | 10-15% increase | Very high |
| $5/day → $15/day | 10-20% reduction | 5-10% increase | Higher |
| Meter ($2/hr) vs free | 15-25% reduction in cruising | 5-10% | Moderate |

**San Francisco's SFpark experiment:**

Variable pricing by demand (higher prices in busy areas/times, lower in underutilized areas). Results:
- Cruising reduced by 50%
- Average parking occupancy stabilized at 60-80% (below the 85% threshold)
- Traffic speeds increased 5-10%
- Transit ridership increased slightly

**For Megacity gameplay:**

Parking pricing as a policy tool:

```rust
enum ParkingPolicy {
    FreeParking,           // default, causes cruising in dense areas
    MeteredStreet {
        rate_per_hour: f32, // $1-5/hr
    },
    ResidentialPermit {
        annual_fee: f32,    // $50-200/year
    },
    DemandPricing {
        target_occupancy: f32, // 0.85 optimal
        min_rate: f32,
        max_rate: f32,
    },
    ParkingMaximum {
        max_spaces_per_unit: f32, // caps parking supply
    },
}

fn parking_revenue(
    policy: &ParkingPolicy,
    metered_spaces: u32,
    avg_occupancy: f32,
    avg_duration_hours: f32,
) -> f32 {
    match policy {
        ParkingPolicy::MeteredStreet { rate_per_hour } => {
            metered_spaces as f32 * avg_occupancy * rate_per_hour
                * avg_duration_hours * 10.0  // 10 hours of metered operation
        }
        ParkingPolicy::DemandPricing { .. } => {
            // Higher revenue from dynamic pricing
            metered_spaces as f32 * avg_occupancy * 3.0  // avg $3/hr
                * avg_duration_hours * 12.0
        }
        _ => 0.0,
    }
}
```

### 6.4 Parking Structures and Land Use

**Space comparison:**

| Parking Type | Sqm per space | Spaces per hectare | Cost per space | Annual maintenance |
|-------------|---------------|--------------------|-----------------|--------------------|
| Surface lot | 30-35 | 280-330 | $5,000-15,000 | $200-400 |
| Above-ground structure | 25-30 | 330-400 per level | $25,000-50,000 | $400-800 |
| Underground | 30-35 | 280-330 per level | $35,000-75,000 | $600-1200 |
| On-street (parallel) | 15-18 | N/A (linear) | $1,000-3,000 | $100-200 |
| On-street (angled) | 12-15 | N/A (linear) | $1,000-3,000 | $100-200 |

**The parking-density paradox:**

In low-density areas, surface parking is cheap and plentiful. No parking problem exists.

In medium-density areas, surface parking consumes so much land that density cannot increase. This creates a "density trap" -- the area needs more density to support transit, but parking requirements prevent densification.

In high-density areas, structured/underground parking is the only option, but its high cost naturally limits parking supply, which promotes transit use. This is a virtuous cycle.

**For Megacity:**

Auto-calculate parking type by zone density level:

```rust
fn parking_type(zone_level: u8) -> ParkingType {
    match zone_level {
        1 => ParkingType::Surface,        // low density, cheap
        2 => ParkingType::Surface,        // still feasible
        3 => ParkingType::Structure,      // transition point
        4 => ParkingType::Underground,    // necessary for density
        5 => ParkingType::Underground,    // dense core
        _ => ParkingType::Surface,
    }
}
```

### 6.5 Park-and-Ride

Park-and-ride facilities combine parking with transit access. They are most effective at the urban-suburban boundary where:
- Land is cheap enough for surface lots
- Transit service is frequent enough to attract riders
- The remaining trip (downtown) would be congested/expensive by car

**Sizing rule of thumb:**
- 250-500 spaces for a bus rapid transit station
- 500-2000 spaces for a commuter rail station
- 1000-5000 spaces for a major rail hub

**Park-and-ride capture area:**

Drivers will divert up to 5-10 minutes from their route to reach a P&R. The effective catchment is roughly a 5-8 km radius.

**Impact on traffic:**

Each park-and-ride user removes one car from the congested portion of the network (typically the last 10-20 km into the CBD). If a 500-space P&R is 80% utilized, that is 400 fewer cars in the downtown core during peak hours -- equivalent to the capacity of nearly one highway lane.

**For Megacity:** Park-and-ride should be placeable at transit stations. The game should model the mode choice shift: citizens who live in the catchment area compare the cost of driving all the way (parking + gas + congestion time) vs. driving to P&R + transit fare.

---

## 7. What Games Get Wrong

### 7.1 Cities: Skylines Lane Mathematics

Cities: Skylines (CS1) has one of the most widely discussed traffic simulation failures in gaming history. Understanding its problems is instructive for designing a better system.

**The fundamental problem: agent-based lane selection**

In CS1, each vehicle agent independently chooses a lane and path. The pathfinding algorithm selects a path at spawn time and assigns lane choices at that point. Vehicles commit to their exit lane far in advance, leading to the infamous "everyone uses one lane of a six-lane road" problem.

**Why this happens mechanically:**

1. **Pre-committed lane assignment:** When a vehicle spawns, it calculates its entire route including which lane to use at each intersection. If the next turn is a right turn in 2 km, the vehicle may enter the rightmost lane immediately, even if the road is empty.

2. **No lane-changing logic:** In vanilla CS1, vehicles rarely change lanes. They pick a lane at the start of a road segment and stay there. If 80% of vehicles at an intersection need to turn right, 80% will be in the rightmost lane, even if there are 5 other empty lanes.

3. **No capacity modeling:** CS1 does not have a concept of road capacity. A road is either flowing or gridlocked. There is no intermediate state. The transition from flow to gridlock is binary, driven by agent collision detection rather than flow theory.

4. **Path recalculation rarity:** Vehicles do not reroute when they encounter congestion. They sit in the queue. In real life, drivers learn to avoid congested routes over days and weeks.

**The traffic flow breakdown:**

In reality, a 6-lane road (3 per direction) has 3x the capacity of a 2-lane road. In CS1, if all vehicles need to turn right at the next intersection, only 1 lane is used regardless of road width. The effective capacity of a 6-lane road equals a 2-lane road in this scenario.

This means the game's fundamental recommendation -- "upgrade to a bigger road" -- often does not help, because the bottleneck is lane utilization, not lane count.

**Numerical example of the problem:**

Real 6-lane road (3/direction), all vehicles go straight:
- Capacity: 3 lanes * 1800 veh/hr = 5400 veh/hr
- At 4000 veh/hr: V/C = 0.74, LOS C, flowing smoothly

CS1 6-lane road, all vehicles turning right at next intersection:
- Effective capacity: 1 lane * (whatever CS1's per-lane throughput is, ~800-1000 agents/hr)
- At 4000 agents/hr: gridlock
- Other 2 lanes: completely empty

### 7.2 TM:PE (Traffic Manager: President Edition) -- The Community Fix

TM:PE is the most popular CS1 mod (millions of subscribers). It addresses many of CS1's problems:

**What TM:PE adds:**

1. **Lane connection editing:** Players can manually assign which lanes connect to which at each intersection. This allows, for example, marking all 3 lanes of an approach as "through" so vehicles spread across them.

2. **Speed limits:** Per-road customizable speed limits, allowing players to create speed differentials that the pathfinder can use.

3. **Priority signs:** Stop, yield, and priority road designations that affect intersection behavior.

4. **Timed traffic lights:** Manual signal timing with phase control, allowing players to create protected left turns, lagging phases, etc.

5. **Vehicle restrictions:** Ban trucks from residential streets, buses-only lanes.

6. **Lane arrow customization:** Override default lane-to-lane connections at intersections.

7. **Junction restrictions:** Control whether vehicles can enter blocked junctions, do U-turns, change lanes near intersections.

8. **Dynamic lane selection (DLS):** The key algorithmic improvement -- vehicles consider lane utilization when choosing lanes, spreading traffic more evenly.

**Why TM:PE shows the design space:**

TM:PE proves that players care deeply about traffic simulation quality. It has millions of users who spend hours manually tuning intersections. This suggests two things:
1. Providing these tools is valuable (power users want control)
2. The base game should not require manual tuning to work correctly (the defaults should be smart)

Megacity should aim for TM:PE-quality defaults with TM:PE-style tools available for power users.

### 7.3 The Agent vs. Aggregate Tension

This is the central design tension in traffic simulation for games.

**Agent-based (microscopic):**

Every vehicle is an individual entity with position, speed, destination, and behavior.

Pros:
- Visually satisfying (players see cars moving)
- Emergent behavior (traffic jams form naturally)
- Individual vehicle tracking (follow a specific citizen's commute)

Cons:
- O(N) cost where N = vehicles (100K citizens means 30-50K simultaneous vehicles at peak)
- Lane-changing, car-following, gap acceptance all need modeling
- Small errors compound: one stuck agent can gridlock the whole network
- Synchronization issues in parallel/ECS architectures

**Aggregate/statistical (macroscopic):**

Traffic is modeled as flows on links. No individual vehicles.

Pros:
- O(E) cost where E = edges in road graph (typically 3-10K for a large city)
- Based on proven traffic flow theory (BPR, fundamental diagram)
- Stable: no agent behavior bugs, no stuck vehicles
- Trivially parallelizable

Cons:
- Nothing to render (no visible cars)
- No individual vehicle tracking
- Misses microscopic phenomena (lane-changing failures, intersection blocking)
- Less "fun" -- players want to see traffic

**The hybrid solution (recommended for Megacity):**

```
                    LOD Boundary
                    |
Aggregate model ←---+---→ Agent model
(entire city)       |     (near camera)
                    |
Background thread   |   Main thread (rendering)
BPR + incremental   |   NaSch or car-following
assignment          |   for visible vehicles
                    |
Updates every       |   Updates every frame
1-5 game-minutes    |   (60 Hz)
```

1. **Background aggregate model** runs on a separate thread. It computes link volumes and travel times using BPR + incremental assignment (Section 1.6). This gives the "ground truth" congestion state for every link.

2. **Foreground agent model** spawns visual vehicles near the camera. The number and behavior of these vehicles is calibrated to match the aggregate model's volumes. If the aggregate model says a link has 800 veh/hr (LOS D), the visual model spawns enough agents to look like LOS D traffic.

3. **LOD transition:** When the camera moves, agents far from the camera are despawned and their trips are absorbed into the aggregate model. Agents are spawned near the camera by sampling from the aggregate model's flow distributions.

This is what Megacity's existing LOD system (Full/Simplified/Abstract tiers) is already designed for. The aggregate model maps to the Abstract tier, while the agent model maps to Full/Simplified.

```rust
/// Determines whether to use agent or aggregate model for a road segment
fn traffic_model_tier(
    segment_pos: Vec2,
    camera_pos: Vec2,
    camera_zoom: f32,
) -> TrafficModelTier {
    let dist = (segment_pos - camera_pos).length();
    let threshold = camera_zoom * 200.0;  // scale with zoom

    if dist < threshold {
        TrafficModelTier::Agent      // individual vehicles
    } else if dist < threshold * 3.0 {
        TrafficModelTier::Simplified // fewer agents, simpler behavior
    } else {
        TrafficModelTier::Aggregate  // statistical only
    }
}
```

### 7.4 Transport Fever 2 Comparison

Transport Fever 2 (TF2) takes a different approach from Cities: Skylines and offers useful lessons.

**TF2's model:**

1. **Full agent simulation:** Every citizen is an agent with a home, workplace, and daily schedule. They choose routes and modes based on travel time.

2. **Multi-modal routing:** Citizens can walk, drive, or use any combination of transit lines. The routing considers walking to stops, waiting, transferring, and walking from the final stop.

3. **Line-based transit:** Transit is defined by lines with stops and vehicle assignments. The game tracks per-line ridership, revenue, and utilization.

4. **Demand-responsive routing:** Citizens reroute when roads are congested or transit options improve. This creates a feedback loop where building a new transit line gradually attracts riders.

**What TF2 gets right:**

- Mode choice is meaningful (citizens actually prefer faster/cheaper options)
- Transit planning is the core gameplay (not just traffic management)
- Line profitability creates economic feedback
- The game scales to thousands of vehicles with LOD

**What TF2 gets wrong:**

- **No capacity modeling on roads:** Like CS1, roads are either flowing or not. There is no concept of LOS or degradation curves.
- **Oversimplified intersection logic:** Vehicles clip through each other at intersections. No signalization or conflict modeling.
- **Binary congestion:** Traffic appears or disappears abruptly rather than gradually degrading.
- **No parking modeling:** All destinations have infinite free parking.
- **Freight is overly complex:** The supply chain system, while interesting, creates brittle networks that are frustrating to optimize.

**Lessons for Megacity:**

From CS1: Do not use pre-committed lane selection. Implement BPR-based capacity degradation on links. Provide TM:PE-style tools as built-in features.

From TF2: Build mode choice into the demand model. Make transit a core gameplay system with financial feedback. Track per-line performance metrics.

From both: Implement the hybrid agent/aggregate model (Section 7.3) to get the best of both worlds.

### 7.5 Common Simplifications That Break Realism

**1. Ignoring intersection capacity:**

Most games model roads as pipes -- wider pipes flow more traffic. But the real bottleneck is usually the intersection at the end of the pipe. A 6-lane road feeding into an unsignalized intersection has effectively the same capacity as a 2-lane road at the intersection.

Fix: Model intersection capacity as a separate constraint. The flow on a link is limited by both the link capacity AND the intersection capacity at each end.

**2. Symmetric demand:**

Games often assume morning peak = evening peak in reverse. In reality:
- Morning peak is sharper and shorter (7:30-8:30 AM, everyone arrives at the same time)
- Evening peak is broader and flatter (4:00-7:00 PM, people leave at different times)
- School trips add a secondary morning peak at 8:00-8:30 AM
- Weekend traffic is totally different (no commute, more dispersed commercial)

Fix: Use time-of-day demand profiles (Section 1.6, Approach 4) with asymmetric AM/PM peaks.

**3. No trip chaining:**

Games assume home -> work -> home. Real trips are:
- Home -> daycare -> work (morning)
- Work -> lunch -> work (midday)
- Work -> grocery -> gym -> home (evening)

Trip chains increase the number of trips, change their spatial pattern, and favor car travel (hard to trip-chain on transit).

Fix: Allow 20-30% of trips to be chained (2-3 stops per tour instead of 1).

**4. No induced demand:**

Building more road capacity should attract more traffic (latent demand). If a new highway reduces travel time, people who previously avoided that corridor (or took transit, or lived closer to work) now drive on it. Within 5-10 years, the highway fills up again.

Fix: After road capacity increases, gradually increase demand on that corridor. A simple model:

```
demand_growth_rate = elasticity * (old_travel_time - new_travel_time) / old_travel_time
```

Where elasticity = 0.5-0.8 for short-run (1-3 years) and 0.8-1.2 for long-run (5-10 years).

**5. Free parking everywhere:**

As Section 6 discusses, parking is never actually "free" -- it is just paid for by someone else (the property owner, all customers, all taxpayers). By making parking invisible and free in games, they eliminate one of the most powerful levers for transportation policy.

Fix: Model parking supply, pricing, and search behavior. Even a simplified version (Section 6.2) dramatically improves realism.

**6. No network effects of transit:**

Games model each transit line independently. But the value of transit is in the network -- a new bus line is more valuable if it connects to existing metro stations. The mode choice model (Section 4.1) with transfer penalties captures this.

**7. Static route choice:**

Vehicles choose their route once at trip start and never change. In reality, drivers learn over time and adapt. Day-to-day dynamics matter.

Fix: Use the incremental assignment approach (Section 1.6) which implicitly models adaptive routing. Or periodically recalculate routes for a fraction of agents.

---

## 8. Implementation Recommendations for Megacity

### 8.1 Phased Implementation Plan

Given Megacity's existing architecture (CSR graph, Bezier segments, LOD citizen system), here is a recommended implementation order:

**Phase 1: BPR-Based Link Costs (Minimal viable traffic)**

Effort: ~1 week

Changes:
- Add `volume: f32` and `capacity: f32` fields to each segment/edge in the CSR graph
- Compute capacity from `RoadType::directional_capacity()` (Section 3.2)
- Apply BPR function to compute edge weights: `weight = free_flow_time * (1 + alpha * (V/C)^beta)`
- Update `csr_find_path_with_traffic()` to use BPR weights instead of constant weights
- Accumulate volumes by counting pathfinding results on each edge

Impact: Traffic will start to self-distribute. Congested roads become slower, alternative routes become attractive. This single change eliminates 80% of the "everyone uses one road" problem.

```rust
// Modify CsrGraph to support dynamic weights
impl CsrGraph {
    pub fn update_weights_bpr(&mut self, volumes: &[f32], capacities: &[f32]) {
        for (i, weight) in self.weights.iter_mut().enumerate() {
            let vc = volumes[i] / capacities[i].max(1.0);
            let alpha = 0.50;  // tune per road type later
            let beta = 4.0;
            let bpr_factor = 1.0 + alpha * vc.powf(beta);
            *weight = (bpr_factor * 100.0) as u32;  // scale to integer weights
        }
    }
}
```

**Phase 2: Traffic Overlay Visualization**

Effort: ~3 days

Changes:
- Compute V/C ratio for each segment
- Map to LOS A-F (Section 1.3)
- Color road segments on the traffic overlay using LOS colors
- Display V/C ratio and LOS grade in road info panel

This gives players feedback on their road network quality without changing any simulation math.

**Phase 3: Intersection Capacity**

Effort: ~1 week

Changes:
- Detect intersection type (3-way, 4-way, roundabout if player-placed)
- Compute intersection capacity (Section 2.3, simplified)
- Limit link flow to min(link_capacity, downstream_intersection_capacity)
- Auto-upgrade intersection type based on connecting road types:
  - Local-Local: unsignalized
  - Local-Avenue: unsignalized (stop on Local)
  - Avenue-Avenue: signalized (2-phase)
  - Avenue-Boulevard: signalized (3-phase)
  - Boulevard-Boulevard: signalized (3-phase)
  - Highway-anything: grade-separated (auto-place interchange)

**Phase 4: Statistical Traffic Assignment**

Effort: ~2 weeks

Changes:
- Divide city into zones (could reuse district system or auto-generate from building clusters)
- Compute trip generation per zone (from population and jobs)
- Build OD matrix using gravity model (Section 4.1, Step 2)
- Run incremental assignment (Section 1.6, Approach 3) on background thread
- Replace per-citizen pathfinding with zone-level assignment for Abstract LOD tier

Performance: Background thread runs assignment every 2-5 game-minutes. Main thread interpolates volumes between updates.

**Phase 5: Public Transit**

Effort: ~3-4 weeks

Changes:
- Add transit infrastructure: bus stops, bus routes, vehicle entities
- Multi-modal pathfinding: walking + transit + walking
- Mode choice for Abstract-tier citizens (Section 4.1)
- Transit financial model: fares, operating costs, ridership tracking
- UI for route creation, modification, performance monitoring

**Phase 6: Advanced Features (Parking, Freight, Signals)**

Effort: ~4-6 weeks

Each is relatively independent:
- Parking: supply tracking, cruising behavior, pricing policies
- Freight: truck trips from industrial/commercial, PCE factors, route restrictions
- Signal timing: Webster's formula, coordination, adaptive control

### 8.2 Data Structures

**Traffic volume storage:**

```rust
/// Per-edge traffic data, parallel arrays to CsrGraph.edges
#[derive(Resource, Default)]
pub struct TrafficVolumes {
    /// Current volume (vehicles per hour) on each edge
    pub volume: Vec<f32>,
    /// Capacity (vehicles per hour) of each edge
    pub capacity: Vec<f32>,
    /// Volume-to-capacity ratio (cached, updated with volumes)
    pub vc_ratio: Vec<f32>,
    /// Level of service (cached)
    pub los: Vec<LevelOfService>,
    /// BPR-computed travel time multiplier (cached)
    pub travel_time_multiplier: Vec<f32>,
}

impl TrafficVolumes {
    pub fn new(edge_count: usize) -> Self {
        Self {
            volume: vec![0.0; edge_count],
            capacity: vec![1800.0; edge_count],  // default, overwritten on init
            vc_ratio: vec![0.0; edge_count],
            los: vec![LevelOfService::A; edge_count],
            travel_time_multiplier: vec![1.0; edge_count],
        }
    }

    pub fn update(&mut self) {
        for i in 0..self.volume.len() {
            let vc = self.volume[i] / self.capacity[i].max(1.0);
            self.vc_ratio[i] = vc;
            self.los[i] = los_from_vc(vc);
            self.travel_time_multiplier[i] = bpr(vc, 0.50, 4.0);
        }
    }
}

fn bpr(vc: f32, alpha: f32, beta: f32) -> f32 {
    1.0 + alpha * vc.powf(beta)
}
```

**Zone-based demand:**

```rust
#[derive(Resource)]
pub struct TravelDemand {
    /// Zone definitions (zone_id -> set of grid cells)
    pub zones: Vec<Zone>,
    /// Trip productions per zone per hour
    pub productions: Vec<f32>,
    /// Trip attractions per zone per hour
    pub attractions: Vec<f32>,
    /// OD matrix: trips[i][j] = trips from zone i to zone j per hour
    pub od_matrix: Vec<Vec<f32>>,
    /// Temporal profile multiplier for current time of day
    pub temporal_factor: f32,
}
```

### 8.3 Performance Budget

Target: All traffic computation must complete within 2ms per game tick at 60 FPS (leaving 14.7ms for rendering and other systems).

| Component | Operations | Time estimate | Frequency |
|-----------|-----------|---------------|-----------|
| BPR weight update | 5000 edges * 3 ops | 15 us | Every tick |
| Agent pathfinding (visible) | 50 paths * 0.2 ms each | 10 ms | Amortized over ticks |
| Volume accumulation | 5000 edges * increment | 5 us | Every tick |
| LOD transition | 100 agents spawn/despawn | 100 us | Camera movement |
| Aggregate assignment | 500 OD pairs * 5000 edges * 5 iterations | 250 ms | Background, every 5 min |
| Intersection delay | 2000 intersections * 5 ops | 10 us | Every tick |
| Transit routing | 200 routes * 20 stops | 50 us | Every tick |

Total per-tick budget: ~0.2 ms on main thread. The expensive aggregate assignment runs on a background thread and never blocks the main loop.

### 8.4 Integration with Existing Megacity Systems

**CsrGraph integration (road_graph_csr.rs):**

The existing `CsrGraph::weights` field currently stores constant weights (all 1). Replace with BPR-computed weights. The `csr_find_path` function already uses weights via `neighbor_weights()` -- no changes needed to the pathfinding algorithm itself.

**RoadSegmentStore integration (road_segments.rs):**

Each `RoadSegment` has `road_type` and `arc_length`. These provide the free-flow speed and distance needed for BPR computation. Add a `volume` field to `RoadSegment` or maintain a parallel volume array indexed by `SegmentId`.

**Movement system integration (movement.rs):**

Citizens currently follow paths computed by `csr_find_path`. The paths themselves do not change -- but the weights used to compute them do. Citizens who have not yet started their trip should use current (congested) weights. Citizens mid-trip should not reroute (too expensive and unrealistic for most trips).

**Economy/budget integration (economy.rs, budget.rs):**

Add transit operating costs and parking revenue as budget line items. Traffic congestion can be monetized as a "productivity loss" metric (total vehicle-hours of delay * value of time) for player feedback.

**Overlay integration (overlay.rs):**

Add a TrafficOverlay mode that colors road segments by LOS. This is the primary player feedback mechanism for traffic conditions.

### 8.5 Tuning and Calibration

The most important tuning parameters and how to set them:

| Parameter | Starting value | Range to test | How to calibrate |
|-----------|---------------|---------------|------------------|
| BPR alpha (local) | 0.80 | 0.50-1.20 | Should show visible color change at V/C 0.5 |
| BPR alpha (highway) | 0.20 | 0.10-0.40 | Should show "cliff" at V/C > 1.0 |
| BPR beta (all) | 4.0 | 3.0-6.0 | Higher = sharper transition at capacity |
| Gravity model beta | 0.10 | 0.06-0.20 | Lower = longer trips, higher = more local |
| Trip rate per household | 7.5/day | 5.0-10.0 | Adjust until road network feels "right" |
| Peak multiplier | 1.8 | 1.5-2.5 | Higher = more dramatic rush hour |
| Transit mode share target | 15-30% | 5-50% | Varies by city type player builds |
| Truck % of traffic | 8% | 5-15% | Higher near industrial zones |
| Parking cruising factor | 30% | 15-40% | Only in high-density commercial |
| Intersection capacity multiplier | 1.0 | 0.7-1.3 | Tune if intersections feel too easy/hard |

### 8.6 What Not to Implement

Some features from real traffic engineering are not worth implementing in a game:

1. **Traffic simulation below 1-second timestep:** Not needed. BPR gives steady-state results. NaSch at 1-second resolution is overkill for anything not visible.

2. **Full HCM intersection analysis:** The Highway Capacity Manual procedure involves 50+ adjustment factors. Use the simplified models in Section 2.

3. **Dynamic traffic assignment (DTA):** Time-varying assignment is computationally expensive and provides minimal gameplay benefit over incremental static assignment.

4. **Microscopic car-following models (Wiedemann, IDM):** These are for research, not games. NaSch is sufficient for visual agents.

5. **Detailed signal optimization (TRANSYT, Synchro):** Auto-compute reasonable signals with Webster's formula. Do not expose full signal timing UI (TM:PE shows this is a niche interest).

6. **Exact mode choice calibration:** The logit model coefficients do not need to match any particular city. Tune for fun, not accuracy.

7. **Full freight supply chain modeling:** TF2 shows this can be frustrating for players. Keep freight as simple truck trip generation from industrial zones.

