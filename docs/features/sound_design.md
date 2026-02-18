# Sound Design and Spatial Audio

## Deep Feature Research for Megacity

This document provides comprehensive design specifications, algorithms, mixing strategies,
and Bevy ECS integration patterns for implementing a complete audio system in Megacity.
Sound is one of the most underutilized feedback channels in city builders -- done well,
it communicates simulation state subconsciously (traffic noise tells you about congestion
without opening an overlay), reinforces the emotional weight of player decisions, and
transforms a visual spreadsheet into a living world.

Every section is designed to map directly onto Megacity's existing 256x256 grid, its
`GameClock` (24-hour cycle with seasons), its `Weather` resource, its `TrafficGrid`,
`NoisePollutionGrid`, `FireGrid`, `ActiveDisaster`, `EventJournal`, and its orbital
camera system (`OrbitCamera` with distance 20-4000 units).

---

## Table of Contents

1. [Audio Architecture Overview](#1-audio-architecture-overview)
   - 1.1 Why bevy_kira_audio Over bevy_audio
   - 1.2 Audio Bus Hierarchy
   - 1.3 Listener Model
   - 1.4 Performance Budget
   - 1.5 Audio LOD System
2. [Spatial Audio for City Soundscapes](#2-spatial-audio-for-city-soundscapes)
   - 2.1 Zone-Based Ambient Layers
   - 2.2 Traffic Audio
   - 2.3 Construction Site Audio
   - 2.4 Distance Attenuation Model
   - 2.5 Sound Occlusion and Urban Canyon Effects
   - 2.6 Audio Emitter Component Design
   - 2.7 Chunk-Based Audio Aggregation
3. [Dynamic Music System](#3-dynamic-music-system)
   - 3.1 Adaptive Music Philosophy
   - 3.2 Vertical Layering (Stem-Based Mixing)
   - 3.3 Horizontal Re-Sequencing
   - 3.4 City State Parameters
   - 3.5 Time-of-Day Musical Palettes
   - 3.6 Crisis and Event Music
   - 3.7 Stinger System
   - 3.8 Transition Smoothness
   - 3.9 Reference Implementations Analysis
4. [Environmental Audio](#4-environmental-audio)
   - 4.1 Weather Sounds
   - 4.2 Seasonal Ambience
   - 4.3 Water Body Audio
   - 4.4 Day/Night Cycle Audio
   - 4.5 Disaster Sounds
   - 4.6 Wind Simulation Audio
5. [Notification and UI Sounds](#5-notification-and-ui-sounds)
   - 5.1 Sound Priority Hierarchy
   - 5.2 Earcon Design Principles
   - 5.3 Volume Hierarchy and Ducking
   - 5.4 Cooldown and Spam Prevention
   - 5.5 Accessibility Requirements
6. [Tool and Interaction Sounds](#6-tool-and-interaction-sounds)
   - 6.1 Road Placement
   - 6.2 Zoning
   - 6.3 Bulldoze
   - 6.4 Building Placement
   - 6.5 Menu Navigation
   - 6.6 Camera Movement
7. [Procedural Audio Generation](#7-procedural-audio-generation)
   - 7.1 Why Procedural Audio
   - 7.2 Traffic Hum Synthesis
   - 7.3 Rain and Weather Synthesis
   - 7.4 Wind Synthesis
   - 7.5 Crowd Murmur Synthesis
   - 7.6 Hybrid Approach
8. [Technical Implementation in Bevy](#8-technical-implementation-in-bevy)
   - 8.1 Plugin Architecture
   - 8.2 Audio Resource and Component Design
   - 8.3 System Scheduling
   - 8.4 Memory Management and Streaming
   - 8.5 Audio Asset Pipeline
   - 8.6 Format Selection
   - 8.7 Spatial Audio Implementation
   - 8.8 Save/Load Considerations
9. [Mixing and Mastering](#9-mixing-and-mastering)
   - 9.1 Loudness Normalization
   - 9.2 Dynamic Range Management
   - 9.3 Frequency Band Allocation
   - 9.4 Ducking Chains
   - 9.5 Master Bus Processing
10. [What Makes City Builder Audio Great](#10-what-makes-city-builder-audio-great)
    - 10.1 SimCity 4
    - 10.2 Cities: Skylines
    - 10.3 Frostpunk
    - 10.4 Anno 1800
    - 10.5 Stardew Valley
    - 10.6 Key Insights for Megacity
11. [Sound Asset Catalog](#11-sound-asset-catalog)
12. [ECS Integration Architecture](#12-ecs-integration-architecture)
13. [Performance Considerations](#13-performance-considerations)

---

## 1. Audio Architecture Overview

The audio system in Megacity is not an afterthought bolted onto a visual simulation -- it is
a first-class feedback channel that runs in parallel with rendering and simulation. This section
establishes the foundational architecture that every subsequent section builds upon.

### 1.1 Why bevy_kira_audio Over bevy_audio

Bevy's built-in `bevy_audio` plugin wraps `rodio` and provides basic play/stop/volume
control. For a city builder with layered music, spatial ambience, and dozens of simultaneous
sound sources, it falls short in several critical areas:

| Feature | bevy_audio (rodio) | bevy_kira_audio (Kira) |
|---|---|---|
| Spatial audio | Manual panning only | Built-in 3D emitter/listener |
| Crossfading | Not supported | Native tween-based crossfades |
| Stem layering | Manual multi-track | Multiple tracks with independent control |
| Audio buses | None | Hierarchical mixer tracks |
| Loop regions | Basic loop | Configurable loop start/end points |
| Playback tweening | None | Volume/pitch/panning tweens with easing |
| Clock sync | None | Clock-quantized playback triggers |
| Real-time effects | None | Filters, reverb (via Kira) |
| Streaming | Full decode on load | On-demand streaming for long audio |
| Latency | Higher (rodio queue) | Lower (~5ms configurable) |

**Recommendation:** Use `bevy_kira_audio` (wrapping the Kira audio library, version 3.x+)
as the primary audio backend. Kira was specifically designed for game audio with features
like clock-quantized transitions and hierarchical mixer routing that map directly onto
our adaptive music requirements.

**Dependency:** `bevy_kira_audio = "0.20"` (for Bevy 0.14+). The crate provides
`AudioPlugin` which replaces Bevy's default audio plugin.

### 1.2 Audio Bus Hierarchy

All audio in Megacity routes through a hierarchical bus (mixer track) structure. Each bus
has independent volume, optional effects (reverb, filter, compression), and can be
individually muted by the player in the settings menu.

```
Master Bus (final output)
+-- Music Bus
|   +-- Base Layer Track
|   +-- Harmonic Layer Track
|   +-- Rhythmic Layer Track
|   +-- Melodic Layer Track
|   +-- Atmospheric Pad Track
|   +-- Stinger Track
+-- Ambience Bus
|   +-- Zone Ambience Sub-bus
|   |   +-- Residential Layer
|   |   +-- Commercial Layer
|   |   +-- Industrial Layer
|   |   +-- Park/Nature Layer
|   +-- Weather Sub-bus
|   |   +-- Rain Layer
|   |   +-- Wind Layer
|   |   +-- Thunder Layer
|   +-- Traffic Sub-bus
|   |   +-- Road Hum Layer
|   |   +-- Horn/Event Layer
|   |   +-- Construction Layer
|   +-- Environmental Sub-bus
|       +-- Water Bodies
|       +-- Wildlife
|       +-- Day/Night Cycle
+-- SFX Bus
|   +-- Tool Sounds
|   +-- Building Events
|   +-- Disaster Sounds
|   +-- One-Shot Effects
+-- UI Bus
    +-- Notification Sounds
    +-- Menu Navigation
    +-- Button Clicks
```

Each bus stores its volume as a `f32` in `[0.0, 1.0]` range. The effective volume of any
sound is the product of its own volume and every ancestor bus volume:

```
effective_volume = sound_volume * track_volume * sub_bus_volume * bus_volume * master_volume
```

**Kira implementation:** Each bus maps to a Kira `TrackHandle`. Sub-buses are child tracks
of parent tracks. Volume changes propagate automatically through the hierarchy.

```rust
// Pseudocode for bus setup
let master = audio_manager.add_sub_track(TrackBuilder::new())?;
let music_bus = audio_manager.add_sub_track(
    TrackBuilder::new().routes(TrackRoutes::parent(master))
)?;
let ambience_bus = audio_manager.add_sub_track(
    TrackBuilder::new().routes(TrackRoutes::parent(master))
)?;
let sfx_bus = audio_manager.add_sub_track(
    TrackBuilder::new().routes(TrackRoutes::parent(master))
)?;
let ui_bus = audio_manager.add_sub_track(
    TrackBuilder::new().routes(TrackRoutes::parent(master))
)?;
```

### 1.3 Listener Model

The audio listener represents the "ears" of the player. In Megacity, the listener is
attached to the camera, specifically the `OrbitCamera` focus point projected to the ground
plane, with distance-based adjustments:

```rust
/// The listener position is the camera's focus point (where it looks at on the ground).
/// This means zooming in/out changes which sounds are prominent, not the listener Y.
fn update_audio_listener(
    orbit: Res<OrbitCamera>,
    mut listener: ResMut<AudioListener>,
) {
    listener.position = orbit.focus;
    // The "audible radius" scales with camera distance.
    // At max zoom-out (4000 units), we hear a wide area at low detail.
    // At min zoom-in (20 units), we hear a small area at high detail.
    listener.audible_radius = orbit.distance * 1.5;
}
```

**Listener radius scaling:** The camera distance directly affects which audio emitters
are active. This creates a natural audio LOD -- zoomed out, you hear the aggregate
city hum; zoomed in, you hear individual dogs barking and cash registers.

| Camera Distance | Audible Radius | Audio Character |
|---|---|---|
| 20-100 | 30-150 | Street level: individual sounds, full detail |
| 100-500 | 150-750 | Neighborhood: blended zone ambience, some individuals |
| 500-2000 | 750-3000 | District: aggregate zone layers, traffic hum |
| 2000-4000 | 3000-6000 | City-wide: overall city drone, music dominant |

### 1.4 Performance Budget

Audio processing must not compete with simulation or rendering for CPU time. The target
budget is **under 2ms per frame** at 60fps (3.3% of frame time).

| Component | Budget | Notes |
|---|---|---|
| Kira audio thread | 0.8ms | Runs on dedicated thread, mixing and effects |
| ECS audio systems | 0.6ms | Emitter culling, distance calc, parameter updates |
| Spatial calculations | 0.3ms | Distance attenuation, occlusion checks |
| Music state machine | 0.1ms | Transition logic, parameter evaluation |
| Headroom | 0.2ms | Safety margin for spikes |
| **Total** | **2.0ms** | |

**Key constraint:** Kira runs its own audio thread separate from Bevy's main thread and
render thread. The ECS systems only need to send commands (play, stop, set volume, set
pitch) to Kira -- they do not process audio samples. This means the 0.6ms ECS budget
is primarily spent on deciding *what* to play, not *how* to play it.

### 1.5 Audio LOD System

Analogous to Megacity's existing visual LOD system (Full/Simplified/Abstract tiers for
citizens), the audio system implements LOD tiers that reduce processing cost at distance:

**Tier 0 -- Full Detail (distance < 150 units, camera distance 20-100):**
- Individual audio emitter components are active
- Per-entity sound selection (specific dog bark, specific car horn)
- Full spatial positioning with HRTF panning
- Sound occlusion raycasting active
- Maximum simultaneous sources: 32

**Tier 1 -- Aggregated (distance 150-750 units, camera distance 100-500):**
- Individual emitters are culled
- Zone-based aggregate layers replace individual sounds
- Spatial positioning is per-chunk (8x8 cell chunks), not per-entity
- No occlusion raycasting (too many sources)
- Aggregate parameters derived from simulation data (traffic density, building count)
- Maximum simultaneous sources: 16

**Tier 2 -- Abstract (distance 750+ units, camera distance 500+):**
- Only zone-level ambient beds play
- Single stereo mix per zone type, volume proportional to zone area visible
- No spatial positioning (stereo only)
- Parameters derived from city-wide stats (`CityStats`)
- Maximum simultaneous sources: 6

**Tier 3 -- Music Only (distance 3000+ units, camera distance 2000+):**
- All spatial audio culled
- Only music bus and a subtle city-wide hum play
- Maximum simultaneous sources: music stems only (5-6)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioLodTier {
    FullDetail,   // < 150 units from listener
    Aggregated,   // 150-750 units
    Abstract,     // 750-3000 units
    MusicOnly,    // 3000+ units
}

impl AudioLodTier {
    pub fn from_camera_distance(distance: f32) -> Self {
        match distance {
            d if d < 100.0 => Self::FullDetail,
            d if d < 500.0 => Self::Aggregated,
            d if d < 2000.0 => Self::Abstract,
            _ => Self::MusicOnly,
        }
    }

    pub fn max_simultaneous_sources(self) -> usize {
        match self {
            Self::FullDetail => 32,
            Self::Aggregated => 16,
            Self::Abstract => 6,
            Self::MusicOnly => 6,
        }
    }
}
```

**Transition smoothing:** When the camera crosses a LOD boundary, audio transitions are
crossfaded over 1.5 seconds to avoid jarring pops. Individual emitters fade out as their
aggregate replacements fade in, and vice versa.

---

## 2. Spatial Audio for City Soundscapes

The soundscape is the sonic identity of the city. Unlike music (which is overlaid) or UI
sounds (which are event-driven), the soundscape is always present and always changing. It
answers the question: "What does this city *sound like* right now, right here?"

### 2.1 Zone-Based Ambient Layers

Each zone type in Megacity has a characteristic ambient sound palette. These are not single
looping tracks but collections of layered elements that combine procedurally based on the
density, time of day, and specific buildings present.

#### 2.1.1 Residential Zone Sounds

Residential zones sound like neighborhoods -- domestic, organic, alive.

**Low-Density Residential (`ZoneType::ResidentialLow`):**

| Sound Element | Frequency | Variations | Time Dependency | Notes |
|---|---|---|---|---|
| Birdsong | Continuous loop | 4 seasonal sets | Dawn peak (5-8am), absent at night | Louder in spring/summer |
| Dog barking | Random interval 30-120s | 3 bark patterns | More at dawn/dusk | Triggered by nearby citizen movement |
| Children playing | Random interval 60-180s | 5 variations | 10am-6pm only, weekdays less | Only if families with children exist |
| Lawn mower | Random interval 300-600s | 2 variations | 9am-5pm, spring/summer only | Probability scales with residential density |
| Wind chimes | Continuous subtle | 1 base layer | All day | Volume scales with wind speed |
| Screen door slam | Random interval 120-240s | 2 variations | 7am-10pm | Very subtle, close zoom only |
| Crickets | Continuous loop | 2 variations | 8pm-5am, spring/summer/autumn | Absent in winter |
| Sprinkler | Random interval 180-360s | 1 variation | 6am-9am, summer | Subtle |
| Car starting/leaving | Random interval 60-180s | 3 variations | Morning/evening commute peaks | Scales with population |

**High-Density Residential (`ZoneType::ResidentialHigh`):**

| Sound Element | Frequency | Variations | Time Dependency | Notes |
|---|---|---|---|---|
| Muffled music/TV | Continuous subtle | 3 variations | Evening peak (6pm-11pm) | Filtered low-pass |
| Baby crying | Random interval 120-300s | 2 variations | More at night | Only if families exist |
| Elevator ding | Random interval 60-120s | 1 variation | All day | Close zoom only |
| Air conditioner hum | Continuous | 1 base layer | Summer only | Volume scales with temp |
| Distant siren | Random interval 180-600s | 3 variations | More at night | Scales with crime rate |
| Apartment bustle | Continuous loop | 2 variations | Peak at 7-9am, 5-7pm | Footsteps, doors, muffled voices |
| Garbage truck | Random interval 300-600s | 1 variation | 6am-8am only | Once per game day |
| Pigeon cooing | Random interval 60-120s | 2 variations | Dawn-dusk | Close zoom only |

#### 2.1.2 Commercial Zone Sounds

Commercial zones are the busiest sonic environments -- human activity at its peak.

**Low-Density Commercial (`ZoneType::CommercialLow`):**

| Sound Element | Frequency | Variations | Time Dependency | Notes |
|---|---|---|---|---|
| Crowd chatter | Continuous loop | 3 density levels | Peak 11am-2pm, 5pm-9pm | Volume scales with commercial activity |
| Cash register / POS beep | Random interval 15-45s | 4 variations | Business hours (8am-10pm) | Higher frequency = more commerce |
| Restaurant clatter | Continuous subtle | 2 variations | 11am-2pm, 6pm-10pm | Dishes, utensils |
| Door bell (shop entry) | Random interval 30-90s | 2 variations | Business hours | Close zoom only |
| Street musician | Random interval 600-1200s | 3 instruments | 10am-8pm | Only in high-happiness areas |
| Coffee machine | Random interval 60-120s | 1 variation | 7am-11am peak | Close zoom only |
| Shopping bags rustling | Random interval 30-60s | 2 variations | 10am-8pm | Very subtle |

**High-Density Commercial (`ZoneType::CommercialHigh`):**

| Sound Element | Frequency | Variations | Time Dependency | Notes |
|---|---|---|---|---|
| Dense crowd murmur | Continuous loop | 3 density levels | 9am-10pm | Louder than low-density |
| Traffic noise blend | Continuous | 1 base layer | All day | From adjacent roads |
| Construction adjacent | Random interval 120-300s | 2 variations | 8am-5pm | If nearby construction |
| Delivery truck | Random interval 120-240s | 2 variations | 5am-9am, 2pm-5pm | Backup beepers |
| HVAC rooftop units | Continuous hum | 1 base layer | All day, louder in summer | Low frequency |
| Neon buzz | Continuous subtle | 1 variation | 6pm-2am | Evening/night only |

#### 2.1.3 Industrial Zone Sounds

Industrial zones are the loudest and most mechanically complex soundscapes.

**Industrial (`ZoneType::Industrial`):**

| Sound Element | Frequency | Variations | Time Dependency | Notes |
|---|---|---|---|---|
| Machinery hum | Continuous loop | 3 factory types | Louder during work hours | Base industrial drone |
| Hammering/banging | Random interval 10-30s | 4 variations | 6am-10pm shift hours | Metallic, sharp |
| Truck backup beeper | Random interval 60-180s | 2 variations | 6am-8pm | Classic "beep beep beep" |
| Forklift | Random interval 120-240s | 2 variations | Work hours | Engine + hydraulics |
| Welding crackle | Random interval 30-90s | 3 variations | Work hours | Close zoom only |
| Steam release | Random interval 60-180s | 2 variations | All day | Hissing sound |
| Conveyor belt | Continuous subtle | 1 variation | Work hours | Rhythmic mechanical |
| Loading dock | Random interval 180-360s | 2 variations | 5am-8pm | Heavy thuds, chains |
| Warning klaxon | Random interval 600-1200s | 1 variation | Work hours | Brief, signals shift change |

#### 2.1.4 Office Zone Sounds

Offices are intentionally quieter -- they represent white-collar work that produces
minimal external noise.

**Office (`ZoneType::Office`):**

| Sound Element | Frequency | Variations | Time Dependency | Notes |
|---|---|---|---|---|
| HVAC system | Continuous subtle | 1 variation | All day | Quiet hum |
| Lobby bustle | Continuous | 2 variations | 8am-9am, 5pm-6pm peaks | Muffled footsteps, elevator |
| Keyboard clatter | Continuous subtle | 1 variation | 9am-5pm | Very subtle, close zoom |
| Printer/copier | Random interval 120-300s | 1 variation | Work hours | Close zoom only |

#### 2.1.5 Park and Nature Sounds

Parks are sonic oases -- they provide relief from urban noise and are among the most
pleasant sounds in the game.

| Sound Element | Frequency | Variations | Time Dependency | Season | Notes |
|---|---|---|---|---|---|
| Birdsong (rich) | Continuous loop | 4 seasonal sets | Dawn chorus peak (5-7am) | Spring/Summer loudest | Multiple species layered |
| Rustling leaves | Continuous | 2 variations | All day | Spring/Summer/Autumn | Volume scales with wind |
| Water fountain | Continuous loop | 2 variations | All day | All (frozen in winter) | Only if park has fountain |
| Duck quacking | Random interval 60-180s | 3 variations | Dawn-dusk | Spring/Summer | Only near water features |
| Jogger footsteps | Random interval 120-240s | 1 variation | 6am-8am, 5pm-7pm | All | Scales with park usage |
| Children laughing | Random interval 90-180s | 3 variations | 10am-6pm | Spring/Summer | If playground in park |
| Bee buzzing | Continuous subtle | 1 variation | 10am-4pm | Summer | Close zoom only |
| Wind through grass | Continuous | 1 variation | All day | Spring/Summer | Scales with wind |
| Owl hooting | Random interval 180-600s | 2 variations | 9pm-4am | All except winter | Night ambience |

#### 2.1.6 Zone Audio Mixing Algorithm

When the listener is near the boundary of multiple zones, sounds from all nearby zones
blend together. The mixing weight for each zone is proportional to the number of cells
of that zone type within the audible radius.

```rust
/// Calculate zone audio weights based on cell composition within listener radius.
fn calculate_zone_weights(
    listener_pos: Vec3,
    audible_radius: f32,
    grid: &WorldGrid,
) -> ZoneAudioWeights {
    let center_x = (listener_pos.x / CELL_SIZE) as i32;
    let center_y = (listener_pos.z / CELL_SIZE) as i32;
    let radius_cells = (audible_radius / CELL_SIZE) as i32;

    let mut residential_low = 0u32;
    let mut residential_high = 0u32;
    let mut commercial_low = 0u32;
    let mut commercial_high = 0u32;
    let mut industrial = 0u32;
    let mut office = 0u32;
    let mut park = 0u32;  // grass cells with no building and trees
    let mut water = 0u32;
    let mut total = 0u32;

    for dy in -radius_cells..=radius_cells {
        for dx in -radius_cells..=radius_cells {
            let x = center_x + dx;
            let y = center_y + dy;
            if x < 0 || y < 0 || x >= GRID_WIDTH as i32 || y >= GRID_HEIGHT as i32 {
                continue;
            }
            // Distance weighting: closer cells contribute more
            let dist_sq = (dx * dx + dy * dy) as f32;
            let radius_sq = (radius_cells * radius_cells) as f32;
            if dist_sq > radius_sq {
                continue;
            }

            let cell = grid.get(x as usize, y as usize);
            total += 1;

            match cell.zone {
                ZoneType::ResidentialLow => residential_low += 1,
                ZoneType::ResidentialHigh => residential_high += 1,
                ZoneType::CommercialLow => commercial_low += 1,
                ZoneType::CommercialHigh => commercial_high += 1,
                ZoneType::Industrial => industrial += 1,
                ZoneType::Office => office += 1,
                ZoneType::None => {
                    if cell.cell_type == CellType::Water {
                        water += 1;
                    } else if cell.cell_type == CellType::Grass {
                        park += 1;
                    }
                }
            }
        }
    }

    if total == 0 { return ZoneAudioWeights::default(); }

    ZoneAudioWeights {
        residential_low: residential_low as f32 / total as f32,
        residential_high: residential_high as f32 / total as f32,
        commercial_low: commercial_low as f32 / total as f32,
        commercial_high: commercial_high as f32 / total as f32,
        industrial: industrial as f32 / total as f32,
        office: office as f32 / total as f32,
        park: park as f32 / total as f32,
        water: water as f32 / total as f32,
    }
}
```

Each weight in `[0.0, 1.0]` drives the volume of that zone's ambient layer. A purely
residential area produces `residential = 1.0, everything else = 0.0`. A mixed-use border
might yield `residential = 0.4, commercial = 0.3, park = 0.3`.

### 2.2 Traffic Audio

Traffic is one of the most important ambient sounds because it directly communicates
simulation state: the player can *hear* congestion without looking at the traffic overlay.

#### 2.2.1 Traffic Sound Components

| Component | Source Data | Sound Character | Volume Mapping |
|---|---|---|---|
| Road hum (base) | `TrafficGrid::density` | Low-frequency drone | `density / 20.0` mapped to volume |
| Engine idle | Congestion > 0.7 | Rumbling, irregular | Fades in above congestion threshold |
| Tire noise | Road type + weather | Whoosh (dry) / hiss (wet) | Pitch scales with `RoadType::speed()` |
| Horn honking | Congestion > 0.8 | Short horn blasts | Random interval decreases with congestion |
| Brake squeal | Congestion spikes | Brief screech | Triggered on rapid density increase |
| Emergency siren | Active fire/crime | Wailing/yelping siren | Only during active emergencies |
| Bus air brake | Transit route cells | Pneumatic hiss | Near bus stops at intervals |

#### 2.2.2 Traffic Volume Formula

The traffic audio volume for a given road cell is derived from the existing `TrafficGrid`:

```
congestion = TrafficGrid::congestion_level(x, y)  // 0.0 to 1.0
road_speed = RoadType::speed()                     // 5 to 100

// Base volume scales with congestion (more cars = louder)
base_volume = congestion * 0.8

// Road type affects frequency content, not just volume
// Highways are louder per-car due to higher speeds
speed_factor = road_speed / 100.0
volume = base_volume * (0.5 + 0.5 * speed_factor)

// Time-of-day modulation
time_mod = match hour {
    0..=5   => 0.2,    // very quiet late night
    6..=8   => 0.9,    // morning commute
    9..=11  => 0.6,    // mid-morning
    12..=13 => 0.7,    // lunch rush
    14..=16 => 0.6,    // afternoon
    17..=19 => 1.0,    // evening commute peak
    20..=22 => 0.4,    // evening wind-down
    23      => 0.3,    // late night
}

final_volume = volume * time_mod

// Pitch also shifts: higher congestion = lower pitch (slower traffic, lower RPM)
pitch = 1.0 - (congestion * 0.3)  // range: 0.7 to 1.0
```

#### 2.2.3 Road Type Sound Characteristics

Each road type has a distinct sonic signature based on its physical characteristics
(lane count, speed, vehicle mix):

| Road Type | Dominant Frequency | Character | Special Sounds |
|---|---|---|---|
| `Path` | None (pedestrian) | Footsteps, bicycle bells | No engine noise at all |
| `Local` | 200-400 Hz | Light cars, domestic | Occasional dog from yards |
| `OneWay` | 200-500 Hz | Similar to Local | Slightly more directional feel |
| `Avenue` | 150-500 Hz | Mixed traffic, medium | Bus air brakes at stops |
| `Boulevard` | 100-600 Hz | Heavy traffic, wide | Trolley/tram bells if transit |
| `Highway` | 80-500 Hz | Constant roar, fast | Truck compression brakes, no pedestrians |

#### 2.2.4 Wet Road Audio

When `Weather::current_event == WeatherEvent::Rain` or `WeatherEvent::Storm`, tire noise
changes character dramatically:

```
// Dry roads: tire whoosh is mid-frequency, subtle
dry_tire_volume = 0.3 * speed_factor

// Wet roads: tire spray adds high-frequency hiss, much louder
wet_tire_volume = 0.6 * speed_factor
wet_spray_volume = 0.4 * speed_factor  // additional "spray" layer

// Also add occasional splash sound when car hits puddle
splash_interval = 10.0 - (congestion * 8.0)  // more frequent in traffic
splash_volume = 0.5
```

#### 2.2.5 Horn Triggering Algorithm

Horns are triggered probabilistically at congested intersections:

```rust
fn should_trigger_horn(
    congestion: f32,
    time_since_last_horn: f32,
    rng_seed: u64,
) -> bool {
    if congestion < 0.6 { return false; }

    // Minimum cooldown prevents horn spam
    let min_cooldown = 3.0; // seconds
    if time_since_last_horn < min_cooldown { return false; }

    // Probability increases with congestion
    // At 0.6 congestion: ~5% chance per second
    // At 1.0 congestion: ~30% chance per second
    let probability = (congestion - 0.6) * 0.75;
    let roll = pseudo_random_f32(rng_seed);

    roll < probability * (1.0 / 60.0) // per-frame probability at 60fps
}
```

### 2.3 Construction Site Audio

When buildings are being constructed (tracked by `BuildingSpawnTimer` or `UpgradeTimer`),
construction audio plays at that grid position. Construction is one of the most recognizable
city-builder sounds and provides important feedback that "something is happening here."

#### 2.3.1 Construction Sound Palette

| Phase | Duration | Sounds | Character |
|---|---|---|---|
| Foundation | First 25% of build time | Excavation rumble, dump trucks, pile driving | Deep, heavy impacts |
| Framing | 25-60% of build time | Hammering, power tools, crane motor | Rhythmic, metallic |
| Finishing | 60-90% of build time | Drilling, welding crackle, compressor | Varied, intermittent |
| Complete | Last 10% | Quieting down, cleanup sounds | Fading out |

#### 2.3.2 Construction Volume Scaling

Construction volume scales with building size (zone density level):

```
// Low-density residential: small house construction
volume_residential_low = 0.4

// High-density residential: apartment building
volume_residential_high = 0.7

// Industrial: factory construction
volume_industrial = 0.8

// Commercial high: skyscraper construction
volume_commercial_high = 0.9

// All construction sounds have working-hours modulation
work_hours_mod = match hour {
    7..=18 => 1.0,    // full activity
    6 | 19 => 0.5,    // startup/shutdown
    _ => 0.0,         // no construction at night
}

final_volume = base_volume * work_hours_mod
```

#### 2.3.3 Construction Audio Emitter

```rust
#[derive(Component)]
pub struct ConstructionAudioEmitter {
    pub grid_x: usize,
    pub grid_y: usize,
    pub build_progress: f32,       // 0.0 to 1.0
    pub building_size: BuildingSize,
    pub phase: ConstructionPhase,
    pub hammer_timer: Timer,       // randomized interval
    pub drill_timer: Timer,
}

#[derive(Debug, Clone, Copy)]
pub enum ConstructionPhase {
    Foundation,  // 0.0 - 0.25
    Framing,     // 0.25 - 0.60
    Finishing,   // 0.60 - 0.90
    Completion,  // 0.90 - 1.00
}

impl ConstructionPhase {
    pub fn from_progress(progress: f32) -> Self {
        match progress {
            p if p < 0.25 => Self::Foundation,
            p if p < 0.60 => Self::Framing,
            p if p < 0.90 => Self::Finishing,
            _ => Self::Completion,
        }
    }
}
```

### 2.4 Distance Attenuation Model

Sound in the real world follows the inverse-square law: intensity decreases proportional
to the square of the distance from the source. In practice, this means a **6 dB reduction
per doubling of distance**.

#### 2.4.1 Core Attenuation Formula

```
// Reference distance: the distance at which the sound is at "full volume"
// Typically 1-2 grid cells (16-32 world units) for point sources
ref_distance = 32.0  // 2 grid cells in world units

// Maximum distance: beyond this, the sound is silent (optimization cutoff)
max_distance = audible_radius  // from AudioLodTier

// Distance from listener to emitter (2D, ignoring Y since city is flat)
distance = sqrt((listener.x - emitter.x)^2 + (listener.z - emitter.z)^2)

// Inverse-square attenuation with reference distance
if distance <= ref_distance:
    attenuation = 1.0
elif distance >= max_distance:
    attenuation = 0.0
else:
    // Classic inverse-square: 6dB per doubling of distance
    attenuation = ref_distance / distance

    // Apply rolloff exponent (1.0 = realistic, <1.0 = slower falloff for gameplay)
    rolloff = 0.8  // slightly slower than realistic for playability
    attenuation = attenuation.powf(rolloff)

    // Smooth fade to zero near max_distance to avoid pop
    fade_start = max_distance * 0.8
    if distance > fade_start:
        fade = 1.0 - (distance - fade_start) / (max_distance - fade_start)
        attenuation *= fade
```

#### 2.4.2 Attenuation Curves by Sound Type

Different sound types use different attenuation parameters because they have different
real-world propagation characteristics:

| Sound Type | Reference Distance | Max Distance | Rolloff | Notes |
|---|---|---|---|---|
| Point source (horn, bark) | 32 units (2 cells) | 400 units (25 cells) | 1.0 | Sharp falloff |
| Area source (zone ambience) | 64 units (4 cells) | 800 units (50 cells) | 0.6 | Gentle falloff |
| Construction | 48 units (3 cells) | 600 units (37 cells) | 0.8 | Medium falloff |
| Traffic hum | 32 units (2 cells) | 500 units (31 cells) | 0.7 | Slightly gentler |
| Disaster (explosion, siren) | 80 units (5 cells) | 2000 units (125 cells) | 0.5 | Very far-reaching |
| Water body | 48 units (3 cells) | 400 units (25 cells) | 0.9 | Near-realistic |
| Weather (rain, wind) | Global | Global | N/A | Non-positional, applies everywhere |
| Music | N/A | N/A | N/A | Non-positional |

#### 2.4.3 dB to Linear Volume Conversion

Kira and most audio APIs work in linear volume (0.0-1.0), but attenuation is naturally
expressed in decibels. Conversion utilities:

```rust
/// Convert decibels to linear volume. 0 dB = 1.0, -6 dB = 0.5, -inf dB = 0.0
fn db_to_linear(db: f32) -> f32 {
    if db <= -80.0 { return 0.0; }
    10.0_f32.powf(db / 20.0)
}

/// Convert linear volume to decibels. 1.0 = 0 dB, 0.5 = -6 dB, 0.0 = -inf dB
fn linear_to_db(linear: f32) -> f32 {
    if linear <= 0.0001 { return -80.0; }
    20.0 * linear.log10()
}

/// 6 dB per doubling of distance
fn distance_attenuation_db(distance: f32, ref_distance: f32) -> f32 {
    if distance <= ref_distance { return 0.0; }
    -20.0 * (distance / ref_distance).log10()
}
```

### 2.5 Sound Occlusion and Urban Canyon Effects

Buildings between the listener and a sound source reduce the perceived volume. This
is critical for making the city feel three-dimensional -- you should hear the highway
on the other side of a row of skyscrapers as muffled and quieter.

#### 2.5.1 Occlusion Raycast Algorithm

For Tier 0 (Full Detail) audio LOD only, we perform a simplified 2D raycast from
listener to emitter on the grid to count intervening buildings:

```rust
/// Count buildings along a line from listener to emitter using Bresenham's line.
/// Returns an occlusion factor in [0.0, 1.0] where 0.0 = fully occluded.
fn calculate_occlusion(
    listener_cell: (usize, usize),
    emitter_cell: (usize, usize),
    grid: &WorldGrid,
) -> f32 {
    let mut occlusion_db = 0.0_f32;

    // Walk cells along the line using Bresenham's algorithm
    for (x, y) in bresenham_line(listener_cell, emitter_cell) {
        if (x, y) == listener_cell || (x, y) == emitter_cell {
            continue; // skip endpoints
        }

        let cell = grid.get(x, y);
        if cell.building_id.is_some() {
            // Each building cell attenuates by 3-8 dB depending on zone type
            let building_attenuation = match cell.zone {
                ZoneType::ResidentialLow => 3.0,    // houses: thin walls
                ZoneType::ResidentialHigh => 6.0,    // apartments: thick concrete
                ZoneType::CommercialLow => 4.0,
                ZoneType::CommercialHigh => 8.0,     // skyscrapers: heavy occlusion
                ZoneType::Industrial => 5.0,         // warehouses: large but thin walls
                ZoneType::Office => 7.0,             // office towers
                _ => 2.0,
            };
            occlusion_db += building_attenuation;
        }
    }

    // Cap total occlusion at -30 dB (effectively silent)
    occlusion_db = occlusion_db.min(30.0);

    // Convert to linear multiplier
    db_to_linear(-occlusion_db)
}
```

#### 2.5.2 Urban Canyon Effect

When the listener is on a road cell flanked by buildings on both sides, a subtle reverb
effect simulates the sound reflecting off building facades. This is the classic "city
canyon" sound that makes dense urban areas acoustically distinct from open suburbs.

**Detection:**
```rust
fn is_urban_canyon(
    listener_cell: (usize, usize),
    grid: &WorldGrid,
) -> (bool, f32) {
    let (x, y) = listener_cell;
    let cell = grid.get(x, y);

    if cell.cell_type != CellType::Road {
        return (false, 0.0);
    }

    // Check if buildings flank the road on opposite sides
    let buildings_north = y > 0 && grid.get(x, y - 1).building_id.is_some();
    let buildings_south = y < GRID_HEIGHT - 1 && grid.get(x, y + 1).building_id.is_some();
    let buildings_east = x < GRID_WIDTH - 1 && grid.get(x + 1, y).building_id.is_some();
    let buildings_west = x > 0 && grid.get(x - 1, y).building_id.is_some();

    let ns_canyon = buildings_north && buildings_south;
    let ew_canyon = buildings_east && buildings_west;

    if ns_canyon || ew_canyon {
        // Canyon intensity based on building density (higher zones = taller buildings = more reverb)
        let avg_density = 0.5; // Could sample actual building heights
        (true, avg_density)
    } else {
        (false, 0.0)
    }
}
```

**Audio effect:** When urban canyon is detected, apply a short reverb (RT60 = 0.3-0.8s)
to the ambience bus. The reverb intensity scales with canyon density.

| Canyon Type | Reverb Time (RT60) | Early Reflections | Character |
|---|---|---|---|
| Suburban street (low-res both sides) | 0.2s | Sparse | Slight echo |
| Urban avenue (mixed-use) | 0.4s | Medium | Noticeable reverb |
| Downtown canyon (commercial high both sides) | 0.8s | Dense | Strong urban reverb |
| Industrial corridor | 0.3s | Medium, metallic | Harsh, reflective |

### 2.6 Audio Emitter Component Design

Every entity or grid region that produces sound carries an audio emitter component.
This is the core ECS pattern for spatial audio.

#### 2.6.1 Point Emitter (Entity-Attached)

```rust
/// Attached to individual entities that produce sound (citizens, vehicles, buildings).
/// Only active when within Tier 0 (Full Detail) audio LOD.
#[derive(Component)]
pub struct AudioEmitter {
    /// World-space position of the sound source.
    pub position: Vec3,

    /// The sound bank this emitter draws from.
    pub sound_bank: SoundBankId,

    /// Current volume before distance attenuation (0.0 to 1.0).
    pub base_volume: f32,

    /// Pitch multiplier (1.0 = normal, 0.5 = octave down, 2.0 = octave up).
    pub pitch: f32,

    /// Reference distance for attenuation (world units).
    pub ref_distance: f32,

    /// Maximum audible distance (world units). Beyond this, emitter is culled.
    pub max_distance: f32,

    /// Whether this emitter is currently playing.
    pub active: bool,

    /// Handle to the Kira sound instance (if playing).
    pub instance_handle: Option<SoundInstanceHandle>,

    /// Cooldown timer to prevent re-triggering too fast.
    pub cooldown: Timer,
}

/// Identifies which collection of sound variations to use.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum SoundBankId {
    DogBark,
    ChildrenPlaying,
    CashRegister,
    MachineryHum,
    CarHorn,
    ConstructionHammer,
    ConstructionDrill,
    BirdSong,
    Siren,
    // ... extensible
}
```

#### 2.6.2 Area Emitter (Chunk-Attached)

```rust
/// Attached to chunk entities (8x8 cell regions) for aggregate zone ambience.
/// Active in Tier 1 (Aggregated) audio LOD.
#[derive(Component)]
pub struct AreaAudioEmitter {
    /// Center position of the chunk in world space.
    pub center: Vec3,

    /// Zone composition weights for this chunk (cached, updated on slow tick).
    pub zone_weights: ZoneAudioWeights,

    /// Traffic density average across roads in this chunk.
    pub avg_traffic_density: f32,

    /// Number of active construction sites in this chunk.
    pub construction_count: u32,

    /// Current playback handles for each active layer.
    pub active_layers: HashMap<ZoneAmbienceLayer, SoundInstanceHandle>,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ZoneAmbienceLayer {
    ResidentialLow,
    ResidentialHigh,
    CommercialLow,
    CommercialHigh,
    Industrial,
    Office,
    Park,
    Water,
    TrafficHum,
    Construction,
}
```

### 2.7 Chunk-Based Audio Aggregation

Megacity already divides the 256x256 grid into 8x8 chunks (32x32 chunks total, CHUNK_SIZE=8).
The audio system reuses this chunking for spatial audio aggregation at Tier 1 LOD.

#### 2.7.1 Chunk Audio Cache

Each chunk maintains a cached audio profile that is recomputed every slow tick
(matching the simulation update rate):

```rust
#[derive(Default)]
pub struct ChunkAudioProfile {
    pub zone_weights: ZoneAudioWeights,
    pub avg_traffic: f32,
    pub max_traffic: f32,
    pub construction_sites: u32,
    pub fire_cells: u32,
    pub has_water: bool,
    pub tree_density: f32,      // 0.0-1.0, affects birdsong volume
    pub building_density: f32,  // 0.0-1.0, affects urban canyon reverb
    pub noise_pollution_avg: f32, // from NoisePollutionGrid
}

fn update_chunk_audio_profiles(
    grid: Res<WorldGrid>,
    traffic: Res<TrafficGrid>,
    noise: Res<NoisePollutionGrid>,
    fire: Res<FireGrid>,
    mut profiles: ResMut<ChunkAudioProfiles>,
) {
    for chunk_y in 0..CHUNKS_PER_AXIS {
        for chunk_x in 0..CHUNKS_PER_AXIS {
            let profile = &mut profiles.chunks[chunk_y * CHUNKS_PER_AXIS + chunk_x];
            *profile = ChunkAudioProfile::default();

            let base_x = chunk_x * CHUNK_SIZE;
            let base_y = chunk_y * CHUNK_SIZE;
            let mut cell_count = 0u32;
            let mut traffic_sum = 0u32;

            for dy in 0..CHUNK_SIZE {
                for dx in 0..CHUNK_SIZE {
                    let x = base_x + dx;
                    let y = base_y + dy;
                    if x >= GRID_WIDTH || y >= GRID_HEIGHT { continue; }

                    let cell = grid.get(x, y);
                    cell_count += 1;

                    // Count zone types
                    match cell.zone {
                        ZoneType::ResidentialLow => profile.zone_weights.residential_low += 1.0,
                        ZoneType::ResidentialHigh => profile.zone_weights.residential_high += 1.0,
                        // ... etc for each zone
                        _ => {}
                    }

                    // Accumulate traffic
                    if cell.cell_type == CellType::Road {
                        traffic_sum += traffic.get(x, y) as u32;
                    }

                    // Check for fire
                    if fire.get(x, y) > 0 {
                        profile.fire_cells += 1;
                    }

                    // Check for water
                    if cell.cell_type == CellType::Water {
                        profile.has_water = true;
                    }
                }
            }

            // Normalize zone weights
            if cell_count > 0 {
                let n = cell_count as f32;
                profile.zone_weights.residential_low /= n;
                profile.zone_weights.residential_high /= n;
                // ... etc
                profile.avg_traffic = traffic_sum as f32 / n;
            }
        }
    }
}
```

#### 2.7.2 Chunk Priority Queue

Not all chunks within the audible radius are equally important. The audio system
maintains a priority queue of chunks sorted by proximity to the listener, and only
the top N chunks get active audio emitters:

```
chunk_priority = 1.0 / (distance_to_listener + 1.0)
                 * (1.0 + zone_weights.max_component())  // boost chunks with strong zone presence
                 * (1.0 + avg_traffic * 0.5)              // boost noisy chunks
```

At Tier 1, the top 8 chunks get active area emitters. At Tier 2, the top 4.

---

## 3. Dynamic Music System

Music in a city builder is not background decoration -- it is the emotional narrator of the
player's story. The best city builder soundtracks (SimCity 4, Frostpunk, Anno 1800) do not
merely play pleasant tracks on shuffle; they *respond* to the state of the city, the time of
day, the player's successes and failures. Megacity's music system must achieve this through
two complementary techniques: **vertical layering** (adding/removing instrument stems to a
continuously-playing arrangement) and **horizontal re-sequencing** (selecting different
musical sections based on game state).

### 3.1 Adaptive Music Philosophy

The fundamental design principle: **the player should never notice the music changing, but
should always feel that it fits.**

Traditional game music approaches and why they fail for city builders:

| Approach | Problem for City Builders |
|---|---|
| Random playlist | No connection to game state; same music during disaster and prosperity |
| State-based tracks | Jarring cuts when switching between tracks; obvious transition points |
| Single ambient loop | Monotonous over long play sessions (city builders are 10+ hour games) |
| Silence | Feels empty; city builders need warmth and atmosphere |

**Megacity's approach:** A hybrid vertical/horizontal system where:
1. Music is always playing (no silence except in menus or if player disables)
2. The *arrangement* (which instruments are active) responds to city state in real-time
3. The *composition* (which section/chord progression/melody) responds to broader state changes
4. Transitions happen at musical boundaries (bar lines) so they always sound intentional
5. The overall energy level tracks with city growth, creating an emotional arc across a session

### 3.2 Vertical Layering (Stem-Based Mixing)

Each musical piece is composed as 5-7 independent stems that can be mixed in and out
independently. All stems share the same tempo, key, and song structure, and are
synchronized to the same transport clock.

#### 3.2.1 Stem Architecture

```
Musical Piece = {
    Base Pad:     Always playing. Sustained chords, atmospheric texture.
                  Instruments: synth pads, strings (pp), organ drones.
                  Role: Provides harmonic foundation and prevents silence.

    Harmonic:     Chord voicings that add warmth and definition.
                  Instruments: piano, guitar, vibraphone, harp.
                  Role: Makes the music feel "musical" vs ambient.
                  Trigger: Population > 500 or happiness > 50.

    Rhythmic:     Gentle pulse that gives the music momentum.
                  Instruments: light percussion, shaker, hi-hat, pizzicato strings.
                  Role: Adds energy and forward motion.
                  Trigger: Population > 2000 or active construction.

    Bass:         Low-end foundation that adds gravity.
                  Instruments: upright bass, synth bass, cello.
                  Role: Makes the music feel grounded and substantial.
                  Trigger: Population > 5000 or industrial zone present.

    Melodic:      The "tune" -- the part the player remembers.
                  Instruments: flute, clarinet, trumpet (muted), piano melody.
                  Role: Creates memorable moments and emotional peaks.
                  Trigger: Happiness > 70 or milestone approaching.

    Accent:       Decorative elements that add color and variation.
                  Instruments: bells, glockenspiel, wind chimes, bird-like synths.
                  Role: Prevents repetition, adds seasonal flavor.
                  Trigger: Random intervals; more in high-happiness states.

    Tension:      Dissonant or urgent elements for negative states.
                  Instruments: low strings tremolo, timpani, dissonant synth.
                  Role: Communicates danger and urgency.
                  Trigger: Disaster active, happiness < 30, budget crisis.
}
```

#### 3.2.2 Stem Volume Control

Each stem's volume is controlled by a parameter value in `[0.0, 1.0]` that is computed
every music tick (once per beat, typically 0.5-1.0 seconds). Volume changes are applied
as Kira tweens with easing to prevent pops:

```rust
#[derive(Resource)]
pub struct MusicMixer {
    pub stems: HashMap<StemType, StemState>,
    pub master_volume: f32,
    pub current_piece: MusicPieceId,
    pub beat_clock: f64,         // current beat position
    pub bpm: f64,                // beats per minute
    pub time_signature: (u8, u8), // e.g., (4, 4)
}

pub struct StemState {
    pub target_volume: f32,      // what we want it to be
    pub current_volume: f32,     // what it is right now (tweening)
    pub handle: Option<SoundInstanceHandle>,
    pub fade_duration: Duration, // how long to tween
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum StemType {
    BasePad,
    Harmonic,
    Rhythmic,
    Bass,
    Melodic,
    Accent,
    Tension,
}
```

#### 3.2.3 Stem Activation Rules

```rust
fn compute_stem_targets(
    stats: &CityStats,
    weather: &Weather,
    clock: &GameClock,
    disaster: &ActiveDisaster,
    budget: &CityBudget,
    effects: &ActiveCityEffects,
) -> HashMap<StemType, f32> {
    let mut targets = HashMap::new();

    // Base pad: always on, volume varies with time of day
    let base_volume = match clock.hour_of_day() {
        0..=4 => 0.4,      // quiet night
        5..=7 => 0.6,      // gentle dawn
        8..=11 => 0.8,     // morning
        12..=16 => 0.9,    // full day
        17..=20 => 0.8,    // evening
        21..=23 => 0.5,    // night
        _ => 0.7,
    };
    targets.insert(StemType::BasePad, base_volume);

    // Harmonic: scales with population and happiness
    let harmonic = if stats.population > 500 {
        let pop_factor = (stats.population as f32 / 10000.0).min(1.0);
        let happy_factor = (stats.average_happiness / 100.0).max(0.0);
        (pop_factor * 0.5 + happy_factor * 0.5).min(1.0)
    } else {
        0.0
    };
    targets.insert(StemType::Harmonic, harmonic);

    // Rhythmic: scales with activity (construction, traffic, time of day)
    let is_active_time = clock.hour_of_day() >= 7 && clock.hour_of_day() <= 20;
    let rhythmic = if is_active_time && stats.population > 2000 {
        let activity = 0.5 + (stats.population as f32 / 50000.0).min(0.5);
        activity
    } else {
        0.0
    };
    targets.insert(StemType::Rhythmic, rhythmic);

    // Bass: scales with city size and industrial presence
    let bass = if stats.population > 5000 {
        let size_factor = (stats.population as f32 / 100000.0).min(0.8);
        let industrial_factor = if stats.industrial_buildings > 0 { 0.2 } else { 0.0 };
        size_factor + industrial_factor
    } else {
        0.0
    };
    targets.insert(StemType::Bass, bass);

    // Melodic: plays during positive moments
    let melodic = if stats.average_happiness > 70.0 && disaster.current.is_none() {
        let joy = ((stats.average_happiness - 70.0) / 30.0).min(1.0);
        joy * 0.8  // cap at 0.8 to leave room for the "special" feel
    } else {
        0.0
    };
    targets.insert(StemType::Melodic, melodic);

    // Accent: occasional pops of color (randomized per beat, more in good states)
    // Handled separately with random triggers, not continuous volume

    // Tension: negative states
    let tension = if disaster.current.is_some() {
        0.9
    } else if effects.epidemic_ticks > 0 {
        0.7
    } else if budget.treasury < 0.0 {
        0.5
    } else if stats.average_happiness < 30.0 {
        0.4
    } else {
        0.0
    };
    targets.insert(StemType::Tension, tension);

    // When tension is high, reduce melodic and accent
    if tension > 0.3 {
        if let Some(m) = targets.get_mut(&StemType::Melodic) {
            *m *= 1.0 - tension;
        }
    }

    targets
}
```

### 3.3 Horizontal Re-Sequencing

While vertical layering controls *which instruments play*, horizontal re-sequencing
controls *what musical section* is playing. This allows the music to have actual
compositional structure rather than being an infinite loop.

#### 3.3.1 Section Types

Each musical piece is divided into sections (like movements in classical music):

| Section | Duration | Character | When to Play |
|---|---|---|---|
| Intro | 16-32 bars | Sparse, atmospheric, establishing | Game start, after load |
| A (Main) | 32-64 bars | The primary theme, warm and inviting | Normal gameplay, prosperity |
| B (Development) | 32-64 bars | Variation on A, more complex | City growing, mid-game |
| C (Contrast) | 16-32 bars | Different mood, provides variety | After long time in A/B |
| Bridge | 8-16 bars | Transitional passage | Between contrasting sections |
| Climax | 16-32 bars | Most energetic, full arrangement | Milestone moment, peak prosperity |
| Reflective | 32-64 bars | Quiet, contemplative | Night time, after disaster, low activity |
| Tension | 16-32 bars | Urgent, dissonant | During disasters, crises |

#### 3.3.2 Section Transition Rules

```rust
pub struct MusicSequencer {
    pub current_section: MusicSection,
    pub bars_in_section: u32,
    pub section_length: u32,       // bars until transition is allowed
    pub transition_pending: Option<MusicSection>,
    pub transition_bar: u32,       // which bar to transition on
}

impl MusicSequencer {
    /// Called every bar boundary. Determines if a section change should occur.
    fn evaluate_transition(
        &mut self,
        city_mood: CityMood,
        clock: &GameClock,
        disaster_active: bool,
    ) {
        self.bars_in_section += 1;

        // Don't transition before minimum section length
        if self.bars_in_section < self.section_length {
            return;
        }

        let next_section = match (self.current_section, city_mood, disaster_active) {
            // Disaster overrides everything
            (_, _, true) => Some(MusicSection::Tension),

            // Night time -> reflective
            (s, _, false) if clock.hour_of_day() >= 22 || clock.hour_of_day() <= 4 =>
                if s != MusicSection::Reflective { Some(MusicSection::Reflective) } else { None },

            // High mood after tension -> climax (relief moment)
            (MusicSection::Tension, CityMood::Thriving, false) =>
                Some(MusicSection::Climax),

            // Normal cycling: A -> B -> C -> Bridge -> A (with randomization)
            (MusicSection::MainA, _, false) if self.bars_in_section > 48 =>
                Some(MusicSection::DevelopmentB),
            (MusicSection::DevelopmentB, _, false) if self.bars_in_section > 48 =>
                Some(MusicSection::ContrastC),
            (MusicSection::ContrastC, _, false) if self.bars_in_section > 24 =>
                Some(MusicSection::Bridge),
            (MusicSection::Bridge, _, false) if self.bars_in_section > 12 =>
                Some(MusicSection::MainA),
            (MusicSection::Climax, _, false) if self.bars_in_section > 24 =>
                Some(MusicSection::MainA),
            (MusicSection::Reflective, _, false)
                if self.bars_in_section > 48 && clock.hour_of_day() >= 5 =>
                Some(MusicSection::Intro),

            _ => None,
        };

        if let Some(next) = next_section {
            // Queue transition for next bar boundary
            self.transition_pending = Some(next);
        }
    }
}
```

#### 3.3.3 Crossfade at Bar Boundaries

Transitions between sections must be quantized to bar boundaries (every 4 beats in 4/4
time) to sound musical. A crossfade of 2-4 beats (one bar) prevents abrupt cuts:

```rust
fn execute_section_transition(
    sequencer: &mut MusicSequencer,
    mixer: &mut MusicMixer,
    audio: &AudioManager,
) {
    let crossfade_duration = Duration::from_secs_f64(
        60.0 / mixer.bpm * 4.0  // one full bar in seconds
    );

    // Fade out current section's stems
    for (_, stem) in mixer.stems.iter_mut() {
        if let Some(handle) = &stem.handle {
            handle.set_volume(
                Volume::Amplitude(0.0),
                Tween {
                    duration: crossfade_duration,
                    easing: Easing::OutCubic,
                    ..Default::default()
                },
            );
        }
    }

    // Start new section's stems with fade-in
    // (new stems are loaded from the next section's asset)
    // ... load and play with volume tween from 0.0 to target
}
```

### 3.4 City State Parameters

The music system reads from several simulation resources to determine its behavior.
These parameters are sampled once per beat (not every frame) to prevent jitter:

```rust
#[derive(Debug, Clone)]
pub struct MusicStateSnapshot {
    // Population scale (drives overall arrangement density)
    pub population_tier: PopulationTier,

    // Emotional state (drives key/mode selection and stem activation)
    pub city_mood: CityMood,

    // Time context (drives time-of-day palette)
    pub time_of_day: TimeOfDayPeriod,

    // Season (drives instrument choice and timbral quality)
    pub season: Season,

    // Crisis state (overrides normal music with tension/urgency)
    pub crisis_level: CrisisLevel,

    // Activity level (drives rhythmic intensity)
    pub activity_level: f32, // 0.0 to 1.0
}

#[derive(Debug, Clone, Copy)]
pub enum PopulationTier {
    Village,     // 0 - 1,000: sparse, intimate
    Town,        // 1,000 - 10,000: growing, hopeful
    City,        // 10,000 - 50,000: established, confident
    Metropolis,  // 50,000 - 200,000: grand, complex
    Megacity,    // 200,000+: epic, overwhelming
}

#[derive(Debug, Clone, Copy)]
pub enum CityMood {
    Struggling,  // happiness < 30, negative trend
    Neutral,     // happiness 30-50
    Content,     // happiness 50-70
    Thriving,    // happiness 70-85
    Euphoric,    // happiness > 85, no crises
}

#[derive(Debug, Clone, Copy)]
pub enum CrisisLevel {
    None,
    Minor,       // budget deficit, low happiness
    Major,       // epidemic, fire spreading
    Catastrophic, // active disaster (tornado, earthquake, flood)
}

impl MusicStateSnapshot {
    pub fn sample(
        stats: &CityStats,
        weather: &Weather,
        clock: &GameClock,
        disaster: &ActiveDisaster,
        budget: &CityBudget,
        effects: &ActiveCityEffects,
    ) -> Self {
        let population_tier = match stats.population {
            0..=999 => PopulationTier::Village,
            1_000..=9_999 => PopulationTier::Town,
            10_000..=49_999 => PopulationTier::City,
            50_000..=199_999 => PopulationTier::Metropolis,
            _ => PopulationTier::Megacity,
        };

        let city_mood = match stats.average_happiness {
            h if h < 30.0 => CityMood::Struggling,
            h if h < 50.0 => CityMood::Neutral,
            h if h < 70.0 => CityMood::Content,
            h if h < 85.0 => CityMood::Thriving,
            _ => CityMood::Euphoric,
        };

        let time_of_day = TimeOfDayPeriod::from_hour(clock.hour);

        let crisis_level = if disaster.current.is_some() {
            CrisisLevel::Catastrophic
        } else if effects.epidemic_ticks > 0 {
            CrisisLevel::Major
        } else if budget.treasury < 0.0 || stats.average_happiness < 25.0 {
            CrisisLevel::Minor
        } else {
            CrisisLevel::None
        };

        Self {
            population_tier,
            city_mood,
            time_of_day,
            season: weather.season,
            crisis_level,
            activity_level: 0.5, // computed from commute hours, construction, etc.
        }
    }
}
```

### 3.5 Time-of-Day Musical Palettes

The `GameClock` hour (0.0-24.0) drives instrument selection, tempo, and key center.
These changes are gradual and musical -- the system does not abruptly switch at exact
hour boundaries but crossfades over 15-30 minutes of game time.

#### 3.5.1 Period Definitions

| Period | Hours | Tempo (BPM) | Key Tendency | Instrument Palette | Mood |
|---|---|---|---|---|---|
| Pre-Dawn | 4:00-5:30 | 60-65 | Dm, Am | Solo piano, sparse strings, glass pad | Quiet anticipation |
| Dawn | 5:30-7:00 | 65-72 | C, G | Piano + flute, light strings, birdsong-like synth | Awakening, gentle hope |
| Morning | 7:00-9:00 | 72-80 | G, D | Full piano, acoustic guitar, light percussion | Energetic optimism |
| Mid-Morning | 9:00-11:00 | 78-82 | D, A | Piano + vibraphone, walking bass, brushes | Settled productivity |
| Midday | 11:00-14:00 | 80-85 | C, F | Full ensemble, brass accents, steady rhythm | Confident, bright |
| Afternoon | 14:00-17:00 | 76-80 | F, Bb | Warm strings, muted trumpet, guitar | Relaxed warmth |
| Golden Hour | 17:00-19:00 | 70-76 | Bb, Eb | Saxophone, piano ballad, warm pads | Reflective beauty |
| Evening | 19:00-21:00 | 65-72 | Eb, Ab | Jazz ensemble, walking bass, brushes, piano | Sophisticated relaxation |
| Night | 21:00-23:00 | 58-65 | Ab, Db | Solo piano, ambient pads, distant strings | Contemplative quiet |
| Late Night | 23:00-2:00 | 50-58 | Db, Gbm | Ambient drone, sparse piano, no percussion | Deep stillness |
| Deep Night | 2:00-4:00 | 45-52 | Fm, Bbm | Drone only, occasional bell tone, silence gaps | Near-silence, minimal |

#### 3.5.2 Transition Interpolation

```rust
#[derive(Debug, Clone, Copy)]
pub enum TimeOfDayPeriod {
    PreDawn,
    Dawn,
    Morning,
    MidMorning,
    Midday,
    Afternoon,
    GoldenHour,
    Evening,
    Night,
    LateNight,
    DeepNight,
}

impl TimeOfDayPeriod {
    pub fn from_hour(hour: f32) -> Self {
        match hour {
            h if h < 4.0 => Self::DeepNight,
            h if h < 5.5 => Self::PreDawn,
            h if h < 7.0 => Self::Dawn,
            h if h < 9.0 => Self::Morning,
            h if h < 11.0 => Self::MidMorning,
            h if h < 14.0 => Self::Midday,
            h if h < 17.0 => Self::Afternoon,
            h if h < 19.0 => Self::GoldenHour,
            h if h < 21.0 => Self::Evening,
            h if h < 23.0 => Self::Night,
            _ => Self::LateNight,
        }
    }

    pub fn target_bpm(self) -> f64 {
        match self {
            Self::PreDawn => 62.0,
            Self::Dawn => 68.0,
            Self::Morning => 76.0,
            Self::MidMorning => 80.0,
            Self::Midday => 82.0,
            Self::Afternoon => 78.0,
            Self::GoldenHour => 73.0,
            Self::Evening => 68.0,
            Self::Night => 62.0,
            Self::LateNight => 54.0,
            Self::DeepNight => 48.0,
        }
    }
}
```

### 3.6 Crisis and Event Music

When a crisis occurs, the music system must respond immediately but musically. Rather than
cutting to "battle music," the system uses the tension stem (already part of the vertical
mix) and modifies the harmonic content of other stems.

#### 3.6.1 Crisis Response Sequence

```
1. Disaster strikes (ActiveDisaster transitions from None to Some)
2. Within 1 bar: tension stem begins fade-in (2-bar tween)
3. Within 2 bars: melodic stem begins fade-out (2-bar tween)
4. Within 4 bars: harmonic stem shifts to minor key variant
5. Within 8 bars: if disaster persists, transition to Tension section
6. On disaster end: reverse sequence over 8-16 bars (relief is gradual)
```

#### 3.6.2 Disaster-Specific Musical Elements

| Disaster Type | Musical Response | Specific Elements |
|---|---|---|
| Tornado | Frantic, dissonant | Tremolo strings, rapid percussion, wind-like synth |
| Earthquake | Heavy, ground-shaking | Deep timpani, low brass, sub-bass rumble, irregular rhythm |
| Flood | Rising tension, then washing | Rising string glissando, water-like synth, harp arpeggios |
| Fire | Urgent, crackling | Staccato strings, snare rolls, brass fanfare fragments |
| Epidemic | Creeping dread | Sustained dissonance, slow pulse, detuned piano |
| Budget Crisis | Anxiety, thinning | Remove instruments one by one, leaving sparse piano |

### 3.7 Stinger System

Stingers are short (2-8 second) musical phrases that punctuate specific events. They play
on top of the current music mix, routed through the stinger track on the music bus.

#### 3.7.1 Stinger Catalog

| Event | Duration | Character | Notes |
|---|---|---|---|
| Population milestone | 4s | Triumphant brass fanfare | Scales with milestone size |
| Building complete | 2s | Gentle chime + string swell | Subtle, frequent |
| Disaster starts | 3s | Dramatic timpani hit + brass stab | Immediately attention-grabbing |
| Disaster ends | 4s | Resolving chord (tension -> major) | Relief and resolution |
| Policy enacted | 2s | Soft woodwind phrase | Informational, not dramatic |
| Achievement unlocked | 3s | Ascending arpeggio + bell | Celebratory but not excessive |
| First of zone type | 3s | Characteristic instrument of zone | e.g., first factory = industrial percussion |
| Budget surplus | 2s | Cash register + ascending strings | Positive, subtle |
| Budget deficit | 2s | Descending bass note + muted horn | Warning, subtle |
| Festival event | 3s | Festive percussion + brass | Joyful, celebratory |
| New connection | 2s | Harp glissando | Highway/airport connection established |

#### 3.7.2 Stinger Playback Rules

```rust
pub struct StingerSystem {
    pub queue: VecDeque<StingerRequest>,
    pub last_played: HashMap<StingerType, f64>, // game time of last play
    pub cooldowns: HashMap<StingerType, f64>,   // minimum seconds between same stinger
}

impl StingerSystem {
    /// Queue a stinger. Respects cooldowns and priority.
    pub fn request(&mut self, stinger: StingerType, priority: StingerPriority, game_time: f64) {
        // Check cooldown
        if let Some(&last) = self.last_played.get(&stinger) {
            let cooldown = self.cooldowns.get(&stinger).copied().unwrap_or(5.0);
            if game_time - last < cooldown {
                return; // too soon
            }
        }

        self.queue.push_back(StingerRequest { stinger, priority, game_time });

        // Sort queue by priority (higher priority plays first)
        self.queue.make_contiguous().sort_by(|a, b|
            b.priority.cmp(&a.priority)
        );

        // Only keep top 3 queued stingers (prevent backlog)
        while self.queue.len() > 3 {
            self.queue.pop_back();
        }
    }

    /// Called every beat. Plays next stinger if ready.
    pub fn tick(&mut self, audio: &AudioManager, game_time: f64) {
        if let Some(request) = self.queue.pop_front() {
            // Play stinger with slight volume duck on music stems
            // Stinger volume: 0.7-1.0 based on priority
            let volume = match request.priority {
                StingerPriority::Low => 0.7,
                StingerPriority::Medium => 0.85,
                StingerPriority::High => 1.0,
            };

            // Duck music stems by 3-6 dB during stinger playback
            // (automatic via Kira sidechain or manual tween)

            self.last_played.insert(request.stinger, game_time);
        }
    }
}
```

### 3.8 Transition Smoothness

Musical transitions must be imperceptible to maintain immersion. Every transition in the
music system uses one of these techniques:

#### 3.8.1 Quantized Transitions

All stem volume changes and section transitions are quantized to the nearest musical
boundary. Kira's clock system enables this:

```rust
/// Quantize a transition to the next bar boundary
fn next_bar_time(current_beat: f64, time_signature_numerator: u8) -> f64 {
    let beats_per_bar = time_signature_numerator as f64;
    let current_bar = (current_beat / beats_per_bar).floor();
    (current_bar + 1.0) * beats_per_bar
}

/// Quantize to the next beat boundary (finer granularity)
fn next_beat_time(current_beat: f64) -> f64 {
    current_beat.ceil()
}
```

#### 3.8.2 Crossfade Curves

Different transition types use different easing curves:

| Transition Type | Fade Duration | Curve | Notes |
|---|---|---|---|
| Stem fade-in | 2-4 bars | EaseInCubic | Gradual entry, not jarring |
| Stem fade-out | 2-4 bars | EaseOutCubic | Gradual exit |
| Section crossfade | 4-8 bars | EaseInOutQuad | Smooth S-curve |
| Crisis onset | 1-2 bars | EaseInQuad | Faster for urgency |
| Crisis resolution | 8-16 bars | EaseOutCubic | Slow relief |
| Stinger duck | 0.5 bars | Linear | Quick but not instant |
| Time-of-day shift | 16-32 bars | Linear | Imperceptibly gradual |

### 3.9 Reference Implementations Analysis

#### 3.9.1 SimCity 4 (2003)

SimCity 4's soundtrack (composed by Jerry Martin and others) is widely regarded as the
gold standard for city builder music.

**What it does right:**
- Jazz-influenced compositions that evoke urban sophistication
- Three distinct music sets for residential, commercial, and industrial focus
- Music changes with city density -- small town gets acoustic guitar, metropolis gets full jazz big band
- Night music is distinctly different (slower, moodier, more piano)
- Each track stands alone as excellent music (not just "functional game audio")

**Notable techniques:**
- Uses actual jazz musicians, giving an organic feel that synthesized music lacks
- Different tracks for different "moods" of the city, not just one adaptive piece
- Some tracks have a melancholy undertone even in positive states, reflecting the
  bittersweet nature of watching a city change

**Lessons for Megacity:**
- Invest in compositional quality -- the music must be good enough to listen to on its own
- Jazz/neo-jazz is an excellent genre fit for city builders (urban association)
- Don't be afraid of emotional complexity (not everything needs to be "happy city" music)

#### 3.9.2 Frostpunk (2018)

Frostpunk's soundtrack (composed by Piotr Musial) won multiple awards and is considered
one of the finest examples of adaptive game music.

**What it does right:**
- Makes you physically *feel* the cold through sound design
- Builds from near-silence to overwhelming orchestral climaxes over the course of a session
- Uses the Discontent and Hope meters as direct music drivers
- The track "The City Must Survive" becomes an anthem that triggers at critical moments
- Silence is used as an instrument -- some of the most powerful moments have no music

**Notable techniques:**
- Vertical layering with 4-6 stems per piece
- Layer activation tied to Hope/Discontent/Temperature game parameters
- Choir enters only at extreme emotional moments (very effective)
- Industrial percussion (anvil strikes, metal clangs) blurs the line between music and SFX
- String tremolo and dissonant harmonics create physical unease

**Lessons for Megacity:**
- Restraint is powerful -- don't play all stems all the time
- Tie specific instruments to specific emotions (choir = transcendence, solo violin = loneliness)
- Let the music *respond* to the simulation, not just accompany it
- Silence after a crisis resolution can be more powerful than triumphant music

#### 3.9.3 Anno 1800 (2019)

Anno 1800's soundtrack (composed by Dynamedion) excels at period-appropriate orchestral
music that adapts to gameplay context.

**What it does right:**
- Different musical themes for Old World vs New World vs Arctic
- Music escalates during naval combat with brass fanfares and driving percussion
- Exploration music is distinct from building music
- Uses period instruments (harpsichord, chamber strings) to reinforce the historical setting
- Building phase music is calm and productive without being boring

**Notable techniques:**
- Horizontal re-sequencing between exploration, building, combat, and event sections
- Region-based music (each map area has its own musical identity)
- Smooth transitions that never break the period immersion
- Victory/defeat stingers that feel emotionally earned

**Lessons for Megacity:**
- Consider having different musical identities for different districts
- Regional variation adds enormous depth to the audio experience
- Stingers should feel like they belong to the current musical piece, not be generic

#### 3.9.4 Stardew Valley (2016)

While not a city builder, Stardew Valley (composed by ConcernedApe/Eric Barone) is
a masterclass in seasonal and time-of-day music for management games.

**What it does right:**
- Each season has a completely different set of tracks (16 per season in the OST)
- Music genuinely changes the *feel* of each season (spring is bouncy, winter is contemplative)
- Night music is dramatically different from day music
- Festival events have unique celebratory music
- Rain replaces outdoor music with a special rain-day track set

**Notable techniques:**
- Complete track replacement per season (no crossfading, just different playlists)
- Weather-reactive music (rain days have their own tracks, not just rain SFX over normal music)
- Location-based music (town, farm, mine, beach all have distinct tracks)
- Uses simple instrumentation (piano, guitar, flute) that doesn't fatigue over long sessions

**Lessons for Megacity:**
- Seasonal music variation is essential for long-session games
- Weather should affect music, not just add rain sounds on top
- Simple, memorable melodies beat complex orchestral textures for replayability
- The player hears this music for hundreds of hours -- avoid fatigue through variety

---

## 4. Environmental Audio

Environmental audio is the ever-present sonic backdrop that tells the player what is
happening in the natural world around their city. Unlike zone ambience (which is human-
generated), environmental audio comes from weather, water, wildlife, and the cycle of
day and night. These sounds exist even in an empty map before any building is placed --
they are the voice of the land itself.

### 4.1 Weather Sounds

Megacity's `Weather` resource tracks `current_event` (Clear, Rain, HeatWave, ColdSnap,
Storm) and `season`. Each weather state has a distinct sonic signature.

#### 4.1.1 Rain Audio

Rain is the most complex weather sound because it has multiple layers that scale with
intensity, interact with surfaces, and change character based on what the rain is falling on.

**Rain Layers:**

| Layer | Volume Mapping | Character | Frequency Range |
|---|---|---|---|
| Distant rain wash | Always present during rain | Continuous white-noise-like wash | 2-8 kHz |
| Close rain drops | Camera distance < 500 | Individual raindrop impacts | 4-12 kHz, transient |
| Rain on pavement | Proportional to road cells in view | Harder, more splashy | 3-10 kHz |
| Rain on rooftops | Proportional to building cells in view | Metallic tapping | 5-14 kHz |
| Rain on foliage | Proportional to tree/park cells in view | Soft rustling patter | 2-6 kHz |
| Rain on water | Proportional to water cells in view | Hollow, resonant drops | 1-4 kHz |
| Gutter/drainage | After 2+ days of rain | Water flowing in pipes | 200-800 Hz |

**Intensity Scaling:**

```rust
fn rain_intensity(weather: &Weather) -> f32 {
    match weather.current_event {
        WeatherEvent::Rain => 0.6,          // moderate rain
        WeatherEvent::Storm => 1.0,         // heavy downpour
        _ => 0.0,
    }
}

fn rain_layer_volumes(intensity: f32, surface_composition: &SurfaceComposition) -> RainLayers {
    RainLayers {
        distant_wash: intensity * 0.7,
        close_drops: intensity * 0.4,
        on_pavement: intensity * 0.5 * surface_composition.road_fraction,
        on_rooftops: intensity * 0.4 * surface_composition.building_fraction,
        on_foliage: intensity * 0.3 * surface_composition.vegetation_fraction,
        on_water: intensity * 0.35 * surface_composition.water_fraction,
        gutters: if intensity > 0.5 { intensity * 0.2 } else { 0.0 },
    }
}
```

#### 4.1.2 Thunder

Thunder accompanies storms (`WeatherEvent::Storm`) and is implemented as randomized
one-shot sounds at varying distances:

```rust
pub struct ThunderSystem {
    pub next_strike_timer: Timer,
    pub storm_active: bool,
}

impl ThunderSystem {
    fn schedule_next_strike(&mut self) {
        // Random interval between 8-30 seconds during storm
        let interval = 8.0 + pseudo_random_f32(seed) * 22.0;
        self.next_strike_timer = Timer::from_seconds(interval, TimerMode::Once);
    }

    fn play_thunder(&self, audio: &AudioManager) {
        // Distance determines delay between lightning flash and thunder
        // (visual flash could be implemented in rendering)
        let distance = pseudo_random_f32(seed); // 0.0 = close, 1.0 = far

        // Close thunder: sharp crack followed by long rumble
        // Distant thunder: just a low, rolling rumble
        // Volume: close = 0.9, far = 0.3
        let volume = 0.9 - (distance * 0.6);

        // Low-pass filter intensity increases with distance
        // Close: full spectrum. Far: mostly sub-200Hz rumble.
        let lowpass_cutoff = 8000.0 - (distance * 6000.0); // Hz
    }
}
```

#### 4.1.3 Wind

Wind is present in all weather states but varies dramatically in intensity:

| Weather State | Wind Volume | Wind Character | Pitch |
|---|---|---|---|
| Clear, Spring/Summer | 0.1-0.2 | Gentle breeze, occasional gust | Mid (500-2000 Hz) |
| Clear, Autumn | 0.2-0.3 | Moderate, with leaf rustle | Mid-low (300-1500 Hz) |
| Clear, Winter | 0.2-0.4 | Cold, biting, whistling | Higher (800-3000 Hz) |
| Rain | 0.3-0.5 | Steady, merging with rain wash | Mid (400-2000 Hz) |
| Storm | 0.6-0.9 | Howling, gusting, threatening | Low-high sweep (200-4000 Hz) |
| HeatWave | 0.05-0.1 | Nearly still, oppressive quiet | Very low (100-500 Hz) |
| ColdSnap | 0.4-0.6 | Harsh, piercing | High (1000-4000 Hz) |

**Wind interaction with buildings:** In dense urban areas, wind channels between buildings
creating a whistling/howling effect. This ties into the urban canyon detection from
Section 2.5.2:

```
wind_urban_mod = if is_urban_canyon {
    1.3  // amplified by canyon effect
} else if building_density > 0.7 {
    0.8  // blocked by dense buildings
} else {
    1.0  // open area, normal wind
}
```

#### 4.1.4 Snow and Cold

Winter weather is characterized by *absence* of sound. Snow muffles everything:

```rust
fn winter_muffling_factor(weather: &Weather) -> f32 {
    match (weather.season, weather.current_event) {
        (Season::Winter, WeatherEvent::ColdSnap) => 0.5,  // heavy snow, very muffled
        (Season::Winter, _) => 0.7,                        // general winter dampening
        _ => 1.0,                                           // no muffling
    }
}
```

This factor is applied as a multiplier on the ambience bus and traffic sub-bus.
Additionally, a low-pass filter (cutoff 2000-4000 Hz) is applied to simulate
snow absorbing high frequencies. The effect is subtle but powerful -- it makes
winter *feel* cold through sound alone.

### 4.2 Seasonal Ambience

Each season in Megacity (90-day cycles: Spring, Summer, Autumn, Winter) has a distinct
ambient layer that crossfades over 2-3 game days at season boundaries.

#### 4.2.1 Season-Specific Sound Beds

**Spring:**
- Dawn chorus: rich, varied birdsong peaking at 5-7am (loudest seasonal birdsong)
- Insect buzz begins (bees, early insects) from 10am-4pm
- Occasional distant thunder from afternoon rain (even if no storm event active)
- Dripping/melting sounds in early spring (if coming from winter)
- Light, warm wind with occasional gentle gusts
- Frog croaking near water cells at dusk (7-9pm)

**Summer:**
- Insect symphony: cicadas (continuous), crickets (evening), buzzing flies
- Air conditioner hum from residential/commercial buildings (scales with temperature)
- Distant lawnmower/yard maintenance sounds in residential areas
- Heat shimmer drone: a very subtle low-frequency hum that suggests oppressive heat
- Birdsong present but less intense than spring (birds are quieter in heat)
- Louder water sounds (people at fountains, more outdoor activity)
- Thunderstorm possibility adds distant rumble even in clear weather

**Autumn:**
- Wind through dry leaves (continuous, increases with wind speed)
- Geese honking (migratory birds, random intervals, directional panning)
- Fewer insects (crickets fade through the season)
- Rain more frequent (reflected in weather system's autumn probabilities)
- Crackling quality to the ambient bed (dry, crisp air)
- Birdsong significantly reduced (some species have left)

**Winter:**
- Near-silence as the dominant character
- Occasional wind gusts that feel cold and biting
- Crunching footsteps (if citizen emitters are active at close zoom)
- Muffled quality to all urban sounds (snow absorption)
- Heating system hum from buildings (replaces summer AC hum)
- Very rare birdsong (only crows, ravens, sparrows -- hardy species)
- Ice creaking near water cells (frozen lakes/rivers)

#### 4.2.2 Seasonal Crossfade

```rust
fn seasonal_crossfade(
    current_day: u32,
    current_season: Season,
    prev_season_bed: &AudioHandle,
    next_season_bed: &AudioHandle,
) {
    let day_in_season = ((current_day - 1) % 90) + 1;

    // Crossfade during first 5 days of new season
    if day_in_season <= 5 {
        let blend = day_in_season as f32 / 5.0;
        prev_season_bed.set_volume(1.0 - blend);
        next_season_bed.set_volume(blend);
    }
}
```

### 4.3 Water Body Audio

Water cells (`CellType::Water`) produce ambient water sounds that vary based on the
size and context of the water body.

#### 4.3.1 Water Sound Types

| Water Context | Sound Character | Detection Method |
|---|---|---|
| River (narrow, flowing) | Rushing, babbling brook | Connected water cells in a line, < 5 cells wide |
| Lake (large, still) | Gentle lapping, occasional plop | Large contiguous water region, > 50 cells |
| Ocean (map edge water) | Waves crashing, gulls, deep roar | Water cells at map boundary |
| Fountain (park feature) | Splashing, tinkling water | Service building with fountain type |
| Storm drain outflow | Rushing water during rain | Water cells adjacent to road during rain |
| Frozen (winter) | Ice creaking, muffled stillness | Any water cell during winter |

#### 4.3.2 Water Audio Parameters

```rust
struct WaterAudioParams {
    base_volume: f32,
    pitch: f32,
    loop_asset: &'static str,
}

fn water_params_for_context(context: WaterContext, weather: &Weather) -> WaterAudioParams {
    match context {
        WaterContext::River => WaterAudioParams {
            base_volume: 0.5,
            pitch: 1.0 + (weather.travel_speed_multiplier() - 1.0) * 0.2, // faster in rain
            loop_asset: "audio/ambience/river_flow.ogg",
        },
        WaterContext::Lake => WaterAudioParams {
            base_volume: 0.3,
            pitch: 1.0,
            loop_asset: "audio/ambience/lake_lapping.ogg",
        },
        WaterContext::Ocean => WaterAudioParams {
            base_volume: 0.7,
            pitch: match weather.current_event {
                WeatherEvent::Storm => 0.8, // deeper, more powerful waves
                _ => 1.0,
            },
            loop_asset: "audio/ambience/ocean_waves.ogg",
        },
        WaterContext::Fountain => WaterAudioParams {
            base_volume: 0.4,
            pitch: 1.1, // slightly higher, more musical
            loop_asset: "audio/ambience/fountain_splash.ogg",
        },
        WaterContext::Frozen => WaterAudioParams {
            base_volume: 0.15,
            pitch: 0.7, // low, creaky
            loop_asset: "audio/ambience/ice_creak.ogg",
        },
    }
}
```

### 4.4 Day/Night Cycle Audio

The existing `GameClock` provides `hour` (0.0-24.0) which drives a continuous ambient
audio modulation synchronized with the visual `update_day_night_cycle` system.

#### 4.4.1 Dawn Chorus (5:00-7:00)

The dawn chorus is one of the most distinctive natural sounds and provides a powerful
sense of time passing. Birds begin singing before sunrise and reach peak intensity
around 30-60 minutes after first light.

```rust
fn dawn_chorus_volume(hour: f32, season: Season) -> f32 {
    // Bell curve centered at 5:45 AM, width varies by season
    let peak_hour = 5.75;
    let width = match season {
        Season::Spring => 1.5,   // long, rich dawn chorus
        Season::Summer => 1.2,   // slightly shorter
        Season::Autumn => 0.8,   // brief
        Season::Winter => 0.0,   // no dawn chorus
    };

    if width == 0.0 { return 0.0; }

    let dist = (hour - peak_hour).abs();
    let volume = (-dist * dist / (2.0 * width * width)).exp();

    // Season also affects peak volume
    let seasonal_peak = match season {
        Season::Spring => 0.8,
        Season::Summer => 0.6,
        Season::Autumn => 0.3,
        Season::Winter => 0.0,
    };

    volume * seasonal_peak
}
```

#### 4.4.2 Daytime Activity Sounds (7:00-19:00)

During the day, human activity sounds dominate. This is handled by the zone ambience
system (Section 2.1), but the environmental layer adds:

- Distant sirens (probability scales with `crime_rate` and city size)
- Aircraft flyovers (if airport exists, random interval 120-300s)
- Church/clock tower bells on the hour (if religious/civic building exists)
- School bell at 8:00, 12:00, 15:00 (if school exists)

#### 4.4.3 Evening Wind-Down (19:00-22:00)

Transition period where human sounds reduce and nature sounds increase:
- Traffic volume fades (handled by traffic time_mod in Section 2.2.2)
- Crickets begin (summer/autumn, starting around 8pm)
- Wind becomes more noticeable (human sounds are quieter, relative wind is louder)
- Occasional domestic sounds (TV through windows, distant conversation)

#### 4.4.4 Night Sounds (22:00-5:00)

Night is the quietest period, but not silent:

| Sound | Time Range | Season | Volume | Notes |
|---|---|---|---|---|
| Crickets | 8pm-4am | Summer, early Autumn | 0.3-0.5 | Continuous chirp |
| Owl hooting | 9pm-4am | All except deep winter | 0.2 | Random interval 120-600s |
| Distant dog bark | 10pm-5am | All | 0.15 | Random interval 300-900s, echoey |
| Wind (amplified) | All night | All | +0.1 vs day | More noticeable without activity noise |
| Distant traffic | All night | All | 0.1 | Very faint highway hum |
| Night insects | 8pm-5am | Summer | 0.2-0.4 | Chorus of various insects |
| Coyote howl | 11pm-3am | All, rural areas | 0.15 | Very rare, only if low density |

#### 4.4.5 Ambient Volume Envelope

The overall ambient volume follows a 24-hour envelope:

```
Hour  0  1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16 17 18 19 20 21 22 23
Vol  .2 .15.1 .1 .15.3 .5 .7 .8 .85.9 .9 .9 .9 .85.8 .8 .85.8 .7 .5 .4 .3 .25
```

This envelope is applied to the overall ambience bus, not to individual layers.
Individual layers have their own modulation on top of this envelope.

### 4.5 Disaster Sounds

Disasters are the most dramatic audio events in the game. They override normal ambient
audio and demand the player's attention.

#### 4.5.1 Tornado

```
Phase 1 - Warning (10 ticks before damage):
  - Distant rumble begins, slowly building (0.2 -> 0.7 volume over 10 ticks)
  - Tornado siren wail (if civil defense exists): classic ascending/descending siren
  - Wind increases dramatically in the weather sub-bus

Phase 2 - Active (TORNADO_DURATION = 50 ticks):
  - Roaring wind at 0.9 volume, low-frequency dominant (100-500 Hz)
  - Debris sounds: cracking wood, shattering glass, metal tearing
  - These debris sounds triggered per destroyed building cell
  - Pitch rises slightly as tornado intensifies
  - Spatial: tornado has a position (center_x, center_y) and the sound
    is spatialized to that point, getting louder as camera approaches

Phase 3 - Aftermath:
  - Wind rapidly fades (0.9 -> 0.1 over 5 seconds)
  - Settling sounds: creaking structures, falling debris
  - Eerie quiet: ambience bus volume reduced to 0.3 for 30 seconds
  - Emergency sirens (fire trucks, ambulances) if emergency services exist
```

#### 4.5.2 Earthquake

```
Phase 1 - Onset:
  - Deep sub-bass rumble (30-80 Hz), building from 0.0 to 1.0 over 3 seconds
  - Camera shake would be visual; audio equivalent is rapid tremolo on all sounds
  - Cracking/groaning of structures (pre-destruction warning)

Phase 2 - Active (EARTHQUAKE_DURATION = 30 ticks):
  - Intense rumble at full volume
  - Building destruction sounds (per cell): concrete crumbling, glass shattering
  - Ground cracking (high-frequency transients mixed with low rumble)
  - All other ambient sounds ducked by -12 dB

Phase 3 - Aftershocks:
  - Smaller rumbles at 0.3-0.5 volume, random interval 10-30 seconds
  - Duration: 60 seconds after main quake ends
  - Dust settling sounds (subtle high-frequency noise)
  - Emergency response sirens
```

#### 4.5.3 Flood

```
Phase 1 - Rising Water:
  - Water rushing sound, building slowly (0.1 -> 0.6 over 20 seconds)
  - Rain intensifies (if not already raining, add rain at 0.8 volume)
  - Alarm sounds from affected buildings (electronic beeping)

Phase 2 - Peak Flood (FLOOD_DURATION = 80 ticks):
  - Continuous water roar at 0.7 volume
  - Spatial: loudest at flood center, radiating outward
  - Occasional impacts (cars, debris hitting structures)
  - Splashing sounds as water moves through streets

Phase 3 - Receding:
  - Water sound slowly fades (0.7 -> 0.1 over 30 seconds)
  - Dripping, draining sounds
  - Mud/silt settling (subtle squelching)
  - Pumping sounds (if water infrastructure is active)
```

#### 4.5.4 Fire

Fire audio ties directly into the existing `FireGrid` (per-cell fire intensity 0-100):

```rust
fn fire_audio_volume(fire_grid: &FireGrid, listener_cell: (usize, usize), radius: i32) -> f32 {
    let (lx, ly) = listener_cell;
    let mut total_intensity = 0u32;
    let mut fire_cells = 0u32;

    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let x = lx as i32 + dx;
            let y = ly as i32 + dy;
            if x >= 0 && y >= 0 && (x as usize) < GRID_WIDTH && (y as usize) < GRID_HEIGHT {
                let intensity = fire_grid.get(x as usize, y as usize);
                if intensity > 0 {
                    fire_cells += 1;
                    total_intensity += intensity as u32;
                }
            }
        }
    }

    if fire_cells == 0 { return 0.0; }

    // Scale: 1 cell at intensity 50 = volume 0.3
    //        10 cells at intensity 80 = volume 0.9
    let avg_intensity = total_intensity as f32 / fire_cells as f32;
    let scale = (fire_cells as f32 / 5.0).min(1.0);
    let intensity_factor = avg_intensity / 100.0;

    (scale * intensity_factor * 0.9).min(0.95)
}
```

**Fire sound components:**
- Crackling: continuous loop, volume scales with intensity
- Roaring: kicks in above intensity 50, adds low-frequency power
- Wood snapping: random one-shots, interval decreases with intensity
- Glass breaking: one-shot when building cell transitions to destroyed
- Fire truck siren: plays when fire service responds (if fire station exists)

### 4.6 Wind Simulation Audio

Megacity has a `wind` module in simulation. Wind audio provides both ambience and
gameplay feedback (wind direction/speed affects fire spread, pollution dispersal).

#### 4.6.1 Wind Audio Layers

```rust
pub struct WindAudioState {
    pub base_layer_volume: f32,    // constant gentle air movement
    pub gust_layer_volume: f32,    // intermittent stronger gusts
    pub whistle_layer_volume: f32, // high-frequency whistling in urban canyons
    pub howl_layer_volume: f32,    // low-frequency howling in storms
}

fn compute_wind_audio(
    wind_speed: f32,        // 0.0 to 1.0 normalized
    is_urban_canyon: bool,
    weather: &Weather,
    season: Season,
) -> WindAudioState {
    let storm_factor = match weather.current_event {
        WeatherEvent::Storm => 2.0,
        WeatherEvent::ColdSnap => 1.5,
        _ => 1.0,
    };

    let seasonal_factor = match season {
        Season::Winter => 1.3,
        Season::Autumn => 1.1,
        Season::Spring => 0.9,
        Season::Summer => 0.7,
    };

    let effective_speed = (wind_speed * storm_factor * seasonal_factor).min(1.0);

    WindAudioState {
        base_layer_volume: 0.1 + effective_speed * 0.3,
        gust_layer_volume: if effective_speed > 0.3 {
            (effective_speed - 0.3) * 0.7
        } else {
            0.0
        },
        whistle_layer_volume: if is_urban_canyon && effective_speed > 0.4 {
            (effective_speed - 0.4) * 0.5
        } else {
            0.0
        },
        howl_layer_volume: if effective_speed > 0.7 {
            (effective_speed - 0.7) * 1.0
        } else {
            0.0
        },
    }
}
```

#### 4.6.2 Wind Directionality

Wind has a direction, and this can be represented in stereo panning. If wind blows
from the west, the wind sound pans slightly toward the left channel:

```rust
fn wind_panning(wind_direction: Vec2, camera_forward: Vec2) -> f32 {
    // Dot product of wind direction with camera's right vector
    let camera_right = Vec2::new(-camera_forward.y, camera_forward.x);
    let pan = wind_direction.dot(camera_right);
    pan.clamp(-0.3, 0.3)  // subtle panning, not hard left/right
}
```

---

## 5. Notification and UI Sounds

UI and notification sounds operate outside the spatial audio system -- they are non-
positional, played through the UI bus, and serve as direct communication between the
game and the player. They must be instantly recognizable, non-fatiguing, and hierarchically
organized so that critical alerts always cut through.

### 5.1 Sound Priority Hierarchy

Megacity's `CityEventType` enum and `EventJournal` already categorize events. The audio
system maps these into a three-tier priority hierarchy:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AudioPriority {
    /// Ambient/info: zone demand shifts, policy effects, stats updates
    /// These are barely audible "ticks" that register subconsciously.
    Info = 0,

    /// Important: building complete, milestone reached, budget change
    /// These are clear but not alarming sounds the player should notice.
    Important = 1,

    /// Critical: disaster alert, budget crisis, epidemic, fire spreading
    /// These demand immediate attention and may duck other audio.
    Critical = 2,
}

impl AudioPriority {
    pub fn from_event(event: &CityEventType) -> Self {
        match event {
            CityEventType::DisasterStrike(_) => Self::Critical,
            CityEventType::BudgetCrisis => Self::Critical,
            CityEventType::Epidemic => Self::Critical,
            CityEventType::BuildingFire(_, _) => Self::Critical,

            CityEventType::MilestoneReached(_) => Self::Important,
            CityEventType::PopulationBoom => Self::Important,
            CityEventType::EconomicBoom => Self::Important,
            CityEventType::Festival => Self::Important,
            CityEventType::ResourceDepleted(_) => Self::Important,

            CityEventType::NewPolicy(_) => Self::Info,
        }
    }
}
```

### 5.2 Earcon Design Principles

An "earcon" is a short, non-verbal audio cue that represents a specific concept -- like
an icon but for ears. Effective earcons in Megacity follow these design rules:

**Rule 1: Distinctness.** Each earcon must be immediately distinguishable from every other.
A disaster alarm must never be confused with a building completion sound.

**Rule 2: Learnability.** After hearing an earcon 3-5 times, the player should associate
it with its meaning without conscious effort. This requires consistent use.

**Rule 3: Emotional congruence.** The sound should *feel like* what it represents. Positive
events get ascending, bright sounds; negative events get descending, dark sounds.

**Rule 4: Brevity.** Most earcons should be 0.3-1.5 seconds long. Only critical alerts
get longer sounds (up to 3 seconds).

**Rule 5: Non-fatigue.** These sounds may play hundreds of times per session. They must not
be annoying at any volume. Avoid harsh frequencies (2-5 kHz), sharp transients, or
overly "cute" sounds.

#### 5.2.1 Earcon Design Specifications

| Notification | Duration | Frequency | Texture | Emotional Quality |
|---|---|---|---|---|
| **Critical** | | | | |
| Disaster alert | 2.0s | 400-800 Hz, pulsing | Alarm-like, 2-tone alternating | Urgent, impossible to ignore |
| Budget crisis | 1.5s | 200-400 Hz, descending | Dull thud + descending tone | Anxiety, warning |
| Epidemic warning | 1.5s | 300-600 Hz, warbling | Unstable, oscillating tone | Unease, biological threat |
| Fire alert | 1.2s | 500-1000 Hz, rapid | Classic fire alarm cadence | Emergency, action needed |
| **Important** | | | | |
| Building complete | 0.8s | 800-1600 Hz, ascending | Bright chime + settle | Satisfaction, accomplishment |
| Milestone reached | 1.2s | 600-1200 Hz, fanfare | Short brass-like phrase | Pride, achievement |
| Population boom | 0.8s | 500-1000 Hz, bubbly | Rising bubbles / popping | Growth, excitement |
| Economic boom | 0.8s | 800-1600 Hz, shimmering | Coin-like + ascending bell | Prosperity, wealth |
| Festival | 1.0s | 600-1200 Hz, festive | Brief celebratory percussion | Joy, community |
| Resource depleted | 1.0s | 300-600 Hz, descending | Hollow, diminishing tone | Loss, warning |
| **Info** | | | | |
| Zone demand change | 0.4s | 1000-2000 Hz | Subtle tick/click | Barely noticeable, ambient |
| Policy effect | 0.5s | 800-1200 Hz | Soft chime | Acknowledged, neutral |
| Advisor message | 0.6s | 600-1000 Hz | Gentle notification ping | Attention, informational |
| Save complete | 0.4s | 1000-1500 Hz | Quick confirmation beep | Reassurance |
| Unlock available | 0.7s | 800-1400 Hz, ascending | Discovery-like sparkle | Curiosity, reward |

### 5.3 Volume Hierarchy and Ducking

When a critical notification plays, it must be audible regardless of what else is
happening. The system uses volume ducking -- temporarily reducing the volume of lower-
priority audio buses when a high-priority sound plays.

#### 5.3.1 Volume Levels by Priority

| Priority | Own Volume (UI bus) | Music Duck | Ambience Duck | SFX Duck |
|---|---|---|---|---|
| Critical | 0.9-1.0 | -6 dB | -4 dB | -3 dB |
| Important | 0.6-0.8 | -2 dB | -1 dB | 0 dB |
| Info | 0.3-0.5 | 0 dB | 0 dB | 0 dB |

#### 5.3.2 Ducking Implementation

```rust
pub struct DuckingState {
    pub music_duck_db: f32,
    pub ambience_duck_db: f32,
    pub sfx_duck_db: f32,
    pub duck_timer: Timer,
    pub release_duration: Duration,
}

impl DuckingState {
    pub fn apply_duck(&mut self, priority: AudioPriority, sound_duration: Duration) {
        let (music, ambience, sfx) = match priority {
            AudioPriority::Critical => (-6.0, -4.0, -3.0),
            AudioPriority::Important => (-2.0, -1.0, 0.0),
            AudioPriority::Info => (0.0, 0.0, 0.0),
        };

        self.music_duck_db = music;
        self.ambience_duck_db = ambience;
        self.sfx_duck_db = sfx;

        // Hold duck for duration of sound, then release
        self.duck_timer = Timer::from_seconds(
            sound_duration.as_secs_f32(),
            TimerMode::Once,
        );
        self.release_duration = Duration::from_millis(300); // 300ms release
    }

    pub fn update(&mut self, dt: Duration) {
        self.duck_timer.tick(dt);
        if self.duck_timer.finished() {
            // Gradually release duck
            let release_speed = dt.as_secs_f32() / self.release_duration.as_secs_f32();
            self.music_duck_db *= 1.0 - release_speed;
            self.ambience_duck_db *= 1.0 - release_speed;
            self.sfx_duck_db *= 1.0 - release_speed;
        }
    }
}
```

### 5.4 Cooldown and Spam Prevention

In a city with thousands of buildings and events, notifications can fire rapidly.
Without cooldowns, the player would be bombarded with overlapping sounds.

#### 5.4.1 Cooldown Rules

```rust
pub struct NotificationCooldowns {
    pub last_played: HashMap<NotificationType, f64>, // game_time
    pub cooldowns: HashMap<NotificationType, f64>,   // seconds
}

impl Default for NotificationCooldowns {
    fn default() -> Self {
        let mut cooldowns = HashMap::new();

        // Critical: always play (very short cooldown just to prevent double-trigger)
        cooldowns.insert(NotificationType::DisasterAlert, 0.5);
        cooldowns.insert(NotificationType::BudgetCrisis, 10.0);
        cooldowns.insert(NotificationType::FireAlert, 3.0);

        // Important: moderate cooldown
        cooldowns.insert(NotificationType::BuildingComplete, 2.0);
        cooldowns.insert(NotificationType::Milestone, 5.0);
        cooldowns.insert(NotificationType::PopulationBoom, 15.0);

        // Info: aggressive cooldown (these fire frequently)
        cooldowns.insert(NotificationType::ZoneDemand, 10.0);
        cooldowns.insert(NotificationType::PolicyEffect, 5.0);

        Self {
            last_played: HashMap::new(),
            cooldowns,
        }
    }
}
```

#### 5.4.2 Notification Batching

When multiple notifications of the same type fire within the cooldown window, the
system batches them and plays a single "multiple" variant:

```
- 1 building complete: single chime
- 2-5 in cooldown window: slightly louder chime with subtle echo
- 6+ in cooldown window: distinct "batch complete" sound (richer, more resonant)
```

This prevents notification overload during rapid construction while still communicating
that "a lot of building is happening."

### 5.5 Accessibility Requirements

Audio-only communication excludes deaf and hard-of-hearing players. Every audio cue
in Megacity must have a visual counterpart.

#### 5.5.1 Visual Indicators for Audio Cues

| Audio Cue | Visual Indicator | Location |
|---|---|---|
| Disaster alert | Flashing screen edge (red), popup banner | Full screen + UI panel |
| Budget crisis | Pulsing treasury icon, red highlight | Toolbar |
| Building complete | Brief particle effect on building, icon pulse | World + minimap |
| Milestone | Centered popup with animation | UI overlay |
| Zone demand change | RCI bar color change + arrow indicator | Demand panel |
| Fire alert | Orange glow on affected area, icon in status bar | World + UI |

#### 5.5.2 Settings Options

The audio settings panel must include:
- Master volume (0-100%)
- Music volume (0-100%)
- Ambience volume (0-100%)
- SFX volume (0-100%)
- UI/Notification volume (0-100%)
- Notification sounds: On / Critical Only / Off
- Subtitles for notification text: On / Off
- Visual notification intensity: Normal / Enhanced (larger, more prominent visuals)
- Mono audio: forces stereo to mono for single-ear hearing
- Audio description mode: additional verbal announcements for key events

---

## 6. Tool and Interaction Sounds

Every player action should have auditory feedback. Tool sounds reinforce the tactile
feel of building a city and make the interface feel responsive.

### 6.1 Road Placement

Road placement is one of the most frequent player actions and needs satisfying audio
that communicates both the action and the cost.

#### 6.1.1 Road Sound Sequence

```
1. Tool selected: subtle "equip" click (0.3s, metallic)
2. First click (start point): firm "anchor" sound (0.2s, thud)
3. Drag/extend: continuous stretching sound
   - Pitch rises slightly as road gets longer (subtle rubber-band feel)
   - Volume proportional to road cost (longer = louder = more expensive)
   - Different texture per road type:
     - Path: soft gravel spreading sound
     - Local: asphalt rolling sound
     - Avenue: heavier construction
     - Boulevard: heavy machinery
     - Highway: heavy industrial
4. Valid placement preview: subtle positive ping each time a new cell is covered
5. Invalid placement: low buzz/error tone (brief)
6. Confirm (release click): satisfying construction "slam" + brief machinery
   - Volume and intensity scale with road length
   - Single cell: light click
   - Long road: substantial construction sound
7. Cancel (right click/escape): deflating "whoosh" sound (0.3s)
```

#### 6.1.2 Road Type Audio Variations

| Road Type | Selection Sound | Placement Sound | Character |
|---|---|---|---|
| Path | Soft click | Gravel crunch | Light, natural |
| Local | Medium click | Asphalt smoothing | Everyday, suburban |
| Avenue | Firm click | Road roller rumble | Professional, urban |
| Boulevard | Heavy click | Heavy machinery | Substantial, serious |
| Highway | Deep thud | Industrial construction | Massive, powerful |
| OneWay | Quick click | Arrow-like whoosh + asphalt | Directional |

### 6.2 Zoning

Zoning paints zone designations onto the grid. The audio should feel like "claiming"
or "designating" territory.

#### 6.2.1 Zone Sound Sequence

```
1. Zone tool selected: color-coded shimmer sound
   - Residential: warm, domestic tone
   - Commercial: bright, energetic tone
   - Industrial: heavy, mechanical tone
   - Office: clean, digital tone
2. Painting (drag across cells): brush-stroke sound
   - Continuous while dragging
   - Subtle "claimed" pop for each newly zoned cell (max 10 per second to prevent spam)
   - Pitch stays constant (unlike road which rises with length)
3. Zone already occupied: muted thud (cannot zone here)
4. Release: settling sound (0.3s)
5. De-zone: reverse brush sound, slightly more muted
```

#### 6.2.2 Zone Color-Sound Mapping

Each zone type has a characteristic pitch center so that experienced players can
zone by ear (accessibility benefit: zone type is communicated through audio, not just color):

| Zone Type | Pitch Center | Instrument Feel | Color Association |
|---|---|---|---|
| Residential Low | C4 (262 Hz) | Warm bell | Green |
| Residential High | E4 (330 Hz) | Fuller bell | Dark green |
| Commercial Low | G4 (392 Hz) | Bright chime | Blue |
| Commercial High | B4 (494 Hz) | Crystal chime | Dark blue |
| Industrial | C3 (131 Hz) | Metallic thud | Yellow |
| Office | A4 (440 Hz) | Clean sine | Teal |

### 6.3 Bulldoze

Demolition is the most destructive player action and should sound appropriately weighty.

#### 6.3.1 Bulldoze Sound Scaling

```rust
fn bulldoze_sound(target: BulldozeTarget) -> BulldozeSoundParams {
    match target {
        BulldozeTarget::EmptyCell => BulldozeSoundParams {
            sound: "bulldoze_light.wav",
            volume: 0.3,
            pitch: 1.2,  // light, quick
        },
        BulldozeTarget::Road => BulldozeSoundParams {
            sound: "bulldoze_road.wav",
            volume: 0.5,
            pitch: 1.0,  // medium
        },
        BulldozeTarget::SmallBuilding => BulldozeSoundParams {
            sound: "bulldoze_building.wav",
            volume: 0.6,
            pitch: 1.0,  // standard destruction
        },
        BulldozeTarget::LargeBuilding => BulldozeSoundParams {
            sound: "bulldoze_building_large.wav",
            volume: 0.8,
            pitch: 0.8,  // deeper, more substantial
        },
        BulldozeTarget::ServiceBuilding => BulldozeSoundParams {
            sound: "bulldoze_building_large.wav",
            volume: 0.9,
            pitch: 0.7,  // very heavy, significant loss
        },
    }
}
```

**Additional bulldoze audio elements:**
- Dust cloud sound (brief white noise burst, 0.3s)
- Debris settling (gentle clatter, 0.5s after main sound)
- Warning confirmation for expensive buildings (distinctive "are you sure" tone)

### 6.4 Building Placement

Service buildings (schools, hospitals, fire stations) are placed individually and need
distinct placement audio.

```
1. Building selected from menu: characteristic sound for building type
   - School: children's laughter snippet
   - Hospital: heart monitor beep
   - Fire station: brief siren
   - Police station: radio dispatch crackle
   - Power plant: electrical hum
   - Water tower: water flowing
2. Placement preview (hovering): subtle pulse matching building footprint
3. Valid location: soft positive glow sound
4. Invalid location: error buzz (same as road invalid)
5. Confirm placement: construction start sound + spending "ka-ching"
   - Cost proportional to building price (louder = more expensive)
6. Cancel: same deflating whoosh as road cancel
```

### 6.5 Menu Navigation

UI navigation sounds are the most frequently heard sounds in the game and must be
exceptionally subtle and non-fatiguing.

| Action | Sound | Duration | Volume | Notes |
|---|---|---|---|---|
| Button hover | Soft tick | 0.05s | 0.15 | Nearly inaudible, just confirms cursor position |
| Button click | Light click | 0.1s | 0.25 | Satisfying but not distracting |
| Panel open | Soft slide/whoosh | 0.15s | 0.2 | Directional (left/right based on panel position) |
| Panel close | Reverse slide | 0.12s | 0.18 | Slightly faster than open |
| Tab switch | Page turn | 0.1s | 0.2 | Subtle paper sound |
| Slider drag | Continuous soft friction | While dragging | 0.1 | Pitch maps to slider value |
| Toggle on | Click up | 0.08s | 0.2 | Higher pitch than off |
| Toggle off | Click down | 0.08s | 0.2 | Lower pitch than on |
| Dropdown open | Pop | 0.08s | 0.18 | Brief |
| Error/invalid | Dull thud | 0.15s | 0.3 | Can't do that |
| Confirmation dialog | Alert chime | 0.3s | 0.35 | Attention-getting |

### 6.6 Camera Movement

Camera sounds are extremely subtle -- they exist only to prevent the camera from
feeling "disembodied."

```
- Fast pan (keyboard or drag): very faint wind whoosh
  - Volume proportional to camera velocity
  - Max volume: 0.1 (barely perceptible)
  - Only plays above a velocity threshold

- Zoom in/out: very faint "zoom" sound
  - Pitch rises when zooming in, falls when zooming out
  - Volume: 0.05-0.08 (almost subliminal)
  - Only plays for scroll wheel, not smooth zoom

- Orbit rotation: no sound (rotation is too continuous)

- Snap to location (if implemented): brief "teleport" whoosh (0.15)
```

**Important:** Camera sounds must NEVER be annoying. If in doubt, make them quieter
or remove them entirely. The player moves the camera constantly, and any perceivable
camera sound becomes intolerable within minutes.

---

## 7. Procedural Audio Generation

Procedural audio generates sound algorithmically in real-time rather than playing back
pre-recorded samples. For a city builder, this offers three major advantages: infinite
variation (no recognizable loops), parameter-driven control (sound responds to simulation
data), and dramatically reduced memory footprint (a traffic hum generator uses kilobytes;
recorded traffic loops use megabytes).

### 7.1 Why Procedural Audio

**Memory comparison for ambient sounds:**

| Sound Type | Sample-Based (looping) | Procedural |
|---|---|---|
| Traffic hum (5 density levels) | 5 x 30s stereo @ 44.1kHz = ~52 MB | ~4 KB code + parameters |
| Rain (3 intensities) | 3 x 60s stereo = ~63 MB | ~6 KB code + parameters |
| Wind (variable speed) | 4 x 30s stereo = ~42 MB | ~3 KB code + parameters |
| Crowd murmur (3 densities) | 3 x 45s stereo = ~47 MB | ~5 KB code + parameters |
| **Total** | **~204 MB** | **~18 KB** |

The trade-off: procedural audio requires CPU processing on the audio thread, and achieving
natural-sounding results requires careful design. The approach for Megacity is **hybrid** --
procedural for continuous/repetitive sounds, sample-based for distinctive one-shots.

### 7.2 Traffic Hum Synthesis

Traffic hum is the most important procedural sound because it is always present, must
scale smoothly with traffic density, and should never sound like a recognizable loop.

#### 7.2.1 Synthesis Architecture

Traffic hum is synthesized from multiple layered noise generators:

```
Traffic Hum = {
    Layer 1: Engine Drone
        - Bandpass-filtered pink noise
        - Center frequency: 120-180 Hz (scales with avg vehicle speed)
        - Bandwidth: 60-100 Hz
        - Volume: 0.3 * traffic_density

    Layer 2: Tire Noise
        - High-pass filtered white noise
        - Cutoff: 1200-2400 Hz (scales with speed)
        - Volume: 0.2 * traffic_density * speed_factor

    Layer 3: Rumble
        - Very low-pass filtered brownian noise
        - Cutoff: 40-80 Hz
        - Volume: 0.15 * traffic_density * heavy_vehicle_fraction

    Layer 4: Variation
        - Slow random amplitude modulation (0.1-0.5 Hz)
        - Applied to all layers
        - Prevents the drone from sounding static
        - Amplitude swing: +/- 15%
}
```

#### 7.2.2 Parameter Mapping

```rust
struct TrafficHumParams {
    // Input: from TrafficGrid
    density: f32,        // 0.0 to 1.0 (congestion_level)
    avg_speed: f32,      // derived from road type and congestion
    heavy_fraction: f32, // fraction of industrial traffic (0.0-0.3)

    // Derived synthesis parameters
    engine_freq: f32,    // 120 + density * 60 Hz
    engine_bw: f32,      // 60 + density * 40 Hz
    tire_cutoff: f32,    // 1200 + avg_speed * 12 Hz
    rumble_vol: f32,     // 0.15 * density * heavy_fraction
    mod_rate: f32,       // 0.1 + density * 0.4 Hz (busier = faster modulation)
}

impl TrafficHumParams {
    fn from_traffic(density: f32, road_type: RoadType, industrial_nearby: bool) -> Self {
        let avg_speed = road_type.speed() * (1.0 - density * 0.7); // congestion slows traffic
        let heavy_fraction = if industrial_nearby { 0.2 } else { 0.05 };

        Self {
            density,
            avg_speed,
            heavy_fraction,
            engine_freq: 120.0 + density * 60.0,
            engine_bw: 60.0 + density * 40.0,
            tire_cutoff: 1200.0 + avg_speed * 12.0,
            rumble_vol: 0.15 * density * heavy_fraction,
            mod_rate: 0.1 + density * 0.4,
        }
    }
}
```

### 7.3 Rain and Weather Synthesis

Rain is another excellent candidate for procedural audio because natural rain has
inherent randomness that loops can never fully capture.

#### 7.3.1 Rain Synthesis Model

```
Rain = {
    Layer 1: Broadband Rain Wash
        - Filtered white noise
        - Spectral shaping: slight emphasis at 2-6 kHz (raindrop impact frequency)
        - Volume: rain_intensity * 0.6
        - Provides the "wall of rain" background

    Layer 2: Individual Drops (close zoom only)
        - Triggered procedurally at random intervals
        - Each drop: brief (10-30ms) white noise burst with fast exponential decay
        - Pitch randomized: 2000-8000 Hz center frequency
        - Rate: rain_intensity * 200 drops/second (spread across stereo field)
        - Spatial: random panning for each drop

    Layer 3: Surface Interaction
        - Modified by surface type under camera
        - Pavement: sharper drops, more high frequency
        - Foliage: softer drops, more mid frequency
        - Water: deeper, more resonant drops (like bubbles)
        - Applied as EQ on Layer 1

    Layer 4: Gutter/Drainage
        - Filtered noise with resonant peaks at 300-600 Hz
        - Only active when rain has been falling for > 60 game minutes
        - Simulates accumulated water flow
        - Volume ramps up slowly: 0.0 to 0.3 over 5 minutes
}
```

#### 7.3.2 Thunder Synthesis

Thunder can be synthesized from shaped noise rather than pre-recorded:

```
Thunder Strike = {
    1. Initial crack (close thunder only):
       - White noise burst, 20ms
       - Full spectrum, sharp attack
       - Volume: 0.8 * closeness

    2. Main body:
       - Brown noise (random walk)
       - Duration: 1-4 seconds (longer for distant thunder)
       - Low-pass filter: 200-2000 Hz (lower = more distant)
       - Amplitude envelope: attack 50ms, sustain variable, release 1-3s

    3. Tail / rolling:
       - Continuation of brown noise
       - Slow amplitude modulation at 1-3 Hz (creates rolling effect)
       - Progressive low-pass: cutoff drops from 1000 to 200 Hz over duration
       - Duration: 2-8 seconds

    4. Distance processing:
       - closeness = 0.0 (very far) to 1.0 (directly overhead)
       - Volume: 0.3 + closeness * 0.7
       - Low-pass cutoff: 500 + closeness * 7500 Hz
       - Pre-delay: (1.0 - closeness) * 2.0 seconds (time between flash and sound)
}
```

### 7.4 Wind Synthesis

Wind is produced by filtered noise with slow, naturalistic modulation:

```
Wind = {
    Base: Pink noise

    Processing chain:
    1. Bandpass filter
       - Center: 300 + wind_speed * 2000 Hz
       - Bandwidth: 200 + wind_speed * 1000 Hz
       - Resonance: 0.3 + wind_speed * 0.5

    2. Amplitude modulation
       - LFO rate: 0.05 + wind_speed * 0.3 Hz (very slow)
       - LFO depth: 0.3 + wind_speed * 0.4
       - Use multiple LFOs at slightly different rates for organic feel

    3. Gust generator (overlaid)
       - Triggered randomly (interval: 5-20 seconds / wind_speed)
       - Short burst of higher-amplitude, higher-frequency noise
       - Attack: 0.5-1.0s, decay: 1.0-3.0s
       - Adds pitch sweep upward then downward

    4. Urban canyon modifier (if applicable)
       - Add resonant filter at 800-1200 Hz (whistling)
       - Tighter bandwidth creates more tonal, eerie quality
}
```

#### 7.4.1 Wind Gust Algorithm

```rust
struct WindGustGenerator {
    next_gust_time: f32,
    current_gust: Option<GustState>,
    wind_speed: f32,
}

struct GustState {
    start_time: f32,
    duration: f32,        // 2-5 seconds
    peak_amplitude: f32,  // 1.5-3.0x base wind
    peak_time: f32,       // 0.3-0.5 of duration (asymmetric: fast rise, slow decay)
}

impl WindGustGenerator {
    fn amplitude_at(&self, time: f32) -> f32 {
        if let Some(gust) = &self.current_gust {
            let elapsed = time - gust.start_time;
            if elapsed > gust.duration { return 1.0; }

            let peak_position = gust.peak_time * gust.duration;
            let normalized = if elapsed < peak_position {
                // Attack phase: fast exponential rise
                (elapsed / peak_position).powf(0.5) // concave up = fast rise
            } else {
                // Decay phase: slow exponential decay
                let decay_elapsed = elapsed - peak_position;
                let decay_duration = gust.duration - peak_position;
                1.0 - (decay_elapsed / decay_duration).powf(2.0) // convex = slow start then fast end
            };

            1.0 + (gust.peak_amplitude - 1.0) * normalized
        } else {
            1.0
        }
    }
}
```

### 7.5 Crowd Murmur Synthesis

Crowd sounds in commercial areas can be synthesized from modulated noise:

```
Crowd Murmur = {
    Base: Band-limited noise (100-4000 Hz, emphasizing 200-600 Hz speech range)

    Processing:
    1. Formant filtering (simulate vocal tract resonances)
       - F1: 300-800 Hz (varies slowly)
       - F2: 1000-2500 Hz (varies slowly)
       - Creates vowel-like quality without actual speech

    2. Amplitude modulation
       - Multiple overlapping envelopes at 2-6 Hz
       - Simulates individual voices starting and stopping
       - More modulators at higher density = more "crowded" sound

    3. Spatial distribution
       - Split into 4-8 independent channels with random panning
       - Each channel has slightly different formant parameters
       - Creates sense of multiple distinct voices in space

    4. Density control
       - Low density: fewer modulators, more silence between "voices"
       - High density: more modulators, continuous murmur
       - Very high density: becomes more noise-like (voices blend into roar)
}
```

### 7.6 Hybrid Approach

The recommended approach for Megacity is hybrid: use procedural generation for
continuous ambient sounds and sample-based playback for distinctive one-shots.

#### 7.6.1 What Should Be Procedural

| Sound | Why Procedural |
|---|---|
| Traffic hum | Must scale smoothly with 0-100% congestion; loops would be recognizable |
| Rain | Infinite variation needed; surface interaction requires parameterization |
| Wind | Must respond to real-time wind speed; gusts need organic randomness |
| Crowd murmur | Density-dependent; loops would repeat noticeably |
| HVAC/machinery hum | Simple synthesis; saves memory for many building types |
| Electrical buzz | Trivial to synthesize; would be wasteful to record |

#### 7.6.2 What Should Be Sample-Based

| Sound | Why Samples |
|---|---|
| Dog bark | Highly recognizable; bad synthesis sounds robotic |
| Car horn | Distinctive and brief; needs to sound "real" |
| Birdsong | Extremely complex natural sound; synthesis sounds artificial |
| Construction impacts | Transient-rich; synthesis lacks the right texture |
| UI clicks/chimes | Must be consistent and polished; design in a DAW |
| Music stems | Obviously sample-based |
| Notification earcons | Designed for specific emotional impact |
| Siren | Could be synthesized but samples are more realistic |
| Thunder (alternative) | Can be either; synthesis offers infinite variation |

#### 7.6.3 Kira Integration for Procedural Audio

Kira does not natively support procedural audio generation -- it expects audio data
from files or streams. The integration approach is to generate audio samples in a
custom `Sound` implementation:

```rust
/// Custom Kira sound that generates procedural audio.
/// Runs on Kira's audio thread, not the main thread.
struct ProceduralTrafficSound {
    params: Arc<AtomicTrafficParams>,  // shared with main thread for updates
    phase: f32,
    noise_state: NoiseGenerator,
    sample_rate: f32,
}

impl Sound for ProceduralTrafficSound {
    fn process(&mut self, dt: f64, output: &mut [Frame]) {
        let params = self.params.load();

        for frame in output.iter_mut() {
            // Generate one sample of traffic hum
            let engine = self.noise_state.pink() * params.engine_volume;
            let tire = self.noise_state.white() * params.tire_volume;
            // ... apply filters, modulation

            let sample = engine + tire;
            *frame = Frame::from_mono(sample);
        }
    }
}
```

**Performance note:** Procedural audio generation runs on Kira's audio thread, which
is separate from the main/render threads. The main thread only needs to update shared
parameters (traffic density, wind speed, etc.) using atomic operations or lock-free
channels. This keeps the procedural audio cost out of the ECS performance budget entirely.

---

## 8. Technical Implementation in Bevy

This section provides the concrete Bevy ECS architecture for the audio system,
including plugin structure, resource design, system scheduling, and integration
with Megacity's existing crate workspace.

### 8.1 Plugin Architecture

Audio would live in a new `audio` crate within the workspace, or as a module within
the existing `rendering` crate (since it is a presentation concern, not simulation).

**Option A: Separate crate (recommended)**
```
crates/
  app/          -- binary, top-level
  simulation/   -- game logic
  rendering/    -- visual presentation
  audio/        -- audio presentation (NEW)
  ui/           -- egui interface
  save/         -- serialization
```

**Option B: Module within rendering**
```
crates/rendering/src/
  audio/
    mod.rs
    ambient.rs
    music.rs
    spatial.rs
    notifications.rs
    procedural.rs
```

**Recommendation:** Option A. Audio is complex enough to warrant its own crate, and it
depends on `simulation` (for reading game state) but not on `rendering` (audio and
visuals are independent).

#### 8.1.1 Plugin Definition

```rust
// crates/audio/src/lib.rs

use bevy::prelude::*;

pub mod ambient;
pub mod bus;
pub mod emitters;
pub mod lod;
pub mod music;
pub mod notifications;
pub mod procedural;
pub mod spatial;
pub mod weather_audio;

pub struct MegacityAudioPlugin;

impl Plugin for MegacityAudioPlugin {
    fn build(&self, app: &mut App) {
        app
            // Kira audio plugin (replaces default bevy_audio)
            .add_plugins(AudioPlugin)

            // Resources
            .init_resource::<AudioSettings>()
            .init_resource::<AudioBuses>()
            .init_resource::<AudioListener>()
            .init_resource::<MusicMixer>()
            .init_resource::<MusicSequencer>()
            .init_resource::<StingerSystem>()
            .init_resource::<NotificationCooldowns>()
            .init_resource::<DuckingState>()
            .init_resource::<ChunkAudioProfiles>()
            .init_resource::<AudioLodState>()
            .init_resource::<WeatherAudioState>()

            // Startup systems
            .add_systems(Startup, (
                bus::setup_audio_buses,
                music::load_music_assets,
                ambient::load_ambient_assets,
                notifications::load_notification_assets,
            ).chain())

            // Update systems -- core audio (every frame)
            .add_systems(Update, (
                spatial::update_audio_listener,
                lod::update_audio_lod_tier,
                spatial::cull_distant_emitters,
                spatial::update_emitter_volumes,
                music::update_music_stem_volumes,
                weather_audio::update_weather_audio,
                notifications::process_notification_queue,
                bus::apply_ducking,
            ).chain())

            // Slow-tick systems (match simulation rate)
            .add_systems(FixedUpdate, (
                ambient::update_chunk_audio_profiles,
                ambient::update_zone_ambience,
                music::sample_city_state,
                music::evaluate_section_transition,
            ))

            // Event listeners
            .add_systems(Update, (
                notifications::on_city_event,
                notifications::on_achievement,
                music::on_disaster_change,
            ));
    }
}
```

### 8.2 Audio Resource and Component Design

#### 8.2.1 Core Resources

```rust
/// Player-configurable audio settings, persisted to disk.
#[derive(Resource, Serialize, Deserialize)]
pub struct AudioSettings {
    pub master_volume: f32,    // 0.0-1.0
    pub music_volume: f32,
    pub ambience_volume: f32,
    pub sfx_volume: f32,
    pub ui_volume: f32,
    pub notifications_mode: NotificationMode,
    pub mono_audio: bool,
    pub spatial_audio_enabled: bool,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            master_volume: 0.8,
            music_volume: 0.7,
            ambience_volume: 0.6,
            sfx_volume: 0.8,
            ui_volume: 0.5,
            notifications_mode: NotificationMode::All,
            mono_audio: false,
            spatial_audio_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum NotificationMode {
    All,
    CriticalOnly,
    Off,
}

/// Handles to the Kira mixer tracks (buses).
#[derive(Resource)]
pub struct AudioBuses {
    pub master: TrackHandle,
    pub music: TrackHandle,
    pub ambience: TrackHandle,
    pub sfx: TrackHandle,
    pub ui: TrackHandle,
    // Sub-buses
    pub zone_ambience: TrackHandle,
    pub weather: TrackHandle,
    pub traffic: TrackHandle,
    pub environmental: TrackHandle,
}

/// Audio listener state, synced to camera.
#[derive(Resource)]
pub struct AudioListener {
    pub position: Vec3,
    pub audible_radius: f32,
    pub current_lod: AudioLodTier,
}

impl Default for AudioListener {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            audible_radius: 1000.0,
            current_lod: AudioLodTier::Abstract,
        }
    }
}
```

#### 8.2.2 Sound Asset Registry

Rather than loading audio assets on-demand (which causes hitches), all frequently-used
sounds are pre-loaded during startup into a registry:

```rust
#[derive(Resource)]
pub struct SoundAssets {
    /// Keyed by SoundBankId. Each bank contains multiple variations.
    pub banks: HashMap<SoundBankId, Vec<Handle<AudioSource>>>,

    /// Music stems, keyed by (piece_id, stem_type)
    pub music_stems: HashMap<(MusicPieceId, StemType), Handle<AudioSource>>,

    /// Notification earcons
    pub notifications: HashMap<NotificationType, Handle<AudioSource>>,

    /// Tool sounds
    pub tool_sounds: HashMap<ToolSoundId, Handle<AudioSource>>,

    /// Ambient loops
    pub ambient_loops: HashMap<AmbienceLoopId, Handle<AudioSource>>,
}
```

### 8.3 System Scheduling

Audio systems must be carefully scheduled to avoid frame spikes and ensure they have
access to the simulation data they need.

#### 8.3.1 System Ordering

```
Frame Start
│
├── FixedUpdate (simulation tick, 10Hz at 1x speed)
│   ├── simulation systems (traffic, buildings, weather, etc.)
│   ├── update_chunk_audio_profiles    -- reads grid, traffic, noise
│   ├── update_zone_ambience           -- adjusts ambient layer volumes
│   ├── sample_city_state              -- reads stats for music
│   └── evaluate_section_transition    -- music sequencer logic
│
├── Update (every frame, 60Hz)
│   ├── update_audio_listener          -- reads OrbitCamera
│   ├── update_audio_lod_tier          -- computes LOD from distance
│   ├── cull_distant_emitters          -- deactivates far emitters
│   ├── update_emitter_volumes         -- distance attenuation + occlusion
│   ├── update_music_stem_volumes      -- tweens stem volumes
│   ├── update_weather_audio           -- reads Weather resource
│   ├── process_notification_queue     -- plays queued notifications
│   └── apply_ducking                  -- adjusts bus volumes for ducking
│
└── Kira Audio Thread (independent, ~5ms buffer)
    ├── Mix all active sounds
    ├── Apply effects (reverb, filter)
    ├── Process procedural generators
    └── Output to hardware
```

#### 8.3.2 Run Conditions

Many audio systems do not need to run every frame. Use Bevy's run conditions to
reduce CPU usage:

```rust
// Only run zone ambience updates on the slow tick
.add_systems(FixedUpdate,
    update_zone_ambience.run_if(resource_exists::<SlowTickTimer>())
)

// Only run music transitions when music is not paused
.add_systems(Update,
    update_music_stem_volumes.run_if(|settings: Res<AudioSettings>| {
        settings.music_volume > 0.0
    })
)

// Only run spatial audio when enabled
.add_systems(Update,
    (cull_distant_emitters, update_emitter_volumes)
        .run_if(|settings: Res<AudioSettings>| {
            settings.spatial_audio_enabled
        })
)
```

### 8.4 Memory Management and Streaming

#### 8.4.1 Audio Memory Budget

Target: **50 MB total audio memory** (modest for a modern game).

| Category | Budget | Format | Notes |
|---|---|---|---|
| Music stems (loaded set) | 20 MB | OGG Vorbis, streamed | Only current piece loaded |
| Ambient loops | 8 MB | OGG Vorbis, streamed | 8-10 loops of 30-60s |
| Sound banks (one-shots) | 10 MB | WAV, pre-loaded | ~200 short sounds |
| Notification earcons | 2 MB | WAV, pre-loaded | ~30 sounds |
| Tool/UI sounds | 2 MB | WAV, pre-loaded | ~40 sounds |
| Procedural generators | ~0 MB | Code only | No audio data |
| Buffer/headroom | 8 MB | | Safety margin |
| **Total** | **50 MB** | | |

#### 8.4.2 Streaming Strategy

Long audio (music, ambient loops) should be streamed from disk rather than fully
decoded into memory:

```rust
// Kira streaming configuration
let music_settings = StaticSoundSettings::new()
    .output_destination(&music_bus)
    .start_position(0.0)
    // Kira handles streaming internally for StaticSoundData
    // loaded from file via kira::sound::static_sound::StaticSoundData::from_file()
;

// For Bevy integration via bevy_kira_audio:
let handle = audio.play(music_asset)
    .with_volume(Volume::Amplitude(0.0))  // start silent, tween in
    .looped()
    .handle();
```

#### 8.4.3 Music Piece Loading

Only one musical piece (all its stems) is loaded at a time. When transitioning
to a new piece, the next piece's stems are loaded asynchronously before the transition:

```rust
fn preload_next_piece(
    commands: &mut Commands,
    asset_server: &AssetServer,
    next_piece: MusicPieceId,
) {
    // Load all stems for the next piece
    let stems = [
        StemType::BasePad,
        StemType::Harmonic,
        StemType::Rhythmic,
        StemType::Bass,
        StemType::Melodic,
        StemType::Accent,
        StemType::Tension,
    ];

    for stem in stems {
        let path = format!("audio/music/{}/{}.ogg", next_piece.name(), stem.filename());
        let handle: Handle<AudioSource> = asset_server.load(&path);
        // Store handle for when transition occurs
    }
}
```

### 8.5 Audio Asset Pipeline

#### 8.5.1 Directory Structure

```
assets/audio/
├── music/
│   ├── village_theme/
│   │   ├── base_pad.ogg
│   │   ├── harmonic.ogg
│   │   ├── rhythmic.ogg
│   │   ├── bass.ogg
│   │   ├── melodic.ogg
│   │   ├── accent.ogg
│   │   └── tension.ogg
│   ├── town_theme/
│   │   └── ... (same structure)
│   ├── city_theme/
│   │   └── ...
│   ├── metropolis_theme/
│   │   └── ...
│   └── stingers/
│       ├── milestone.ogg
│       ├── disaster_start.ogg
│       ├── disaster_end.ogg
│       ├── building_complete.ogg
│       └── achievement.ogg
├── ambience/
│   ├── zones/
│   │   ├── residential_low_day.ogg
│   │   ├── residential_low_night.ogg
│   │   ├── residential_high_day.ogg
│   │   ├── commercial_day.ogg
│   │   ├── commercial_night.ogg
│   │   ├── industrial_day.ogg
│   │   ├── office_day.ogg
│   │   └── park.ogg
│   ├── weather/
│   │   ├── rain_light.ogg
│   │   ├── rain_heavy.ogg
│   │   ├── wind_gentle.ogg
│   │   ├── wind_strong.ogg
│   │   └── storm.ogg
│   ├── nature/
│   │   ├── birds_spring.ogg
│   │   ├── birds_summer.ogg
│   │   ├── crickets.ogg
│   │   ├── dawn_chorus.ogg
│   │   └── night_insects.ogg
│   └── water/
│       ├── river_flow.ogg
│       ├── lake_lapping.ogg
│       ├── ocean_waves.ogg
│       └── fountain_splash.ogg
├── sfx/
│   ├── construction/
│   │   ├── hammer_01.wav
│   │   ├── hammer_02.wav
│   │   ├── drill_01.wav
│   │   ├── crane_motor.wav
│   │   └── backup_beeper.wav
│   ├── traffic/
│   │   ├── horn_01.wav
│   │   ├── horn_02.wav
│   │   ├── horn_03.wav
│   │   ├── brake_squeal.wav
│   │   ├── bus_air_brake.wav
│   │   └── siren_01.wav
│   ├── nature/
│   │   ├── dog_bark_01.wav
│   │   ├── dog_bark_02.wav
│   │   ├── bird_chirp_01.wav
│   │   ├── owl_hoot_01.wav
│   │   └── cricket_chirp.wav
│   ├── disasters/
│   │   ├── tornado_roar.ogg
│   │   ├── earthquake_rumble.ogg
│   │   ├── flood_rush.ogg
│   │   ├── fire_crackle.ogg
│   │   ├── glass_break.wav
│   │   └── debris_settle.wav
│   └── buildings/
│       ├── cash_register.wav
│       ├── door_bell.wav
│       ├── elevator_ding.wav
│       └── ac_hum.ogg
├── ui/
│   ├── notifications/
│   │   ├── disaster_alert.wav
│   │   ├── budget_crisis.wav
│   │   ├── milestone.wav
│   │   ├── building_complete.wav
│   │   ├── policy_enacted.wav
│   │   └── achievement.wav
│   ├── tools/
│   │   ├── road_click.wav
│   │   ├── road_stretch.ogg
│   │   ├── road_confirm.wav
│   │   ├── zone_paint.ogg
│   │   ├── zone_claim.wav
│   │   ├── bulldoze_light.wav
│   │   ├── bulldoze_heavy.wav
│   │   ├── cancel.wav
│   │   └── error_buzz.wav
│   └── menu/
│       ├── hover.wav
│       ├── click.wav
│       ├── panel_open.wav
│       ├── panel_close.wav
│       └── toggle.wav
└── procedural/
    └── (empty -- procedural sounds are generated in code)
```

### 8.6 Format Selection

| Format | Use Case | Compression | Decode Latency | Streaming | Quality |
|---|---|---|---|---|---|
| OGG Vorbis | Music, long loops | ~10:1 | 5-20ms | Excellent | Good at 128-192 kbps |
| WAV (PCM) | Short SFX, UI | None (1:1) | ~0ms | Not needed | Perfect |
| FLAC | Archival only | ~2:1 lossless | 5-10ms | Possible | Perfect |
| MP3 | Not recommended | ~10:1 | Higher | Good | Inferior to OGG at same bitrate |

**Specifications:**
- Sample rate: 44,100 Hz for all audio (48,000 Hz is unnecessary for non-cinematic audio)
- Bit depth: 16-bit for WAV SFX, OGG handles this internally
- Channels: Mono for spatial SFX (spatialized by engine), Stereo for music/ambient
- OGG quality: 6 (roughly 192 kbps) for music, 4 (roughly 128 kbps) for ambient loops

### 8.7 Spatial Audio Implementation

#### 8.7.1 Kira Spatial Audio

Kira provides spatial audio through its `SpatialScene` feature, which handles
3D positioning, distance attenuation, and panning:

```rust
fn setup_spatial_audio(
    mut commands: Commands,
    mut audio_manager: ResMut<AudioManager>,
) {
    // Create a spatial scene with our attenuation settings
    let scene = audio_manager.add_spatial_scene(
        SpatialSceneSettings::new()
            .listener_position(Vec3::ZERO)
    ).unwrap();

    commands.insert_resource(SpatialAudioScene(scene));
}

fn update_spatial_emitter(
    spatial_scene: Res<SpatialAudioScene>,
    listener: Res<AudioListener>,
    emitters: Query<(&AudioEmitter, &Transform)>,
) {
    // Update listener position
    spatial_scene.0.set_listener_position(listener.position);

    for (emitter, transform) in &emitters {
        if let Some(handle) = &emitter.instance_handle {
            // Update emitter position in the spatial scene
            handle.set_position(transform.translation);
        }
    }
}
```

#### 8.7.2 HRTF for Headphones

Head-Related Transfer Function (HRTF) processing simulates how sound reaches each
ear differently based on direction, providing accurate 3D positioning for headphone
users. Kira supports HRTF through its spatial scene panning law:

```rust
// HRTF is configured per spatial scene
let scene = audio_manager.add_spatial_scene(
    SpatialSceneSettings::new()
        // Use HRTF panning for headphone users
        // Falls back to stereo panning for speakers
);
```

**Settings option:** Provide a toggle between "Headphones (HRTF)" and "Speakers (Stereo)"
in the audio settings. HRTF on speakers sounds wrong (too narrow), and stereo panning on
headphones sounds flat (no depth).

### 8.8 Save/Load Considerations

Audio state is transient -- it should NOT be serialized in save files. When loading a
save, the audio system reconstructs its state from simulation data:

```rust
fn on_save_loaded(
    stats: Res<CityStats>,
    weather: Res<Weather>,
    clock: Res<GameClock>,
    disaster: Res<ActiveDisaster>,
    mut music: ResMut<MusicMixer>,
    mut ambient: ResMut<ZoneAmbienceState>,
) {
    // Sample current city state and set music accordingly
    let snapshot = MusicStateSnapshot::sample(
        &stats, &weather, &clock, &disaster, /* ... */
    );

    // Start music at appropriate state (skip intro, go directly to correct section)
    music.initialize_from_snapshot(&snapshot);

    // Set ambient layers to correct volumes (no crossfade, immediate)
    ambient.initialize_from_grid(/* grid data */);

    // Weather audio matches current weather
    // All spatial emitters will be created naturally by the chunk update system
}
```

**What is NOT saved:**
- Music playback position (restart from appropriate section)
- Ambient layer volumes (reconstructed from grid)
- Active sound instances (all sounds restart)
- Audio settings (saved separately in a settings file, not the game save)

**What IS saved (in settings file, not game save):**
- Master/Music/Ambience/SFX/UI volume levels
- Notification mode
- Mono audio toggle
- Headphone/speaker mode

---

## 9. Mixing and Mastering

A game with dozens of simultaneous audio sources will sound like noise without careful
mixing. This section establishes loudness targets, frequency allocation, dynamic range
management, and the ducking chains that keep everything intelligible.

### 9.1 Loudness Normalization

All audio assets should be normalized to consistent loudness levels before being
integrated into the game. This ensures that a dog bark and a car horn have predictable
relative volumes controlled by the game's mixing logic rather than the source recording level.

#### 9.1.1 LUFS Targets by Category

LUFS (Loudness Units relative to Full Scale) is the broadcast standard for loudness
measurement. It accounts for perceived loudness across frequencies, unlike simple
peak measurement.

| Category | Target LUFS | True Peak Max | Notes |
|---|---|---|---|
| Music stems (individual) | -20 LUFS | -3 dBTP | Quiet individually; full mix should hit -14 |
| Music (full mix target) | -14 LUFS | -1 dBTP | Standard for interactive media |
| Ambient loops | -18 LUFS | -3 dBTP | Background; should not dominate |
| SFX one-shots | -16 LUFS | -1 dBTP | Need to punch through mix |
| UI/Notification (info) | -22 LUFS | -6 dBTP | Very subtle |
| UI/Notification (important) | -16 LUFS | -3 dBTP | Clear but not alarming |
| UI/Notification (critical) | -12 LUFS | -1 dBTP | Demands attention |
| Disaster sounds | -10 LUFS | -1 dBTP | Loudest category, rare |

#### 9.1.2 Normalization Process

```
For each audio asset:
1. Measure integrated LUFS using loudness meter (ffmpeg, dpMeter, etc.)
2. Calculate gain adjustment: target_LUFS - measured_LUFS = gain_dB
3. Apply gain adjustment
4. Verify true peak does not exceed limit
5. If true peak exceeds, apply limiter with 1ms attack, 100ms release
6. Re-measure and iterate if needed
```

### 9.2 Dynamic Range Management

City builder audio has extreme dynamic range -- from near-silence at 2am in a small
village to overwhelming noise during a tornado hitting a dense metropolis. Without
compression, quiet sounds get lost and loud sounds clip.

#### 9.2.1 Per-Bus Compression Settings

| Bus | Threshold | Ratio | Attack | Release | Makeup Gain |
|---|---|---|---|---|---|
| Master | -6 dBFS | 2:1 | 30ms | 300ms | +2 dB |
| Music | -12 dBFS | 1.5:1 | 50ms | 500ms | +1 dB |
| Ambience | -10 dBFS | 2:1 | 20ms | 200ms | +2 dB |
| SFX | -8 dBFS | 3:1 | 5ms | 100ms | +3 dB |
| UI | -6 dBFS | 4:1 | 1ms | 50ms | +3 dB |

**Master limiter:** A brick-wall limiter at -0.3 dBFS on the master bus prevents
any clipping under any combination of simultaneous sounds.

### 9.3 Frequency Band Allocation

To prevent frequency masking (where one sound makes another inaudible because they
occupy the same frequencies), each audio category is given a primary frequency band:

```
Frequency Spectrum Allocation:

20 Hz ──────── 80 Hz ──────── 200 Hz ──────── 600 Hz ──────── 2 kHz ──────── 8 kHz ──────── 20 kHz
   │              │               │                │               │              │              │
   │  Sub-bass    │   Low-mids    │    Mid-range    │   Upper-mids  │   Presence   │    Air       │
   │              │               │                │               │              │              │
   │  Earthquake  │  Traffic hum  │  Crowd murmur  │  Music melody │  Birdsong    │  Rain detail │
   │  Thunder     │  Industrial   │  Dialogue       │  Horns        │  Notific.    │  Wind detail │
   │  Wind howl   │  Bass stem    │  Music harmony  │  Constr. hits │  Bells/chime │  Crickets    │
   │              │  Engine drone │  AC hum         │  Siren        │  Dog bark    │  Shimmer     │
```

#### 9.3.1 EQ Recommendations

When sources conflict in the same frequency band, use subtle EQ on the less important
source to create space:

| Conflict | Solution |
|---|---|
| Music bass vs traffic rumble | High-pass music bass at 60 Hz; low-pass traffic at 150 Hz |
| Birdsong vs notification chimes | Notch birdsong slightly at chime frequency during playback |
| Crowd murmur vs music harmony | Sidechain crowd to music; duck crowd -2 dB when harmony plays |
| Rain detail vs crickets | They naturally occupy similar ranges; reduce crickets during rain |
| Construction vs industrial ambient | They're thematically related; let them blend |

### 9.4 Ducking Chains

Ducking is automatic volume reduction of one bus when another bus is active. This
ensures intelligibility of important sounds.

#### 9.4.1 Ducking Matrix

```
When THIS plays...    → ...duck THESE by:
────────────────────────────────────────────────
Disaster SFX          → Music -8 dB, Ambience -6 dB, Traffic -4 dB
Critical notification → Music -6 dB, Ambience -4 dB
Stinger               → Music stems -3 dB (not base pad)
Important notification→ Music -2 dB
Construction (loud)   → Zone ambience -2 dB (same chunk only)
Fire audio            → Normal ambience in fire area -4 dB
Weather (storm)       → Zone ambience -3 dB, Traffic -2 dB
```

#### 9.4.2 Ducking Envelope

All ducking uses a consistent envelope to prevent pumping artifacts:

```
Attack (duck onset):  50-100ms   -- fast enough to not miss the trigger
Hold:                 Duration of trigger sound + 100ms
Release (duck end):   300-500ms  -- slow release prevents pumping
```

```rust
struct DuckEnvelope {
    state: DuckState,
    current_db: f32,
    target_db: f32,
    attack_rate: f32,   // dB per second
    release_rate: f32,  // dB per second
    hold_timer: Timer,
}

enum DuckState {
    Idle,
    Attacking,
    Holding,
    Releasing,
}

impl DuckEnvelope {
    fn update(&mut self, dt: f32) {
        match self.state {
            DuckState::Idle => {},
            DuckState::Attacking => {
                self.current_db -= self.attack_rate * dt;
                if self.current_db <= self.target_db {
                    self.current_db = self.target_db;
                    self.state = DuckState::Holding;
                }
            },
            DuckState::Holding => {
                self.hold_timer.tick(Duration::from_secs_f32(dt));
                if self.hold_timer.finished() {
                    self.state = DuckState::Releasing;
                }
            },
            DuckState::Releasing => {
                self.current_db += self.release_rate * dt;
                if self.current_db >= 0.0 {
                    self.current_db = 0.0;
                    self.state = DuckState::Idle;
                }
            },
        }
    }
}
```

### 9.5 Master Bus Processing

The final audio output goes through the master bus with these processing stages:

```
Signal Flow (Master Bus):

Individual tracks
    ↓
Sub-bus mixing (zone, weather, traffic, environmental)
    ↓
Bus-level compression (per-bus settings from 9.2.1)
    ↓
Bus-level ducking (from 9.4)
    ↓
Bus volume (player settings)
    ↓
Master bus sum
    ↓
Master compression (gentle, 2:1 at -6 dBFS)
    ↓
Master volume (player settings)
    ↓
Brick-wall limiter (-0.3 dBFS)
    ↓
Output to hardware (stereo or HRTF binaural)
```

---

## 10. What Makes City Builder Audio Great

This section analyzes the audio design of landmark city builders and management games,
extracting the design principles that Megacity should adopt.

### 10.1 SimCity 4 (2003)

SimCity 4 remains the gold standard for city builder audio 20+ years after release.

**Soundtrack (Jerry Martin, Kent Jolly, Marc Russo, Kirk Casey):**
- Genre: Contemporary jazz with ambient electronic elements
- 35+ tracks across three game modes (Mayor, God, My Sim)
- Mayor mode (main gameplay) has tracks ranging from upbeat swing to contemplative solo piano
- The diversity of the soundtrack prevents fatigue over hundreds of hours
- Standout tracks: "Bohemian Street Jam" (bustling commercial), "Sim Broadway" (metropolitan energy),
  "Night Breeze" (peaceful nighttime), "By The Bay" (waterfront contemplation)

**What makes it work:**
- **Quality over quantity:** Each track is a polished, radio-ready composition
- **Emotional range:** Not every track is "happy city music" -- there are melancholy,
  tense, and mysterious pieces that reflect the complexity of city management
- **Genre cohesion:** Jazz is inherently urban, making every track feel thematically appropriate
- **Non-intrusive:** Despite being excellent music, the tracks never fight for attention
  with the gameplay -- they support without dominating

**Zone-specific audio:**
- Zooming into residential areas brings domestic sounds (sprinklers, children, dogs)
- Commercial zones add crowd chatter and shop door bells
- Industrial zones introduce machinery and truck noise
- The transition between zones is seamless as you pan the camera

**Critical lesson:** Hire real musicians. SimCity 4's soundtrack was performed by professional
jazz musicians, and the organic quality of human performance is irreplaceable by synthesis
or sample libraries alone.

### 10.2 Cities: Skylines (2015)

Cities: Skylines has pleasant but ultimately forgettable audio that serves as a
cautionary tale about "good enough" sound design.

**Soundtrack:**
- Genre: Light electronic ambient, corporate-feeling instrumental
- Perfectly adequate background music that never offends or excites
- Low replay value -- many long-time players turn music off and play their own
- Expansion packs added themed music (snow, night, campus) that improved variety

**Chirper notification system:**
- Twitter-like notification bird that chirps when citizens have complaints/comments
- Became an iconic (and divisive) design choice
- The chirp sound itself became annoying to many players -- a key lesson in earcon design
- Modders created "Chirper silence" mods, indicating the sound was too intrusive

**What we learn:**
- **Forgettable is worse than absent:** If the player turns off your music to play Spotify,
  your soundtrack has failed. It should be indispensable.
- **Notification frequency matters as much as notification sound:** The Chirper was annoying
  not because the sound was bad, but because it fired too often
- **The game lacked adaptive elements:** Music did not respond to city state, reducing its
  emotional impact to zero
- **Spatial audio was minimal:** Zooming into different areas sounded similar, a missed
  opportunity for environmental storytelling

### 10.3 Frostpunk (2018)

Frostpunk (Piotr Musial, composer) is the emotional pinnacle of management game audio.

**Soundtrack:**
- Genre: Orchestral/choral with industrial percussion and folk influences
- Deeply emotional, sometimes overwhelmingly so
- The music makes you FEEL the cold, the desperation, the moral weight of decisions
- "The City Must Survive" is one of gaming's most powerful musical moments

**What makes it exceptional:**

1. **Silence as a tool:** During the quietest, most desperate moments, the music
   drops out entirely. The only sounds are wind, the generator hum, and distant
   coughing. Then, when hope returns, the music swells back in -- and the emotional
   impact is devastating.

2. **Industrial music from industrial sounds:** The percussion includes actual anvil
   strikes, steam venting, and gear grinding. This blurs the boundary between music
   and sound effects, making the entire audio landscape feel unified.

3. **Hope/Discontent as music drivers:** The game's two primary meters directly
   control the emotional character of the music. High Hope brings warmer harmonics,
   more major-key content, and eventually the choir. High Discontent brings dissonance,
   minor keys, and aggressive percussion.

4. **Adaptive but authored:** The music feels composed, not algorithmic. Each adaptive
   layer was written by a composer to work in combination with every other layer.
   This is the key distinction between good and bad adaptive music -- it must be
   *composed* adaptively, not *mixed* adaptively.

**Critical lesson:** The strongest audio design emerges when the music system is designed
in tandem with the game mechanics, not grafted on afterward. Frostpunk's Hope/Discontent
meters exist partly because they drive the music so effectively.

### 10.4 Anno 1800 (2019)

Anno 1800 (Dynamedion) demonstrates how period-appropriate instrumentation reinforces
game setting.

**Soundtrack:**
- Genre: 19th-century orchestral with folk instruments
- Uses instruments from the era: chamber strings, harpsichord, accordion, mandolin
- Different regions (Old World, New World, Arctic, Enbesa) have distinct musical identities
  using region-appropriate instruments (pan flute for New World, throat singing for Arctic)

**Adaptive elements:**
- Music escalates during naval combat with brass fanfares and martial drums
- Building phases use calm, productive music (strings, woodwinds)
- Trade route music has a sense of movement and journey
- Festival events trigger celebratory music with dancing rhythms

**What we learn:**
- **Thematic consistency:** Every instrument choice reinforces the setting
- **Regional variety:** Having distinct music for different areas prevents monotony
  and makes each area feel unique
- **Combat escalation:** Even in a primarily peaceful builder, moments of tension
  (disasters in our case) should have dramatically different music that makes
  the player's heart rate increase

### 10.5 Stardew Valley (2016)

Stardew Valley (ConcernedApe) demonstrates the power of seasonal and locational
music in a long-session game.

**Soundtrack:**
- Genre: Indie folk/acoustic, chiptune-influenced
- 70+ tracks covering all seasons, locations, events, and weather
- Each season has 8-10 unique tracks that COMPLETELY change the feel of the game
- Spring: bouncy, pastoral, optimistic
- Summer: lazy, warm, occasionally upbeat
- Fall: melancholy, harvest-themed, contemplative
- Winter: slow, crystalline, beautiful in its starkness

**Key techniques:**
- Rain overrides outdoor music with rain-specific tracks (not just rain SFX over normal music)
- Each location has its own music (farm, town, beach, mine, forest)
- Festivals have unique celebratory tracks
- Night market has its own exotic music
- Music volume automatically reduces in the mines (replaced by atmospheric sounds)

**What we learn:**
- **Seasonal music is transformative:** Stardew Valley's seasons feel completely different
  largely because of the music, not just the visuals
- **Weather affects music, not just SFX:** Rain should not just add rain sounds on top
  of sunny-day music -- it should change the underlying musical mood
- **Simplicity:** Clean, memorable melodies with simple instrumentation are more effective
  for long-session games than complex orchestral arrangements
- **Volume and presence:** The game knows when to be quiet (mines, night, rain) and when
  to be present (festivals, sunny farm days)

### 10.6 Key Insights for Megacity

Synthesizing lessons from all reference games:

1. **Audio communicates simulation state subconsciously.** A player who can hear
   congestion, prosperity, or danger without looking at an overlay is a player who
   is deeply immersed. This is the highest goal of city builder audio design.

2. **Adaptive music must be composed, not just mixed.** Every stem must work musically
   with every other stem in every possible combination. This requires a composer who
   understands the adaptive system, not a post-production mixing engineer.

3. **Silence is an instrument.** The absence of sound is as communicative as its presence.
   Night, winter, post-disaster quietude -- these moments of near-silence make the
   sound that follows more impactful.

4. **Notification sounds can become the most hated feature in the game.** Design them
   with extreme restraint. When in doubt, make them quieter, less frequent, or optional.

5. **Invest in quality.** SimCity 4's soundtrack is remembered 20 years later. Cities:
   Skylines' soundtrack is not. The difference is not technical -- it is artistic.
   Budget for real musicians, real compositions, and real sound design.

6. **The soundscape should evolve with the city.** A village of 500 people should
   sound fundamentally different from a metropolis of 500,000. This evolution should
   be gradual and feel earned -- the player's city *grew* into this sound.

7. **Zone identity through sound.** Experienced players should be able to identify
   which zone they are looking at with their eyes closed, just from the ambient sound.
   Residential = birds + dogs. Commercial = crowds + registers. Industrial = machines.

8. **Weather and seasons change everything.** Frostpunk's cold, Stardew's rain, Anno's
   regions -- the best audio designs treat environmental variation as a core feature,
   not a cosmetic overlay.

---

## 11. Sound Asset Catalog

This section provides a complete inventory of every sound asset needed for the initial
implementation, organized by category with production notes.

### 11.1 Music Assets

| Asset ID | Description | Format | Duration | Variations | Priority |
|---|---|---|---|---|---|
| `music/village/base_pad` | Gentle synth pad, C major, 70 BPM | OGG | 4 min loop | 1 | P0 |
| `music/village/harmonic` | Acoustic guitar arpeggios | OGG | 4 min loop | 1 | P0 |
| `music/village/rhythmic` | Light shaker rhythm | OGG | 4 min loop | 1 | P1 |
| `music/village/melodic` | Solo flute melody | OGG | 4 min loop | 1 | P1 |
| `music/village/tension` | Minor key drone, tremolo strings | OGG | 4 min loop | 1 | P1 |
| `music/town/base_pad` | Warmer pad, piano chords | OGG | 5 min loop | 1 | P0 |
| `music/town/harmonic` | Piano + vibraphone | OGG | 5 min loop | 1 | P0 |
| `music/town/rhythmic` | Brushed drums, walking bass | OGG | 5 min loop | 1 | P1 |
| `music/town/bass` | Upright bass | OGG | 5 min loop | 1 | P1 |
| `music/town/melodic` | Clarinet/muted trumpet | OGG | 5 min loop | 1 | P1 |
| `music/town/accent` | Glockenspiel, bells | OGG | 5 min loop | 1 | P2 |
| `music/town/tension` | Dissonant piano, timpani | OGG | 5 min loop | 1 | P1 |
| `music/city/base_pad` | Full string section pad | OGG | 6 min loop | 1 | P0 |
| `music/city/harmonic` | Jazz piano voicings | OGG | 6 min loop | 1 | P0 |
| `music/city/rhythmic` | Full drum kit with brushes | OGG | 6 min loop | 1 | P1 |
| `music/city/bass` | Electric bass, walking | OGG | 6 min loop | 1 | P1 |
| `music/city/melodic` | Saxophone melody | OGG | 6 min loop | 1 | P1 |
| `music/city/accent` | Brass stabs, percussion fills | OGG | 6 min loop | 1 | P2 |
| `music/city/tension` | Aggressive strings, snare rolls | OGG | 6 min loop | 1 | P1 |
| `music/metropolis/*` | Full orchestral, 7 stems | OGG | 7 min loop | 1 each | P1 |
| `music/night/*` | Ambient piano, 4 stems | OGG | 8 min loop | 1 each | P1 |

**Stingers:**

| Asset ID | Description | Format | Duration | Priority |
|---|---|---|---|---|
| `stinger/milestone_small` | Ascending chime, brief fanfare | OGG | 3s | P0 |
| `stinger/milestone_large` | Full brass fanfare | OGG | 5s | P1 |
| `stinger/disaster_start` | Timpani hit + brass stab | OGG | 3s | P0 |
| `stinger/disaster_end` | Resolving chord progression | OGG | 4s | P0 |
| `stinger/building_complete` | Gentle chime + string swell | OGG | 2s | P0 |
| `stinger/achievement` | Ascending arpeggio + bell | OGG | 3s | P1 |
| `stinger/policy` | Soft woodwind phrase | OGG | 2s | P2 |
| `stinger/budget_positive` | Ascending coin-like tones | OGG | 2s | P2 |
| `stinger/budget_negative` | Descending muted horn | OGG | 2s | P2 |

### 11.2 Ambient Assets

| Asset ID | Format | Duration | Loop | Notes |
|---|---|---|---|---|
| `amb/residential_low_day` | OGG | 60s | Yes | Birds, distant dogs, suburban quiet |
| `amb/residential_low_night` | OGG | 60s | Yes | Crickets, wind, distant dog |
| `amb/residential_high_day` | OGG | 45s | Yes | Muffled TV, elevator, apartment bustle |
| `amb/residential_high_night` | OGG | 45s | Yes | Distant siren, muffled music, quiet |
| `amb/commercial_day` | OGG | 45s | Yes | Crowd chatter, registers, door bells |
| `amb/commercial_night` | OGG | 45s | Yes | Neon buzz, reduced crowds, bar sounds |
| `amb/industrial_day` | OGG | 45s | Yes | Machinery, hammering, trucks |
| `amb/industrial_night` | OGG | 45s | Yes | Reduced machinery, security hum |
| `amb/office_day` | OGG | 45s | Yes | HVAC, muffled keyboards, elevator |
| `amb/park` | OGG | 60s | Yes | Rich birdsong, rustling, water |
| `amb/dawn_chorus` | OGG | 90s | Yes | Multi-species birdsong crescendo |
| `amb/rain_light` | OGG | 60s | Yes | Gentle patter, soft wash |
| `amb/rain_heavy` | OGG | 60s | Yes | Intense downpour, water streaming |
| `amb/storm` | OGG | 60s | Yes | Wind-driven rain, distant thunder |
| `amb/wind_gentle` | OGG | 60s | Yes | Light breeze, leaf rustle |
| `amb/wind_strong` | OGG | 45s | Yes | Howling, gusting, whistling |
| `amb/river` | OGG | 45s | Yes | Flowing water, bubbling |
| `amb/lake` | OGG | 45s | Yes | Gentle lapping, occasional plop |
| `amb/ocean` | OGG | 60s | Yes | Waves, gulls, deep roar |
| `amb/fountain` | OGG | 30s | Yes | Splashing, tinkling |
| `amb/crickets` | OGG | 30s | Yes | Evening cricket chorus |
| `amb/night_insects` | OGG | 45s | Yes | Mixed insect chorus, summer |

### 11.3 SFX Assets

| Asset ID | Format | Duration | Variations | Priority |
|---|---|---|---|---|
| `sfx/dog_bark` | WAV | 0.5-1.0s | 3 | P1 |
| `sfx/children_playing` | WAV | 2-4s | 5 | P2 |
| `sfx/cash_register` | WAV | 0.5s | 4 | P1 |
| `sfx/car_horn` | WAV | 0.3-0.8s | 3 | P0 |
| `sfx/brake_squeal` | WAV | 0.5s | 2 | P1 |
| `sfx/bus_air_brake` | WAV | 0.8s | 2 | P2 |
| `sfx/siren` | WAV | 2.0s | 3 | P0 |
| `sfx/hammer` | WAV | 0.3s | 4 | P1 |
| `sfx/drill` | WAV | 1.0s | 2 | P1 |
| `sfx/crane_motor` | WAV | 2.0s | 1 | P2 |
| `sfx/backup_beeper` | WAV | 1.5s | 2 | P1 |
| `sfx/owl_hoot` | WAV | 1.5s | 2 | P2 |
| `sfx/bird_chirp` | WAV | 0.3s | 5 | P1 |
| `sfx/glass_break` | WAV | 0.8s | 3 | P1 |
| `sfx/debris_fall` | WAV | 1.0s | 2 | P1 |
| `sfx/fire_crackle` | OGG | 5.0s loop | 2 | P0 |
| `sfx/tornado_roar` | OGG | 10.0s loop | 1 | P0 |
| `sfx/earthquake_rumble` | OGG | 8.0s loop | 1 | P0 |
| `sfx/flood_rush` | OGG | 10.0s loop | 1 | P0 |
| `sfx/thunder_close` | WAV | 3.0s | 3 | P1 |
| `sfx/thunder_distant` | WAV | 5.0s | 3 | P1 |
| `sfx/elevator_ding` | WAV | 0.3s | 1 | P2 |
| `sfx/door_bell` | WAV | 0.5s | 2 | P2 |
| `sfx/ac_hum` | OGG | 5.0s loop | 1 | P2 |

### 11.4 UI and Tool Assets

| Asset ID | Format | Duration | Priority |
|---|---|---|---|
| `ui/hover` | WAV | 0.05s | P0 |
| `ui/click` | WAV | 0.1s | P0 |
| `ui/panel_open` | WAV | 0.15s | P0 |
| `ui/panel_close` | WAV | 0.12s | P0 |
| `ui/tab_switch` | WAV | 0.1s | P1 |
| `ui/toggle_on` | WAV | 0.08s | P1 |
| `ui/toggle_off` | WAV | 0.08s | P1 |
| `ui/error` | WAV | 0.15s | P0 |
| `ui/confirm` | WAV | 0.3s | P0 |
| `ui/notif_disaster` | WAV | 2.0s | P0 |
| `ui/notif_budget_crisis` | WAV | 1.5s | P0 |
| `ui/notif_milestone` | WAV | 1.2s | P0 |
| `ui/notif_building` | WAV | 0.8s | P0 |
| `ui/notif_policy` | WAV | 0.5s | P1 |
| `ui/notif_achievement` | WAV | 1.0s | P1 |
| `tool/road_click` | WAV | 0.2s | P0 |
| `tool/road_stretch` | OGG | 3.0s loop | P0 |
| `tool/road_confirm` | WAV | 0.5s | P0 |
| `tool/zone_paint` | OGG | 2.0s loop | P0 |
| `tool/zone_claim` | WAV | 0.1s | P0 |
| `tool/bulldoze_light` | WAV | 0.5s | P0 |
| `tool/bulldoze_heavy` | WAV | 1.0s | P0 |
| `tool/cancel` | WAV | 0.3s | P0 |
| `tool/building_select` | WAV | 0.3s | P0 |
| `tool/building_place` | WAV | 0.5s | P0 |

**Total unique audio assets:** ~120 files
**Estimated total disk size:** ~80-100 MB uncompressed, ~30-40 MB as OGG/WAV mix
**Estimated memory footprint at runtime:** ~50 MB (streaming for long assets)

---

## 12. ECS Integration Architecture

This section provides the complete ECS architecture showing how audio systems integrate
with Megacity's existing simulation and rendering systems.

### 12.1 Component Summary

```rust
// === Audio Components (attached to entities) ===

/// Point audio emitter for individual entities (Tier 0 only)
#[derive(Component)]
pub struct AudioEmitter { /* see Section 2.6.1 */ }

/// Area audio emitter for chunks (Tier 1)
#[derive(Component)]
pub struct AreaAudioEmitter { /* see Section 2.6.2 */ }

/// Construction audio emitter (attached to building entities during construction)
#[derive(Component)]
pub struct ConstructionAudioEmitter { /* see Section 2.3.3 */ }

/// Fire audio emitter (attached to entities with OnFire component)
#[derive(Component)]
pub struct FireAudioEmitter {
    pub intensity: f32,
    pub handle: Option<SoundInstanceHandle>,
}

/// Disaster audio emitter (attached to disaster entity)
#[derive(Component)]
pub struct DisasterAudioEmitter {
    pub disaster_type: DisasterType,
    pub phase: DisasterAudioPhase,
    pub handles: Vec<SoundInstanceHandle>,
}
```

### 12.2 Resource Summary

```rust
// === Audio Resources (global state) ===

pub struct AudioSettings { /* player preferences, see 8.2.1 */ }
pub struct AudioBuses { /* Kira track handles, see 8.2.1 */ }
pub struct AudioListener { /* camera-synced listener, see 8.2.1 */ }
pub struct AudioLodState { /* current LOD tier, see 1.5 */ }
pub struct SoundAssets { /* pre-loaded sound handles, see 8.2.2 */ }
pub struct MusicMixer { /* stem states and volumes, see 3.2.2 */ }
pub struct MusicSequencer { /* section tracking, see 3.3.2 */ }
pub struct MusicStateSnapshot { /* sampled city state, see 3.4 */ }
pub struct StingerSystem { /* stinger queue and cooldowns, see 3.7.2 */ }
pub struct NotificationCooldowns { /* per-type cooldowns, see 5.4.1 */ }
pub struct DuckingState { /* active ducking envelopes, see 5.3.2 */ }
pub struct ChunkAudioProfiles { /* per-chunk audio data, see 2.7.1 */ }
pub struct WeatherAudioState { /* current weather sound state, see 4.1 */ }
pub struct WindGustGenerator { /* procedural wind gusts, see 7.4.1 */ }
```

### 12.3 System Dependency Graph

```
Simulation Systems (existing)
    │
    ├── update_weather ──────────────────────────→ update_weather_audio
    ├── update_traffic ──────────────────────────→ update_chunk_audio_profiles
    ├── update_noise_pollution ──────────────────→ update_chunk_audio_profiles
    ├── update_fire ─────────────────────────────→ fire_audio_sync
    ├── trigger_disaster ────────────────────────→ on_disaster_change
    ├── update_stats ────────────────────────────→ sample_city_state
    ├── buildings::spawn_buildings ──────────────→ construction_audio_spawn
    ├── buildings::complete_construction ────────→ construction_audio_remove + stinger
    └── events::generate_events ────────────────→ on_city_event

Rendering Systems (existing)
    │
    ├── camera::apply_orbit_camera ─────────────→ update_audio_listener
    └── day_night::update_day_night_cycle ──────→ (audio reads GameClock directly)

Audio Systems (new)
    │
    ├── update_audio_listener (Update, every frame)
    │   reads: OrbitCamera
    │   writes: AudioListener
    │
    ├── update_audio_lod_tier (Update, every frame)
    │   reads: AudioListener, OrbitCamera
    │   writes: AudioLodState
    │
    ├── cull_distant_emitters (Update, every frame)
    │   reads: AudioListener, AudioLodState
    │   writes: AudioEmitter (active flag), AreaAudioEmitter
    │
    ├── update_emitter_volumes (Update, every frame)
    │   reads: AudioListener, AudioEmitter positions
    │   writes: AudioEmitter volumes (-> Kira)
    │
    ├── update_chunk_audio_profiles (FixedUpdate, slow tick)
    │   reads: WorldGrid, TrafficGrid, NoisePollutionGrid, FireGrid
    │   writes: ChunkAudioProfiles
    │
    ├── update_zone_ambience (FixedUpdate, slow tick)
    │   reads: ChunkAudioProfiles, AudioListener, GameClock
    │   writes: AreaAudioEmitter volumes (-> Kira)
    │
    ├── sample_city_state (FixedUpdate, slow tick)
    │   reads: CityStats, Weather, GameClock, ActiveDisaster, CityBudget
    │   writes: MusicStateSnapshot
    │
    ├── update_music_stem_volumes (Update, every frame)
    │   reads: MusicStateSnapshot, MusicMixer
    │   writes: MusicMixer stems (-> Kira tweens)
    │
    ├── evaluate_section_transition (FixedUpdate, slow tick)
    │   reads: MusicStateSnapshot, MusicSequencer
    │   writes: MusicSequencer (section changes)
    │
    ├── update_weather_audio (Update, every frame)
    │   reads: Weather, GameClock, AudioListener
    │   writes: WeatherAudioState (-> Kira)
    │
    ├── process_notification_queue (Update, every frame)
    │   reads: notification events, NotificationCooldowns
    │   writes: UI bus sounds (-> Kira), DuckingState
    │
    ├── apply_ducking (Update, every frame, after process_notification_queue)
    │   reads: DuckingState
    │   writes: bus volumes (-> Kira)
    │
    ├── on_city_event (Update, event listener)
    │   reads: CityEvent
    │   writes: notification queue, StingerSystem
    │
    ├── on_disaster_change (Update, event listener)
    │   reads: ActiveDisaster (changed)
    │   writes: MusicMixer (tension override), DisasterAudioEmitter
    │
    ├── fire_audio_sync (Update, every frame)
    │   reads: FireGrid, OnFire components
    │   writes: FireAudioEmitter
    │
    ├── construction_audio_spawn (Update, event-driven)
    │   reads: newly spawned Building with build timer
    │   writes: spawns ConstructionAudioEmitter component
    │
    └── construction_audio_remove (Update, event-driven)
        reads: completed Building
        writes: removes ConstructionAudioEmitter, triggers stinger
```

### 12.4 Event-Driven Audio Triggers

Rather than polling every frame, some audio triggers should use Bevy events:

```rust
/// Sent when a notification should produce a sound.
#[derive(Event)]
pub struct PlayNotificationSound {
    pub notification_type: NotificationType,
    pub priority: AudioPriority,
}

/// Sent when a stinger should play.
#[derive(Event)]
pub struct PlayStinger {
    pub stinger_type: StingerType,
    pub priority: StingerPriority,
}

/// Sent when a tool action produces a sound.
#[derive(Event)]
pub struct PlayToolSound {
    pub tool_sound: ToolSoundId,
    pub position: Option<Vec3>,  // Some for spatial, None for UI
    pub volume_override: Option<f32>,
}

/// Sent when a building completes or is destroyed (for one-shot SFX).
#[derive(Event)]
pub struct BuildingAudioEvent {
    pub event_type: BuildingAudioEventType,
    pub position: Vec3,
    pub size: BuildingSize,
}

pub enum BuildingAudioEventType {
    ConstructionStart,
    ConstructionComplete,
    Demolished,
    CaughtFire,
    FireExtinguished,
    Upgraded,
    Abandoned,
}
```

### 12.5 Integration with Existing Systems

The audio crate reads from simulation resources but never writes to them. This
unidirectional data flow prevents audio from affecting gameplay:

```
simulation (reads/writes game state)
    ↓ (read-only)
audio (reads game state, writes to Kira audio engine)
    ↓
Kira audio thread (produces audio output)
    ↓
Hardware (speakers/headphones)
```

**No simulation dependencies on audio:** If the audio crate is removed or disabled,
the game runs identically. Audio is purely presentational.

---

## 13. Performance Considerations

### 13.1 CPU Cost Breakdown

At maximum load (dense city, disaster active, all zones present, storm weather):

| System | Frequency | Est. Cost | Notes |
|---|---|---|---|
| `update_audio_listener` | 60 Hz | 0.01 ms | Read camera, write listener |
| `update_audio_lod_tier` | 60 Hz | 0.01 ms | Compare distance to thresholds |
| `cull_distant_emitters` | 60 Hz | 0.1 ms | Iterate active emitters, distance check |
| `update_emitter_volumes` | 60 Hz | 0.15 ms | Distance attenuation for ~32 emitters |
| `update_chunk_audio_profiles` | 10 Hz | 0.3 ms | Iterate 1024 chunks, sample grid |
| `update_zone_ambience` | 10 Hz | 0.1 ms | Set volumes on ~8 active chunks |
| `sample_city_state` | 10 Hz | 0.02 ms | Read stats, compute mood |
| `update_music_stem_volumes` | 60 Hz | 0.05 ms | 7 stem volume tweens |
| `evaluate_section_transition` | 10 Hz | 0.02 ms | State machine evaluation |
| `update_weather_audio` | 60 Hz | 0.05 ms | Rain/wind/thunder layers |
| `process_notification_queue` | 60 Hz | 0.02 ms | Usually no-op |
| `apply_ducking` | 60 Hz | 0.02 ms | Envelope updates |
| `fire_audio_sync` | 60 Hz | 0.05 ms | Check fire grid near listener |
| **Total (worst case)** | | **~0.9 ms** | Well under 2ms budget |

### 13.2 Memory Cost Breakdown

| Category | Items | Est. Memory | Notes |
|---|---|---|---|
| Sound asset handles | ~120 | ~2 KB | Just Bevy handles, not audio data |
| Loaded WAV data | ~80 files | ~10 MB | Short SFX, fully decoded |
| Streaming OGG buffers | ~10 active | ~5 MB | 512KB buffer per stream |
| Music stems (current piece) | 7 stems | ~15 MB | Streamed, ~2MB buffer each |
| ChunkAudioProfiles | 1024 chunks | ~80 KB | Simple structs |
| Emitter components | ~50 active | ~5 KB | Small components |
| Kira internal state | - | ~2 MB | Mixer, effects, scheduling |
| Procedural generator state | ~4 generators | ~1 KB | Minimal state per generator |
| **Total** | | **~32 MB** | Under 50 MB budget |

### 13.3 Optimization Strategies

#### 13.3.1 Spatial Hashing for Emitter Culling

Rather than checking distance for every possible emitter, use the existing
`SpatialIndex` pattern from the simulation crate:

```rust
/// Spatial hash for audio emitters. Only emitters in nearby cells are evaluated.
/// Cells are 128x128 world units (8 grid cells).
fn cull_emitters_spatial(
    listener: &AudioListener,
    spatial_index: &SpatialAudioIndex,
) -> Vec<Entity> {
    let cell_x = (listener.position.x / 128.0) as i32;
    let cell_z = (listener.position.z / 128.0) as i32;
    let radius_cells = (listener.audible_radius / 128.0).ceil() as i32;

    let mut active = Vec::new();
    for dy in -radius_cells..=radius_cells {
        for dx in -radius_cells..=radius_cells {
            if let Some(emitters) = spatial_index.get(cell_x + dx, cell_z + dy) {
                active.extend(emitters);
            }
        }
    }
    active
}
```

#### 13.3.2 Throttled Occlusion Raycasting

Occlusion raycasting (Section 2.5.1) is the most expensive per-emitter operation.
Throttle it to avoid frame spikes:

```rust
/// Only recompute occlusion for a subset of emitters each frame.
/// With 32 max emitters and 4 per frame, full refresh takes 8 frames (133ms).
const OCCLUSION_UPDATES_PER_FRAME: usize = 4;

fn update_occlusion_throttled(
    mut emitters: Query<&mut AudioEmitter>,
    grid: Res<WorldGrid>,
    listener: Res<AudioListener>,
    mut update_index: Local<usize>,
) {
    let emitter_list: Vec<_> = emitters.iter().collect();
    let count = emitter_list.len();
    if count == 0 { return; }

    for i in 0..OCCLUSION_UPDATES_PER_FRAME.min(count) {
        let idx = (*update_index + i) % count;
        // Recompute occlusion for emitter at idx
        // ...
    }
    *update_index = (*update_index + OCCLUSION_UPDATES_PER_FRAME) % count.max(1);
}
```

#### 13.3.3 Voice Stealing

When the maximum simultaneous source limit is reached, the system must decide which
sounds to stop. Priority-based voice stealing ensures important sounds always play:

```rust
fn steal_voice(
    active_sounds: &mut Vec<ActiveSound>,
    new_sound: &SoundRequest,
    max_voices: usize,
) -> Option<usize> {
    if active_sounds.len() < max_voices {
        return None; // No stealing needed
    }

    // Find the lowest-priority, quietest sound to steal from
    let mut best_victim = None;
    let mut best_score = f32::MAX;

    for (i, sound) in active_sounds.iter().enumerate() {
        // Never steal from critical sounds
        if sound.priority == AudioPriority::Critical { continue; }

        // Score: lower = more stealable
        // Priority weight + volume weight
        let score = sound.priority as u32 as f32 * 100.0
                  + sound.current_volume * 50.0
                  + (if sound.is_looping { 0.0 } else { -20.0 }); // prefer stealing one-shots near end

        if score < best_score {
            best_score = score;
            best_victim = Some(i);
        }
    }

    best_victim
}
```

#### 13.3.4 Audio System Disable for Low-End Hardware

For players on very low-end hardware, provide a "Minimal Audio" option that:
- Disables all spatial audio (only stereo music + UI sounds)
- Disables procedural audio generation
- Reduces max simultaneous sources to 8
- Disables occlusion raycasting
- Estimated CPU savings: ~0.7ms per frame

### 13.4 Testing and Profiling

#### 13.4.1 Audio-Specific Profiling

```rust
/// Diagnostic resource for audio system performance monitoring.
#[derive(Resource, Default)]
pub struct AudioDiagnostics {
    pub active_emitters: u32,
    pub active_kira_sounds: u32,
    pub audio_system_ms: f32,
    pub kira_thread_load: f32,  // 0.0-1.0
    pub memory_used_mb: f32,
    pub current_lod_tier: AudioLodTier,
    pub music_section: String,
    pub active_stems: Vec<String>,
    pub weather_layers: Vec<String>,
}
```

Display this in a debug overlay (toggled in dev builds) alongside existing simulation
diagnostics.

#### 13.4.2 Audio Stress Test

Create a test scenario that maximizes audio load:
- 256x256 grid fully built out (all zone types present)
- Active disaster (tornado at city center)
- Storm weather
- Maximum camera zoom-in (Tier 0 LOD, 32 emitters active)
- All notification types firing simultaneously
- Music at full complexity (all stems active)

The system must maintain <2ms total under this load. If it exceeds budget, use the
degradation path: reduce max emitters, increase throttle interval, disable occlusion.

### 13.5 Latency Requirements

| Sound Category | Max Acceptable Latency | Notes |
|---|---|---|
| UI click/hover | <10ms | Must feel instantaneous |
| Tool placement | <20ms | Tight feedback loop with visual |
| Notification | <50ms | Prompt but not instant |
| Spatial emitter start | <100ms | Can be slightly delayed |
| Music transition | <500ms (quantized) | Quantized to beat/bar, not frame-accurate |
| Ambient layer change | <1000ms | Gradual crossfade, not instant |

**Kira buffer size:** Set to 256 samples at 44.1kHz = ~5.8ms latency. This provides
good responsiveness for UI sounds while keeping CPU usage reasonable. If UI latency
is problematic, can reduce to 128 samples (~2.9ms) at the cost of higher CPU usage
on the audio thread.

---

## Summary

Sound design for Megacity is a deep, multi-layered system that touches every aspect
of the game. The key architectural decisions are:

1. **Use Kira (via bevy_kira_audio)** for its superior crossfading, spatial audio,
   hierarchical mixing, and clock-quantized transitions.

2. **Hierarchical audio bus structure** with independent volume controls for music,
   ambience, SFX, and UI.

3. **Audio LOD system** that scales from individual sound sources at street level to
   aggregate zone ambience at city-wide zoom, matching the existing visual LOD tiers.

4. **Adaptive music with vertical layering and horizontal re-sequencing** that responds
   to population, happiness, time of day, season, and crises.

5. **Procedural audio for continuous sounds** (traffic, rain, wind, crowd) to achieve
   infinite variation without memory-heavy sample loops.

6. **Priority-based notification system** with cooldowns, ducking, and mandatory visual
   counterparts for accessibility.

7. **Zone-based spatial audio** that lets experienced players identify zones by sound
   alone, using the existing grid and chunk infrastructure.

8. **Strict performance budget** of <2ms per frame, achieved through spatial hashing,
   throttled occlusion, voice stealing, and the natural separation of Kira's audio
   thread from Bevy's main thread.

The ultimate goal: a player who closes their eyes should be able to tell you the time
of day, the season, the weather, whether they are in a residential or commercial district,
whether traffic is congested, and whether the city is thriving or struggling -- all from
sound alone.
