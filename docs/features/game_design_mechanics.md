# Game Design Mechanics Research

Feature research for city builder game design. Focused on fun, engagement, and good
game design rather than pure realism. Organized by design domain with concrete
implementation ideas.

---

## Table of Contents

1. [Progression & Unlocks](#1-progression--unlocks)
2. [Challenge Modes](#2-challenge-modes)
3. [Player Expression](#3-player-expression)
4. [Social & Multiplayer](#4-social--multiplayer)
5. [UI/UX](#5-uiux)
6. [Simulation Depth](#6-simulation-depth)
7. [Quality of Life](#7-quality-of-life)
8. [Monetization Models](#8-monetization-models)
9. [Performance & Scale](#9-performance--scale)
10. [Audio & Feedback](#10-audio--feedback)
11. [Mod Support](#11-mod-support)

---

## 1. Progression & Unlocks

### 1.1 Milestone Systems

The gold standard in the genre is a two-layer progression: **milestones** gate access to
services and tools, while **development/tech trees** let players specialize within those
unlocked categories.

**Cities: Skylines II approach:**
- 20 milestones from "Tiny Village" to "Megapolis"
- Expansion Points (XP) earned both passively (population growth, happiness) and
  actively (placing buildings, constructing roads, upgrading services)
- Each milestone grants: monetary reward, Development Points, Expansion Permits, new
  city services, policies, and management options
- Development Trees per service let you spend points to unlock advanced buildings

**Design principles that work:**
- Gate complexity, not fun. Early game should feel good, not stripped-down.
- Active XP (for building things) keeps players engaged. Passive XP (population,
  happiness) rewards good city management.
- Let players save Development Points for later -- do not force immediate spending.
- Avoid population-only gates. Mixed criteria (population + happiness + services built)
  feels more rewarding.

### 1.2 Tech Trees & Development Trees

**Branching specialization:**
- Each service category (fire, police, education, healthcare, transport) gets its own
  mini tech tree
- Players can specialize: pour all points into education for a university city, or
  spread evenly for a balanced metropolis
- Unlock tiers within each tree: Basic -> Advanced -> Specialized -> Landmark

**Cross-domain tech trees (Anno-style):**
- Research nodes that unlock cross-map abilities (e.g., trade routes, global stockpiles)
- Tech that changes how existing systems work (e.g., automated garbage collection,
  smart traffic lights)
- Political/ideological tech paths (Tropico-style) that unlock different governance
  tools

### 1.3 Era Progression

**Time period advancement:**
- Start in a historical or simple era, progress through technological ages
- Each era changes: available buildings, citizen expectations, visual style, music
- Era transitions could be milestone-gated or triggered by specific achievements
- Visual transformation of the city as it ages (wooden -> brick -> concrete -> glass)

**Implementation idea -- "City Ages":**
- Age 1: Settlement (basic housing, dirt roads, well water)
- Age 2: Town (paved roads, electricity, basic services)
- Age 3: City (highways, public transit, skyscrapers)
- Age 4: Metropolis (metro systems, smart infrastructure, landmark buildings)
- Age 5: Megacity (automated systems, arcologies, monorails)
- Each age changes the visual palette, available buildings, and citizen expectations

### 1.4 Prestige & New Game+ Mechanics

**Against the Storm model (roguelike meta-progression):**
- Each "run" is a temporary settlement; failing or succeeding feeds into a permanent
  meta-city
- Meta-currency spent on permanent upgrades that make future runs easier
- Meta-progression has a ceiling so it cannot trivialize the game

**Prestige reset model (idle-game inspired):**
- "Prestige" your city to start fresh with bonuses: faster building, more starting cash,
  cosmetic unlocks
- Each prestige tier unlocks a new challenge modifier or visual theme
- Prestige level visible on leaderboards and in city stats

**New Game+ for city builders:**
- Restart with harder conditions but keep certain unlocks (cosmetics, blueprints,
  knowledge of citizen needs)
- Harder citizen expectations, more frequent disasters, tighter budgets
- Unlockable difficulty modifiers chosen at NG+ start

### 1.5 Achievement Systems

**Types of achievements:**
- **Milestone achievements**: reach population thresholds, build specific landmarks
- **Challenge achievements**: survive a disaster with zero casualties, maintain 90%+
  happiness for 10 years
- **Creative achievements**: build a city with no cars, create a park larger than 100
  tiles
- **Hidden achievements**: discovered through emergent play (e.g., "every citizen named
  after a famous person")
- **Progressive achievements**: bronze/silver/gold tiers for the same goal

**Achievement rewards that feel meaningful:**
- Unlock cosmetic building variants (art deco fire station, gothic library)
- Unlock new city themes/color palettes
- Unlock advisor voice lines or personalities
- Unlock sandbox mode options (unlimited money, all buildings, custom disasters)

### 1.6 Unlockable Buildings

**Unlock categories:**
- **Population-gated**: need X citizens to unlock civic center
- **Service-gated**: need level 3 education to unlock university
- **Achievement-gated**: build 50 parks to unlock botanical garden
- **Tech-tree-gated**: research renewable energy to unlock solar farm
- **Secret/hidden**: discover through exploration or specific build patterns

**Signature/Landmark buildings:**
- Unique, one-per-city buildings with powerful effects and dramatic visuals
- Require specific conditions to unlock (e.g., 95% happiness + 100k population)
- Provide area-of-effect bonuses and serve as visual city centerpieces

---

## 2. Challenge Modes

### 2.1 Scenario System

**Pre-built scenarios with specific win conditions:**
- "Revive Detroit": start with a bankrupt, depopulated city; reach 50k happy citizens
- "Island Paradise": limited land, tourism-only economy, hurricane season
- "Arctic Outpost": extreme cold, supply drops only, survive 20 years
- "Traffic Nightmare": existing city with terrible traffic; reduce average commute by 50%
- "Green City": zero emissions within 15 years

**Scenario components:**
- Fixed starting map with pre-placed elements
- Specific win/lose conditions with timers
- Modified rules (no highways, limited budget, restricted building types)
- Star rating system (1-3 stars based on how well you exceed goals)
- Community-created scenarios via editor

### 2.2 Roguelike Elements

**Against the Storm is the definitive model here.** Key design lessons:

- **"The city is your avatar"**: the settlement itself is the thing you level up, lose,
  and rebuild -- not individual characters
- **Random blueprint draws**: you do not get to pick which buildings are available each
  run; you must adapt to what you are given
- **Session length matters**: ~1 hour per settlement keeps things tight and prevents the
  "achieved stability, now bored" problem
- **Environmental pressure**: storms, seasons, and forest dangers keep pushing the player
  out of equilibrium
- **Meta-progression with a ceiling**: permanent upgrades exist but cap out, so late-game
  skill matters more than grind

**Roguelike elements adaptable to a traditional city builder:**
- **Weekly challenge runs**: same seed, same starting conditions, leaderboard-ranked
- **Random event cards**: each month/year, draw an event that modifies gameplay
- **Blueprint drafting**: at each milestone, choose 1 of 3 random building unlocks
- **Seasonal modifiers**: each in-game year has randomly generated seasonal effects

### 2.3 Time-Limited Goals

- **Quarterly objectives**: "build 3 schools this quarter" for bonus funds
- **Emergency challenges**: "tornado approaching in 5 minutes -- evacuate the district"
- **Investor demands**: "reach 10k population in 2 years or lose funding"
- **Timed scenarios**: complete an objective within a real-time limit

### 2.4 Difficulty Modes

- **Sandbox**: unlimited money, all buildings, no disasters, pure creative mode
- **Normal**: balanced difficulty, gradual progression
- **Hard**: tighter budgets, more demanding citizens, frequent disasters
- **Survival**: limited resources, no loans, permadeath for city if bankrupt
- **Custom difficulty**: individual sliders for each difficulty factor (budget multiplier,
  disaster frequency, citizen patience, etc.)

### 2.5 Crisis Management Scenarios

- **Pandemic**: healthcare system stress test, quarantine zones, economic impact
- **Economic recession**: revenue drops 40%, must cut services or take loans
- **Infrastructure collapse**: bridge failure, water main break, power grid overload
- **Natural disasters**: earthquake, flood, tornado, wildfire -- each requires different
  preparation and response
- **Political crisis**: faction unrest (Frostpunk 2 model), protests, strikes

---

## 3. Player Expression

### 3.1 Building Customization

**Visual style options:**
- Multiple architectural styles per building type (modern, classical, art deco,
  brutalist, futuristic)
- Color palette customization for roofs, walls, and accents
- Building material selection (brick, glass, concrete, wood) that affects appearance
- Facade details: balconies, awnings, signage, rooftop gardens

**Functional customization:**
- Interior layout for key buildings (Metropolis 1998 model: draw rooms, place furniture)
- Building upgrades that visually change the structure (add a floor, expand wings)
- Modular building pieces that snap together

### 3.2 District Theming

- **District style system**: assign an architectural theme to each district
- **Theme packs**: "Old Town", "Business District", "Arts Quarter", "Waterfront",
  "Industrial Heritage"
- **Street furniture**: benches, lampposts, planters, fountains per district style
- **Road surface options**: cobblestone, asphalt, brick, gravel per district
- **Landscaping presets**: tree types, flower beds, hedge styles per district

### 3.3 Props & Decorations

- **Decorative objects**: statues, fountains, murals, clock towers, arches, gates
- **Street-level details**: food carts, newspaper stands, bus shelters, bike racks
- **Seasonal decorations**: holiday lights, autumn leaves, spring flowers
- **Unlockable props**: earned through achievements, milestones, or challenges
- **Prop placement tools**: free placement, snap-to-road, snap-to-building, rotation

### 3.4 Parks & Public Spaces

- **Modular park builder**: combine paths, benches, trees, playgrounds, ponds, sports
  courts, amphitheaters
- **Park templates**: pre-designed layouts that can be customized
- **Community gardens**: functional food production + aesthetic value
- **Plazas and squares**: configurable open spaces with events and markets
- **Waterfront promenades**: boardwalks, piers, marinas

### 3.5 Naming & Identity

- **Name everything**: streets, districts, buildings, parks, bridges, transit lines
- **City flag/logo designer**: simple shape/color editor for a city emblem
- **City motto**: player-written, displayed on city hall and welcome signs
- **Named citizens**: option to name specific citizens and track their lives
- **Memorial naming**: name buildings after citizens who died in disasters

### 3.6 Photography & Recording

**Photo mode (Cities: Skylines II model):**
- Camera body/lens controls (focal length, aperture shape, depth of field)
- Color grading (white balance, brightness, saturation, contrast)
- Weather/atmosphere manipulation (clouds, fog, rain, time of day)
- UI-free screenshots with customizable resolution
- Filter presets (vintage, noir, cinematic, postcard)

**Timelapse recording:**
- Record city growth from founding to megacity as a video
- Configurable speed and camera path
- Export as video file or GIF
- Share directly to community gallery

**Cinematic camera:**
- Keyframed camera paths for smooth flyovers
- Dolly, crane, and orbit movements
- Speed ramping and slow motion
- Export cinematic clips

---

## 4. Social & Multiplayer

### 4.1 City Sharing

- **City export/import**: share save files with full or partial city data
- **City showcase gallery**: upload screenshots, stats, and description to an in-game
  gallery
- **City tours**: other players can load your city as a read-only "tourism" mode and
  explore it
- **City ratings**: players rate each other's cities on aesthetics, efficiency, creativity
- **Featured cities**: weekly highlights curated by community vote or editorial picks

### 4.2 Cooperative Building

**Shared city model (Conan Unconquered / Ymir style):**
- 2-4 players share a single map, each managing different districts
- Resource trading between districts
- Shared infrastructure (power grid, water, transit)
- Cooperative goals (reach combined population of 500k)

**Asynchronous cooperation:**
- Build a district, share it; another player builds an adjacent district
- "City relay": each player gets 10 minutes to build, then passes to the next
- Collaborative challenge scenarios

### 4.3 Competitive Features

- **Leaderboards**: ranked by population, happiness, budget surplus, efficiency metrics
- **Weekly challenge seeds**: everyone plays the same map/scenario, ranked by score
- **Head-to-head**: two players compete on mirrored maps for the best city
- **Speed runs**: fastest time to reach specific milestones
- **City duel**: compete for citizens -- better services attract people from opponent's
  city

### 4.4 Community Features

- **Workshop/mod sharing**: Steam Workshop or custom platform for mods, assets, maps
- **Screenshot feed**: social media-style feed of player screenshots
- **Blueprint marketplace**: share and download building templates and district layouts
- **Map sharing**: upload custom maps and terrain for others to play
- **Community events**: monthly themes (e.g., "Green Cities Month" with special
  achievements)

---

## 5. UI/UX

### 5.1 Data Visualization

**Heatmap overlays (toggle on/off):**
- Population density
- Land value
- Traffic congestion / flow
- Pollution (air, noise, water)
- Crime rates
- Fire risk
- Happiness / citizen satisfaction
- Power/water coverage
- Education access
- Healthcare access
- Service coverage (police, fire, garbage)

**Design principles for heatmaps:**
- Warm colors (red/orange) = high intensity/problem areas
- Cool colors (blue/green) = low intensity/healthy areas
- Semi-transparent overlay that does not obscure the city
- Toggle between different overlays; never show more than one at a time
- Smooth interpolation between data points, not blocky per-cell coloring

**Charts and graphs:**
- Population growth over time (line chart)
- Budget breakdown (stacked bar chart or pie chart)
- Income vs. expenses trend line
- Happiness by district (bar chart)
- Service coverage statistics
- Comparative before/after charts
- Export data as CSV for hardcore players

### 5.2 Advisor System

**Multiple advisor personalities:**
- **City Planner**: warns about zoning issues, suggests optimal building placement
- **Finance Minister**: budget warnings, tax optimization suggestions
- **Transportation Chief**: traffic hotspot alerts, transit route suggestions
- **Environmental Officer**: pollution warnings, green space recommendations
- **Fire Chief / Police Chief**: coverage gaps, risk zone alerts
- **Education Director**: school capacity issues, literacy rates

**Advisor behavior design:**
- Proactive: warn before a crisis happens ("Traffic on Main Street is approaching
  capacity")
- Reactive: explain what went wrong after a failure ("The fire spread because this
  district has no fire station within range")
- Contextual: appear only when relevant to the player's current action
- Dismissible: never block gameplay; appear as sidebar notifications
- Configurable: players can disable specific advisors or set notification thresholds
- Personality: advisors can have opinions that sometimes conflict, making the player
  the tiebreaker

### 5.3 Notification System

**Notification tiers:**
- **Critical (red)**: budget crisis, disaster incoming, major service failure -- requires
  immediate attention, prominent placement
- **Warning (yellow)**: traffic congestion building, happiness dropping, school at
  capacity -- important but not urgent
- **Info (blue)**: milestone approaching, new building unlocked, citizen feedback --
  informational only
- **Success (green)**: milestone reached, achievement unlocked, goal completed --
  celebratory

**Notification UX:**
- Non-blocking: notifications appear in a sidebar or top bar, never cover gameplay
- Clickable: clicking a notification jumps the camera to the relevant location
- Notification log: scrollable history of all notifications with timestamps
- Filters: show/hide by category and severity
- Sound cues: distinct sounds per tier (subtle for info, urgent for critical)

### 5.4 Tutorial System

**Progressive disclosure:**
- Teach one mechanic at a time, in context, as the player needs it
- First road placement triggers road-building tutorial
- First zone placement triggers zoning tutorial
- Never front-load all tutorials at game start

**Tutorial types:**
- **Guided overlays**: highlight the relevant UI element with arrow + short text
- **Advisor-driven**: the City Planner walks you through your first neighborhood
- **Interactive challenges**: "Place a fire station to cover this area" with success
  feedback
- **Reference library**: all tutorials accessible in a help menu for re-reading
- **Tooltip hints**: hover over any UI element for a brief explanation
- **Contextual tips**: appear based on player behavior (e.g., "Did you know you can
  upgrade this road?")

### 5.5 Info Panels & Statistics

**City dashboard:**
- At-a-glance summary: population, happiness, budget, date/time
- Expandable panels for each category
- Trend indicators (arrows showing direction of change)
- Comparison to previous month/year

**Building info panel:**
- Building name, type, and level
- Service radius visualization
- Current usage / capacity
- Upkeep cost
- Effect on surrounding buildings
- Upgrade options

**Citizen info panel (for named/tracked citizens):**
- Name, age, occupation, home address
- Daily routine visualization
- Happiness factors (housing, commute, services, environment)
- Life events timeline

### 5.6 City Milestones Timeline

- Visual timeline showing the history of the city
- Key events: founding, first 1k population, first disaster, milestones reached
- Before/after comparison slider for any two points in time
- Exportable as an image or sharable link

---

## 6. Simulation Depth

### 6.1 Day/Night Cycle Effects

**Gameplay effects (not just cosmetic):**
- Nighttime: reduced traffic, increased crime risk, nightlife districts activate,
  power demand shifts (residential up, commercial down)
- Morning rush: traffic spike, school zones active, buses fill up
- Evening: commercial peak, restaurant/entertainment demand, parks busy
- Night workers: hospitals, police, fire operate 24/7; some industry runs night shifts

**Visual effects:**
- Dynamic lighting: streetlights turn on, building windows glow
- Shadow movement throughout the day
- Golden hour / blue hour atmospheric effects
- Neon signs and commercial lighting at night
- Headlights and taillights on vehicles at night

### 6.2 Seasons

**Seasonal gameplay effects:**
- **Spring**: increased construction speed, parks bloom, tourism uptick
- **Summer**: higher water demand, AC power spike, outdoor events, heat waves possible
- **Autumn**: harvest festivals, leaf color change, school year begins, tourism for
  foliage
- **Winter**: heating demand spike, snow removal needed, icy roads (traffic slowdown),
  holiday events, potential blizzards

**Infrastructure affected by seasons:**
- Road maintenance increases in winter (salt, plowing)
- Parks have seasonal appearances and different visitor patterns
- Solar panels less effective in winter, wind turbines more variable
- Crops and community gardens follow growing seasons
- Construction slows in extreme weather

### 6.3 Citizen Daily Routines

**Simulated daily life:**
- Wake up -> commute -> work -> lunch -> work -> commute -> evening activity -> sleep
- Children: school schedule, playground visits, extracurricular activities
- Elderly: park visits, healthcare appointments, community center
- Different routines for different employment types (office, retail, industrial, service)
- Weekend routines differ from weekdays

**Routine disruptions:**
- Road closures force alternate commutes
- Power outages cancel work/school
- Disasters displace citizens from routines
- Events (festivals, holidays) modify behavior

### 6.4 Named Citizens & Stories

**Citizen identity system:**
- Each citizen has: name, age, family, occupation, home, workplace, happiness factors
- Life events: birth, education, first job, marriage, children, retirement, death
- Personality traits: adventurous, homebody, social, workaholic -- affects behavior
- Citizens remember events: "I survived the great flood of Year 5"

**Story generation:**
- Procedural narrative snippets based on citizen life events
- "Maria Chen moved to Oakville District, got a job at the new tech campus, and is
  thriving with 92% happiness"
- Notification when tracked citizens experience significant events
- Optional "citizen spotlight" feature that highlights an interesting citizen's story
  periodically

### 6.5 Emergent Storytelling

**Banished model -- stories from systems:**
- No scripted narrative; stories emerge from overlapping simulation systems
- A food shortage in winter leads to citizen death leads to labor shortage leads to
  further decline -- or heroic recovery
- The player's decisions create unique narrative arcs every playthrough

**Frostpunk 2 model -- political narrative from simulation:**
- Factions form based on player decisions (the Zeitgeist system: Adaptation/Progress,
  Equality/Merit, Reason/Tradition)
- Communities (moderate) can be persuaded; Factions (radical) vote their conscience
- Laws passed in council ripple through the entire social fabric
- Every political decision has winners and losers, creating dramatic tension
- Faction relations affect trust, which affects governance ability

**Procedural events:**
- Events generated from simulation state: if pollution is high + citizen health is low,
  generate "Mysterious Illness" event
- Events have multiple response options with different trade-offs
- Event chains: one event can trigger follow-up events based on player response
- Events reference actual city data (district names, citizen names, specific buildings)

### 6.6 Weather System

- Weather affects citizen mood, traffic speed, construction time, and power demand
- Rain: slower traffic, reduced outdoor activity, increased indoor activity
- Extreme heat: water demand spike, health risks for elderly, AC power draw
- Snow/ice: road hazards, heating demand, snow plow requirements
- Wind: affects pollution dispersal, wind turbine output, tree growth
- Weather forecast: 3-day forecast displayed in UI, affects player planning

---

## 7. Quality of Life

### 7.1 Undo/Redo

**Essential implementation:**
- Undo/redo for all placement actions (roads, buildings, zones, props)
- Action history stack with configurable depth (default: 50 actions)
- Undo should refund cost of placed objects
- Visual feedback: undone objects ghost/fade before disappearing
- Keyboard shortcuts: Ctrl+Z / Ctrl+Y (or Cmd on Mac)
- Undo grouping: a road drag-draw counts as one undo action, not per-segment

### 7.2 Blueprints

- **Save blueprint**: select an area, save all buildings/roads/zones as a reusable
  template
- **Place blueprint**: stamp a saved blueprint onto a new location with preview overlay
- **Blueprint library**: organized by category (residential block, industrial zone,
  park layout, interchange)
- **Share blueprints**: export to file or upload to community workshop
- **Auto-adjust**: blueprints adapt to terrain height differences when placed
- **Cost preview**: show total cost before placing a blueprint

### 7.3 Copy/Paste Areas

- **Area selection tool**: rectangle or lasso select a portion of the city
- **Copy**: captures all buildings, roads, zones, and props in the selection
- **Paste**: place the copied area with rotation and mirror options
- **Move**: pick up and relocate a section of the city (with cost for moving buildings)

### 7.4 Road Templates

- **Preset road layouts**: roundabout, cloverleaf interchange, grid block, cul-de-sac
- **Custom templates**: save any road configuration as a reusable template
- **Road upgrade tool**: upgrade an existing road type in-place (2-lane -> 4-lane)
  without rebuilding
- **Road style presets**: combine road type + street trees + median + sidewalk width into
  a single "road style"
- **Parallel road tool**: draw two parallel roads at a configurable distance

### 7.5 Auto-Bulldoze

- **Area demolition**: select a rectangular or freeform area to demolish everything
  within it
- **Selective demolition**: choose what to bulldoze (only buildings, only roads, only
  trees, or all)
- **Auto-bulldoze abandoned**: automatically remove buildings that have been abandoned
  for X months
- **Auto-bulldoze burned**: automatically clear buildings destroyed by fire
- **Demolition cost**: show total demolition cost before confirming
- **Confirmation dialog**: "Are you sure?" for large demolitions to prevent accidents

### 7.6 Snap & Grid Tools

- **Snap-to-grid**: buildings and roads align to the global grid
- **Free-form mode**: disable grid snapping for organic/creative layouts
- **Angle snapping**: roads snap to 15/30/45/90 degree angles
- **Guide lines**: visual guides showing alignment with nearby buildings and roads
- **Height snap**: buildings snap to terrain height or can be manually adjusted
- **Road guides**: Cities: Skylines II style -- visual helpers for parallel roads, even
  spacing, and curve alignment

### 7.7 Search & Filter

- **Building search**: type to search all available buildings by name or category
- **Filter by**: cost, category, era, service type, size, unlock status
- **Recently used**: quick access to the last 10 buildings placed
- **Favorites**: star any building for quick access in a favorites toolbar
- **Quick-access toolbar**: customizable hotbar with frequently used tools and buildings

### 7.8 Additional QoL

- **Construction queue**: queue multiple buildings to be placed in order
- **Auto-save**: configurable interval (every 5/10/15 minutes)
- **Multiple save slots**: at least 20 save slots per city
- **Save thumbnails**: visual preview of each save file
- **Performance monitor**: FPS counter, simulation speed indicator, memory usage
- **Keyboard shortcuts**: fully remappable
- **Accessibility**: colorblind modes, scalable UI, screen reader support for menus

---

## 8. Monetization Models

### 8.1 What Works

**Premium base game + substantial DLC (the Paradox model, done right):**
- Full-featured base game that feels complete on its own
- DLC adds genuinely new systems, not content that feels like it was cut from the base
  game
- Expansion packs ($15-30) that add major features (seasons, natural disasters, mass
  transit)
- Cosmetic/content packs ($5-10) for building styles, themes, and props
- Free updates alongside paid DLC to keep non-paying players engaged

**What players will happily pay for:**
- New gameplay systems (transit types, new services, seasons)
- New building sets and architectural styles (cosmetic)
- New maps and scenarios
- New music packs / radio stations
- Expansion packs that add 10+ hours of new content

### 8.2 What Annoys Players

**The Cities: Skylines 2 cautionary tale:**
- Releasing paid DLC before the base game is polished causes massive backlash
- "Bridges and Ports" DLC was criticized as content that should have been in the base game
- "Beach Properties" DLC received such negative reviews it was pulled from the store and
  refunded
- Result: developer shakeup, DLC delays, loss of community trust

**Universal frustrations:**
- Pay-to-win mechanics (buying gameplay advantages)
- Intrusive purchase pop-ups that interrupt gameplay
- Time gates designed to sell "skip" items
- Hidden costs or unclear pricing
- Overpriced cosmetics that feel like they should be earnable
- Paid cosmetics replacing free earnable content
- DLC that feels like cut base-game content
- Aggressive monetization that sacrifices game depth

### 8.3 Player-Friendly Monetization Principles

1. **Ship a complete, polished base game first** -- DLC comes after players trust you
2. **Free updates alongside paid DLC** -- every paid release should come with a free
   patch
3. **Cosmetic purchases only** -- never sell gameplay advantages
4. **Fair pricing** -- $5-10 for content packs, $15-30 for expansions
5. **Transparent** -- clear about what is included in each purchase
6. **Earnable cosmetics coexist with purchasable ones** -- do not hollow out the reward
   loop
7. **Community involvement** -- let modders create content, then hire the best modders to
   make official DLC (Creator Packs model)
8. **Season passes only if content is clearly defined** -- never sell a vague promise

### 8.4 DLC Strategy Recommendations

**Expansion pack cadence:**
- Major expansion every 6-9 months ($20-25)
- Content/cosmetic pack every 3-4 months ($5-10)
- Free update with every paid release
- Community creator packs (revenue shared with modders)

**Expansion pack ideas (each adds a new system):**
- "Seasons & Weather": full seasonal cycle with gameplay effects
- "Mass Transit": trains, metros, trams, ferries, cable cars
- "Natural Disasters": earthquakes, floods, tornadoes + emergency services
- "Tourism & Leisure": hotels, attractions, theme parks, beaches
- "Green Cities": renewable energy, electric vehicles, eco-buildings
- "Campus Life": university system with research bonuses
- "Industries": detailed production chains, resource extraction
- "Airports": international connections, tourism boost
- "Historic Districts": preservation, heritage buildings, cultural tourism

---

## 9. Performance & Scale

### 9.1 LOD Systems

**Essential LOD implementation:**
- Per-mesh LOD: at least 3 levels (full detail, simplified, billboard/impostor)
- LOD transitions based on camera distance, with smooth crossfade to avoid popping
- Citizens: Full model (nearby) -> simplified model (mid-range) -> dots/particles (far)
  -> aggregate representation (very far)
- Buildings: full geometry -> reduced poly -> simplified box with texture -> merged chunk
- Vehicles: full model -> simplified -> headlight/taillight dots -> traffic flow
  visualization

**Cities: Skylines 2 anti-patterns to avoid:**
- Never render full-detail meshes at all distances (CS2 rendered teeth on characters at
  all distances)
- Never skip LOD variants for character meshes entirely
- Never treat every object as a shadow caster regardless of size/distance
- Proper culling is essential: frustum culling, occlusion culling, distance culling

**LOD budget targets:**
- Reduce rendered triangle counts by 70-90% through proper LOD
- Target: 3-5x GPU performance improvement from LOD alone
- Combine mesh LOD + texture LOD (mipmapping) + shader LOD (simplified materials at
  distance)

### 9.2 Simulation vs. Rendering Split

**Architecture (Citybound / ECS model):**
- Simulation runs on its own thread(s), completely independent of rendering
- Rendering reads simulation state but never writes to it
- Simulation can tick at a different rate than rendering (e.g., sim at 10Hz, render at
  60Hz)
- This is already the Bevy ECS model -- leverage it fully

**Data-oriented design:**
- Consolidated actor state in contiguous memory for cache locality
- Batch processing of similar entities (all citizens update together, not interleaved
  with buildings)
- SIMD-friendly data layouts where possible
- Avoid scattered heap allocations for per-entity data

### 9.3 Handling 100K+ Citizens

**Tiered simulation detail:**
- **Individual simulation**: only for citizens near the camera or being tracked
- **Group simulation**: citizens in the same building/vehicle are simulated as a batch
- **Statistical simulation**: distant districts use aggregate statistics instead of
  per-citizen simulation
- **"Growing agents"**: a single agent represents multiple citizens for pathfinding and
  movement

**Citybound approach (Rust, actor-based):**
- Actors and message passing for type-safe, high-performance updates
- Broadcast message handling optimized for millions of receivers
- Data-oriented memory layout inspired by game engine design
- Early prototypes: 400,000 cars in real-time (sufficient for ~4 million population)

**Practical optimizations:**
- Spatial partitioning (grid, quadtree) for O(1) neighbor queries
- Pathfinding caching: do not re-path every frame; cache and invalidate on road changes
- Batch citizen updates: process all citizens in a chunk together
- Async simulation: long-running calculations (pathfinding, economy) run on background
  threads
- Simulation LOD: nearby chunks simulate every tick; distant chunks simulate every
  Nth tick

### 9.4 Streaming & Chunked Loading

- Divide the world into chunks; only fully simulate/render loaded chunks
- Background loading of chunks as camera moves
- Serialize/deserialize chunk state to disk for very large cities
- Priority-based loading: chunks near camera load first, chunks near roads load second

### 9.5 Multithreading Strategy

- **Main thread**: input, UI, orchestration
- **Simulation threads**: citizen AI, economy, services, pathfinding
- **Render thread**: Bevy's built-in render pipeline
- **Background threads**: save/load, asset streaming, analytics

---

## 10. Audio & Feedback

### 10.1 Ambient City Sounds

**Layered ambient system:**
- Base layer: city hum that scales with population density
- Location layers: traffic sounds near roads, construction near building sites, nature
  in parks, water near rivers/coast
- Time-of-day layers: birds in morning, crickets at night, traffic at rush hour
- Weather layers: rain, wind, thunder, snow (muffled sounds)
- Seasonal layers: autumn wind, summer cicadas, winter quiet

**Dynamic mixing:**
- Volume and mix changes based on camera position and zoom level
- Zoomed out: generalized city hum, birds, distant traffic
- Zoomed in: specific sounds for nearby buildings, individual car sounds, conversations
- Indoor/outdoor transition when entering building detail view

### 10.2 Dynamic Music

**Music that reflects city state:**
- **Thriving city**: upbeat, optimistic, major key, full instrumentation
- **Growing city**: energetic, forward-moving, building momentum
- **Struggling city**: more subdued, minor key, reduced instrumentation
- **Crisis**: tense, urgent, percussion-heavy
- **Night**: calm, jazzy, ambient
- **Morning**: fresh, awakening, rising energy

**Adaptive music system:**
- Layer-based: add/remove instrument layers based on city mood
- Seamless transitions between mood states (crossfade, not hard cut)
- Context-sensitive: building placement has satisfying construction sounds, demolition
  has heavier sounds
- Player-configurable: volume sliders for music, SFX, ambient, notifications

### 10.3 Action Feedback Sounds

**Satisfying interaction sounds:**
- Road placement: "click-drag-snap" with satisfying confirmation tone
- Building placement: thud/stamp with construction sound starting
- Zone painting: brush sound with color-coded pitch (residential = warm, commercial =
  bright, industrial = heavy)
- Bulldozing: crunch/crash with debris sounds
- Upgrade: ascending tone, "level up" feel
- Milestone reached: celebratory fanfare
- Error/invalid placement: gentle negative tone (not harsh or punishing)
- Menu interactions: subtle clicks and hover sounds

### 10.4 Notification Sounds

- **Critical alert**: urgent tone, cuts through other audio
- **Warning**: attention-getting but not alarming
- **Info**: gentle chime
- **Achievement**: triumphant short fanfare
- **Milestone**: extended fanfare with visual celebration
- Each notification category should have a distinct but cohesive sound

### 10.5 Citizen Feedback System

**"Chirper" style social feed (Cities: Skylines model):**
- Citizens post messages about city conditions
- Messages reflect real simulation data: "Love the new park in my district!" or "Traffic
  on 5th Street is unbearable"
- Mix of complaint, praise, humor, and flavor text
- Clickable: click a message to jump to the relevant location
- Filterable: show only complaints, only praise, or all
- Configurable: can be minimized or disabled entirely

**Citizen voice system:**
- When zoomed in close to citizens, hear occasional voice barks
- Happy citizens: humming, chatting, laughing
- Unhappy citizens: grumbling, sighing, complaining
- Context-specific: construction workers shouting, children playing, traffic honking

---

## 11. Mod Support

### 11.1 What Modders Want

Based on the Cities: Skylines modding community, modders consistently want:

1. **Rich programming APIs** with broad access to game code and systems
2. **Stable mod dependency management** that survives game updates without breaking mods
3. **Easy asset importing** for buildings, textures, roads, vehicles, and props
4. **In-game editors** for maps, assets, and buildings
5. **Official mod platform integration** (Steam Workshop, mod.io, or custom)
6. **Documentation**: clear, maintained API docs with examples
7. **Mod loading without load order issues** (dependency resolution, not ordered lists)
8. **Performance-aware modding tools** that guide modders toward efficient implementations

### 11.2 Most Popular Mod Categories

Based on Cities: Skylines and Anno 1800 communities:

1. **Custom buildings and assets**: new building models, textures, and variants (the
   single most popular category)
2. **Gameplay tweaks**: modified game rules, balance changes, new mechanics
3. **Infrastructure mods**: new road types, transit options, utility systems
4. **UI improvements**: better info panels, additional overlays, quality-of-life tools
5. **Visual overhauls**: lighting mods, weather mods, color grading, LOD improvements
6. **Map templates and terrain**: new maps, terrain brushes, pre-built geography
7. **Building skins**: alternative appearances for existing buildings
8. **Production chains**: modified or new resource processing chains
9. **Quality of life tools**: undo/redo, move buildings, better bulldoze
10. **Realism mods**: more realistic traffic, economics, citizen behavior

### 11.3 Scripting API Design

**API layers:**
- **Data API**: read/write access to game state (buildings, citizens, roads, zones,
  economy)
- **Event API**: subscribe to game events (building placed, citizen born, disaster
  started)
- **UI API**: create custom UI panels, overlays, and tools
- **Rendering API**: add custom shaders, visual effects, and overlays
- **Simulation API**: hook into simulation ticks, add custom simulation systems

**Language choice:**
- Scripting language for simple mods (Lua, Rhai, or similar)
- Full Rust API for performance-critical mods (via dynamic linking or WASM)
- Configuration files (TOML/JSON) for simple data-only mods (building stats, balance
  tweaks)

### 11.4 Asset Importing

**Asset pipeline:**
- Import 3D models (glTF, FBX, OBJ) as building/prop/vehicle assets
- Texture import (PNG, JPEG) for custom skins and decals
- Audio import (WAV, OGG) for custom sound effects and music
- Automatic LOD generation from imported high-poly models
- Material/shader assignment in an in-game editor
- Collision mesh auto-generation or manual specification
- Preview tool: see imported assets in-game before publishing

### 11.5 Map Editor

**Map editor features:**
- Terrain sculpting: raise, lower, smooth, flatten, paint textures
- Water placement: rivers, lakes, coastlines with adjustable water level
- Resource placement: mark natural resource deposits
- Starting conditions: place initial roads, buildings, and infrastructure
- Climate settings: temperature range, rainfall, wind patterns
- Scenario conditions: set win/lose conditions, starting budget, available buildings
- Test mode: play-test the map without leaving the editor

### 11.6 Mod Distribution

- **Official mod portal**: in-game browser for discovering, installing, and updating mods
- **One-click install**: no manual file management
- **Mod collections**: curated sets of compatible mods
- **Mod ratings and reviews**: community feedback
- **Creator tools**: upload, version, and manage mods
- **Revenue sharing**: option for creators to monetize quality mods (creator packs)
- **Compatibility checking**: warn about conflicting mods before installation
- **Mod sandboxing**: mods cannot access filesystem or network beyond defined boundaries

---

## Appendix: Key Reference Games

| Game | Key Lesson |
|---|---|
| Cities: Skylines I | Mod support and community are everything; the base game was "good enough" but mods made it legendary |
| Cities: Skylines II | Do not ship unoptimized, do not charge for DLC before the base game is polished |
| Against the Storm | Roguelike structure solves the "achieved stability = bored" problem; meta-progression with a ceiling |
| Frostpunk 2 | Political faction system creates emergent narrative and meaningful trade-offs |
| Anno 1800 | Deep production chains + beautiful world + official mod support (late but welcomed) |
| Manor Lords | Historical authenticity and visual beauty drive engagement even in early access |
| Timberborn | Unique non-human factions and water mechanics prove that theme innovation works |
| The Wandering Village | Building on a moving creature proves unconventional settings attract attention |
| Dorfromantik | Puzzle-city hybrid proves relaxed, score-based city builders have a huge audience |
| Banished | Emergent storytelling from simple systems; no tech tree can work if resource management is compelling |
| Tropico 6 | Political/humor angle; dictator fantasy as an engagement driver |
| Foundation | Organic, grid-free building with strong QoL updates (undo/redo, part replacement) |

---

## Appendix: Priority Feature Matrix

Features ranked by estimated **player impact** vs. **implementation effort**:

### High Impact, Lower Effort
- Undo/redo system
- Notification tiers with click-to-jump
- Heatmap overlays (leverage existing simulation data)
- Achievement system with cosmetic rewards
- Photo mode (camera controls + UI hide)
- Day/night visual cycle
- Dynamic ambient sound layers
- Building search and favorites toolbar

### High Impact, Medium Effort
- Milestone-based progression system
- Development trees per service
- Advisor system with contextual warnings
- Blueprint save/load system
- Scenario system with pre-built challenges
- Named citizens with life events
- District theming
- Seasonal visual changes

### High Impact, Higher Effort
- Full seasonal gameplay effects
- Roguelike challenge mode with meta-progression
- Cooperative multiplayer
- Deep mod/scripting API
- Asset import pipeline
- Political faction system (Frostpunk 2 style)
- Procedural event/narrative system
- Cinematic camera with export
- Citizen daily routine simulation
- Map editor

### Lower Impact, Lower Effort (Nice to Have)
- City flag/logo designer
- Street naming
- Screenshot gallery
- Leaderboards for challenge scenarios
- Music mood transitions
- Citizen voice barks
- Seasonal decorations
- Construction queue
