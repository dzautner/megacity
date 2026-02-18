# Underground Infrastructure: Deep Design Document

## Table of Contents

1. [The Core Design Decision: Explicit Pipes vs Coverage Zones](#the-core-design-decision)
2. [Water Supply Network](#water-supply-network)
3. [Sewage/Wastewater Network](#sewagewastewater-network)
4. [Stormwater Drainage](#stormwater-drainage)
5. [Power Distribution](#power-distribution)
6. [Metro/Subway Systems](#metrosubway-systems)
7. [Underground View/Layer System](#underground-viewlayer-system)
8. [Utility Tunnels (Advanced)](#utility-tunnels-advanced)
9. [Implementation Architecture](#implementation-architecture)

---

## The Core Design Decision

### The Fundamental Tension

Underground infrastructure in city builders sits at the intersection of two opposing forces: **simulation depth** (players want to feel like they are engineering a real city) and **playability** (nobody wants to spend 20 minutes laying sewer pipes to connect 50 houses). Every city builder in history has taken a different position on this spectrum, and each approach has generated strong player reactions. The choice made here will define a significant portion of the game's identity.

### Approach 1: Cities: Skylines 1 -- Explicit Pipe Drawing

Cities: Skylines 1 required players to draw individual water pipe segments underground and place power lines above ground. Each building needed to be within a certain distance of a pipe to receive water service.

**How it worked mechanically:**
- Player enters underground view mode (press Page Down or click the toggle).
- Surface terrain becomes semi-transparent, buildings show as ghosted outlines.
- Player draws pipe segments along desired routes using the same road-drawing UI metaphor.
- Water flows from water pumping stations through connected pipes.
- Buildings within ~6 cells of a pipe automatically connect (service lateral implied).
- Sewage flows through the same pipe network to sewage outflow or treatment plants.
- Power uses above-ground power lines with the same connection logic, but roads also carry power along their length automatically.

**Why players liked it:**
- Satisfying sense of building something. The underground view felt like a second layer of the city.
- Strategic depth: you could optimize pipe routing, avoid redundant infrastructure, plan for expansion.
- Visible cause-and-effect: "this neighborhood has no water because I forgot to connect it" is immediately legible.
- Discovery moments: "Oh! I need to extend pipes before I can zone this new area" teaches infrastructure planning.
- Power lines across open terrain were visually interesting and created spatial constraints (NIMBYism, land use).

**Why players disliked it:**
- At scale (50k+ population), pipe management becomes tedious. You are essentially drawing a second road network underground.
- The UI for underground view was clunky. Camera controls felt different, reference to surface features was limited.
- Pipes never broke, never aged, never had capacity limits. The "simulation" was really just connectivity BFS.
- Unrealistic: real cities do not have a pipe from each building to a central main. Service laterals connect buildings to distribution mains, which connect to trunk mains. The CS1 model was topologically flat.
- Players developed a meta-strategy of just running pipes under every road, which eliminated all decision-making. If the optimal strategy is "always do the same thing," the mechanic adds tedium without strategy.

**Simulation depth score:** Medium. It felt deep but the actual simulation was trivially binary (connected/not connected). No pressure, no capacity, no aging.

### Approach 2: Cities: Skylines 2 -- Automatic Coverage

CS2 removed pipes entirely. Water and sewage became coverage-based: place a water pumping station and all buildings within a certain radius receive water automatically. Power plants similarly radiate coverage. No underground view, no pipe drawing, no power lines.

**How it worked mechanically:**
- Place a water facility (pumping station, water tower, etc.) on the map.
- All buildings within the facility's coverage radius automatically have water service.
- Coverage radius can be expanded by upgrading the facility.
- Same pattern for sewage: place treatment plant, coverage area receives sewage service.
- Power: place power plant, coverage radius provides electricity. Alternatively, roads carry power along their length.
- No underground view at all -- everything is surface-level UI overlays.

**Why Colossal Order chose this approach:**
- Reduces micromanagement at scale. A city of 200k does not need 10,000 pipe segments managed individually.
- Allows focusing development resources on other systems (traffic, economy, etc.).
- Simplifies the codebase -- no need for a separate underground rendering layer, underground pathfinding, pipe entity management, etc.
- Reduces save file sizes (no pipe entities to serialize).

**Why the community reacted negatively:**
- "Dumbing down" perception. Players felt a layer of gameplay was removed. City builders attract simulation enthusiasts who want more systems, not fewer.
- Loss of the "aha" moment when you realize a neighborhood has no water because of a missing pipe connection. Coverage zones are too forgiving.
- No failure states. In CS1, forgetting to connect pipes was a real mistake with real consequences. In CS2, you just place a water facility and everything works. There is no skill expression.
- Reduced sense of progression. In CS1, extending infrastructure to a new neighborhood felt like an accomplishment. In CS2, you just plop a building.
- The coverage-radius approach makes facility placement the only decision, and with large radii, even that decision is trivial.
- Community consensus: the pipe system was one of CS1's best features, and removing it was a mistake.

**Player sentiment data point:** Multiple community surveys after CS2 launch showed "bring back pipes" as a top-5 requested feature. Modders immediately began working on pipe mods.

### Approach 3: SimCity 2013 -- Roads Carry Everything

SimCity (2013) took the simplest possible approach: all utilities flow through roads. If a building is on a road, it has water, power, and sewage. Period.

**How it worked mechanically:**
- Place a power plant anywhere connected to the road network, and all buildings on connected roads have power.
- Water pumping stations work the same way -- connected to road = has water.
- Sewage is implicit -- all sewage flows through roads to treatment plants.
- No pipes, no power lines, no underground view.

**Why this works for SimCity's design:**
- SimCity 2013 was designed for casual/social play. Small map sizes (2km x 2km) meant infrastructure complexity was unnecessary.
- Fast city-building loop: zone, road, build, profit. No infrastructure friction.
- Multiplayer region focus meant individual cities were small and quick to build.

**Why this does not work for Megacity:**
- Completely unrealistic. Roads do not carry water pipes in real life (well, some utility corridors follow road rights-of-way, but the abstraction is too aggressive).
- Zero strategic depth for infrastructure. The only decision is "is this building on a road?" which is always yes.
- No failure states, no emergent gameplay, no interesting tradeoffs.
- Players who want a serious city builder actively mock this approach.
- SimCity 2013 is widely considered a failure, and its infrastructure simplification is cited as one reason.

### Approach 4: The Hybrid Model (Recommended for Megacity)

The hybrid approach takes the best elements from all three models and introduces a tiered infrastructure system that adds depth without tedium.

**Core philosophy: "Player draws trunk mains, buildings auto-connect to nearest main, local distribution is implied."**

#### How it works:

**Tier 1: Trunk Mains (Player-Drawn)**
- Player draws major water mains, sewer trunk lines, power transmission lines, and gas mains.
- These are drawn in an underground view using the same Bezier-curve UI as road segments.
- Trunk mains have capacity limits, pressure/flow simulation, and age-based failure probabilities.
- They follow a separate underground grid layer and can cross under roads, buildings, and open terrain.
- Visual feedback: thick colored lines showing flow direction, capacity usage, and pressure.

**Tier 2: Distribution Network (Automatic)**
- When a trunk main passes within a configurable distance of a road, a distribution branch auto-connects.
- The distribution network extends along roads automatically (like CS2's coverage, but only along roads connected to a trunk main).
- Buildings within ~3 cells of a road with distribution service auto-connect via implied service laterals.
- This means: player does not draw individual pipes to each building, but does need to ensure trunk mains reach each neighborhood.

**Tier 3: Service Laterals (Fully Automatic)**
- Individual building connections are always automatic and invisible.
- No player interaction required.
- Capacity is determined by the distribution network capacity, which is determined by trunk main capacity.

#### Why this works:

**Strategic depth without tedium:**
- Player makes meaningful decisions about where trunk mains go, what capacity to build, where to place treatment plants and pump stations.
- Player does NOT micromanage individual building connections.
- The "forgotten neighborhood" scenario still exists: if you do not extend a trunk main to a new area, buildings there have no water/sewer/power.
- But extending service to a new area is a single trunk main draw, not 200 individual pipe segments.

**Realistic failure modes:**
- Trunk mains can burst (age-based probability), causing localized service outages.
- Trunk mains have capacity limits -- if a neighborhood grows beyond what the main can serve, pressure drops, service degrades.
- Treatment plants can be overwhelmed, causing sewage overflow events.
- Power grid can suffer cascade failures if a transmission line goes down.

**Scalable complexity:**
- Early game (small city): player draws 2-3 trunk mains from water source to small road network. Simple and fast.
- Mid game (medium city): player has a network of 15-20 trunk mains with branching topology. Capacity planning becomes important.
- Late game (large city): player upgrades trunk mains, adds redundancy, builds utility tunnels, manages aging infrastructure. Deep but not tedious because the system operates at the trunk level, not the building level.

**Compatibility with existing Megacity architecture:**
- The existing `WorldGrid` uses `has_power` and `has_water` booleans per cell. The hybrid model replaces these with quantitative values (pressure, capacity utilization).
- The existing BFS propagation in `propagate_utilities()` maps directly to the distribution network auto-connection logic.
- The existing `UtilitySource` component maps to the treatment/generation facilities.
- The underground grid can be a new `Resource` (`UndergroundGrid`) that runs alongside `WorldGrid`.

#### Tradeoff analysis:

| Criterion | CS1 Explicit | CS2 Coverage | SimCity Roads | Hybrid |
|-----------|-------------|-------------|---------------|--------|
| Strategic depth | Medium | Low | None | High |
| Tedium at scale | High | None | None | Low |
| Realism | Low-Medium | Low | Very Low | Medium-High |
| Implementation effort | Medium | Low | Very Low | High |
| Player satisfaction | Medium-High | Low | Low | Expected High |
| Failure modes | Binary (connected/not) | None | None | Graduated |
| Scalability to 1M citizens | Poor (too many entities) | Good | Good | Good |
| Underground view required | Yes | No | No | Yes |

#### Recommendation:

**Implement the Hybrid Model.** The implementation cost is higher, but it is the right long-term investment. Players who choose a city builder want depth. The hybrid model provides depth at the strategic level (trunk infrastructure planning) while eliminating tedium at the tactical level (individual building connections). It also creates meaningful failure modes that drive interesting gameplay: aging pipes burst, undersized mains create pressure problems, power grid topology determines blackout patterns.

The existing Megacity codebase is well-suited to this approach. The `WorldGrid` + `RoadNetwork` + `RoadSegmentStore` architecture provides all the building blocks needed for underground infrastructure as a parallel system.

---

## Water Supply Network

### Overview: From Source to Tap

A municipal water supply system is, in engineering terms, a pressurized closed-loop distribution network. Water is collected from a source, treated to potable standards, pressurized, distributed through a hierarchical pipe network, and delivered to individual buildings. The entire system must maintain positive pressure at all points to prevent contamination (negative pressure can suck groundwater or sewage into pipes through cracks).

For Megacity, the water system is the most important underground utility to get right because:
1. It interacts with terrain (elevation determines pressure).
2. It interacts with weather (drought depletes sources, storms increase demand on treatment).
3. It interacts with health (contaminated water causes disease outbreaks).
4. It interacts with fire response (no water pressure = fire department cannot operate).
5. It creates the most visible player-facing consequences when it fails.

### Water Sources

Each water source type has distinct characteristics that create interesting gameplay tradeoffs:

#### River Intake

**Real-world mechanics:** A concrete structure built on a riverbank with screened intake pipes submerged in the river. Water is pumped from the river into a raw water main that leads to a treatment plant.

**Game implementation:**
```
WaterSourceType::RiverIntake {
    capacity_liters_per_day: 50_000_000,  // 50 ML/day - serves ~80,000 people
    raw_water_quality: 0.4,               // 0.0 (toxic) to 1.0 (pristine)
    elevation_requirement: "must be placed on a cell adjacent to a river/water cell",
    construction_cost: 15_000,
    monthly_operating_cost: 800,
    pollution_sensitivity: true,          // output quality degrades if WaterPollutionGrid > threshold
    drought_sensitivity: 0.3,             // capacity drops 30% during drought events
}
```

**Placement constraints:**
- Must be placed on a grass cell directly adjacent (neighbors4) to a water cell of type river (not ocean/lake -- the Tel Aviv coastline is saltwater).
- The adjacent water cell's `WaterPollutionGrid` level affects the raw water quality entering the treatment chain.
- During drought weather events, capacity reduces by the `drought_sensitivity` factor.

**Strategic considerations:**
- Cheapest high-capacity source.
- Vulnerable to upstream industrial pollution (the existing `WaterPollutionGrid` system directly feeds into this).
- Vulnerable to drought (the existing `Weather` system can trigger capacity reduction).
- Location-constrained: only works near rivers. In the Tel Aviv map, the Yarkon River (y~185) is the only significant freshwater source.

#### Lake Pump Station

**Real-world mechanics:** Similar to river intake but draws from a standing body of water. Less sensitive to drought (lakes have more storage) but more sensitive to algal blooms and thermal stratification.

**Game implementation:**
```
WaterSourceType::LakePump {
    capacity_liters_per_day: 30_000_000,  // 30 ML/day - serves ~50,000 people
    raw_water_quality: 0.5,               // standing water is generally cleaner than river
    elevation_requirement: "adjacent to lake water cell",
    construction_cost: 12_000,
    monthly_operating_cost: 600,
    pollution_sensitivity: true,
    drought_sensitivity: 0.15,            // lakes buffer drought better than rivers
    seasonal_quality_variation: true,     // summer algal blooms reduce quality
}
```

**Strategic considerations:**
- Better drought resilience than river intake.
- Seasonal quality variation in summer (algal blooms) can require additional treatment.
- Not available on the Tel Aviv map (no lakes), but relevant for future map generation.

#### Groundwater Well

**Real-world mechanics:** A bored well with a submersible pump that extracts water from underground aquifers. Water quality depends on local geology and nearby contamination sources.

**Game implementation:**
```
WaterSourceType::GroundwaterWell {
    capacity_liters_per_day: 5_000_000,   // 5 ML/day - serves ~8,000 people
    raw_water_quality: "dynamic, read from WaterQualityGrid at well location",
    elevation_requirement: "GroundwaterGrid level at location must be > 80",
    construction_cost: 5_000,
    monthly_operating_cost: 200,
    pollution_sensitivity: true,          // reads directly from WaterQualityGrid
    drought_sensitivity: 0.5,            // heavy dependency on groundwater table
    aquifer_depletion: true,             // continuous pumping lowers GroundwaterGrid locally
}
```

**Integration with existing systems:**
- The `GroundwaterGrid` (already implemented, see `crates/simulation/src/groundwater.rs`) provides the water table level. The well can only be placed where groundwater level > 80 (out of 255).
- The `WaterQualityGrid` determines raw water quality. A well placed near industrial zones will produce low-quality water requiring more treatment.
- Continuous pumping depletes the local groundwater table (the existing `update_groundwater()` system already has building-based drawdown logic). Well pumps accelerate this depletion.
- Over-pumping creates a "cone of depression" -- the groundwater level drops in a radius around the well, potentially affecting nearby wells and vegetation.

**Strategic considerations:**
- Low capacity, low cost -- good for early game or small neighborhoods.
- Quality depends entirely on local conditions.
- Can be placed anywhere with sufficient groundwater, not limited to water bodies.
- Over-reliance causes aquifer depletion, a long-term sustainability challenge.
- Multiple wells in close proximity interfere with each other (competing drawdown cones).

#### Desalination Plant

**Real-world mechanics:** Reverse osmosis membranes or thermal distillation that removes salt from seawater. Extremely energy-intensive (3-6 kWh per cubic meter). Produces brine waste that must be disposed of.

**Game implementation:**
```
WaterSourceType::Desalination {
    capacity_liters_per_day: 100_000_000, // 100 ML/day - serves ~160,000 people
    raw_water_quality: 0.95,              // near-pristine output (salt removed)
    elevation_requirement: "must be adjacent to ocean/sea water cell",
    construction_cost: 80_000,
    monthly_operating_cost: 5_000,
    power_consumption_multiplier: 3.0,    // requires 3x normal industrial power
    brine_output: true,                   // generates water pollution at outfall
    drought_sensitivity: 0.0,            // drought-proof -- ocean does not run dry
}
```

**Strategic considerations:**
- Drought-proof. This is the key advantage and a major late-game investment.
- Extremely expensive to build and operate. High power consumption makes it dependent on a robust power grid.
- Produces brine discharge that increases `WaterPollutionGrid` levels at the outfall location.
- Ideal for coastal cities like Tel Aviv. The western coastline provides unlimited placement opportunities.
- Environmental tradeoff: solve water scarcity but create marine pollution.

#### Reservoir/Dam

**Real-world mechanics:** An impoundment dam across a river valley creates a reservoir that stores water during wet periods for use during dry periods. Provides flood control as a secondary benefit.

**Game implementation:**
```
WaterSourceType::Reservoir {
    capacity_liters_per_day: 80_000_000,  // 80 ML/day base, but varies with stored volume
    storage_capacity_days: 180,           // can supply full capacity for 180 days without inflow
    raw_water_quality: 0.6,              // reservoir water is moderate quality
    elevation_requirement: "must span a river with elevation differential > 0.1",
    construction_cost: 120_000,
    monthly_operating_cost: 1_500,
    flood_control: true,                  // reduces flood disaster severity in downstream cells
    footprint: (6, 4),                   // large structure
    fill_rate: "proportional to upstream river flow and rainfall",
}
```

**Strategic considerations:**
- Highest single-source capacity.
- Provides drought buffer through stored water.
- Provides flood control (reduces `Flood` disaster damage downstream).
- Massive construction cost and land footprint. Requires specific terrain (river + elevation change).
- Long-term asset: the reservoir fills gradually over rainy seasons.
- On the Tel Aviv map, the Yarkon River at approximately (x=100-195, y=185) could support a small reservoir if the player builds a dam at a narrowing point.

### Water Treatment

Raw water from any source must be treated before distribution. Treatment quality directly affects public health.

#### Treatment Levels

**Level 1: Basic Filtration**
```
TreatmentLevel::Basic {
    output_quality: 0.5,        // minimum viable -- prevents cholera, not much else
    capacity_multiplier: 1.0,   // no throughput penalty
    construction_cost: 8_000,
    monthly_cost: 400,
    health_modifier: -5.0,      // slight health penalty vs standard treatment
    footprint: (2, 2),
}
```
- Screens, sedimentation, basic chlorination.
- Acceptable for early game but causes minor health issues long-term.
- Buildings served by basic-treated water have a health penalty applied to their residents via the existing `CitizenDetails.health` system.

**Level 2: Standard Treatment**
```
TreatmentLevel::Standard {
    output_quality: 0.8,        // good quality, meets regulatory standards
    capacity_multiplier: 0.85,  // some throughput loss from treatment processes
    construction_cost: 25_000,
    monthly_cost: 1_200,
    health_modifier: 0.0,       // baseline -- no penalty
    footprint: (3, 3),
}
```
- Coagulation, flocculation, sedimentation, filtration, disinfection.
- The expected standard for a well-run city.
- No health modifier -- this is the baseline.

**Level 3: Advanced Treatment**
```
TreatmentLevel::Advanced {
    output_quality: 0.95,       // excellent quality
    capacity_multiplier: 0.7,   // significant throughput reduction
    construction_cost: 60_000,
    monthly_cost: 3_500,
    health_modifier: +2.0,      // slight health bonus for ultra-clean water
    footprint: (4, 4),
    removes_industrial_contaminants: true,  // can process polluted source water
}
```
- Activated carbon, UV disinfection, membrane filtration, advanced oxidation.
- Required when source water is polluted (industrial runoff, algal toxins).
- Health bonus: residents in areas served by advanced treatment have slightly better health outcomes.
- Capacity tradeoff: advanced treatment processes more slowly, so the effective throughput is only 70% of the pipe capacity.

#### Treatment Chain

The treatment chain is: **Source --> Raw Water Main --> Treatment Plant --> Treated Water Main --> Distribution**

In the hybrid model:
1. Player places a water source (e.g., river intake at the Yarkon).
2. Player draws a raw water trunk main from the source to a treatment plant.
3. Player builds a treatment plant of chosen level.
4. Player draws treated water trunk mains from the treatment plant toward neighborhoods.
5. Distribution to buildings along roads is automatic.

The treatment plant acts as a **transformer node** in the network: it accepts raw water on one side and outputs treated water on the other side. This is analogous to a power substation transforming high voltage to distribution voltage.

### Pressure Simulation

Water pressure is the single most interesting simulation mechanic in the water system because it interacts with terrain elevation, creating spatial gameplay.

#### The Physics (Simplified for Game)

Real water pressure follows: `P = rho * g * h`, where `h` is the height of the water column above the measurement point. In practical terms:
- For every 10 meters of elevation, water pressure changes by approximately 1 atmosphere (14.7 psi or ~100 kPa).
- Water flows downhill naturally (gravity). To push water uphill, you need pumps.
- A water tower works by lifting water to a height and letting gravity provide pressure to the distribution network below.

#### Game Model

```rust
struct WaterPressure {
    // Pressure at each cell, in normalized units (0.0 = no pressure, 1.0 = ideal, >1.0 = excess)
    // Below 0.3: insufficient for upper-floor service (high-rise buildings affected)
    // Below 0.1: insufficient for ground-floor service (all buildings affected)
    // Above 1.5: risk of pipe bursts
    pressure: Vec<f32>,  // 256*256 = 65,536 entries
}
```

**Pressure calculation algorithm:**
1. Start at each pressure source (water tower, pump station, treatment plant output).
2. BFS outward through the trunk main network.
3. At each step, adjust pressure based on elevation difference:
   ```
   pressure_at_neighbor = pressure_at_current - (elevation_neighbor - elevation_current) * ELEVATION_PRESSURE_FACTOR
   ```
4. Also subtract a friction loss per unit distance:
   ```
   pressure_at_neighbor -= distance * FRICTION_LOSS_PER_CELL
   ```
5. Also subtract a demand loss based on how many buildings are drawing water from this section:
   ```
   pressure_at_neighbor -= local_demand / pipe_capacity
   ```

**Elevation Pressure Factor:**
- The Megacity grid uses elevation values in the range 0.15-0.65 (from the Tel Aviv terrain generation).
- `ELEVATION_PRESSURE_FACTOR = 3.0` means a cell at elevation 0.65 relative to a source at elevation 0.35 loses `(0.65 - 0.35) * 3.0 = 0.9` pressure units.
- This makes hilltop neighborhoods challenging to serve -- they need pump stations or water towers at elevation.

**Water Towers:**
- A water tower is essentially a "pressure battery." It stores water at elevation and provides consistent pressure to the surrounding area.
- In-game, a water tower acts as a pressure source with its effective pressure determined by its elevation + tower height bonus:
  ```
  effective_pressure = 1.0 + TOWER_HEIGHT_BONUS - (tower_elevation - cell_elevation) * ELEVATION_PRESSURE_FACTOR
  ```
- Tower height bonus = 0.5 (equivalent to ~15m of elevation).
- This means a water tower at elevation 0.5 provides good pressure to everything below elevation 0.5 + 0.5/3.0 = ~0.67, but poor pressure to anything above.

**Pump Stations:**
- Pump stations actively boost pressure at a point in the network.
- They consume power (integration with the power grid) and add a configurable pressure boost.
- Placement strategy: put pump stations at the base of hills to push water uphill to elevated neighborhoods.
- Failure mode: if power goes out, pump stations stop, and elevated neighborhoods lose pressure.

#### Pressure Effects on Gameplay

| Pressure Level | Effect |
|---------------|--------|
| > 1.5 | Pipe burst risk increases. Old pipes are especially vulnerable. |
| 1.0 - 1.5 | Ideal operating range. Full service to all buildings. |
| 0.5 - 1.0 | Adequate. No visible issues but fire response slightly impaired. |
| 0.3 - 0.5 | Low pressure warning. High-rise buildings (level 3+) lose water on upper floors. Happiness penalty. |
| 0.1 - 0.3 | Critical low pressure. All buildings affected. Significant happiness and health penalties. Fire response severely impaired. |
| 0.0 - 0.1 | No service. Buildings functionally have no water. Abandonment risk. |

**Fire Hydrant Integration:**
- Fire response effectiveness is modulated by local water pressure.
- The existing fire system (`crates/simulation/src/fire.rs`) uses `ServiceCoverageGrid::has_fire()` to determine fire coverage.
- With the pressure system, fire coverage effectiveness becomes: `coverage_effectiveness = base_coverage * pressure_factor`.
- At pressure < 0.3, fire trucks arrive but cannot effectively fight the fire (no water pressure in hydrants).
- This creates an emergent scenario: a hillside neighborhood with a fire station but no pump station has fire coverage on paper but cannot actually fight fires effectively.

### Pipe Types

The hybrid model uses three tiers of pipes, but only the first tier is player-drawn:

#### Trunk Main (Player-Drawn)

```rust
struct TrunkMain {
    // Same Bezier curve representation as RoadSegment
    id: PipeSegmentId,
    start_node: PipeNodeId,
    end_node: PipeNodeId,
    p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2,
    pipe_type: TrunkMainType,
    capacity_liters_per_second: f32,
    current_flow: f32,
    age_days: u32,
    condition: f32,  // 1.0 = new, 0.0 = failed
    material: PipeMaterial,
    diameter_mm: u32,
}

enum TrunkMainType {
    RawWater,     // Source to treatment plant
    TreatedWater, // Treatment plant to distribution
    Sewage,       // Collection to treatment
    Storm,        // Stormwater drainage
    Combined,     // Combined sewer (handles both sewage + storm)
}

enum PipeMaterial {
    CastIron,         // Cheap, corrodes, 50-year life
    DuctileIron,      // Moderate cost, 75-year life
    HDPE,             // Moderate cost, 100-year life, flexible
    ReinforcedConcrete, // Expensive, 100+ year life, large diameter
    Steel,            // Most expensive, 80-year life, highest pressure rating
}
```

**Capacity and sizing:**
- Small trunk main (300mm): 50 L/s, serves ~5,000 people, costs $500/cell
- Medium trunk main (600mm): 200 L/s, serves ~20,000 people, costs $1,500/cell
- Large trunk main (1200mm): 800 L/s, serves ~80,000 people, costs $5,000/cell
- Mega trunk main (2400mm): 3,200 L/s, serves ~300,000 people, costs $15,000/cell

Player can upgrade trunk mains in-place (dig up, replace with larger pipe) at ~60% of new construction cost but with temporary service disruption to connected buildings.

#### Distribution Main (Automatic Along Roads)

```rust
struct DistributionNetwork {
    // Not individual entities -- represented as a grid property
    // Each road cell either has or does not have distribution service
    // Capacity is derived from the trunk main feeding it
    served_by_trunk: Option<PipeSegmentId>,
    distribution_capacity: f32,  // inherited from trunk main, divided by branch count
    pressure: f32,               // calculated from trunk pressure minus elevation delta
}
```

Distribution mains auto-extend along roads within a configurable radius of trunk main connections. Default radius: 15 cells from any trunk main connection point. This means a single trunk main running through a neighborhood automatically provides distribution service to all roads within 15 cells.

#### Service Lateral (Fully Automatic, Invisible)

Buildings within 3 cells of a road with distribution service automatically connect. This is purely a coverage check, similar to the existing `propagate_utilities()` BFS but informed by the distribution network state rather than simple source proximity.

### Demand Calculation

Water demand determines how much flow a pipe network must carry and whether sources/treatment plants have sufficient capacity.

**Per-capita demand (based on real-world figures adjusted for game balance):**

```rust
fn water_demand_per_person_per_day(zone_type: ZoneType, building_level: u8) -> f32 {
    // Returns liters per person per day
    match zone_type {
        ZoneType::ResidentialLow => 200.0 + building_level as f32 * 20.0,
        // Low-density: larger lots, gardens to water, pools
        ZoneType::ResidentialHigh => 150.0 + building_level as f32 * 10.0,
        // High-density: smaller units, shared infrastructure, more efficient
        ZoneType::CommercialLow => 300.0,
        // Small shops, restaurants (per employee equivalent)
        ZoneType::CommercialHigh => 500.0,
        // Large commercial: cooling systems, food service, cleaning
        ZoneType::Industrial => 1500.0,
        // Industrial: process water, cooling, cleaning. 5-10x residential.
        ZoneType::Office => 100.0,
        // Office: mostly just restrooms and kitchenettes
        ZoneType::None => 0.0,
    }
}
```

**Demand modifiers:**
- **Season:** Summer 1.3x (garden watering, cooling), Winter 0.9x (already in `Weather::water_multiplier()`).
- **Weather event:** Heat wave 1.6x, rain 0.8x.
- **Building level:** Higher-level buildings have slightly higher per-capita demand (luxury fixtures, more bathrooms per unit).
- **Time of day:** Peak demand 7-9 AM and 5-8 PM (morning showers, evening cooking). Minimum 2-4 AM (sleeping). Peak is ~2x average. This integrates with the existing `GameClock` system.

**Aggregate demand calculation (per trunk main):**
1. Sum demand from all buildings served by this trunk main's distribution network.
2. Apply seasonal and weather modifiers.
3. Apply time-of-day peak factor.
4. Compare to trunk main capacity.
5. If demand > capacity: pressure drops proportionally. Buildings furthest from the trunk main (or highest elevation) lose service first.

### Failure Modes

Failure modes are what make infrastructure interesting as a game mechanic. Without failures, pipes are just a prerequisite checkbox.

#### Pipe Bursts

**Mechanics:**
- Each trunk main has a `condition` value that degrades over time based on material, age, and soil conditions.
- Condition degrades at: `condition -= (1.0 / expected_life_days) * soil_corrosivity_factor`
- When condition drops below 0.2, there is a per-tick probability of burst: `burst_chance = (0.2 - condition) * 0.01`
- When a burst occurs:
  - All buildings downstream of the burst lose service immediately.
  - Water pools on the surface at the burst location (visual: water spout geyser animation).
  - Repair crews are dispatched automatically if a maintenance budget exists.
  - Repair takes time proportional to pipe size: small 2 days, medium 5 days, large 10 days.
  - During repair, player can route water through alternative trunk mains if the network has redundancy.

**Player response:**
- Proactive: inspect and replace aging pipes before they burst (maintenance budget system).
- Reactive: manage the outage by ensuring network redundancy.
- Strategic: choose pipe materials based on expected maintenance lifecycle cost vs upfront cost.

**Integration with existing systems:**
- Burst events generate entries in the `EventJournal` (already implemented).
- Repair costs deduct from the `CityBudget.treasury` (already implemented).
- Affected citizens suffer happiness penalties via the existing `happiness::update_happiness` system.

#### Contamination Events

**Mechanics:**
- If a pipe passes through an area with high `WaterPollutionGrid` levels and the pipe condition is below 0.5 (cracked), contamination can enter the drinking water supply.
- Contamination probability: `contam_chance = water_pollution_level * (1.0 - condition) * 0.001`
- When contamination occurs:
  - All buildings downstream receive contaminated water.
  - Health penalty applied to all residents in affected buildings: 2.0 health/day.
  - A "boil water advisory" event is generated (EventJournal entry).
  - Advisory lasts until the contaminated pipe section is repaired and the section is flushed.

**Integration with existing systems:**
- Uses `WaterPollutionGrid` (already implemented).
- Uses `WaterQualityGrid` from the groundwater system (already implemented).
- Health penalties apply through the same mechanism as `groundwater_health_penalty()` in `crates/simulation/src/groundwater.rs`.

#### Drought (Source Depletion)

**Mechanics:**
- During extended `WeatherEvent::HeatWave` or low-rainfall periods, water sources deplete:
  - River intake: capacity drops to `base_capacity * (1.0 - drought_severity)`.
  - Groundwater wells: capacity drops as `GroundwaterGrid` levels fall.
  - Lakes: slow depletion over multiple drought seasons.
  - Reservoirs: stored water draws down; if storage hits 0, capacity = 0.
  - Desalination: unaffected.
- When total source capacity < total demand:
  - Pressure drops system-wide.
  - The game can trigger "water rationing" policy (reduces residential demand by 20% but causes happiness penalty).
  - Player must invest in drought-resistant sources (desalination, reservoirs) or demand reduction.

**Integration with existing systems:**
- `Weather::current_event` and `Season` determine drought conditions.
- `GroundwaterGrid` already tracks aquifer levels.
- `Policies` resource (already implemented) can include water rationing as a toggle.

---

## Sewage/Wastewater Network

### The Gravity Mechanic: Why Sewage is the Best Infrastructure System for a Game

Of all underground infrastructure, sewage is the most compelling game mechanic because of one physical constraint: **sewage flows downhill by gravity**. Unlike water supply (which can be pressurized to go anywhere) or power (which flows through wires regardless of terrain), sewage fundamentally interacts with topography. The player must plan sewer routes that slope downhill from collection points to treatment plants, or install expensive pump stations where gravity routing is impossible.

This constraint transforms sewer planning from a simple connectivity exercise into a spatial puzzle. In the Tel Aviv map, with the coastline on the west (low elevation ~0.15-0.35) and hills rising to the east (elevation ~0.5-0.65), the natural sewage flow direction is west toward the coast. But the treatment plant should not be on the beachfront (land value, aesthetics, tourism). So the player faces a genuine tradeoff: put the treatment plant at a low point (cheaper gravity collection) or at a more convenient location (requires pump stations).

### Gravity Flow Simulation

#### The Physics (Simplified)

Real sewage pipes (called "gravity sewers" or "sanitary sewers") are not pressurized. They are partially-filled pipes that rely on a continuous downhill slope to move wastewater. The minimum slope depends on pipe diameter:
- 200mm pipe: 1:200 slope (0.5% grade, or 5mm drop per meter)
- 300mm pipe: 1:300 slope (0.33%)
- 600mm pipe: 1:600 slope (0.17%)
- 1200mm pipe: 1:1000 slope (0.1%)

Larger pipes need less slope because the larger cross-section produces more gravitational force relative to friction.

#### Game Model

```rust
struct SewerSegment {
    // Same Bezier representation as road/water segments
    id: PipeSegmentId,
    start_node: PipeNodeId,
    end_node: PipeNodeId,
    p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2,
    pipe_diameter: SewerSize,
    capacity_liters_per_second: f32,
    current_load: f32,
    slope: f32,            // calculated from elevation difference / arc_length
    flow_direction: FlowDirection,  // determined by slope: always downhill
    age_days: u32,
    condition: f32,
}

enum SewerSize {
    Small,   // 200mm, 10 L/s, serves ~2,000 people
    Medium,  // 450mm, 50 L/s, serves ~10,000 people
    Large,   // 900mm, 200 L/s, serves ~40,000 people
    Trunk,   // 1800mm, 1000 L/s, serves ~200,000 people
    Interceptor, // 3000mm, 5000 L/s, main city interceptor
}

enum FlowDirection {
    StartToEnd,  // normal gravity flow: start node is higher than end node
    EndToStart,  // reverse: end node is higher
    Flat,        // insufficient slope -- needs pump or redesign
}
```

#### Slope Validation

When the player draws a sewer trunk main, the game calculates the slope along the route using the `WorldGrid` elevation data:

```rust
fn calculate_sewer_slope(segment: &SewerSegment, grid: &WorldGrid) -> SewerSlopeResult {
    let start_elevation = grid.get(
        segment.start_grid_x,
        segment.start_grid_y
    ).elevation;
    let end_elevation = grid.get(
        segment.end_grid_x,
        segment.end_grid_y
    ).elevation;

    let elevation_diff = start_elevation - end_elevation;
    let slope = elevation_diff / segment.arc_length;

    let min_slope = segment.pipe_diameter.minimum_slope();

    if slope.abs() < min_slope {
        SewerSlopeResult::InsufficientSlope {
            actual: slope,
            required: min_slope,
            recommendation: "Increase pipe diameter or add a pump station",
        }
    } else if elevation_diff > 0.0 {
        SewerSlopeResult::GravityFlow {
            direction: FlowDirection::StartToEnd,
            slope,
        }
    } else {
        SewerSlopeResult::GravityFlow {
            direction: FlowDirection::EndToStart,
            slope: slope.abs(),
        }
    }
}
```

**UI Feedback During Drawing:**
- As the player draws a sewer segment, the UI shows a real-time slope indicator.
- Green: adequate slope for gravity flow. Arrow shows flow direction.
- Yellow: marginal slope. Will work but may have sediment buildup issues.
- Red: insufficient slope or uphill. Cannot function without a pump station.
- The elevation profile is displayed as a mini cross-section diagram alongside the Bezier preview (extending the existing `cursor_preview::draw_bezier_preview()` system).

#### Flow Direction and Network Topology

Sewer networks are tree-structured (not mesh). Flow converges from many small pipes into fewer large pipes, all flowing toward the treatment plant. This is fundamentally different from water supply (which is a pressurized mesh) and power (which can be ring/mesh topology).

**Network validation rules:**
1. Every sewer segment must have a valid gravity flow direction or be served by a pump station.
2. Every collection point must have a path (via gravity flow and pump stations) to a treatment plant.
3. No flow loops are allowed (sewage flowing in a circle would be catastrophic).
4. The network must form a directed acyclic graph (DAG) with treatment plants as sinks.

The existing `CsrGraph` (CSR-format adjacency graph used for road pathfinding) could be reused for sewer network flow analysis, but with directed edges following gravity.

### Combined vs Separate Sewer Systems

This is a design choice the player makes per-district, with major implications for stormwater management and pollution events.

#### Combined Sewer System (CSS)

**Real-world context:** Most cities built before 1950 have combined sewers. A single pipe carries both sanitary sewage (from toilets, sinks, showers) and stormwater runoff (from rain). This was simpler and cheaper to build, but creates a critical failure mode: during heavy rain, the combined flow overwhelms the system and untreated sewage spills into waterways (called a "Combined Sewer Overflow" or CSO event).

**Game implementation:**
```rust
struct CombinedSewerSegment {
    // All fields from SewerSegment, plus:
    stormwater_capacity_fraction: f32,  // what fraction of capacity is allocated to storm runoff
    cso_threshold: f32,                 // flow level above which overflow occurs
    cso_outfall_location: Option<(usize, usize)>,  // where overflow discharges (river/coast)
}
```

**Gameplay effects:**
- **Cheaper to build:** Combined sewers cost 70% of separate system (one pipe instead of two).
- **CSO events during storms:** When `Weather::current_event == WeatherEvent::Storm` or `WeatherEvent::Rain`, stormwater enters the combined sewer. If total flow > capacity, overflow discharges at outfall points.
- **Overflow consequences:**
  - `WaterPollutionGrid` increases at outfall location and downstream.
  - Health penalty for citizens near outfall areas.
  - Happiness penalty city-wide ("raw sewage in the river" is a media event).
  - Environmental regulation penalty (if policies include environmental standards).
- **Upgrade path:** Player can later separate the system by building dedicated storm drains (expensive retrofit).

**Strategic value:**
- Early game: build combined sewers because they are cheap.
- Mid game: start experiencing CSO events as the city grows and impervious surface area increases stormwater runoff.
- Late game: invest in sewer separation, green infrastructure, or CSO storage tunnels.

This progression mirrors real urban development history and creates a natural mid-to-late game infrastructure challenge.

#### Separate Sewer System (SSS)

**Game implementation:**
- Two independent pipe networks: sanitary sewer and storm drain.
- Sanitary sewer carries only sewage to the wastewater treatment plant.
- Storm drain carries only rainwater, discharging directly to waterways (no treatment needed if properly managed).
- No CSO events under normal conditions.
- Higher upfront cost (two networks instead of one) but lower long-term risk.

**Gameplay effects:**
- **More expensive:** 100% cost (vs 70% for combined).
- **No CSO events:** Storm water goes to storm drains, sewage goes to treatment.
- **Separate capacity planning:** Must size each system independently.
- **Illegal connections:** If a policy "enforcement" level is low, some buildings may illegally connect storm drains to sanitary sewers (or vice versa), causing localized issues. This is a flavor mechanic, not a core system.

### Lift/Pump Stations

When terrain makes gravity flow impossible, the player must build pump stations to lift sewage over elevation obstacles.

**Real-world mechanics:** A "lift station" or "pump station" is an underground chamber with submersible pumps that collect gravity-fed sewage and pump it uphill to a higher elevation, where gravity flow can resume. The sewage is pumped through a "force main" (pressurized pipe) from the lift station to a discharge point at higher elevation.

**Game implementation:**
```rust
struct LiftStation {
    grid_x: usize,
    grid_y: usize,
    pump_capacity_liters_per_second: f32,
    lift_height: f32,       // how much elevation the station can overcome
    power_consumption: f32,  // kW required to operate
    wet_well_volume: f32,    // storage capacity when pumps are off
    pumps_active: bool,      // false if power failure
    age_days: u32,
    condition: f32,
}
```

**Gameplay effects:**
- **Power dependency:** Lift stations require power. If the power grid fails, the lift station stops, and sewage backs up.
- **Backup generators:** Player can add backup generators (additional cost) to keep critical lift stations running during power outages. This is a redundancy vs cost tradeoff.
- **Wet well storage:** When pumps stop, the wet well stores incoming sewage temporarily. Capacity is typically 15-30 minutes of average flow. After that, sewage overflows.
- **Maintenance:** Lift stations require regular maintenance. Neglected stations fail more frequently.
- **Sound/odor:** Lift stations generate noise pollution (integrate with `NoisePollutionGrid`) and are undesirable near residential areas.

**Strategic placement:**
- Place at the low points where gravity-collected sewage needs to be lifted over a ridge to reach the treatment plant.
- Minimize lift stations because each one is a point of failure and ongoing cost.
- The Tel Aviv map's terrain (rising from west coast to east hills) means sewage naturally flows westward. If the treatment plant is on the eastern side of the city (near the Ayalon Highway corridor at x~185), multiple lift stations would be needed to push sewage uphill from the coastal neighborhoods.

### Wastewater Treatment Plants

Treatment plants are the sink nodes of the sewer network. All collected sewage must eventually reach a treatment plant.

#### Treatment Levels

**Primary Treatment:**
```
WastewaterTreatment::Primary {
    removal_efficiency: 0.40,    // removes 40% of contaminants (solids settle out)
    effluent_quality: 0.3,       // discharged water is still quite polluted
    capacity_population_equivalent: 50_000,
    construction_cost: 30_000,
    monthly_cost: 1_500,
    footprint: (4, 3),
    odor_radius: 15,             // cells of NoisePollution/odor impact
}
```
- Screens, grit removal, primary sedimentation.
- Cheapest but discharged effluent still pollutes receiving waters significantly.
- Increases `WaterPollutionGrid` at the effluent outfall.

**Secondary Treatment:**
```
WastewaterTreatment::Secondary {
    removal_efficiency: 0.85,
    effluent_quality: 0.7,
    capacity_population_equivalent: 100_000,
    construction_cost: 80_000,
    monthly_cost: 4_000,
    footprint: (5, 4),
    odor_radius: 10,
}
```
- Biological treatment (activated sludge or trickling filters).
- Standard for modern cities. Effluent is clean enough for safe river discharge.
- Moderate `WaterPollutionGrid` impact at outfall.

**Tertiary Treatment (Advanced):**
```
WastewaterTreatment::Tertiary {
    removal_efficiency: 0.98,
    effluent_quality: 0.95,
    capacity_population_equivalent: 200_000,
    construction_cost: 200_000,
    monthly_cost: 12_000,
    footprint: (6, 5),
    odor_radius: 5,
    water_reuse: true,           // effluent can be recycled as non-potable water
}
```
- Nutrient removal, advanced filtration, disinfection.
- Effluent is clean enough for water reuse (irrigation, industrial cooling, toilet flushing).
- Water reuse reduces demand on the potable water supply system by up to 30%.
- Minimal `WaterPollutionGrid` impact.

#### Capacity Planning

**Rule of thumb:** Sewage generation = ~80% of water consumption. This is because some water is consumed (drinking, cooking, garden evaporation) rather than entering the sewer.

```rust
fn sewage_generation_per_person_per_day(zone_type: ZoneType) -> f32 {
    water_demand_per_person_per_day(zone_type, 1) * 0.8
}
```

If the treatment plant capacity is exceeded:
1. First 110%: Treatment efficiency degrades (effluent quality drops).
2. 110-130%: Partial bypass -- some untreated sewage is discharged directly.
3. Above 130%: Full bypass -- raw sewage discharge at outfall. Major health and environmental event.
4. Sewage backs up into the collection system, potentially causing street-level flooding in low-lying areas.

### Overflow Mechanics

Sewage overflow is one of the most impactful failure events in the game because it affects health, happiness, water quality, land value, and tourism simultaneously.

**CSO (Combined Sewer Overflow) Event:**
- Triggered when: rain/storm weather + combined sewer system + flow > 80% capacity.
- Duration: lasts as long as the weather event plus 1-2 days of aftermath.
- Effects:
  - `WaterPollutionGrid` increases by 30-80 at outfall cells, spreading via existing diffusion logic.
  - Health penalty to citizens within 5 cells of overflow points.
  - Happiness penalty of -10 to all citizens in the affected district.
  - Land value drops by 15% within 10 cells of overflow points.
  - Tourism penalty: -20% tourist visits for 7 days after event.
  - `EventJournal` entry: "Combined sewer overflow event in [district name]."

**Treatment Plant Overflow:**
- Triggered when: sewage flow > plant capacity.
- Effects similar to CSO but more severe because it is continuous (not weather-dependent).
- Fix: expand treatment plant capacity, build additional plants, or reduce water consumption upstream.

**Sewer Backup:**
- Triggered when: pipe capacity exceeded + no overflow outfall available.
- Sewage backs up and eventually surfaces at manholes in low-lying areas.
- Visual: brown water pooling on surface cells (terrain rendering modification).
- Health penalty: severe. Exposed raw sewage causes disease outbreaks.
- Building abandonment risk in affected cells.

---

## Stormwater Drainage

### Why Stormwater Matters

Stormwater drainage is the most terrain-interactive infrastructure system and the one most affected by the player's development decisions. Every building, road, and parking lot the player places replaces permeable ground (grass, soil) with impervious surface (concrete, asphalt). Impervious surface prevents rainfall from soaking into the ground and instead generates surface runoff that must be collected and conveyed somewhere. A city with 50% impervious coverage generates roughly 5x the stormwater runoff of undeveloped land.

This creates a progressive infrastructure challenge:
- Small village: no stormwater infrastructure needed. Rain soaks into the ground.
- Small city (10k pop): some localized flooding during storms. Player notices water pooling.
- Medium city (50k pop): regular flooding without drainage infrastructure. Player must invest.
- Large city (200k+ pop): major flood events without comprehensive drainage. Underground storm drains, retention ponds, and green infrastructure all needed.

This progression is ideal for a city builder because it emerges naturally from the player's actions (building more = more runoff) rather than being an arbitrary unlock.

### Surface Runoff Calculation

**Impervious Surface Tracking:**

Each grid cell has an effective imperviousness value based on its contents:

```rust
fn cell_imperviousness(cell: &Cell, building: Option<&Building>) -> f32 {
    match cell.cell_type {
        CellType::Water => 0.0,   // water cells absorb (they ARE the drainage target)
        CellType::Road => 0.95,   // roads are almost fully impervious
        CellType::Grass => {
            if let Some(building) = building {
                // Building footprint is impervious, but zone type matters
                match building.zone_type {
                    ZoneType::Industrial => 0.90,      // large paved areas, warehouses
                    ZoneType::CommercialHigh => 0.85,   // large buildings, parking
                    ZoneType::CommercialLow => 0.70,    // smaller lots, some landscaping
                    ZoneType::Office => 0.80,           // office parks
                    ZoneType::ResidentialHigh => 0.75,   // apartments with some common area
                    ZoneType::ResidentialLow => 0.45,    // houses with yards
                    ZoneType::None => 0.05,             // undeveloped grass
                }
            } else {
                0.05  // natural grass, nearly fully permeable
            }
        }
    }
}
```

**Runoff calculation per storm event:**

```rust
fn calculate_runoff(
    grid: &WorldGrid,
    buildings: &Query<&Building>,
    weather: &Weather,
    rainfall_mm: f32,  // mm of rainfall in this time period
) -> Vec<f32> {
    // Returns runoff volume in liters per cell for this time period
    let cell_area_m2 = (CELL_SIZE * CELL_SIZE) as f32;  // 16*16 = 256 m2
    let mut runoff = vec![0.0f32; GRID_WIDTH * GRID_HEIGHT];

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let cell = grid.get(x, y);
            let imperv = cell_imperviousness(cell, /* lookup building */);
            // Rational method: Q = C * i * A
            // C = runoff coefficient (imperviousness)
            // i = rainfall intensity (mm/hr -> L/m2/hr)
            // A = area (m2)
            let runoff_depth_mm = rainfall_mm * imperv;
            let runoff_volume_liters = runoff_depth_mm * cell_area_m2;
            // 1mm of rain on 1m2 = 1 liter
            runoff[y * GRID_WIDTH + x] = runoff_volume_liters;
        }
    }
    runoff
}
```

**Rainfall intensity by weather event:**

| Weather Event | Rainfall (mm/hr) | Duration (hrs) | Return Period |
|--------------|-------------------|----------------|---------------|
| Clear | 0 | 0 | - |
| Light Rain | 5 | 4-8 | Monthly |
| Rain | 15 | 2-6 | Quarterly |
| Storm | 40 | 1-3 | Annual |
| Severe Storm | 80 | 0.5-1 | 10-year |
| Catastrophic | 150 | 0.5 | 100-year |

The existing `Weather` system generates Rain and Storm events. These map to the middle tiers. Severe and catastrophic events can be added as rare disaster-level events.

### Surface Drainage Systems

**Roadside Ditches:**
- Cheapest option. Open channels along road edges collect surface runoff.
- Auto-generated along roads if the player enables "roadside drainage" in road properties.
- Capacity: handles up to 15 mm/hr rainfall intensity.
- Visual: subtle channel visible at road edges.
- Limitation: only works along roads. Does not handle areas far from roads.
- No underground component -- purely surface.

**Channels and Culverts:**
- Open channels (player-drawn) that convey water from high ground to low ground or to water bodies.
- Culverts pass under roads (automatically created where channels cross roads).
- Capacity: 15-50 mm/hr depending on size.
- Cost: $50-200 per cell depending on size.
- Can double as park amenities if landscaped ("stormwater feature" as a park element).

### Underground Storm Drains

In a modern separate sewer system, underground storm drains are dedicated pipes that collect surface runoff through catch basins (street-level drain grates) and convey it to an outfall (river, ocean, retention pond).

**Game implementation:**

Storm drains use the same trunk main infrastructure as sewage, but are a separate network:

```rust
struct StormDrainSegment {
    // Same Bezier representation
    id: PipeSegmentId,
    start_node: PipeNodeId,
    end_node: PipeNodeId,
    p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2,
    diameter: StormDrainSize,
    capacity_liters_per_second: f32,
    current_flow: f32,
    slope: f32,
    outfall: Option<OutfallLocation>,
}

enum StormDrainSize {
    Small,    // 450mm, handles minor street drainage
    Medium,   // 900mm, handles neighborhood drainage
    Large,    // 1500mm, handles district drainage
    Box,      // 2000x1500mm box culvert, handles major drainage
    Tunnel,   // 3000mm+ tunnel, handles city-scale drainage
}
```

**Catch basin coverage:**
- Storm drains only collect runoff from cells within their surface collection area.
- Collection area = all road cells within 3 cells of the drain alignment, plus adjacent non-road cells within 1 cell of a road.
- Cells outside the collection area contribute runoff that flows overland according to elevation gradient (toward lower cells).

**Outfall requirements:**
- Every storm drain network must terminate at an outfall: a discharge point into a water body or retention facility.
- Outfall types: river outfall, ocean outfall, retention pond connection, infiltration basin.
- Storm drain discharge at outfall may carry pollutants (road oil, sediment, litter) -- contributes to `WaterPollutionGrid` at a lower level than sewage.

### Retention and Detention Facilities

These are storage facilities that temporarily hold stormwater to reduce peak flow rates and flooding.

**Retention Pond:**
```
StormwaterFacility::RetentionPond {
    storage_volume_liters: 500_000,    // holds 500 cubic meters
    footprint: (3, 3),                  // 3x3 grid cells
    construction_cost: 15_000,
    monthly_cost: 200,
    treatment_effect: true,            // sediment settles, minor water quality improvement
    aesthetic_bonus: 5.0,              // can be landscaped as a park feature
    evaporation_rate: 0.02,            // per day, higher in summer
    groundwater_recharge: true,        // slowly recharges GroundwaterGrid in radius
}
```

- Permanently holds water (has a pool even in dry weather).
- Provides water quality treatment through natural sedimentation.
- Can be landscaped as an amenity (positive land value effect, functions as a park).
- Slowly recharges groundwater (integrates with `GroundwaterGrid`).

**Detention Basin:**
```
StormwaterFacility::DetentionBasin {
    storage_volume_liters: 1_000_000,  // larger than retention pond
    footprint: (4, 3),
    construction_cost: 10_000,         // cheaper than retention pond (simpler construction)
    monthly_cost: 100,
    drain_rate: 0.1,                   // fraction of volume drained per hour
    dual_use: true,                    // can be used as sports field when dry
}
```

- Dry between storms (drains completely within 24-48 hours).
- Larger storage capacity per area than retention ponds.
- Can function as sports fields, parks, or open space when dry (dual use).
- No water quality treatment benefit.
- No groundwater recharge.

**Underground Storage Tanks/Tunnels:**
```
StormwaterFacility::UndergroundStorage {
    storage_volume_liters: 5_000_000,  // very large
    footprint: (2, 2),                  // small surface footprint (it's underground)
    construction_cost: 80_000,          // expensive
    monthly_cost: 500,
    depth_layer: UndergroundLayer::Medium,
    drain_rate: 0.05,                   // drains to treatment plant or outfall
}
```

- No surface land use impact (underground).
- Very expensive but ideal for dense urban areas where surface space is unavailable.
- Common in real-world cities: Tokyo's massive underground cisterns, Chicago's Tunnel and Reservoir Plan.
- Connects to the sewer system or dedicated drain for emptying.

### Green Infrastructure

Green infrastructure represents modern approaches to stormwater management that reduce runoff at the source rather than collecting and conveying it.

**Rain Gardens:**
```
GreenInfrastructure::RainGarden {
    infiltration_rate_mm_per_hr: 25.0,
    capture_area_cells: 4,              // handles runoff from 4 surrounding cells
    footprint: (1, 1),
    construction_cost: 2_000,
    monthly_cost: 50,
    aesthetic_bonus: 3.0,
    groundwater_recharge: true,
    pollution_filtering: true,          // removes some pollutants from runoff
}
```

- Small landscaped depressions planted with native vegetation.
- Absorb and filter runoff from surrounding impervious surfaces.
- Attractive and improve land value.
- Low capacity but many can be distributed throughout the city.
- Suitable for residential neighborhoods.

**Permeable Pavement:**
```
GreenInfrastructure::PermeablePavement {
    infiltration_rate_mm_per_hr: 40.0,
    cost_premium_vs_regular: 1.5,       // 50% more expensive than regular pavement
    applicable_to: [RoadType::Local, RoadType::Path],  // only low-traffic roads
    durability_reduction: 0.8,          // wears out 20% faster
    ice_risk_in_winter: true,           // water in pores freezes, creating black ice
}
```

- Road surface that allows water to pass through into the ground below.
- Only suitable for low-traffic roads (Local, Path). Heavy vehicles destroy the pore structure.
- Reduces runoff from roads by 60-80%.
- Winter hazard: water in pores freezes, creating ice and frost heave damage. Seasonal maintenance needed.
- Applied as a road upgrade (modify existing road properties) rather than a separate infrastructure placement.

**Bioswales:**
```
GreenInfrastructure::Bioswale {
    infiltration_rate_mm_per_hr: 15.0,
    capacity_liters: 20_000,
    length_cells: 4,                    // linear feature, 4 cells long
    construction_cost: 5_000,
    monthly_cost: 100,
    pollution_filtering: true,
    aesthetic_bonus: 2.0,
}
```

- Vegetated channels that slow, filter, and infiltrate stormwater.
- Linear features that run along roads or between developments.
- Moderate capacity, good filtration.
- Can serve as landscaping along boulevards (aesthetic bonus for Boulevard road type).

**Green Roofs:**
```
GreenInfrastructure::GreenRoof {
    retention_depth_mm: 25.0,           // retains first 25mm of rainfall
    imperviousness_reduction: 0.5,      // reduces building imperviousness by 50%
    applicable_to: [ZoneType::ResidentialHigh, ZoneType::CommercialHigh, ZoneType::Office],
    cost_per_building: 5_000,
    monthly_cost: 100,
    energy_savings: 0.05,               // 5% reduced heating/cooling
    aesthetic_bonus: 2.0,
    urban_heat_island_reduction: true,
}
```

- Applied as a building upgrade (extends the existing `building_upgrade.rs` system).
- Reduces building-level imperviousness by 50%.
- Retains the first 25mm of rainfall, significantly reducing runoff from minor storms.
- Insulation benefit: reduces heating and cooling demand (integrates with `HeatingGrid`).
- Reduces urban heat island effect.
- Only applicable to flat-roofed buildings (high-density zones).

### Flood Risk Model

When drainage capacity is exceeded, flooding occurs. The flood model determines which cells flood and how severely.

**Flood depth calculation:**
1. Calculate total runoff per cell (from rainfall and imperviousness).
2. Subtract drainage capacity (storm drains, green infrastructure infiltration).
3. Excess runoff accumulates as surface water.
4. Surface water flows downhill according to elevation gradient (8-connected cellular automaton).
5. Water pools in local elevation minima (depressions).
6. Flood depth at each cell determines damage.

```rust
fn simulate_surface_flow(
    excess_runoff: &[f32],  // liters per cell
    grid: &WorldGrid,
) -> Vec<f32> {
    // Returns flood depth in mm per cell
    let mut depth = vec![0.0f32; GRID_WIDTH * GRID_HEIGHT];

    // Convert excess runoff to depth
    let cell_area_m2 = CELL_SIZE * CELL_SIZE;  // 256 m2
    for i in 0..depth.len() {
        depth[i] = excess_runoff[i] / cell_area_m2;  // liters / m2 = mm
    }

    // Iterative flow: water moves to lowest neighbor
    // Run 10 iterations to approximate steady state
    for _ in 0..10 {
        let snapshot = depth.clone();
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let idx = y * GRID_WIDTH + x;
                let current_elev = grid.get(x, y).elevation + snapshot[idx] * 0.001;
                // 0.001 converts mm to the elevation scale

                // Find lowest neighbor
                let (neighbors, ncount) = grid.neighbors4(x, y);
                let mut min_elev = current_elev;
                let mut min_idx = idx;
                for &(nx, ny) in &neighbors[..ncount] {
                    let nidx = ny * GRID_WIDTH + nx;
                    let n_elev = grid.get(nx, ny).elevation + snapshot[nidx] * 0.001;
                    if n_elev < min_elev {
                        min_elev = n_elev;
                        min_idx = nidx;
                    }
                }

                if min_idx != idx && current_elev > min_elev {
                    let transfer = (current_elev - min_elev) * 500.0;
                    // transfer in mm, proportional to head difference
                    let transfer = transfer.min(depth[idx] * 0.25);
                    // max 25% of current depth per iteration
                    depth[idx] -= transfer;
                    depth[min_idx] += transfer;
                }
            }
        }
    }

    depth
}
```

**Flood damage thresholds:**

| Flood Depth (mm) | Effect |
|-------------------|--------|
| 0-50 | Nuisance flooding. Wet streets, minor inconvenience. No building damage. |
| 50-200 | Minor flooding. Basements may flood. Traffic speed -30%. |
| 200-500 | Moderate flooding. Ground-floor damage to low-density buildings. Building condition -10%. Traffic impossible on flooded roads. |
| 500-1000 | Severe flooding. Significant structural damage. Building condition -25%. Evacuation required. |
| > 1000 | Catastrophic flooding. Building destruction possible. Major health emergency. |

**Integration with existing systems:**
- `Weather` triggers rainfall events.
- `DisasterType::Flood` (already exists) represents catastrophic flood events.
- The flood model provides graduated response rather than the current binary disaster system.
- `TrafficGrid` is affected: flooded road cells have zero throughput.
- `HealthGrid` is affected: standing floodwater breeds disease.
- `LandValueGrid` is affected: flood-prone areas lose value.

---

## Power Distribution

### Overview: From Generation to Consumption

The electrical power grid is a hierarchical system that steps voltage down from generation level (~20 kV) through transmission (~110-500 kV) to distribution (~11 kV) to consumer level (~120/240V). Each step involves a transformer, and each link in the chain has capacity limits and failure modes.

Unlike water and sewage, power does not interact with elevation or gravity. But it has its own unique constraints:
- **Supply must equal demand at every instant.** Unlike water (which can be stored in tanks), electricity cannot be economically stored at scale (without batteries or pumped hydro). If demand exceeds supply, the grid frequency drops, and protective systems trip, causing blackouts.
- **Network topology determines reliability.** A radial network (single path from source to consumer) fails completely if any link breaks. A ring or mesh network provides redundancy.
- **Load varies dramatically by time of day.** Peak demand (summer afternoon, everyone running AC) can be 2x average demand.

The current Megacity implementation (`propagate_utilities()` in `crates/simulation/src/utilities.rs`) uses a simple BFS flood-fill from `UtilitySource` entities with a range limit. This models coverage but not capacity, load, or failure. The underground infrastructure upgrade replaces this with a proper grid simulation.

### Generation

The existing `UtilityType` enum already includes several power generation types. The upgrade adds capacity and output characteristics:

```rust
struct PowerPlant {
    plant_type: PowerPlantType,
    capacity_mw: f32,           // maximum output in megawatts
    current_output_mw: f32,     // actual current output
    fuel_cost_per_mwh: f32,     // operating cost
    emissions_factor: f32,      // CO2 per MWh, affects PollutionGrid
    reliability: f32,           // probability of being available (0.0-1.0)
    ramp_rate_mw_per_hour: f32, // how fast output can change
    min_output_fraction: f32,   // minimum stable output (nuclear can't go below 50%)
}

enum PowerPlantType {
    // Existing types mapped to characteristics:
    Coal {
        capacity_mw: 500.0,
        fuel_cost: 30.0,
        emissions: 1.0,       // highest
        reliability: 0.90,
        ramp_rate: 10.0,      // slow to ramp
        min_output: 0.3,
    },
    NaturalGas {
        capacity_mw: 300.0,
        fuel_cost: 50.0,
        emissions: 0.4,
        reliability: 0.95,
        ramp_rate: 50.0,      // fast ramp - good peaking plant
        min_output: 0.1,
    },
    Nuclear {
        capacity_mw: 1000.0,
        fuel_cost: 10.0,
        emissions: 0.0,
        reliability: 0.92,
        ramp_rate: 2.0,       // very slow ramp
        min_output: 0.5,      // can't go below 50%
    },
    Solar {
        capacity_mw: 100.0,   // per farm
        fuel_cost: 0.0,
        emissions: 0.0,
        reliability: "varies by time of day and weather",
        ramp_rate: "determined by sun",
        min_output: 0.0,
    },
    Wind {
        capacity_mw: 50.0,    // per turbine cluster
        fuel_cost: 0.0,
        emissions: 0.0,
        reliability: "varies by WindState.speed",
        ramp_rate: "determined by wind",
        min_output: 0.0,
    },
    Geothermal {
        capacity_mw: 200.0,
        fuel_cost: 5.0,
        emissions: 0.05,
        reliability: 0.98,    // very reliable baseload
        ramp_rate: 5.0,
        min_output: 0.4,
    },
}
```

**Solar output integration with existing systems:**
- Solar output = `capacity_mw * solar_factor`.
- `solar_factor` is determined by `GameClock.hour`: 0 at night, peaks at noon.
- `solar_factor *= weather_factor`: 1.0 for Clear, 0.3 for Rain, 0.1 for Storm.
- `solar_factor *= season_factor`: 1.0 for Summer, 0.7 for Spring/Autumn, 0.4 for Winter.

**Wind output integration:**
- Wind output = `capacity_mw * wind_factor`.
- `wind_factor` is determined by `WindState.speed` (already implemented in `crates/simulation/src/wind.rs`).
- Cut-in speed: below `speed < 0.1`, output = 0.
- Rated speed: at `speed >= 0.5`, output = capacity.
- Linear between cut-in and rated.
- Cut-out speed: above `speed > 0.9`, turbines shut down for safety, output = 0.

### Transmission Network

High-voltage transmission lines carry power from generation plants to substations near population centers. These are the "highways" of the power grid.

**Above-ground transmission lines:**
```rust
struct TransmissionLine {
    id: PowerLineId,
    start_node: PowerNodeId,  // generation plant or substation
    end_node: PowerNodeId,    // substation
    p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2,  // Bezier curve
    capacity_mw: f32,
    current_load_mw: f32,
    voltage_kv: f32,          // 110, 220, or 500 kV
    line_type: TransmissionLineType,
    condition: f32,
    age_days: u32,
}

enum TransmissionLineType {
    Overhead {
        tower_spacing_cells: usize,  // visual: towers placed every N cells
        right_of_way_width: usize,   // cells of land use restriction
        noise_radius: u8,            // electromagnetic hum
        visual_impact: f32,          // negative aesthetic effect on land value
        construction_cost_per_cell: f64,
        weather_vulnerability: f32,  // storms can knock down lines
    },
    Underground {
        depth_layer: UndergroundLayer,
        construction_cost_per_cell: f64,  // 5-10x overhead
        weather_vulnerability: f32,       // immune to storms
        visual_impact: f32,              // none
        repair_difficulty: f32,          // harder to repair than overhead
    },
}
```

**Above-ground vs underground tradeoffs:**

| Factor | Overhead | Underground |
|--------|----------|-------------|
| Construction cost | $2,000/cell | $12,000/cell |
| Maintenance cost | $50/cell/year | $20/cell/year |
| Storm vulnerability | High (ice, wind, trees) | None |
| Visual impact | Negative (land value -10% within 3 cells) | None |
| Noise | Low hum (NoisePollutionGrid +2 within 2 cells) | None |
| Repair time | 1-3 days | 5-15 days |
| Land use restriction | 3-cell wide right-of-way, no buildings | None (underground) |
| Capacity | 500 MW max per line | 300 MW max per cable |

**Strategic considerations:**
- Underground cables are ideal through urban areas (no visual/noise impact) but very expensive.
- Overhead lines are practical for long-distance transmission across undeveloped areas.
- Player can mix: overhead through rural/industrial areas, underground through residential/commercial.
- Storms (existing `Weather` system) can damage overhead lines, causing outages. Underground cables are immune.

### Substations

Substations transform voltage from transmission level to distribution level. They are the critical nodes of the power grid.

```rust
struct Substation {
    grid_x: usize,
    grid_y: usize,
    capacity_mw: f32,
    current_load_mw: f32,
    voltage_in_kv: f32,     // transmission voltage
    voltage_out_kv: f32,    // distribution voltage
    transformer_count: u8,  // redundancy
    coverage_radius: u32,   // cells of distribution coverage
    footprint: (usize, usize),
    noise_level: u8,        // transformers hum
    construction_cost: f64,
    monthly_cost: f64,
}
```

**Substation tiers:**

| Tier | Capacity | Coverage Radius | Footprint | Cost | Noise |
|------|----------|----------------|-----------|------|-------|
| Neighborhood | 20 MW | 20 cells | 1x1 | $5,000 | Low |
| District | 100 MW | 40 cells | 2x2 | $20,000 | Medium |
| Regional | 500 MW | 80 cells | 3x3 | $80,000 | High |

**Capacity overload:**
- At 80-100% load: normal operation. Temperature rises slightly.
- At 100-120%: overload warning. Transformer life shortened. Can sustain for hours.
- At 120-150%: critical overload. Protective systems activate within minutes.
- Above 150%: immediate trip. Substation disconnects to prevent damage. All served buildings lose power.

### Grid Topology and Reliability

The topology of the power network determines how it responds to failures.

#### Radial Network (Simplest)

```
Plant → Substation A → [buildings]
                     → Substation B → [buildings]
```

- Each substation has one path back to the plant.
- If any link fails, everything downstream loses power.
- Cheapest to build.
- Appropriate for early game / small city.

#### Ring Network (Moderate Reliability)

```
Plant → Sub A → Sub B → Sub C → Plant (loop back)
         ↓        ↓        ↓
    [buildings] [buildings] [buildings]
```

- Each substation has two paths to a source.
- If one link fails, power can flow through the other direction.
- Costs ~30% more than radial (additional line segments to close the ring).
- Appropriate for mid-game / medium city.

#### Mesh Network (Highest Reliability)

```
Plant A → Sub A ←→ Sub B ← Plant B
           ↕    ×    ↕
         Sub C ←→ Sub D
```

- Multiple interconnected paths.
- Can survive multiple simultaneous failures.
- Most expensive (many redundant connections).
- Appropriate for late game / large city / critical areas (hospitals, data centers).

**Implementation:**

The power grid topology is represented as a graph where:
- Nodes = generation plants and substations
- Edges = transmission/distribution lines with capacity

When a failure occurs (line down, plant offline, substation overload):
1. Remove the failed component from the graph.
2. Recalculate power flow from all sources to all loads.
3. If any load cannot be reached from any source, those buildings lose power.
4. If remaining paths are over capacity, load shedding occurs (rolling blackouts).

This is essentially a max-flow/min-cut problem on a graph, which can be solved efficiently using the existing CSR graph infrastructure (adapted from `road_graph_csr.rs`).

### Blackout Cascade Simulation

Power grid cascading failures are one of the most dramatic real-world infrastructure events, and they make excellent game mechanics because the player can see the consequences of their design decisions.

**How a cascade works:**
1. A trigger event occurs: transmission line failure (storm), plant trip (mechanical failure), or demand spike (heat wave + everyone turns on AC).
2. Power that was flowing through the failed component must be redistributed to other paths.
3. Those other paths may now be overloaded, causing them to also trip.
4. Each trip redistributes more load to remaining components, potentially overloading them too.
5. The cascade stops when the system reaches a stable state -- which may be with a large portion of the grid blacked out.

**Game implementation:**

```rust
fn simulate_blackout_cascade(
    power_grid: &PowerGrid,
    initial_failure: FailureEvent,
) -> BlackoutResult {
    let mut grid = power_grid.clone();
    let mut blackout_areas: Vec<(usize, usize)> = Vec::new();
    let mut cascade_depth = 0;

    // Remove initial failure
    grid.remove_component(initial_failure.component_id);

    loop {
        // Recalculate power flow
        let flow_result = grid.solve_power_flow();

        // Check for overloaded components
        let mut new_failures: Vec<ComponentId> = Vec::new();
        for component in &grid.components {
            if flow_result.load(component.id) > component.capacity * 1.2 {
                // Overloaded beyond trip threshold
                new_failures.push(component.id);
            }
        }

        if new_failures.is_empty() {
            break;  // Cascade stopped
        }

        // Trip overloaded components
        for failure in &new_failures {
            grid.remove_component(*failure);
        }

        cascade_depth += 1;

        // Safety valve: max 10 cascade iterations
        if cascade_depth >= 10 {
            break;
        }
    }

    // Determine which areas lost power
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            if !grid.can_reach_source(x, y) {
                blackout_areas.push((x, y));
            }
        }
    }

    BlackoutResult {
        affected_cells: blackout_areas,
        cascade_depth,
        duration_hours: estimate_repair_time(&new_failures),
    }
}
```

**Blackout effects:**
- `has_power = false` for all affected cells. The existing `Cell.has_power` flag handles this.
- Buildings without power: no lighting (visual darkening via `day_night.rs`), no cooling/heating, no elevators (high-rise buildings partially non-functional).
- Lift stations without power: sewage backups (cascading infrastructure failure).
- Pump stations without power: water pressure drops in elevated areas.
- Traffic signals offline: traffic speed reduction in affected areas (`TrafficGrid` modifier).
- Happiness penalty: -15 for affected citizens.
- Economic loss: commercial and office buildings cannot operate, tax revenue drops.
- Crime increase: darkness + non-functional security systems (`CrimeGrid` modifier).

**Player response options:**
- Build redundant grid topology (rings, mesh) to prevent cascades.
- Install backup generators at critical facilities (hospitals, lift stations, pump stations).
- Build energy storage (batteries) to buffer short-term supply gaps.
- Implement demand response policies: during peak demand, automatically reduce non-essential loads.

### Distribution to Buildings

In the hybrid model, power distribution from substations to buildings works similarly to water distribution:

1. Substations define a coverage area (configurable radius).
2. Within the coverage area, distribution extends along roads automatically.
3. Buildings within 3 cells of a road with distribution service have power.
4. The existing `propagate_utilities()` BFS is upgraded to check for substation coverage rather than raw utility source proximity.

The key change from the current implementation: instead of `UtilitySource.range` defining a simple BFS radius, the substation's capacity determines how many buildings it can serve. Once the substation is at capacity, additional buildings in its coverage area do NOT receive power, even if they are within range. This creates demand-driven expansion: the player must add substations as the city grows, not just increase range.

---

## Metro/Subway Systems

### Design Philosophy

Metro systems are the crown jewel of underground infrastructure in a city builder. They represent the highest-cost, highest-impact infrastructure investment a player can make. A well-designed metro network transforms city traffic patterns, land values, and development density. A poorly designed one wastes billions in virtual currency.

The existing Megacity codebase already has `ServiceType::SubwayStation` as a service building. The underground infrastructure upgrade expands this from a single-building placement into a full network system with tunnels, lines, stations, rolling stock, and ridership simulation.

### Tunnel Construction Methods

The construction method determines cost, disruption, routing flexibility, and construction time. The player chooses a method when drawing a metro tunnel.

#### Cut-and-Cover

**Real-world mechanics:** Dig a trench along a street, build the tunnel structure in the trench, and cover it back up. The street is restored on top. This was the original metro construction method (London Underground, New York subway) and is still used where a metro line follows an existing road corridor.

**Game implementation:**
```rust
struct CutAndCoverTunnel {
    // Follows road alignment -- can only be drawn along existing roads
    road_alignment: Vec<(usize, usize)>,  // sequence of road cells above the tunnel
    depth: UndergroundLayer::Shallow,
    width: TunnelWidth::Standard,         // single track pair
    construction_cost_per_cell: f64,      // $8,000 per cell
    construction_time_per_cell_days: u32, // 5 days per cell
    surface_disruption_radius: u8,        // 3 cells -- traffic blocked during construction
}
```

**Gameplay effects during construction:**
- Road cells above the construction site are temporarily impassable (traffic reroutes).
- Construction noise affects happiness of nearby citizens (`NoisePollutionGrid` boost).
- Building access disrupted: commercial buildings adjacent to construction lose revenue.
- Construction proceeds at a visible pace: the player sees the trench progress cell by cell.

**Constraints:**
- Must follow existing road alignments. Cannot tunnel through building plots.
- Shallow depth only (avoids conflict with deeper utility pipes in most cases, but may conflict with sewers).
- Cannot cross water cells (rivers/coast) underground -- needs a bridge or bored tunnel for that.
- Stations must be at road intersections (entrances emerge at street level).

**When to use:**
- Early metro lines that follow major avenues.
- Cost-effective for straight routes along existing roads.
- Appropriate for the initial metro line (e.g., along Dizengoff Street or Ibn Gabirol Street on the Tel Aviv map).

#### Bored Tunnel (TBM -- Tunnel Boring Machine)

**Real-world mechanics:** A tunnel boring machine (a massive cylindrical cutting head) is lowered into a launch pit, bores through the earth, and lines the tunnel with concrete segments as it goes. The TBM can travel deep underground, independent of surface streets, and can pass under rivers, buildings, and other obstacles.

**Game implementation:**
```rust
struct BoredTunnel {
    // Free routing -- can go anywhere, any direction
    p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2,  // Bezier curve (not constrained to roads)
    depth: UndergroundLayer,  // Shallow, Medium, or Deep
    width: TunnelWidth,
    construction_cost_per_cell: f64,      // $25,000 per cell (3x cut-and-cover)
    construction_time_per_cell_days: u32, // 3 days per cell (faster than cut-and-cover once TBM is running)
    tbm_launch_cost: f64,                 // $50,000 one-time cost to set up TBM
    surface_disruption_radius: u8,        // 0 -- no surface disruption (it's deep underground)
    minimum_curve_radius_cells: u8,       // 8 cells -- TBMs cannot make sharp turns
}
```

**Gameplay effects during construction:**
- No surface disruption (the whole point of TBM).
- High upfront cost for the TBM launch.
- Construction is faster per cell once the TBM is running but has a long startup.
- Minimum curve radius constraint: TBMs are physically large and cannot make sharp turns. This forces the player to plan sweeping curves for underground routes, adding spatial challenge.

**Constraints:**
- Minimum curve radius of 8 cells (128m at CELL_SIZE=16). Sharp turns are not possible.
- Must avoid other underground infrastructure at the same depth layer (collision detection).
- TBM launch and retrieval pits need surface access (2x2 footprint at each end of the tunnel).
- Cannot bore through water-saturated ground (cells with `GroundwaterGrid` level > 200) without additional dewatering cost.

**When to use:**
- Routes that must cross rivers, go under built-up areas, or require deep tunnels.
- Later metro lines that need to cross or go below existing infrastructure.
- Express tunnels that take direct routes independent of surface street layout.

#### Cost Comparison

| Method | Cost/Cell | Time/Cell | Surface Disruption | Routing Flexibility | Depth |
|--------|-----------|-----------|-------------------|---------------------|-------|
| Cut-and-Cover | $8,000 | 5 days | High (3-cell radius) | Low (follows roads only) | Shallow |
| Bored (Shallow) | $25,000 | 3 days | None | High (free routing) | Shallow |
| Bored (Medium) | $30,000 | 3.5 days | None | High | Medium |
| Bored (Deep) | $40,000 | 4 days | None | High | Deep |

**Real-world reference:** Actual metro tunnel costs range from $100M to $2B+ per kilometer. At CELL_SIZE=16m, a cell-length of ~16m gives:
- Cut-and-cover: ~$8,000/cell = $500M/km (realistic for cut-and-cover in a medium-cost city)
- Bored: ~$25,000/cell = $1.56B/km (realistic for deep bored tunnel in an expensive city like NYC)

These costs are scaled for game balance but maintain the correct relative ratios.

### Station Design

Metro stations are the interface between the underground rail system and the surface city. Their design determines capacity, land use impact, and passenger experience.

#### Station Types

**Basic Station:**
```rust
struct MetroStation {
    grid_x: usize,
    grid_y: usize,
    station_type: StationType,
    platform_length: PlatformLength,
    entrance_locations: Vec<(usize, usize)>,  // surface cells where entrances are
    depth: UndergroundLayer,
    construction_cost: f64,
    monthly_cost: f64,
    daily_ridership: u32,
    capacity_passengers_per_hour: u32,
}

enum StationType {
    Side {
        // Two platforms flanking the tracks
        width_cells: usize,  // 2 (narrow) to 3 (standard)
        construction_cost: 50_000.0,
        capacity_modifier: 1.0,
    },
    Island {
        // Single central platform between tracks
        width_cells: usize,  // 2 (standard) to 3 (wide)
        construction_cost: 60_000.0,
        capacity_modifier: 1.1,  // slightly higher capacity (both sides accessible)
    },
    Stacked {
        // One platform above the other (for crossing lines)
        width_cells: usize,  // 2
        depth_layers: 2,     // occupies two underground layers
        construction_cost: 120_000.0,
        capacity_modifier: 0.9,  // slightly lower (vertical transfer adds delay)
        transfer_station: true,
    },
    CrossPlatform {
        // Platforms arranged for easy same-direction transfers
        width_cells: usize,  // 4 (very wide)
        construction_cost: 150_000.0,
        capacity_modifier: 1.3,  // highest capacity
        transfer_station: true,
    },
}
```

#### Platform Length

Platform length determines the maximum train length, which determines capacity per train.

```rust
enum PlatformLength {
    Short,    // 4 cars, 600 passengers/train, footprint 4 cells
    Standard, // 6 cars, 900 passengers/train, footprint 6 cells
    Long,     // 8 cars, 1200 passengers/train, footprint 8 cells
    Extended, // 10 cars, 1500 passengers/train, footprint 10 cells
}
```

**Upgrade path:** Platforms can be extended after construction, but at significant cost (must excavate additional underground space). This encourages the player to plan for future capacity when building stations initially.

**Capacity calculation:**
```
station_hourly_capacity = (trains_per_hour * passengers_per_train) * station_type_modifier
trains_per_hour = 60 / headway_minutes
```

Example: Standard platform (900 pax/train) with 3-minute headway = 20 trains/hour = 18,000 passengers/hour. With island platform modifier (1.1) = 19,800 pax/hr.

#### Entrance Placement

Station entrances are surface-level buildings that provide access to the underground platform. Their placement affects:
- **Pedestrian catchment:** Entrances within 5 cells of a building make that building "metro accessible." Multiple entrances spread across a neighborhood maximize the catchment area.
- **Surface land use:** Each entrance occupies a 1x1 surface cell. In dense urban areas, this means displacing a building or park.
- **Accessibility:** Entrances must be on or adjacent to a road cell (pedestrians need to walk to them).

**Ridership estimation:**
```rust
fn estimate_station_ridership(
    station: &MetroStation,
    grid: &WorldGrid,
    buildings: &Query<&Building>,
) -> u32 {
    let mut potential_riders = 0u32;
    let catchment_radius = 10;  // cells -- about 160m, typical walk shed

    for entrance in &station.entrance_locations {
        for dy in -(catchment_radius as i32)..=(catchment_radius as i32) {
            for dx in -(catchment_radius as i32)..=(catchment_radius as i32) {
                let x = entrance.0 as i32 + dx;
                let y = entrance.1 as i32 + dy;
                if x < 0 || y < 0 || (x as usize) >= GRID_WIDTH || (y as usize) >= GRID_HEIGHT {
                    continue;
                }
                let ux = x as usize;
                let uy = y as usize;
                let cell = grid.get(ux, uy);
                if let Some(building_entity) = cell.building_id {
                    // Building occupants within catchment are potential riders
                    // Actual ridership depends on whether their destination
                    // is also near a metro station
                    if let Ok(building) = buildings.get(building_entity) {
                        let dist = (dx.abs() + dy.abs()) as f32;
                        let distance_decay = 1.0 - (dist / catchment_radius as f32);
                        potential_riders += (building.occupants as f32 * distance_decay * 0.3) as u32;
                        // 30% mode share for metro within walking distance
                    }
                }
            }
        }
    }

    potential_riders
}
```

### Network Topology and Line Planning

#### Network Growth Stages

**Stage 1: Single Line**
- The first metro line. Typically connects a major residential area to a major employment center.
- On the Tel Aviv map: Jaffa (south) to Ramat Aviv (north) via the city center.
- 8-15 stations, 10-20 km of tunnel.
- Total cost: ~$500M-$1B in game currency.
- Ridership: 50,000-100,000 daily.

**Stage 2: Crossing Lines (X or +)**
- Second line crosses the first, creating one or two transfer stations.
- Transforms the network from a line into a connected system.
- On the Tel Aviv map: East-west line from the Ayalon corridor to the coast, crossing the N-S line at a central station.
- Transfer stations require stacked or cross-platform design (more expensive).

**Stage 3: Full Network (Multiple Lines)**
- 3-5 lines with multiple transfer points.
- Full coverage of the urbanized area.
- Network effects: ridership increases non-linearly as more destinations become accessible.
- On the Tel Aviv map: additional lines serving Ramat HaSharon (north), Bat Yam (south), and Holon (east).

#### Line Properties

```rust
struct MetroLine {
    id: MetroLineId,
    name: String,            // "Red Line", "Green Line", etc.
    color: Color,            // for map display
    stations: Vec<MetroStationId>,  // ordered list
    tunnel_segments: Vec<TunnelSegmentId>,
    rolling_stock: RollingStock,
    headway_seconds: u32,    // time between trains
    average_speed_kph: f32,
    daily_ridership: u32,
    revenue_per_ride: f64,
    operating_cost_per_day: f64,
}

struct RollingStock {
    train_count: u32,        // number of trains on the line
    cars_per_train: u32,     // must be <= platform_length
    passengers_per_car: u32, // 150 standing + seated
    maximum_speed_kph: f32,
    acceleration_mps2: f32,
    age_years: u32,
    condition: f32,
    purchase_cost_per_train: f64,
}
```

**Headway calculation:**
```
minimum_headway_seconds = (line_length_km / average_speed_kph * 3600 * 2) / train_count
// Factor of 2: trains run in both directions, so effective frequency doubles
```

A line with 10 trains, 15km length, and 35 km/h average speed:
- Round trip time = 15/35 * 3600 * 2 = ~3,085 seconds (~51 minutes)
- Minimum headway = 3,085 / 10 = ~309 seconds (~5.1 minutes)

The player can add more trains to reduce headway (increase frequency) but each train costs money to purchase and operate.

### Integration with Surface Transit

Metro stations should integrate with the surface transit network to maximize ridership and accessibility.

**Bus-metro integration:**
- The existing `ServiceType::BusDepot` provides bus service.
- Bus routes should terminate at or pass through metro station locations.
- A "transit hub" designation can be applied to metro stations, which automatically attracts bus route termini.
- In the movement system (`crates/simulation/src/movement.rs`), citizen pathfinding should consider metro + bus as a combined transit option.

**Pedestrian access:**
- Metro stations generate pedestrian traffic. A station with 50,000 daily riders means 50,000 people walking to/from station entrances.
- This pedestrian flow should affect:
  - `TrafficGrid` (pedestrian density near entrances)
  - Commercial land value (foot traffic = customers)
  - Noise levels (crowds)
  - Safety/crime (crowded areas can have pickpocketing)

**Park-and-ride:**
- Suburban metro stations can have park-and-ride facilities (parking lots at the station).
- This extends the catchment area beyond walking distance.
- Park-and-ride generates vehicle traffic to the station but reduces vehicle traffic beyond it.

### Construction Phases

Metro construction should be a visible, multi-phase process that creates temporary disruption and generates anticipation.

**Phase 1: Planning and Approval (instant in game, but with cost)**
- Player draws the line route and places stations.
- Total cost is calculated and displayed.
- Cost is deducted from treasury (or financed via the existing `LoanBook` system).
- All station sites are marked on the surface (planning markers).

**Phase 2: Tunnel Boring / Excavation**
- For cut-and-cover: road cells above the tunnel are blocked. Construction progress visible as an excavation.
- For bored tunnel: TBM launch pits visible at each end. Progress tracked but not visible on surface.
- Duration: depends on tunnel length and method. A 15km bored tunnel takes ~1-2 in-game years.
- Construction creates noise and disruption during Phase 2.

**Phase 3: Station Construction**
- Stations are built simultaneously with tunnel boring.
- Station excavation is visible on the surface (large construction site).
- Duration: 6-12 months per station.

**Phase 4: Track and Systems Installation**
- Tracks, signaling, power systems, ventilation installed.
- Duration: 3-6 months after tunnel completion.
- Not visible on surface.

**Phase 5: Testing and Commissioning**
- Test runs (no revenue passengers).
- Duration: 2-3 months.
- Occasional test train movements visible at stations.

**Phase 6: Operational**
- Line opens. Immediate ridership (pent-up demand).
- Land values near stations jump (10-30% increase within 5-cell radius).
- Commercial development accelerates near stations.
- Traffic reduction on parallel surface roads.

### Cost Modeling

**Capital costs (one-time):**
| Component | Cost (game currency) |
|-----------|---------------------|
| Cut-and-cover tunnel, per cell | $8,000 |
| Bored tunnel (shallow), per cell | $25,000 |
| Bored tunnel (deep), per cell | $40,000 |
| Basic station (side platform) | $50,000 |
| Transfer station (stacked) | $120,000 |
| Transfer station (cross-platform) | $150,000 |
| TBM launch pit | $50,000 |
| Train (6-car set) | $30,000 |
| Platform extension (per cell) | $15,000 |

**Example: Tel Aviv North-South Line**
- Route: Jaffa (y=40) to Ramat Aviv (y=230), ~190 cells = ~3 km in-game
- 12 stations (10 side, 2 island)
- Bored tunnel (shallow): 190 * $25,000 = $4,750,000
- Stations: 10 * $50,000 + 2 * $60,000 = $620,000
- 15 trains: 15 * $30,000 = $450,000
- TBM: 2 * $50,000 = $100,000
- **Total: ~$5,920,000**

**Operating costs (monthly):**
| Component | Monthly Cost |
|-----------|-------------|
| Per station | $2,000 |
| Per train | $1,500 |
| Per tunnel cell | $50 |
| Total line (example above) | $46,100/month |

**Revenue:**
- Fare per ride: $1-3 (configurable via policy).
- At 80,000 daily riders and $2.00 fare: $160,000/day = $4,800,000/month.
- Net: $4,800,000 - $46,100 = $4,753,900/month profit.
- This makes metro profitable at high ridership, creating a strong incentive to build well-designed networks.

---

## Underground View/Layer System

### The Rendering Challenge

Rendering underground infrastructure presents a fundamental UX challenge: the player needs to see what is underground while maintaining spatial reference to what is on the surface. Without surface reference, the player cannot orient themselves or make meaningful placement decisions. But showing everything at once creates visual clutter that makes both surface and underground illegible.

Every city builder that has attempted underground rendering has taken a different approach, each with tradeoffs.

### Approach Analysis

#### CS1 X-Ray Toggle

Cities: Skylines 1 used a simple toggle: press Page Down to enter underground view. In this mode:
- Surface terrain becomes semi-transparent (ghosted blue-grey).
- Buildings are rendered as faint outlines at 15% opacity.
- Roads are rendered as semi-transparent lines.
- Underground pipes and tunnels are rendered at full opacity and color.
- The camera altitude does not change -- just the rendering mode.

**Pros:** Simple to understand. Clear spatial reference. Easy to implement.
**Cons:** Abrupt transition. Hard to see pipe-to-building relationships when buildings are ghosted. No depth perception for multi-layer underground.

#### CS2's No Underground View

CS2 eliminated the underground view entirely, showing utility information as surface overlays only.

**Pros:** No rendering complexity. Surface view is never disrupted.
**Cons:** No underground infrastructure to show. This approach is not viable for Megacity since the hybrid model requires underground trunk mains and metro tunnels.

#### Real Engineering Software (GIS/CAD)

Engineering applications use layer-based visibility: each infrastructure type is a separate layer that can be toggled on/off. Colors indicate type (blue = water, brown = sewer, red = power, etc.).

**Pros:** Maximum flexibility. Power users can show exactly what they need.
**Cons:** Too complex for a game interface. Players do not want to manage 15 layer toggles.

### Recommended Approach: Depth-Based Transparency with Layer Tabs

The recommended approach combines the simplicity of CS1's toggle with the layering capability needed for multi-depth underground infrastructure.

#### UI Structure

**Tab Bar at screen bottom (or sidebar):**
```
[Surface] [Shallow] [Medium] [Deep] [All Underground]
```

Each tab activates a depth-specific view:

**Surface (default):**
- Normal game rendering. No underground infrastructure visible.
- Overlay modes (P, O, T, etc.) work as currently implemented.
- Status icons show when buildings lack water/power (via `status_icons.rs`).

**Shallow (~0-5m depth):**
- Surface terrain rendered at 30% opacity (ghosted earth tone, not blue-grey).
- Surface roads rendered as white outlines (reference for navigation).
- Surface buildings rendered as footprint outlines at 20% opacity.
- Shallow infrastructure rendered at full opacity:
  - Water distribution mains (blue lines)
  - Sewer trunk lines (brown/green lines)
  - Storm drains (grey lines)
  - Cut-and-cover metro tunnels (dark outline with track markings)
  - Shallow power cables (red lines)
- Grid overlay showing elevation contours (helps plan gravity sewers).

**Medium (~5-20m depth):**
- Surface terrain at 15% opacity.
- Surface buildings as faint footprints.
- Shallow infrastructure at 30% opacity (visible for reference but not dominant).
- Medium infrastructure at full opacity:
  - Metro tunnels (bored)
  - Utility tunnels
  - Deep water trunk mains
  - Deep sewer interceptors

**Deep (>20m depth):**
- Surface terrain at 10% opacity.
- Only deep infrastructure visible:
  - Deep bored metro tunnels
  - Geological features (if implemented: bedrock, aquifers)
  - Deep utility tunnels

**All Underground:**
- Surface terrain at 20% opacity.
- All underground infrastructure visible, color-coded by type and alpha-coded by depth.
- Shallow = full opacity, medium = 70% opacity, deep = 50% opacity.
- This is the "overview" mode for seeing the complete underground picture.

#### Bevy Implementation

The rendering system needs a new resource to track the current view mode and a way to adjust material properties based on it.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewLayer {
    #[default]
    Surface,
    Shallow,
    Medium,
    Deep,
    AllUnderground,
}

#[derive(Resource, Default)]
pub struct UndergroundViewState {
    pub active_layer: ViewLayer,
}
```

**Terrain rendering changes:**

The existing `terrain_render.rs` uses `spawn_terrain_chunks()` to create chunk meshes with per-vertex colors. When underground view is active, the terrain material needs to be swapped to a semi-transparent version.

```rust
fn update_terrain_transparency(
    view_state: Res<UndergroundViewState>,
    mut terrain_materials: Query<&mut MeshMaterial3d<StandardMaterial>, With<TerrainChunk>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !view_state.is_changed() {
        return;
    }

    let alpha = match view_state.active_layer {
        ViewLayer::Surface => 1.0,
        ViewLayer::Shallow => 0.30,
        ViewLayer::Medium => 0.15,
        ViewLayer::Deep => 0.10,
        ViewLayer::AllUnderground => 0.20,
    };

    for material_handle in &mut terrain_materials {
        if let Some(material) = materials.get_mut(&material_handle.0) {
            material.base_color.set_alpha(alpha);
            material.alpha_mode = if alpha < 1.0 {
                AlphaMode::Blend
            } else {
                AlphaMode::Opaque
            };
        }
    }
}
```

**Building rendering changes:**

The existing `building_render.rs` manages building meshes. When underground view is active, buildings should be rendered as ghosted outlines.

```rust
fn update_building_transparency(
    view_state: Res<UndergroundViewState>,
    mut building_visibility: Query<&mut Visibility, With<BuildingMesh>>,
    mut building_materials: Query<&mut MeshMaterial3d<StandardMaterial>, With<BuildingMesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !view_state.is_changed() {
        return;
    }

    let alpha = match view_state.active_layer {
        ViewLayer::Surface => 1.0,
        ViewLayer::Shallow => 0.20,
        ViewLayer::Medium => 0.10,
        ViewLayer::Deep => 0.05,
        ViewLayer::AllUnderground => 0.15,
    };

    for material_handle in &mut building_materials {
        if let Some(material) = materials.get_mut(&material_handle.0) {
            if alpha < 0.05 {
                // Effectively invisible at deep layer
                material.base_color = Color::NONE;
            } else {
                material.base_color.set_alpha(alpha);
                material.alpha_mode = AlphaMode::Blend;
            }
        }
    }
}
```

**Underground infrastructure rendering:**

Underground infrastructure needs its own set of mesh entities, spawned when the player builds trunk mains, metro tunnels, etc.

```rust
#[derive(Component)]
struct UndergroundMesh {
    infrastructure_type: UndergroundType,
    depth_layer: UndergroundLayer,
}

#[derive(Debug, Clone, Copy)]
enum UndergroundType {
    WaterMain,
    SewerMain,
    StormDrain,
    PowerCable,
    MetroTunnel,
    MetroStation,
    UtilityTunnel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UndergroundLayer {
    Shallow,  // 0-5m
    Medium,   // 5-20m
    Deep,     // 20m+
}
```

Underground meshes are rendered with the following logic:
```rust
fn update_underground_mesh_visibility(
    view_state: Res<UndergroundViewState>,
    mut query: Query<(&UndergroundMesh, &mut Visibility, &mut MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !view_state.is_changed() {
        return;
    }

    for (underground, mut visibility, material_handle) in &mut query {
        let should_show = match view_state.active_layer {
            ViewLayer::Surface => false,
            ViewLayer::Shallow => underground.depth_layer == UndergroundLayer::Shallow,
            ViewLayer::Medium => {
                underground.depth_layer == UndergroundLayer::Medium
                || underground.depth_layer == UndergroundLayer::Shallow
            }
            ViewLayer::Deep => {
                underground.depth_layer == UndergroundLayer::Deep
                || underground.depth_layer == UndergroundLayer::Medium
            }
            ViewLayer::AllUnderground => true,
        };

        *visibility = if should_show { Visibility::Inherited } else { Visibility::Hidden };

        // Alpha based on depth relevance
        if should_show {
            if let Some(mat) = materials.get_mut(&material_handle.0) {
                let alpha = match (view_state.active_layer, underground.depth_layer) {
                    (ViewLayer::AllUnderground, UndergroundLayer::Shallow) => 1.0,
                    (ViewLayer::AllUnderground, UndergroundLayer::Medium) => 0.7,
                    (ViewLayer::AllUnderground, UndergroundLayer::Deep) => 0.5,
                    (layer, depth) if layer_matches_depth(layer, depth) => 1.0,
                    _ => 0.3,  // visible but not the focused layer
                };
                mat.base_color.set_alpha(alpha);
            }
        }
    }
}
```

#### Visual Language for Underground Infrastructure

Each infrastructure type has a distinct color and rendering style to make underground views immediately legible:

| Infrastructure | Color | Line Style | Rendering |
|---------------|-------|-----------|-----------|
| Water main (raw) | Dark blue | Solid thick | Cylinder mesh |
| Water main (treated) | Light blue | Solid thick | Cylinder mesh |
| Sewer trunk | Brown/olive | Solid thick | Cylinder mesh (larger) |
| Storm drain | Grey | Dashed | Cylinder mesh |
| Power cable | Red/orange | Solid thin | Cable mesh with glow |
| Metro tunnel | Dark grey | Solid, very thick | Rectangular cross-section mesh |
| Metro station | White | Filled rectangle | Platform mesh with walls |
| Utility tunnel | Purple | Solid thick | Rectangular cross-section mesh |

**Flow visualization (optional, toggleable):**
- Water mains: animated blue particles flowing in the pipe direction.
- Sewer: animated brown particles flowing downhill.
- Power cables: animated electrical spark effect.
- Metro: animated train markers moving along tunnel path.

This integrates with the existing `OverlayMode` system. New overlay modes:

```rust
enum OverlayMode {
    // Existing modes...
    None, Power, Water, Traffic, Pollution, LandValue, Education, Garbage, Noise, WaterPollution,
    // New underground-specific overlay modes:
    WaterPressure,        // heat map of water pressure (blue = high, red = low)
    SewerCapacity,        // heat map of sewer load vs capacity (green = ok, red = overloaded)
    StormwaterRisk,       // heat map of flood risk based on drainage capacity
    PowerLoad,            // heat map of power load vs substation capacity
    MetroRidership,       // heat map of metro station catchment and ridership
    UndergroundConflicts, // highlights cells where underground infrastructure overlaps at same depth
}
```

#### Underground Collision Detection

Multiple underground infrastructure systems must coexist without physical conflicts. The collision detection system prevents the player from placing two infrastructure elements at the same depth in the same cell.

```rust
struct UndergroundOccupancy {
    // For each cell, track what exists at each depth layer
    shallow: Vec<Option<UndergroundType>>,  // 256*256
    medium: Vec<Option<UndergroundType>>,
    deep: Vec<Option<UndergroundType>>,
}

impl UndergroundOccupancy {
    fn can_place(&self, x: usize, y: usize, layer: UndergroundLayer, infra_type: UndergroundType) -> bool {
        let idx = y * GRID_WIDTH + x;
        let existing = match layer {
            UndergroundLayer::Shallow => self.shallow[idx],
            UndergroundLayer::Medium => self.medium[idx],
            UndergroundLayer::Deep => self.deep[idx],
        };

        match existing {
            None => true,  // empty cell, can place
            Some(existing_type) => {
                // Some combinations are allowed (e.g., small pipes can share space)
                can_colocate(existing_type, infra_type)
            }
        }
    }
}

fn can_colocate(a: UndergroundType, b: UndergroundType) -> bool {
    // Small utility pipes can share space with each other
    // Metro tunnels cannot share space with anything
    // Utility tunnels contain multiple utilities and exclude individual pipes
    match (a, b) {
        (UndergroundType::MetroTunnel, _) | (_, UndergroundType::MetroTunnel) => false,
        (UndergroundType::MetroStation, _) | (_, UndergroundType::MetroStation) => false,
        (UndergroundType::UtilityTunnel, _) | (_, UndergroundType::UtilityTunnel) => false,
        // Small pipes can coexist
        (UndergroundType::WaterMain, UndergroundType::PowerCable) => true,
        (UndergroundType::PowerCable, UndergroundType::WaterMain) => true,
        // Same type cannot double-up
        (a, b) if a == b => false,
        // Default: allow coexistence
        _ => true,
    }
}
```

**UI feedback during placement:**
- When the player is drawing a trunk main or metro tunnel, cells that would conflict with existing underground infrastructure are highlighted in red.
- A tooltip explains the conflict: "Cannot place water main here: metro tunnel occupies this cell at shallow depth."
- The player can switch to a different depth layer to avoid the conflict (e.g., run the water main at medium depth to pass under a shallow metro tunnel).

---

## Utility Tunnels (Advanced)

### Concept: The "Underground Highway"

A utility tunnel (also called a "common utility duct," "utilidor," or "pipe gallery") is a large walkable or drive-through tunnel that contains multiple utility lines: water mains, sewer pipes, power cables, telecommunications cables, gas lines, and sometimes district heating pipes. Instead of burying each utility in its own trench (requiring separate excavation for installation and repair), all utilities share a single large tunnel with internal racks and mounts.

### Real-World Examples

**Helsinki (Finland):**
- One of the world's most extensive utility tunnel networks: over 200 km of tunnels.
- Contains water, sewer, power, telecom, and district heating.
- All tunnels are carved into bedrock at 20-30m depth.
- Maintenance vehicles can drive through the tunnels.
- No need to dig up streets for utility repairs -- ever.

**Tokyo (Japan):**
- The "Common Utility Duct" program began in the 1970s after earthquake damage to scattered utilities.
- 100+ km of shared tunnels under major roads.
- Seismically designed with flexible joints.
- Reduced earthquake-related utility damage by 90%.

**Singapore:**
- The Marina Bay district uses a comprehensive utility tunnel network.
- Includes automated waste collection (pneumatic tubes).
- Monitoring sensors throughout for leak detection.

**Montreal (Canada):**
- The "Utiliduc" network serves the downtown core.
- Originally built for Expo 67, expanded since.
- Includes steam heating, chilled water, and power.

### Game Mechanic: The Late-Game Investment

Utility tunnels are a premium late-game infrastructure option that the player unlocks after reaching a certain population or development level. They represent the highest tier of underground infrastructure planning.

#### Properties

```rust
struct UtilityTunnel {
    // Bezier curve routing (same as other underground infrastructure)
    id: TunnelId,
    start_node: TunnelNodeId,
    end_node: TunnelNodeId,
    p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2,
    depth_layer: UndergroundLayer,  // typically Medium
    width: TunnelWidth,
    construction_cost_per_cell: f64,   // $20,000/cell -- expensive
    monthly_maintenance_per_cell: f64, // $100/cell -- but lower than sum of individual utilities
    utilities_contained: Vec<UtilitySlot>,
    condition: f32,
    age_days: u32,
    has_vehicle_access: bool,       // wide enough for maintenance vehicles
    has_monitoring_sensors: bool,   // automated leak/fault detection
}

struct UtilitySlot {
    utility_type: UndergroundType,
    capacity: f32,
    occupied: bool,
    condition: f32,
}

enum TunnelWidth {
    Narrow,    // 2m wide, walkable, 3 utility slots
    Standard,  // 3m wide, small vehicle, 5 utility slots
    Wide,      // 5m wide, full vehicle access, 8 utility slots
    Mega,      // 8m wide, double-deck, 12 utility slots
}
```

#### Slot System

The utility tunnel has a fixed number of utility slots determined by its width. Each slot can hold one utility line:

| Slot Contents | Capacity Equivalent | Notes |
|--------------|---------------------|-------|
| Water Main | 1 medium trunk main | Accessible for inspection and repair |
| Sewer Main | 1 medium trunk main | Gravity slope still required within tunnel |
| Power Cable | 1 distribution cable | High-voltage requires separation from water |
| Telecom Bundle | Fiber optic + copper | Future-proofing for data infrastructure |
| District Heating | Hot water pipe | Integrates with `HeatingGrid` system |
| Gas Main | Natural gas distribution | Not yet implemented, future expansion |
| Chilled Water | Cooling distribution | Not yet implemented, future expansion |
| Pneumatic Waste | Automated waste collection | Not yet implemented, future expansion |
| Empty/Reserved | Available for future use | Can be filled later without excavation |

**Strategic value of empty slots:** One of the key advantages of utility tunnels is that the player can build the tunnel with empty slots and fill them later as the city grows. Adding a new utility to an existing tunnel costs only the pipe/cable material cost (not excavation). This is dramatically cheaper than trenching a new individual pipe.

#### Cost-Benefit Analysis

**Individual utility trenching (baseline):**
- Water main: $3,000/cell construction + $30/cell/month maintenance
- Sewer main: $3,500/cell construction + $35/cell/month maintenance
- Power cable: $2,000/cell construction + $20/cell/month maintenance
- Telecom: $1,000/cell construction + $10/cell/month maintenance
- Total (4 utilities): $9,500/cell construction + $95/cell/month maintenance
- Plus: road disruption for each separate excavation (4 disruptions)
- Plus: each utility has independent failure probability

**Utility tunnel (all 4 utilities):**
- Tunnel: $20,000/cell construction + $100/cell/month maintenance
- Utility installation: 4 * $500/cell = $2,000/cell (just the pipes/cables, no excavation)
- Total: $22,000/cell construction + $100/cell/month maintenance
- Plus: zero road disruption (tunnel is underground)
- Plus: single point of monitoring, faster repair, coordinated maintenance

**Break-even analysis:**
- Higher upfront cost ($22,000 vs $9,500 per cell -- 2.3x)
- Slightly higher monthly cost ($100 vs $95 per cell)
- But: zero disruption to roads during construction and maintenance
- But: 50% lower repair times (all utilities accessible in one tunnel)
- But: future utility additions cost only $500/cell instead of $3,000+
- But: reduced failure probability (protected environment, monitored)

The tunnel pays for itself when:
1. The corridor is under a high-traffic road (disruption cost is high).
2. The player anticipates needing to add utilities later (empty slots are valuable).
3. The corridor is in a dense urban area where trenching is extremely disruptive.
4. The corridor crosses an area where future development is planned (build once).

#### Maintenance Access

Utility tunnels have internal maintenance access, which changes the repair model:

**Individual buried pipes (conventional):**
- Detection: slow. Surface-level symptoms (wet spot, sinkhole) may take days to appear.
- Diagnosis: requires excavation or camera inspection.
- Repair: excavate -> repair -> backfill -> repave. 3-10 days. Road closed.
- Collateral damage: excavation may damage adjacent utilities.

**Utility tunnel:**
- Detection: fast. Monitoring sensors detect leaks, temperature changes, gas levels in real-time.
- Diagnosis: walk/drive through tunnel, visually inspect.
- Repair: access from inside tunnel. No excavation. 0.5-2 days. No road closure.
- Collateral damage: none. Each utility is on a separate rack.

**Game implementation of monitoring:**
```rust
struct TunnelMonitoring {
    // Continuous monitoring reduces failure consequences
    leak_detection: bool,       // detect water/sewer leaks within 1 hour
    thermal_detection: bool,    // detect power cable overheating
    gas_detection: bool,        // detect gas leaks (if gas utility present)
    automated_valves: bool,     // can isolate leaking sections automatically
    camera_coverage: bool,      // visual inspection without entering tunnel
    cost_per_cell: f64,         // $2,000 one-time installation
}
```

With monitoring active:
- Pipe bursts are detected 10x faster (1 hour vs 1 day).
- Affected area is isolated 5x faster (automated valves close within minutes).
- Repair time is 3x shorter (no excavation needed).
- Net effect: a pipe burst in a utility tunnel affects 5-10% as many citizens as the same burst in a conventional buried pipe.

#### Unlock Requirements

Utility tunnels should be a late-game feature to prevent the player from trivializing early infrastructure challenges:

```rust
fn can_build_utility_tunnel(unlock_state: &UnlockState, stats: &CityStats) -> bool {
    // Requires:
    // 1. Population > 50,000
    // 2. At least 3 different utility types already built (water, sewer, power)
    // 3. Development points invested in Infrastructure technology tree
    // 4. Treasury > $500,000 (to demonstrate financial capability)
    stats.population > 50_000
        && unlock_state.has_unlock("advanced_infrastructure")
        && stats.utility_types_built >= 3
}
```

This integrates with the existing `UnlockState` resource (`crates/simulation/src/unlocks.rs`).

---

## Implementation Architecture

### Design Principles

The underground infrastructure system must integrate cleanly with the existing Megacity codebase while maintaining the performance characteristics that allow the game to handle 1M+ citizens. The key principles are:

1. **Reuse existing patterns.** The `WorldGrid`, `RoadSegmentStore`, `CsrGraph`, `ServiceCoverageGrid`, and `SlowTickTimer` patterns are well-established and performant. Underground infrastructure should follow the same patterns rather than inventing new ones.

2. **Lazy recalculation.** Network state (pressure, flow, capacity utilization) should only be recalculated when infrastructure changes or on periodic intervals -- never every tick. The `SlowTickTimer` (every 100 ticks) is appropriate for most utility calculations.

3. **Grid-based simulation, entity-based infrastructure.** Individual trunk mains and metro tunnels are entities (for rendering, selection, and per-instance properties like age and condition). But the aggregate simulation state (pressure at each cell, sewer capacity at each cell) is stored in grid Resources for O(1) lookup.

4. **Separation of concerns.** Rendering, simulation, UI, and serialization are in separate crates. Underground infrastructure adds data structures to `simulation`, rendering to `rendering`, UI panels to `ui`, and save/load support to `save`.

### Grid Representation

#### Option A: Separate UndergroundGrid Resource

Create a new `UndergroundGrid` resource that runs alongside `WorldGrid`:

```rust
#[derive(Resource, Serialize, Deserialize)]
pub struct UndergroundGrid {
    pub width: usize,
    pub height: usize,

    // Per-cell state for each utility type
    pub water_pressure: Vec<f32>,           // 0.0-2.0, 65,536 entries
    pub water_flow: Vec<f32>,               // liters/second at this cell
    pub water_source: Vec<Option<Entity>>,  // which trunk main feeds this cell

    pub sewer_load: Vec<f32>,               // fraction of capacity used
    pub sewer_slope: Vec<f32>,              // local slope (for gravity flow validation)
    pub sewer_connected: Vec<bool>,         // can reach a treatment plant

    pub storm_capacity: Vec<f32>,           // liters/second drainage capacity
    pub storm_connected: Vec<bool>,         // has storm drain connection

    pub power_supply: Vec<f32>,             // MW available at this cell
    pub power_demand: Vec<f32>,             // MW demanded by buildings at this cell
    pub power_substation: Vec<Option<Entity>>, // which substation serves this cell

    // Underground occupancy (for collision detection)
    pub shallow_occupant: Vec<Option<UndergroundType>>,
    pub medium_occupant: Vec<Option<UndergroundType>>,
    pub deep_occupant: Vec<Option<UndergroundType>>,
}
```

**Memory footprint:** At 256x256 = 65,536 cells:
- `f32` arrays: 65,536 * 4 bytes = 256 KB each. With ~10 arrays = 2.5 MB.
- `bool` arrays: 65,536 * 1 byte = 64 KB each. With ~3 arrays = 192 KB.
- `Option<Entity>` arrays: 65,536 * 8 bytes = 512 KB each. With ~3 arrays = 1.5 MB.
- `Option<UndergroundType>` arrays: 65,536 * 2 bytes = 128 KB each. With 3 layers = 384 KB.
- **Total: ~4.6 MB.** Well within acceptable memory bounds.

**Pros:** Clean separation from surface grid. Can be loaded/unloaded independently. No changes to existing `WorldGrid` struct.
**Cons:** Requires cross-referencing with `WorldGrid` for elevation data, building locations, etc.

#### Option B: Extended Cell in WorldGrid

Add fields to the existing `Cell` struct:

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Cell {
    // Existing fields:
    pub elevation: f32,
    pub cell_type: CellType,
    pub zone: ZoneType,
    pub road_type: RoadType,
    pub building_id: Option<Entity>,
    pub has_power: bool,
    pub has_water: bool,

    // New underground fields:
    pub water_pressure: f32,
    pub sewer_connected: bool,
    pub storm_capacity: f32,
    pub power_load_fraction: f32,
}
```

**Pros:** All data in one place. No cross-referencing.
**Cons:** Increases `Cell` size from ~24 bytes to ~40 bytes. Total `WorldGrid` goes from ~1.6 MB to ~2.6 MB. More concerning: changes to `Cell` affect serialization format, requiring save file migration.

#### Recommendation: Option A (Separate UndergroundGrid)

Option A is better because:
1. It does not modify the existing `WorldGrid` or `Cell` structs, which are used throughout the codebase.
2. It can be added incrementally without touching existing systems.
3. The separate resource can have its own change detection (`is_changed()`), avoiding unnecessary recalculation when only surface elements change.
4. Save file format for `WorldGrid` is unchanged; underground data is a new section in the save file.
5. The 4.6 MB memory cost is negligible.

### Pipe/Tunnel Entity Architecture

#### Entity Components

Underground infrastructure elements (trunk mains, metro tunnels, utility tunnels) are represented as Bevy entities with components:

```rust
// === Water Infrastructure ===

#[derive(Component, Serialize, Deserialize)]
pub struct WaterTrunkMain {
    pub pipe_type: WaterPipeType,  // RawWater, TreatedWater
    pub material: PipeMaterial,
    pub diameter: PipeDiameter,    // Small, Medium, Large, Mega
    pub capacity_liters_per_second: f32,
    pub current_flow: f32,
    pub age_days: u32,
    pub condition: f32,
}

#[derive(Component, Serialize, Deserialize)]
pub struct WaterSource {
    pub source_type: WaterSourceType,
    pub capacity_liters_per_day: f32,
    pub raw_quality: f32,
    pub grid_x: usize,
    pub grid_y: usize,
}

#[derive(Component, Serialize, Deserialize)]
pub struct WaterTreatmentPlant {
    pub treatment_level: TreatmentLevel,
    pub capacity_liters_per_day: f32,
    pub current_throughput: f32,
    pub grid_x: usize,
    pub grid_y: usize,
}

// === Sewer Infrastructure ===

#[derive(Component, Serialize, Deserialize)]
pub struct SewerTrunkMain {
    pub sewer_type: SewerType,  // Sanitary, Storm, Combined
    pub diameter: SewerDiameter,
    pub capacity_liters_per_second: f32,
    pub current_flow: f32,
    pub slope: f32,
    pub flow_direction: FlowDirection,
    pub age_days: u32,
    pub condition: f32,
}

#[derive(Component, Serialize, Deserialize)]
pub struct LiftStation {
    pub pump_capacity: f32,
    pub lift_height: f32,
    pub power_consumption: f32,
    pub has_backup_generator: bool,
    pub grid_x: usize,
    pub grid_y: usize,
}

// === Power Infrastructure ===

#[derive(Component, Serialize, Deserialize)]
pub struct TransmissionLine {
    pub line_type: TransmissionLineType,  // Overhead, Underground
    pub capacity_mw: f32,
    pub current_load_mw: f32,
    pub voltage_kv: f32,
    pub condition: f32,
    pub age_days: u32,
}

#[derive(Component, Serialize, Deserialize)]
pub struct Substation {
    pub tier: SubstationTier,
    pub capacity_mw: f32,
    pub current_load_mw: f32,
    pub coverage_radius: u32,
    pub grid_x: usize,
    pub grid_y: usize,
}

// === Metro Infrastructure ===

#[derive(Component, Serialize, Deserialize)]
pub struct MetroTunnel {
    pub construction_method: TunnelMethod,  // CutAndCover, Bored
    pub depth_layer: UndergroundLayer,
    pub condition: f32,
    pub age_days: u32,
}

#[derive(Component, Serialize, Deserialize)]
pub struct MetroStation {
    pub station_type: StationType,
    pub platform_length: PlatformLength,
    pub entrance_cells: Vec<(usize, usize)>,
    pub depth_layer: UndergroundLayer,
    pub daily_ridership: u32,
    pub grid_x: usize,
    pub grid_y: usize,
}

#[derive(Component, Serialize, Deserialize)]
pub struct MetroLine {
    pub name: String,
    pub color: [f32; 4],
    pub headway_seconds: u32,
    pub train_count: u32,
    pub cars_per_train: u32,
}

// === Shared ===

/// All underground linear infrastructure shares this Bezier geometry component.
/// This mirrors the RoadSegment pattern from road_segments.rs.
#[derive(Component, Serialize, Deserialize)]
pub struct UndergroundSegment {
    pub start_node: UndergroundNodeId,
    pub end_node: UndergroundNodeId,
    pub p0: Vec2,
    pub p1: Vec2,
    pub p2: Vec2,
    pub p3: Vec2,
    pub depth_layer: UndergroundLayer,
    pub arc_length: f32,
    pub rasterized_cells: Vec<(usize, usize)>,
}
```

#### The UndergroundSegmentStore

Mirroring the `RoadSegmentStore` pattern, an `UndergroundSegmentStore` manages all underground infrastructure segments and nodes:

```rust
#[derive(Resource, Default, Serialize, Deserialize)]
pub struct UndergroundSegmentStore {
    pub nodes: Vec<UndergroundNode>,
    pub segments: Vec<UndergroundSegmentData>,
    next_node_id: u32,
    next_segment_id: u32,
}

pub struct UndergroundNode {
    pub id: UndergroundNodeId,
    pub position: Vec2,
    pub depth_layer: UndergroundLayer,
    pub connected_segments: Vec<UndergroundSegmentId>,
}

pub struct UndergroundSegmentData {
    pub id: UndergroundSegmentId,
    pub entity: Entity,  // the Bevy entity with the type-specific component
    pub start_node: UndergroundNodeId,
    pub end_node: UndergroundNodeId,
    pub p0: Vec2, pub p1: Vec2, pub p2: Vec2, pub p3: Vec2,
    pub segment_type: UndergroundType,
    pub depth_layer: UndergroundLayer,
    pub arc_length: f32,
    pub rasterized_cells: Vec<(usize, usize)>,
}
```

The `UndergroundSegmentStore` provides:
- `add_segment()`: creates a new underground segment, rasterizes it to the `UndergroundGrid`.
- `remove_segment()`: removes a segment and clears its footprint from the grid.
- `find_or_create_node()`: finds existing node at position or creates new one (same as `RoadSegmentStore`).
- `get_connected_network()`: returns all segments connected to a given node (for network analysis).

The rasterization process is identical to the road segment rasterization: sample points along the Bezier curve at intervals of `CELL_SIZE`, convert to grid coordinates, mark those cells in `UndergroundGrid` as occupied by the segment's type and depth layer.

### Network Simulation Systems

#### Water Network Simulation

The water network simulation replaces the existing `propagate_utilities()` BFS for water with a pressure-aware network model.

```rust
pub fn simulate_water_network(
    slow_timer: Res<SlowTickTimer>,
    mut underground: ResMut<UndergroundGrid>,
    grid: Res<WorldGrid>,
    segment_store: Res<UndergroundSegmentStore>,
    sources: Query<&WaterSource>,
    treatment_plants: Query<&WaterTreatmentPlant>,
    trunk_mains: Query<(&WaterTrunkMain, &UndergroundSegment)>,
    buildings: Query<&Building>,
    weather: Res<Weather>,
    clock: Res<GameClock>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // Phase 1: Calculate total demand
    let demand_per_cell = calculate_demand_grid(&grid, &buildings, &weather, &clock);

    // Phase 2: Calculate supply from sources
    let total_supply: f32 = sources.iter().map(|s| {
        s.capacity_liters_per_day / 86400.0 // convert to liters/second
        * weather_supply_modifier(&weather, &s.source_type)
    }).sum();

    // Phase 3: Calculate treatment capacity
    let total_treatment: f32 = treatment_plants.iter().map(|tp| {
        tp.capacity_liters_per_day / 86400.0
    }).sum();

    // Phase 4: Effective supply = min(raw supply, treatment capacity)
    let effective_supply = total_supply.min(total_treatment);

    // Phase 5: Propagate pressure through trunk mains
    // BFS from treatment plant outputs through trunk main network
    // Pressure decreases with elevation gain and friction loss
    underground.water_pressure.fill(0.0);

    for tp in &treatment_plants {
        propagate_water_pressure(
            &mut underground,
            &grid,
            &segment_store,
            tp.grid_x,
            tp.grid_y,
            1.0,  // full pressure at treatment plant output
            &demand_per_cell,
        );
    }

    // Phase 6: Update cell-level has_water based on pressure
    // This bridges to the existing WorldGrid.has_water flag
    // for compatibility with existing systems
    // (done in a separate bridging system below)
}
```

**Bridging system (compatibility with existing code):**

The existing codebase checks `cell.has_water` and `cell.has_power` in many places (happiness, abandonment, building spawning, etc.). Rather than rewriting all those systems, a bridging system updates the existing flags based on the new simulation:

```rust
pub fn bridge_underground_to_grid(
    underground: Res<UndergroundGrid>,
    mut grid: ResMut<WorldGrid>,
) {
    if !underground.is_changed() {
        return;
    }

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let idx = y * GRID_WIDTH + x;

            // Water: available if pressure > minimum threshold
            grid.get_mut(x, y).has_water = underground.water_pressure[idx] > 0.1;

            // Power: available if supply > 0 at this cell
            grid.get_mut(x, y).has_power = underground.power_supply[idx] > 0.0;
        }
    }
}
```

This bridging approach means:
- All existing systems that check `has_water` / `has_power` continue to work unchanged.
- The new underground systems provide the detailed simulation.
- The bridge converts detailed state (pressure, supply) into the binary flags that existing systems use.
- Gradual migration: over time, individual systems can be upgraded to read from `UndergroundGrid` directly for more nuanced behavior (e.g., fire response using actual pressure values instead of binary has_water).

#### Sewer Network Simulation

The sewer network is simulated as a directed acyclic graph (DAG) where flow follows gravity:

```rust
pub fn simulate_sewer_network(
    slow_timer: Res<SlowTickTimer>,
    mut underground: ResMut<UndergroundGrid>,
    grid: Res<WorldGrid>,
    segment_store: Res<UndergroundSegmentStore>,
    sewer_mains: Query<(&SewerTrunkMain, &UndergroundSegment)>,
    treatment_plants: Query<&WastewaterTreatmentPlant>,
    lift_stations: Query<(&LiftStation, &UndergroundSegment)>,
    buildings: Query<&Building>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // Phase 1: Calculate sewage generation per cell
    // sewage = water_consumption * 0.8
    let sewage_per_cell = calculate_sewage_grid(&grid, &buildings);

    // Phase 2: Trace sewage flow downstream through gravity network
    // Starting from each building, follow connected sewer mains downhill
    // Each segment accumulates flow from upstream segments
    let mut segment_flows: HashMap<UndergroundSegmentId, f32> = HashMap::new();

    for (sewer, segment) in &sewer_mains {
        // Calculate flow contribution from cells connected to this segment
        let local_load: f32 = segment.rasterized_cells.iter()
            .map(|(cx, cy)| sewage_per_cell[*cy * GRID_WIDTH + *cx])
            .sum();

        // Add upstream contributions (from segments whose end_node = this segment's start_node)
        let upstream_flow = get_upstream_flow(&segment_store, segment.start_node, &segment_flows);

        let total_flow = local_load + upstream_flow;
        segment_flows.insert(segment.id(), total_flow);

        // Update underground grid with load fraction
        let load_fraction = total_flow / sewer.capacity_liters_per_second;
        for (cx, cy) in &segment.rasterized_cells {
            underground.sewer_load[*cy * GRID_WIDTH + *cx] = load_fraction;
        }
    }

    // Phase 3: Check for overloaded segments
    for (sewer, segment) in &sewer_mains {
        if let Some(&flow) = segment_flows.get(&segment.id()) {
            if flow > sewer.capacity_liters_per_second {
                // Overflow event -- trigger sewage backup or CSO
                trigger_sewer_overflow(segment, flow, sewer.capacity_liters_per_second);
            }
        }
    }

    // Phase 4: Verify all sewage reaches a treatment plant
    underground.sewer_connected.fill(false);
    for tp in &treatment_plants {
        // BFS upstream from treatment plant through sewer network
        mark_sewer_connected(&mut underground, &segment_store, tp.grid_x, tp.grid_y);
    }
}
```

#### Power Network Simulation

The power network simulation uses a simplified power flow model:

```rust
pub fn simulate_power_network(
    slow_timer: Res<SlowTickTimer>,
    mut underground: ResMut<UndergroundGrid>,
    grid: Res<WorldGrid>,
    segment_store: Res<UndergroundSegmentStore>,
    power_plants: Query<&PowerPlant>,
    substations: Query<&Substation>,
    transmission_lines: Query<(&TransmissionLine, &UndergroundSegment)>,
    buildings: Query<&Building>,
    weather: Res<Weather>,
    clock: Res<GameClock>,
) {
    if !slow_timer.should_run() {
        return;
    }

    // Phase 1: Calculate total generation capacity
    let total_generation: f32 = power_plants.iter().map(|pp| {
        pp.current_output_mw(&weather, &clock)
    }).sum();

    // Phase 2: Calculate total demand
    let total_demand: f32 = calculate_power_demand(&grid, &buildings, &weather, &clock);

    // Phase 3: Check supply/demand balance
    let supply_ratio = total_generation / total_demand.max(0.001);
    // supply_ratio > 1.0: surplus, < 1.0: deficit

    // Phase 4: Distribute power through transmission lines to substations
    underground.power_supply.fill(0.0);
    underground.power_demand.fill(0.0);

    for substation in &substations {
        // Check if substation can reach a power plant through transmission lines
        let connected = can_reach_power_source(
            &segment_store,
            substation.grid_x,
            substation.grid_y,
            &power_plants,
        );

        if !connected {
            continue;  // substation is islanded -- no power
        }

        // Distribute power to cells within coverage radius
        let available_mw = (substation.capacity_mw * supply_ratio).min(substation.capacity_mw);
        let mut distributed: f32 = 0.0;

        // BFS from substation through roads (same pattern as existing propagate_utilities)
        propagate_power_from_substation(
            &mut underground,
            &grid,
            substation,
            available_mw,
            &mut distributed,
        );

        // Track actual load on substation
        // (update substation.current_load_mw via commands or mutable query)
    }

    // Phase 5: Check for overloaded substations and transmission lines
    // (triggers blackout cascade if overloaded)
}
```

### Performance Considerations

#### Recalculation Frequency

The underground infrastructure simulation does NOT need to run every tick. Network state changes slowly:
- **Water pressure:** changes when infrastructure is built/destroyed, or when demand changes significantly (time of day, seasonal). Recalculate on `SlowTickTimer` (every 100 ticks = ~10 seconds at 10Hz fixed update).
- **Sewer flow:** changes when infrastructure is built/destroyed, or during storm events. Recalculate on `SlowTickTimer`, plus immediate recalculation during storm weather events.
- **Power load:** changes with time of day (peak vs off-peak). Recalculate on `SlowTickTimer`.
- **Infrastructure condition:** degrades over real-time months. Calculate daily (check once per `GameClock.day` change, similar to `Weather::update_weather`).

```rust
pub fn should_recalculate_underground(
    slow_timer: &SlowTickTimer,
    underground: &UndergroundGrid,
    segment_store: &UndergroundSegmentStore,
) -> bool {
    // Recalculate if:
    // 1. Infrastructure changed (segment added/removed) -- check via is_changed()
    // 2. SlowTickTimer fired
    // 3. Weather event started/ended (immediate recalc for storm->sewer interaction)
    slow_timer.should_run() || segment_store.is_changed()
}
```

#### Simulation Complexity

**Water pressure propagation:** BFS from each treatment plant output through trunk main network. Number of trunk main segments is typically 50-200 for a large city. Each segment rasterizes to ~10-30 cells. Total BFS visits: ~2,000-6,000 cells. This is trivial (< 1ms).

**Sewer flow calculation:** Topological sort of sewer segment DAG, then accumulate flow downstream. Same complexity as water BFS. < 1ms.

**Power flow:** BFS from each substation through road network (reuses existing `propagate_utilities` BFS pattern). The existing BFS handles this efficiently. Each substation covers ~1,000-3,000 cells. With 5-20 substations, total visits: ~10,000-60,000. At < 1ms per substation, total: < 20ms. This is acceptable for a SlowTickTimer system.

**Stormwater flow:** Surface flow simulation during storm events. This is a cellular automaton over the full grid (65,536 cells) with 10 iterations. ~655,000 cell operations. At ~10ns per operation, this is ~7ms. Acceptable for storm events (which are infrequent).

**Total overhead:** < 30ms per SlowTickTimer tick, which fires every ~10 seconds of real time. Negligible impact on frame rate.

#### Memory Optimization

The `UndergroundGrid` uses `Vec<f32>` for pressure/flow arrays. If memory becomes a concern (unlikely at 256x256), these can be replaced with `Vec<u8>` or `Vec<u16>` with fixed-point encoding:

```rust
// Option: u16 fixed-point for pressure (0-65535 maps to 0.0-2.0)
pub fn pressure_to_u16(pressure: f32) -> u16 {
    (pressure.clamp(0.0, 2.0) * 32767.5) as u16
}
pub fn u16_to_pressure(encoded: u16) -> f32 {
    encoded as f32 / 32767.5
}
```

This halves the memory for pressure arrays (256 KB -> 128 KB) at the cost of reduced precision. Not necessary for 256x256 but relevant if grid sizes increase.

### Serialization

The `save` crate (`crates/save/src/serialization.rs`) handles game state serialization. Underground infrastructure adds three new serializable data groups:

```rust
#[derive(Serialize, Deserialize)]
pub struct UndergroundSaveData {
    pub segment_store: UndergroundSegmentStore,
    pub grid_state: UndergroundGridState,
    pub metro_lines: Vec<MetroLineData>,
}

#[derive(Serialize, Deserialize)]
pub struct UndergroundGridState {
    // Only need to serialize the infrastructure layout, not derived state
    // (pressure, flow, etc. are recalculated on load)
    pub shallow_occupants: Vec<Option<UndergroundType>>,
    pub medium_occupants: Vec<Option<UndergroundType>>,
    pub deep_occupants: Vec<Option<UndergroundType>>,
}

#[derive(Serialize, Deserialize)]
pub struct MetroLineData {
    pub name: String,
    pub color: [f32; 4],
    pub station_ids: Vec<u32>,
    pub tunnel_segment_ids: Vec<u32>,
    pub headway_seconds: u32,
    pub train_count: u32,
    pub cars_per_train: u32,
}
```

**Key serialization principle:** Only serialize the infrastructure layout and entity properties. Derived state (pressure maps, flow calculations, capacity utilization) is recalculated after loading. This keeps save files small and avoids version-sensitivity in the derived calculation algorithms.

**Save file size estimate:**
- `UndergroundSegmentStore`: ~200 segments * ~200 bytes = ~40 KB
- `UndergroundGridState`: 3 * 65,536 * 2 bytes = ~384 KB
- `MetroLineData`: ~5 lines * ~500 bytes = ~2.5 KB
- **Total: ~427 KB** additional save data. Acceptable.

**Post-load recalculation:**

After loading underground save data, the following systems run in sequence to rebuild derived state:

```rust
fn rebuild_underground_state_on_load(
    segment_store: Res<UndergroundSegmentStore>,
    mut underground: ResMut<UndergroundGrid>,
    grid: Res<WorldGrid>,
    // ... all query parameters for the simulation systems
) {
    // 1. Rasterize all segments to the underground grid
    for segment in &segment_store.segments {
        for (cx, cy) in &segment.rasterized_cells {
            let idx = *cy * GRID_WIDTH + *cx;
            match segment.depth_layer {
                UndergroundLayer::Shallow => {
                    underground.shallow_occupant[idx] = Some(segment.segment_type);
                }
                UndergroundLayer::Medium => {
                    underground.medium_occupant[idx] = Some(segment.segment_type);
                }
                UndergroundLayer::Deep => {
                    underground.deep_occupant[idx] = Some(segment.segment_type);
                }
            }
        }
    }

    // 2. Run all network simulations once to populate derived state
    simulate_water_network(/* params */);
    simulate_sewer_network(/* params */);
    simulate_power_network(/* params */);

    // 3. Bridge to WorldGrid flags
    bridge_underground_to_grid(/* params */);
}
```

### System Registration

All underground infrastructure systems are registered in the `SimulationPlugin::build()` method, following the existing patterns.

The system schedule ordering ensures proper data dependencies:

```rust
// In SimulationPlugin::build():

// Phase 1: Simulate underground networks (on SlowTickTimer)
.add_systems(
    FixedUpdate,
    (
        underground::simulate_water_network,
        underground::simulate_sewer_network,
        underground::simulate_power_network,
    )
        .after(weather::update_weather)   // weather affects supply/demand
        .after(buildings::building_spawner) // new buildings affect demand
        .run_if(underground_should_recalculate),
)

// Phase 2: Bridge underground state to grid flags (immediately after simulation)
.add_systems(
    FixedUpdate,
    underground::bridge_underground_to_grid
        .after(underground::simulate_power_network),
)

// Phase 3: Existing utility-dependent systems run after bridge
// (happiness, abandonment, etc. -- already scheduled after propagate_utilities)
// The bridge replaces propagate_utilities, so the ordering is preserved.

// Phase 4: Stormwater simulation (only during rain/storm events)
.add_systems(
    FixedUpdate,
    underground::simulate_stormwater
        .after(weather::update_weather)
        .run_if(is_raining),
)

// Phase 5: Infrastructure aging (daily)
.add_systems(
    FixedUpdate,
    underground::age_infrastructure
        .run_if(day_changed),
)

// Phase 6: Failure events (random, on SlowTickTimer)
.add_systems(
    FixedUpdate,
    (
        underground::check_pipe_bursts,
        underground::check_blackout_triggers,
        underground::process_sewer_overflows,
    )
        .after(underground::simulate_water_network),
)
```

**Run condition helpers:**

```rust
fn underground_should_recalculate(
    slow_timer: Res<SlowTickTimer>,
    segments: Res<UndergroundSegmentStore>,
) -> bool {
    slow_timer.should_run() || segments.is_changed()
}

fn is_raining(weather: Res<Weather>) -> bool {
    matches!(weather.current_event, WeatherEvent::Rain | WeatherEvent::Storm)
}

fn day_changed(clock: Res<GameClock>) -> bool {
    clock.is_changed() && clock.hour == 0
}
```

### Migration Path from Current Utilities System

The current `propagate_utilities()` system in `crates/simulation/src/utilities.rs` uses a simple BFS flood-fill from `UtilitySource` entities. The migration to the underground infrastructure system should be gradual:

**Phase 1: Add UndergroundGrid alongside existing system.**
- The `UndergroundGrid` resource is added.
- The `bridge_underground_to_grid` system runs and writes to the same `WorldGrid.has_water` and `WorldGrid.has_power` fields.
- The old `propagate_utilities()` still runs but is overwritten by the bridge.
- Both systems coexist, ensuring backward compatibility.

**Phase 2: Replace propagation with underground simulation.**
- `propagate_utilities()` is removed.
- `bridge_underground_to_grid()` is the sole source of `has_water` / `has_power` values.
- Existing `UtilitySource` entities are migrated to `WaterSource` + `WaterTreatmentPlant` + `Substation` entities.
- The `UtilityType` enum is deprecated in favor of the more specific component types.

**Phase 3: Upgrade dependent systems.**
- `fire.rs`: instead of checking `has_water`, check `water_pressure > 0.3` for effective fire response.
- `happiness.rs`: instead of binary `has_water`/`has_power`, use graduated pressure/supply for nuanced happiness effects.
- `abandonment.rs`: instead of binary check, use duration-of-outage tracking.
- `health.rs`: water quality from treatment level affects health outcomes.

Each phase can be shipped independently, allowing iterative development and testing.

### Rendering Integration

Underground infrastructure meshes are managed by the `rendering` crate. The rendering follows the same pattern as road segment meshes (`road_render::sync_road_segment_meshes`):

```rust
// In rendering/src/underground_render.rs:

pub fn sync_underground_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    segment_store: Res<UndergroundSegmentStore>,
    view_state: Res<UndergroundViewState>,
    existing_meshes: Query<(Entity, &UndergroundMeshMarker)>,
) {
    if !segment_store.is_changed() && !view_state.is_changed() {
        return;
    }

    // Only spawn meshes when underground view is active
    if view_state.active_layer == ViewLayer::Surface {
        // Hide all underground meshes
        for (entity, _) in &existing_meshes {
            commands.entity(entity).insert(Visibility::Hidden);
        }
        return;
    }

    // For each segment in the store, ensure a mesh entity exists
    // and update its visibility based on depth layer
    for segment_data in &segment_store.segments {
        let should_show = should_show_at_layer(
            view_state.active_layer,
            segment_data.depth_layer,
        );

        if should_show {
            // Generate or update pipe/tunnel mesh along Bezier curve
            let mesh = generate_pipe_mesh(
                segment_data,
                &segment_store,
            );
            // Spawn or update entity with the mesh
            // ...
        }
    }
}

fn generate_pipe_mesh(
    segment: &UndergroundSegmentData,
    store: &UndergroundSegmentStore,
) -> Mesh {
    // Sample points along Bezier curve
    let points: Vec<Vec3> = (0..=32)
        .map(|i| {
            let t = i as f32 / 32.0;
            let p2d = evaluate_bezier(segment.p0, segment.p1, segment.p2, segment.p3, t);
            // Y coordinate is depth-based (negative, below surface)
            let depth_y = match segment.depth_layer {
                UndergroundLayer::Shallow => -2.0,
                UndergroundLayer::Medium => -8.0,
                UndergroundLayer::Deep => -20.0,
            };
            Vec3::new(p2d.x, depth_y, p2d.y)  // note: Bevy Y-up coordinate system
        })
        .collect();

    // Generate tube/box mesh along the path
    let radius = match segment.segment_type {
        UndergroundType::WaterMain => 1.5,
        UndergroundType::SewerMain => 2.0,
        UndergroundType::StormDrain => 1.8,
        UndergroundType::PowerCable => 0.8,
        UndergroundType::MetroTunnel => 4.0,
        UndergroundType::UtilityTunnel => 3.5,
        _ => 1.0,
    };

    generate_tube_along_path(&points, radius, 8)  // 8 radial segments
}
```

The pipe mesh generation creates a cylinder or box mesh that follows the Bezier curve, similar to how road meshes are generated but rendered below the terrain surface.

### New Module Structure

The underground infrastructure system adds the following modules:

```
crates/simulation/src/
    underground/
        mod.rs              -- Module root, exports all types
        water_network.rs    -- Water supply simulation
        sewer_network.rs    -- Sewage/wastewater simulation
        stormwater.rs       -- Stormwater drainage and flooding
        power_grid.rs       -- Power generation/distribution simulation
        metro.rs            -- Metro/subway simulation
        infrastructure.rs   -- Shared types (UndergroundGrid, segments, nodes)
        aging.rs            -- Infrastructure condition degradation
        failures.rs         -- Pipe bursts, blackouts, overflow events

crates/rendering/src/
    underground_render.rs   -- Mesh generation for underground infrastructure
    underground_view.rs     -- View layer switching, transparency management

crates/ui/src/
    underground_panel.rs    -- UI panels for underground info, construction
    metro_panel.rs          -- Metro line management UI
```

Each module follows the existing pattern: types defined at top, systems as `pub fn` taking Bevy system parameters, tests in `#[cfg(test)] mod tests` at bottom.

---

## Summary: Implementation Priority

The underground infrastructure system is large. Here is the recommended implementation order, prioritized by gameplay impact and dependency structure:

### Phase 1: Foundation (Weeks 1-3)
1. `UndergroundGrid` resource and `UndergroundSegmentStore` -- the data structures.
2. `bridge_underground_to_grid` -- compatibility bridge so existing systems keep working.
3. `UndergroundViewState` and depth-layer rendering -- visual foundation.
4. Basic underground segment drawing UI (reuse road drawing UI with depth selector).

### Phase 2: Water Supply (Weeks 3-5)
5. Water source types (river intake, groundwater well, desalination).
6. Water treatment plants (basic/standard/advanced).
7. Water trunk main drawing and pressure simulation.
8. Auto-distribution along roads (BFS from trunk main connection points).
9. Pressure-based effects (fire response, high-rise water, happiness).

### Phase 3: Sewage (Weeks 5-7)
10. Sewer trunk main drawing with gravity slope validation.
11. Slope visualization during drawing (green/yellow/red).
12. Wastewater treatment plants.
13. Lift/pump stations.
14. Combined vs separate sewer systems.
15. CSO overflow events during storms.

### Phase 4: Stormwater (Weeks 7-8)
16. Impervious surface tracking.
17. Storm drain network.
18. Retention/detention facilities.
19. Green infrastructure options.
20. Flood risk calculation.

### Phase 5: Power Grid Upgrade (Weeks 8-10)
21. Power plant output characteristics (capacity, ramp rate, fuel type).
22. Transmission lines (overhead/underground).
23. Substations with capacity limits.
24. Grid topology analysis (radial/ring/mesh).
25. Blackout cascade simulation.

### Phase 6: Metro (Weeks 10-14)
26. Tunnel drawing (cut-and-cover and bored).
27. Station placement and design.
28. Metro line definition and rolling stock.
29. Ridership estimation.
30. Construction phases (visual progress).
31. Integration with surface transit.

### Phase 7: Advanced (Weeks 14-16)
32. Utility tunnels.
33. Infrastructure aging and maintenance.
34. Failure events (pipe bursts, contamination).
35. Underground overlay modes (pressure, capacity, risk).
36. Metro line profitability and expansion planning.

Total estimated implementation time: 14-16 weeks for a single developer, or 7-8 weeks for two developers working in parallel (one on simulation, one on rendering/UI).

This is a major feature set that transforms Megacity from a surface-only city builder into a true multi-layer infrastructure simulation. The hybrid model for utilities, gravity-based sewer mechanics, pressure-based water, and cascade-capable power grid create the kind of emergent, interconnected systems that city builder enthusiasts love.















