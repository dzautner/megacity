# Megacity: Master Architecture Document

## Purpose

This document is the definitive roadmap for Megacity. It synthesizes all 18 research documents into an actionable development plan, maps every proposed feature against the existing codebase, defines dependency ordering, assigns priority tiers, and establishes concrete milestones. Every system in the game -- existing or planned -- is catalogued here with its current state, target state, dependencies, and estimated effort.

The 18 source research documents are:

1. `infrastructure_engineering.md` -- Road engineering, transit operations, water/sewer/power realities
2. `game_design_mechanics.md` -- Progression, unlocks, challenge modes, player expression, QoL
3. `community_wishlists.md` -- 20 categories of player requests from CS1/CS2/SimCity communities
4. `indie_competitors.md` -- Unique mechanics from Workers & Resources, Tropico, Foundation, Anno, etc.
5. `cities_skylines_analysis.md` -- Deep CS1/CS2 mechanics analysis with specific numbers and formulas
6. `economic_simulation.md` -- Municipal finance, property tax, labor markets, economic cycles
7. `transportation_simulation.md` -- Traffic flow math, BPR functions, intersection capacity, freight
8. `urban_planning_zoning.md` -- Zoning paradigms, form-based codes, building growth, walkability
9. `environment_climate.md` -- Pollution (4 types), water systems, energy grid, weather, disasters, waste
10. `historical_demographics_services.md` -- City growth patterns, demographics, civic service operations
11. `social_agent_simulation.md` -- Agent demographics, life stages, happiness, crime, health, governance
12. `camera_controls_ux.md` -- Camera system, overlays, selection, tool UX, accessibility
13. `underground_infrastructure.md` -- Water/sewer pipes, power distribution, metro, underground view
14. `endgame_replayability.md` -- Late-game challenges, mega-projects, scenarios, roguelite elements
15. `modding_architecture.md` -- Plugin system, scripting (Lua/WASM), asset pipeline, data-driven design
16. `save_system_architecture.md` -- ECS serialization, versioning, entity remapping, performance
17. `procedural_terrain.md` -- Heightmap generation, erosion, water bodies, biomes, resources
18. `sound_design.md` -- Spatial audio, dynamic music, environmental audio, UI sounds

---

## Table of Contents

1. [System Inventory](#1-system-inventory)
2. [Dependency Graph](#2-dependency-graph)
3. [Priority Tiers](#3-priority-tiers)
4. [Milestone Definitions](#4-milestone-definitions)
5. [Codebase Audit: What Exists vs What Is Needed](#5-codebase-audit)
6. [Cross-System Integration Points](#6-cross-system-integration-points)
7. [Risk Assessment](#7-risk-assessment)
8. [Appendix A: Module-Level Status Matrix](#appendix-a-module-level-status-matrix)
9. [Appendix B: Research Document Coverage Map](#appendix-b-research-document-coverage-map)

---

## 1. System Inventory

Every distinct system or feature across all 18 research documents, grouped into 20 major systems. For each system, the inventory lists what currently exists in the codebase and what the research documents propose as the target.

---

### 1.1 Core Grid and Terrain

**What exists in the codebase:**

- `grid.rs`: 256x256 `WorldGrid` with `Cell` struct containing `elevation: f32`, `cell_type: CellType` (Grass/Water/Road), `zone: ZoneType`, `road_type: Option<RoadType>`, `building_id: Option<Entity>`, and utility flags (`has_power`, `has_water`). Grid-to-world conversion at `CELL_SIZE=16.0`.
- `terrain.rs`: Single-pass OpenSimplex2 noise via `fastnoise-lite` at frequency 0.008, normalized to [0,1], threshold at 0.35 for water. No fractal layering, no erosion, no biomes.
- `config.rs`: Constants `GRID_WIDTH=256`, `GRID_HEIGHT=256`, `CELL_SIZE=16.0`, `CHUNK_SIZE=8`.
- `terrain_render.rs`: Chunk-based flat mesh rendering (y=0.0 for all vertices), vertex-colored by cell type and overlay state. 32x32 chunks (1024 total). Dirty-flag system for selective rebuild.
- `natural_resources.rs`: `ResourceGrid` with hash-based placement keyed on elevation bands. Resources: Coal, Oil, OreDeposit, FertileSoil, Forest. `ResourceBalance` tracks extraction vs reserves.
- `lib.rs` (init): Hardcoded Tel Aviv terrain with custom coastline function, Yarkon River, and neighborhood-based elevation.

**What the research documents propose (from `procedural_terrain.md`):**

- Multi-octave fractal Brownian motion (fBm) with 4-8 octaves for natural-looking heightmaps
- Domain warping for organic terrain shapes
- Height distribution shaping (power curves, plateau injection)
- Particle-based hydraulic erosion simulation for realistic river valleys
- Thermal erosion for cliff and talus slope formation
- Procedural river placement with width variation and meanders
- Lake detection via flood-fill from local minima
- Ocean/coastline generation with beaches and cliffs
- Whittaker-diagram biome classification based on temperature and moisture
- Vegetation density per biome
- Seed-based generation with playability guarantees (flat buildable area, water access, resources)
- Soil type grid affecting agriculture and construction cost
- Flood plains and earthquake fault lines as terrain features
- Terrain modification by player (flatten, raise, lower, create canals)
- Heightmap-aware terrain rendering (vertex Y from elevation, normal-based lighting)
- LOD for terrain chunks at different zoom levels

**Gap analysis:**

The current terrain is either hardcoded (Tel Aviv) or single-pass noise. The gap to the target is substantial: fBm layering, erosion simulation, procedural water bodies, biome classification, terrain modification tools, and 3D terrain rendering (currently flat at y=0). However, the core grid infrastructure (`WorldGrid`, chunk system, dirty flags) is solid and extensible. The terrain system is a foundation-level concern but can be incrementally improved -- the current flat terrain is playable, just not visually or geologically interesting.

---

### 1.2 Road Network and Segments

**What exists in the codebase:**

- `road_segments.rs`: `RoadSegmentStore` with cubic Bezier curves. Each `RoadSegment` has control points (p0-p3), `RoadType`, arc length, and rasterized cell list. `SegmentNode` / `SegmentNodeId` for intersection topology. Methods: `add_segment`, `add_straight_segment`, `find_or_create_node`, `rasterize` (writes to grid and `RoadNetwork`).
- `roads.rs`: `RoadNetwork` as adjacency list graph (`HashMap<RoadNode, Vec<(RoadNode, f32)>>`). Grid-cell-based nodes. Used as fallback when CSR graph is not available.
- `road_graph_csr.rs`: Compressed Sparse Row graph built from `RoadNetwork`. A* pathfinding with `csr_find_path` and traffic-aware variant `csr_find_path_with_traffic`. Rebuilt on `RoadNetwork` change detection.
- `grid.rs`: `RoadType` enum with 6 types: Local (2-lane, 30 km/h), Avenue (4-lane, 50), Boulevard (6-lane, 60), Highway (4-lane divided, 100), OneWay (2-lane, 40), Path (pedestrian, 5). Each type defines speed, cost, lane count, width, zoning eligibility, vehicle allowance, half-width for rendering.
- `road_render.rs`: Mesh generation for road segments with width from `RoadType::half_width()`.
- `road_maintenance.rs`: `RoadConditionGrid` (u8 per cell, 0-100 PCI), degradation based on traffic, repair system, maintenance budget and stats.
- `traffic.rs`: `TrafficGrid` (u16 per cell), updated from citizen movement positions.
- `traffic_accidents.rs`: `AccidentTracker`, spawn/process accident events based on traffic density.
- `contraction_hierarchy.rs`: Dead code -- module exists but is never used.

**What the research documents propose (from `infrastructure_engineering.md`, `transportation_simulation.md`, `community_wishlists.md`):**

- Road hierarchy enforcement (local-collector-arterial-highway) with bottleneck detection
- Level of Service (LOS A-F) system based on volume-to-capacity ratio, with visual and functional changes per tier
- BPR function for travel time calculation: `t = t_0 * (1 + alpha * (V/C)^beta)`
- Induced demand modeling (highway expansion provides temporary relief, then fills back up)
- Braess's Paradox detection (adding roads can worsen traffic)
- Signalized intersection capacity modeling (1600-1900 veh/hr green/lane)
- Roundabout builder tool, interchange templates
- Modular road designer (mix lanes: car, bus, bike, parking, median, trees)
- Road anarchy (remove slope limitations), Move It equivalent (reposition placed roads)
- Proper merging behavior at highway on-ramps
- Lane-level simulation (vehicles use all lanes, not just shortest-path lane)
- Speed-flow curves per road type
- Undo button for road placement
- Grid snapping improvements
- Non-linear pavement degradation (S-curve: years 0-10 slow, 10-15 accelerating, 15-20 rapid failure)
- The "1-to-6 rule" for maintenance economics

**Gap analysis:**

The road system is one of the most complete subsystems. Bezier segments, CSR pathfinding, traffic density, road maintenance, and accident systems all exist and are functional. Key gaps: no LOS system (traffic density is raw u16, not mapped to A-F grades), no BPR function for travel times (pathfinding uses static edge weights), no induced demand modeling, no lane-level simulation, no modular road designer, no undo system, and the contraction hierarchy is dead code. The road maintenance degradation model appears to be linear rather than the S-curve described in research. Road building UX tools (roundabout builder, interchange templates) do not exist.

---

### 1.3 Zoning and Land Use

**What exists in the codebase:**

- `grid.rs`: `ZoneType` enum with 7 values: None, ResidentialLow, ResidentialHigh, CommercialLow, CommercialHigh, Industrial, Office. Zone stored per cell on the grid.
- `zones.rs`: `ZoneDemand` resource tracking demand for each zone type. `update_zone_demand` system calculates demand based on population ratios and employment.
- `lib.rs` (init): `apply_zones` function assigns zones based on proximity to roads (manhattan distance 5) and neighborhood-based rules (Jaffa, White City, coastal strip, etc.).
- `input.rs`: Zone painting tool -- player can paint zones on cells adjacent to roads.

**What the research documents propose (from `urban_planning_zoning.md`, `cities_skylines_analysis.md`, `community_wishlists.md`):**

- Euclidean (cumulative hierarchy) zoning where C-1 zones can contain houses
- Form-based codes as an alternative/overlay system (regulate building form, not use)
- Japanese 12-zone system as inspiration for nuanced zone types
- Houston "no zoning" mode as a game variant
- Mixed-use zoning (commercial ground floor + residential above)
- ResidentialMedium density tier (missing from current enum)
- FAR (Floor Area Ratio) controls per zone, with bonuses and transfers
- Parking minimums/maximums as zone parameters
- Setback, height limit, and lot coverage controls
- NIMBY/YIMBY mechanic (citizen opposition to rezoning)
- Eminent domain events
- Historic preservation districts
- Urban growth boundaries
- Inclusionary zoning requirements
- Transit-oriented development (TOD) zone overlays
- 15-minute city scoring algorithm
- Walk Score methodology
- Neighborhood quality index
- Zone demand driven by market ROI model (developers build what is profitable, not just what is demanded)

**Gap analysis:**

The current system is basic Euclidean exclusive zoning with 6 zone types. It works but lacks mixed-use, form-based codes, FAR controls, and the rich urban planning mechanics described in the research. The zone demand system is functional but simplistic (population ratios). Missing: ResidentialMedium, mixed-use type, form-based overlay, walkability scoring, NIMBY mechanics, and market-driven development logic. The zone painting tool works but has no undo, no district-level zone policies, and no visual feedback for zone eligibility beyond adjacency.

---

### 1.4 Building Growth and Development

**What exists in the codebase:**

- `buildings.rs`: `Building` component with `zone_type`, `level` (u8, 1-3), `grid_x`, `grid_y`, `capacity`, `occupants`. `BuildingSpawnTimer` for throttled spawning. `building_spawner` system iterates grid per zone type, finds eligible cells, spawns buildings. `progress_construction` handles construction delay.
- `building_upgrade.rs`: `UpgradeTimer` for throttled upgrades. `upgrade_buildings` checks land value thresholds (>120 for level 2, >180 for level 3) and service coverage. `downgrade_buildings` checks for missing utilities, low happiness.
- `abandonment.rs`: `check_building_abandonment` marks buildings as abandoned when they have no power/water, very low happiness, or no occupants for extended time. `process_abandoned_buildings` despawns after grace period. `abandoned_land_value_penalty` reduces nearby land value.
- `building_meshes.rs`: Procedural mesh generation for buildings based on zone type and level.
- `building_render.rs`: Mesh spawning, updating, construction visuals (scaled-up during construction), orphan cleanup. Planted tree mesh management.

**What the research documents propose (from `urban_planning_zoning.md`, `cities_skylines_analysis.md`):**

- 5-level building system (CS1 style) instead of current 3 levels
- Level-up requirements combining land value + service coverage + education + pollution thresholds
- Market-driven growth: developer ROI model where buildings grow based on expected profitability
- Building variety generation (procedural facade variation, corner lot buildings)
- Demolition and replacement logic (old buildings torn down for denser ones when land value rises)
- Multi-cell buildings (2x2, 3x3, 4x4 plots for large buildings)
- Construction material requirements (buildings need delivered materials to construct)
- Building age and depreciation affecting quality and value
- Over-education problem handling (industry needs uneducated workers)
- Special/unique buildings as milestone rewards
- Landmark buildings with city-wide effects

**Gap analysis:**

The building system is functional with 3 levels, construction delay, upgrade/downgrade, and abandonment. Key gaps: only 3 levels instead of 5, no multi-cell buildings (everything is 1x1), no building variety (same mesh per type/level), no construction material requirements, no building age/depreciation, no market-driven growth model. The building spawner iterates the full grid per zone type, which is a known performance issue that could be solved with an eligible-cell list.

---

### 1.5 Citizen Lifecycle and Agent Simulation

**What exists in the codebase:**

- `citizen.rs`: `Citizen` marker component. `CitizenDetails` with age (u8), gender, education (0-3), happiness, health, salary, savings. `Personality` with ambition, sociability, materialism, resilience (all f32 0-1). `Needs` component. `Family` component. `CitizenState` enum with 10 states (AtHome, CommutingToWork, Working, etc.). `LifeStage` enum (Child through Retired). `Position`, `Velocity`, `PathCache`, `HomeLocation`, `WorkLocation`.
- `citizen_spawner.rs`: Spawns new citizens when residential buildings have capacity and zone demand exists.
- `lifecycle.rs`: `age_citizens` ages citizens over time, handles death at old age. `emigration` removes unhappy citizens.
- `life_simulation.rs`: `update_needs`, `education_advancement`, `salary_payment`, `job_seeking`, `life_events` (marriage, children, divorce), `retire_workers`, `evolve_personality`, `update_health`.
- `education_jobs.rs`: `job_matching` assigns unemployed citizens to workplaces. `assign_workplace_details` handles detailed job assignment. `EmploymentStats` tracking.
- `movement.rs`: `citizen_state_machine` drives daily schedule (home -> work -> shop -> leisure -> home). `DestinationCache` for shops/leisure/schools. `process_path_requests` runs A* pathfinding. `move_citizens` interpolates position along paths.
- `happiness.rs`: `ServiceCoverageGrid` (bitflags per cell). `update_happiness` combines housing quality, commute, services, environment, economy, safety into happiness score.
- `homelessness.rs`: `Homeless` component, shelter-seeking, recovery systems.
- `welfare.rs`: `WelfareStats`, welfare payment processing.
- `immigration.rs`: `CityAttractiveness` score, `ImmigrationStats`, `immigration_wave` spawns new citizens based on attractiveness.
- `virtual_population.rs`: `VirtualPopulation` for scaling beyond real entity count. `adjust_real_citizen_cap` manages the ratio.
- `lod.rs`: Three-tier LOD (Full/Simplified/Abstract) based on viewport distance. Compress/decompress systems for off-screen citizens.

**What the research documents propose (from `social_agent_simulation.md`, `historical_demographics_services.md`):**

- Extended demographics: ethnicity, religion, income class, occupation, marital status, household size, dependents, net worth, debt, rent burden
- 6-level education (None through Doctorate) instead of current 0-3
- Income follows log-normal distribution with Gini coefficient tracking
- Schelling segregation model for neighborhood demographics
- Daily schedule generation based on demographics and personality
- Decision-making models (utility maximization for location choice, job choice, mode choice)
- Social mobility tracking (income quintile movement over time)
- Crime simulation with full pipeline (poverty -> crime rate -> policing -> courts -> prison)
- Disease and epidemic modeling
- Detailed immigration/emigration with push/pull factor analysis
- Political opinions per citizen, voting behavior, faction alignment
- Social network modeling (friends, neighbors, coworkers influence opinions)
- Household formation and dissolution (marriage markets, roommates)
- Aging population challenges (pension burden, healthcare demand)
- 100K+ agent performance patterns (archetype-based ECS optimization)

**Gap analysis:**

The citizen system is the most developed subsystem in the codebase. It has a working state machine, pathfinding, needs system, life events, education, job matching, happiness, homelessness, welfare, immigration, and LOD. The virtual population system allows scaling. Key gaps: demographics are shallow (no ethnicity, religion, income class, occupation detail), education is 0-3 not 0-5, no Schelling segregation, no political opinions/voting, no social networks, no disease modeling, no daily schedule generation (the state machine is deterministic, not schedule-driven). The `LifeSimTimer` is not serialized, causing all life events to fire on load.

---

### 1.6 Traffic and Pathfinding

**What exists in the codebase:**

- `road_graph_csr.rs`: CSR graph built from `RoadNetwork`. A* with `csr_find_path` and traffic-aware `csr_find_path_with_traffic`. Edge weights incorporate road speed and traffic density.
- `pathfinding_sys.rs`: `nearest_road_grid` finds closest road cell to a building.
- `movement.rs`: `process_path_requests` runs up to `MAX_PATHS_PER_TICK=64` A* queries per tick. Citizens follow paths as waypoint lists.
- `traffic.rs`: `TrafficGrid` (u16 per cell), updated by aggregating citizen positions on road cells.
- `traffic_accidents.rs`: Accident spawning based on traffic density, processing with response time from fire stations.
- `road_maintenance.rs`: Road condition degrades with traffic, affects travel speed.

**What the research documents propose (from `transportation_simulation.md`, `infrastructure_engineering.md`):**

- Fundamental diagram of traffic flow (q = k * v relationship)
- BPR function for travel time: `t = t_0 * (1 + 0.15 * (V/C)^4)`
- Greenshields/Drake speed-density models
- Static traffic assignment (user equilibrium via Frank-Wolfe algorithm)
- Dynamic traffic assignment for time-varying demand
- Intersection capacity modeling with signal timing
- Turn penalties and turn prohibition support
- Freight movement (goods trucks on road network)
- Parking simulation (parking search time, garage vs street parking)
- Mode choice model (car vs transit vs walk vs bike)
- Vehicle type differentiation (trucks slower than cars)
- Congestion propagation (spillback from saturated links)
- Time-of-day traffic variation (AM/PM peak, off-peak)
- Origin-destination matrix computation

**Gap analysis:**

The pathfinding system is functional and performant (CSR A*, 64 queries/tick cap). Traffic density is tracked but not used for dynamic travel time calculation (no BPR function). There is no intersection modeling, no freight movement, no mode choice, no parking simulation, no congestion propagation. The traffic-aware pathfinding variant exists but uses a simple density penalty rather than the BPR function. The system handles the current scale (10K-50K citizens) but would need optimization for 100K+.

---

### 1.7 Public Transit

**What exists in the codebase:**

- `services.rs`: `ServiceType` includes BusDepot, TrainStation, SubwayStation, TramDepot, FerryPier, SmallAirstrip, RegionalAirport, InternationalAirport. These exist as service buildings with coverage radii.
- `airport.rs`: `AirportStats`, `update_airports` system tied to tourism.
- No transit line system, no transit vehicles, no route planning, no ridership modeling.

**What the research documents propose (from `infrastructure_engineering.md`, `transportation_simulation.md`, `community_wishlists.md`):**

- Bus system: routes drawn on road network, vehicles follow routes at set headways, citizens choose bus if faster than driving
- Tram/light rail: dedicated track or shared road, fixed stations, higher capacity than buses
- Metro/subway: underground rail with stations, multi-depth tunnels, high capacity
- Commuter rail: connects to outside connections, serves suburban areas
- Ferry: water-based routes connecting separated land areas
- Hierarchical transit networks: buses feed trams, trams feed metro, metro feeds rail
- Transfer hubs with inter-modal connections
- Transit line profitability and ridership tracking
- Timetable support (departure frequency/times)
- Express/local service on same line
- Transit-oriented development bonuses
- Ridership-based demand (citizens choose transit based on travel time comparison with driving)
- Transit priority (bus lanes, signal priority)

**Gap analysis:**

This is a major gap. Transit service buildings exist (bus depots, train stations, subway stations) but they function only as coverage-radius service buildings, not as actual transit infrastructure. There are no transit lines, no transit vehicles, no route planning tools, no ridership simulation. Building a functional transit system is a T2 priority and depends on the road network, citizen movement system, and economy.

---

### 1.8 Economy and Budget

**What exists in the codebase:**

- `economy.rs`: `CityBudget` with treasury, tax_rate (single flat rate), monthly income/expenses. `collect_taxes` system: per-citizen flat tax + commercial building income + land value bonus + tourism income. Service expenses based on building count.
- `budget.rs`: `ExtendedBudget` with `ZoneTaxRates` (separate rates per zone type), `ServiceBudgets` (per-service spending), historical monthly data (last 12 months).
- `loans.rs`: `LoanBook` with active loans, `process_loan_payments`, `update_credit_rating`, `BankruptcyEvent`.
- `market.rs`: `MarketPrices` with commodity prices and market events. `update_market_prices` with supply/demand oscillation.
- `production.rs`: `CityGoods` tracking produced/consumed goods. `assign_industry_type` and `update_production_chains` for basic industry simulation.
- `imports_exports.rs`: `TradeConnections` for import/export with outside world.
- `wealth.rs`: `WealthStats` with Gini coefficient, wealth tiers, income distribution tracking.
- `specialization.rs`: `CitySpecializations` and `SpecializationBonuses` for city economic focus.
- `outside_connections.rs`: `OutsideConnections` for highway/rail/sea/air trade links.

**What the research documents propose (from `economic_simulation.md`):**

- Property tax on assessed building value (not per-citizen flat tax)
- Millage rate system with separate rates for residential, commercial, industrial
- Assessment ratios (fraction of market value taxed)
- Sales tax, income tax as additional revenue sources
- Tax increment financing (TIF) districts
- Development impact fees
- User fees for utilities (water, sewer, power bills)
- Bond issuance for capital projects
- Economic cycles (boom/bust with 7-10 year periodicity)
- Labor market with wage determination based on supply/demand
- Commercial demand driven by population spending power
- Industrial demand driven by trade connections and resource availability
- Office demand driven by education level of workforce
- Inflation modeling
- Real estate market (housing prices based on supply/demand/land value)
- Rent calculation per building
- Business profitability simulation (revenue - costs - taxes = profit, negative = closure)
- Supply chain economics (input costs, output prices, margins)

**Gap analysis:**

The economy system is moderately developed. It has zone-specific tax rates, loans with credit ratings, market prices, production chains, trade connections, and wealth tracking. Key gaps: tax is still partially per-citizen rather than property-based, no sales/income tax, no TIF districts, no bond issuance, no economic cycles, no labor market wage determination, no rent calculation, no business profitability simulation, no inflation. The transition from per-citizen flat tax to property tax is a high-priority change described in detail in the economic simulation research doc.

---

### 1.9 Land Value

**What exists in the codebase:**

- `land_value.rs`: `LandValueGrid` (u8 per cell, 0-255, default 50). `update_land_value` runs on slow timer, resets to base 50, then adds/subtracts based on: water proximity (+20), service coverage (+5-15 per service type), road type bonus (+5-15), zone type modifier, pollution penalty (-1 per pollution unit), building level bonus.

**What the research documents propose (from `economic_simulation.md`, `cities_skylines_analysis.md`, `urban_planning_zoning.md`):**

- Land value as the central integrating variable (everything feeds into it, everything reads from it)
- Hedonic pricing model: land value = f(accessibility, amenities, negative externalities)
- Accessibility component: distance to jobs, transit stations, highway ramps
- Amenity component: parks, schools, cultural facilities, views (water, skyline)
- Negative externality component: pollution, noise, crime, industrial adjacency
- Neighborhood spillover effects (high-value buildings raise neighbors, blight lowers them)
- Land value determines building level-up potential, rent, property tax revenue
- Land value visualization as key planning overlay
- Historical tracking of land value changes over time
- Land speculation (vacant land near planned infrastructure increases in value)
- View corridors (buildings with water/park views get bonus)

**Gap analysis:**

The land value system exists and runs but is simplistic. It resets to 50 every cycle and recalculates from scratch rather than using incremental updates or diffusion. Missing: accessibility component (no distance-to-jobs calculation), no view corridors, no neighborhood spillover diffusion, no historical tracking, no speculation mechanics. The u8 range (0-255) may be too coarse for fine-grained property tax calculations. The system needs to become the central integrating hub that connects services, transport, environment, and economy.

---

### 1.10 Services (Police, Fire, Health, Education, Garbage, Parks, Death Care, Postal, Heating)

**What exists in the codebase:**

- `services.rs`: `ServiceBuilding` component with `service_type`, `grid_x`, `grid_y`, `radius`. `ServiceType` enum with 40+ types spanning fire, police, health, education, parks, garbage, civic, transport, telecom, welfare, postal, heating, water treatment. `coverage_radius` method returns radius per type.
- `happiness.rs`: `ServiceCoverageGrid` with bitflags per cell (health, education, police, park, entertainment, telecom, transport, fire). `update_service_coverage` marks cells within each service building's radius.
- `fire.rs`: `FireGrid` tracking active fires per cell. `start_random_fires`, `spread_fire`, `extinguish_fires`, `fire_damage` system chain. Fire stations reduce fire spread in coverage area.
- `forest_fire.rs`: `ForestFireGrid` and `ForestFireStats` for wildfires in tree areas.
- `crime.rs`: `CrimeGrid` (u8 per cell). `update_crime` generates crime from low land value, high density, low police coverage. Crime reduces happiness and land value.
- `health.rs`: `HealthGrid` (u8 per cell). `update_health_grid` based on hospital coverage, pollution, pollution penalties.
- `education.rs`: `EducationGrid`, `propagate_education` spreads education coverage from school buildings.
- `garbage.rs`: `GarbageGrid` (u8 per cell). `update_garbage` generates garbage from buildings, reduced by landfill/recycling coverage.
- `death_care.rs`: `DeathCareGrid` and `DeathCareStats`. `death_care_processing` handles deceased citizens, cemetery/crematorium capacity.
- `postal.rs`: `PostalCoverage` and `PostalStats`. `update_postal_coverage` from post offices.
- `heating.rs`: `HeatingGrid` and `HeatingStats`. `update_heating` based on heating buildings and weather temperature.
- `noise.rs`: `NoisePollutionGrid`. `update_noise_pollution` from roads, industry, airports.
- `groundwater.rs`: `GroundwaterGrid`, `WaterQualityGrid`, `GroundwaterStats`. `update_groundwater` models water table, `groundwater_health_penalty` for contamination.
- `water_pollution.rs`: `WaterPollutionGrid`. `update_water_pollution` from sewage, industry. `water_pollution_health_penalty`.

**What the research documents propose (from `historical_demographics_services.md`, `environment_climate.md`, `cities_skylines_analysis.md`):**

- Multi-tier service buildings (fire house -> fire station -> fire HQ) with different capacities and radii
- Service vehicle dispatch simulation (fire trucks, ambulances, police cars travel on road network)
- Response time calculation based on road distance and traffic conditions
- Service capacity vs demand (hospitals have bed counts, schools have student caps)
- Service quality degradation when over-capacity
- Disease/epidemic system with hospital surge capacity
- Education pipeline: kindergarten -> elementary -> high school -> university, with graduation rates
- Education affects job eligibility and salary (already partially implemented)
- Garbage truck routing and collection schedules
- Waste management hierarchy: reduce, reuse, recycle, landfill, incinerate
- Cemetery capacity and expansion
- Library and museum happiness bonuses
- Sports facilities and stadiums for large-event simulation
- Telecom infrastructure (cell towers, data centers) for modern city requirements

**Gap analysis:**

Services are broadly implemented -- the `ServiceType` enum has 40+ types, coverage grids exist for all major service categories, and per-cell effects are computed. However, all services operate on a simple coverage-radius model with no dispatch simulation, no capacity limits, no vehicle routing. Fire is the most detailed (spread, extinguish, damage) but even it lacks fire truck dispatch. The multi-tier building variants exist in the enum but may not all be functionally differentiated. Education propagation exists but the pipeline (graduation rates, capacity limits per school) is underdeveloped.

---

### 1.11 Environment (Pollution, Weather, Climate, Trees)

**What exists in the codebase:**

- `pollution.rs`: `PollutionGrid` (u8 per cell). `update_pollution` generates pollution from industrial buildings and traffic, with diffusion to neighbors.
- `noise.rs`: `NoisePollutionGrid`. `update_noise_pollution` from roads (based on road type), industry, airports.
- `water_pollution.rs`: `WaterPollutionGrid`. Pollution from sewage and industry, health penalties.
- `weather.rs`: `Weather` resource with temperature, precipitation, wind_speed, wind_direction, season. `update_weather` cycles through seasons with temperature variation.
- `wind.rs`: `WindState` resource. `update_wind` for wind direction and speed changes.
- `trees.rs`: `TreeGrid` (bool per cell). `tree_effects` provides pollution reduction and land value boost near trees.
- `natural_resources.rs`: `ResourceGrid` with resource deposits. `update_resource_production` tracks extraction.

**What the research documents propose (from `environment_climate.md`):**

- 4-domain pollution model (air, water, noise, soil) with distinct dispersion models
- Gaussian plume model for air pollution dispersion (wind-aware)
- Source strengths by building type (detailed emission rates)
- Pollution health effects with dose-response curves
- Technology upgrades that reduce emissions (scrubbers, catalytic converters)
- Stormwater management and flooding
- Urban heat island effect
- Heating and cooling degree days affecting energy demand
- Seasonal modifiers for all systems (agriculture, construction, tourism, energy)
- Extreme weather events (heat waves, cold snaps, storms)
- Climate change progression over long game time
- Waste-to-energy facilities
- Recycling rate tracking
- Composting systems
- Soil contamination from industrial brownfields
- Green infrastructure (rain gardens, green roofs, bioswales)

**Gap analysis:**

The environment system covers all four pollution domains (air, water, noise, soil/groundwater), weather with seasons, wind, and trees. Key gaps: no Gaussian plume dispersion (current model is simple neighbor diffusion), no technology upgrades for emission reduction, no stormwater/flooding, no urban heat island, no climate change progression, no green infrastructure options. The weather system exists but is cosmetic -- it changes temperature and precipitation values but these have limited gameplay effects beyond heating demand. Wind exists but does not affect pollution dispersion.

---

### 1.12 Water, Sewage, and Stormwater Infrastructure

**What exists in the codebase:**

- `utilities.rs`: `UtilitySource` component with `utility_type` (PowerPlant, WaterTower, SolarPanel, WindTurbine, SewagePlant), `grid_x`, `grid_y`, `range`. `propagate_utilities` uses BFS flood-fill from utility sources to set `has_power` and `has_water` flags on grid cells within range.
- `grid.rs`: Per-cell `has_power: bool` and `has_water: bool` flags.
- `groundwater.rs`: Groundwater table simulation with recharge and depletion.

**What the research documents propose (from `underground_infrastructure.md`, `environment_climate.md`):**

- Explicit pipe network for water distribution (draw pipes, connect buildings)
- Water pressure zones and distribution modeling
- Pipe capacity limits and flow calculation
- Sewage network separate from water supply
- Sewage treatment plant capacity and effluent quality
- Stormwater drainage network (separate from sewage = separated system; combined = combined sewer overflow risk)
- Flooding from inadequate stormwater capacity
- Underground view/layer system for pipe management
- Pipe aging, breaks, and maintenance
- Water quality modeling (treatment levels, contamination events)
- Desalination plants for coastal cities
- Reservoir and dam construction
- Water recycling/reclamation
- Automatic pipe-along-road option (CS2 style simplification as toggle)

**Gap analysis:**

The current utility system is the simplest possible model: coverage-radius BFS from utility source buildings. There are no pipes, no pressure, no capacity, no sewage network, no stormwater. The research documents describe a spectrum from CS2's automatic coverage (current implementation) to CS1's explicit pipe drawing to a full pressure-flow simulation. The recommended approach is a hybrid: pipes auto-follow roads by default (reducing tedium) but with capacity limits and upgrade tiers (maintaining strategic depth). This is a T2 system.

---

### 1.13 Power Grid

**What exists in the codebase:**

- `utilities.rs`: PowerPlant, SolarPanel, WindTurbine as utility types. BFS propagation sets `has_power` on grid cells within range. No grid balancing, no capacity tracking, no blackouts.

**What the research documents propose (from `environment_climate.md`, `underground_infrastructure.md`):**

- Energy demand calculation per building (residential, commercial, industrial have different profiles)
- Time-of-day demand curves (peak in evening for residential, daytime for commercial)
- Seasonal demand variation (heating in winter, cooling in summer)
- Generation types: coal, gas, nuclear, solar, wind, hydro, geothermal, waste-to-energy
- Each type has capacity, fuel cost, pollution, reliability characteristics
- Grid balancing: supply must meet demand or brownouts/blackouts occur
- Blackout cascading effects (traffic lights fail, hospitals on backup, citizen panic)
- Energy storage (batteries, pumped hydro) for renewable intermittency
- Peak pricing economics
- Power line/cable routing (above ground or underground)
- Smart grid features for late-game technology
- Carbon tax policy interaction

**Gap analysis:**

The power system is purely binary (has_power or not) with no demand calculation, no capacity tracking, no generation mix, no grid balancing, no blackouts. This is one of the largest gaps between current implementation and research targets. A basic demand/supply power grid is a T2 priority.

---

### 1.14 Natural Disasters

**What exists in the codebase:**

- `disasters.rs`: `ActiveDisaster` resource. `trigger_random_disaster` spawns earthquakes, floods, tornados, meteors at random intervals. `process_active_disaster` applies damage over time. `apply_earthquake_damage` destroys buildings in affected area.
- `fire.rs`: Fire spread and extinguish mechanics.
- `forest_fire.rs`: Forest fire simulation in tree areas.

**What the research documents propose (from `environment_climate.md`, `endgame_replayability.md`):**

- Earthquake simulation: fault lines on terrain, Richter scale magnitude, building damage based on construction quality, aftershock sequences
- Flood simulation: river overflow based on precipitation, stormwater capacity, flood plain mapping, levee/dam construction
- Wildfire simulation: spread based on wind, vegetation density, humidity, firebreak effectiveness
- Tornado simulation: path generation, Fujita scale damage, warning system effectiveness
- Volcanic events: lava flow, ash fall, evacuation mechanics
- Tsunami: coastal flooding triggered by offshore earthquake
- Hurricane/typhoon: multi-day event with storm surge, wind damage, flooding
- Disaster preparedness: early warning systems, emergency shelters, evacuation routes
- Insurance system: disaster damage costs offset by insurance premiums
- Disaster recovery: rebuilding grants, federal aid
- Player-triggered disasters (sandbox mode)

**Gap analysis:**

Basic disaster infrastructure exists (random triggers, active disaster state, earthquake damage). Key gaps: no terrain-aware disaster behavior (earthquakes don't follow fault lines, floods don't follow elevation), no disaster preparedness mechanics, no insurance system, no recovery grants, no evacuation simulation. The current implementation is "random bad thing damages stuff" rather than a simulation of natural phenomena.

---

### 1.15 Governance and Politics

**What exists in the codebase:**

- `policies.rs`: `Policies` resource with toggleable policy flags and modifiers. Policies affect tax rates, service efficiency, environmental rules, building permissions.
- `districts.rs`: `Districts` and `DistrictMap` for dividing the city into named districts. `aggregate_districts` computes per-district statistics. `district_stats` reports population, jobs, happiness per district.
- `events.rs`: `EventJournal` for historical event log. `ActiveCityEffects` for temporary modifiers. `MilestoneTracker` for progression tracking. `random_city_events` generates events (economic boom, infrastructure failure, etc.).
- `advisors.rs`: `AdvisorPanel` with AI-generated advice based on city state.
- `achievements.rs`: `AchievementTracker` with milestone-based achievement checking.

**What the research documents propose (from `social_agent_simulation.md`, `indie_competitors.md`, `game_design_mechanics.md`, `endgame_replayability.md`):**

- Political faction system (Tropico-style): environmentalists, business, labor, NIMBY, progressive, conservative
- Election system with campaigns and consequences
- Citizen political opinions driven by demographics and city conditions
- City council / advisory board with faction representatives
- Referendum mechanic for controversial decisions (highway through neighborhood, factory zoning)
- Corruption simulation
- Lobbying by interest groups
- Media coverage affecting public opinion
- Protest mechanics (citizens march when unhappy about specific issues)
- Policy complexity: policies have positive effects AND negative trade-offs
- Era progression: policies available change with city age and technology level
- Constitutional laws (Tropico-style) that set fundamental governance parameters
- Mayor approval rating as ongoing metric
- Campaign promises that create binding objectives

**Gap analysis:**

The governance system has basic policies, districts, events, advisors, and achievements. These are functional but shallow. Key gaps: no citizen political opinions, no elections, no factions, no protests, no referendum mechanics, no lobbying, no media. The advisor system exists but appears to be simple threshold-based advice rather than faction-driven political simulation. The policy system is toggle-based rather than the nuanced tradeoff system described in research. This is a T3 system that adds significant depth but is not needed for core gameplay.

---

### 1.16 Modding and Data-Driven Architecture

**What exists in the codebase:**

- Bevy plugin architecture: the game already uses `SimulationPlugin`, `RenderingPlugin`, `UiPlugin`, `SavePlugin` as separate crate-level plugins.
- No modding API, no scripting integration, no asset pipeline for custom content, no data files (all game parameters are hardcoded in Rust).

**What the research documents propose (from `modding_architecture.md`):**

- Mod SDK crate with stable API that does not expose internals
- Hot-reloading native plugins via dynamic library loading
- Mod load ordering and dependency resolution
- Scripting language integration (Lua via mlua, WASM via wasmtime, or Rhai)
- Custom building/vehicle/prop asset pipeline with validation
- Data-driven architecture: move all hardcoded values to data files (RON/JSON/TOML)
- Override hierarchy: base game data -> mod data -> user data
- Steam Workshop integration for mod distribution
- Mod manager UI
- Sandboxing and security for mod code
- Backward compatibility strategy (stable API versioning)
- 80% of mods should be pure data, 15% lightweight scripting, 5% native plugins

**Gap analysis:**

The modding system does not exist. However, the codebase is structured well for future modding: the Bevy plugin architecture naturally supports mod plugins, and the workspace crate layout provides clean separation. The first step toward moddability is data-driven architecture -- extracting hardcoded values (building stats, road parameters, service radii, policy effects) into external data files. This is a T4 system for the full modding SDK but the data-driven foundation should start earlier (T2-T3).

---

### 1.17 Save/Load System

**What exists in the codebase:**

- `save/src/lib.rs`: `SavePlugin` with `SaveGameEvent`, `LoadGameEvent`, `NewGameEvent`. Systems: `handle_save`, `handle_load`, `handle_new_game`.
- `save/src/serialization.rs`: Custom serialization layer with `SaveData` struct. Flat save structs (`SaveCitizen`, `SaveBuilding`, etc.) with primitive types, no Entity references. Binary encoding via bitcode. Entity remapping on load. Saves grid, citizens, buildings, services, utilities, budget, roads, segments, policies, weather, unlocks, extended budget, loans.
- Known issues: `LifeSimTimer` not serialized (life events fire on load). `PathCache`/`Velocity` not serialized (commuting citizens lose paths). `VirtualPopulation` count not serialized.

**What the research documents propose (from `save_system_architecture.md`):**

- Version migration system with per-version migration functions
- Delta/incremental saves (only save changed entities)
- Autosave with configurable interval and slot rotation
- Cloud save integration (Steam Cloud)
- Save file integrity checking (checksums)
- Save file compression (zstd)
- Sub-second save times for 100K+ citizens
- Post-load reconstruction of derived state (road graph, spatial indices, coverage grids)
- Complete serialization inventory (every resource and component audited)
- Save file security (prevent tampering for achievement integrity)
- Testing strategy: round-trip tests, fuzz testing, migration tests

**Gap analysis:**

The save system is functional with binary encoding and entity remapping. Key gaps: no version migration system (format changes require code updates and invalidate old saves), no incremental saves, no autosave, no cloud save, no integrity checking, no compression. The missing serialization for LifeSimTimer, PathCache, Velocity, and VirtualPopulation are known bugs. Save performance at 100K+ citizens has not been tested.

---

### 1.18 Camera, Controls, and UX

**What exists in the codebase:**

- `camera.rs`: `OrbitCamera` with focus point, yaw, pitch (5-80 degrees), distance. Pan via keyboard (WASD) and mouse drag. Zoom via scroll wheel. Orbit via right-click drag. Left-click drag for interaction.
- `input.rs`: `ActiveTool` enum (Select, ZoneRes, ZoneCom, ZoneInd, ZoneOffice, Bulldoze, BuildRoad, PlaceService, PlaceUtility, PlantTrees). `CursorGridPos` for mouse-to-grid mapping. `handle_tool_input` processes clicks. `keyboard_tool_switch` for hotkeys. `StatusMessage` for feedback text.
- `cursor_preview.rs`: Visual preview of current tool action (zone highlight, road preview).
- `overlay.rs`: `OverlayState` enum (None, Traffic, Pollution, LandValue, Fire, Crime, Health, Education, Garbage, Services, Noise, Power, Water, WaterPollution, NaturalResources, Happiness, Zones, Wind, Trees, Heating, Groundwater, PostalCoverage). Toggle via keyboard shortcuts.
- `building_render.rs`: Building mesh management, construction visuals.
- `citizen_render.rs`: Citizen sprite spawning and updating based on LOD tier.
- `road_render.rs`: Bezier road mesh synchronization.
- `day_night.rs`: Day/night cycle visual changes (ambient light, directional light color).
- `status_icons.rs`: Building status icons (no power, no water, abandoned).
- `props.rs`: Tree props, road props (lamp posts), parked cars.
- `ui/toolbar.rs`: Top toolbar with tool selection, speed controls, budget display.
- `ui/info_panel.rs`: Side panel with city stats, building inspection, policies, event journal.
- `ui/milestones.rs`: Milestone progression UI.
- `ui/graphs.rs`: Historical data graphs (population, budget, happiness over time).
- `ui/theme.rs`: Visual theme configuration.

**What the research documents propose (from `camera_controls_ux.md`):**

- Smooth camera interpolation (exponential lerp, frame-rate independent)
- Dynamic pitch limits based on zoom (far = steep only, close = street-level)
- Pitch-dependent FOV (telephoto at low pitch, wide at high pitch)
- Camera bookmarks (save/recall positions)
- Camera follow mode (track a citizen, vehicle, or service vehicle)
- Cinematic/photo mode (hide UI, free camera, DOF, color grading)
- First-person mode (walk the city at street level)
- Zoom-dependent LOD transitions (seamless)
- Enhanced data overlays with better color ramps and legends
- Selection mechanics: click-select, box-select, lasso-select
- Multi-select for batch operations
- Right-click context menus
- Undo/redo for all player actions
- Road building UX improvements (snap modes, angle constraints, elevation control)
- Blueprint/template system for reusable layouts
- Search/filter for buildings and citizens
- Notification center with categorized alerts
- Minimap
- Controller support
- Accessibility (colorblind modes, screen reader support, key remapping)

**Gap analysis:**

The camera and UX systems are functional with orbital camera, tool system, overlays, toolbar, and info panel. Key gaps: no camera smoothing (direct application, no lerp), no camera bookmarks, no follow mode, no cinematic mode, no first-person mode, no undo/redo system, no multi-select, no context menus, no minimap, no blueprint system, no controller support, no accessibility features. The overlay system is comprehensive (22 overlay types) but could benefit from better visualization (color ramps, legends, side-by-side comparison).

---

### 1.19 Sound and Music

**What exists in the codebase:**

- No audio system whatsoever. No sound effects, no music, no audio plugin.

**What the research documents propose (from `sound_design.md`):**

- `bevy_kira_audio` integration for audio playback
- Audio bus hierarchy (master -> music, SFX, ambient, UI)
- Spatial audio for city soundscapes (zone-based ambient layers)
- Traffic audio (engine hum proportional to traffic density)
- Construction site audio
- Distance attenuation model tied to camera distance
- Sound occlusion and urban canyon effects
- Dynamic music system with vertical layering (stem-based mixing)
- Horizontal re-sequencing based on city state
- Time-of-day musical palettes (dawn, day, dusk, night)
- Crisis and event music stingers
- Environmental audio (weather sounds, seasonal ambience, water bodies)
- Notification and UI sounds with priority hierarchy and cooldowns
- Tool interaction sounds (road placement, zoning, bulldoze)
- Procedural audio generation (traffic hum, rain, wind, crowd murmur)
- Audio LOD system (aggregate sounds at far zoom, individual at close zoom)
- Chunk-based audio aggregation for performance

**Gap analysis:**

Complete gap. No audio exists. This is a T4 system -- the game is fully playable without sound, but sound dramatically improves the experience. The research doc is extremely detailed with specific implementation patterns for Bevy, making implementation straightforward when prioritized.

---

### 1.20 Endgame and Replayability

**What exists in the codebase:**

- `unlocks.rs`: `UnlockState` with development points. `award_development_points` based on population and happiness.
- `achievements.rs`: `AchievementTracker` with milestone-based achievements.
- `milestones.rs` (ui): Milestone progression display.
- `events.rs`: Random city events providing temporary modifiers.
- `specialization.rs`: City specialization system.
- `advisors.rs`: AI advisor panel.

**What the research documents propose (from `endgame_replayability.md`, `game_design_mechanics.md`):**

- Escalating late-game challenges (the "hour 20 problem" -- games become boring when cities stabilize)
- Infrastructure decay and rebuild cycles forcing ongoing engagement
- Demographic shifts (aging population, immigration waves, gentrification)
- Political complexity scaling with city size
- Environmental debt accumulating over time
- Legacy infrastructure constraints (old roads/pipes too small for modern demand)
- Congestion ceiling requiring transit investment
- Housing crisis and affordability mechanics
- Mega-projects as aspirational endgame goals (space elevator, arcology, world's fair)
- Scenario and challenge modes with specific objectives and constraints
- Roguelite elements and meta-progression (Against the Storm model)
- Prestige/New Game+ systems
- Scoring and achievement systems with leaderboards
- Procedural events generated from simulation state (not random dice rolls)
- Victory conditions and goal structures
- Multiple map types for replayability (coastal, island, mountain, desert, river delta)
- Era progression (settlement -> town -> city -> metropolis -> megacity)
- Difficulty modifiers selectable at game start

**Gap analysis:**

The endgame systems are skeletal. Unlocks, achievements, events, and specializations exist but do not create the ongoing tension described in research. The fundamental "positive feedback convergence" problem (city stabilizes and becomes boring) is not addressed. Key gaps: no escalating challenges, no infrastructure decay cycles beyond road maintenance, no demographic shifts, no housing crisis mechanics, no mega-projects, no scenario mode, no roguelite elements, no era progression. This is a T3-T4 concern but the design decisions need to be made early because they affect core system architecture (systems must generate instability, not just converge to equilibrium).

---

## 2. Dependency Graph

Systems do not exist in isolation. Each depends on others for data, events, or functional prerequisites. This section maps the dependency structure as a directed acyclic graph (DAG) and identifies the critical path.

### 2.1 Text DAG

```
Level 0 (No Dependencies - Foundations):
  [Grid/Terrain]
  [Game Clock]
  [Camera/Input]
  [Config/Constants]

Level 1 (Depends on Grid):
  [Road Network]     <- Grid
  [Zoning]           <- Grid
  [Terrain Render]   <- Grid, Camera

Level 2 (Depends on Roads + Zones):
  [Buildings]        <- Grid, Zoning, Road Network
  [Pathfinding]      <- Road Network
  [Road Render]      <- Road Network, Camera
  [Overlays]         <- Grid, Camera

Level 3 (Depends on Buildings + Pathfinding):
  [Citizens]         <- Buildings (homes/jobs), Pathfinding
  [Utilities]        <- Grid, Buildings (placement)
  [Services]         <- Grid, Buildings (placement)
  [Land Value]       <- Grid, Road Network, Services

Level 4 (Depends on Citizens):
  [Traffic]          <- Citizens (movement), Road Network
  [Economy/Budget]   <- Citizens (tax base), Buildings (property tax)
  [Happiness]        <- Citizens, Services, Traffic, Land Value, Pollution
  [Life Simulation]  <- Citizens, Economy, Services

Level 5 (Depends on Traffic + Economy):
  [Public Transit]   <- Road Network, Citizens, Economy, Traffic
  [Road Maintenance] <- Traffic, Economy (budget)
  [Pollution]        <- Traffic, Buildings (industry), Wind
  [Crime]            <- Citizens, Land Value, Services (police)

Level 6 (Depends on Multiple L4-L5 Systems):
  [Building Upgrade] <- Land Value, Services, Happiness, Economy
  [Abandonment]      <- Utilities, Happiness, Economy
  [Immigration]      <- Happiness, Economy, Services (attractiveness)
  [Weather]          <- Game Clock (seasonal)
  [Fire]             <- Services (fire stations), Buildings

Level 7 (Depends on Broad System State):
  [Disasters]        <- Grid, Buildings, Services, Weather
  [Events]           <- Economy, Happiness, Population (city stats)
  [Districts]        <- Grid, Buildings, Citizens, Economy
  [Achievements]     <- Stats (everything)

Level 8 (Meta-Systems):
  [Save/Load]        <- Everything (serialization of all state)
  [Governance]       <- Citizens (opinions), Economy, Services, Happiness
  [Endgame]          <- Everything (emergent from mature city state)

Level 9 (Polish/Extension):
  [Sound]            <- Grid, Traffic, Weather, Events, Camera
  [Modding]          <- Everything (stable API over all systems)
  [Procedural Terrain] <- Grid (replaces hardcoded terrain)
```

### 2.2 Critical Path

The longest dependency chain determines the minimum sequential work required before the game is playable:

```
Grid -> Roads -> Zoning -> Buildings -> Citizens -> Traffic -> Economy -> Happiness -> Immigration -> Endgame
```

This is a 10-step chain. Every link must be functional for the game to be a game. The current codebase has implemented all 10 steps to at least a basic level, meaning the critical path is already traversed. Future work is about deepening each node, not unblocking new ones.

### 2.3 Key Dependency Clusters

**The Land Value Hub:**
Land value is the most connected node in the dependency graph. It reads from:
- Road Network (accessibility)
- Services (coverage)
- Pollution (negative)
- Noise (negative)
- Crime (negative)
- Buildings (neighborhood quality)
- Transit (accessibility bonus)
- Water views, parks (amenity)
- Trees (greenery bonus)

And it feeds into:
- Building upgrade/downgrade
- Property tax revenue
- Building growth decisions
- Citizen housing choice
- Immigration attractiveness

Any change to land value ripples through the entire simulation. This makes it both powerful (one system connects everything) and dangerous (bugs here affect everything).

**The Happiness Hub:**
Happiness is the second most connected node:
- Reads: housing quality, commute time, service coverage, pollution exposure, crime, noise, weather, tax rate, employment, health, education, social needs
- Feeds: immigration/emigration, building upgrade/downgrade, crime rate, political opinion, productivity

**The Traffic Nexus:**
Traffic connects physical simulation to economic simulation:
- Reads: citizen movement, freight, transit ridership, road capacity, signal timing
- Feeds: commute time (happiness), air/noise pollution, road wear, accident rate, economic productivity, land value (accessibility)

---

## 3. Priority Tiers

Every system is assigned to a tier based on its necessity for gameplay, its dependency position, and its impact on player experience. Each tier includes a rough complexity estimate and a description of what the game feels like when that tier is complete.

---

### T0 -- Foundation (Must Exist for Anything to Work)

These systems are the substrate upon which everything else is built. Without them, there is no game. All T0 systems currently exist in the codebase at a functional level.

| System | Status | Estimated Work to Target | Notes |
|--------|--------|--------------------------|-------|
| Grid (256x256, cell types, coordinates) | **Functional** | 1 pw | Minor: add soil type, slope calculation |
| Terrain generation | **Basic** | 3-4 pw | Needs fBm, erosion, biomes for proc gen |
| Terrain rendering | **Functional** | 2-3 pw | Needs heightmap vertices (currently flat y=0) |
| Road segment system (Bezier curves) | **Functional** | 1-2 pw | Needs intersection improvements |
| Road network graph (CSR, A*) | **Functional** | 1 pw | Performance optimization at scale |
| Road rendering | **Functional** | 1 pw | Visual polish, lane markings |
| Basic zone types (R/C/I/O) | **Functional** | 1 pw | Add ResidentialMedium, Mixed-Use |
| Zone painting tool | **Functional** | 0.5 pw | Minor UX improvements |
| Camera and basic input | **Functional** | 1-2 pw | Add smoothing, bookmarks |
| Game clock (24hr cycle, seasons) | **Functional** | 0.5 pw | Minor: serialization fix |

**Estimated total: 12-16 person-weeks**

**Game feel at T0 completion:** A flat grid where you can draw roads, paint zones, and move a camera around. No buildings, no citizens, no simulation. This is a terrain editor with road tools. The current codebase is well past this point.

---

### T1 -- Core Simulation (Makes It a Real Game)

These systems create the core gameplay loop: zone -> build -> populate -> tax -> expand. Without T1, you have a sandbox toy. With T1, you have a game with goals, constraints, and feedback.

| System | Status | Estimated Work to Target | Notes |
|--------|--------|--------------------------|-------|
| Building spawning and growth (3 levels) | **Functional** | 2-3 pw | Expand to 5 levels, add variety |
| Building construction delay | **Functional** | 0.5 pw | Add construction materials requirement |
| Citizen spawning | **Functional** | 0.5 pw | Minor improvements |
| Citizen state machine (home/work/shop/leisure) | **Functional** | 2 pw | Add schedule-based variation |
| Citizen movement and pathfinding | **Functional** | 2 pw | BPR travel time, path caching |
| Traffic density tracking | **Functional** | 1 pw | Add LOS grading, visual feedback |
| Basic economy (taxes, expenses, treasury) | **Functional** | 3-4 pw | Property tax, budget categories |
| Basic services (fire, police, health, education) | **Functional** | 2-3 pw | Add capacity limits, dispatch |
| Basic utilities (power, water) | **Functional** | 2-3 pw | Add demand/supply, blackouts |
| Happiness system | **Functional** | 1-2 pw | Tune weights, add more factors |
| Basic land value | **Functional** | 2 pw | Add accessibility component |
| Zone demand calculation | **Functional** | 1 pw | Market-driven demand |
| Life simulation (aging, death, education) | **Functional** | 1-2 pw | Fix serialization, tune rates |
| Immigration/emigration | **Functional** | 1 pw | Tune attractiveness factors |
| Save/load | **Functional** | 3-4 pw | Version migration, autosave |
| UI toolbar and info panel | **Functional** | 2-3 pw | Polish, more info displays |
| Basic pollution (air, noise) | **Functional** | 1-2 pw | Wind-aware dispersion |
| Building upgrade/downgrade | **Functional** | 1-2 pw | More level-up criteria |
| Abandonment | **Functional** | 0.5 pw | Minor tuning |

**Estimated total: 28-40 person-weeks**

**Game feel at T1 completion:** A playable city builder. You zone land, buildings grow, citizens move in, you collect taxes, build services, manage traffic. The core loop works. It feels like a simplified Cities: Skylines without transit, advanced economics, or polish. You can play for 5-10 hours before the systems feel shallow. This is roughly where the current codebase is.

---

### T2 -- Depth (Makes It Good)

These systems add the layers that transform a toy into a compelling game. Each T2 system creates new decision spaces and interactions that extend playtime.

| System | Status | Estimated Work to Target | Notes |
|--------|--------|--------------------------|-------|
| 5-level building system | **Partial** (3 levels) | 3-4 pw | New level thresholds, models |
| Multi-cell buildings (2x2, 3x3, 4x4) | **Not started** | 4-5 pw | Grid allocation, rendering |
| Building variety (procedural facades) | **Not started** | 4-6 pw | Mesh variation, corner lots |
| Public transit (bus lines, routes) | **Not started** | 6-8 pw | Line drawing, vehicles, ridership |
| Metro/subway system | **Not started** | 6-8 pw | Underground layer, stations |
| Mode choice model (car vs transit vs walk) | **Not started** | 3-4 pw | Utility-based mode selection |
| Water/sewage pipe network | **Not started** | 5-7 pw | Pipe drawing, capacity, underground view |
| Power grid (demand/supply/blackouts) | **Not started** | 4-5 pw | Generation mix, grid balancing |
| Property tax system | **Partial** (zone rates exist) | 3-4 pw | Full assessment, millage rates |
| Economic cycles (boom/bust) | **Not started** | 3-4 pw | Periodic oscillation, triggers |
| Weather gameplay effects | **Partial** (heating exists) | 2-3 pw | Storms, seasonal modifiers |
| Advanced pollution (wind dispersion) | **Partial** | 2-3 pw | Gaussian plume, technology upgrades |
| Crime pipeline (poverty -> crime -> policing) | **Partial** (basic crime grid) | 3-4 pw | Crime types, justice system |
| Service vehicle dispatch | **Not started** | 4-5 pw | Vehicles on road network, response time |
| Education pipeline (K-12 + university) | **Partial** | 2-3 pw | Graduation rates, capacity |
| Garbage truck routing | **Not started** | 2-3 pw | Collection routes, landfill capacity |
| Road hierarchy enforcement | **Partial** (types exist) | 2-3 pw | Bottleneck detection, LOS display |
| District-level policies | **Partial** (districts exist) | 2-3 pw | Per-district tax/service settings |
| Procedural terrain (fBm + erosion) | **Basic** | 5-7 pw | Full terrain pipeline |
| 3D terrain rendering (elevation mesh) | **Not started** | 3-4 pw | Vertex height from elevation |
| Data-driven game parameters | **Not started** | 4-6 pw | Extract constants to data files |

**Estimated total: 73-100 person-weeks**

**Game feel at T2 completion:** A competitive city builder. Transit, infrastructure networks, economic depth, and building variety make it feel like a complete game. You can play for 20-40 hours. Comparable to Cities: Skylines 1 in scope, potentially better in specific areas (traffic simulation, economic depth). This is the target for a strong Early Access launch.

---

### T3 -- Differentiation (Makes It Special)

These systems make Megacity do things no other city builder does. They are the unique selling points that justify the game's existence in a market with Cities: Skylines.

| System | Status | Estimated Work to Target | Notes |
|--------|--------|--------------------------|-------|
| Form-based zoning codes | **Not started** | 4-5 pw | Alternative to Euclidean zoning |
| Mixed-use buildings | **Not started** | 3-4 pw | Commercial ground + residential above |
| NIMBY/YIMBY citizen mechanics | **Not started** | 3-4 pw | Citizen opposition to development |
| Political faction system | **Not started** | 5-7 pw | Factions, elections, approval |
| Citizen political opinions | **Not started** | 3-4 pw | Per-citizen political state |
| Advanced demographics (ethnicity, income class) | **Not started** | 3-4 pw | Extended CitizenDetails |
| Schelling segregation model | **Not started** | 2-3 pw | Neighborhood demographics |
| Housing affordability crisis mechanics | **Not started** | 3-4 pw | Rent burden, displacement |
| Gentrification simulation | **Not started** | 2-3 pw | Emergent from land value + demographics |
| Social mobility tracking | **Not started** | 2-3 pw | Income quintile movement |
| Business profitability simulation | **Not started** | 3-4 pw | Revenue - costs = profit |
| Labor market with wage determination | **Not started** | 3-4 pw | Supply/demand wage setting |
| Production chains (W&R style) | **Partial** (basic) | 4-5 pw | Deeper commodity chains |
| Stormwater and flooding | **Not started** | 4-5 pw | Drainage capacity, flood events |
| Real estate market | **Not started** | 3-4 pw | Housing prices, speculation |
| 15-minute city scoring | **Not started** | 2-3 pw | Walkability metrics |
| Walkability overlay | **Not started** | 1-2 pw | Visual accessibility scoring |
| Advanced advisors (faction-aligned) | **Partial** (basic advisors) | 2-3 pw | Faction-driven advice |
| Era progression | **Not started** | 5-7 pw | Visual and mechanical changes |
| Historic preservation | **Not started** | 2-3 pw | Protection districts |

**Estimated total: 60-82 person-weeks**

**Game feel at T3 completion:** A city builder that urban planning enthusiasts and political simulation fans consider best-in-class. The social simulation creates emergent stories. Segregation, gentrification, political tension, and housing crises arise organically from simulation dynamics. Reviews mention "systems that no other game has." This is the differentiated product. Comparable to the depth of Tropico's politics combined with Cities: Skylines' city building.

---

### T4 -- Polish (Makes It Great)

These systems elevate the game from good to excellent. They are the difference between "solid indie game" and "genre-defining title."

| System | Status | Estimated Work to Target | Notes |
|--------|--------|--------------------------|-------|
| Sound system (spatial audio, music) | **Not started** | 8-12 pw | Full audio pipeline |
| Dynamic music system | **Not started** | 4-6 pw | Adaptive stems, time-of-day |
| Natural disaster improvements | **Partial** (basic) | 4-5 pw | Terrain-aware, preparedness |
| Disaster insurance system | **Not started** | 2-3 pw | Premiums, payouts |
| Mega-projects (arcology, space elevator) | **Not started** | 5-7 pw | Multi-stage construction |
| Scenario/challenge modes | **Not started** | 4-5 pw | Objective system, constraints |
| Camera follow mode | **Not started** | 1-2 pw | Track citizen/vehicle |
| First-person city walk | **Not started** | 3-4 pw | Street-level camera, LOD |
| Photo/cinematic mode | **Not started** | 2-3 pw | Hide UI, DOF, color grading |
| Undo/redo system | **Not started** | 4-5 pw | Command pattern, full history |
| Blueprint/template system | **Not started** | 3-4 pw | Save/load layouts |
| Minimap | **Not started** | 1-2 pw | Corner map with navigation |
| Controller support | **Not started** | 3-4 pw | Gamepad input mapping |
| Accessibility (colorblind, key remap) | **Not started** | 2-3 pw | Color schemes, rebinding |
| Advanced save system (autosave, cloud) | **Partial** | 3-4 pw | Autosave rotation, Steam Cloud |
| Achievement/prestige system | **Partial** (basic) | 2-3 pw | Leaderboards, NG+ |
| Interchange templates | **Not started** | 2-3 pw | Pre-built highway designs |
| Roundabout builder tool | **Not started** | 2-3 pw | Automated roundabout creation |
| Modular road designer | **Not started** | 5-7 pw | Lane customization per road |

**Estimated total: 60-86 person-weeks**

**Game feel at T4 completion:** A polished, release-ready game. Sound brings the city to life. Scenarios provide replayability. The first-person camera creates viral screenshot/video moments. Undo/redo removes frustration. Accessibility widens the audience. Ready for a confident 1.0 launch or strong Early Access with "Very Positive" Steam reviews.

---

### T5 -- Stretch (Aspirational)

These systems would make Megacity the definitive city builder for a generation. They are high-risk, high-reward features that may not be feasible for initial release.

| System | Status | Estimated Work to Target | Notes |
|--------|--------|--------------------------|-------|
| Full modding SDK (native plugins) | **Not started** | 12-16 pw | Stable API, hot-reload |
| Scripting integration (Lua or WASM) | **Not started** | 8-12 pw | Sandboxed scripting |
| Custom asset pipeline (buildings, vehicles) | **Not started** | 6-8 pw | Import/validate/load |
| Steam Workshop integration | **Not started** | 4-6 pw | Upload/download/manage |
| Mod manager UI | **Not started** | 2-3 pw | Enable/disable/order |
| Regional/multi-city play | **Not started** | 12-16 pw | Multiple maps, trade |
| Multiplayer (cooperative city building) | **Not started** | 20-30 pw | Networking, sync |
| Full procedural terrain pipeline | **Partial** (basic noise) | 6-8 pw | Erosion, biomes, seeds |
| Lane-level traffic simulation | **Not started** | 8-12 pw | Per-lane vehicle tracking |
| Roguelite meta-progression | **Not started** | 5-7 pw | Meta-currency, unlocks |
| Climate change long-term progression | **Not started** | 3-4 pw | Sea level, temperature |
| Disease/epidemic simulation | **Not started** | 4-5 pw | Contagion model, hospitals |
| Parking simulation | **Not started** | 3-4 pw | Parking search, garages |
| Construction material delivery | **Not started** | 3-4 pw | Material logistics |

**Estimated total: 96-135 person-weeks**

**Game feel at T5 completion:** The definitive city builder. Mods extend the game infinitely. Regional play creates a meta-game. Lane-level traffic is the most realistic in any game. Climate change creates 100-year gameplay arcs. This is the 5-year vision.

---

## 4. Milestone Definitions

Each milestone represents a concrete, testable state of the game. Milestones are defined by what the player can do, not by what code exists.

---

### M1 -- Playable Prototype

**Target: Current state (already achieved)**

**Definition:** A single-map city builder where the core loop functions: road placement, zone painting, building growth, citizen simulation, basic services, traffic, and economy. The city grows, generates revenue, and can reach 50K+ population.

**What works:**
- Draw Bezier roads of 6 types on a 256x256 grid
- Paint 6 zone types adjacent to roads
- Buildings spawn automatically in zoned areas, upgrade through 3 levels
- Citizens spawn in residential buildings, commute to work via A* pathfinding
- Traffic density accumulates on road cells from citizen movement
- Economy: flat tax per citizen + commercial income, service expenses deducted
- 40+ service types with coverage-radius effects
- Utilities (power, water) via BFS flood-fill
- Happiness from services, commute, environment, economy
- Life simulation: aging, death, education, marriage, children
- Immigration based on city attractiveness
- Weather with seasonal cycles
- Fire spread and extinguishment
- Crime from low land value and low police coverage
- Pollution from industry and traffic
- Road maintenance and degradation
- Save/load with binary encoding
- 22 data overlays
- LOD system for citizens (Full/Simplified/Abstract)
- Virtual population for scaling beyond entity count
- Day/night cycle rendering
- Milestone-based progression and achievements
- District system with statistics
- Advisor panel
- Loans with credit rating

**What the player experiences:** A functional but rough city builder. You can build a city, watch it grow, and manage its services. The simulation runs, citizens move, buildings grow. It feels like an early alpha -- systems work but lack depth, variety, and polish. There is no sound, limited building variety, and the terrain is flat.

**Gate criteria for M1:** All items above are functional. The game does not crash during normal play. Save/load round-trips without data loss (except known missing fields).

---

### M2 -- Core Loop Complete

**Target: T0 + T1 fully polished. Game is fun to play for 10+ hours.**

**Additions beyond M1:**

- [ ] 5-level building system with clear visual progression
- [ ] Property tax replacing per-citizen flat tax
- [ ] Budget categories: separate income/expense lines for each service type
- [ ] LOS A-F traffic grading with visual feedback (road color changes)
- [ ] BPR-based travel time in pathfinding
- [ ] Power demand/supply balance with brownout/blackout consequences
- [ ] Water demand/supply with shortage effects
- [ ] Service capacity limits (hospital beds, school seats)
- [ ] Save versioning with migration system
- [ ] Autosave with configurable interval
- [ ] Camera smoothing (exponential lerp)
- [ ] Improved zone demand (market-driven)
- [ ] Fix: LifeSimTimer serialization
- [ ] Fix: PathCache/Velocity serialization
- [ ] Fix: VirtualPopulation serialization
- [ ] Improved UI: budget breakdown panel, service coverage details
- [ ] At least 2-3 building mesh variants per zone type per level
- [ ] Construction material cost (deducted from budget, not delivered)

**What the player experiences:** A complete, if simple, city builder. Every system feels like it matters. Running out of power causes real problems. Traffic jams have measurable economic impact. The budget requires careful management. Buildings clearly improve as the city develops. Saving and loading is reliable. The game is fun for 10-15 hours before systems feel fully explored. Comparable to a focused indie city builder like Mini Motorways in engagement depth, though in a different genre.

**Gate criteria for M2:** Playtesters report the game is "fun" without prompting. No critical bugs. Save files survive version updates via migration. Budget can go negative and trigger consequences.

---

### M3 -- Feature Complete

**Target: T0 + T1 + T2. Game is competitive with Cities: Skylines 1.**

**Additions beyond M2:**

- [ ] Public transit: bus lines with routes, stops, and vehicles
- [ ] Metro system with underground stations and lines
- [ ] Mode choice: citizens choose car, transit, or walking based on travel time
- [ ] Water/sewage pipe network (auto-along-road with manual override)
- [ ] Power grid with generation mix (coal, gas, solar, wind, nuclear)
- [ ] Economic cycles: periodic boom/bust with fiscal consequences
- [ ] 3D terrain rendering with heightmap-driven vertex positions
- [ ] Procedural terrain generation (fBm + erosion + water bodies)
- [ ] Seed-based map generation for replayability
- [ ] Advanced pollution: wind-aware air dispersion, soil contamination
- [ ] Crime pipeline: poverty -> crime rate -> policing effectiveness
- [ ] Service vehicle dispatch (fire trucks, ambulances on road network)
- [ ] Education pipeline with graduation rates and capacity
- [ ] Garbage collection routing
- [ ] District-level policies (per-district tax rates, service levels)
- [ ] Multi-cell buildings (at least 2x2)
- [ ] Building variety: 4-6 mesh variants per zone type per level
- [ ] Road hierarchy enforcement with bottleneck warnings
- [ ] Weather gameplay effects: storms damage buildings, snow slows traffic
- [ ] Data-driven game parameters (building stats, road configs in data files)
- [ ] Basic sound effects (road placement, zoning, notifications)

**What the player experiences:** A fully featured city builder that stands alongside Cities: Skylines 1. Transit creates new strategic decisions. Infrastructure networks (water, power) add planning depth. Economic cycles prevent the "stable plateau." Procedural terrain means every map is different. The crime and education pipelines create emergent social dynamics. This is a game you can play for 40+ hours. Steam reviews would say "surprisingly deep for an indie title."

**Gate criteria for M3:** All T2 systems functional. Transit ridership > 0. Pipe networks buildable. Power blackouts occur when supply < demand. Economic cycles observable. Terrain visually 3D. At least 3 map seeds produce playable, distinct maps.

---

### M4 -- Differentiated

**Target: T0 + T1 + T2 + T3. Game does things no other city builder does.**

**Additions beyond M3:**

- [ ] Form-based zoning as alternative to Euclidean
- [ ] Mixed-use buildings (commercial ground floor + residential above)
- [ ] Political faction system with citizen opinions and elections
- [ ] NIMBY/YIMBY mechanics for controversial developments
- [ ] Advanced demographics (income class, occupation detail)
- [ ] Schelling segregation model for neighborhoods
- [ ] Housing affordability crisis and gentrification
- [ ] Business profitability and closure simulation
- [ ] Labor market with wage determination
- [ ] 15-minute city walkability scoring
- [ ] Stormwater management and flood risk
- [ ] Era progression with visual and mechanical changes
- [ ] Historic preservation districts
- [ ] Real estate market simulation
- [ ] Production chains (deeper commodity system)
- [ ] Advanced advisor system (faction-aligned recommendations)

**What the player experiences:** A city builder that tells stories. Gentrification displaces residents. Political factions oppose your highway project. Housing crises emerge from economic growth. Elections force you to balance competing interests. Neighborhoods develop distinct characters through organic segregation and development patterns. Reviews say "the social simulation is unlike anything in the genre." Urban planning enthusiasts consider it the most realistic city builder ever made. 60-100+ hours of engagement.

**Gate criteria for M4:** At least 3 political factions functional. Elections occur. Housing affordability varies by neighborhood. Segregation emerges without scripting. Form-based zoning produces visually distinct neighborhoods from Euclidean zoning.

---

### M5 -- Ship (Early Access or 1.0)

**Target: T0 + T1 + T2 + T3 + T4. Polished and ready for commercial release.**

**Additions beyond M4:**

- [ ] Full sound system (spatial audio, dynamic music, UI sounds)
- [ ] Natural disaster improvements (terrain-aware, preparedness)
- [ ] Mega-projects as aspirational goals
- [ ] Scenario/challenge modes (at least 5 scenarios)
- [ ] Camera follow mode and first-person city walk
- [ ] Photo/cinematic mode
- [ ] Undo/redo for all player actions
- [ ] Blueprint/template system
- [ ] Minimap
- [ ] Controller support
- [ ] Accessibility (colorblind modes, key remapping)
- [ ] Advanced save system (autosave, cloud save, compression)
- [ ] Achievement/prestige system with Steam integration
- [ ] Interchange templates and roundabout builder
- [ ] Tutorial/onboarding flow
- [ ] Localization infrastructure (string tables, UI layout)

**What the player experiences:** A polished, complete game. Sound brings the city to life. Scenarios provide structured challenges. The first-person camera creates shareable moments. Undo removes frustration. The game feels professional, not indie. Ready for "Very Positive" Steam reviews. 100+ hours of content.

**Gate criteria for M5:** No critical or major bugs. Sound plays during all major game events. At least 5 scenarios completable. Undo/redo works for all placement tools. Game runs at 60fps with 100K citizens. All Steam achievements earnable.

---

### M6 -- Full Release

**Target: All tiers. Modding ecosystem, post-launch content pipeline.**

**Additions beyond M5:**

- [ ] Full modding SDK with native plugin support
- [ ] Scripting integration (Lua or WASM)
- [ ] Custom asset pipeline
- [ ] Steam Workshop integration
- [ ] Mod manager UI
- [ ] Regional/multi-city play
- [ ] Full procedural terrain pipeline
- [ ] Lane-level traffic simulation
- [ ] Additional map themes (desert, arctic, tropical, mountain)
- [ ] Roguelite meta-progression mode
- [ ] Climate change long-term progression
- [ ] Disease/epidemic simulation
- [ ] Post-launch content: DLC themes (airports, universities, industries)

**What the player experiences:** The definitive city builder. Mods extend the game infinitely. Regional play creates a meta-game that extends engagement for hundreds of hours. The modding community produces content that the base game could never. This is the 3-5 year vision after initial release.

**Gate criteria for M6:** Modding SDK stable (no breaking API changes for 6+ months). At least 10 community mods published. Regional play supports 4+ connected cities. Workshop integration functional.

---

## 5. Codebase Audit

### 5.1 Simulation Crate (`crates/simulation/src/`)

Complete inventory of every module in the simulation crate, with current state assessment, gap analysis, and estimated work.

| Module | Lines (est.) | State | Gap vs Research | Work to Target |
|--------|-------------|-------|-----------------|----------------|
| `grid.rs` | ~200 | **Complete** | Needs soil type, slope fields | 1 pw |
| `terrain.rs` | ~50 | **Stub** | Needs full fBm + erosion pipeline | 5-7 pw |
| `config.rs` | ~10 | **Complete** | Needs data-driven externalization | 1 pw |
| `road_segments.rs` | ~300 | **Functional** | Needs intersection improvements | 1-2 pw |
| `roads.rs` | ~100 | **Functional** | Serves as fallback to CSR | 0.5 pw |
| `road_graph_csr.rs` | ~250 | **Functional** | Needs BPR function, intersection model | 2-3 pw |
| `pathfinding_sys.rs` | ~50 | **Functional** | Minor utility, complete | 0 pw |
| `zones.rs` | ~80 | **Functional** | Needs market-driven demand model | 2-3 pw |
| `buildings.rs` | ~200 | **Functional** | Needs 5 levels, multi-cell, variety | 4-5 pw |
| `building_upgrade.rs` | ~100 | **Functional** | Needs expanded criteria, 5 levels | 2 pw |
| `abandonment.rs` | ~120 | **Functional** | Minor tuning needed | 0.5 pw |
| `citizen.rs` | ~180 | **Functional** | Needs extended demographics | 2-3 pw |
| `citizen_spawner.rs` | ~100 | **Functional** | Minor improvements | 0.5 pw |
| `movement.rs` | ~300 | **Functional** | Needs mode choice, BPR travel time | 3-4 pw |
| `life_simulation.rs` | ~250 | **Functional** | Needs deeper life events, fix timer | 2-3 pw |
| `lifecycle.rs` | ~100 | **Functional** | Minor improvements | 0.5 pw |
| `traffic.rs` | ~80 | **Functional** | Needs LOS grading, time-of-day | 2-3 pw |
| `traffic_accidents.rs` | ~100 | **Functional** | Minor improvements | 0.5 pw |
| `economy.rs` | ~150 | **Functional** | Needs property tax overhaul | 3-4 pw |
| `budget.rs` | ~200 | **Functional** | Needs more expense categories | 1-2 pw |
| `land_value.rs` | ~120 | **Functional** | Needs accessibility, spillover | 3-4 pw |
| `happiness.rs` | ~200 | **Functional** | Needs weight tuning, more factors | 1-2 pw |
| `services.rs` | ~200 | **Functional** | Needs capacity, dispatch | 3-4 pw |
| `education.rs` | ~80 | **Functional** | Needs pipeline, graduation rates | 2-3 pw |
| `education_jobs.rs` | ~150 | **Functional** | Needs wage determination | 2-3 pw |
| `health.rs` | ~60 | **Basic** | Needs disease model, capacity | 3-4 pw |
| `crime.rs` | ~80 | **Basic** | Needs crime types, justice pipeline | 3-4 pw |
| `fire.rs` | ~200 | **Functional** | Needs dispatch, response time | 2-3 pw |
| `forest_fire.rs` | ~100 | **Functional** | Minor improvements | 0.5 pw |
| `disasters.rs` | ~150 | **Functional** | Needs terrain-aware behavior | 3-4 pw |
| `pollution.rs` | ~100 | **Basic** | Needs Gaussian plume, wind | 3-4 pw |
| `noise.rs` | ~80 | **Functional** | Minor improvements | 0.5 pw |
| `water_pollution.rs` | ~100 | **Functional** | Minor improvements | 1 pw |
| `groundwater.rs` | ~150 | **Functional** | Minor improvements | 1 pw |
| `weather.rs` | ~100 | **Functional** | Needs gameplay effects | 2-3 pw |
| `wind.rs` | ~60 | **Basic** | Needs to affect pollution | 1-2 pw |
| `trees.rs` | ~60 | **Functional** | Minor improvements | 0.5 pw |
| `natural_resources.rs` | ~120 | **Functional** | Needs discovery mechanics | 1-2 pw |
| `utilities.rs` | ~100 | **Basic** | Needs demand/supply, pipe network | 5-7 pw |
| `production.rs` | ~150 | **Functional** | Needs deeper commodity chains | 3-4 pw |
| `market.rs` | ~100 | **Functional** | Needs real estate market | 3-4 pw |
| `imports_exports.rs` | ~80 | **Functional** | Minor improvements | 1 pw |
| `loans.rs` | ~150 | **Functional** | Needs bond issuance | 1-2 pw |
| `policies.rs` | ~100 | **Functional** | Needs tradeoff system | 2-3 pw |
| `districts.rs` | ~120 | **Functional** | Needs per-district policies | 2-3 pw |
| `tourism.rs` | ~80 | **Functional** | Minor improvements | 1 pw |
| `airport.rs` | ~60 | **Functional** | Minor improvements | 0.5 pw |
| `outside_connections.rs` | ~80 | **Functional** | Minor improvements | 0.5 pw |
| `specialization.rs` | ~100 | **Functional** | Minor improvements | 1 pw |
| `wealth.rs` | ~60 | **Functional** | Minor improvements | 0.5 pw |
| `unlocks.rs` | ~80 | **Functional** | Needs tech tree system | 2-3 pw |
| `achievements.rs` | ~100 | **Functional** | Needs Steam integration | 1-2 pw |
| `events.rs` | ~150 | **Functional** | Needs state-driven events | 2-3 pw |
| `advisors.rs` | ~100 | **Functional** | Needs faction alignment | 2-3 pw |
| `homelessness.rs` | ~100 | **Functional** | Minor improvements | 0.5 pw |
| `welfare.rs` | ~60 | **Functional** | Minor improvements | 0.5 pw |
| `immigration.rs` | ~100 | **Functional** | Minor tuning | 0.5 pw |
| `death_care.rs` | ~100 | **Functional** | Minor improvements | 0.5 pw |
| `postal.rs` | ~80 | **Functional** | Minor improvements | 0.5 pw |
| `heating.rs` | ~100 | **Functional** | Minor improvements | 0.5 pw |
| `road_maintenance.rs` | ~120 | **Functional** | Needs S-curve degradation | 1-2 pw |
| `stats.rs` | ~100 | **Functional** | Needs more metrics | 1 pw |
| `time_of_day.rs` | ~80 | **Functional** | Complete | 0 pw |
| `spatial_grid.rs` | ~80 | **Functional** | Complete | 0 pw |
| `virtual_population.rs` | ~80 | **Functional** | Needs serialization | 0.5 pw |
| `lod.rs` | ~150 | **Functional** | Minor improvements | 0.5 pw |
| `contraction_hierarchy.rs` | ~??? | **Dead code** | Either implement or remove | 1 pw |

**Summary:** 68 modules, of which approximately 55 are at least "Functional" state. 13 are "Basic" or "Stub" state. No module is truly "Not started" -- every listed module file exists and contains working code. The simulation crate is remarkably comprehensive for a prototype. Total estimated work to bring all existing modules to research-doc targets: approximately 95-130 person-weeks. This does not include wholly new systems (transit lines, modding SDK, multiplayer).

---

### 5.2 Rendering Crate (`crates/rendering/src/`)

| Module | State | Gap vs Research | Work to Target |
|--------|-------|-----------------|----------------|
| `camera.rs` | **Functional** | Needs smoothing, bookmarks, follow mode | 3-4 pw |
| `input.rs` | **Functional** | Needs undo/redo, multi-select, context menus | 4-5 pw |
| `cursor_preview.rs` | **Functional** | Minor improvements | 0.5 pw |
| `terrain_render.rs` | **Functional** | Needs 3D heightmap, LOD per chunk | 3-4 pw |
| `building_render.rs` | **Functional** | Needs variety, multi-cell support | 3-4 pw |
| `building_meshes.rs` | **Functional** | Needs more models, procedural variation | 4-6 pw |
| `citizen_render.rs` | **Functional** | Minor improvements | 0.5 pw |
| `road_render.rs` | **Functional** | Needs lane markings, intersection detail | 2-3 pw |
| `overlay.rs` | **Functional** | Needs better color ramps, legends | 1-2 pw |
| `day_night.rs` | **Functional** | Needs sky dome, cloud shadows | 2-3 pw |
| `status_icons.rs` | **Functional** | Minor improvements | 0.5 pw |
| `props.rs` | **Functional** | Needs more prop types | 1-2 pw |

**Summary:** 12 modules, all functional. Total estimated work to target: 25-35 person-weeks. The rendering crate is solid; the main gaps are visual variety (building models) and 3D terrain.

---

### 5.3 UI Crate (`crates/ui/src/`)

| Module | State | Gap vs Research | Work to Target |
|--------|-------|-----------------|----------------|
| `lib.rs` | **Functional** | Plugin setup, complete | 0 pw |
| `toolbar.rs` | **Functional** | Needs more tools, categories | 2-3 pw |
| `info_panel.rs` | **Functional** | Needs budget breakdown, service detail | 3-4 pw |
| `milestones.rs` | **Functional** | Needs tech tree UI | 2-3 pw |
| `graphs.rs` | **Functional** | Needs more graph types | 1-2 pw |
| `theme.rs` | **Functional** | Minor polish | 0.5 pw |

**Summary:** 6 modules, all functional. Total estimated work to target: 9-13 person-weeks. The UI is serviceable but will need significant expansion for T2+ features (transit line UI, pipe placement UI, policy UI, election UI).

---

### 5.4 Save Crate (`crates/save/src/`)

| Module | State | Gap vs Research | Work to Target |
|--------|-------|-----------------|----------------|
| `lib.rs` | **Functional** | Needs autosave, version migration | 2-3 pw |
| `serialization.rs` | **Functional** | Missing fields (timer, paths, virt pop) | 2-3 pw |

**Summary:** 2 modules, functional but with known gaps. Total estimated work: 4-6 person-weeks for M2 targets, 6-10 person-weeks for full M5 targets (cloud save, compression, integrity checking).

---

### 5.5 App Crate (`crates/app/src/`)

| Module | State | Notes |
|--------|-------|-------|
| `main.rs` | **Functional** | Bevy App setup, plugin registration |

The app crate is a thin wrapper. It will need modification when adding new crate-level plugins (audio, modding) but is otherwise complete.

---

## 6. Cross-System Integration Points

The most complex bugs and balance issues occur where systems interact. This section documents the key integration chains and their current state.

---

### 6.1 The Development Chain

**Flow:** Road Placement -> Zone Eligibility -> Building Spawn -> Citizen Move-in -> Tax Revenue -> Service Funding -> Happiness -> More Immigration

This is the primary positive feedback loop. It already works end-to-end in the current codebase:

1. **Road Placement** (`input.rs` -> `road_segments.rs`): Player draws road, Bezier segment created, rasterized to grid cells, `RoadNetwork` updated, `CsrGraph` rebuilt.
2. **Zone Eligibility** (`grid.rs`): Cells within 5 manhattan distance of road become zone-eligible. Player paints zones.
3. **Building Spawn** (`buildings.rs`): `building_spawner` runs on timer, finds eligible zoned cells without buildings, spawns `Building` entity with level 1.
4. **Citizen Move-in** (`citizen_spawner.rs`): When residential buildings have capacity and zone demand exists, new citizens are spawned with `HomeLocation` pointing to the building.
5. **Tax Revenue** (`economy.rs`): `collect_taxes` counts citizens, applies per-citizen tax rate, adds commercial building income. Revenue enters treasury.
6. **Service Funding** (`economy.rs`): Service expenses deducted from treasury based on building count.
7. **Happiness** (`happiness.rs`): `update_happiness` computes per-citizen score from services, commute, environment, economy.
8. **Immigration** (`immigration.rs`): `CityAttractiveness` computed from happiness, services, employment. `immigration_wave` spawns new citizens.

**Integration risks:**
- Building spawner iterates full grid per zone type (performance at scale)
- Tax is partially per-citizen flat rate rather than property-based (unrealistic)
- No feedback from happiness to zone demand (happy city should attract more development)
- Immigration wave spawns citizens immediately rather than queuing for next residential building
- No capacity check on services (hospital with 10 beds serves unlimited citizens)

---

### 6.2 The Land Value Chain

**Flow:** Services + Transit + Environment + Buildings + Crime -> Land Value -> Building Level -> Property Tax -> Revenue -> More Services

This is the value creation cycle. It partially works:

1. **Service coverage** (`happiness.rs`): `ServiceCoverageGrid` marks cells within service radii. Works.
2. **Transit accessibility**: Does not exist. No transit system, no accessibility scoring.
3. **Environment** (`pollution.rs`, `noise.rs`, `trees.rs`): Pollution and noise reduce land value, trees increase it. Works but simple.
4. **Building quality**: Higher-level buildings increase neighbor land value slightly. Works.
5. **Crime** (`crime.rs`): Crime reduces land value. Works.
6. **Land value computation** (`land_value.rs`): Resets to 50 each cycle, adds/subtracts factors. Works but no smoothing, no history, no momentum.
7. **Building level-up** (`building_upgrade.rs`): Checks land value thresholds (>120 for level 2, >180 for level 3). Works.
8. **Property tax**: Currently per-citizen, not property-based. Broken link in chain.
9. **Revenue to services**: Service funding from budget. Works.

**Integration risks:**
- Land value resets to base 50 every cycle -- no persistence, no momentum, no gradual change
- No accessibility component (distance to jobs, transit)
- Property tax not based on land value (the most important link is missing)
- Building level-up thresholds use raw u8 land value -- sensitive to rebalancing
- No view corridor bonus (water views, park views)
- No neighborhood spillover (high-value buildings should raise neighbor values via diffusion)

---

### 6.3 The Traffic Nexus

**Flow:** Citizen Commuting + Freight + Transit + Road Capacity -> Traffic Density -> Commute Time -> Happiness + Pollution + Road Wear + Economic Productivity

This chain partially works:

1. **Citizen movement** (`movement.rs`): Citizens follow A* paths on road network. Positions update each tick. Works.
2. **Traffic density** (`traffic.rs`): `update_traffic_density` counts citizens on road cells. Works but crude (citizen count, not vehicle-equivalent).
3. **Commute time** (`happiness.rs`): Commute time factored into happiness. Works but uses simple distance, not traffic-aware travel time.
4. **Pollution** (`pollution.rs`): Traffic density contributes to air pollution. Works.
5. **Road wear** (`road_maintenance.rs`): Traffic degrades road condition. Works.
6. **Freight**: Does not exist. No goods movement on road network.
7. **Transit**: Does not exist. No ridership, no mode split.
8. **Capacity**: No per-road capacity limit. Traffic density is tracked but not capped.

**Integration risks:**
- No BPR function: commute time does not increase nonlinearly with congestion
- Traffic density is not used in pathfinding (traffic-aware pathfinding exists but uses a simple penalty)
- No feedback from congestion to citizen behavior (no route switching, no departure time shifting)
- Missing freight creates unrealistic traffic patterns (all traffic is commuter traffic)
- No parking component (citizens teleport from road to building)

---

### 6.4 The Happiness Web

**Flow:** Housing + Commute + Services + Environment + Economy + Safety -> Happiness -> Immigration + Emigration + Building Upgrade + Crime + Productivity

This is the most connected integration point. It works broadly:

1. **Housing quality**: Based on building level and zone type. Works.
2. **Commute** (`happiness.rs`): Distance-based commute penalty. Works but not traffic-aware.
3. **Service coverage** (`happiness.rs`): Bitflag-based coverage check. Works.
4. **Environment**: Pollution and noise reduce happiness. Works.
5. **Economy**: Tax rate and employment status affect happiness. Works.
6. **Safety**: Crime level reduces happiness. Works.
7. **Happiness output** (`happiness.rs`): Weighted sum of all factors produces per-citizen happiness. Works.
8. **Immigration** (`immigration.rs`): Average happiness feeds attractiveness. Works.
9. **Emigration** (`lifecycle.rs`): Very unhappy citizens leave. Works.
10. **Building upgrade** (`building_upgrade.rs`): Happiness threshold for upgrades. Works.
11. **Crime** (`crime.rs`): Unhappiness contributes to crime rate. Partially works.
12. **Productivity**: Not implemented (no output-per-worker calculation).

**Integration risks:**
- Happiness formula weights are hardcoded and may not be well-balanced
- No commute time calculation from traffic -- just raw distance
- Missing factors: weather happiness, wealth satisfaction, social needs fulfillment, political satisfaction
- All factors contribute linearly -- no diminishing returns or critical thresholds
- Happiness updates on slow timer (every 100 ticks) -- can feel laggy

---

### 6.5 The Emergency Services Chain

**Flow:** Random Event (fire, crime, accident, disaster) -> Service Building Coverage Check -> Response (extinguish, arrest, treat, repair) -> Outcome (damage, citizen health, happiness)

This chain partially works for fire, less for others:

1. **Fire** (`fire.rs`): Random fires start, spread to neighbors, fire stations reduce spread in coverage area, extinguish after time. Building damage and destruction. Works end-to-end.
2. **Crime** (`crime.rs`): Crime generates from conditions but there is no police response simulation (no dispatch, no arrest, no courts).
3. **Health** (`health.rs`): Health grid affects citizen health stat. No ambulance dispatch, no hospital capacity.
4. **Accidents** (`traffic_accidents.rs`): Accidents spawn from traffic density, processed with response time. Basic but functional.
5. **Disasters** (`disasters.rs`): Random disasters (earthquake, flood, tornado) apply area damage. No evacuation, no warning, no preparedness.

**Integration risks:**
- Fire is the only service with actual gameplay simulation (spread + extinguish)
- Police, health, education are coverage-radius checks only, no active response
- No dispatch simulation for any service (no vehicles on road network)
- Disaster response is passive (damage happens regardless of preparedness)

---

### 6.6 The Water/Power/Utility Chain

**Flow:** Utility Placement -> Propagation (BFS) -> Cell Coverage Flags -> Building Functionality -> Citizen Happiness -> Abandonment if Missing

This chain works but is too simple:

1. **Placement** (`lib.rs` init, `input.rs`): Utility sources placed on grid.
2. **Propagation** (`utilities.rs`): BFS flood-fill from sources sets `has_power` / `has_water` on cells within range.
3. **Building check** (`abandonment.rs`): Buildings without power/water trigger abandonment warnings.
4. **Happiness** (`happiness.rs`): Utility coverage affects happiness.

**Integration risks:**
- No demand/supply balance (a single power plant powers infinite buildings)
- No capacity limits (water tower serves unlimited population)
- No pipe/cable routing (pure range-based coverage)
- No blackout/water shortage events
- No cost differentiation between utility types (coal cheap but polluting, solar clean but intermittent)

---

## 7. Risk Assessment

### 7.1 Performance at Scale (100K+ Citizens)

**Risk level: HIGH**

**Current state:** The game spawns 10K citizens with the Tel Aviv initial layout. Virtual population allows representing more citizens statistically. The LOD system compresses off-screen citizens. The CSR graph handles A* efficiently. Path requests are capped at 64/tick.

**Known bottlenecks:**
- Building spawner iterates full 256x256 grid per zone type per tick
- Land value resets and recomputes all 65K cells every slow tick
- Pollution diffusion iterates all 65K cells
- Happiness updates all citizens (no spatial partitioning for service coverage lookup)
- Traffic density counts citizens on each road cell (O(N) per citizen)

**Mitigation strategies from research:**
- Archetype-based ECS batching (already using Bevy ECS)
- Spatial partitioning for service coverage (use `SpatialGrid` more aggressively)
- Incremental land value updates (only recompute changed cells, not full grid)
- Event-driven building spawning (eligible-cell list instead of full grid scan)
- Traffic aggregation by road segment rather than per-cell
- Virtual population ratio scaling (more virtual, fewer real at high population)
- GPU compute for grid-wide operations (pollution diffusion, land value propagation)

**Estimated work to achieve 100K real entities at 60fps:** 4-6 person-weeks of optimization work, primarily in spatial indexing and eliminating full-grid scans.

---

### 7.2 Economic Balance

**Risk level: HIGH**

**Current state:** The economy is a simple income-minus-expenses model that tends to stabilize. Once the player builds enough citizens and services, the budget becomes permanently positive. There is no economic pressure beyond the initial phase.

**Balance risks:**
- Per-citizen flat tax scales linearly, never creating diminishing returns
- Service costs are per-building, not per-population, making services cheaper per capita at scale
- No maintenance cost scaling (roads, pipes, buildings don't degrade costlier over time)
- No economic cycles to create periodic fiscal stress
- Loans exist but bankruptcy is easily avoidable
- No inflation to erode purchasing power
- No competing expenditure demands (defense, research, etc.)

**Mitigation strategies:**
- Property tax with assessment creates nonlinear revenue (depends on building quality AND land value)
- Service costs should scale with population (hospital serving 10K costs more than one serving 1K)
- Infrastructure maintenance costs should grow with city age and size
- Economic cycles (boom/bust every 7-10 game-years) prevent permanent stability
- Wage inflation should increase service worker costs
- Mandatory spending categories (pensions, debt service) should grow over time
- Supply/demand dynamics in the real estate market create price volatility

**Estimated work for economic balance:** 6-10 person-weeks of design, implementation, and extensive playtesting.

---

### 7.3 Traffic Simulation Accuracy vs Performance

**Risk level: MEDIUM-HIGH**

**Current state:** Traffic is tracked as a u16 density per cell, updated by counting citizen positions. Pathfinding uses CSR A* with optional traffic-aware cost weighting. This is adequate for 10K citizens but does not model congestion realistically.

**The core tradeoff:** Realistic traffic simulation (agent-based, lane-level, signal-aware) is computationally expensive. Cities: Skylines 2 attempted more realistic traffic and suffered severe performance issues. The challenge is finding the right abstraction level -- detailed enough to create meaningful gameplay, efficient enough to run at 60fps with 100K+ citizens.

**Recommended approach:**
- Use BPR function for link-level travel time (macroscopic, not microscopic)
- Aggregate traffic by road segment, not per-cell (reduces grid operations)
- Use static traffic assignment (equilibrium) updated every slow tick, not dynamic per-timestep
- Reserve lane-level simulation for T5 (stretch goal)
- Focus gameplay on network design (hierarchy, connectivity, bottlenecks) rather than vehicle behavior

**Estimated work for BPR-based traffic:** 3-4 person-weeks. Lane-level would be 8-12 person-weeks additional.

---

### 7.4 Save/Load Stability Across Versions

**Risk level: MEDIUM**

**Current state:** Save files are binary-encoded custom structs. No version field, no migration system. Any change to the save data structures breaks existing saves.

**Risks:**
- Adding new fields to save structs silently breaks deserialization
- Removing or renaming fields causes data loss
- Entity count changes between versions cause remapping failures
- New systems added between versions have no saved state (default initialization may cause issues)
- No backward compatibility guarantee for player saves

**Mitigation strategies from research:**
- Add version field to save file header (implement immediately)
- Write per-version migration functions (v1->v2, v2->v3, etc.)
- Use field-optional encoding (bitcode supports this with `Option<T>`)
- Test save round-trips in CI (save with old version, load with new version)
- Autosave rotation (keep last 3-5 autosaves so one bad save does not destroy progress)
- Checksum verification on load

**Estimated work:** 3-4 person-weeks for robust versioning and migration. Should be done before any public release.

---

### 7.5 Modding API Stability

**Risk level: MEDIUM (long-term)**

**Current state:** No modding API exists. All game logic is in Rust crates with tightly coupled internal types.

**Risks:**
- Internal type changes break mods (if mods depend on specific struct layouts)
- Bevy version upgrades change component storage, system scheduling, and rendering APIs
- Save files with mod-added entities need special handling
- Malicious mods can access host system (native plugins are unsandboxed)
- Mod dependency conflicts can crash the game

**Mitigation strategies:**
- Define mod SDK as a separate crate with a stable public API
- Use semantic versioning for the mod API (breaking changes = major version bump)
- Prefer data-driven modding over code modding (80% of mods should be data files)
- Use WASM for sandboxed scripting (cannot access filesystem, network)
- Maintain a compatibility test suite that runs against all registered mods on API changes

**Estimated work:** This is a T5 concern. 20-30 person-weeks for a full SDK. The immediate priority (T2) is data-driven architecture -- extracting hardcoded values into data files that are trivially moddable.

---

### 7.6 Bevy Engine Stability

**Risk level: MEDIUM**

**Current state:** Bevy is pre-1.0 and makes breaking API changes with every minor version (0.13 -> 0.14 -> 0.15). Each upgrade requires nontrivial migration of component definitions, system scheduling, and rendering APIs.

**Risks:**
- Bevy 0.15/0.16/etc upgrades may require weeks of migration work
- Rendering API changes could invalidate terrain and building rendering code
- ECS storage changes could affect save/load serialization
- Plugin trait changes could break the workspace crate architecture

**Mitigation strategies:**
- Pin to a specific Bevy version and upgrade deliberately (not on every release)
- Maintain a compatibility layer between game code and Bevy API
- Upgrade Bevy during planned maintenance windows, not mid-feature development
- Follow Bevy migration guides immediately when they are published

**Estimated work per Bevy upgrade:** 2-5 person-weeks depending on the scope of breaking changes.

---

### 7.7 Scope Creep

**Risk level: HIGH**

The 18 research documents contain a staggering amount of detail -- easily 5+ years of full-time development if every feature were implemented as described. The risk is that development spreads too thin across too many systems, resulting in 20 half-finished features rather than 10 polished ones.

**Mitigation strategies:**
- Strict tier discipline: complete all T1 before starting T2 features
- Each sprint should improve one system to its next tier, not add new systems
- "Vertical slice" development: implement one feature fully (road -> building -> citizen -> tax) rather than partially implementing many features
- Regular playtesting to identify which systems actually improve the player experience
- Cut T5 features ruthlessly if they threaten T3-T4 quality

---

## Appendix A: Module-Level Status Matrix

State categories:
- **Dead**: Code exists but is never called
- **Stub**: Module declared, minimal or placeholder implementation
- **Basic**: Core data structures exist, basic logic runs, significant functionality missing
- **Functional**: System works end-to-end for its current scope, may need expansion or polish
- **Complete**: System meets its current-tier requirements fully

### Simulation Modules

```
Module                    State         Tier   Priority
------                    -----         ----   --------
grid                      Functional    T0     -
terrain                   Stub          T0     M3
config                    Complete      T0     -
road_segments             Functional    T0     -
roads                     Functional    T0     -
road_graph_csr            Functional    T0     M2 (BPR)
pathfinding_sys           Functional    T0     -
zones                     Functional    T0     M2
buildings                 Functional    T1     M2 (5-level)
building_upgrade          Functional    T1     M2
abandonment               Functional    T1     -
citizen                   Functional    T1     M3 (demographics)
citizen_spawner           Functional    T1     -
movement                  Functional    T1     M2 (BPR travel)
life_simulation           Functional    T1     M2 (fix timer)
lifecycle                 Functional    T1     -
traffic                   Functional    T1     M2 (LOS)
traffic_accidents         Functional    T1     -
economy                   Functional    T1     M2 (property tax)
budget                    Functional    T1     M2
land_value                Functional    T1     M2 (accessibility)
happiness                 Functional    T1     M2 (tune)
services                  Functional    T1     M3 (dispatch)
education                 Functional    T1     M3 (pipeline)
education_jobs            Functional    T1     M3 (wages)
health                    Basic         T1     M3
crime                     Basic         T2     M3
fire                      Functional    T1     M3 (dispatch)
forest_fire               Functional    T2     -
disasters                 Functional    T4     M5
pollution                 Basic         T2     M3 (wind)
noise                     Functional    T2     -
water_pollution           Functional    T2     -
groundwater               Functional    T2     -
weather                   Functional    T2     M3
wind                      Basic         T2     M3
trees                     Functional    T2     -
natural_resources         Functional    T2     -
utilities                 Basic         T2     M3 (demand/supply)
production                Functional    T2     M3
market                    Functional    T2     M3
imports_exports           Functional    T2     -
loans                     Functional    T1     -
policies                  Functional    T2     M3
districts                 Functional    T2     M3
tourism                   Functional    T2     -
airport                   Functional    T2     -
outside_connections       Functional    T2     -
specialization            Functional    T2     -
wealth                    Functional    T2     -
unlocks                   Functional    T1     M2
achievements              Functional    T4     M5
events                    Functional    T2     M3
advisors                  Functional    T2     M4
homelessness              Functional    T2     -
welfare                   Functional    T2     -
immigration               Functional    T1     M2
death_care                Functional    T2     -
postal                    Functional    T2     -
heating                   Functional    T2     -
road_maintenance          Functional    T2     -
stats                     Functional    T1     -
time_of_day               Functional    T0     -
spatial_grid              Functional    T0     -
virtual_population        Functional    T1     M2 (serialize)
lod                       Functional    T1     -
contraction_hierarchy     Dead          -      Remove or implement
```

### Rendering Modules

```
Module                    State         Tier   Priority
------                    -----         ----   --------
camera                    Functional    T0     M2 (smoothing)
input                     Functional    T0     M4 (undo/redo)
cursor_preview            Functional    T0     -
terrain_render            Functional    T0     M3 (3D height)
building_render           Functional    T1     M2 (variety)
building_meshes           Functional    T1     M2 (more models)
citizen_render            Functional    T1     -
road_render               Functional    T0     M3 (lane marks)
overlay                   Functional    T1     M2 (legends)
day_night                 Functional    T2     -
status_icons              Functional    T1     -
props                     Functional    T2     -
```

### UI Modules

```
Module                    State         Tier   Priority
------                    -----         ----   --------
toolbar                   Functional    T1     M2 (budget panel)
info_panel                Functional    T1     M2 (detail views)
milestones                Functional    T1     M3 (tech tree)
graphs                    Functional    T1     M2 (more metrics)
theme                     Functional    T1     -
```

### Save Modules

```
Module                    State         Tier   Priority
------                    -----         ----   --------
lib (save plugin)         Functional    T1     M2 (autosave)
serialization             Functional    T1     M2 (version, fixes)
```

---

## Appendix B: Research Document Coverage Map

This maps each research document to the systems it primarily informs, with the percentage of its content already reflected in the codebase.

| Research Document | Primary Systems | Codebase Coverage | Key Unimplemented Ideas |
|-------------------|----------------|-------------------|-------------------------|
| `infrastructure_engineering.md` | Roads, Transit, Water, Power | ~30% | BPR function, LOS, transit ops, pipe pressure, grid balancing |
| `game_design_mechanics.md` | Progression, QoL, UX | ~25% | Tech trees, challenge modes, era progression, NG+ |
| `community_wishlists.md` | All systems | ~20% | Lane-level traffic, mixed-use, parking, bicycle, undo |
| `indie_competitors.md` | Economy, Politics, Production | ~15% | Faction system, construction materials, era progression |
| `cities_skylines_analysis.md` | Zoning, Traffic, Economy, Services | ~35% | 5-level buildings, over-education, DLC systems |
| `economic_simulation.md` | Economy, Budget, Land Value | ~25% | Property tax, TIF, bonds, economic cycles, rent |
| `transportation_simulation.md` | Traffic, Transit, Freight | ~20% | BPR, intersection capacity, freight, parking, mode choice |
| `urban_planning_zoning.md` | Zoning, Buildings, Land Use | ~20% | Form-based codes, FAR, NIMBY, walkability scoring |
| `environment_climate.md` | Pollution, Weather, Power, Water, Waste | ~30% | Gaussian plume, stormwater, grid balancing, recycling |
| `historical_demographics_services.md` | Citizens, Services, Growth | ~15% | Demographics depth, service capacity, era growth |
| `social_agent_simulation.md` | Citizens, Crime, Health, Politics | ~15% | Extended demographics, segregation, politics, disease |
| `camera_controls_ux.md` | Camera, Controls, Overlays | ~35% | Smoothing, follow mode, first-person, controller |
| `underground_infrastructure.md` | Water, Sewer, Power, Metro | ~10% | Pipe networks, underground view, metro tunnels |
| `endgame_replayability.md` | Endgame, Progression, Scenarios | ~10% | Escalating challenges, mega-projects, scenarios |
| `modding_architecture.md` | Modding, Data-Driven | ~5% | Everything (no modding exists) |
| `save_system_architecture.md` | Save/Load | ~40% | Version migration, autosave, cloud, compression |
| `procedural_terrain.md` | Terrain, Grid | ~10% | fBm, erosion, biomes, terrain modification |
| `sound_design.md` | Sound, Music | ~0% | Everything (no audio exists) |

**Overall codebase coverage of research document content: approximately 20-25%.** The foundation is solid, but the majority of the depth described in the research documents remains to be implemented. The existing codebase represents roughly M1 completion and partial M2 completion. Reaching M3 requires implementing approximately 75% of the remaining research content. M4-M6 require the rest plus new systems not yet conceived.

---

## Appendix C: Quick Reference -- What to Build Next

Based on the dependency graph, priority tiers, and current codebase state, here is the recommended build order for the next 3-4 months of development (assuming a team of 1-2 developers):

### Sprint 1 (Weeks 1-4): M2 Foundation Fixes
1. Fix save serialization gaps (LifeSimTimer, PathCache, VirtualPopulation)
2. Add save file version header and first migration function
3. Implement autosave with 3-slot rotation
4. Add camera smoothing (exponential lerp)
5. Expand building levels from 3 to 5

### Sprint 2 (Weeks 5-8): M2 Economy Overhaul
1. Property tax system replacing per-citizen flat tax
2. Budget UI with detailed income/expense breakdown
3. LOS A-F traffic grading with visual overlay
4. BPR function in pathfinding edge weights
5. Power demand/supply balance with brownout events

### Sprint 3 (Weeks 9-12): M2 Depth Pass
1. Water demand/supply with shortage effects
2. Service capacity limits (hospital beds, school seats)
3. Improved zone demand (market-driven with price signals)
4. Building mesh variety (2-3 variants per type per level)
5. Construction cost deduction from budget

### Sprint 4 (Weeks 13-16): M3 Transit and Terrain
1. Bus transit system (line drawing, stops, vehicles, ridership)
2. Mode choice (citizens evaluate car vs bus vs walk)
3. fBm terrain generation with seed
4. 3D terrain rendering (vertex Y from elevation)
5. Wind-aware pollution dispersion

This sequence addresses the highest-impact gaps first (save stability, economic realism, traffic feedback, transit) while building toward the M3 feature-complete milestone.

---

*Document generated from analysis of 18 research documents and the complete Megacity codebase.*
*Last updated: 2026-02-18*
