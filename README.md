# Megacity

A city builder simulation built with [Bevy](https://bevyengine.org/) (Rust ECS game engine). Simulates a living city with 10,000+ citizens who commute, work, age, marry, and respond to the urban environment you create.

![City overview with coastline, road network, and UI panels](screenshots/city-overview.png)

![Close-up 3D view with building inspector and skyscrapers](screenshots/city-closeup.png)

![Aerial view of the coastline and river](screenshots/coastline-aerial.png)

![Zoomed-in view showing zone colors and building detail](screenshots/zones-detail.png)

## Features

### City Simulation
- **256x256 grid world** with terrain, coastlines, rivers, and distinct neighborhoods
- **Zoning system** with 6 zone types: Residential Low/High, Commercial Low/High, Industrial, and Office
- **Building lifecycle** - construction phase, upgrades (up to level 5), abandonment, and demolition
- **Districts** with per-district statistics and policy overrides

### Citizens
- **Individual agent simulation** - each citizen has age, gender, education, personality traits, health, happiness, savings, and needs
- **State machine movement** - citizens commute between home and work along road paths, with activities at each location
- **Life events** - aging, education advancement, job seeking, marriage, children, retirement, death
- **LOD system** - Full/Simplified/Abstract tiers with virtual population scaling to simulate 1M+ citizens while only tracking ~10K entities
- **Immigration and emigration** driven by city attractiveness metrics

### Transportation
- **Bezier curve road segments** - smooth roads with Local, Avenue, Boulevard, Highway, and Path types
- **CSR graph pathfinding** with A* and traffic-aware routing
- **Traffic density simulation** with congestion modeling
- **Road maintenance** - degradation from traffic, repair budgets, condition tracking
- **Traffic accidents** spawning from high-congestion areas

### Economy
- **Tax collection** with property tax system
- **City budget** with treasury, income, and expenditure tracking
- **Extended budget** with per-category breakdowns
- **Loan system** with credit ratings, interest rates, and bankruptcy events
- **Import/export trade** via outside connections
- **Production chains** - industrial buildings produce goods, market prices fluctuate
- **Wealth tracking** across the population

### Services & Infrastructure
- **Power and water** utility networks with propagation radius
- **Fire system** - random fires, fire spread, fire station coverage, extinguishing
- **Crime simulation** with police coverage reducing crime rates
- **Health system** with hospital coverage and citizen health tracking
- **Education** - elementary schools, high schools, universities with coverage grids
- **Death care** processing
- **Postal service** coverage
- **Garbage collection** and waste management
- **Heating** grid for cold weather

### Environment
- **Weather system** with temperature, precipitation, and wind
- **Pollution grid** from industrial zones and traffic
- **Noise pollution** from roads and industry
- **Water pollution** with health penalties
- **Groundwater** simulation with quality tracking
- **Land value** grid influenced by services, pollution, and proximity
- **Natural resources** - generation and extraction
- **Forest fires** spreading from regular fires to tree areas
- **Trees** with environmental effects

### Disasters & Events
- **Natural disasters** - earthquakes with structural damage
- **Random city events** with active effects on the simulation
- **Achievement system** tracking milestones
- **Advisor panel** with contextual suggestions

### Game Systems
- **Save/load** with bitcode serialization and file versioning with migration support
- **Unlocks** and development points progression
- **City specializations** computed from economic mix
- **Policies** system for city-wide rules
- **Tourism** simulation with airport connections
- **Homelessness** and welfare systems

## Architecture

```
crates/
  app/          Entry point, asset loading, window setup
  simulation/   All game logic: citizens, economy, services, environment
  rendering/    Bevy rendering: meshes, terrain, roads, overlays, props
  ui/           egui-based UI: toolbar, info panels, budget views
  save/         Save/load serialization with versioning
```

The simulation runs on a fixed 10Hz timestep. Rendering runs at full frame rate with LOD-based culling. A spatial grid enables O(1) lookups for nearest-destination queries.

Road geometry uses cubic Bezier curves stored in `RoadSegmentStore` (source of truth), rasterized to the grid for cell-level queries and indexed in a CSR graph for pathfinding.

## Building & Running

Requires Rust (stable) and system dependencies for Bevy:

```bash
# macOS
brew install cmake

# Ubuntu/Debian
sudo apt-get install -y libasound2-dev libudev-dev pkg-config libwayland-dev

# Build and run
cargo run --release -p app
```

## License

All rights reserved.
