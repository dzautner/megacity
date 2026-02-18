# Cities: Skylines Deep Mechanics Analysis

## Purpose

This document is a detailed technical reference for implementing city-builder systems in Megacity.
It covers Cities: Skylines 1 (CS1, 2015) and Cities: Skylines 2 (CS2, 2023) internals,
including specific numbers, formulas, thresholds, simulation quirks, DLC systems,
modding ecosystem insights, and community feedback on what worked and what failed.

---

## Table of Contents

1. [Zoning and Building Growth](#1-zoning-and-building-growth)
2. [Road System and Traffic](#2-road-system-and-traffic)
3. [Citizen Simulation (Agents / Cims)](#3-citizen-simulation)
4. [Economy and Budget](#4-economy-and-budget)
5. [Services and Coverage](#5-services-and-coverage)
6. [Land Value and Desirability](#6-land-value-and-desirability)
7. [Education System](#7-education-system)
8. [Healthcare and Deathcare](#8-healthcare-and-deathcare)
9. [Pollution (Ground, Water, Noise)](#9-pollution)
10. [Water and Electricity Infrastructure](#10-water-and-electricity-infrastructure)
11. [Public Transportation](#11-public-transportation)
12. [Districts and Policies](#12-districts-and-policies)
13. [Milestones and Unlocks](#13-milestones-and-unlocks)
14. [CS1 DLC Systems (All DLCs)](#14-cs1-dlc-systems)
15. [CS2 Architecture Changes](#15-cs2-architecture-changes)
16. [CS2 Failures and Community Criticism](#16-cs2-failures)
17. [Modding Ecosystem Analysis](#17-modding-ecosystem)
18. [Implementation Recommendations for Megacity](#18-implementation-recommendations)

---

## 1. Zoning and Building Growth

### 1.1 Zone Placement Mechanics (CS1)

Zones in CS1 are painted on cells adjacent to roads. The grid is global (not per-road) with each
cell being 8m x 8m. When a road is placed, the game creates "frontage" -- cells along the road
that are eligible for zoning. Zoning extends up to 4 cells deep perpendicular to the road.

Key mechanics:

- **Grid alignment**: The zone grid aligns to road direction. Curved roads create irregular zone
  shapes. The grid tries to maintain perpendicular alignment to the nearest road segment.
- **Frontage requirement**: A building plot MUST have at least one cell touching a road on its
  front face. Buildings without road access will not grow.
- **Plot sizes**: Growable buildings occupy rectangular plots. Residential and commercial use
  plots of 1x1 up to 4x4 cells. Industrial and office use 1x1 up to 4x4 as well. The game
  selects building prefabs that fit available contiguous zoned area.
- **Corner lots**: The game has special corner building prefabs that can occupy L-shaped lots
  where two roads meet. These are relatively rare in the vanilla asset pool.

### 1.2 Zone Types

CS1 has four base zone types, each with a density sub-type:

| Zone | Low Density | High Density | Notes |
|------|------------|--------------|-------|
| Residential | Single-family homes, 1-5 households | Apartments, 5-50+ households | HD unlocked at ~7,000 pop |
| Commercial | Small shops, 1-5 workers | Large stores/malls, 5-20 workers | HD unlocked at ~7,000 pop |
| Industrial | Warehouses/factories, 5-20 workers | N/A (no HD variant) | Specializations instead |
| Office | N/A (no LD variant) | Office towers, 5-50 workers | Unlocked at ~7,500 pop |

In CS2, the distinction changed: instead of LD/HD being separate zone types, there is a
"density" slider per zone type. Mixed-use zoning was also added (commercial ground floor +
residential above), a feature the CS1 community had begged for since launch.

### 1.3 Building Leveling System (CS1)

Buildings in CS1 "level up" through 5 levels. Each level replaces the building model with a
larger/nicer one and changes its stats. The leveling is driven by a scoring system.

**Level-up requirements** (all conditions must be met simultaneously):

**Residential:**
- Level 1 -> 2: Land value >= 14, requires water + electricity
- Level 2 -> 3: Land value >= 30, requires 1 education service (elementary)
- Level 3 -> 4: Land value >= 50, requires 2 education services, parks nearby
- Level 4 -> 5: Land value >= 70, requires all 3 education services, multiple parks,
  no pollution, fire/police/health coverage

**Commercial:**
- Level 1 -> 2: Land value >= 14, requires water + electricity, some customers
- Level 2 -> 3: Land value >= 36, requires fire coverage, adequate goods supply
- Level 3 -> 4: (only for HD commercial) Land value >= 60, education level of workers

**Industrial:**
- Level 1 -> 2: Requires water + electricity, workers
- Level 2 -> 3: Requires fire coverage, adequate worker education (but NOT too high --
  over-educated workers make industry UNHAPPY, a notorious CS1 mechanic)

**Office:**
- Levels 1-3: Based on land value, worker education, and service coverage
- Office specifically WANTS highly educated workers (inverse of industrial)

**The over-education problem**: One of CS1's most discussed mechanics. Industrial buildings
want uneducated workers. If you build too many schools, your industry becomes unhappy because
all citizens get educated and don't want to work in factories. This was somewhat realistic but
extremely frustrating in gameplay. The "Schools Out" policy could partially address this.
CS2 tried to fix this with a job-market system where educated citizens could still work
lower-tier jobs but at reduced happiness.

### 1.4 Building Level Stats (CS1)

Approximate household/worker counts by level:

**Low-density Residential:**
| Level | Households | Land Value Required | Visual |
|-------|-----------|-------------------|--------|
| 1 | 1-2 | 0 | Small house |
| 2 | 2-3 | 14 | Nicer house |
| 3 | 3-4 | 30 | Large house |
| 4 | 4-5 | 50 | Very nice house |
| 5 | 5-6 | 70 | Mansion/estate |

**High-density Residential:**
| Level | Households | Land Value Required | Visual |
|-------|-----------|-------------------|--------|
| 1 | 4-8 | 0 | Small apartment |
| 2 | 8-16 | 14 | Medium apartment |
| 3 | 16-30 | 30 | Large apartment |
| 4 | 30-50 | 50 | Tower |
| 5 | 50-80 | 70 | Luxury tower |

### 1.5 Abandonment and Demolition

Buildings can become "abandoned" when conditions deteriorate:
- No water/power for extended time (~2 game weeks)
- No road access (road deleted)
- Too much pollution (ground or noise)
- Crime too high (>50% crime rate in area)
- Dead bodies not collected (deathcare failure)
- No customers (commercial, if citizens don't visit)
- No goods (commercial, if no industry/imports to supply them)
- No workers (if unemployment is 0% and building can't fill positions)
- Taxes too high (>12% starts causing complaints, >15% causes abandonment)
- Leveling down: If conditions drop below current level requirements

Abandoned buildings:
- Stop functioning (no tax revenue, no service)
- Generate a small negative land value effect on neighbors
- Eventually get demolished automatically (after ~4 game weeks) or can be manually bulldozed
- The auto-demolish creates a "rubble" period before the cell becomes available again

### 1.6 CS2 Zoning Changes

CS2 made several significant changes:

- **Lot system**: Instead of grid-snapped cells, CS2 uses a "lot" system where buildings
  create irregular lot boundaries. This allows more organic-looking neighborhoods.
- **Signature buildings**: Unique growable buildings that can appear once per city and occupy
  special lots.
- **Mixed-use zones**: Ground-floor commercial with residential above. A single building
  serves both zone functions.
- **Rent-based system**: Buildings don't "level up" in the same way. Instead, rent determines
  building viability. If rent is too high relative to income, tenants leave. If land value
  supports it, buildings upgrade. This system was deeply broken at CS2 launch (see section 16).
- **Building upgrades**: Instead of wholesale replacement, buildings can add extensions,
  additional floors, etc.

---

## 2. Road System and Traffic

### 2.1 Road Types and Hierarchy (CS1)

CS1 ships with a deep road hierarchy. Each road type has specific lane configurations, speed
limits, and zoning eligibility. Roads are the backbone of the simulation because ALL agent
pathfinding (citizens, service vehicles, goods trucks) routes over the road network graph.

**Base road types and stats:**

| Road Type | Lanes | Speed (km/h) | Zoning | Width (cells) | Cost/cell |
|-----------|-------|---------------|--------|---------------|-----------|
| Two-lane road | 1+1 | 40 | Yes | 2 | 10 |
| Four-lane road | 2+2 | 40 | Yes | 3 | 16 |
| Six-lane road | 3+3 | 40 | Yes | 4 | 24 |
| Two-lane highway | 1+1 | 100 | No | 2 | 20 |
| Four-lane highway | 2+2 | 100 | No | 3 | 30 |
| Six-lane highway | 3+3 | 100 | No | 4 | 40 |
| One-way (2-lane) | 2 | 40 | Yes | 2 | 12 |
| One-way (4-lane) | 4 | 40 | Yes | 3 | 20 |
| One-way (6-lane) | 6 | 40 | No | 4 | 28 |

Additional variants include:
- **Decorated roads**: Same specs but with trees, increased land value (+6-10 radius effect)
- **Gravel roads**: 2-lane, 30 km/h, cheapest option, generates noise
- **Highway on-ramps**: Single-lane connectors for highway interchanges
- **Asymmetric roads**: 1+2 or 2+3 lane configurations (added in DLC/free patches)

### 2.2 Intersections and Nodes

Roads in CS1 are composed of **segments** (straight or curved pieces) connected by **nodes**
(intersection points). The game's internal representation is a graph where:

- Each **node** stores: position, elevation, road type, traffic light state, connected segments
- Each **segment** stores: start/end node, lanes, speed, direction, curve control points
- Segments use Bezier curves internally (cubic Bezier with 2 control points)

**Intersection rules:**
- Up to 8 segments can connect at a single node (though >4 causes problems)
- Traffic lights are auto-generated when 2+ segments of "major" road types connect
- Stop signs are placed on minor roads connecting to major roads
- The game classifies roads as "major" or "minor" based on lane count for priority
- T-intersections with a 2-lane road meeting a 6-lane road: the 2-lane gets a stop sign

**Traffic light timing:**
- Default cycle: 30 seconds total (red+green+amber per phase)
- The game divides the cycle into phases based on intersection geometry
- A 4-way intersection gets 4 phases (one per approach)
- Right-turn-on-red is NOT simulated in vanilla CS1 (this is a major TM:PE feature)
- No dedicated turn phases in vanilla -- this causes massive left-turn backup issues

### 2.3 Pathfinding Algorithm (CS1)

CS1 uses a modified A* pathfinding algorithm for all agents. This is one of the most
performance-critical systems and the source of many gameplay issues.

**How it works:**

1. Agent needs to travel from building A to building B
2. The pathfinder calculates a route over the road/rail/path network graph
3. Route is computed ONCE at trip start and cached on the agent
4. Agent follows the cached path -- it does NOT recalculate mid-trip
5. If a road is deleted while an agent is on it, the agent "despawns" (poof)

**Cost function for pathfinding:**
The A* cost function considers multiple factors:
- **Distance**: Base cost proportional to segment length
- **Speed**: Adjusted for road speed limit (faster roads are "cheaper")
- **Traffic**: Current congestion level adds cost (but see the lane problem below)
- **Turns**: Left turns have a small cost penalty
- **Traffic lights**: Add a time penalty (~5-15 seconds of "cost")
- **Road type transitions**: Changing road type adds a small penalty
- **Tolls**: If toll booths are present (After Dark DLC), adds the toll cost

**The critical flaw -- segment-level traffic, not lane-level:**
CS1 evaluates traffic at the SEGMENT level, not the LANE level. This means:
- A 6-lane road with traffic in lane 1 reports "congested" for ALL 6 lanes
- Agents avoid the "congested" road even though 5 lanes are empty
- This creates oscillating behavior: all agents pick one route, it congests, they all
  switch to another, THAT congests, repeat
- The result is the characteristic CS1 traffic pattern where agents use only 1 lane of a
  multi-lane road

### 2.4 The Lane Mathematics Problem

This is the single most infamous issue in Cities: Skylines. Citizens use only ONE lane of
multi-lane roads, creating absurd single-file traffic on 6-lane highways.

**Root cause analysis:**

The lane selection algorithm works as follows:
1. At each intersection, the agent must choose which lane to be in for the NEXT segment
2. Lane choice is determined by the agent's NEXT TURN after the upcoming segment
3. If the agent will turn right 3 segments ahead, it gets in the rightmost lane NOW
4. If turning left, it gets in the leftmost lane NOW
5. If going straight, it picks the lane that corresponds to the "straight" lanes

**The degenerate case:**
On a long straight road with no turns, ALL agents pick the same "default straight" lane,
which in CS1 is typically lane 0 (rightmost/outermost). Nobody spreads across lanes because
the algorithm only considers "which lane do I need for my next turn" not "which lane is
emptiest."

**Why Colossal Order couldn't easily fix this:**
- The lane system is tightly coupled to intersection behavior
- Changing lane selection to consider current lane occupancy would require agents to
  re-evaluate lanes continuously (expensive with 65k agents)
- The "pocket car" system (see below) means agents materialize at road edges, always
  starting in lane 0
- A proper fix would require a complete rewrite of the traffic AI

**TM:PE's fix:**
Traffic Manager: President Edition (the most essential CS1 mod) added:
- "Dynamic Lane Selection" (DLS): Agents consider current lane traffic density
- Lane connector tool: Players manually assign which lanes connect through intersections
- Speed limits per lane: Slow right lanes, fast left lanes
- Ban specific vehicle types from lanes
- Result: Traffic flow improvement of 40-60% on typical road networks

### 2.5 Vehicle Behavior and Pocket Cars

**Pocket cars** is the community term for CS1's approach to vehicle spawning:
- Citizens do NOT own persistent vehicle entities
- When a citizen decides to drive, a car entity is created at the building's road connection
- When the citizen arrives, the car entity is DESTROYED
- Cars literally appear and disappear at buildings
- This is computationally efficient but visually breaks immersion

**Vehicle stats:**
- Cars: Speed varies 20-100 km/h, acceleration ~3 m/s^2
- Trucks (goods/industry): Speed 20-80 km/h, slower acceleration
- Service vehicles: Speed varies by type (ambulance 80 km/h, fire truck 60 km/h)
- Buses: 40 km/h max on city roads
- All vehicles use the same basic steering model (simplified kinematic bicycle model)

**Despawn mechanic:**
- CS1 has a controversial "despawn" feature: if a vehicle is stuck for too long (~3 minutes
  real time, depending on settings), it simply vanishes
- This masks traffic problems -- gridlock doesn't persist because stuck vehicles delete
- TM:PE lets players disable despawning, which reveals the TRUE horror of traffic flow
- With despawn off, a badly designed interchange can gridlock an entire city permanently

### 2.6 CS2 Traffic Changes

CS2 reworked traffic substantially but introduced new problems:

- **Lane system rewrite**: CS2 actually has proper lane selection with vehicles spreading
  across available lanes. This was the #1 improvement over CS1.
- **Persistent vehicles**: No more pocket cars. Citizens own vehicles that park at their
  homes and workplaces. Parking lots and garages are needed.
- **Parking simulation**: Agents drive to destination, then search for parking. If no
  parking available, they circle. This created massive traffic in dense areas at launch.
- **Pathfinding updates**: CS2 can recalculate paths mid-trip in some cases
- **Performance disaster**: The improved simulation was massively more expensive. CS2
  cities of 50,000 ran at 15 FPS on high-end hardware at launch. CS1 could handle
  200,000+ at the same framerate.
- **Road maintenance**: Roads now degrade over time and need maintenance crews. A new cost
  and logistics layer.

### 2.7 Roundabouts

CS1 has no built-in roundabout tool. Players must manually construct them using one-way
curved road segments. This is extremely tedious but roundabouts are by far the most
efficient intersection type because:

- No traffic lights (yield-on-entry)
- Continuous flow for all directions
- Eliminates left-turn conflicts
- A single 2-lane roundabout outperforms a 6-lane signalized intersection

Community knowledge on optimal roundabout sizes:
- **Small** (diameter ~30m / ~4 cells): Handles low traffic, good for residential
- **Medium** (diameter ~50m / ~6 cells): Handles medium traffic, most versatile
- **Large** (diameter ~80m+ / ~10 cells): Handles highway-level traffic
- Rule of thumb: entering roads should be 1 lane less than the roundabout

CS2 added a native roundabout tool but it was initially buggy -- generated roundabouts
often had incorrect lane connections, creating gridlock at the entry/exit points.

---

## 3. Citizen Simulation (Agents / Cims)

### 3.1 Agent Limits and Lifecycle (CS1)

CS1 simulates individual citizens (called "cims") as agents with lifecycle state machines.
The hard engine limit is **65,536 citizen agents** and **16,384 vehicle entities** active
simultaneously.

**Important distinction**: The game's DISPLAYED population can exceed 65k because:
- Not all "residents" are simulated as active agents
- Buildings contain "virtual citizens" who exist only as numbers
- Active agents represent a sample of the total population
- At ~100,000 displayed pop, roughly 40% of citizens are fully simulated
- At ~500,000 displayed pop, roughly 10% are fully simulated
- The ratio is managed dynamically based on performance

**Agent states:**
Each citizen cycles through a state machine:
1. **At Home** -> decides to go to work, shop, or leisure
2. **Moving** -> pathfinding to destination, on road/transit/walking
3. **At Work** -> stays for a work period (~8 game hours)
4. **At Shop** -> stays briefly (~1 game hour), then returns home
5. **At Leisure** -> visits park/entertainment (~2 game hours)
6. **Sick** -> needs healthcare
7. **Dead** -> needs deathcare pickup (yes, this is an agent state)

### 3.2 Citizen Demographics

Citizens have persistent properties:
- **Age**: 0-360 (game units, not years). Mapped to lifecycle stages:
  - Child: 0-14 (attends elementary school)
  - Teen: 15-44 (attends high school)
  - Young Adult: 45-89 (attends university or works)
  - Adult: 90-179 (works)
  - Senior: 180-360 (retired, may still work part-time)
  - Death: Age varies, base ~300, affected by healthcare access
- **Education level**: Uneducated, Elementary, High School, University
- **Wealth**: Low, Medium, High (determines which commercial they visit, land value preference)
- **Health**: 0-100, affected by pollution, healthcare access, age
- **Happiness**: 0-100, composite of many factors (see section 6)
- **Family unit**: Citizens form households. 1 household = 1 residential unit demand

### 3.3 Daily Routine Simulation

The game runs a day/night cycle where citizens follow schedules:

- **Morning (6:00-9:00)**: Citizens leave homes for work/school. This is peak traffic.
- **Midday (12:00-13:00)**: Some citizens go to lunch (commercial demand spike).
- **Evening (17:00-20:00)**: Citizens return home. Second traffic peak.
- **Night (22:00-6:00)**: Reduced activity. Some entertainment venues active.

**How the game creates traffic demand:**
Every simulated citizen independently decides their next action when their current activity
timer expires. The decision weights are:
- 60% chance of going to work/school (during work hours)
- 20% chance of going shopping
- 15% chance of going to leisure/parks
- 5% chance of "wandering" (random destination)

These weights shift based on time of day, creating realistic rush-hour patterns.

### 3.4 The Death Wave Problem

One of CS1's most notorious emergent behaviors. When a city is built rapidly:

1. A large batch of residential zones is placed at once
2. Citizens move in simultaneously -- they're all approximately the same age
3. ~200 game-time-units later, they ALL DIE AT THE SAME TIME
4. The city experiences a "death wave" -- hundreds of buildings need deathcare simultaneously
5. Hearses can't keep up, bodies pile up, neighbors get unhappy, buildings abandon
6. Mass abandonment causes population crash, tax revenue crash, service spiral
7. New residents move in simultaneously, and the cycle repeats

**Mitigation strategies players discovered:**
- Build residential zones gradually over time (stagger the age cohorts)
- Over-build deathcare capacity (3-4x what seems necessary)
- Use the "slow aging" mod to spread out natural death timing
- Place crematoriums/cemeteries distributed across the city, not centralized

CS2 addressed this by randomizing initial citizen ages when they move in, though
a weaker version of the problem still exists.

### 3.5 Citizen Movement Mechanics

**Walking:**
- Citizens walk at ~4 km/h (1.1 m/s)
- Maximum walk distance before choosing vehicle: ~400m (approximately 50 cells)
- Citizens consider walking if destination is close enough
- Pedestrian paths (no vehicles) are a separate network layer
- Walking citizens use sidewalks attached to roads or dedicated paths

**Driving:**
- Citizens "own" a car (pocket car system, see 2.5)
- They drive from building to nearest road node, follow path, arrive at destination road node
- Parking is not simulated in CS1 -- car just disappears

**Public transit:**
- Citizens evaluate transit cost vs driving cost
- Transit cost = walk to stop + wait time + ride time + walk from stop
- If transit cost < (drive time * comfort_factor), citizen takes transit
- The comfort_factor makes driving preferred (~1.5x multiplier on transit time to be competitive)
- Citizens can chain multiple transit types (walk -> bus -> metro -> walk)

### 3.6 CS2 Citizen Changes

CS2 made citizens more realistic but at enormous performance cost:

- **Persistent identity**: All citizens have names, jobs, homes, relationships
- **Household simulation**: Families form, have children, children grow up
- **Aging**: More granular aging with actual birth/death events
- **Personal vehicles**: Cars are entities that exist in the world, need parking
- **Commute preferences**: Citizens have preferred transport modes based on trip distance
- **Performance impact**: Simulating ~50k "real" citizens in CS2 was equivalent in CPU cost
  to ~200k citizens in CS1's hybrid approach

---

## 4. Economy and Budget

### 4.1 Income Sources (CS1)

The CS1 economy is relatively simple but has important emergent properties.

**Tax revenue** (primary income, ~60-80% of city budget):
- Residential tax: Default 9%, adjustable 1-29%, applied per household
- Commercial tax: Default 9%, adjustable 1-29%, applied per commercial building
- Industrial tax: Default 9%, adjustable 1-29%, applied per industrial building
- Office tax: Default 9%, adjustable 1-29%, applied per office building
- Taxes can be set independently per zone type AND per density (LD vs HD)
- Revenue = tax_rate * building_income * building_level_multiplier

**Tax sensitivity thresholds:**
| Rate | Effect |
|------|--------|
| 1-9% | No complaints |
| 10-11% | Minor happiness penalty, ~5% |
| 12% | Noticeable happiness penalty, ~10% |
| 13-14% | Significant complaints, some buildings stop leveling |
| 15%+ | Buildings start abandoning |
| 20%+ | Mass abandonment |
| 29% | City death spiral |

**Other income:**
- **Outside connections**: Income from goods exported via highway, rail, ship, air
- **Tourism**: Visitors spend money at commercial buildings and attractions
- **Toll booths**: Can be placed on roads (After Dark DLC), earn per-vehicle toll
- **Park entrance fees**: (Parklife DLC) ticket revenue from parks
- **Transit fares**: Small per-ride income from public transport users

### 4.2 Expenses

**Service costs** (the major expense categories):
- Road maintenance: Per-segment cost based on road type and length
- Water/sewage: Per pump/treatment plant operating cost
- Electricity: Per power plant operating cost minus revenue from selling excess
- Education: Per school operating cost (scales with capacity)
- Healthcare: Per clinic/hospital operating cost
- Police: Per station operating cost
- Fire: Per station operating cost
- Garbage: Per facility operating cost plus per-truck cost
- Public transit: Per line operating cost (heavily subsidized, rarely profitable)
- Parks: Per park maintenance cost

**Budget sliders:**
Each service category has a budget slider (50-150% of base cost). Setting it:
- Below 100%: Reduces effectiveness (fewer vehicles dispatched, less coverage)
- Above 100%: Increases effectiveness (more vehicles, faster response)
- The relationship is NOT linear. 150% budget gives ~120% effectiveness (diminishing returns)
- Below 50% is not allowed

### 4.3 Economic Cycles

CS1 has a simple economic cycle:
- When commercial buildings have high goods supply and customer demand: they prosper
- When industrial output exceeds local commercial demand: goods are exported (income)
- When commercial demand exceeds supply: goods are imported (cost)
- The "generic industry" chain: Raw materials -> processed goods -> commercial supply

The Industries DLC added a production chain system:
- Specialized resources (oil, ore, forestry, farming) mined/harvested
- Processed in specialized facilities
- Sold to commercial or exported
- This creates a real supply chain with logistics requirements

### 4.4 CS2 Economy -- The Launch Disaster

CS2's economy was broken at launch in multiple intersecting ways, creating a crisis
that became the #1 community complaint:

**The rent problem:**
- CS2 introduced a rent system where building operating costs are passed to tenants
- Rent was calculated as: building_upkeep + land_value_tax + service_fees
- At launch, this formula was miscalibrated. High-density buildings had operating costs
  so high that rent exceeded citizen income
- Citizens couldn't pay rent -> buildings went "abandoned" -> death spiral
- The famous "Not Enough Customers" + "Rent Too High" notification combo

**The production cost problem:**
- Industrial production costs were set too high relative to commercial sale prices
- Companies went bankrupt constantly, creating goods shortages
- The "Not Enough Goods" notification appeared everywhere
- Empty shelves -> citizens unhappy -> abandonment cascade

**The subsidy trap:**
- To keep buildings alive, the city had to subsidize services heavily
- But the subsidies drained the treasury
- Many players reported that ANY city above ~30,000 pop went bankrupt
- The "profitable city" was essentially impossible at launch without exploits

**Post-launch patches:**
Colossal Order spent 6+ months rebalancing:
- Patch 1.0.14f1: Reduced building upkeep costs by ~30%
- Patch 1.0.15f1: Adjusted rent calculation, reduced service fees
- Patch 1.0.18f1: Major economy overhaul, rebalanced entire production chain
- Patch 1.1.0f1: "Economy 2.0" -- near-complete rewrite of cost formulas
- Even after all patches, the economy remained less stable than CS1's simpler model

### 4.5 Money Exploits (CS1)

Well-known CS1 money exploits:

1. **Hydro power printing**: Build a dam on any river. The electricity sale revenue often
   exceeds the construction cost within 1-2 game years. Stack multiple dams.
2. **Education tax trick**: Set education budget to 50% early game. Schools still work
   (just slower), saves massive money during the cash-tight early period.
3. **Industry tax exploitation**: Specialized industry (from Industries DLC) has
   different tax sensitivity. Oil industry tolerates 12-13% tax rate.
4. **Transit profit lines**: Some bus/metro lines connecting dense areas to commercial
   districts can actually turn a profit (rare but possible with optimization).

---

## 5. Services and Coverage

### 5.1 Service Coverage Model (CS1)

Services in CS1 operate on a coverage area model. Each service building radiates an
effect within a radius, and buildings within that radius receive the service benefit.

**Coverage is NOT a boolean** -- it's a gradient:
- At the service building: 100% coverage
- At the edge of radius: ~10% coverage
- The falloff is roughly linear
- Multiple service buildings stack (diminishing returns)

### 5.2 Service Types and Stats

**Police:**
| Building | Radius | Capacity | Vehicle | Cost/week |
|----------|--------|----------|---------|-----------|
| Police Station | ~500m | 20 cells | 5 patrol cars | 700 |
| Police HQ | ~800m | 35 cells | 10 patrol cars + helicopter | 1,200 |
| Prison | N/A | Holds 200 criminals | Prison vans | 2,000 |

Mechanics:
- Police reduce crime rate within coverage area
- Crime rate = base_crime - police_effect + (population_density * 0.1) - (education * 0.05)
- Uneducated, unemployed citizens commit more crime
- High land value areas have lower base crime
- Crime rate affects happiness and land value (feedback loop)
- Police vehicles patrol streets, respond to "crime scenes" (events)

**Fire:**
| Building | Radius | Vehicles | Cost/week |
|----------|--------|----------|-----------|
| Fire Station | ~500m | 5 fire trucks | 600 |
| Fire House | ~250m | 2 fire trucks | 300 |

Mechanics:
- Fire hazard accumulates over time on buildings NOT covered by fire service
- Hazard reaches critical level -> random fire event
- Fire trucks dispatched to put out fire (takes ~30 game minutes)
- If fire truck can't reach building (traffic!), building burns down and is destroyed
- Adjacent buildings can catch fire (spread within ~2 cell radius)
- Fire hazard resets to 0 after fire truck visit
- Industrial buildings have higher base fire hazard than residential

**Garbage:**
| Building | Collection Radius | Capacity | Vehicles | Cost/week |
|----------|------------------|----------|----------|-----------|
| Landfill | ~500m | 480,000 units | 9 garbage trucks | 600 |
| Incinerator | ~500m | Infinite (burns) | 12 trucks | 1,500 |
| Recycling Center | ~500m | Infinite (processes) | 6 trucks | 1,000 |

Mechanics:
- Buildings generate garbage at a rate proportional to their population/workers
- Garbage trucks pick up garbage on a route (not on-demand)
- If garbage exceeds threshold, buildings get "Too Much Garbage" complaint
- Uncollected garbage reduces land value and can cause sickness
- Landfills fill up and must be eventually emptied (truck by truck!) or decommissioned
- Incinerators burn garbage but create pollution

### 5.3 Service Vehicle Dispatching

This is a critical system that causes many player frustrations:

**The nearest-vehicle problem:**
When a service is needed (fire, police, healthcare, deathcare), the game dispatches the
nearest available vehicle. But "nearest" is calculated as straight-line distance, NOT
road distance. This means:
- A fire station 200m away across a river might send a truck that has to drive 5km around
- Meanwhile, a station 400m away by road (but further straight-line) sits idle
- This algorithm is the cause of many "WHY ARE MY SERVICES SO BAD" complaints

**TM:PE improvement:**
TM:PE can change dispatch to use road-network distance instead of Euclidean distance,
which dramatically improves service effectiveness.

**Vehicle limits per building:**
Each service building has a hard cap on active vehicles. If all vehicles are deployed,
the building cannot respond to new events until one returns, regardless of how critical
the event is.

---

## 6. Land Value and Desirability

### 6.1 Land Value Calculation (CS1)

Land value is a per-cell score (0-100+) that determines building levels and citizen wealth.
It's calculated as a sum of positive and negative factors:

**Positive factors:**
- Proximity to services: police, fire, health, education (+5-15 each)
- Proximity to parks and plazas (+5-20 depending on park type/size)
- Proximity to decorated roads (tree-lined) (+2-6)
- Proximity to water bodies (ocean, river) (+5-10, diminishes with distance)
- Proximity to unique buildings/monuments (+10-30)
- Building level (existing high-level buildings boost neighbors) (+1-5)
- Low crime rate (+0-10)

**Negative factors:**
- Proximity to industrial zones (-5-15)
- Proximity to pollution sources (-5-20)
- Noise pollution (-3-10)
- Proximity to cemeteries/crematoriums (-3-8)
- Proximity to garbage facilities (-5-15)
- Crime rate (-0-15)
- Proximity to highways/busy roads (noise) (-2-5)

**The propagation model:**
Land value propagates through cells. High-value areas "pull up" neighboring cells,
creating natural value gradients. The game updates land value iteratively each
simulation tick, using a weighted average of current value + neighbor values + factor
contributions. This creates emergent "nice neighborhoods" and "bad neighborhoods."

### 6.2 Land Value and Building Level Feedback

There's an important feedback loop:
1. Services + parks increase land value
2. Higher land value enables building level-ups
3. Higher-level buildings house more citizens, generate more tax
4. More tax revenue enables more services
5. Goto 1

The reverse is also true (death spiral):
1. Service cut (budget crisis) -> coverage drops
2. Land value falls -> buildings level down
3. Less tax revenue -> worse budget crisis
4. More service cuts -> more land value drops
5. Abandonment cascade

Understanding this feedback loop is essential for stable city design.

---

## 7. Education System

### 7.1 Education Tiers (CS1)

CS1 has a 3-tier education system that profoundly affects city development:

**Elementary School:**
- Capacity: 600 students
- Radius: ~700m
- Cost: 800/week
- Educates children (age 0-14)
- Duration: Full childhood period
- Graduates gain "educated" status

**High School:**
- Capacity: 1,000 students
- Radius: ~1,200m
- Cost: 1,100/week
- Educates teens (age 15-44)
- Requires elementary education first
- Graduates gain "well educated" status

**University:**
- Capacity: 2,500 students
- Radius: City-wide (students will commute)
- Cost: 1,500/week
- Educates young adults (age 45-89)
- Requires high school education first
- Graduates gain "highly educated" status

**Campus DLC extended university into a multi-building campus system with:**
- Trade Schools (for industrial workers)
- Liberal Arts Colleges
- Universities (science/tech)
- Each has a leveling system based on student count and academic works
- Campus areas can generate "academic works" that provide city bonuses

### 7.2 Education Effects on City

Education level of the workforce has major downstream effects:

- **Industrial demand**: Uneducated and elementary-educated workers needed
  - Highly educated citizens REFUSE industrial jobs (CS1 base game)
  - This creates the famous "Not Enough Workers" / "Not Enough Educated Workers" paradox
  - Industry wants dumb workers, offices want smart workers, you can't satisfy both
- **Office demand**: Requires well-educated and highly-educated workers
- **Commercial demand**: Prefers educated workers but tolerates all levels
- **Crime**: Each education level reduces crime rate by ~15-20%
- **Land value**: Educated populations drive land value up (they demand better housing)
- **Health**: Educated citizens live longer (visit healthcare more regularly)
- **Fire risk**: Educated populations have slightly lower fire hazard

### 7.3 The Education Pipeline Problem

A subtle CS1 issue: education takes TIME. When you build a university:
1. Children must first complete elementary (~2 game years)
2. Then complete high school (~2 game years)
3. Then complete university (~2 game years)
4. Total time to produce a highly educated citizen from scratch: ~6 game years
5. During this time, offices won't fill, industry complains about over-education
6. The transition period is extremely painful for city management

**CS2 changes:**
- Education levels are more granular
- Educated citizens CAN work lower-tier jobs (but with happiness penalty)
- The "over-education" problem is significantly reduced
- Education buildings have more capacity variance (small school to large campus)

---

## 8. Healthcare and Deathcare

### 8.1 Healthcare System (CS1)

**Medical buildings:**

| Building | Capacity | Ambulances | Helicopters | Radius | Cost/week |
|----------|----------|------------|-------------|--------|-----------|
| Medical Clinic | 100 patients | 3 | 0 | ~400m | 400 |
| Hospital | 500 patients | 10 | 1 | ~800m | 1,600 |
| Medical Center | 1000 patients | 25 | 3 | ~1200m | 3,200 |

**Health mechanics:**
- Citizens have a health value 0-100
- Health decreases from: age, pollution exposure, noise pollution, no healthcare access
- Health increases from: healthcare coverage, parks nearby, clean environment
- When health drops below ~30, citizen becomes "sick"
- Sick citizens generate ambulance requests
- If ambulance doesn't arrive in time, citizen dies
- Dead citizens need deathcare (see below)

**Healthcare coverage effect on population:**
- 0% coverage: Average lifespan ~180 age units
- 50% coverage: Average lifespan ~250 age units
- 100% coverage: Average lifespan ~320 age units
- Over-coverage (multiple facilities): Diminishing returns, max lifespan ~350

### 8.2 Deathcare System (CS1)

Deathcare is one of CS1's most unique and frequently frustrating systems. Dead citizens
don't just disappear -- they become "corpses" that must be physically collected.

**Deathcare buildings:**

| Building | Capacity (bodies) | Hearses | Cost/week |
|----------|-------------------|---------|-----------|
| Cemetery | 2,000 | 6 | 500 |
| Crematorium | Infinite | 10 | 800 |

**How deathcare works:**
1. Citizen dies -> building has a "dead body" flag
2. Hearse dispatched from nearest cemetery/crematorium (Euclidean distance!)
3. Hearse drives to building, picks up body
4. Hearse returns to facility
5. Cemetery stores body (fills up over time)
6. Crematorium burns body immediately (never fills up)

**Deathcare failure cascade:**
- If all hearses busy -> bodies accumulate
- Buildings with uncollected bodies > 2 game weeks -> neighbors complain
- Neighboring citizens become unhappy -> "dead person being collected" modifier
- Extended body accumulation -> buildings abandon
- The famous "skull icon" death spiral

**Cemetery management:**
- Cemeteries have finite capacity (2,000 bodies)
- When full, they must be "emptied" -- this sends hearses to relocate ALL bodies
- During emptying, the cemetery cannot accept new bodies
- This creates a gap in deathcare coverage, potentially triggering cascades
- Expert strategy: Never rely on cemeteries alone; always pair with crematoriums

### 8.3 CS2 Healthcare Changes

CS2 attempted to make healthcare more realistic:
- Multiple hospital types (general, specialized, children's)
- Healthcare cost per citizen (adds to city expenses)
- Ambulance routing improved (road distance instead of Euclidean for dispatch)
- Deathcare simplified slightly -- bodies handled faster
- Death wave issue reduced by randomized initial ages

---

## 9. Pollution

### 9.1 Ground Pollution (CS1)

Ground pollution is a cell-based value that spreads through soil:

**Sources:**
- Industrial buildings: Major source, radius ~10-20 cells
- Landfills: Large radius, ~15 cells, persists after decommissioning
- Some power plants (coal, oil): Radius ~10 cells
- Incinerators: Small radius, ~8 cells

**Mechanics:**
- Pollution value is 0-255 per cell
- Spreads to adjacent cells each simulation tick (diffusion model)
- Decay rate: Very slow (~1 unit per game week naturally)
- Trees absorb pollution (each tree removes ~1-3 units per tick in radius)
- Water can carry pollution downstream (pumping station location critical!)

**Effects:**
- Land value reduction: -1 land value per ~5 pollution units
- Citizen health: Citizens in polluted areas lose ~1 health per game week per 10 pollution
- Building level: Polluted areas cannot reach building levels 4-5
- Soil recovery after removing source: ~20-50 game weeks depending on severity

### 9.2 Water Pollution (CS1)

Water pollution is separate from ground pollution and follows water flow:

**Sources:**
- Sewage outlets: Discharge polluted water into rivers/ocean
- Industrial buildings near water: Seep pollution into water table
- Landfills near water: Contaminate groundwater

**Mechanics:**
- Pollution flows downstream in rivers (uses fluid simulation)
- If a water pump is downstream from a sewage outlet, pumped water is polluted
- Polluted water distributed to buildings causes sickness
- Water treatment plants reduce output pollution by ~85%
- The #1 beginner mistake in CS1: placing water intake downstream from sewage output

### 9.3 Noise Pollution (CS1)

Noise pollution is separate from ground/water pollution:

**Sources and values:**
| Source | Noise Level | Radius |
|--------|------------|--------|
| Highway (6-lane) | 80 | ~8 cells |
| Major road (4-6 lane) | 60 | ~5 cells |
| Minor road (2-lane) | 20 | ~2 cells |
| Industrial building | 40-70 | ~6 cells |
| Commercial building | 20-40 | ~3 cells |
| Airport | 100 | ~20 cells |
| Train tracks | 50 | ~4 cells |
| Power plants | 30-80 | ~8 cells |

**Effects:**
- Residential happiness reduction proportional to noise level
- Land value reduction: -1 per ~10 noise units
- No health effect (unlike ground pollution)
- Noise barriers (After Dark DLC): Walls along roads that block ~50% noise

### 9.4 CS2 Pollution Changes

CS2 added more pollution granularity:
- **Air pollution**: Separate visual system with particles/smog. Affected by wind direction.
- **Ground pollution**: Similar to CS1 but with better visualization
- **Water pollution**: Same downstream flow model
- **Noise pollution**: More realistic sound propagation considering building occlusion
- **Telecom radiation**: New concept where cell towers create small "pollution" zones
  (controversial design choice, removed/reduced in patches)

---

## 10. Water and Electricity Infrastructure

### 10.1 Water System (CS1)

**Water supply chain:**
1. Water source (river, lake, ocean, or groundwater)
2. Water pumping station (extracts water)
3. Water pipes (underground network, connects to all buildings)
4. Buildings consume water
5. Sewage generated (per building, proportional to water consumed)
6. Sewage pipes (separate or combined network)
7. Sewage outlet (dumps into water body) or water treatment plant

**Water buildings:**

| Building | Capacity | Pumping | Cost/week | Notes |
|----------|----------|---------|-----------|-------|
| Water Tower | 4,800 m3/week | From ground | 320 | No water body needed |
| Water Pumping Station | 38,400 m3/week | From water body | 640 | Must be on shore |
| Water Treatment Plant | N/A | Treats sewage | 960 | Reduces outflow pollution 85% |
| Inland Water Treatment | 32,000 m3/week | Combined pump + treat | 1,280 | Added in DLC |

**Pipe network:**
- Water pipes form an underground network independent of roads
- Pipes have infinite capacity (no flow simulation, just connectivity)
- A building is "connected" if a pipe is within 2 cells
- Pipes decay over time in CS2 (not in CS1)
- The pipe grid resolution matches the zone grid (8m cells)

**Critical design considerations:**
- Water pumps must be placed upstream of sewage outlets
- Pumping polluted water distributes pollution to all connected buildings
- Cities on flat terrain without rivers can only use water towers (limited capacity)
- Water towers pump from groundwater, which can be polluted by nearby industry

### 10.2 Electricity System (CS1)

**Power plants:**

| Plant | Output (MW) | Cost/week | Pollution | Notes |
|-------|------------|-----------|-----------|-------|
| Wind Turbine | 8 | 100 | None | Intermittent, noisy |
| Solar Power Plant | 16 | 1,200 | None | Intermittent (day only) |
| Coal Power Plant | 40 | 800 | Heavy | Cheapest per MW |
| Oil Power Plant | 40 | 1,000 | Moderate | |
| Nuclear Power Plant | 640 | 3,200 | None* | *Can melt down in Natural Disasters DLC |
| Geothermal Plant | 40 | 2,800 | None | Expensive but clean |
| Hydro Dam | Varies | 1,200 | None | Output depends on water flow |
| Fusion Power Plant | 1,600 | 8,000 | None | End-game, enormous output |

**Electricity network:**
- Power travels through the zone grid automatically (buildings conduct to adjacent buildings)
- Dedicated power lines are needed to bridge gaps (unbuilt areas, over water)
- Buildings within 2 cells of a powered building receive power automatically
- No voltage/capacity simulation -- just connectivity and total supply vs demand
- Power plants ramp up/down based on demand (no wasted generation)

**Grid failure cascade:**
If power supply drops below demand:
- The game doesn't brown-out gradually -- it fully powers SOME buildings and fully cuts OTHERS
- Which buildings lose power is essentially random (based on graph traversal order)
- This can cause seemingly random abandonment across the city
- Smart players maintain 15-20% power surplus at all times

### 10.3 CS2 Infrastructure Changes

CS2 made infrastructure systems more complex:
- **Water pipes have capacity** and can be upgraded
- **Electrical grid has voltage levels** (low/medium/high voltage)
- **Transformer stations** needed to step down from high to low voltage
- **Underground infrastructure view** improved for visibility
- **Pipe/wire aging**: Infrastructure degrades and needs replacement
- Performance cost of these simulations contributed to CS2's performance problems

---

## 11. Public Transportation

### 11.1 Transit Types (CS1)

CS1 has a rich public transit system with multiple vehicle types, each serving different
capacity and speed niches:

**Bus:**
- Capacity: 30 passengers per bus
- Speed: Limited by road speed, typically 30-40 km/h in city
- Cost: ~400/week per line + ~100/week per bus
- Stop spacing: Player-defined, optimal ~300-400m apart
- Strengths: Cheapest, most flexible routing (follows roads)
- Weaknesses: Stuck in traffic, low capacity, slow
- The "bus bunching" problem: Multiple buses on same line cluster together due to
  traffic variations. The front bus picks up all passengers, rear buses run empty.

**Tram (Snowfall DLC):**
- Capacity: 90 passengers per tram
- Speed: 40 km/h (dedicated lanes immune to traffic)
- Cost: ~600/week per line (plus track construction)
- Requires dedicated tram tracks OR road+tram combo roads
- Better than buses for medium-density corridors

**Metro/Subway:**
- Capacity: 180 passengers per train
- Speed: 80 km/h (underground, no traffic interaction)
- Cost: ~1,200/week per line + ~500 per station
- Station construction is expensive (underground excavation)
- The workhorse of large CS1 cities -- highest capacity-per-cost underground
- Stations create significant surface land value boost (+15-25)

**Train:**
- Capacity: 240 passengers per train
- Speed: 120 km/h on dedicated rail
- Cost: ~2,000/week per line + ~800 per station
- Requires dedicated rail lines (NOT on roads)
- Best for long-distance routes and inter-city connections
- Train stations are huge land footprint (~6x6 cells minimum)

**Monorail (Mass Transit DLC):**
- Capacity: 180 passengers
- Speed: 80 km/h (elevated, no traffic interaction)
- Cost: ~1,000/week per line
- Elevated tracks can be built over existing roads
- Visually distinctive but functionally similar to metro

**Cable Car (Mass Transit DLC):**
- Capacity: 30 passengers per gondola
- Speed: 30 km/h
- Cost: ~400/week per line
- Can traverse extreme elevation changes
- Mainly for mountain/valley cities and tourist attractions

**Ferry (Mass Transit DLC):**
- Capacity: 50 passengers per ferry
- Speed: 30 km/h on water routes
- Cost: ~600/week per line
- Requires waterfront stops
- Useful for cities with rivers/harbors dividing districts

**Blimp (Mass Transit DLC):**
- Capacity: 35 passengers
- Speed: 50 km/h (air route, straight line)
- Cost: ~800/week per line
- Point-to-point air service
- More a novelty than serious transit; low capacity, expensive

**Helicopter (Mass Transit DLC):**
- Capacity: 15 passengers
- Speed: 120 km/h
- Cost: ~1,200/week per line
- Fastest transit option, lowest capacity
- Emergency services (ambulance helicopter) uses same system

### 11.2 Transit Line Optimization

The CS1 community developed extensive transit optimization knowledge:

**Line design principles:**
- Lines should be "loop" or "back-and-forth" patterns
- Optimal line length: 4-8 stops (longer lines have reliability problems)
- Terminus station should have a layover area (space for vehicles to turn around)
- Intersecting lines should share transfer stations (modal interchange)

**The "donut" network:**
Expert players use a hierarchical transit design:
1. High-capacity ring lines (metro) around major centers
2. Radial feeder lines (bus/tram) from suburbs to metro stations
3. Express lines (train) connecting distant districts
4. This mirrors real transit network design (hub-and-spoke)

**Vehicle count tuning:**
- Too few vehicles: Long wait times, overcrowded, passengers walk instead
- Too many vehicles: Bunching, wasted money, vehicles queueing at stops
- Optimal formula: vehicle_count = ceil(line_travel_time / desired_headway)
- Desired headway for metro: 3-5 min; for bus: 5-8 min

### 11.3 Transit Pathfinding and Mode Choice

When citizens evaluate transit:
1. Calculate walk time to nearest stop/station
2. Estimate wait time (based on line frequency)
3. Calculate ride time (based on route and speed)
4. Calculate walk time from destination stop to destination
5. Total transit time = sum of all above
6. Compare to: drive time * comfort_penalty_factor (~1.3-1.5x)
7. If transit time < adjusted drive time, citizen uses transit

**Multi-modal trips:**
Citizens can chain modes: Walk -> Bus -> Metro -> Walk
The pathfinder evaluates all possible chains and picks the fastest.
Transfer penalties: Each mode change adds ~3 minutes of "perceived" cost.

### 11.4 CS2 Transit Changes

CS2 overhauled public transit:
- **Transit line management UI** completely redesigned (and was widely criticized for
  being worse than CS1's relatively clean system)
- **Passenger counting**: Better per-line ridership data
- **Vehicle overcrowding**: Visible crowding on platforms, passengers skip full vehicles
- **Walking distance increased**: Citizens walk further to transit in CS2 (~500m vs ~400m)
- **Transit fees**: Per-ride income more impactful
- **Major criticism**: Transit was much harder to make financially viable in CS2.
  Lines that would be affordable in CS1 bankrupted cities in CS2 due to the economy rebalance.

---

## 12. Districts and Policies

### 12.1 Districts (CS1)

Districts are player-drawn areas that allow localized policy application and specialization.

**District painting:**
- Paint with a brush tool over the city map
- Districts can be any shape, any size
- Buildings within a district boundary belong to that district
- Districts can overlap with zone types (a district can contain res + com + ind)
- Districts automatically track population, employment, land value, etc.

### 12.2 Policies (CS1)

Policies are toggleable modifiers that affect gameplay. They can be applied city-wide or
per-district. There are ~35 policies in the base game, expanded to ~55 with all DLCs.

**Notable policies with specific effects:**

| Policy | Effect | Cost | Notes |
|--------|--------|------|-------|
| Free Public Transport | Transit ridership +25-50% | All transit revenue lost | Great for reducing traffic |
| High-Rise Ban | Prevents level 4-5 buildings | None | Controls skyline |
| Heavy Traffic Ban | Bans trucks from district roads | Small industry penalty | Reduces noise/road damage |
| Schools Out | Reduces education rate by 50% | Saves education budget | Helps retain industry workers |
| Small Business Enthusiast | Prevents commercial levelup past 2 | None | Keeps neighborhood character |
| Big Business Benefactor | Bonus to commercial level-up | Small cost | More tax from commercial |
| Industrial Space Planning | +50% industrial output | Slight pollution increase | Good for export economy |
| Recycling | Reduces garbage generation by 20% | +10% garbage budget | |
| Parks & Rec | +10% park land value boost | +10% parks budget | |
| Smoke Detector Distribution | -50% fire hazard | 0.5/citizen/week | Cheap and highly effective |
| Pet Ban | -10% garbage, -5% happiness | None | Quirky but useful |
| Encourage Biking | +15% cycling, -10% car trips | None | Requires bike infrastructure |
| Old Town | Prevents building growth/change | None | Preserves aesthetic districts |
| Combustion Engine Ban | Bans private cars in district | None | Green Cities DLC |
| Organic and Local Produce | -50% goods import, +20% cost | 5/building/week | Green Cities DLC |

### 12.3 Industrial Specializations

Within districts, industrial zones can be specialized:

**Generic Industry**: Default, processes imported raw materials into goods
**Forest Industry**: Harvests trees, processes lumber. Requires forested area.
**Agricultural Industry**: Farms crops. Requires fertile land.
**Oil Industry**: Extracts oil. Requires oil deposit (shown in resource overlay).
**Ore Industry**: Mines ore/metal. Requires ore deposit.

Each specialization:
- Uses different building models
- Has different worker requirements
- Produces different goods for supply chain
- The resource deposits are FINITE (except farming) -- eventually oil/ore runs out
- Smart players transition from extraction to processing before deposits deplete

### 12.4 CS2 District Changes

CS2 changed districts and policies significantly:
- **Zoning modifiers**: Instead of just painting districts, CS2 lets you apply zone
  modifiers directly (like "European style" or "row houses")
- **Policy system expanded**: More granular policies with numerical sliders instead of
  binary toggles
- **City-wide vs district policies**: Same concept but with more options
- **Signature buildings**: Unique buildings that can appear in specific zones

---

## 13. Milestones and Unlocks

### 13.1 Milestone System (CS1)

CS1 uses population milestones to gate content, providing a sense of progression:

| Milestone | Population | Key Unlocks |
|-----------|-----------|-------------|
| Little Hamlet | 0 | Basic roads, R/C/I zones, water, power |
| Worthy Village | 240 | Healthcare, deathcare, garbage |
| Tiny Town | 1,200 | Fire, police, education (elementary) |
| Boom Town | 2,600 | High school, parks, policies |
| Busy Town | 5,000 | Unique buildings, bus lines |
| Big Town | 7,500 | High density zones, metro, office zones |
| Small City | 12,000 | University, train, cargo |
| Big City | 20,000 | International airport, ferry |
| Grand City | 36,000 | Tax office, more unique buildings |
| Capital City | 50,000 | Stock exchange, monument unlocks start |
| Colossal City | 65,000 | Space elevator, hadron collider, etc. |
| Megalopolis | 80,000 | All monuments, max tile purchases |

**Map tile system:**
- The total map is 9x9 tiles (81 tiles), each tile is ~2km x 2km
- Base game: Can purchase up to 9 tiles (3x3 area)
- With "81 Tiles" mod: All 81 tiles purchasable (most popular mod, >5M subscribers)
- Tile purchases are gated by milestones
- Each tile costs increasing amounts of money

### 13.2 Unique Buildings

Unique buildings are special structures unlocked by meeting specific requirements:

- Some unlock at milestones (population thresholds)
- Some require specific conditions:
  - "Plaza of the Dead": Have 100+ dead citizens at once
  - "Lazaret Plaza": Cure 1000 sick citizens
  - "Posh Mall": Have commercial buildings reach level 3 in 5+ areas
  - "Sea-and-Sky Scraper": Have 10,000+ population and 95%+ building level coverage
- Unique buildings provide large land value boosts and tourist attractions
- Monuments (end-game) require ALL unique buildings in their category to be built first

### 13.3 CS2 Progression Changes

CS2 uses an XP-based "Development Points" system:
- Earn XP from placing zones, services, achieving goals
- Spend development points on an unlock tree
- More flexible than linear milestone gates
- Players criticized this as "gamifying" what should be organic progression
- Some felt it was too grindy -- needing to earn points for basic features like parks

---

## 14. CS1 DLC Systems (All DLCs)

CS1 received extensive DLC support over its 8-year lifespan (2015-2023). Each DLC added
systems that significantly changed gameplay. Below is every major DLC with its specific
mechanical additions.

### 14.1 After Dark (October 2015) -- First Expansion

**Theme**: Day/night cycle and nightlife economy

**Key additions:**
- **Day/night cycle**: Visual cycle plus gameplay effects
  - Commercial tax revenue shifts: +30% at night from "leisure" commercial
  - Crime rate +50% at night
  - Some services reduced effectiveness at night (schools close)
  - Street lights on roads (automatic, cost included in road maintenance)
- **Leisure/Tourism commercial specialization**:
  - Leisure: Bars, clubs, restaurants. Higher nighttime revenue.
  - Tourism: Hotels, souvenir shops. Revenue from tourists.
  - Both are district-level specializations for commercial zones
- **Taxi service**: New transit type
  - Taxis pick up citizens, drive them to destinations
  - Taxi depot: Spawns taxis, ~8 per depot
  - Taxis reduce car traffic somewhat
- **Prison**: Separate building for holding criminals (police stations overflow to prison)
- **Bike lanes**: Road variants with dedicated bike lanes
  - Cyclists travel at ~15 km/h
  - Some citizens switch from car to bike based on distance and bike infrastructure
- **Bus lanes**: Road variants with dedicated bus lanes (buses don't get stuck in traffic)
- **Toll booths**: Placeable on roads, charge per vehicle, earnable income
- **Cargo hub**: Combined train+truck cargo terminal (major logistics improvement)

### 14.2 Snowfall (February 2016)

**Theme**: Winter weather and heating infrastructure

**Key additions:**
- **Snow maps**: Maps with permanent or seasonal snow
- **Heating system**: Buildings require heating in cold weather
  - Heating plants (boilers): Produce heat, distributed via pipes
  - Geothermal heating: Uses natural heat source, expensive but efficient
  - If no heating: Citizens get sick, happiness drops, buildings can abandon
  - Heat pipes are a THIRD underground network (in addition to water + sewage)
- **Road maintenance**: Snow-covered roads reduce speed and cause accidents
  - Road maintenance depots deploy snowplows
  - Snowplows clear roads (limited vehicle count, must route efficiently)
  - Unplowed roads: Speed -40%, accident chance increased
- **Tram transit**: (Described in section 11)
  - Tram tracks can be placed on roads (road+tram variants)
  - Dedicated tram-only roads also available
  - Tram depots required for vehicle maintenance
- **Winter unique buildings**: 5 new unique buildings (ice hockey arena, ski resort, etc.)

### 14.3 Natural Disasters (November 2016)

**Theme**: Destructive events and disaster response

**Key additions:**
- **Disaster types:**
  - Thunderstorm: Lightning strikes can start fires, heavy rain reduces visibility
  - Tsunami: Massive wave hits coastal cities, destroys buildings near shore
  - Tornado: Moves across map, destroys buildings in path (width varies by intensity)
  - Earthquake: Ground shaking damages buildings city-wide (intensity-based)
  - Sinkhole: Ground collapses in localized area, destroying buildings and roads
  - Meteor strike: Targeted destruction with fire spread
  - Forest fire: Spreads through trees, can reach buildings
  - Each disaster has intensity levels 1-10 affecting damage radius and severity

- **Disaster response:**
  - Disaster Response Unit: Spawns helicopters and trucks for rescue/repair
  - Emergency shelters: Citizens evacuate to shelters (reduce casualties)
  - Evacuation routes: Can designate roads for evacuation priority
  - Early warning systems: Detect disasters before they hit, gives evacuation time
  - Rebuild cost: Destroyed buildings must be re-placed and re-grow

- **Scenario mode**: Pre-built cities with specific disaster challenges
- **Random disaster toggle**: Can enable/disable random disasters in sandbox

### 14.4 Mass Transit (May 2017)

**Theme**: Extended transportation options

**Key additions:**
- **Monorail**: Elevated transit, 180 passengers, 80 km/h (details in section 11)
- **Cable Car**: Gondola-style, handles elevation changes
- **Ferry**: Water-based transit
- **Blimp**: Air-based novelty transit
- **Transit hubs**: Multi-modal stations combining:
  - Bus-metro hub
  - Train-metro hub
  - Monorail-bus hub
  - These are critical for efficient network design -- single building serves as transfer point
- **Road naming**: Can name individual roads (aesthetic feature, helps management)
- **Emergency vehicle override**: Emergency vehicles can use wrong-way lanes to bypass traffic
- **New road types**: Asymmetric roads (1+2, 2+3 lane configurations)

### 14.5 Green Cities (October 2017)

**Theme**: Eco-friendly city planning

**Key additions:**
- **Self-sufficient residential specialization**: Buildings with solar panels, green roofs
  - -50% electricity consumption
  - -30% water consumption
  - +10% happiness
  - Slightly higher building cost
- **Organic commercial specialization**: Health food stores, farmers markets
  - Reduced goods demand (local production)
  - Higher land value contribution
  - Fewer delivery trucks (less traffic)
- **IT Cluster office specialization**: Tech companies
  - Higher tax revenue
  - Requires highly educated workers (even more than normal office)
  - Very low pollution
- **Eco-friendly service buildings**: Green versions of service buildings
  - Floating garbage collector (water cleanup without pumps)
  - Recycling centers with better efficiency
  - Green power plants (advanced solar, advanced wind)
- **New policies**:
  - Combustion Engine Ban: No private cars in district (forces walking/transit/bikes)
  - Electric Cars: All vehicles in district are electric (reduces noise)
  - Organic and Local Produce: Changes commercial supply chain
- **Pollution reduction mechanics**: More tools for fighting existing pollution

### 14.6 Parklife (May 2018)

**Theme**: Park management and leisure

**Key additions:**
- **Park districts**: Draw a park district, gate it with an entrance, manage it as a unit
  - Parks level up based on visitor count and prop placement
  - Level 1: Local park (free entry, basic)
  - Level 2: Small park (entry fee possible, moderate attractions)
  - Level 3: City park (tourist attraction, significant revenue)
  - Level 4: National park (major destination, high revenue)
  - Level 5: Star attraction (landmark status)
- **Park types**:
  - City Park: Urban park with playground, fountains, gardens
  - Amusement Park: Rides, roller coasters, entertainment
  - Nature Reserve: Natural areas with trails, wildlife
  - Zoo: Animal exhibits (specific animals can be placed)
- **Park props**: Hundreds of placeable props within park districts
  - Benches, trees, pathways, rides, exhibits, food stalls
  - Each prop contributes to park level and attractiveness
- **Walking/hiking paths**: New pedestrian-only path types for parks
- **Camera mode**: Free-camera mode for screenshots (popular with content creators)
- **Sightseeing bus**: Tourist bus route, generates tourism income
- **Park entry fees**: Revenue generation from park visitors
- **Park maintenance**: Buildings within parks that employ gardeners/maintenance staff

### 14.7 Industries (October 2018)

**Theme**: Deep production chains and industrial management

**Key additions:**
- **Industry areas** (similar to park districts):
  - Paint a district, assign a resource type (farming, forestry, oil, ore)
  - Place extraction buildings, processing buildings, storage buildings
  - Industry areas level up based on worker count and output
  - Levels 1-5 with increasing efficiency and building options

- **Production chains:**
  - Farming: Grain -> Flour Mill -> Bakery (or Animal Pasture -> Meat -> Food Factory)
  - Forestry: Trees -> Sawmill -> Furniture Factory (or Paper Mill -> Printing Press)
  - Oil: Oil Extraction -> Petroleum Refinery -> Plastics Factory (or Fuel)
  - Ore: Mining -> Smelter -> Steel Mill (or Glass Factory)
  - Unique Factory: End-point that combines multiple processed goods into luxury products
  - Each step in the chain is a separate building with truck logistics between them

- **Warehouses and storage:**
  - Generic warehouse: Stores any goods, acts as buffer in supply chain
  - Specialized storage: Grain silo, log yard, oil tank, ore depot
  - Storage has capacity limits, overflow exported automatically
  - The logistics of keeping supply chains fed is the core challenge

- **Unique Factories**: 5 end-game factories, each requiring 2+ processed goods:
  - Ship factory, furniture factory, food factory, etc.
  - Each produces "luxury goods" that sell for high export value
  - Requires careful logistics setup to keep supplied

- **Postal service**: Mail carriers deliver mail (a new service layer)
  - Post office + post sorting facility
  - Mail delivery affects happiness
  - Postal trucks add to traffic

- **Industry vehicle management**: Track individual vehicles, see routes, optimize logistics

### 14.8 Campus (May 2019)

**Theme**: University campus management

**Key additions:**
- **Campus districts**: Draw a university campus area (similar to park/industry districts)
  - Three campus types: Trade School, Liberal Arts, University
  - Each has unique buildings and bonuses

- **Campus leveling** (based on student count and academic output):
  - Level 1: Small College (requires 500 students)
  - Level 2: Medium College (1,000 students + 3 academic works)
  - Level 3: Large College (2,500 students + 7 academic works)
  - Level 4: Prestigious University (5,000 students + 15 academic works)
  - Level 5: World-Famous University (10,000 students + 30 academic works)

- **Academic works**: Generated by specific campus buildings
  - Library, lab, lecture hall, etc. each produce academic works over time
  - Academic works count toward campus level

- **Campus buildings**: Dormitories, faculty buildings, stadiums, gyms, labs
  - Each has capacity and generates specific effects
  - Stadiums host sports events (ticket revenue)

- **Varsity sports**: Teams play games, generate revenue and happiness
- **Campus policies**: Specific policies for education districts
- **Museum/Gallery**: Cultural buildings that display artworks (from campus production)

### 14.9 Sunset Harbor (March 2020)

**Theme**: Water management and fishing

**Key additions:**
- **Fishing industry**: Fishing harbors, fishing routes, fish market
  - Fishing boats follow player-drawn routes on water
  - Fish sold at markets (revenue) or supplied to commercial (goods)
  - Overfishing possible -- fish populations can deplete
- **Inland water treatment**: Combined pump + treatment for non-coastal cities
- **Water pumping improvements**: Better control over pump rate
- **Aviation**: New aircraft types, helicopter public transit
- **Trolleybus**: Electric bus connected to overhead wires (between bus and tram)
  - Uses roads (no dedicated track needed)
  - Faster than bus, cheaper than tram
  - Requires overhead wire infrastructure
- **Intercity bus**: Long-distance bus terminal connecting to outside map
- **Overground metro**: Metro stations that can be placed at surface level
  - Cheaper than underground but takes surface space
- **Child/Elder care**: Daycare and eldercare buildings (new service type)
  - Daycare lets parents work (increases workforce)
  - Eldercare keeps seniors healthy longer
- **Waste processing**: Advanced recycling with multiple waste streams

### 14.10 Airports (January 2022)

**Theme**: Airport construction and management

**Key additions:**
- **Modular airport construction**: Build airports piece by piece
  - Runways (multiple lengths for different aircraft sizes)
  - Taxiways (connect runways to terminals)
  - Terminals (passenger processing, gates)
  - Parking structures
  - Cargo terminals
  - Fuel stations
  - Control tower
- **Airport area** (district-like): Draw boundary, place airport buildings inside
- **Airport leveling**: Levels 1-5 based on passenger throughput
- **Airlines**: Different airlines serve routes based on airport level
- **Cargo flights**: Air cargo supplements truck/rail logistics
- **Airport income**: Passenger fees, cargo fees, retail/parking revenue
- **Noise impact**: Airports generate massive noise pollution (realistic flight paths)
- **Road connections**: Airport requires highway-level road connections for traffic flow

### 14.11 Content Creator Packs and Radio Stations

In addition to major DLCs, CS1 sold many small content packs:
- **Art Deco**: Building style set (aesthetic only)
- **High-Tech Buildings**: Modern architecture set
- **European Suburbia**: European-style low-density buildings
- **University City**: Campus-themed buildings
- **Modern City Center**: Downtown building assets
- **Bridges & Piers**: Infrastructure asset variety
- **Train Stations**: Modular train station designs
- **Africa in Miniature**: African-themed buildings
- **Shopping Malls**: Large commercial building assets
- ~15 radio station DLCs (in-game radio flavor, no gameplay impact)

These packs totaled ~$100+ on top of the ~$180 for all major DLC, a common criticism.

### 14.12 Free Updates

Colossal Order also shipped substantial free patches alongside DLC:
- Road asset variety (asymmetric roads, speed bumps)
- European building style (toggle between American and European)
- Map editor improvements
- Performance optimizations
- UI quality-of-life improvements
- Modding API expansions
- These free updates were well-received and built community goodwill

---

## 15. CS2 Architecture Changes

### 15.1 Engine and Technology

CS1 was built on Unity 5 (later upgraded through Unity versions over its lifespan). CS2
moved to Unity's latest version with the DOTS (Data-Oriented Technology Stack) / ECS
(Entity Component System) architecture -- or at least attempted to. This was a fundamental
architectural change.

**CS1 engine characteristics:**
- Traditional Unity MonoBehaviour/GameObject architecture
- Single-threaded simulation with multithreaded rendering
- C# managed code for simulation, Unity native code for rendering
- 16-bit citizen IDs (hence the 65,536 limit)
- 16-bit vehicle IDs (16,384 limit)
- 32-bit building IDs
- Custom pathfinder running on a separate thread
- Save files: Serialized C# objects, typically 20-80 MB
- Modding: C# assemblies loaded via Harmony patching or direct Unity API access

**CS2 engine changes:**
- Unity DOTS/ECS for simulation entities (citizens, buildings, vehicles)
- Burst compiler for hot simulation loops
- Job system for multithreaded simulation
- However, significant portions of the code were NOT fully converted to ECS
- Many systems still used managed C# with GC pressure
- The hybrid architecture created the worst of both worlds: ECS complexity without
  full ECS performance
- GPU-heavy rendering pipeline (custom renderer, not Unity standard)
- Volumetric clouds, global illumination, advanced lighting
- These visual features were beautiful but crushed GPU performance

### 15.2 Simulation Scale Changes

CS2 aimed for larger, more detailed cities but struggled with performance:

**Entity limits (CS2 vs CS1):**
| Entity | CS1 Limit | CS2 Target | CS2 Actual Performance |
|--------|-----------|------------|----------------------|
| Citizens | 65,536 (active) | ~1,000,000 | ~100,000 before unplayable |
| Vehicles | 16,384 | ~100,000 | ~30,000 before performance death |
| Buildings | 49,152 | ~200,000 | ~50,000 realistically |
| Road segments | 36,864 | Unlimited | ~20,000 before issues |
| Trees | 250,000 | 1,000,000+ | Worked but GPU-heavy |

**Why CS2 performed so poorly:**
1. Every citizen was fully simulated (no "virtual" citizens like CS1)
2. Every vehicle was a persistent world entity (no pocket cars)
3. Pathfinding was more complex (parking search, lane changes)
4. The rendering pipeline was extremely heavy (volumetric everything)
5. LOD system was poorly implemented at launch -- distant buildings rendered
   at near-full detail
6. Memory usage: CS2 easily consumed 16-24 GB RAM, vs CS1's 4-8 GB
7. CPU-GPU synchronization issues caused stuttering regardless of hardware

### 15.3 Simulation Speed

CS1 had 3 simulation speeds (1x, 2x, 3x) that worked reliably. CS2's simulation speeds
were deeply problematic:

- 1x speed: Already below 30 FPS for many players at city sizes > 30k
- 2x speed: Simulation couldn't keep up, causing desync between visual and simulation state
- 3x speed: Essentially broken for months, causing simulation errors and crashes
- The "play at 1x or don't play" situation was a major complaint
- CS1 players routinely played at 3x speed for hours -- CS2 made this impossible

### 15.4 Map and Terrain

**CS1 map:**
- 9x9 tile grid, ~17 km x 17 km total area
- Heightmap resolution: 1081 x 1081 (each heightmap pixel = ~16m)
- Sea level fixed, rivers pre-baked in map editor
- Trees: Individual entities with growth simulation
- Terrain texturing: Basic splat map with ~4 terrain types

**CS2 map:**
- Larger initial area (~38 km x 38 km playable)
- Higher resolution heightmap
- Dynamic water simulation (rivers can flood, dam breaks)
- Seasonal visual changes (cosmetic, not gameplay-affecting at launch)
- Much more detailed terrain with procedural scattering
- Performance cost of the larger, more detailed map was significant

### 15.5 Save System

CS2's save system had notable issues:

- Save file sizes ballooned: 50-300 MB vs CS1's 20-80 MB
- Save times: 15-45 seconds for large cities (CS1: 3-10 seconds)
- Load times: 1-3 minutes (CS1: 15-45 seconds)
- Autosave caused noticeable gameplay hitches (simulation pause during save)
- Save corruption reports were common in the first months
- CS1's save system was battle-tested over 8 years; CS2's was effectively beta quality

### 15.6 UI and UX Changes

CS2 redesigned the entire UI, which received heavily mixed reviews:

**Improvements:**
- Info views overhauled with better visualization
- Road builder improved with guidelines and snapping
- Zoning preview shows what will grow
- Building placement preview
- Better underground infrastructure visualization

**Regressions:**
- Transit line management was significantly worse than CS1
- Budget screen less intuitive
- District painting tools less responsive
- Policy interface confusing (harder to find specific policies)
- Info overlays sometimes misleading (showing data that wasn't relevant)
- Many UI elements required more clicks to reach the same function
- No keyboard shortcuts for many common actions (CS1 had comprehensive hotkeys)
- The UI was designed for both PC and eventual console release, resulting in a
  "consolized" feel that PC players resented

---

## 16. CS2 Failures and Community Criticism

### 16.1 Launch State (October 2023)

Cities: Skylines 2 launched on October 24, 2023, to devastating community reception.
The game received "Mixed" reviews on Steam (dropping to ~60% positive), a shocking
result for the sequel to a beloved game that maintained "Very Positive" (~90%) for 8 years.

**The core problems at launch:**

1. **Performance**: The single biggest complaint. The game ran at 15-25 FPS on hardware
   that ran CS1 at 60+ FPS. No amount of settings adjustment helped significantly. Even
   players with RTX 4090 GPUs and top-tier CPUs reported sub-30 FPS at modest city sizes.
   The game shipped without recommended hardware specs being achievable on ANY consumer hardware.

2. **Economy broken**: As described in section 4.4. Cities went bankrupt. Buildings
   abandoned. The rent/upkeep calculations were fundamentally miscalibrated.

3. **Missing features**: Features present in CS1 base game were MISSING from CS2 at launch:
   - No custom assets/mods (the mod framework wasn't ready)
   - No map editor
   - No scenario editor
   - Limited transit options (fewer than CS1 with DLCs)
   - No bicycle infrastructure
   - No tram system
   - Limited unique buildings
   - No tourism district specialization

4. **Simulation bugs**:
   - Citizens teleporting
   - Pathfinding creating infinite loops
   - Service vehicles getting stuck permanently
   - Electricity grid disconnecting randomly
   - Water pressure calculation errors flooding buildings
   - Garbage trucks ignoring entire neighborhoods

5. **Visual issues**:
   - Teeth rendering on citizen models (infamously bad close-up faces)
   - LOD popping (buildings visibly changing detail level)
   - Texture streaming issues (blurry textures at medium distances)
   - Shadow artifacts
   - The game looked worse at medium/low settings than CS1 at maximum

### 16.2 The Modding Situation

This was perhaps the most strategically damaging failure. CS1's success was BUILT on modding.
The Steam Workshop had 700,000+ items for CS1. Many players only bought CS1 because of mods.

**CS2 modding problems:**
- Paradox announced "Paradox Mods" platform would replace Steam Workshop
  - Community immediately suspicious (Paradox controlling mod distribution)
  - The platform wasn't ready at launch
  - No asset import pipeline at launch
  - Code mods required a new framework (not compatible with CS1 mods)
  - Months of delay before basic modding tools were available
- The modding community was the lifeblood of CS1. Alienating modders was existential.
- Many prominent CS1 modders publicly stated they wouldn't develop for CS2
- As of mid-2024, the CS2 Workshop had ~5,000 items vs CS1's 700,000+

### 16.3 The "Beach Properties" Asset Pack Incident

In March 2024, Paradox released a paid cosmetic DLC ("Beach Properties") for CS2 -- while
the game was still fundamentally broken. The community reaction was volcanic:

- The asset pack cost $9.99 for ~20 building assets
- The base game still had massive performance issues
- Economy was still partially broken
- Basic features were still missing
- The community interpreted this as Paradox prioritizing monetization over fixing the game
- Steam reviews plummeted further
- Major content creators publicly condemned the decision
- Paradox eventually made the pack free as damage control
- This incident became a case study in how NOT to manage post-launch DLC

### 16.4 Comparison: What CS1 Had at the Same Point

6 months after launch comparisons were stark:

| Feature | CS1 (6 months post-launch) | CS2 (6 months post-launch) |
|---------|---------------------------|---------------------------|
| Performance | Stable 60 FPS | 20-30 FPS struggles |
| Workshop items | ~50,000 mods/assets | ~5,000 |
| Economy | Functional, balanced | Partially fixed, still issues |
| First DLC | After Dark (well-received) | Beach Properties (disaster) |
| Player reviews | Very Positive (89%+) | Mixed (~60%) |
| Active players | Growing month-over-month | Declining month-over-month |
| City size viable | 200,000+ population | ~50,000 before performance death |
| Simulation speed | All 3 speeds working | 3x broken, 2x unreliable |

### 16.5 Specific Technical Failures Discovered by Community

The CS2 modding/datamining community found numerous technical issues:

1. **Texture memory leak**: The game never freed certain texture allocations, meaning
   VRAM usage grew continuously during play sessions. A 2-hour session could consume
   2-3 GB more VRAM than a fresh start.

2. **Pathfinding recomputation**: CS2 recalculated certain paths every frame instead of
   caching them, wasting enormous CPU cycles.

3. **Entity iteration**: Some systems iterated ALL entities when they only needed a subset.
   For example, the happiness calculation iterated every citizen entity even for citizens
   in buildings that hadn't changed state.

4. **Draw call batching failure**: Despite using ECS, the renderer failed to properly
   batch draw calls for similar buildings. Each building was often a separate draw call.

5. **Garbage collection storms**: The managed C# portions created allocation pressure,
   triggering .NET GC collections that caused frame stuttering. This is exactly the
   problem that ECS/DOTS was supposed to solve, but incomplete adoption defeated the purpose.

6. **Simulation tick inconsistency**: The simulation tick rate wasn't fixed. Under
   heavy load, ticks would skip, causing simulation time to desynchronize with real time.
   This meant building construction, citizen aging, and economic ticks were unreliable.

### 16.6 What CS2 Got Right

Despite the problems, CS2 did make genuine improvements in some areas:

- **Road building tool**: More intuitive, better snapping, parallel/grid modes
- **Lane physics**: Vehicles actually use all lanes (solving CS1's #1 complaint)
- **Mixed-use zoning**: Long-requested feature, well-implemented conceptually
- **Visual quality** (when it ran): Beautiful lighting, weather, and atmosphere
- **Terrain tools**: Better landscaping and terrain modification
- **Signature buildings**: Interesting concept for unique growable architecture
- **Modular service buildings**: Upgradeable service buildings instead of replacement
- **Chirper removal**: The annoying Twitter-clone notification bird was gone (controversial
  -- some players missed it)

### 16.7 Lessons for Megacity

The CS2 debacle provides critical lessons:

1. **Performance is non-negotiable**: Players will accept simpler visuals for stable framerate.
   CS1 at 60 FPS beat CS2 at 20 FPS in player preference every time.
2. **Don't simulate what you can't afford**: CS2 tried to simulate everything at full
   fidelity. CS1's hybrid approach (mix of real and virtual citizens) was more sustainable.
3. **Economy must work at launch**: A broken economy ruins ALL other systems because
   players can't build cities large enough to engage with those systems.
4. **Modding support is existential**: For city builders, the mod community IS the
   long-tail revenue and player retention strategy.
5. **Don't ship paid DLC for a broken game**: Self-explanatory.
6. **Feature parity with predecessor**: Missing features that existed in the previous game
   (even DLC features) is perceived as regression, not "room for future DLC."

---

## 17. Modding Ecosystem Analysis

### 17.1 Overview of CS1 Modding

CS1's modding scene was one of the largest and most active in gaming history. At its peak:
- 700,000+ Steam Workshop items
- 2,000+ code mods (gameplay modifications)
- 600,000+ custom assets (buildings, props, vehicles, trees)
- Many mods had millions of subscribers
- Some modders became professional game developers or were hired by Colossal Order

The modding ecosystem reveals EXACTLY where the base game fell short. Every popular mod
represents a gap in the base game that players felt strongly enough about to seek a solution.

### 17.2 Traffic Manager: President Edition (TM:PE)

**Subscribers**: ~8 million (the most subscribed code mod)
**Purpose**: Complete traffic AI overhaul and manual traffic control

**What TM:PE adds (and what it reveals about base game gaps):**

1. **Manual traffic light configuration:**
   - Set green/red duration per phase
   - Create custom phase sequences
   - Set up dedicated left-turn phases
   - Enable right-turn-on-red
   - Create actuated signals (respond to traffic demand)
   - **Gap revealed**: CS1's auto-generated traffic lights are inflexible and inefficient.
     The base game has NO way to customize light timing.

2. **Lane connectors:**
   - Manually assign which incoming lanes connect to which outgoing lanes at intersections
   - Critical for complex interchanges
   - Example: On a 6-lane road approaching a highway on-ramp, you can assign the
     right 2 lanes to the ramp and left 4 lanes to continue straight
   - **Gap revealed**: CS1's automatic lane connections are frequently wrong at complex
     intersections, especially custom-built interchanges.

3. **Speed limits:**
   - Set speed limits per road segment (override road type defaults)
   - Useful for slowing traffic in residential areas or speeding up arterials
   - **Gap revealed**: CS1 has no per-segment speed control. All roads of the same type
     have the same speed.

4. **Vehicle restrictions:**
   - Ban specific vehicle types from road segments (no trucks on residential streets)
   - Create bus-only or emergency-only lanes
   - **Gap revealed**: CS1 only has the "Heavy Traffic Ban" policy at district level,
     which is too coarse for effective traffic management.

5. **Priority signs:**
   - Place yield signs, stop signs, priority road designations
   - Control right-of-way at unsignalized intersections
   - **Gap revealed**: CS1 auto-assigns priority based on road type with no player control.

6. **Dynamic Lane Selection (DLS):**
   - Vehicles consider lane congestion when choosing lanes
   - Spreads traffic across all available lanes
   - Configurable with aggression parameters
   - **Gap revealed**: This is the fix for CS1's famous single-lane traffic. The base
     game's lane selection algorithm is fundamentally broken.

7. **Parking AI:**
   - Improved parking behavior (CS1 has no parking simulation, but this mod adds basic
     parking search logic)
   - Can force citizens to park and walk
   - **Gap revealed**: CS1 completely ignores parking. Cars just disappear.

8. **Junction restrictions:**
   - Enable/disable U-turns at intersections
   - Allow/block pedestrian crossings at specific intersections
   - Enable/disable entering blocked junctions
   - **Gap revealed**: CS1 gives no control over intersection behavior beyond road type.

9. **Despawn control:**
   - Toggle vehicle despawning on/off
   - With despawn off, traffic problems become permanent until resolved
   - Most hardcore players disable despawning
   - **Gap revealed**: CS1 masks traffic problems by making stuck vehicles vanish.

10. **Timed traffic lights:**
    - Set up coordinated "green wave" timing across multiple intersections
    - Time lights so that vehicles hitting green at one intersection also hit green at the next
    - **Gap revealed**: No signal coordination exists in the base game.

**Performance impact of TM:PE:**
- TM:PE adds ~15-30% CPU overhead due to more complex traffic calculations
- The DLS system is the most expensive feature
- Large cities (200k+) with TM:PE can see significant FPS drops
- The mod is highly optimized (multiple rewrites over its history) but fundamentally
  does more work per vehicle per tick

### 17.3 Ploppable RICO (Revisited)

**Subscribers**: ~3 million
**Purpose**: Convert growable buildings into "ploppable" (manually placeable) buildings
and vice versa

**What RICO does:**
- "RICO" stands for Residential, Industrial, Commercial, Office
- Normally, buildings in these zones GROW automatically based on zone painting
- RICO lets you manually PLACE specific buildings like you place service buildings
- You choose exactly which building goes where, with exact positioning
- Buildings placed via RICO don't need to satisfy level-up requirements
- You can place a Level 5 high-rise in a low-value area

**What it reveals:**
- Players want precise control over their city's appearance
- The automatic growth system, while realistic, frustrates players who have a vision
  for their city's look
- The RNG of which building model spawns in a zone is a pain point
- City builders have two audiences: efficiency optimizers (who like auto-growth) and
  city artists (who want pixel-perfect control). CS1 base only served the first group.

**RICO settings per building:**
- Manually set household count, worker count, construction cost
- Override zone type (make a residential building ploppable in commercial zone)
- Set realistic or custom values
- Community-maintained "RICO settings" databases for popular Workshop assets

### 17.4 Move It!

**Subscribers**: ~5 million
**Purpose**: Move, rotate, and precisely position ANY placed object

**What Move It does:**
- Select any building, road node, segment, tree, prop, or decal
- Move it with pixel-precision in any direction (including vertical)
- Rotate to any angle (not just grid-aligned)
- Copy-paste objects
- Align objects to grid, to other objects, or to road edges
- Undo/redo
- Bulk selection and movement
- Height offset (raise/lower buildings from ground level)

**What it reveals:**
- CS1's placement system is extremely rigid:
  - Buildings snap to zone grid only
  - Roads snap to angle increments
  - No way to rotate buildings freely
  - No way to adjust position after placement (must bulldoze and re-place)
- City-builder players spend ENORMOUS time on aesthetic arrangement
- The inability to nudge things by a few pixels is maddening
- Move It is considered the single most essential quality-of-life mod

### 17.5 81 Tiles

**Subscribers**: ~5 million
**Purpose**: Unlock all 81 map tiles (vs base game's 9-tile limit)

**What it reveals:**
- The 9-tile limit was the most hated restriction in CS1
- Players wanted to build SPRAWLING cities, not compact ones
- The base game's limit was due to memory/performance concerns
- With 81 tiles, the playable area goes from ~36 km^2 to ~324 km^2
- Performance impact: Moderate (mainly memory, as most tiles are empty)
- This mod has more subscribers than many full games
- CS2 increased the default buildable area significantly but still had a tile limit

### 17.6 Loading Screen Mod (LSM)

**Subscribers**: ~4 million
**Purpose**: Optimize asset loading and reduce RAM usage

**What it does:**
- Replaces Unity's default asset loading pipeline
- Loads assets more efficiently (deduplicates shared textures)
- Skips loading unused assets (CS1 base loads ALL subscribed Workshop items)
- Shares textures between similar assets
- Provides loading progress bar with detailed info
- Reports asset errors and conflicts
- Can reduce RAM usage by 30-50% for heavily modded games

**What it reveals:**
- CS1's base asset loading was highly inefficient
- Players with 2,000+ Workshop assets (common for detailers) would run out of RAM
- The base game provided zero feedback during loading (just a spinner)
- Unity's asset management was a bottleneck that the game team didn't optimize
- This mod was so essential that Colossal Order acknowledged it should have been
  base game functionality

### 17.7 Network Extensions 2 (NExt2)

**Subscribers**: ~3 million
**Purpose**: Additional road types not in the base game

**Roads added:**
- 8-lane roads (base game maxes at 6)
- Single-lane one-way roads (critical for highway ramps)
- 2-lane highways (compact highway variant)
- National roads (high speed, no zoning)
- Small heavy roads (2-lane roads that allow heavy traffic without zone restrictions)
- Pedestrian roads (cars banned, pedestrian-priority)
- Alleys (very narrow, low-speed roads)
- Bus-only roads (entire road restricted to buses)

**What it reveals:**
- CS1's road selection, while extensive, missed critical variants
- Single-lane one-way roads are ESSENTIAL for building realistic highway interchanges
  and their absence was a notable oversight
- Players need more granularity in road types than the base game offers
- The highway system in particular lacked intermediate road types

### 17.8 Fine Road Tool / Fine Road Anarchy

**Subscribers**: ~2 million each
**Purpose**: Remove placement restrictions and enable precise road construction

**Fine Road Tool:**
- Place roads at specific heights (overpass, underpass, at grade)
- Adjust road slope precisely
- Enable/disable ground conforming
- Toggle between different road modes (ground, elevated, bridge, tunnel)
- Place roads at angles the base game doesn't allow

**Fine Road Anarchy:**
- Remove collision detection (allow overlapping roads)
- Allow road placement in restricted areas
- Build on slopes too steep for normal placement
- Bypass minimum segment length
- Ignore height restrictions

**What they reveal:**
- CS1's road building has extremely strict placement rules that prevent many
  realistic road configurations
- Building realistic highway interchanges is nearly impossible without these mods
- The "elevated road" system is too rigid (fixed heights, limited angles)
- Players need the ability to "break the rules" for creative road design
- The base game optimizes for ease of use at the cost of expressive power

### 17.9 Intersection Marking Tool (IMT)

**Subscribers**: ~1.5 million
**Purpose**: Paint road markings on intersections

**What it does:**
- Paint lane markings through intersections (the base game stops markings at intersection edge)
- Add crosswalk markings
- Paint stop lines, yield lines
- Custom colors and patterns for road markings
- Fillers (paint the intersection surface)

**What it reveals:**
- CS1's intersections are visually bare -- no lane markings, no guidance
- This is purely cosmetic but hugely popular, showing how much players care about visual detail
- The base game rendering of intersections is the weakest visual element

### 17.10 Realistic Population Revisited

**Subscribers**: ~1 million
**Purpose**: Fix unrealistic household/worker counts

**What it does:**
- Adjusts building population to realistic values
- A 20-story apartment tower might house 200-500 people (vanilla: ~50-80)
- A large office tower might employ 1000+ (vanilla: ~50)
- Scales all related systems (demand, services, traffic) accordingly
- Makes population count "realistic" relative to the visual city

**What it reveals:**
- CS1's displayed population is wildly unrealistic relative to building sizes
- The 65k agent limit forces the game to under-count building populations
- Players want their city's population to "make sense" visually
- This mod creates much higher demand for services and traffic,
  making the game harder but more engaging
- With this mod, a dense city of visually ~2 million people will report ~2 million
  instead of CS1's ~200,000

### 17.11 Other Notable Mods

**Prop & Tree Anarchy**: Remove placement restrictions for decorative props and trees.
Reveals: Base game is too strict about where you can place decorative elements.

**Surface Painter**: Paint terrain textures (grass, dirt, pavement) under/around buildings.
Reveals: Base game terrain painting is too coarse for detailed work.

**Theme Mixer**: Mix and match visual themes (buildings, terrain, atmosphere).
Reveals: Base game visual themes are all-or-nothing; players want to blend styles.

**Advanced Vehicle Options**: Control vehicle speed, capacity, and spawning per type.
Reveals: Base game gives no control over vehicle parameters.

**Real Time**: Makes the day/night cycle realistic length (24 real-time minutes = 1 game day).
Reveals: Base game's time compression is too aggressive for immersive play.

**Procedural Objects**: Scale, stretch, and deform any object.
Reveals: Base game assets are fixed-size, limiting creative expression.

**Extra Landscaping Tools**: Better terrain modification, tree placement, water source placement.
Reveals: Base game landscaping tools are limited.

**Roundabout Builder**: Auto-generate roundabouts with one click.
Reveals: The base game's inability to build roundabouts easily is a pain point.

### 17.12 The Mod Dependency Problem

CS1 modding had a dark side: mod dependency hell.

- Many mods depend on other mods (e.g., most detailing mods need Move It + Prop Anarchy)
- Mod updates could break dependent mods
- Subscribing to 100+ mods (common) created fragile load orders
- Game updates sometimes broke ALL mods simultaneously
  - This happened with major Unity version upgrades
  - The community would be unable to play for days/weeks while modders updated
- Some essential mods were maintained by single developers
  - When a modder quit, their mod became "abandoned"
  - Other modders would fork and maintain, but the transition caused instability
- The "Loading Screen Mod" itself exists because the base game can't handle heavy modding

**Implications for Megacity:**
- If modding is supported, it should have a stable API that doesn't break on updates
- Core QoL features (move objects, free placement) should be in the base game
- Asset loading should be designed for thousands of custom items from day one
- Mod conflict detection should be a built-in feature

---

## 18. Implementation Recommendations for Megacity

### 18.1 What CS1 Got Right (Adopt These)

These are proven design patterns from CS1 that should be adapted for Megacity:

1. **Simple but deep economy**: CS1's economy is easy to understand (tax + services = budget)
   but has depth through zone type interactions, supply chains, and feedback loops.
   Resist the temptation to make the economy "realistic" (see CS2's disaster).

2. **Service coverage model**: Radius-based coverage is intuitive and visually
   communicable. Players understand "build a fire station to cover this area."
   The key improvement: use road-network distance for vehicle dispatch, not Euclidean.

3. **Land value as the master variable**: CS1's system where land value drives building
   growth, which drives tax revenue, which enables services, which drives land value,
   creates a compelling feedback loop. This is the core gameplay loop.

4. **Progressive unlocks**: Gating content behind population milestones gives clear
   goals and prevents information overload. The pacing of CS1's unlocks (new tools
   every 2,000-5,000 population) keeps players engaged.

5. **District-based policies**: Applying rules to specific areas is more engaging than
   city-wide toggles. It creates optimization puzzles (industrial district with heavy
   traffic ban, residential with high-rise ban, etc.).

6. **Building leveling**: Visual and functional progression of buildings based on
   environmental conditions is satisfying and communicates city health clearly.

### 18.2 What CS1 Got Wrong (Avoid These)

1. **Lane selection algorithm**: Implement proper lane selection from day one. Vehicles
   should consider lane occupancy, not just upcoming turn direction. This was CS1's
   biggest long-term complaint and most impactful mod target.

2. **Euclidean dispatch**: Service vehicles should pathfind on the road network, not
   beeline to the nearest building. This is a simple algorithm change with massive
   gameplay improvement.

3. **Death wave susceptibility**: Randomize initial citizen ages on move-in. If all
   residents of a neighborhood are the same age, stagger them within a +-10% range.

4. **Over-education dead end**: Don't force educated citizens to refuse jobs. Instead,
   give them a happiness penalty for working below their education level but let them
   work. This preserves the incentive to match education to jobs without creating an
   unsolvable dilemma.

5. **Fixed placement**: From the start, allow rotation and micro-positioning of
   buildings. "Move It" being the most essential CS1 mod tells you everything about
   what the base game was missing.

6. **No parking simulation**: CS1's pocket cars are efficient but break immersion.
   CS2's full parking simulation was too expensive. The middle ground: designated
   parking zones per building that hold a fixed number of vehicles, visible but not
   individually pathfinding-simulated.

### 18.3 What CS2 Attempted That's Worth Doing Right

1. **Mixed-use zoning**: The concept is great. Implementation should be simple:
   a building serves both residential and commercial function, splitting its
   footprint. Tax revenue comes from both components.

2. **Persistent vehicles**: Rather than pocket cars, have vehicles that exist but use
   simplified parking (snap to building's parking allocation, no circling simulation).

3. **Lane system that works**: CS2 proved that agents CAN use all lanes. The implementation
   just needs to be performance-optimized (our ECS/Bevy architecture should help).

4. **Building upgrades**: Instead of demolish-and-rebuild for leveling, having buildings
   add floors or wings is visually better and avoids the "disappearing building" problem.

5. **Road maintenance**: Roads degrading over time adds a cost layer that's realistic
   and creates interesting budget decisions. Keep it simple: degradation rate per
   segment based on traffic volume, maintenance depot dispatches crews.

### 18.4 Critical Systems Priority Order

Based on the CS1/CS2 analysis, implementation priority should be:

**Tier 1 -- Core Loop (must work perfectly):**
1. Road placement and pathfinding (if traffic doesn't work, nothing works)
2. Zone growth and building leveling (the visual reward loop)
3. Basic economy (tax revenue, service costs, budget balance)
4. Water and electricity (binary requirements -- building either has them or doesn't)

**Tier 2 -- Engagement Systems:**
5. Service coverage (police, fire, health, education)
6. Land value calculation (the hidden variable that drives everything)
7. Public transit (bus + metro minimum viable set)
8. Districts and policies (player expression and optimization)

**Tier 3 -- Depth and Content:**
9. Production chains (industry specialization, supply logistics)
10. Citizen lifecycle (aging, education, demographics)
11. Pollution systems (ground, noise, water)
12. Tourism and leisure

**Tier 4 -- Polish and Expansion:**
13. Day/night cycle with gameplay effects
14. Weather and seasonal effects
15. Disasters (if desired)
16. Advanced transit (trains, monorail, ferry)
17. Modding support / asset pipeline

### 18.5 Performance Budgets (Learning from CS2's Failure)

Based on CS2's mistakes, Megacity should set hard performance targets:

- **Target**: 60 FPS at 100,000 population on mid-range hardware
- **Simulation tick budget**: 16ms maximum per tick at 1x speed
- **Pathfinding budget**: 2ms per tick for batch pathfinding operations
- **Agent count**: Scale virtual vs simulated agents dynamically:
  - 0-10k pop: 100% simulated agents
  - 10k-50k pop: 50% simulated, 50% virtual
  - 50k-200k pop: 25% simulated, 75% virtual
  - 200k+: 10% simulated, 90% virtual
- **Rendering budget**: LOD system must aggressively cull and simplify distant objects
- **Memory budget**: Target 4 GB for base game, 8 GB with heavy custom content
- **Save/load**: Target 3 seconds save, 10 seconds load for 200k city

These numbers are informed by CS1's proven performance at scale and CS2's failure to
maintain playable framerates. CS1 proved that players prefer smooth performance over
visual fidelity. The simulation can be deep without being computationally exhaustive.

### 18.6 Traffic System Specific Recommendations

Traffic is the make-or-break system for any city builder. Based on 8 years of CS1 TM:PE
development and CS2's improvements:

1. **Lane selection**: Cost function should include:
   - Distance to destination (base cost)
   - Current lane occupancy (critical -- CS1 lacked this)
   - Upcoming turn requirements (CS1's ONLY consideration)
   - Lane change cost (small penalty for each lane change needed)
   - Speed differential (fast lane vs slow lane)

2. **Intersection control**: From day one, support:
   - Traffic lights with configurable timing
   - Stop signs and yield signs
   - Roundabout detection (auto-configure yield-on-entry for circular road patterns)
   - Priority road designation

3. **Pathfinding**: Use a tiered approach:
   - Pre-computed highway-level routes (contraction hierarchy or similar)
   - Dynamic local routing for last-mile (A* on local road network)
   - Periodic recalculation (not every tick, but every N ticks or on road network change)
   - Traffic-aware cost function that updates with current congestion

4. **Vehicle despawning**: Never silently despawn vehicles. If a vehicle is stuck for
   >5 minutes, show a warning icon on the stuck location and add a happiness penalty
   to the source/destination buildings. Let the player see and fix the problem.

5. **Service vehicle priority**: Emergency vehicles should have lane-clearing behavior
   (other vehicles pull to the side) and intersection priority (lights go green).

### 18.7 Economy Design Recommendations

CS2's economic failure provides clear anti-patterns:

1. **Start simple**: Revenue = sum(tax_rate * building_income * level_multiplier) for each building.
   Expenses = sum(service_costs). Surplus = Revenue - Expenses.

2. **Tune conservatively**: Default taxes should produce slight surplus. Players should
   NOT go bankrupt at any city size unless they massively overspend on services.

3. **Tax sensitivity**: Make it forgiving. 9-12% should be consequence-free. 13-15% should
   cause minor complaints. Only 18%+ should cause abandonment. CS1's thresholds are a
   good starting point.

4. **Avoid rent simulation**: CS2's rent model was a trap. Rent = upkeep + land_value_tax
   created death spirals because the formula could produce rents exceeding income.
   Just use a simple building-level system where buildings either meet requirements or don't.

5. **Budget feedback**: Make it IMMEDIATELY OBVIOUS when the budget is trending toward crisis.
   CS2 let players accumulate debt slowly until sudden cascading bankruptcy. Show projected
   budget 5 game-years forward.

### 18.8 Visual Building Growth Recommendations

The building growth system is the primary visual reward of city builders:

1. **Deterministic building selection**: Given the same zone type, lot size, and level,
   always produce the same building (within a set). This lets players predict and plan.
   CS1's random building selection frustrated city designers.

2. **Building families**: Create "families" of buildings that share a visual style.
   When a neighborhood grows, adjacent buildings should come from the same family,
   creating cohesive block aesthetics.

3. **Growth animation**: Buildings should visibly "construct" (scaffolding -> framing
   -> facade) rather than popping into existence. This is a small detail that adds
   enormous polish.

4. **Level-up visual transition**: When a building levels up, it should add floors/wings
   (CS2 approach) rather than being demolished and rebuilt (CS1 approach). Show the
   neighborhood improving over time, not constantly being rebuilt.

5. **Lot flexibility**: Support irregular lot shapes at road curves and intersections.
   CS1's rectangular-only lots created obvious grid artifacts. Even slight angle
   tolerance (allowing 5-10 degree rotation to match road curve) helps enormously.

### 18.9 Summary: The Megacity Competitive Advantage

Based on this analysis, Megacity's competitive advantages over CS1/CS2 should be:

1. **Performance first**: Bevy's ECS architecture, combined with a disciplined simulation
   budget, should deliver CS1-level performance with deeper simulation.

2. **Traffic that works**: Proper lane selection, configurable intersections, and
   transparent pathfinding (show WHY agents take specific routes) from day one.

3. **Stable economy**: Simple, tunable, never-death-spiral economy with clear feedback.

4. **Flexible placement**: Move, rotate, and adjust buildings from the start. No mod needed.

5. **Scalable agent simulation**: The LOD/virtual citizen hybrid approach, already partially
   implemented, is the proven architecture. CS1 proved it works at 500k+ displayed pop.

6. **Mod-friendly architecture**: If modding is planned, design the asset pipeline and
   simulation API for it from the beginning, not as an afterthought.

The city builder genre is surprisingly underserved despite CS1's massive success. CS2's
failure created a market opportunity. The community is actively looking for alternatives
that deliver what CS2 promised: deep simulation at playable framerates.
