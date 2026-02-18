# Urban Planning and Zoning: Deep Feature Research

## Table of Contents

1. [Zoning Paradigms: Real-World Systems as Game Mechanics](#1-zoning-paradigms)
   1.1 [Euclidean (Hierarchical) Zoning](#11-euclidean-hierarchical-zoning)
   1.2 [Form-Based Codes](#12-form-based-codes)
   1.3 [Japanese 12-Zone System](#13-japanese-12-zone-system)
   1.4 [Houston: No Zoning](#14-houston-no-zoning)
   1.5 [Current Megacity Implementation vs. Alternatives](#15-current-megacity-implementation-vs-alternatives)
2. [Building Types, Dimensions, and Densities](#2-building-types-dimensions-and-densities)
   2.1 [Residential Zone Tiers](#21-residential-zone-tiers)
   2.2 [Commercial Zone Tiers](#22-commercial-zone-tiers)
   2.3 [Industrial Zone Tiers](#23-industrial-zone-tiers)
   2.4 [Office Zone Tiers](#24-office-zone-tiers)
   2.5 [Mixed-Use Buildings](#25-mixed-use-buildings)
   2.6 [Grid Cell Mapping and FAR Values](#26-grid-cell-mapping-and-far-values)
3. [Building Growth and Development](#3-building-growth-and-development)
   3.1 [Cities: Skylines Approach (Service-Driven Leveling)](#31-cities-skylines-approach)
   3.2 [SimCity 4 Approach (Density as Separate Axis)](#32-simcity-4-approach)
   3.3 [Market-Driven Growth (Developer ROI Model)](#33-market-driven-growth)
   3.4 [Building Variety Generation](#34-building-variety-generation)
   3.5 [Demolition and Replacement Logic](#35-demolition-and-replacement-logic)
   3.6 [Proposed Hybrid System for Megacity](#36-proposed-hybrid-system-for-megacity)
4. [Street Patterns and Urban Morphology](#4-street-patterns-and-urban-morphology)
   4.1 [Grid Pattern (Manhattan)](#41-grid-pattern-manhattan)
   4.2 [Radial Pattern (Paris)](#42-radial-pattern-paris)
   4.3 [Organic Pattern (Medieval Cities)](#43-organic-pattern-medieval-cities)
   4.4 [Cul-de-Sac Suburbs](#44-cul-de-sac-suburbs)
   4.5 [Barcelona Superblocks](#45-barcelona-superblocks)
   4.6 [Street Dimensions and Right-of-Way](#46-street-dimensions-and-right-of-way)
   4.7 [Street Pattern Detection Algorithm](#47-street-pattern-detection-algorithm)
5. [Neighborhood Design and Walkability](#5-neighborhood-design-and-walkability)
   5.1 [15-Minute City Scoring Algorithm](#51-15-minute-city-scoring-algorithm)
   5.2 [Walk Score Methodology (Simplified)](#52-walk-score-methodology-simplified)
   5.3 [Perry's Neighborhood Unit](#53-perrys-neighborhood-unit)
   5.4 [Transit-Oriented Development (TOD)](#54-transit-oriented-development-tod)
   5.5 [Neighborhood Quality Index](#55-neighborhood-quality-index)
6. [Advanced Zoning Mechanics](#6-advanced-zoning-mechanics)
   6.1 [NIMBY/YIMBY as Game Mechanic](#61-nimbyyimby-as-game-mechanic)
   6.2 [Eminent Domain Events](#62-eminent-domain-events)
   6.3 [Historic Preservation](#63-historic-preservation)
   6.4 [Urban Growth Boundaries](#64-urban-growth-boundaries)
   6.5 [Inclusionary Zoning](#65-inclusionary-zoning)
   6.6 [Parking Minimums (Donald Shoup)](#66-parking-minimums-donald-shoup)
   6.7 [Floor Area Ratio (FAR) Bonuses and Transfers](#67-floor-area-ratio-far-bonuses-and-transfers)
7. [Implementation Roadmap](#7-implementation-roadmap)
   7.1 [Phase 1: Enhanced Zone Types](#71-phase-1-enhanced-zone-types)
   7.2 [Phase 2: Form-Based Overlay](#72-phase-2-form-based-overlay)
   7.3 [Phase 3: Advanced Mechanics](#73-phase-3-advanced-mechanics)
   7.4 [Phase 4: Neighborhood Scoring](#74-phase-4-neighborhood-scoring)

---

## 1. Zoning Paradigms

### 1.1 Euclidean (Hierarchical) Zoning

Euclidean zoning, named after Village of Euclid v. Ambler Realty (1926), is the dominant zoning model in the United States and the default paradigm for virtually every city builder game ever made. It divides land into discrete, mutually exclusive use categories arranged in a hierarchy.

**Core Principle: Cumulative Hierarchy**

The key insight -- and the one most city builders get wrong -- is that traditional Euclidean zoning is _cumulative_. Higher (more restrictive) zones exclude lower uses, but lower zones permit higher uses:

```
Hierarchy (most restrictive to least):
  R-1  Single-Family Residential (only houses)
  R-2  Two-Family Residential (R-1 uses + duplexes)
  R-3  Multi-Family Residential (R-1, R-2 + apartments)
  C-1  Neighborhood Commercial (all residential + small shops)
  C-2  General Commercial (C-1 + offices, restaurants)
  M-1  Light Manufacturing (C-2 + clean industry, warehouses)
  M-2  Heavy Manufacturing (anything goes)
```

In practice, this means a C-1 zone can contain houses. An M-1 zone can contain offices. Only R-1 is truly single-use. Most city builders (including Cities: Skylines) use _exclusive_ zoning instead, where residential zones only grow residential buildings. This is simpler but produces unrealistic monotone neighborhoods.

**What Euclidean Zoning Regulates (the "Dimensional Standards")**

Each zone type specifies a detailed set of dimensional controls:

| Control | R-1 Typical | R-4 Typical | C-2 Typical | M-1 Typical |
|---------|-------------|-------------|-------------|-------------|
| Min. lot size | 5,000-10,000 sq ft | 2,500-5,000 sq ft | None | None |
| Min. lot width | 50-75 ft | 25-40 ft | None | None |
| Max. building height | 35 ft (2.5 stories) | 65-120 ft | 45-85 ft | 45 ft |
| Max. lot coverage | 30-40% | 60-80% | 80-100% | 80-100% |
| Front setback | 20-30 ft | 10-20 ft | 0-10 ft | 10-25 ft |
| Side setback | 5-15 ft | 0-5 ft | 0 ft | 5-10 ft |
| Rear setback | 15-25 ft | 10-15 ft | 0-10 ft | 10-20 ft |
| Max. FAR | 0.3-0.5 | 2.0-6.0 | 2.0-5.0 | 1.0-3.0 |
| Parking spaces/unit | 1-2 | 0.5-1.5 | 1 per 300 sq ft | 1 per 500 sq ft |

**Game Implementation Considerations**

In a 256x256 grid with 16.0 cell size, one grid cell represents 16m x 16m = 256 sq meters = 2,755 sq ft. This is roughly half a typical single-family lot. Implementation options:

- **1 cell = 1 small lot**: R-1 buildings occupy 1-2 cells (realistic for small houses)
- **2x2 cells = 1 standard lot**: 32m x 32m = 1,024 sq m = 11,023 sq ft (standard US suburban lot)
- **Current approach**: 1 cell = 1 building of any size, capacity scaled by level

The current Megacity `ZoneType` enum has 7 variants (None, ResidentialLow, ResidentialHigh, CommercialLow, CommercialHigh, Industrial, Office). A Euclidean system would expand this to at minimum 8-10 distinct zone types with hierarchical permission rules.

**Proposed Expanded Enum:**

```rust
pub enum ZoneType {
    None,
    // Residential (cumulative upward)
    SingleFamily,       // R-1: detached houses only
    LowDensityRes,      // R-2: R-1 + duplexes, townhouses
    MedDensityRes,      // R-3: R-2 + small apartments (3-6 stories)
    HighDensityRes,     // R-4: R-3 + towers (7-40+ stories)
    // Commercial
    NeighborhoodComm,   // C-1: small shops, corner stores, cafes
    GeneralComm,        // C-2: offices, restaurants, retail
    // Industrial
    LightIndustrial,    // M-1: warehouses, clean manufacturing
    HeavyIndustrial,    // M-2: factories, processing plants
    // Special
    Office,             // O: office-only district
    MixedUse,           // MU: residential above commercial
}
```

### 1.2 Form-Based Codes

Form-based codes (FBCs), developed in the New Urbanist movement and formalized in the SmartCode by Andres Duany and Elizabeth Plater-Zyberk, represent a fundamental paradigm shift: **regulate the form of buildings, not their use**. Where Euclidean zoning asks "What happens inside this building?", form-based codes ask "What does this building look like from the street?"

**The Transect: T1 through T6**

Form-based codes organize the built environment along a rural-to-urban transect:

```
T1 - Natural Zone
  No development. Preserved wilderness, wetlands, parks.
  Equivalent: CellType::Grass / CellType::Water with no zone
  Building coverage: 0%

T2 - Rural Zone
  Farms, very sparse settlement, country roads.
  Lot size: 10+ acres (not applicable in a city game)
  FAR: <0.1

T3 - Sub-Urban
  Detached houses, large yards, curving streets.
  Lot coverage: 20-40%. Height: 1-2.5 stories. Setbacks: generous.
  FAR: 0.3-0.6
  Building types: single-family detached, accessory dwelling units
  Density: 2-8 units/acre, ~5-20 residents/acre

T4 - General Urban
  Rowhouses, small apartments, mixed-use main streets.
  Lot coverage: 40-70%. Height: 2-4 stories. Setbacks: small/zero.
  FAR: 1.0-2.5
  Building types: townhouses, 3-flats, live/work, small retail
  Density: 12-30 units/acre, ~30-75 residents/acre

T5 - Urban Center
  Mid-rise apartments, offices, commercial buildings.
  Lot coverage: 60-90%. Height: 3-8 stories. Setbacks: zero (build-to line).
  FAR: 2.0-6.0
  Building types: apartment buildings, mixed-use, office mid-rise
  Density: 30-100 units/acre, ~75-250 residents/acre

T6 - Urban Core
  High-rise towers, skyscrapers, maximum intensity.
  Lot coverage: 80-100%. Height: 8-40+ stories. No max (often).
  FAR: 5.0-15.0+
  Building types: residential towers, office skyscrapers, hotels
  Density: 100-500+ units/acre, ~250-1250+ residents/acre
```

**Why This Is Better for Games**

1. **Eliminates the "industrial next to residential" problem** -- you do not need separate use-based zones because form controls are per-building
2. **Gradual transitions** -- T3 naturally buffers between T2 and T4, creating realistic suburbs-to-downtown gradients
3. **Mixed-use by default** -- a T4 zone allows shops on the ground floor with apartments above without needing a special zone type
4. **Simpler player mental model** -- "how dense/tall should buildings be here?" instead of "should this be residential or commercial?"

**Implementation as Overlay System**

Rather than replacing `ZoneType`, form-based codes work best as an _overlay_ on top of use-based zoning:

```rust
/// Form-based transect overlay. Controls physical form independent of use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TransectZone {
    #[default]
    None,       // No form control (legacy behavior)
    T1Natural,  // No building allowed
    T2Rural,    // 1 story, 0.1 FAR, full setbacks
    T3Suburban, // 1-2.5 stories, 0.5 FAR, standard setbacks
    T4Urban,    // 2-4 stories, 2.0 FAR, small setbacks
    T5Center,   // 3-8 stories, 5.0 FAR, zero setbacks
    T6Core,     // 8-40 stories, 12.0 FAR, zero setbacks
}

impl TransectZone {
    pub fn max_stories(self) -> u8 {
        match self {
            Self::None => 40, // unconstrained
            Self::T1Natural => 0,
            Self::T2Rural => 1,
            Self::T3Suburban => 3,
            Self::T4Urban => 4,
            Self::T5Center => 8,
            Self::T6Core => 40,
        }
    }

    pub fn max_far(self) -> f32 {
        match self {
            Self::None => 15.0,
            Self::T1Natural => 0.0,
            Self::T2Rural => 0.1,
            Self::T3Suburban => 0.6,
            Self::T4Urban => 2.5,
            Self::T5Center => 6.0,
            Self::T6Core => 15.0,
        }
    }

    pub fn max_lot_coverage(self) -> f32 {
        match self {
            Self::None => 1.0,
            Self::T1Natural => 0.0,
            Self::T2Rural => 0.2,
            Self::T3Suburban => 0.4,
            Self::T4Urban => 0.7,
            Self::T5Center => 0.9,
            Self::T6Core => 1.0,
        }
    }

    /// Minimum front setback in grid cells (1 cell = 16m)
    pub fn front_setback_cells(self) -> u8 {
        match self {
            Self::None | Self::T5Center | Self::T6Core | Self::T1Natural => 0,
            Self::T2Rural => 2,  // ~32m (generous rural setback)
            Self::T3Suburban => 1, // ~16m (standard suburban)
            Self::T4Urban => 0,    // build-to-line
        }
    }
}
```

### 1.3 Japanese 12-Zone System

The Japanese zoning system (yoto-chiiki) is arguably the most elegant zoning system in the world for game design purposes. Established under the City Planning Act of 1968 and refined in 1992, it uses 12 (originally 8, now 13 as of 2018) zone categories. The critical insight is that **Japanese zones specify what is EXCLUDED, not what is INCLUDED**. This creates a permissive, cumulative hierarchy that naturally produces the mixed-use neighborhoods Japan is famous for.

**The 12 (+1) Japanese Zones**

```
Category I: Exclusively Residential (most restrictive)
  1. Class 1 Exclusively Low-Rise Residential
     - ONLY: detached houses, duplexes (max 2 stories, 10m height)
     - Small shops/offices up to 50 sq m allowed within houses
     - FAR: 0.5-1.0, Building coverage: 30-60%
     - Equivalent: R-1 single-family, but allows home businesses

  2. Class 2 Exclusively Low-Rise Residential
     - Same as (1) + convenience stores up to 150 sq m
     - A single 7-Eleven in a neighborhood of houses
     - This produces the charming Japanese residential street feel

Category II: Primarily Residential
  3. Class 1 Mid/High-Rise Residential
     - Allows apartments (no height limit based on zone alone)
     - Shops up to 500 sq m, hospitals, universities
     - FAR: 1.0-2.0, Building coverage: 30-60%

  4. Class 2 Mid/High-Rise Residential
     - Same as (3) + shops/offices up to 1,500 sq m
     - Karaoke bars, small offices, 2-story retail
     - This is where Japanese commercial streets emerge

Category III: Residential-Compatible
  5. Class 1 Residential
     - Hotels, shops up to 3,000 sq m, small factories up to 50 sq m
     - FAR: 1.0-3.0

  6. Class 2 Residential
     - Same as (5) but shops/offices up to 10,000 sq m
     - Karaoke, bowling alleys, auto repair
     - This is a typical Japanese mixed neighborhood

Category IV: Commercial
  7. Neighborhood Commercial
     - No limits on commercial size
     - Small factories up to 50 sq m
     - Theaters, nightclubs allowed
     - FAR: 1.5-4.0

  8. Commercial
     - Everything allowed EXCEPT heavy/dangerous industry
     - Theaters, department stores, offices, apartments
     - FAR: 2.0-8.0 (13.0 in designated districts)

Category V: Industrial
  9. Quasi-Industrial
     - Everything allowed including light factories
     - Hazardous factories excluded
     - FAR: 2.0-4.0

  10. Industrial
     - Everything EXCEPT certain residential types
      (schools, hospitals, hotels prohibited)
     - FAR: 1.0-4.0

  11. Exclusively Industrial
     - Factories only, residences prohibited
     - The only zone that truly excludes residential
     - FAR: 1.0-4.0

  12. Urbanization Control Area
     - Limits development to prevent sprawl

  13. Pastoral Residential (added 2018)
     - Farmland + low-density housing
```

**Why Japanese Zoning Is Brilliant for Games**

1. **Exclusion-based logic is simpler to compute**: Instead of checking "is building type X in the allowed list for zone Y," check "is building type X in the exclusion list for zone Y." The exclusion list is always shorter.

2. **National uniformity**: Unlike the US where every municipality has different rules, Japan has ONE national zoning system. The mayor cannot arbitrarily ban things. This maps perfectly to a game where you want consistent, predictable rules.

3. **Mixed-use emerges naturally**: A Zone 6 (Class 2 Residential) neighborhood will organically grow restaurants, offices, and small shops alongside apartments. No need for the player to micromanage "this block residential, this block commercial."

4. **Height is controlled by separate overlay**: Japan uses a shadow-angle regulation (slant-line restriction) instead of absolute height limits. Buildings must not cast shadows beyond a calculated line, which naturally creates the stepped-back profile of Japanese apartment buildings. This is a separate system from zoning.

**Implementation as Exclusion Sets:**

```rust
pub enum JapaneseZone {
    ExclusiveLowRes1,   // Zones 1-2: almost everything excluded
    ExclusiveLowRes2,
    MidHighRes1,        // Zones 3-4: large retail/industry excluded
    MidHighRes2,
    Residential1,       // Zones 5-6: heavy industry excluded
    Residential2,
    NeighborhoodComm,   // Zone 7: heavy industry excluded
    Commercial,         // Zone 8: only heavy industry excluded
    QuasiIndustrial,    // Zone 9: only hazardous industry excluded
    Industrial,         // Zone 10: schools/hospitals excluded
    ExclusiveIndustrial,// Zone 11: residences excluded
}

impl JapaneseZone {
    /// Returns true if the given building type is PROHIBITED in this zone.
    pub fn excludes(&self, building: &BuildingType) -> bool {
        match self {
            Self::ExclusiveLowRes1 => {
                // Only single-family, duplexes, and home offices allowed
                !matches!(building,
                    BuildingType::SingleFamily |
                    BuildingType::Duplex |
                    BuildingType::HomeOffice
                )
            }
            Self::Commercial => {
                // Only heavy/hazardous industry excluded
                matches!(building,
                    BuildingType::HeavyFactory |
                    BuildingType::ChemicalPlant |
                    BuildingType::Refinery
                )
            }
            Self::ExclusiveIndustrial => {
                // Only zone that excludes residences
                building.is_residential() || building.is_school() || building.is_hospital()
            }
            // ... other zones
        }
    }

    /// Maximum floor area for shops/offices allowed in this zone (sq meters).
    /// Returns None for no limit.
    pub fn max_commercial_floor_area(&self) -> Option<f32> {
        match self {
            Self::ExclusiveLowRes1 => Some(50.0),
            Self::ExclusiveLowRes2 => Some(150.0),
            Self::MidHighRes1 => Some(500.0),
            Self::MidHighRes2 => Some(1500.0),
            Self::Residential1 => Some(3000.0),
            Self::Residential2 => Some(10000.0),
            _ => None, // no limit
        }
    }
}
```

### 1.4 Houston: No Zoning

Houston, Texas is the largest city in the United States (and the developed world) with no formal zoning ordinance. The city voted down zoning referenda in 1948, 1962, and 1993. With a metro population of about 7 million, Houston demonstrates that cities can function -- and even thrive economically -- without traditional zoning. But "no zoning" does not mean "no regulation."

**What Houston Actually Has Instead of Zoning:**

1. **Deed restrictions** (private covenants): Neighborhoods create legally binding agreements among property owners. These are _more_ restrictive than zoning in many cases -- a deed restriction can specify paint colors, fence heights, lawn maintenance requirements. They are enforced by homeowners' associations, not the city.

2. **Minimum lot sizes**: The city mandates 5,000 sq ft minimum lots within Loop 610, 6,500 sq ft outside. This effectively prevents dense development in suburban areas more effectively than any zoning.

3. **Setback requirements**: Standard setbacks still apply (25 ft front for residential, varies for commercial).

4. **Parking minimums**: Until recently, Houston required massive parking minimums (1 space per 200 sq ft of retail, 1.333 per apartment unit). This consumed enormous amounts of land.

5. **Buffer requirements**: Heavy industry must provide buffers when adjacent to residential (typically 200-500 ft).

**Game Design Implications: The "No Zoning" Mode**

A no-zoning mode creates fascinating emergent gameplay:

```rust
pub struct NoZoningConfig {
    /// Buildings spawn based purely on market demand + land value
    pub market_driven: bool,
    /// Private deed restrictions (player creates, neighborhoods vote on)
    pub deed_restrictions_enabled: bool,
    /// Minimum lot sizes still enforced
    pub min_lot_size_cells: u8,  // default 2 (2x2 = 32x32m)
    /// Buffer distance required between incompatible uses
    pub industrial_buffer_cells: u8, // default 4 (64m)
    /// Parking requirements consume land
    pub parking_minimum: ParkingRequirement,
}
```

In a no-zoning mode:
- The player does NOT paint zones on the map
- Buildings grow organically based on: land value, road access, nearby amenities, market demand
- A house might appear next to a factory (unless deed restrictions prevent it)
- Commercial strips form naturally along major roads (this is exactly what happens in Houston)
- Land values self-sort: expensive land gets offices/retail, cheap land gets industry
- Players can create deed restriction districts to control specific neighborhoods
- The result is chaotic but functional, mimicking Houston's actual urban form

**What Houston Shows About City Builder Design:**

The key lesson is that **land value is the real zoning mechanism**. When land is expensive (near amenities, good transit, waterfront), developers build the highest-value use: offices, luxury apartments, upscale retail. When land is cheap (far from downtown, near noise/pollution), developers build warehouses, auto repair shops, trailer parks. This self-sorting by price is more realistic than most city builders model.

### 1.5 Current Megacity Implementation vs. Alternatives

**Current State (from codebase analysis):**

The current `ZoneType` enum in `crates/simulation/src/grid.rs` provides 7 variants:
- `None` - unzoned
- `ResidentialLow` - max level 3 (capacity: 10 / 30 / 80)
- `ResidentialHigh` - max level 5 (capacity: 50 / 200 / 500 / 1000 / 2000)
- `CommercialLow` - max level 3 (capacity: 8 / 25 / 60)
- `CommercialHigh` - max level 5 (capacity: 30 / 100 / 300 / 600 / 1200)
- `Industrial` - max level 5 (capacity: 20 / 60 / 150 / 300 / 600)
- `Office` - max level 5 (capacity: 30 / 100 / 300 / 700 / 1500)

This is a standard Euclidean exclusive zoning system with density baked into the zone type (low vs. high). The zone demand system in `zones.rs` uses saturation ratios (built/zoned) to drive RCI demand.

**Comparison Matrix:**

| Feature | Current Megacity | Euclidean (Full) | Form-Based | Japanese | No-Zoning |
|---------|-----------------|------------------|------------|----------|-----------|
| Zone count | 7 | 10-12 | 6 (T1-T6) | 12-13 | 0 |
| Mixed-use | No | No (by default) | Yes (native) | Yes (native) | Yes (emergent) |
| Density control | Zone type (L/H) | FAR + height | Transect tier | FAR + shadow | Land value |
| Use separation | Strict | Cumulative | None | Exclusion-based | Market-driven |
| Player complexity | Low | Medium | Low-Medium | Medium | Low (less control) |
| Realism | Low | Medium | High | High | Medium-High |
| Implementation effort | Done | Medium | Medium | High | Medium |

**Recommended Approach: Layered Hybrid**

Rather than choosing one paradigm, the most effective game design layers multiple systems:

1. **Base layer**: Simplified Euclidean zones (current system, expanded to 10 types)
2. **Overlay layer**: Form-based transect (controls height/density independent of use)
3. **District layer**: Policy overrides per district (already partially implemented via `DistrictPolicies`)
4. **Market layer**: Land-value-driven growth within zone constraints

This allows players to zone broadly (residential here, commercial there) while optionally using transect overlays to control density (T3 suburbs near the edge, T6 core downtown) and district policies to fine-tune behavior (ban heavy industry in the Historic district).

---

## 2. Building Types, Dimensions, and Densities

This section provides exhaustive dimensional data for every building type that can appear in each zone, mapped to the Megacity grid system (256x256, CELL_SIZE=16.0m, so 1 cell = 16m x 16m = 256 sq m = 2,755 sq ft).

### 2.1 Residential Zone Tiers

#### R-1: Single-Family Detached (Low-Density Residential)

| Property | Real-World | Grid Mapping |
|----------|-----------|--------------|
| Lot width | 40-80 ft (12-24m) | 1 cell wide |
| Lot depth | 100-150 ft (30-46m) | 2-3 cells deep |
| Footprint | 1,200-2,500 sq ft | 1 cell (scaled) |
| Stories | 1-2 | Level 1-2 |
| Height | 15-30 ft (5-10m) | Render: 5-10m |
| Lot coverage | 25-40% | N/A (1 cell = 1 building) |
| FAR | 0.3-0.5 | Capacity: 4-8 residents |
| Residents | 2-5 per unit (1 unit per lot) | **4 per cell** at level 1 |
| Parking | 1-2 car garage, driveway | Included (no separate parking cell) |
| Density | 3-8 units/acre | ~6 units/acre at 1 cell each |
| Visual examples | Cape Cod, Ranch, Colonial, Split-level | 4-6 distinct models |
| Yard/setback | 20 ft front, 5 ft sides, 15 ft rear | Green buffer rendered |

**Detailed building pool for R-1:**
```
Level 1 (newly built):
  - Small Ranch House: 1 cell, 1 story, capacity 4, render 6m tall
  - Cape Cod: 1 cell, 1.5 stories, capacity 4, render 7m tall
  - Bungalow: 1 cell, 1 story, capacity 3, render 5m tall

Level 2 (upgraded / new construction):
  - Two-Story Colonial: 1 cell, 2 stories, capacity 6, render 9m tall
  - Split-Level: 1 cell, 1.5 stories, capacity 5, render 8m tall
  - Large Ranch with Addition: 1 cell, 1 story, capacity 6, render 6m tall

Level 3 (premium):
  - McMansion: 1-2 cells, 2 stories, capacity 8, render 10m tall
  - Modern House: 1 cell, 2 stories, capacity 6, render 9m tall
  - Victorian: 1 cell, 2.5 stories, capacity 8, render 11m tall
```

#### R-2: Townhouses and Duplexes (Low-Medium Density)

| Property | Real-World | Grid Mapping |
|----------|-----------|--------------|
| Lot width | 16-30 ft per unit (5-9m) | 1 cell per 2-3 units |
| Lot depth | 80-120 ft (24-37m) | 2 cells deep |
| Footprint | 800-1,500 sq ft per unit | 1 cell = 2-3 units |
| Stories | 2-3 | Level 1-3 |
| Height | 25-40 ft (8-12m) | Render: 8-12m |
| Lot coverage | 50-70% | N/A |
| FAR | 0.8-1.5 | Capacity: 6-18 residents |
| Residents | 2-4 per unit, 2-6 units per lot | **8 per cell** at level 1 |
| Parking | Rear garage or street | Included |
| Density | 8-20 units/acre | ~15 units/acre |
| Visual examples | Row houses, brownstones, duplexes, triplexes | 4-6 distinct models |

**Detailed building pool for R-2:**
```
Level 1:
  - Duplex: 1 cell, 2 stories, capacity 8 (2 units x 4), render 9m
  - Row House Block (3-unit): 1 cell, 2 stories, capacity 10, render 9m

Level 2:
  - Brownstone Row (4-unit): 1 cell, 3 stories, capacity 16, render 12m
  - Triplex: 1 cell, 2.5 stories, capacity 12, render 10m

Level 3:
  - Premium Townhouse Row: 1 cell, 3 stories, capacity 20, render 12m
  - Mixed Townhouse/Flat: 1 cell, 3 stories, capacity 24, render 13m
```

#### R-3: Mid-Rise Apartments (Medium Density)

| Property | Real-World | Grid Mapping |
|----------|-----------|--------------|
| Lot width | 50-150 ft (15-46m) | 1-2 cells wide |
| Lot depth | 100-200 ft (30-60m) | 2-3 cells deep |
| Building footprint | 3,000-15,000 sq ft | 1-2 cells |
| Stories | 3-6 | Level 1-3 |
| Height | 35-75 ft (11-23m) | Render: 11-23m |
| Lot coverage | 50-80% | N/A |
| FAR | 1.5-3.5 | Capacity: 30-120 residents |
| Units | 6-40 per building | ~10-30 per cell |
| Residents | 15-100 per building | **30 per cell** at level 1 |
| Parking | Surface lot or tuck-under | 0.5-1 cell for parking at ground |
| Density | 20-60 units/acre | ~40 units/acre |
| Visual examples | Walk-up apartments, garden apartments, courtyard buildings | 5-8 distinct models |

**Detailed building pool for R-3:**
```
Level 1:
  - 3-Story Walk-Up: 1 cell, 3 stories, capacity 30 (10 units), render 12m
  - Garden Apartment: 1 cell, 2 stories, capacity 24 (12 units), render 9m

Level 2:
  - 4-Story Apartment Building: 1 cell, 4 stories, capacity 60 (20 units), render 15m
  - Courtyard Apartment: 2x2 cells, 3 stories, capacity 80, render 12m

Level 3:
  - 6-Story Mid-Rise: 1 cell, 6 stories, capacity 120 (40 units), render 22m
  - U-Shaped Apartment Complex: 2x2 cells, 5 stories, capacity 160, render 18m
```

#### R-4: High-Rise Residential (High Density)

| Property | Real-World | Grid Mapping |
|----------|-----------|--------------|
| Lot width | 100-300 ft (30-90m) | 2-4 cells wide |
| Lot depth | 100-300 ft (30-90m) | 2-4 cells deep |
| Building footprint | 5,000-40,000 sq ft | 2x2 to 4x4 cells |
| Stories | 7-40+ | Level 1-5 |
| Height | 75-500+ ft (23-150m+) | Render: 23-150m |
| Lot coverage | 30-60% (tower + podium) | 2x2 cells, tower on 1 cell |
| FAR | 3.0-15.0 | Capacity: 100-2000 residents |
| Units | 50-500+ per building | ~50-500 per building |
| Residents | 100-1,250+ per building | **100 per cell** at level 1 |
| Parking | Underground garage (1-3 levels below grade) | Implied, no extra cell |
| Density | 60-500+ units/acre | ~120 units/acre |
| Visual examples | Point towers, slab towers, tower-on-podium, supertall | 8-12 distinct models |

**Detailed building pool for R-4:**
```
Level 1 (7-12 stories):
  - Slab Apartment Block: 2x1 cells, 10 stories, capacity 100, render 35m
  - Point Tower (small): 1 cell, 12 stories, capacity 80, render 42m

Level 2 (13-20 stories):
  - Mid-Rise Tower: 1 cell, 18 stories, capacity 200, render 60m
  - Tower-on-Podium: 2x2 cells, 15 stories, capacity 300, render 52m

Level 3 (21-30 stories):
  - Residential Tower: 1 cell, 25 stories, capacity 500, render 85m
  - Twin Tower Complex: 2x2 cells, 22 stories, capacity 600, render 75m

Level 4 (31-40 stories):
  - Luxury Tower: 1 cell, 35 stories, capacity 400 (larger units), render 120m
  - High-Rise Complex: 3x3 cells, 30 stories, capacity 1000, render 105m

Level 5 (40+ stories):
  - Supertall Residential: 1 cell, 40+ stories, capacity 500, render 140m
  - Mega Complex: 4x4 cells, 35 stories, capacity 2000, render 120m
```

### 2.2 Commercial Zone Tiers

#### C-1: Neighborhood Commercial (Low-Density Commercial)

| Property | Real-World | Grid Mapping |
|----------|-----------|--------------|
| Lot width | 20-60 ft (6-18m) | 1 cell |
| Lot depth | 60-120 ft (18-37m) | 1-2 cells |
| Footprint | 1,000-5,000 sq ft | 1 cell |
| Stories | 1-2 | Level 1-2 |
| Height | 15-30 ft (5-10m) | Render: 5-10m |
| FAR | 0.5-1.5 | Capacity: 8-30 workers |
| Workers | 2-20 per establishment | **8 per cell** at level 1 |
| Parking | Front/side surface lot | Included |
| Visual examples | Corner store, strip mall, diner, gas station, laundromat | 6-8 models |

**Detailed building pool for C-1:**
```
Level 1:
  - Corner Store: 1 cell, 1 story, capacity 8 workers, render 5m
  - Small Strip Mall: 2x1 cells, 1 story, capacity 15, render 5m
  - Gas Station: 1 cell, 1 story, capacity 5, render 4m
  - Diner/Cafe: 1 cell, 1 story, capacity 8, render 5m

Level 2:
  - Large Strip Mall: 3x1 cells, 1 story, capacity 25, render 6m
  - 2-Story Retail: 1 cell, 2 stories, capacity 20, render 9m
  - Neighborhood Market: 1 cell, 1 story, capacity 15, render 6m

Level 3:
  - Premium Retail: 1 cell, 2 stories, capacity 40, render 10m
  - Restaurant Row: 2x1 cells, 2 stories, capacity 50, render 10m
```

#### C-2: Main Street / General Commercial

| Property | Real-World | Grid Mapping |
|----------|-----------|--------------|
| Lot width | 25-100 ft (8-30m) | 1-2 cells |
| Lot depth | 100-200 ft (30-60m) | 2-3 cells |
| Footprint | 2,500-20,000 sq ft | 1-2 cells |
| Stories | 2-5 | Level 1-3 |
| Height | 25-65 ft (8-20m) | Render: 8-20m |
| FAR | 1.0-3.0 | Capacity: 25-150 workers |
| Workers | 10-100+ per building | **30 per cell** at level 1 |
| Parking | Street + rear lot or small garage | Included |
| Visual examples | Downtown shops, restaurants, small offices, banks, pharmacies | 8-10 models |

#### C-3: Big Box / Commercial District

| Property | Real-World | Grid Mapping |
|----------|-----------|--------------|
| Lot width | 200-600 ft (60-180m) | 4-8 cells |
| Lot depth | 200-600 ft (60-180m) | 4-8 cells |
| Building footprint | 40,000-200,000 sq ft | 4x4 to 6x6 cells |
| Stories | 1-2 | Level 1-2 |
| Height | 20-40 ft (6-12m) | Render: 6-12m |
| FAR | 0.2-0.5 (mostly parking!) | Capacity: 50-200 workers |
| Workers | 30-300 per store | **60 per building** at level 1 |
| Parking | Massive surface lot (50-80% of land area) | 8-16 cells parking lot |
| Visual examples | Walmart, Target, Costco, Home Depot, car dealerships | 4-6 models |
| Notes | These are land-value destroyers -- huge impervious footprints, car-dependent |

#### C-4: Office Tower / High-Density Commercial

| Property | Real-World | Grid Mapping |
|----------|-----------|--------------|
| Lot width | 100-300 ft (30-90m) | 2-4 cells |
| Lot depth | 100-300 ft (30-90m) | 2-4 cells |
| Building footprint | 10,000-60,000 sq ft | 2x2 to 3x3 cells |
| Stories | 5-80+ | Level 1-5 |
| Height | 65-1,000+ ft (20-300m+) | Render: 20-300m |
| FAR | 5.0-15.0+ | Capacity: 100-3000 workers |
| Workers/floor | 50-200 per floor | Scaled by level |
| Workers total | 500-10,000+ per building | **100 per cell** at level 1 |
| Parking | Underground garage (2-5 levels) | Implied |
| Visual examples | Glass office tower, postmodern skyscraper, Art Deco tower | 10-15 models |

**Detailed building pool for C-4:**
```
Level 1 (5-10 stories):
  - Small Office Building: 1 cell, 8 stories, capacity 100, render 28m
  - Mid-Rise Office: 2x2 cells, 6 stories, capacity 200, render 22m

Level 2 (11-20 stories):
  - Office Tower: 1 cell, 16 stories, capacity 300, render 55m
  - Corporate Campus: 3x3 cells, 10 stories, capacity 500, render 35m

Level 3 (21-35 stories):
  - Glass Curtain Wall Tower: 2x2 cells, 28 stories, capacity 800, render 95m
  - Modern Office Tower: 1 cell, 30 stories, capacity 600, render 100m

Level 4 (36-55 stories):
  - Class A Office Tower: 2x2 cells, 45 stories, capacity 1500, render 155m
  - Postmodern Skyscraper: 1 cell, 50 stories, capacity 1000, render 170m

Level 5 (55+ stories):
  - Supertall Office Tower: 2x2 cells, 60+ stories, capacity 3000, render 210m
  - Headquarters Tower: 3x3 cells, 55 stories, capacity 2500, render 190m
```

### 2.3 Industrial Zone Tiers

#### I-1: Light Industrial / Flex Space

| Property | Real-World | Grid Mapping |
|----------|-----------|--------------|
| Lot size | 10,000-50,000 sq ft | 2x2 to 3x3 cells |
| Building footprint | 5,000-30,000 sq ft | 2x2 cells typical |
| Stories | 1-2 | Level 1-2 |
| Height | 20-40 ft (6-12m) | Render: 6-12m |
| FAR | 0.3-0.8 | Capacity: 15-60 workers |
| Workers | 5-50 per facility | **20 per cell** at level 1 |
| Noise radius | 2-3 cells | `noise_radius: 3` |
| Pollution | Low (5-15 units) | Render: no smoke |
| Visual examples | Warehouse, light manufacturing, tech shop, fulfillment center | 4-6 models |

#### I-2: Medium Industrial

| Property | Real-World | Grid Mapping |
|----------|-----------|--------------|
| Lot size | 50,000-200,000 sq ft | 3x3 to 5x5 cells |
| Building footprint | 20,000-100,000 sq ft | 3x3 cells typical |
| Stories | 1-3 | Level 1-3 |
| Height | 30-60 ft (10-18m) | Render: 10-18m |
| FAR | 0.5-1.5 | Capacity: 40-200 workers |
| Workers | 20-200 per facility | **60 per cell** at level 1 |
| Noise radius | 4-6 cells | `noise_radius: 5` |
| Pollution | Medium (15-40 units) | Render: light smoke stacks |
| Visual examples | Factory, food processing, printing plant, auto assembly | 5-8 models |

#### I-3: Heavy Industrial

| Property | Real-World | Grid Mapping |
|----------|-----------|--------------|
| Lot size | 200,000-2,000,000+ sq ft | 5x5 to 10x10 cells |
| Building footprint | 50,000-500,000 sq ft | 4x4 to 8x8 cells |
| Stories | 1-4 | Level 1-5 |
| Height | 40-120 ft (12-37m) + stacks up to 300 ft | Render: 12-37m + stacks |
| FAR | 0.3-1.0 | Capacity: 100-600 workers |
| Workers | 50-5,000 per facility | **150 per building** at level 1 |
| Noise radius | 6-10 cells | `noise_radius: 8` |
| Pollution | High (40-100 units) | Render: heavy smoke, colored effluent |
| Buffer required | 200-500 ft (4-8 cells) from residential | Enforced by zoning system |
| Visual examples | Steel mill, oil refinery, chemical plant, power plant | 4-6 models |

**Detailed industrial building pool:**
```
I-1 Level 1: Warehouse, capacity 20, 1 cell, render 8m
I-1 Level 2: Fulfillment Center, capacity 60, 2x2 cells, render 12m
I-1 Level 3: Tech Manufacturing, capacity 150, 2x2 cells, render 14m

I-2 Level 1: Small Factory, capacity 40, 2x2 cells, render 12m
I-2 Level 2: Processing Plant, capacity 120, 3x3 cells, render 16m
I-2 Level 3: Assembly Plant, capacity 250, 3x3 cells, render 18m

I-3 Level 1: Heavy Factory, capacity 100, 3x3 cells, render 20m
I-3 Level 2: Steel Works, capacity 200, 4x4 cells, render 25m
I-3 Level 3: Integrated Mill, capacity 350, 5x5 cells, render 30m
I-3 Level 4: Refinery Complex, capacity 450, 6x6 cells, render 35m
I-3 Level 5: Mega Factory, capacity 600, 8x8 cells, render 40m
```

### 2.4 Office Zone Tiers

The Office zone occupies a middle ground between commercial and industrial. In many real cities, office districts are a separate zoning category because they generate different traffic patterns (peak morning/evening vs. all-day for retail) and different urban forms (corporate campuses vs. street-level retail).

#### O-1: Professional Office / Office Park

| Property | Real-World | Grid Mapping |
|----------|-----------|--------------|
| Lot size | 5,000-50,000 sq ft | 1-3 cells |
| Building footprint | 3,000-20,000 sq ft | 1-2 cells |
| Stories | 1-4 | Level 1-2 |
| Height | 15-55 ft (5-17m) | Render: 5-17m |
| FAR | 0.3-1.5 | Capacity: 20-100 workers |
| Workers | 10-80 per building | **30 per cell** at level 1 |
| Parking | Surface lot (typical suburban) | 1-2 cells parking |
| Visual examples | Doctor's office, law firm, small tech company, dental office | 4-6 models |

#### O-2: Office Tower District

Uses the same parameters as C-4 (Office Tower / High-Density Commercial) above. In practice, high-density office and high-density commercial zones share the same building forms -- the distinction is in the use mix (office-only vs. retail + office).

| Property | Real-World | Grid Mapping |
|----------|-----------|--------------|
| Stories | 10-80+ | Level 1-5 |
| Height | 120-1,000+ ft (37-300m+) | Render: 37-300m |
| FAR | 5.0-25.0 | Capacity: 100-1500 workers/cell |
| Workers total | 500-15,000 per building | Scaled by level |

### 2.5 Mixed-Use Buildings

Mixed-use buildings are the most realistic urban building type and the hardest to implement in traditional city builders. They require a building to serve _multiple_ zone types simultaneously.

**Mixed-Use Type Matrix:**

| Type | Ground Floor | Upper Floors | Stories | FAR | Workers | Residents |
|------|-------------|--------------|---------|-----|---------|-----------|
| MU-1 Shophouse | Retail | 1-2 apartments | 2-3 | 1.0-2.0 | 5 | 8 |
| MU-2 Main Street | Retail/Restaurant | Apartments | 3-5 | 2.0-4.0 | 15 | 30 |
| MU-3 Urban Mixed | Retail + Office | Apartments | 5-12 | 3.0-6.0 | 40 | 80 |
| MU-4 Tower Mixed | Retail podium | Office mid + Res top | 15-40+ | 6.0-15.0 | 200 | 400 |

**Implementation:**

```rust
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct MixedUseBuilding {
    /// Commercial/retail component (ground floor + lower floors)
    pub commercial_capacity: u32,
    pub commercial_occupants: u32,
    /// Office component (mid floors, if any)
    pub office_capacity: u32,
    pub office_occupants: u32,
    /// Residential component (upper floors)
    pub residential_capacity: u32,
    pub residential_occupants: u32,
    /// Total stories
    pub stories: u8,
    pub grid_x: usize,
    pub grid_y: usize,
    pub level: u8,
}

impl MixedUseBuilding {
    /// Returns (commercial_jobs, office_jobs, residential_capacity)
    pub fn capacity_for_level(level: u8) -> (u32, u32, u32) {
        match level {
            1 => (5, 0, 8),    // MU-1 Shophouse
            2 => (15, 0, 30),  // MU-2 Main Street
            3 => (20, 20, 80), // MU-3 Urban Mixed
            4 => (40, 80, 200),// MU-4 Tower Mixed (small)
            5 => (80, 200, 400), // MU-4 Tower Mixed (large)
            _ => (0, 0, 0),
        }
    }
}
```

### 2.6 Grid Cell Mapping and FAR Values

**Floor Area Ratio (FAR) Explained**

FAR = Total floor area / Lot area. A FAR of 2.0 on a 10,000 sq ft lot means 20,000 sq ft of building floor area. This could be a 2-story building covering the entire lot, a 4-story building covering half the lot, or a 10-story building covering 20% of the lot.

**Megacity Grid Cell FAR Calculations**

One grid cell = 16m x 16m = 256 sq m = 2,755 sq ft of lot area.

| FAR | Total Floor Area | Stories at 100% coverage | Stories at 50% coverage | Typical Zone |
|-----|-----------------|-------------------------|------------------------|-------------|
| 0.3 | 77 sq m | 0.3 (small house) | 0.6 | R-1 |
| 0.5 | 128 sq m | 0.5 (medium house) | 1.0 | R-1 |
| 1.0 | 256 sq m | 1.0 | 2.0 | R-2, C-1 |
| 1.5 | 384 sq m | 1.5 | 3.0 | R-2, C-1 |
| 2.0 | 512 sq m | 2.0 | 4.0 | R-3, C-2 |
| 3.0 | 768 sq m | 3.0 | 6.0 | R-3, C-2 |
| 5.0 | 1,280 sq m | 5.0 | 10.0 | R-4, C-3, O-2 |
| 8.0 | 2,048 sq m | 8.0 | 16.0 | C-4, Downtown |
| 12.0 | 3,072 sq m | 12.0 | 24.0 | C-4, CBD |
| 15.0 | 3,840 sq m | 15.0 | 30.0 | T6 Core only |

**Density Conversions (per grid cell = 256 sq m)**

| Density Metric | R-1 | R-2 | R-3 | R-4 L1 | R-4 L5 |
|---------------|-----|-----|-----|--------|--------|
| Units/cell | 1 | 2-3 | 6-12 | 15-30 | 60-120 |
| Residents/cell | 3-5 | 6-10 | 15-30 | 50-100 | 200-500 |
| Units/acre (15.4 cells/acre) | 15 | 38 | 123 | 385 | 1,540 |
| Residents/acre | 62 | 123 | 385 | 1,230 | 6,160 |
| FAR | 0.3-0.5 | 0.8-1.5 | 1.5-3.0 | 3.0-6.0 | 8.0-15.0 |

Note: 1 acre = 4,047 sq m. At 256 sq m per cell, 1 acre = 15.8 cells. For a city block of 10x10 cells (100 cells = 25,600 sq m = 6.3 acres):

| Zone | Buildings in 10x10 block | Total residents | Total workers | People/acre |
|------|------------------------|-----------------|---------------|-------------|
| R-1 L1 | ~80 (20% roads/green) | 320 | 0 | 51 |
| R-2 L2 | ~80 | 1,280 | 0 | 203 |
| R-3 L2 | ~64 (some 2x2) | 3,840 | 0 | 610 |
| R-4 L3 | ~40 (mostly 2x2) | 20,000 | 0 | 3,175 |
| C-2 L2 | ~64 | 0 | 6,400 | N/A |
| I-2 L2 | ~25 (3x3 buildings) | 0 | 3,000 | N/A |
| MU-3 L3 | ~64 | 5,120 | 2,560 | 1,222 |

**FAR Implementation in Building Spawner:**

```rust
/// Compute maximum building level allowed by FAR constraints.
pub fn max_level_for_far(zone: ZoneType, transect: TransectZone) -> u8 {
    let max_far = transect.max_far();
    let zone_max = zone.max_level();

    // Find highest level whose implied FAR does not exceed the transect limit
    for level in (1..=zone_max).rev() {
        let capacity = Building::capacity_for_level(zone, level);
        // Rough FAR estimate: capacity * 20 sq m per person / 256 sq m per cell
        let implied_far = (capacity as f32 * 20.0) / 256.0;
        if implied_far <= max_far {
            return level;
        }
    }
    1 // minimum level
}
```

---

## 3. Building Growth and Development

This section analyzes how buildings appear, grow, upgrade, and get demolished across different city builder paradigms, then proposes a hybrid system for Megacity.

### 3.1 Cities: Skylines Approach (Service-Driven Leveling)

Cities: Skylines uses a building level system driven primarily by service coverage:

**How It Works:**
1. Player paints zone on ground
2. Building spawns at level 1 after road connection and demand exist
3. Building levels up (1 to 5) based on accumulated service satisfaction:
   - Land value (derived from parks, services, distance from undesirable things)
   - Crime rate (low crime = faster leveling)
   - Pollution level (low pollution = faster leveling)
   - Education availability
   - Health coverage
   - Fire protection
   - Leisure (parks, plazas)
4. Higher-level buildings have more residents/workers and look better
5. Buildings can also _de-level_ if services are removed

**Scoring Formula (reverse-engineered from C:S behavior):**
```
level_score = (land_value * 0.3)
            + (1.0 - crime_rate) * 0.15
            + (1.0 - pollution) * 0.15
            + (education_coverage) * 0.15
            + (health_coverage) * 0.10
            + (fire_coverage) * 0.05
            + (leisure_coverage) * 0.10

Level 1: score >= 0.0
Level 2: score >= 0.3
Level 3: score >= 0.5
Level 4: score >= 0.7
Level 5: score >= 0.85
```

**Strengths:**
- Simple to understand: provide services, buildings improve
- Creates incentive to invest in infrastructure
- Visual feedback: neighborhoods visibly improve as services arrive

**Weaknesses:**
- **Creates extreme uniformity**: Every level-5 residential zone looks the same because the same services produce the same outcome everywhere
- **Density is locked to zone type**: Low-density zone always produces houses, high-density always produces towers, regardless of location/demand
- **No market dynamics**: Buildings level up for free -- no concept of construction cost, developer investment, or economic viability
- **Binary outcome**: A building is either level N or level N+1, no gradual improvement
- **Unrealistic renovation**: A 1-story house magically transforms into a 3-story apartment (the building changes shape in place)

### 3.2 SimCity 4 Approach (Density as Separate Axis)

SimCity 4 (2003) introduced the most nuanced building growth system in any commercial city builder, separating density from wealth:

**The 3x3 Matrix:**

```
               Wealth Level
              $     $$    $$$
Density  Low  |  R$  | R$$  | R$$$  |   (Houses, small apartments)
         Med  |  R$  | R$$  | R$$$  |   (Medium apartments)
         High |  R$  | R$$  | R$$$  |   (Towers)
```

This creates 9 distinct residential building categories (and equivalent for commercial/industrial), each with its own building pool of 5-15 distinct models. A poor neighborhood does not automatically become rich by adding services; it becomes a well-serviced poor neighborhood.

**How SimCity 4 Controls Density:**

1. **Zone density is set by the player** (light/medium/dense), not earned
2. **Wealth (stage/quality) is driven by:**
   - Desirability (services, parks, transit, education, health)
   - Land value
   - Mayor rating
   - Tax rates for that wealth tier
   - Cap relief (demand from outside the city)
3. **Buildings grow to fill available demand:**
   - A dense R$$$ zone only gets buildings if there is demand for wealthy high-density housing
   - If there is no demand, the zone sits vacant
   - Demand is driven by jobs (commercial/industrial employment attracts residential demand)

**Key Insight: The "Stage" System**

SimCity 4 buildings have 8 stages of visual growth within each density/wealth combination:

```
Stage 1: Empty lot (just zoned)
Stage 2: Construction begins (foundation)
Stage 3: Small building appears
Stage 4-5: Building grows taller/wider
Stage 6-7: Building reaches near-maximum
Stage 8: Maximum building for this density/wealth
```

Each stage has its own 3D model. A Stage 8 R$$$ Dense building is a luxury high-rise tower. A Stage 8 R$ Light building is a run-down single-family house. The stage is determined by demand and desirability, not just service coverage.

**Why This Is Superior:**

- **Diverse neighborhoods**: Rich and poor areas look fundamentally different even at the same density
- **Organic growth**: Buildings gradually get taller as demand increases
- **Vacancies**: Zones can sit empty if there is no demand, creating realistic vacant lots
- **Wealth segregation**: Naturally emerges from land value differences, not player painting

### 3.3 Market-Driven Growth (Developer ROI Model)

A market-driven system models real estate development as an investment decision. This is the most realistic approach but also the most computationally expensive.

**The Developer Decision Model:**

Every tick (or every N ticks), the system evaluates each buildable lot:

```
For each vacant lot with zoning and road access:
  1. Estimate land value = f(location, services, amenities, accessibility)
  2. For each allowable building type B:
     a. construction_cost = base_cost(B) + site_prep
     b. expected_revenue = rent(B) * occupancy_rate * time_horizon
     c. roi = (expected_revenue - construction_cost) / construction_cost
     d. risk_factor = f(demand_volatility, competition, city_stability)
     e. adjusted_roi = roi * (1.0 - risk_factor)
  3. Select building B with highest adjusted_roi
  4. If adjusted_roi > minimum_threshold (typically 8-15%):
     Spawn building B
  5. Otherwise:
     Lot remains vacant
```

**For existing buildings, the REDEVELOPMENT calculation:**

```
For each lot with an existing building:
  1. current_building_value = depreciated_construction_cost + improvements
  2. current_revenue = actual_rent * current_occupancy
  3. land_value = assessed_land_value (from LandValueGrid)
  4. If land_value > current_building_value + demolition_cost:
     // The land is "underbuilt" -- could support a more valuable building
     5. best_new_building = evaluate all allowable types (same as new construction)
     6. new_building_value = best_new_building.revenue - best_new_building.cost
     7. redevelopment_profit = new_building_value - (current_building_value + demolition_cost)
     8. If redevelopment_profit > threshold:
        Demolish current building
        Start construction of new building
```

**Simplified Implementation for Megacity:**

```rust
pub struct DeveloperDecision {
    pub lot_x: usize,
    pub lot_y: usize,
    pub best_building: Option<BuildingTemplate>,
    pub expected_roi: f32,
}

pub fn evaluate_lot(
    grid: &WorldGrid,
    land_value: &LandValueGrid,
    demand: &ZoneDemand,
    x: usize,
    y: usize,
) -> DeveloperDecision {
    let cell = grid.get(x, y);
    let lv = land_value.get(x, y) as f32;

    // Land value determines maximum viable building cost
    let max_investment = lv * 100.0; // $100 per land value point

    // Demand determines occupancy expectation
    let expected_occupancy = demand.demand_for(cell.zone).clamp(0.3, 0.95);

    // Find best building type for this zone and land value
    let mut best: Option<BuildingTemplate> = None;
    let mut best_roi = 0.0f32;

    for template in BuildingTemplate::for_zone(cell.zone) {
        let cost = template.construction_cost;
        if cost > max_investment {
            continue; // Too expensive for this land value
        }

        let monthly_revenue = template.capacity as f32 * expected_occupancy * template.rent_per_unit;
        let annual_revenue = monthly_revenue * 12.0;
        let roi = (annual_revenue * 10.0 - cost) / cost; // 10-year ROI

        if roi > best_roi {
            best_roi = roi;
            best = Some(template.clone());
        }
    }

    DeveloperDecision {
        lot_x: x,
        lot_y: y,
        best_building: best,
        expected_roi: best_roi,
    }
}
```

### 3.4 Building Variety Generation

One of the most common criticisms of city builders is visual monotony -- neighborhoods where every building looks identical. Real cities have tremendous building variety even within the same zone type. This section describes algorithms for generating that variety.

**Weighted Random Pool System:**

Each zone type + level combination has a pool of building templates with weights:

```rust
pub struct BuildingPool {
    pub entries: Vec<BuildingPoolEntry>,
    pub total_weight: f32,
}

pub struct BuildingPoolEntry {
    pub template: BuildingTemplate,
    pub weight: f32,          // Base weight (higher = more common)
    pub min_land_value: u8,   // Minimum land value to spawn
    pub max_land_value: u8,   // Maximum land value to spawn
    pub requires_corner: bool, // Only spawns at road intersections
    pub max_per_district: u8,  // Prevents duplicate landmarks
    pub era: BuildingEra,     // Historic period for visual style
}

pub enum BuildingEra {
    PreWar,     // Before 1940: ornate, brick, detailed
    MidCentury, // 1940-1970: simple, functional, boxy
    Late20th,   // 1970-2000: postmodern, varied materials
    Modern,     // 2000+: glass, steel, minimalist
}

impl BuildingPool {
    /// Select a building from the pool using weighted random selection,
    /// filtered by contextual constraints.
    pub fn select(
        &self,
        rng: &mut impl Rng,
        land_value: u8,
        is_corner: bool,
        district_counts: &HashMap<usize, u8>,
    ) -> Option<&BuildingTemplate> {
        let eligible: Vec<&BuildingPoolEntry> = self.entries.iter()
            .filter(|e| {
                land_value >= e.min_land_value
                && land_value <= e.max_land_value
                && (!e.requires_corner || is_corner)
                && district_counts.get(&e.template.id).copied().unwrap_or(0) < e.max_per_district
            })
            .collect();

        if eligible.is_empty() {
            return None;
        }

        let total: f32 = eligible.iter().map(|e| e.weight).sum();
        let mut roll = rng.gen::<f32>() * total;

        for entry in &eligible {
            roll -= entry.weight;
            if roll <= 0.0 {
                return Some(&entry.template);
            }
        }

        eligible.last().map(|e| &e.template)
    }
}
```

**Example Building Pool for R-3 Level 2 (Mid-Rise Apartments):**

```
Pool: R3_L2 (14 entries)
  Weight 30: "Standard 4-Story Apartment" (common, everywhere)
  Weight 25: "Brick Walk-Up" (common, lower land value)
  Weight 20: "Modern 4-Story" (common, higher land value)
  Weight 15: "L-Shaped Apartment" (moderately common)
  Weight 10: "Courtyard Apartment" (less common, corner lots)
  Weight 8:  "Art Deco Apartment" (less common, PreWar era)
  Weight 6:  "Mid-Century Modern" (uncommon, MidCentury era)
  Weight 5:  "Garden Apartment" (uncommon, requires greenspace)
  Weight 4:  "Converted Warehouse" (rare, near industrial)
  Weight 3:  "Boutique Apartment" (rare, high land value only)
  Weight 2:  "Historic Renovation" (rare, historic district only)
  Weight 2:  "Student Housing" (rare, near university)
  Weight 1:  "Artist Loft" (very rare, high culture district)
  Weight 1:  "Co-Housing" (very rare, high education area)
```

**Color/Material Variation System:**

Beyond selecting different building models, each building should have randomized materials and colors to prevent the "copy-paste" look:

```rust
pub struct BuildingAppearance {
    /// Index into the material palette for this building type
    pub material_variant: u8,    // 0-5 (brick, stucco, concrete, glass, wood, stone)
    /// Color tint applied to the base material
    pub color_tint: [f32; 3],    // RGB multiplier, randomized within range
    /// Roof style variation
    pub roof_variant: u8,        // 0-3 (flat, gabled, hip, mansard)
    /// Window pattern variation
    pub window_variant: u8,      // 0-3 (regular grid, offset, large pane, mixed)
    /// Age/weathering (affects texture)
    pub weathering: f32,         // 0.0 (new) to 1.0 (old), increases over time
}

impl BuildingAppearance {
    pub fn random_for_zone(zone: ZoneType, land_value: u8, rng: &mut impl Rng) -> Self {
        let base_color = match zone {
            ZoneType::ResidentialLow => [0.9, 0.85, 0.75],  // warm earth tones
            ZoneType::ResidentialHigh => [0.7, 0.75, 0.8],  // cool grays/blues
            ZoneType::CommercialLow => [0.85, 0.8, 0.7],    // warm commercial
            ZoneType::CommercialHigh => [0.6, 0.65, 0.75],  // corporate blue/gray
            ZoneType::Industrial => [0.5, 0.5, 0.5],         // gray/neutral
            ZoneType::Office => [0.55, 0.6, 0.7],            // blue-gray glass
            _ => [0.8, 0.8, 0.8],
        };

        // Add random variation (+/- 10%)
        let tint = [
            base_color[0] * (0.9 + rng.gen::<f32>() * 0.2),
            base_color[1] * (0.9 + rng.gen::<f32>() * 0.2),
            base_color[2] * (0.9 + rng.gen::<f32>() * 0.2),
        ];

        Self {
            material_variant: rng.gen_range(0..6),
            color_tint: tint,
            roof_variant: if land_value > 150 { rng.gen_range(0..4) } else { rng.gen_range(0..2) },
            window_variant: rng.gen_range(0..4),
            weathering: 0.0,
        }
    }
}
```

### 3.5 Demolition and Replacement Logic

Real-world redevelopment follows a simple economic rule: **demolish when the land is worth more than the building**. This is the fundamental mechanism behind gentrification, urban renewal, and natural building turnover.

**The Redevelopment Decision:**

```
redevelopment_trigger = land_value > (building_value + demolition_cost)

Where:
  land_value = LandValueGrid.get(x, y) * land_value_multiplier
  building_value = construction_cost * (1 - depreciation_rate * age)
  demolition_cost = construction_cost * 0.1 to 0.3 (10-30% of original cost)
```

**Depreciation Rates:**
```
Residential: 1-2% per game-year (50-100 year lifespan)
Commercial:  2-3% per game-year (33-50 year lifespan)
Industrial:  3-5% per game-year (20-33 year lifespan)
Office:      1.5-2.5% per game-year (40-67 year lifespan)
```

**Detailed Replacement Algorithm:**

```rust
pub fn evaluate_redevelopment(
    building: &Building,
    grid: &WorldGrid,
    land_value: &LandValueGrid,
    demand: &ZoneDemand,
    game_age: u32, // building age in game-days
) -> RedevelopmentDecision {
    let cell = grid.get(building.grid_x, building.grid_y);
    let lv = land_value.get(building.grid_x, building.grid_y) as f32;

    // Current building value (depreciates over time)
    let base_value = building_base_value(building.zone_type, building.level);
    let age_years = game_age as f32 / 365.0;
    let depreciation = depreciation_rate(building.zone_type);
    let current_value = base_value * (1.0 - depreciation * age_years).max(0.1);

    // Demolition cost
    let demo_cost = base_value * 0.15;

    // Land value for best possible use
    let potential_land_value = lv * 1000.0; // scale to dollar equivalent

    // Can we build something better?
    let demand_factor = demand.demand_for(cell.zone);
    let max_viable_level = max_level_for_land_value(cell.zone, lv);
    let potential_building_value = building_base_value(cell.zone, max_viable_level);

    let profit = potential_building_value - current_value - demo_cost;
    let roi = profit / (potential_building_value + demo_cost);

    if roi > 0.15 && demand_factor > 0.3 && max_viable_level > building.level {
        RedevelopmentDecision::Demolish {
            new_level: max_viable_level,
            expected_profit: profit,
        }
    } else if current_value < demo_cost * 0.5 {
        // Building is basically worthless -- abandon it
        RedevelopmentDecision::Abandon
    } else {
        RedevelopmentDecision::Keep
    }
}

pub enum RedevelopmentDecision {
    Keep,
    Demolish { new_level: u8, expected_profit: f32 },
    Abandon,
}
```

**Abandonment Mechanics:**

When buildings are not demolished but become undesirable (high crime, no services, low demand), they should transition through degradation states:

```
Occupied -> Declining (occupancy dropping) -> Vacant -> Abandoned -> Demolished (auto-clear)

Timeline:
  Declining: 6-12 game-months of low demand + poor services
  Vacant: occupancy hits 0, building remains for 3-6 game-months
  Abandoned: visual degradation (boarded windows, graffiti), attracts crime +5
  Auto-demolish: after 12 game-months of abandonment, building despawns
```

The current codebase has an `abandonment.rs` file (untracked) which likely implements some of this. The key design consideration is that abandonment should be a slow, visible process that gives the player time to intervene.

### 3.6 Proposed Hybrid System for Megacity

Combining the best elements of each approach into a system that works with Megacity's existing architecture:

**Layer 1: Zone-Based Spawning (Current System, Enhanced)**

Keep the current `building_spawner` system but enhance it:
- Select building from weighted pool (not just default capacity)
- Check FAR limits from transect overlay before spawning
- Consider land value when selecting building template
- Add construction time (already implemented via `UnderConstruction`)

**Layer 2: Service-Driven Quality (Simplified C:S)**

Instead of C:S-style leveling, use a continuous "building quality" score:
```rust
pub struct BuildingQuality {
    pub score: f32,        // 0.0 to 100.0
    pub trend: f32,        // -1.0 to +1.0 (improving/declining per tick)
    pub last_upgrade: u32, // game-day of last level change
}
```

Quality score is computed from:
- Service coverage (health, education, police, fire, parks) = 40% weight
- Land value = 20% weight
- Pollution + noise (negative) = 15% weight
- Crime (negative) = 10% weight
- Infrastructure (power, water, road condition) = 15% weight

When quality stays above threshold for N days, building upgrades.
When quality stays below threshold, building downgrades.

**Layer 3: Market-Driven Density Changes**

Instead of the player micromanaging density, density naturally increases when:
1. Land value exceeds a threshold for the current density
2. Demand for that zone type is high (>0.5)
3. Adjacent buildings are already at higher density

This means a ResidentialLow zone can _organically densify_ into ResidentialHigh if the market supports it, mimicking real-world gentle density increases (homeowner adds a granny flat, house replaced by duplex, duplex replaced by 4-plex, etc.).

```rust
pub fn should_densify(
    building: &Building,
    land_value: u8,
    demand: f32,
    neighbor_levels: &[u8],
) -> bool {
    let lv_threshold = match building.zone_type {
        ZoneType::ResidentialLow => 120,  // need high land value to densify
        ZoneType::CommercialLow => 100,
        _ => 255, // already high density or non-densifiable
    };

    let avg_neighbor_level = if neighbor_levels.is_empty() {
        0.0
    } else {
        neighbor_levels.iter().map(|&l| l as f32).sum::<f32>() / neighbor_levels.len() as f32
    };

    land_value > lv_threshold
        && demand > 0.5
        && building.level >= building.zone_type.max_level()
        && avg_neighbor_level > building.level as f32 + 0.5
}
```

**Layer 4: Demolition/Replacement (Automated)**

Run the redevelopment evaluation every 100 ticks on a random subset of buildings (e.g., 1% per evaluation). This keeps the computational cost low while ensuring a natural turnover rate.

---

## 4. Street Patterns and Urban Morphology

Street patterns are the DNA of a city. They determine block shapes, building orientations, traffic flow, walkability, and the fundamental character of neighborhoods. This section analyzes five major street pattern archetypes and their implementation as game mechanics.

### 4.1 Grid Pattern (Manhattan)

**Real-World Characteristics:**
- Originated in ancient Greek colonies (Hippodamus of Miletus, 5th century BC)
- Most famously implemented in Manhattan's 1811 Commissioners' Plan
- Standard American city layout (Philadelphia, Chicago, Portland, Salt Lake City)
- Produces rectangular blocks with predictable dimensions

**Manhattan Block Dimensions (the reference standard):**
```
Standard Manhattan block:
  East-West: 900 ft (274m) = ~17 cells
  North-South: 250 ft (76m) = ~5 cells
  Including streets: 20 x 6 cell blocks

Typical American grid block:
  300 ft x 300 ft (91m x 91m) = ~6x6 cells
  Including streets: 7 x 7 cells (road on each side)

Portland's "short blocks":
  200 ft x 200 ft (61m x 61m) = ~4x4 cells
  Including streets: 5 x 5 cells
  (Widely considered ideal for walkability)
```

**Why Grids Work for Games:**
- Trivially maps to the 256x256 grid
- Maximizes usable lot area (rectangular lots are space-efficient)
- Provides maximum connectivity (many route options, resilient to road closures)
- Simple A* pathfinding with low branching factor
- Players naturally build grids because it is easiest

**Grid Pattern Metrics:**
```
Intersection density: 1 per block (highest of any pattern)
Block perimeter: 4 * block_side
Connectivity index: typically 1.4-1.6 (ratio of links to nodes)
Route directness: ~1.2-1.4 (Manhattan distance / Euclidean distance)
Dead ends: 0 (no cul-de-sacs)
```

**Implementation (Auto-Grid Tool):**

```rust
/// Generate a grid street pattern within a rectangular area.
pub fn generate_grid_pattern(
    grid: &mut WorldGrid,
    start_x: usize,
    start_y: usize,
    width: usize,
    height: usize,
    block_size: usize,    // cells between roads (4-8 typical)
    road_type: RoadType,
) {
    // Horizontal roads
    let mut y = start_y;
    while y < start_y + height && y < grid.height {
        for x in start_x..(start_x + width).min(grid.width) {
            let cell = grid.get_mut(x, y);
            cell.cell_type = CellType::Road;
            cell.road_type = road_type;
        }
        y += block_size + 1; // block_size cells of lots + 1 cell of road
    }

    // Vertical roads
    let mut x = start_x;
    while x < start_x + width && x < grid.width {
        for y in start_y..(start_y + height).min(grid.height) {
            let cell = grid.get_mut(x, y);
            cell.cell_type = CellType::Road;
            cell.road_type = road_type;
        }
        x += block_size + 1;
    }
}
```

### 4.2 Radial Pattern (Paris)

**Real-World Characteristics:**
- Paris (Haussmann's renovation, 1853-1870): Broad boulevards radiating from central plazas
- Washington DC (L'Enfant Plan, 1791): Diagonal avenues over a grid
- Moscow, Vienna, Barcelona (partial): Ring roads with radial spokes
- Creates dramatic vistas and monumental axes
- Produces irregular, triangular blocks at intersections (wedge-shaped buildings)

**Dimensions:**
```
Haussmann Boulevard:
  Width: 100-130 ft (30-40m) = 2-3 cells wide
  Building height: 65 ft (20m) = 5-6 stories (mandated uniform)
  Street-wall: continuous facade, zero setback
  Block depth: 150-300 ft (46-91m) behind the facade

Ring Road (Ringstrasse, Vienna):
  Width: 190 ft (58m) = 3-4 cells wide
  Diameter: ~2.5 miles (4km) = ~250 cells

Radial Avenue:
  Length from center: 1-3 miles (1.6-4.8km) = 100-300 cells
  Angle between avenues: 15-45 degrees
  Number of spokes: 8-24 (Paris has ~12 major axes)
```

**Game Implementation Challenges:**
- Diagonal roads on a square grid create jagged edges
- Wedge-shaped lots at intersections waste space
- Traffic circles (roundabouts) need special intersection logic
- Requires Bezier curves from `RoadSegmentStore` for smooth diagonals

**Radial Pattern Detection (is the player building a radial city?):**

```rust
pub fn detect_radial_pattern(
    grid: &WorldGrid,
    center_x: usize,
    center_y: usize,
    radius: usize,
) -> RadialScore {
    let mut spoke_count = 0;
    let mut ring_count = 0;

    // Check for spokes: lines of road cells radiating from center
    for angle_deg in (0..360).step_by(15) {
        let angle = (angle_deg as f64).to_radians();
        let mut road_cells = 0;
        let mut total_cells = 0;

        for r in 1..radius {
            let x = center_x as f64 + r as f64 * angle.cos();
            let y = center_y as f64 + r as f64 * angle.sin();
            let gx = x.round() as usize;
            let gy = y.round() as usize;
            if grid.in_bounds(gx, gy) {
                total_cells += 1;
                if grid.get(gx, gy).cell_type == CellType::Road {
                    road_cells += 1;
                }
            }
        }

        if total_cells > 0 && road_cells as f32 / total_cells as f32 > 0.7 {
            spoke_count += 1;
        }
    }

    // Check for rings: concentric circles of road cells
    for r in (5..radius).step_by(5) {
        let circumference = (2.0 * std::f64::consts::PI * r as f64) as usize;
        let mut road_cells = 0;

        for i in 0..circumference {
            let angle = (i as f64 / circumference as f64) * 2.0 * std::f64::consts::PI;
            let x = center_x as f64 + r as f64 * angle.cos();
            let y = center_y as f64 + r as f64 * angle.sin();
            let gx = x.round() as usize;
            let gy = y.round() as usize;
            if grid.in_bounds(gx, gy) && grid.get(gx, gy).cell_type == CellType::Road {
                road_cells += 1;
            }
        }

        if road_cells as f32 / circumference as f32 > 0.5 {
            ring_count += 1;
        }
    }

    RadialScore {
        spokes: spoke_count,
        rings: ring_count,
        is_radial: spoke_count >= 6 && ring_count >= 2,
    }
}
```

### 4.3 Organic Pattern (Medieval Cities)

**Real-World Characteristics:**
- Medieval European cities: narrow, winding streets following topography
- Middle Eastern medinas: dense, labyrinthine patterns
- Japanese historic districts: irregular blocks following ancient paths and waterways
- Unplanned: streets followed cow paths, property boundaries, terrain
- Creates intimate, human-scale spaces but confusing navigation

**Dimensions:**
```
Medieval lane: 6-15 ft (2-5m) wide = less than 1 cell (render as narrow)
Medieval street: 15-30 ft (5-10m) wide = 1 cell
Market square: 100-300 ft (30-90m) = 2x2 to 6x6 cells
Block size: irregular, 50-200 ft on each side
Maximum block perimeter: often very large (dead-end alleys)
```

**Game Mechanic: Historic District Bonus**

If the game detects an organic pattern (low intersection regularity, many curves, small blocks), it can be classified as a "historic district" with bonuses:

```rust
pub struct OrganicPatternMetrics {
    pub intersection_angle_variance: f32,  // High = organic
    pub block_size_variance: f32,          // High = organic
    pub average_street_width: f32,         // Low = organic
    pub dead_end_ratio: f32,               // Higher in organic patterns
    pub curve_frequency: f32,              // Roads that change direction often
}

impl OrganicPatternMetrics {
    pub fn is_organic(&self) -> bool {
        self.intersection_angle_variance > 30.0  // degrees
        && self.block_size_variance > 0.4        // coefficient of variation
        && self.average_street_width < 1.5       // cells
    }

    /// Tourism and cultural bonus for organic street patterns
    pub fn heritage_bonus(&self) -> f32 {
        if self.is_organic() {
            15.0 // +15 tourism attractiveness, +10 land value
        } else {
            0.0
        }
    }
}
```

### 4.4 Cul-de-Sac Suburbs

**Real-World Characteristics:**
- Dominant American suburban pattern since the 1950s
- Curved streets, cul-de-sacs (dead-end loops), no through traffic
- Designed to minimize traffic speed and volume on residential streets
- Maximizes lot area but minimizes connectivity
- Creates car-dependent neighborhoods with poor walkability

**Dimensions:**
```
Cul-de-sac bulb: 80-100 ft (24-30m) diameter = 2 cell radius
Cul-de-sac length: 300-1,000 ft (91-305m) = 6-19 cells
Local collector road: 60 ft (18m) ROW = 1 cell
Lot size: 7,000-15,000 sq ft = 3-6 cells
Block size: irregular, often 500-1,000 ft (30-60 cells)
```

**Game Metrics:**
```
Intersection density: LOW (typically 0.3-0.5x grid equivalent)
Dead end ratio: 30-60% of all road terminuses are dead ends
Connectivity index: 0.8-1.1 (poor, near-tree topology)
Route directness: 1.8-3.0 (long, winding paths)
Vehicle trips: HIGH (every trip requires collector -> arterial -> collector)
Walk Score: 10-30 (car-dependent)
Traffic on arterials: VERY HIGH (all traffic funneled to few roads)
```

**Why Cul-de-Sacs Are a Game Design Trap:**

Players who build cul-de-sac suburbs should experience realistic consequences:
1. **Massive traffic on collector roads**: All trips funnel to 1-2 exit points
2. **Poor service coverage**: Fire trucks, ambulances, garbage trucks have to traverse dead-end loops
3. **Low Walk Score**: Residents cannot walk to services, must drive
4. **High infrastructure cost per capita**: More road surface per household
5. **Higher childhood obesity**: Yes, this is a documented real-world effect (no sidewalks, unsafe for kids to walk)

```rust
/// Detect cul-de-sac ratio in a district and apply penalties
pub fn cul_de_sac_penalty(
    grid: &WorldGrid,
    district_cells: &[(usize, usize)],
) -> f32 {
    let mut dead_ends = 0;
    let mut intersections = 0;

    for &(x, y) in district_cells {
        if grid.get(x, y).cell_type != CellType::Road {
            continue;
        }

        let (neighbors, count) = grid.neighbors4(x, y);
        let road_neighbors = neighbors[..count].iter()
            .filter(|&&(nx, ny)| grid.get(nx, ny).cell_type == CellType::Road)
            .count();

        if road_neighbors == 1 {
            dead_ends += 1;
        } else if road_neighbors >= 3 {
            intersections += 1;
        }
    }

    let total_termini = dead_ends + intersections;
    if total_termini == 0 {
        return 0.0;
    }

    let dead_end_ratio = dead_ends as f32 / total_termini as f32;

    // Penalty scales with dead-end ratio
    // 0% dead ends = 0 penalty
    // 50% dead ends = -5 happiness, +10% traffic on collectors
    // 80% dead ends = -10 happiness, +30% traffic on collectors
    dead_end_ratio * 15.0
}
```

### 4.5 Barcelona Superblocks

**Real-World Background:**
Barcelona's Superblocks (Superilles) are a revolutionary urban planning concept first proposed by Salvador Rueda in 1993 and implemented starting in 2016. The idea is simple: take a 3x3 block of the Eixample grid (each block is 113m x 113m in Barcelona) and restrict through-traffic to the perimeter roads. Interior streets become pedestrian plazas, playgrounds, and green space.

**Dimensions:**
```
Barcelona Eixample block: 113m x 113m = 7x7 cells
Superblock: 3x3 blocks = 339m x 339m = 21x21 cells
Interior roads: converted to pedestrian (speed 5-10 km/h, no through traffic)
Perimeter roads: remain as avenues (30-50 km/h)
Green space gained: ~70% of interior road surface
```

**Why This Is a Great Game Mechanic:**

Superblocks create a fascinating tradeoff:
- **Positive**: Massive happiness bonus from reduced noise/pollution, more green space, safer streets, higher land value
- **Negative**: Reduced road network capacity, longer vehicle routes, potential traffic congestion on perimeter
- **Player decision**: Which neighborhoods benefit most from superblock conversion? (High residential density = best candidates)

**Implementation as a District-Level Policy:**

```rust
pub struct SuperblockConfig {
    /// Center cell of the superblock
    pub center_x: usize,
    pub center_y: usize,
    /// Radius in cells (typically 10-12 cells = ~3 blocks)
    pub radius: usize,
    /// Which interior roads become pedestrian
    pub converted_roads: Vec<(usize, usize)>,
}

pub fn apply_superblock(
    grid: &mut WorldGrid,
    config: &SuperblockConfig,
) {
    let r = config.radius as i32;
    let cx = config.center_x as i32;
    let cy = config.center_y as i32;

    for dy in -r..=r {
        for dx in -r..=r {
            let x = (cx + dx) as usize;
            let y = (cy + dy) as usize;

            if !grid.in_bounds(x, y) {
                continue;
            }

            let cell = grid.get(x, y);
            if cell.cell_type != CellType::Road {
                continue;
            }

            // Perimeter roads stay as-is
            let is_perimeter = dx.abs() == r || dy.abs() == r;
            if is_perimeter {
                continue;
            }

            // Interior roads become pedestrian paths
            let cell_mut = grid.get_mut(x, y);
            cell_mut.road_type = RoadType::Path;
            // This automatically: reduces noise (Path noise = 0),
            // increases land value (less traffic), improves walkability
        }
    }
}

/// Superblock effects on surrounding cells
pub struct SuperblockEffects {
    pub happiness_bonus: f32,       // +8-12 for residents inside
    pub land_value_bonus: u8,       // +15-25 for cells inside
    pub noise_reduction: u8,        // -10-20 noise level
    pub pollution_reduction: u8,    // -5-10 air pollution
    pub traffic_penalty_perimeter: f32, // +20-40% congestion on perimeter roads
    pub green_space_cells: u32,     // number of road cells converted to pedestrian
}
```

**Superblock Scoring (should the player build one here?):**

```
Benefit = (residential_population_inside * happiness_bonus_per_person)
        + (land_value_increase * property_tax_rate * affected_cells)
        + (tourism_boost * commercial_activity)
        - (traffic_congestion_cost * vehicles_rerouted * avg_trip_length_increase)
        - (commercial_revenue_loss * affected_businesses * accessibility_reduction)

Optimal locations:
  - High residential density (many people benefit)
  - Low existing commercial (less disruption)
  - Well-connected perimeter roads (can absorb rerouted traffic)
  - Already high land value (amplifies the increase)
```

### 4.6 Street Dimensions and Right-of-Way

**Complete Street Cross-Sections (mapped to Megacity grid):**

```
Path (Pedestrian Only):
  Real: 8-12 ft (2.5-3.7m)
  Grid: 1 cell (16m, but renders as narrow path with greenery)
  Lanes: 0 vehicle lanes
  Speed: 5 km/h (walking)
  Parking: None
  Trees: Yes
  Cost: $5

Local Street (2-lane):
  Real: 36-44 ft (11-13m) ROW
    2 travel lanes @ 10 ft each = 20 ft
    2 parking lanes @ 7 ft each = 14 ft
    2 sidewalks @ 4 ft each = 8 ft
    Total: 42 ft ROW
  Grid: 1 cell (16m ROW, realistic for a narrow 2-lane)
  Lanes: 2
  Speed: 30 km/h
  Parking: Street parking (absorbed into capacity)
  Cost: $10

Avenue (4-lane):
  Real: 66-84 ft (20-26m) ROW
    4 travel lanes @ 11 ft each = 44 ft
    Median: 6-10 ft (optional)
    2 parking lanes @ 8 ft each = 16 ft
    2 sidewalks @ 6 ft each = 12 ft
    Total: 78 ft ROW
  Grid: 1 cell (compressed; could be 2 cells for realism)
  Lanes: 4
  Speed: 50 km/h
  Parking: Street parking
  Cost: $20

Boulevard (6-lane):
  Real: 100-130 ft (30-40m) ROW
    6 travel lanes @ 11 ft each = 66 ft
    2 medians @ 8 ft each = 16 ft
    2 bike lanes @ 5 ft each = 10 ft
    2 sidewalks @ 8 ft each = 16 ft
    Total: 108 ft ROW (minimum)
  Grid: 1 cell (heavily compressed) or 2 cells for visual accuracy
  Lanes: 6
  Speed: 60 km/h
  Parking: None (or off-street)
  Cost: $30

Highway (4-lane divided):
  Real: 120-180 ft (37-55m) ROW
    4 travel lanes @ 12 ft each = 48 ft
    Median (often grassy): 30-60 ft = ~45 ft
    2 shoulders @ 10 ft each = 20 ft
    Sound walls + buffer: 30-50 ft total
    Total: 140-170 ft ROW
  Grid: 2-3 cells wide (currently 1 cell, unrealistically narrow)
  Lanes: 4 (divided)
  Speed: 100 km/h
  Parking: None (illegal)
  Cost: $40
  Noise: HIGH (radius 8 cells)
  Zoning: NO zoning allowed (no frontage access)

One-Way (2-lane):
  Real: 30-40 ft (9-12m) ROW
    2 travel lanes @ 10 ft each = 20 ft
    1 parking lane @ 8 ft = 8 ft
    2 sidewalks @ 4 ft each = 8 ft
    Total: 36 ft ROW
  Grid: 1 cell
  Lanes: 2 (same direction)
  Speed: 40 km/h
  Cost: $15
```

**Critical Note on Current Implementation:**

The current `RoadType::width_cells()` returns 1 for ALL road types. This is unrealistic for highways and boulevards but acceptable for gameplay simplicity. If visual realism is desired, highways should be 2-3 cells wide, which means:
- Boulevard: 2 cells (32m ROW)
- Highway: 3 cells (48m ROW), consistent with real-world ROW
- This reduces buildable area but creates realistic highway corridors

### 4.7 Street Pattern Detection Algorithm

A system that detects what pattern the player is building and provides bonuses/feedback:

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StreetPatternType {
    Grid,           // Regular rectangular grid
    Radial,         // Spokes from center with ring roads
    Organic,        // Irregular, winding streets
    CulDeSac,       // Dead-end heavy suburban
    Superblock,     // Grid with interior pedestrian conversion
    Mixed,          // Combination of patterns
}

pub struct StreetPatternAnalysis {
    pub pattern: StreetPatternType,
    pub grid_regularity: f32,      // 0.0-1.0 (1.0 = perfect grid)
    pub connectivity_index: f32,   // links/nodes ratio
    pub dead_end_ratio: f32,       // proportion of dead ends
    pub average_block_size: f32,   // in cells
    pub intersection_density: f32, // intersections per 100 cells
}

pub fn analyze_street_pattern(grid: &WorldGrid) -> StreetPatternAnalysis {
    let mut nodes = 0u32;      // intersections (3+ road neighbors)
    let mut links = 0u32;      // road segments between nodes
    let mut dead_ends = 0u32;  // 1 road neighbor
    let mut total_roads = 0u32;

    // Count road cells and classify intersections
    for y in 0..grid.height {
        for x in 0..grid.width {
            if grid.get(x, y).cell_type != CellType::Road {
                continue;
            }
            total_roads += 1;

            let (n4, count) = grid.neighbors4(x, y);
            let road_neighbors = n4[..count].iter()
                .filter(|&&(nx, ny)| grid.get(nx, ny).cell_type == CellType::Road)
                .count();

            match road_neighbors {
                0 => {} // isolated road cell
                1 => dead_ends += 1,
                2 => {} // mid-segment
                _ => { nodes += 1; links += road_neighbors as u32; }
            }
        }
    }

    links /= 2; // each link counted twice (from each end)

    let connectivity = if nodes > 0 {
        links as f32 / nodes as f32
    } else {
        0.0
    };

    let dead_end_ratio = if nodes + dead_ends > 0 {
        dead_ends as f32 / (nodes + dead_ends) as f32
    } else {
        0.0
    };

    // Grid regularity: check if intersections are evenly spaced
    // (simplified: variance of distances between adjacent intersections)
    let grid_regularity = compute_grid_regularity(grid);

    let pattern = if grid_regularity > 0.8 && dead_end_ratio < 0.1 {
        StreetPatternType::Grid
    } else if dead_end_ratio > 0.4 {
        StreetPatternType::CulDeSac
    } else if grid_regularity < 0.3 {
        StreetPatternType::Organic
    } else {
        StreetPatternType::Mixed
    };

    let intersection_density = if total_roads > 0 {
        nodes as f32 / total_roads as f32 * 100.0
    } else {
        0.0
    };

    StreetPatternAnalysis {
        pattern,
        grid_regularity,
        connectivity_index: connectivity,
        dead_end_ratio,
        average_block_size: estimate_block_size(grid, total_roads),
        intersection_density,
    }
}
```

**Pattern Bonuses and Penalties:**

| Pattern | Walk Score | Traffic Efficiency | Land Value | Tourism | Emergency Response |
|---------|-----------|-------------------|------------|---------|-------------------|
| Grid | +15 | +10% | Neutral | Neutral | +20% (fast routing) |
| Radial | +10 | +5% (to center) | +5 (near center) | +10 (vistas) | +10% |
| Organic | +5 | -15% | +10 (charm) | +20 (heritage) | -15% (confusing) |
| Cul-de-Sac | -20 | -30% | -5 | -10 | -25% |
| Superblock | +25 | -10% (perimeter) | +15 | +5 | -5% (interior only) |

---

## 5. Neighborhood Design and Walkability

### 5.1 15-Minute City Scoring Algorithm

The 15-Minute City concept, popularized by Carlos Moreno at the Sorbonne, proposes that every resident should be able to reach essential daily services within a 15-minute walk or bike ride. This has become a major urban planning goal in Paris (under Mayor Anne Hidalgo), Melbourne, Portland, and dozens of other cities.

**What "15 Minutes" Means in Megacity Units:**

```
Walking speed: ~5 km/h = ~83 m/min
15-minute walk: ~1,250 m = 78 cells (at 16m/cell)
10-minute walk: ~830 m = 52 cells
5-minute walk: ~415 m = 26 cells

Cycling speed: ~15 km/h = ~250 m/min
15-minute bike ride: ~3,750 m = 234 cells (nearly the entire map)
10-minute bike ride: ~2,500 m = 156 cells
5-minute bike ride: ~1,250 m = 78 cells

Driving speed (urban): ~30 km/h = ~500 m/min
5-minute drive: ~2,500 m = 156 cells
```

Note: Walking distance should follow actual paths (roads, paths) not straight-line distance. Use A* pathfinding on the pedestrian-accessible road network (all roads except Highway) for accurate walking distance.

**The 6 Essential Categories (Moreno Model):**

1. **Live**: Housing (residential buildings) -- always satisfied at home
2. **Work**: Employment (commercial, industrial, office buildings) -- within 15 min
3. **Supply**: Daily shopping (grocery, pharmacy, convenience) -- within 10 min
4. **Care**: Health services (clinic, hospital) -- within 15 min
5. **Learn**: Education (schools, libraries) -- within 15 min
6. **Enjoy**: Recreation (parks, culture, entertainment) -- within 10 min

**Scoring Algorithm:**

```rust
pub struct FifteenMinuteScore {
    pub total: f32,          // 0.0 to 100.0
    pub work: f32,           // 0-100 (can you reach a job?)
    pub supply: f32,         // 0-100 (grocery/shops within 10 min walk)
    pub care: f32,           // 0-100 (healthcare within 15 min walk)
    pub learn: f32,          // 0-100 (education within 15 min walk)
    pub enjoy: f32,          // 0-100 (parks/culture within 10 min walk)
    pub transit: f32,        // 0-100 (public transit within 5 min walk)
}

pub fn compute_15min_score(
    grid: &WorldGrid,
    buildings: &[&Building],
    services: &[&ServiceBuilding],
    home_x: usize,
    home_y: usize,
) -> FifteenMinuteScore {
    // Walking distances via BFS on pedestrian-accessible roads
    let walk_distances = bfs_walk_distances(grid, home_x, home_y, 78); // max 78 cells

    // Work score: nearest job-providing building
    let nearest_job = buildings.iter()
        .filter(|b| b.zone_type.is_job_zone())
        .filter_map(|b| walk_distances.get(&(b.grid_x, b.grid_y)))
        .min()
        .copied()
        .unwrap_or(999);

    let work_score = if nearest_job <= 78 {
        ((78 - nearest_job) as f32 / 78.0 * 100.0).min(100.0)
    } else {
        0.0
    };

    // Supply score: nearest commercial building (grocery proxy)
    let nearest_shop = buildings.iter()
        .filter(|b| b.zone_type.is_commercial())
        .filter_map(|b| walk_distances.get(&(b.grid_x, b.grid_y)))
        .min()
        .copied()
        .unwrap_or(999);

    let supply_score = if nearest_shop <= 52 { // 10 min
        ((52 - nearest_shop) as f32 / 52.0 * 100.0).min(100.0)
    } else if nearest_shop <= 78 { // 10-15 min
        ((78 - nearest_shop) as f32 / 78.0 * 50.0) // partial credit
    } else {
        0.0
    };

    // Care score: nearest health service
    let nearest_health = services.iter()
        .filter(|s| ServiceBuilding::is_health(s.service_type))
        .filter_map(|s| walk_distances.get(&(s.grid_x, s.grid_y)))
        .min()
        .copied()
        .unwrap_or(999);

    let care_score = if nearest_health <= 78 {
        ((78 - nearest_health) as f32 / 78.0 * 100.0).min(100.0)
    } else {
        0.0
    };

    // Learn score: nearest education service
    let nearest_edu = services.iter()
        .filter(|s| ServiceBuilding::is_education(s.service_type))
        .filter_map(|s| walk_distances.get(&(s.grid_x, s.grid_y)))
        .min()
        .copied()
        .unwrap_or(999);

    let learn_score = if nearest_edu <= 78 {
        ((78 - nearest_edu) as f32 / 78.0 * 100.0).min(100.0)
    } else {
        0.0
    };

    // Enjoy score: nearest park or entertainment
    let nearest_park = services.iter()
        .filter(|s| ServiceBuilding::is_park(s.service_type))
        .filter_map(|s| walk_distances.get(&(s.grid_x, s.grid_y)))
        .min()
        .copied()
        .unwrap_or(999);

    let enjoy_score = if nearest_park <= 52 { // 10 min
        ((52 - nearest_park) as f32 / 52.0 * 100.0).min(100.0)
    } else {
        0.0
    };

    // Transit score: nearest transit stop
    let nearest_transit = services.iter()
        .filter(|s| ServiceBuilding::is_transport(s.service_type))
        .filter_map(|s| walk_distances.get(&(s.grid_x, s.grid_y)))
        .min()
        .copied()
        .unwrap_or(999);

    let transit_score = if nearest_transit <= 26 { // 5 min
        ((26 - nearest_transit) as f32 / 26.0 * 100.0).min(100.0)
    } else if nearest_transit <= 52 { // 5-10 min
        ((52 - nearest_transit) as f32 / 52.0 * 50.0)
    } else {
        0.0
    };

    // Weighted total
    let total = work_score * 0.25
              + supply_score * 0.20
              + care_score * 0.15
              + learn_score * 0.15
              + enjoy_score * 0.15
              + transit_score * 0.10;

    FifteenMinuteScore {
        total,
        work: work_score,
        supply: supply_score,
        care: care_score,
        learn: learn_score,
        enjoy: enjoy_score,
        transit: transit_score,
    }
}
```

**Performance Optimization:**

Computing 15-minute scores for every cell is expensive (BFS per cell). Strategies:

1. **Sample-based**: Compute for 1% of residential cells per slow tick, aggregate by district
2. **Grid-based aggregation**: Compute once per 16x16 district (existing `DISTRICT_SIZE`), use district center as representative point
3. **Cache with invalidation**: Store scores, invalidate when buildings/services change in the district
4. **Approximation**: Use Manhattan distance instead of BFS for initial scoring, BFS only for cells near threshold boundaries

### 5.2 Walk Score Methodology (Simplified)

Walk Score (walkscore.com) is the most widely recognized walkability metric in the United States and is used in real estate listings. Their algorithm was published in a 2011 paper and can be simplified for game use.

**Original Walk Score Algorithm:**

1. For a given address, identify the nearest establishment in 13 categories:
   - Grocery (weight 3), Restaurants (weight 0.75 each, max 10), Shopping (weight 0.5 each, max 5), Coffee (weight 1.25 each, max 2), Banks (weight 1), Parks (weight 1), Schools (weight 1), Books (weight 0.5), Entertainment (weight 0.5)

2. For each category, apply a distance decay function:
   ```
   Raw score = weight * decay(distance)

   Distance decay:
     0-400m (5 min):     1.00 (full score)
     400-800m (10 min):   0.75
     800-1200m (15 min):  0.50
     1200-1600m (20 min): 0.25
     >1600m (>20 min):    0.00
   ```

3. Sum all raw scores and normalize to 0-100.

4. Apply an "intersection density penalty" -- areas with few intersections per square mile get a deduction (this penalizes sprawl).

**Simplified Walk Score for Megacity (9 categories):**

```rust
pub struct WalkScoreWeights {
    pub grocery: f32,       // Commercial buildings (weight 3.0)
    pub restaurant: f32,    // Commercial Low/High (weight 2.0)
    pub school: f32,        // Education services (weight 1.5)
    pub park: f32,          // Park services (weight 1.5)
    pub health: f32,        // Health services (weight 1.5)
    pub transit: f32,       // Transport services (weight 1.0)
    pub entertainment: f32, // Entertainment services (weight 1.0)
    pub office: f32,        // Office/job access (weight 0.5)
    pub police: f32,        // Safety (weight 0.5)
}

impl Default for WalkScoreWeights {
    fn default() -> Self {
        Self {
            grocery: 3.0,
            restaurant: 2.0,
            school: 1.5,
            park: 1.5,
            health: 1.5,
            transit: 1.0,
            entertainment: 1.0,
            office: 0.5,
            police: 0.5,
        }
    }
}

/// Distance decay function (0-1)
pub fn walk_score_decay(distance_cells: u32) -> f32 {
    // Convert cells to approximate meters (1 cell = 16m)
    // But we use Manhattan distance on grid, so scale accordingly
    match distance_cells {
        0..=25 => 1.0,     // 0-400m: full credit
        26..=50 => 0.75,   // 400-800m: 3/4 credit
        51..=75 => 0.50,   // 800-1200m: half credit
        76..=100 => 0.25,  // 1200-1600m: quarter credit
        _ => 0.0,          // >1600m: no credit
    }
}

pub fn compute_walk_score(
    grid: &WorldGrid,
    buildings: &[&Building],
    services: &[&ServiceBuilding],
    x: usize,
    y: usize,
) -> u8 {
    let weights = WalkScoreWeights::default();
    let mut total_score: f32 = 0.0;
    let max_possible: f32 = 12.0; // sum of all weights

    // For each category, find nearest and apply decay
    let categories: Vec<(Box<dyn Fn(&&Building) -> bool>, Box<dyn Fn(&&ServiceBuilding) -> bool>, f32)> = vec![
        // ... category filter functions and weights
    ];

    // Simplified: use precomputed ServiceCoverageGrid flags
    let idx = y * grid.width + x;

    // Grocery/Supply: nearest commercial building
    let nearest_commercial = buildings.iter()
        .filter(|b| b.zone_type.is_commercial())
        .map(|b| manhattan_dist(x, y, b.grid_x, b.grid_y))
        .min()
        .unwrap_or(999);
    total_score += weights.grocery * walk_score_decay(nearest_commercial);

    // ... similar for each category

    // Intersection density bonus/penalty
    let intersection_density = count_intersections_in_radius(grid, x, y, 25);
    let connectivity_factor = (intersection_density as f32 / 10.0).clamp(0.5, 1.2);

    let raw_score = (total_score / max_possible) * 100.0 * connectivity_factor;

    raw_score.clamp(0.0, 100.0) as u8
}
```

**Walk Score Rating Bands:**
```
90-100: Walker's Paradise (daily errands do not require a car)
70-89:  Very Walkable (most errands can be accomplished on foot)
50-69:  Somewhat Walkable (some errands can be accomplished on foot)
25-49:  Car-Dependent (most errands require a car)
0-24:   Almost All Errands Require a Car
```

**Game Integration:**

Walk Score can be used as a multiplier on several existing systems:
- **Happiness**: +0 to +10 based on Walk Score tier
- **Land value**: Walk Score > 70 adds +5-15 land value
- **Traffic generation**: High Walk Score reduces vehicle trips by 10-30%
- **Health**: High Walk Score improves citizen health by +3-8 (walking is exercise)
- **Commercial revenue**: Walk Score > 60 increases commercial building revenue by 10-20%

### 5.3 Perry's Neighborhood Unit

Clarence Perry's Neighborhood Unit (1929) is one of the most influential concepts in urban planning. It defines the ideal residential neighborhood as:

**The Five Principles:**

1. **Size**: The neighborhood should house enough people to support one elementary school (typically 5,000-9,000 people, ~160 acres / ~2,560 cells in Megacity)

2. **Boundaries**: Bounded by arterial streets on all sides (no through traffic within the neighborhood)

3. **Open spaces**: A system of small parks and recreation spaces, totaling at least 10% of neighborhood area

4. **Institutional sites**: Schools, churches, and community buildings clustered near the center

5. **Local shops**: Commercial areas at the periphery, particularly at intersections with arterial streets (where traffic exists)

6. **Internal streets**: A network of local streets designed for residential traffic only, with no shortcuts for through traffic

**Mapping to Megacity Grid:**

```
Perry Neighborhood Unit:
  Total area: ~160 acres = ~2,500 cells (50x50 cell square)
  Population: 5,000-9,000 people
  Density: 20-36 people/acre = ~2-3.6 per cell

Layout (50x50 cells):
  Perimeter: Avenues/Boulevards (rows 0, 49 and columns 0, 49)
  Corners: Commercial buildings (4x4 cell commercial zones at each corner)
  Center: Elementary school + park (6x6 cell complex)
  Interior roads: Local streets in grid or curvilinear pattern
  Residential: Fill remaining area (approximately 1,800 cells)

  Open space: 250 cells (10%) distributed as:
    - Central park: 36 cells (6x6)
    - Pocket parks: 4 x 16 cells = 64 cells (4x4 each)
    - Playground: 2 x 25 cells = 50 cells (5x5 each)
    - Green buffers: ~100 cells along arterials
```

**Detection Algorithm (does this district match the Perry Unit?):**

```rust
pub struct PerryUnitScore {
    pub total: f32,                // 0-100
    pub has_school_center: bool,   // School within 5 cells of geographic center
    pub has_park_center: bool,     // Park within 5 cells of center
    pub arterial_boundary: f32,    // % of perimeter that is arterial road
    pub open_space_ratio: f32,     // % of area that is open/park
    pub commercial_at_edges: bool, // Commercial concentrated at perimeter
    pub through_traffic: f32,      // Amount of non-local traffic (lower = better)
}

pub fn evaluate_perry_unit(
    grid: &WorldGrid,
    buildings: &[&Building],
    services: &[&ServiceBuilding],
    district_center_x: usize,
    district_center_y: usize,
    district_radius: usize,
) -> PerryUnitScore {
    let r = district_radius;

    // 1. Check for school near center
    let has_school = services.iter().any(|s| {
        ServiceBuilding::is_education(s.service_type)
        && manhattan_dist(district_center_x, district_center_y, s.grid_x, s.grid_y) <= 5
    });

    // 2. Check for park near center
    let has_park = services.iter().any(|s| {
        ServiceBuilding::is_park(s.service_type)
        && manhattan_dist(district_center_x, district_center_y, s.grid_x, s.grid_y) <= 5
    });

    // 3. Check arterial boundary percentage
    let perimeter_cells = 4 * (2 * r); // approximate perimeter
    let mut perimeter_arterials = 0;
    // ... count Boulevard/Avenue/Highway on perimeter

    let arterial_ratio = perimeter_arterials as f32 / perimeter_cells as f32;

    // 4. Open space ratio
    let total_cells = (2 * r) * (2 * r);
    let open_cells = count_open_space(grid, district_center_x, district_center_y, r);
    let open_ratio = open_cells as f32 / total_cells as f32;

    // 5. Commercial at edges
    let commercial_at_edge = buildings.iter()
        .filter(|b| b.zone_type.is_commercial())
        .filter(|b| {
            let dx = (b.grid_x as i32 - district_center_x as i32).abs();
            let dy = (b.grid_y as i32 - district_center_y as i32).abs();
            dx.max(dy) >= (r as i32 - 3) // within 3 cells of boundary
        })
        .count();

    let total_commercial = buildings.iter()
        .filter(|b| b.zone_type.is_commercial())
        .count();

    let commercial_edge_ratio = if total_commercial > 0 {
        commercial_at_edge as f32 / total_commercial as f32
    } else {
        0.0
    };

    // Score
    let mut score = 0.0;
    if has_school { score += 20.0; }
    if has_park { score += 15.0; }
    score += arterial_ratio * 20.0;
    score += (open_ratio / 0.10).min(1.0) * 15.0; // full marks at 10% open space
    score += commercial_edge_ratio * 15.0;
    score += (1.0 - 0.0) * 15.0; // placeholder for through-traffic metric

    PerryUnitScore {
        total: score.min(100.0),
        has_school_center: has_school,
        has_park_center: has_park,
        arterial_boundary: arterial_ratio,
        open_space_ratio: open_ratio,
        commercial_at_edges: commercial_edge_ratio > 0.6,
        through_traffic: 0.0, // computed from traffic simulation
    }
}
```

### 5.4 Transit-Oriented Development (TOD)

Transit-Oriented Development concentrates dense, mixed-use development around transit stations. The concept was formalized by Peter Calthorpe in 1993 and is now standard practice in cities worldwide.

**The TOD Model (Density Tapering):**

```
Distance from Station    Density (FAR)    Use Mix
0-400m (0-25 cells):     4.0-8.0 FAR      Mixed-use (retail ground floor, offices/residential above)
400-800m (25-50 cells):  2.0-4.0 FAR      Residential + neighborhood commercial
800-1200m (50-75 cells): 1.0-2.0 FAR      Medium-density residential
>1200m (>75 cells):      0.5-1.0 FAR      Low-density residential (if any)

Station area itself: 2x2 to 4x4 cells (transit station + plaza)
```

**The "Station Area Plan" Pattern:**

```
Typical TOD Cluster (diameter 100 cells = 1.6 km):

           +---------+
          /           \
         / R-2  R-2   \     <-- Low density residential (FAR 0.8)
        / R-2  R-3  R-2\
       / R-3   R-3   R-3\   <-- Medium density (FAR 1.5)
      | R-3  R-4   R-4  R-3|
      | R-4  MU-3  MU-3 R-4| <-- High density mixed-use (FAR 4.0)
      |MU-3 [STATION] MU-3 | <-- Station + plaza
      | R-4  MU-3  MU-3 R-4|
      | R-3  R-4   R-4  R-3|
       \ R-3   R-3   R-3 /
        \ R-2  R-3  R-2 /
         \ R-2  R-2   /
          \           /
           +---------+
```

**Implementation as Auto-Zoning Suggestion:**

```rust
pub struct TODPlan {
    pub station_x: usize,
    pub station_y: usize,
    pub station_type: ServiceType,
    pub zones: Vec<(usize, usize, ZoneType, TransectZone)>,
}

pub fn generate_tod_plan(
    station_x: usize,
    station_y: usize,
    station_type: ServiceType,
    grid: &WorldGrid,
) -> TODPlan {
    let mut zones = Vec::new();

    // Determine station importance (affects density ring sizes)
    let importance = match station_type {
        ServiceType::SubwayStation | ServiceType::TrainStation => 3,
        ServiceType::TramDepot | ServiceType::BusDepot => 2,
        ServiceType::FerryPier => 1,
        _ => 0,
    };

    let ring_radii = match importance {
        3 => [8, 20, 40, 60],  // Major station: dense core 8 cells, medium 20, etc.
        2 => [5, 15, 30, 45],  // Medium station
        1 => [3, 10, 20, 35],  // Minor station
        _ => [2, 8, 15, 25],   // Bus stop
    };

    for dy in -(ring_radii[3] as i32)..=(ring_radii[3] as i32) {
        for dx in -(ring_radii[3] as i32)..=(ring_radii[3] as i32) {
            let x = station_x as i32 + dx;
            let y = station_y as i32 + dy;

            if x < 0 || y < 0 || x >= grid.width as i32 || y >= grid.height as i32 {
                continue;
            }

            let dist = ((dx * dx + dy * dy) as f32).sqrt() as usize;
            let (zone, transect) = if dist <= ring_radii[0] {
                (ZoneType::CommercialHigh, TransectZone::T6Core) // Immediate station area
            } else if dist <= ring_radii[1] {
                (ZoneType::ResidentialHigh, TransectZone::T5Center) // High-density residential
            } else if dist <= ring_radii[2] {
                (ZoneType::ResidentialHigh, TransectZone::T4Urban) // Medium density
            } else if dist <= ring_radii[3] {
                (ZoneType::ResidentialLow, TransectZone::T3Suburban) // Low density
            } else {
                continue;
            };

            // Only suggest for buildable cells
            let cell = grid.get(x as usize, y as usize);
            if cell.cell_type == CellType::Grass && cell.building_id.is_none() {
                zones.push((x as usize, y as usize, zone, transect));
            }
        }
    }

    TODPlan {
        station_x,
        station_y,
        station_type,
        zones,
    }
}
```

**TOD Effects on Game Mechanics:**

| Metric | Effect at Station (0-400m) | Effect at 400-800m | Effect at 800m+ |
|--------|---------------------------|--------------------|-----------------|
| Land value | +30-50 | +15-25 | +5-10 |
| Walk Score | +20-30 | +10-15 | +5 |
| Traffic reduction | -40% vehicle trips | -25% | -10% |
| Parking demand | -50% (parking minimums should decrease) | -30% | -10% |
| Commercial viability | +40% revenue (foot traffic) | +15% | Neutral |
| Residential demand | +30% | +20% | +10% |
| Noise penalty | +5-15 (trains are loud) | +3-5 | 0 |

### 5.5 Neighborhood Quality Index

A comprehensive neighborhood quality score that combines all the above systems into a single index for player feedback and AI-driven growth decisions.

**The Neighborhood Quality Index (NQI):**

```rust
pub struct NeighborhoodQuality {
    pub nqi: f32,                    // 0-100, the headline number
    pub walkability: f32,            // Walk Score (0-100)
    pub fifteen_min: f32,            // 15-Minute City score (0-100)
    pub perry_unit: f32,             // Perry Unit conformance (0-100)
    pub transit_access: f32,         // Transit proximity (0-100)
    pub green_space: f32,            // % of area that is park/green
    pub service_coverage: f32,       // % of essential services covered
    pub safety: f32,                 // Inverse of crime (0-100)
    pub environmental: f32,          // Inverse of pollution + noise (0-100)
    pub infrastructure: f32,         // Power + water + road condition (0-100)
    pub aesthetic: f32,              // Building variety + historic character (0-100)
}

impl NeighborhoodQuality {
    pub fn compute(
        walkability: f32,
        fifteen_min: f32,
        perry_unit: f32,
        transit_access: f32,
        green_space_ratio: f32,
        service_coverage_pct: f32,
        crime_level: f32,
        pollution_level: f32,
        noise_level: f32,
        infrastructure_pct: f32,
        building_variety: f32,
    ) -> Self {
        let safety = (100.0 - crime_level).max(0.0);
        let environmental = (100.0 - pollution_level * 0.5 - noise_level * 0.5).max(0.0);
        let green = (green_space_ratio * 1000.0).min(100.0); // 10% green = 100

        let nqi = walkability * 0.15
                + fifteen_min * 0.15
                + perry_unit * 0.05
                + transit_access * 0.10
                + green * 0.10
                + service_coverage_pct * 0.15
                + safety * 0.10
                + environmental * 0.10
                + infrastructure_pct * 0.05
                + building_variety * 0.05;

        Self {
            nqi: nqi.clamp(0.0, 100.0),
            walkability,
            fifteen_min,
            perry_unit,
            transit_access,
            green_space: green,
            service_coverage: service_coverage_pct,
            safety,
            environmental,
            infrastructure: infrastructure_pct,
            aesthetic: building_variety,
        }
    }
}
```

**NQI Rating Bands:**
```
90-100: World-Class Neighborhood (comparable to best in Amsterdam, Tokyo, Vienna)
75-89:  Excellent (desirable urban living, strong demand)
60-74:  Good (functional, comfortable, competitive)
45-59:  Average (adequate but improvable)
30-44:  Below Average (declining, at risk of abandonment)
15-29:  Poor (significant problems, active disinvestment)
0-14:   Failing (abandoned, slum conditions)
```

**NQI Effects on Game Systems:**

| NQI Band | Land Value | Demand | Immigration | Crime | Property Tax Revenue |
|----------|-----------|--------|-------------|-------|---------------------|
| 90-100 | +40% | +50% | +3 per tick | -30% | +25% |
| 75-89 | +20% | +30% | +2 per tick | -15% | +15% |
| 60-74 | +5% | +10% | +1 per tick | Neutral | +5% |
| 45-59 | Neutral | Neutral | Neutral | Neutral | Neutral |
| 30-44 | -10% | -15% | -1 per tick | +10% | -10% |
| 15-29 | -25% | -30% | -2 per tick | +25% | -20% |
| 0-14 | -40% | -50% | -3 per tick | +50% | -40% |

---

## 6. Advanced Zoning Mechanics

### 6.1 NIMBY/YIMBY as Game Mechanic

NIMBY (Not In My Back Yard) and YIMBY (Yes In My Back Yard) represent the most politically contentious aspect of real-world urban planning. The conflict between existing residents who want to preserve neighborhood character and new arrivals who need housing is the central drama of cities like San Francisco, London, Sydney, and Auckland. This conflict creates excellent game mechanics.

**The Core Tension:**

- **NIMBY residents**: Already live in the neighborhood. They want: low density, quiet streets, no new construction, historic preservation, parking, single-family zoning. They vote, attend council meetings, file lawsuits.
- **YIMBY advocates**: Need housing. They want: more density, more transit, reduced parking requirements, upzoning, mixed-use. They represent future residents who do not yet live there (and therefore cannot vote on local issues).

**Implementation as a Political Pressure System:**

```rust
#[derive(Resource, Default, Serialize, Deserialize)]
pub struct NimbyPressure {
    /// Per-district NIMBY intensity (0.0 = no resistance, 1.0 = maximum resistance)
    pub district_nimby: Vec<f32>,
    /// Global YIMBY movement strength (rises with housing shortage)
    pub yimby_strength: f32,
    /// Current zoning change proposals blocked by NIMBYs
    pub blocked_proposals: Vec<ZoningProposal>,
}

pub struct ZoningProposal {
    pub x: usize,
    pub y: usize,
    pub current_zone: ZoneType,
    pub proposed_zone: ZoneType,
    pub proposed_by: ProposalSource,
    pub nimby_opposition: f32,   // 0.0-1.0
    pub yimby_support: f32,      // 0.0-1.0
    pub status: ProposalStatus,
}

pub enum ProposalSource {
    Player,          // Direct player action
    Developer,       // AI developer seeking to build
    CityPlanner,     // Advisor recommendation
}

pub enum ProposalStatus {
    Proposed,
    UnderReview,     // 30-90 day public comment period
    Approved,
    Blocked,         // NIMBY opposition exceeded threshold
    Appealed,        // Player overrode NIMBY (costs political capital)
}
```

**NIMBY Intensity Calculation:**

```rust
pub fn compute_nimby_intensity(
    district: &District,
    current_zone: ZoneType,
    proposed_zone: ZoneType,
    avg_land_value: f32,
    avg_wealth: f32,
    homeownership_rate: f32,
) -> f32 {
    let mut intensity = 0.0;

    // Base NIMBY: scales with wealth and homeownership
    // Wealthy homeowners have most to lose from change
    intensity += avg_wealth * 0.2;           // 0-1 scale
    intensity += homeownership_rate * 0.3;   // renters are less NIMBY

    // Type-specific opposition
    let change_severity = match (current_zone, proposed_zone) {
        // Upzoning always generates opposition
        (ZoneType::ResidentialLow, ZoneType::ResidentialHigh) => 0.6,  // Moderate
        (ZoneType::ResidentialLow, ZoneType::CommercialHigh) => 0.9,   // Severe
        (ZoneType::ResidentialLow, ZoneType::Industrial) => 1.0,       // Maximum

        // Nearby industrial is universally opposed
        (_, ZoneType::Industrial) => 0.8,

        // Mixed-use is moderately controversial
        (ZoneType::ResidentialLow, ZoneType::CommercialLow) => 0.4,

        // Downzoning is rarely opposed (NIMBYs love it)
        (ZoneType::ResidentialHigh, ZoneType::ResidentialLow) => -0.3, // Support!

        _ => 0.2, // Default mild opposition to any change
    };
    intensity += change_severity;

    // Land value effect: high land value = more NIMBY (more to protect)
    intensity += (avg_land_value / 200.0).min(0.3);

    // Cap at 1.0
    intensity.clamp(0.0, 1.0)
}
```

**YIMBY Strength Calculation:**

```rust
pub fn compute_yimby_strength(
    housing_shortage: f32,    // unhoused / total population
    avg_rent_burden: f32,     // avg rent as % of income (0.3+ is burdened)
    vacancy_rate: f32,        // proportion of empty units (healthy: 0.05-0.08)
    young_adult_pct: f32,     // % of population age 20-35
    education_level: f32,     // average education (proxy for activism)
) -> f32 {
    let mut strength = 0.0;

    // Housing crisis drives YIMBY movement
    strength += (housing_shortage * 5.0).min(0.3);

    // Rent burden drives support for more housing
    if avg_rent_burden > 0.30 {
        strength += ((avg_rent_burden - 0.30) * 3.0).min(0.3);
    }

    // Low vacancy = tight market = more YIMBY energy
    if vacancy_rate < 0.05 {
        strength += ((0.05 - vacancy_rate) * 10.0).min(0.2);
    }

    // Young adults and educated citizens more likely to support density
    strength += young_adult_pct * 0.1;
    strength += (education_level / 3.0) * 0.1;

    strength.clamp(0.0, 1.0)
}
```

**Gameplay Consequences:**

When the player tries to upzone (change ResidentialLow to ResidentialHigh):

1. **NIMBY opposition** triggers based on district characteristics
2. **Public hearing event**: 30-90 game-day delay before zone change takes effect
3. **If NIMBY > YIMBY + player_political_capital**: Zone change is **blocked**
   - Player can override using political capital (costs mayor approval rating)
   - Or player can compromise (upzone to medium instead of high density)
4. **If YIMBY > NIMBY**: Zone change proceeds normally
5. **Side effects**:
   - Overriding NIMBYs: -5 to -15 happiness for existing residents (they feel ignored)
   - Yielding to NIMBYs: Housing shortage worsens, rents increase, homelessness rises
   - Compromise: Moderate satisfaction, moderate effect

**Political Capital System:**

```rust
pub struct PoliticalCapital {
    pub points: f32,           // 0-100, starts at 50
    pub approval_rating: f32,  // 0.0-1.0, public approval
}

impl PoliticalCapital {
    pub fn override_nimby(&mut self, nimby_intensity: f32) -> bool {
        let cost = nimby_intensity * 15.0; // Strong NIMBY costs up to 15 points
        if self.points >= cost {
            self.points -= cost;
            self.approval_rating -= nimby_intensity * 0.05;
            true
        } else {
            false // Not enough political capital
        }
    }

    pub fn gain_from_good_outcomes(&mut self, jobs_created: u32, housing_built: u32) {
        self.points += (jobs_created as f32 * 0.01 + housing_built as f32 * 0.005).min(5.0);
        self.approval_rating = (self.approval_rating + 0.01).min(1.0);
    }
}
```

### 6.2 Eminent Domain Events

Eminent domain (compulsory purchase in the UK, expropriation in most of the world) is the power of government to take private property for public use, with compensation. In city builders, this enables the player to force demolition of existing buildings to build infrastructure -- but at a cost.

**When Eminent Domain Is Needed:**
- Building a highway through an existing neighborhood
- Creating a new transit line through dense urban fabric
- Expanding a road from 2 lanes to 6 lanes
- Building a new school/hospital where buildings already exist
- Creating a park or public space in a built-up area

**Implementation:**

```rust
pub struct EminentDomainAction {
    pub cells: Vec<(usize, usize)>,
    pub affected_buildings: Vec<Entity>,
    pub affected_residents: u32,
    pub affected_workers: u32,
    pub compensation_cost: f64,     // Market value + premium (125-150% of assessed value)
    pub political_cost: f32,        // Political capital spent
    pub happiness_impact: f32,      // Negative happiness for displaced + neighbors
    pub purpose: EminentDomainPurpose,
}

pub enum EminentDomainPurpose {
    RoadWidening,
    TransitLine,
    PublicFacility,
    ParkCreation,
    UrbanRenewal,
}

pub fn calculate_eminent_domain_cost(
    grid: &WorldGrid,
    land_value: &LandValueGrid,
    buildings: &Query<&Building>,
    cells: &[(usize, usize)],
) -> EminentDomainAction {
    let mut total_compensation = 0.0f64;
    let mut affected_residents = 0u32;
    let mut affected_workers = 0u32;
    let mut affected_buildings = Vec::new();

    for &(x, y) in cells {
        let cell = grid.get(x, y);
        let lv = land_value.get(x, y) as f64;

        // Land compensation: 150% of assessed land value
        total_compensation += lv * 150.0;

        if let Some(entity) = cell.building_id {
            if let Ok(building) = buildings.get(entity) {
                // Building compensation: replacement cost
                let building_value = building_base_value(building.zone_type, building.level);
                total_compensation += building_value as f64 * 1.25; // 125% of value

                if building.zone_type.is_residential() {
                    affected_residents += building.occupants;
                } else {
                    affected_workers += building.occupants;
                }
                affected_buildings.push(entity);
            }
        }
    }

    // Political cost scales with number of affected people
    let political_cost = (affected_residents + affected_workers) as f32 * 0.1;

    // Happiness impact: displaced people are very unhappy, neighbors moderately unhappy
    let happiness_impact = -(affected_residents as f32 * 0.5 + affected_workers as f32 * 0.2);

    EminentDomainAction {
        cells: cells.to_vec(),
        affected_buildings,
        affected_residents,
        affected_workers,
        compensation_cost: total_compensation,
        political_cost,
        happiness_impact,
        purpose: EminentDomainPurpose::UrbanRenewal,
    }
}
```

**Game Events Triggered by Eminent Domain:**

1. **Protest event**: Affected residents organize protests (happiness -10 city-wide for 30 days)
2. **Relocation**: Displaced residents need new housing (increases residential demand spike)
3. **Legal challenge**: 10% chance of court challenge adding 50% to compensation cost and 90-day delay
4. **Media coverage**: If more than 100 residents affected, triggers "Urban Renewal Controversy" event
5. **Long-term effect**: After new facility built, gradual happiness recovery as benefits materialize

### 6.3 Historic Preservation

Historic preservation creates a tension between growth and character. Historically designated buildings cannot be demolished or significantly altered, which constrains development but increases tourism and cultural value.

**Historic Building Criteria:**

A building becomes eligible for historic designation when:
1. **Age**: Building has existed for at least 50 game-years (18,250 days)
2. **Significance**: Building level >= 3 when constructed (was important when built)
3. **Context**: Surrounded by other old buildings (historic district, not isolated relic)
4. **Cultural value**: In a district with Culture specialization score >= 25

**Implementation:**

```rust
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct HistoricDesignation {
    pub designated_day: u32,      // Game-day when designated
    pub preservation_level: PreservationLevel,
    pub tourism_bonus: f32,       // Additional tourism attractiveness
    pub maintenance_multiplier: f32, // Historic buildings cost more to maintain
}

pub enum PreservationLevel {
    Contributing,   // Part of a historic district, cannot be demolished
    Landmark,       // Individually significant, cannot be altered at all
    Protected,      // Exterior must be preserved, interior can be modernized
}

pub struct HistoricDistrict {
    pub name: String,
    pub cells: Vec<(usize, usize)>,
    pub designation_year: u32,
    pub max_building_height: u8,    // Cannot build taller than historic context
    pub facade_requirements: bool,  // New construction must match historic character
    pub demolition_prohibited: bool,
}

/// Effects of historic preservation on game systems
pub fn historic_preservation_effects(
    district: &HistoricDistrict,
    building_count: u32,
) -> HistoricEffects {
    HistoricEffects {
        // Positive
        tourism_bonus: building_count as f32 * 2.0,  // +2 per historic building
        land_value_bonus: 15,                          // +15 land value
        happiness_bonus: 5.0,                          // Residents value historic character
        cultural_score_bonus: building_count as f32 * 1.5,

        // Negative / constraints
        max_height: district.max_building_height,
        no_demolition: district.demolition_prohibited,
        maintenance_multiplier: 1.5,  // 50% more expensive to maintain
        development_restriction: true, // Cannot upzone
        renovation_cost_multiplier: 2.0, // Renovations cost 2x (historic materials)
    }
}
```

**Player Dilemma:**

Historic preservation forces the player to choose between:
- **Preserve**: High land value, tourism, happiness, cultural score, but limited density and expensive maintenance
- **Demolish**: Unlock land for high-density development, lose tourism and cultural benefits, NIMBY backlash
- **Adaptive reuse**: Convert historic buildings to new uses (old factory -> loft apartments, old church -> restaurant). Expensive but preserves character while adding residents/workers.

### 6.4 Urban Growth Boundaries

An Urban Growth Boundary (UGB) is a line drawn around a city beyond which urban development is prohibited or restricted. Portland, Oregon has had a UGB since 1979. Seoul, South Korea has a massive greenbelt. London's Green Belt dates to 1947.

**How UGBs Work in Real Cities:**

1. Line drawn around existing urban area
2. Development inside the boundary: permitted (with normal zoning)
3. Development outside the boundary: prohibited or severely limited (agriculture, forestry only)
4. Boundary reviewed every 10-20 years and potentially expanded if housing demand warrants it
5. Effect: forces densification of existing urban area instead of sprawl

**Game Implementation:**

```rust
#[derive(Resource, Serialize, Deserialize)]
pub struct UrbanGrowthBoundary {
    pub enabled: bool,
    pub boundary_cells: Vec<(usize, usize)>, // Cells on the boundary line
    pub inside_mask: Vec<bool>,              // True = inside UGB, one per grid cell
    pub last_expansion_day: u32,
    pub expansion_cooldown: u32,             // Days between allowed expansions
}

impl UrbanGrowthBoundary {
    /// Check if a cell is inside the growth boundary
    pub fn is_inside(&self, x: usize, y: usize) -> bool {
        if !self.enabled {
            return true; // No UGB = everything allowed
        }
        let idx = y * GRID_WIDTH + x;
        self.inside_mask.get(idx).copied().unwrap_or(false)
    }

    /// Generate initial UGB at a radius from city center
    pub fn create_circular(center_x: usize, center_y: usize, radius: usize) -> Self {
        let mut inside_mask = vec![false; GRID_WIDTH * GRID_HEIGHT];
        let mut boundary_cells = Vec::new();

        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let dx = x as i32 - center_x as i32;
                let dy = y as i32 - center_y as i32;
                let dist_sq = dx * dx + dy * dy;
                let r_sq = (radius * radius) as i32;

                if dist_sq <= r_sq {
                    inside_mask[y * GRID_WIDTH + x] = true;

                    // Mark cells near the boundary edge
                    let inner_r_sq = ((radius - 1) * (radius - 1)) as i32;
                    if dist_sq >= inner_r_sq {
                        boundary_cells.push((x, y));
                    }
                }
            }
        }

        Self {
            enabled: true,
            boundary_cells,
            inside_mask,
            last_expansion_day: 0,
            expansion_cooldown: 365 * 5, // Can only expand every 5 years
        }
    }
}
```

**UGB Game Effects:**

| Metric | With UGB | Without UGB |
|--------|----------|-------------|
| Inner land values | +20-40% (scarcity) | Baseline |
| Housing costs | +15-30% | Baseline |
| Density inside | Higher (forced) | Lower (sprawl) |
| Farmland preserved | Yes | No |
| Infrastructure cost | Lower per capita | Higher per capita |
| Traffic patterns | More transit-friendly | More car-dependent |
| Political pressure | Constant (from developers) | None |
| Greenbelt tourism | +10 | None |

**Expansion Events:**

When housing demand exceeds supply by 20%+ for 2+ years inside the UGB:
1. Advisory triggers: "Growth boundary review recommended"
2. Player can choose to expand the boundary (add 10-20 cells outward)
3. Expansion triggers: land value increase on newly included cells, construction boom
4. NIMBY resistance from existing residents near boundary (they liked the green buffer)

### 6.5 Inclusionary Zoning

Inclusionary zoning (IZ) requires developers to include a percentage of affordable housing in new developments. This is a major policy tool in cities like New York, San Francisco, London, and Vancouver.

**How IZ Works:**

```
Standard development: 100 units, all at market rate
With 15% IZ: 85 market-rate units + 15 affordable units
  Affordable = rent capped at 30% of area median income (AMI)
  For 60% AMI household: ~$1,200/month instead of ~$2,500/month market rate

Developer compensation (to make it financially viable):
  - Density bonus: allowed 15-35% more units than zoning permits
  - Tax abatements: property tax reduction for 10-20 years
  - Expedited permits: skip the 6-12 month review process
  - Parking reduction: fewer required parking spaces
  - Fee waivers: skip impact fees
```

**Game Implementation:**

```rust
pub struct InclusionaryZoningPolicy {
    pub enabled: bool,
    pub affordable_percentage: f32,     // 0.10 to 0.25 (10-25%)
    pub income_threshold: f32,          // % of median income (0.6 = 60% AMI)
    pub density_bonus: f32,             // Extra density allowed (0.15 = 15%)
    pub applies_to_min_units: u32,      // Only applies to buildings with 10+ units
    pub in_lieu_fee: f64,               // Developer can pay fee instead per unit
}

pub fn apply_inclusionary_zoning(
    policy: &InclusionaryZoningPolicy,
    building: &mut Building,
) {
    if !policy.enabled || building.capacity < policy.applies_to_min_units {
        return;
    }

    let affordable_units = (building.capacity as f32 * policy.affordable_percentage).ceil() as u32;
    let bonus_units = (building.capacity as f32 * policy.density_bonus).floor() as u32;

    // Building gets more total capacity (density bonus)
    building.capacity += bonus_units;

    // But some units are affordable (generate less tax revenue)
    // Track this as a component
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct AffordableHousing {
    pub affordable_units: u32,
    pub market_units: u32,
    pub income_cap: f32,       // Maximum income for affordable unit residents
    pub rent_discount: f32,    // Discount vs. market rate (typically 0.4-0.6 = 40-60% discount)
}
```

**Effects on Game Systems:**

| Metric | With IZ (15%) | With IZ (25%) |
|--------|--------------|--------------|
| Total units built | +10% (density bonus) | +20% |
| Market-rate rent | +3-5% (scarcity) | +5-8% |
| Low-income housing | +15% supply | +25% supply |
| Developer profit | -5-10% | -10-20% |
| Homelessness | -20% | -35% |
| Income diversity | +15% | +25% |
| NIMBY opposition | +10% (more density) | +15% |
| Political capital cost | 5 points | 10 points |
| Overall happiness | +2-3 (mixed communities) | +3-5 |

### 6.6 Parking Minimums (Donald Shoup)

Donald Shoup's "The High Cost of Free Parking" (2005, updated 2011) is one of the most influential urban planning books of the 21st century. His central argument: **minimum parking requirements in zoning codes are the single most destructive force in American urbanism**.

**The Problem with Parking Minimums:**

A typical American zoning code requires:
```
Residential: 1.5-2.0 parking spaces per dwelling unit
Office: 1 space per 200-300 sq ft of floor area
Retail: 1 space per 200-250 sq ft
Restaurant: 1 space per 3-4 seats
Hospital: 1 space per 2 beds + 1 per employee
```

Each parking space requires ~300 sq ft (28 sq m) including access lanes. This means:

```
100-unit apartment building:
  150-200 parking spaces required
  150 * 300 sq ft = 45,000 sq ft of parking
  = 1 acre of surface parking, or 1.5 underground levels
  Cost: $5,000-10,000 per surface space, $25,000-60,000 per underground space
  Total parking cost: $750,000 (surface) to $12,000,000 (underground)
  This cost is passed to residents in rent: $50-300/month per unit

50,000 sq ft office building:
  167-250 parking spaces required
  167 * 300 sq ft = 50,100 sq ft of parking
  The parking lot is LARGER than the building itself!
```

**Game Implementation: Parking as a Land-Use Consumer**

```rust
pub struct ParkingPolicy {
    pub residential_ratio: f32,   // Spaces per dwelling unit (default 1.5, Shoup: 0.0)
    pub commercial_ratio: f32,    // Spaces per 1000 sq ft (default 4.0, Shoup: 0.0)
    pub industrial_ratio: f32,    // Spaces per 1000 sq ft (default 2.0)
    pub mode: ParkingMode,
}

pub enum ParkingMode {
    Minimums,       // Traditional: require parking (American default)
    Maximums,       // European approach: cap parking to discourage driving
    MarketRate,     // Shoup's ideal: no requirement, price on-street parking
    Eliminated,     // No parking requirements at all (some cities post-2020)
}

impl ParkingPolicy {
    /// Calculate how many grid cells of parking a building "consumes"
    /// This represents land that cannot be used for other buildings
    pub fn parking_cells_required(&self, building: &Building) -> u32 {
        let spaces = match building.zone_type {
            ZoneType::ResidentialLow | ZoneType::ResidentialHigh => {
                (building.capacity as f32 * self.residential_ratio) as u32
            }
            ZoneType::CommercialLow | ZoneType::CommercialHigh => {
                // Estimate floor area from capacity
                let floor_area_1000 = building.capacity as f32 * 0.3; // ~300 sq ft per worker
                (floor_area_1000 * self.commercial_ratio) as u32
            }
            ZoneType::Industrial => {
                let floor_area_1000 = building.capacity as f32 * 0.5;
                (floor_area_1000 * self.industrial_ratio) as u32
            }
            _ => 0,
        };

        // Each cell provides ~17 surface parking spaces (256 sq m / 15 sq m per space)
        // Underground: multiply by 2-3 levels
        (spaces / 17).max(0)
    }

    /// Effect on development density -- parking minimums reduce effective FAR
    pub fn effective_far_reduction(&self) -> f32 {
        match self.mode {
            ParkingMode::Minimums => 0.3,   // 30% of buildable area consumed by parking
            ParkingMode::Maximums => 0.15,  // 15% (some parking still built)
            ParkingMode::MarketRate => 0.05, // Minimal (only economically viable parking)
            ParkingMode::Eliminated => 0.0, // No reduction
        }
    }
}
```

**Shoup's Three Reforms (as policy options):**

1. **Eliminate parking minimums**: More housing per acre, lower construction costs, higher density
   - Effect: +20% residential capacity, -15% construction cost, -10% car usage
   - Requires: Good transit coverage (Walk Score > 60) or residents complain

2. **Charge market price for on-street parking**: Pricing parking at its true cost ($1-5/hour in real cities)
   - Effect: +5-10% city revenue, -20% parking demand, +15% commercial activity (turnover)
   - Requirement: Parking meters (new infrastructure investment)

3. **Return parking revenue to neighborhoods**: The revenue from parking meters goes to the local district for improvements
   - Effect: +10% district happiness, +5 land value, reduces NIMBY opposition to parking reform

**Game Decision:**

```
Old policy: 1.5 spaces/unit minimum, free on-street parking
  Result: Low density, sprawl, traffic, cheap housing (parking cost hidden in rent)

New policy (Shoup Reform):
  Step 1: Reduce minimums to 0.5 spaces/unit
    -> +30% density, some resident complaints about parking difficulty
  Step 2: Price on-street parking at market rate
    -> +$X/month revenue, -15% traffic, commercial activity improves
  Step 3: Return revenue to districts
    -> +10 happiness in affected districts, reduce NIMBY opposition
  Step 4: Eliminate minimums entirely
    -> Maximum density, requires excellent transit, maximum controversy
```

### 6.7 Floor Area Ratio (FAR) Bonuses and Transfers

FAR bonuses and transfers are sophisticated zoning tools that create market incentives for public benefits.

**FAR Bonus (Incentive Zoning):**

Developers receive extra buildable floor area in exchange for providing public amenities:

```
Base FAR: 6.0 (zone allows 6x lot area as floor space)

Bonuses available:
  +1.0 FAR: Public plaza at ground level (min 2,000 sq ft)
  +2.0 FAR: Affordable housing (20% of units below 60% AMI)
  +0.5 FAR: Transit connection (direct access to subway)
  +0.5 FAR: LEED Gold certification (green building)
  +1.0 FAR: Public school or daycare in building
  +0.3 FAR: Public art installation ($500K+ value)
  +0.5 FAR: Underground parking (removing surface parking)

Maximum with bonuses: 12.0 FAR (2x base)
```

**Implementation:**

```rust
pub struct FARBonus {
    pub bonus_type: FARBonusType,
    pub far_increase: f32,
    pub cost_to_developer: f64,    // What the developer must provide
    pub public_benefit: PublicBenefit,
}

pub enum FARBonusType {
    PublicPlaza,           // Ground-floor public space
    AffordableHousing,     // % of units below market rate
    TransitConnection,     // Direct connection to transit station
    GreenBuilding,         // Environmental certification
    CommunityFacility,     // Public school, daycare, community center
    PublicArt,             // Art installation
    HistoricPreservation,  // Preserving a historic facade
}

pub enum PublicBenefit {
    HappinessBonus(f32),        // +X happiness for nearby residents
    LandValueBonus(u8),         // +X land value for nearby cells
    ServiceCoverage(ServiceType), // Provides service coverage
    AffordableUnits(u32),       // Provides N affordable units
}
```

**Transfer of Development Rights (TDR):**

TDR allows unused FAR from one property (the "sending site") to be transferred to another property (the "receiving site"). This is how cities preserve historic buildings and open space without compensating owners through eminent domain.

```
Sending site: Historic 3-story building on a lot zoned for 10.0 FAR
  Used FAR: 3.0 (the existing building)
  Unused FAR: 7.0 (what could be built but is not)
  The owner SELLS the unused 7.0 FAR to a developer elsewhere

Receiving site: Modern tower on a lot also zoned for 10.0 FAR
  Base FAR: 10.0
  Purchased FAR: +7.0
  Total allowed: 17.0 FAR (70% taller than zoning allows!)

Result: Historic building is preserved (owner compensated by FAR sale)
        Modern tower is taller (developer gets more floor area)
        City gets both preservation AND growth
```

This is the mechanism behind some of New York City's tallest buildings. The air rights above Grand Central Terminal (a landmarked building that cannot be demolished) were sold to developers of nearby One Vanderbilt, allowing it to rise 1,401 feet.

**Game Implementation:**

```rust
pub struct DevelopmentRightsTransfer {
    pub sending_site: (usize, usize),
    pub receiving_site: (usize, usize),
    pub far_transferred: f32,
    pub price: f64,                    // Market price for the transfer
    pub sending_reason: TDRReason,
}

pub enum TDRReason {
    HistoricPreservation,    // Protect a historic building
    OpenSpacePreservation,   // Protect a park or garden
    AgriculturePreservation, // Protect farmland (UGB variant)
    LandmarkProtection,      // Protect a landmark
}

pub fn calculate_tdr_price(
    land_value_sending: u8,
    land_value_receiving: u8,
    far_transferred: f32,
) -> f64 {
    // Price based on the receiving site's land value
    // (the value of the additional floor area at the receiving location)
    let per_far_unit = land_value_receiving as f64 * 50.0;
    per_far_unit * far_transferred as f64
}
```

---

## 7. Implementation Roadmap

This section maps the features described above to the existing Megacity codebase, prioritized by impact and implementation complexity. Each phase builds on the previous one.

### 7.1 Phase 1: Enhanced Zone Types

**Goal**: Expand the current 7-zone system to provide more granular density control without breaking existing systems.

**Current State:**
- `ZoneType` enum in `crates/simulation/src/grid.rs` has 7 variants
- `Building::capacity_for_level()` in `crates/simulation/src/buildings.rs` defines capacity per zone+level
- `building_spawner` in `buildings.rs` iterates full grid per zone type
- `update_zone_demand` in `zones.rs` uses 4 demand channels (R/C/I/O)

**Changes Required:**

1. **Expand `ZoneType` enum** (grid.rs):
```rust
// Current: 7 variants
// Proposed: 12 variants (add SingleFamily, MedDensityRes, LightIndustrial,
//   HeavyIndustrial, MixedUse)
// Migration: ResidentialLow -> SingleFamily OR LowDensityRes based on density
//            Industrial -> LightIndustrial OR HeavyIndustrial based on pollution
```

2. **Add FAR limits per zone** (new field on Cell or computed from zone type):
```rust
impl ZoneType {
    pub fn base_far(self) -> f32 {
        match self {
            ZoneType::SingleFamily => 0.5,
            ZoneType::LowDensityRes => 1.5,
            ZoneType::MedDensityRes => 3.0,
            ZoneType::HighDensityRes => 12.0,
            // ... etc
        }
    }
}
```

3. **Update `capacity_for_level`** to reflect new zone types with realistic capacity values from Section 2.

4. **Update `ZoneDemand`** to add MedDensity and MixedUse demand channels.

5. **Update save/load** (`crates/save/src/serialization.rs`) to handle new enum variants with backward compatibility.

**Estimated effort**: 2-3 days
**Impact**: Medium (more zone variety, better density control)
**Risk**: Low (additive change, existing zones still work)

### 7.2 Phase 2: Form-Based Overlay

**Goal**: Add a transect overlay system that controls building form independent of use.

**Changes Required:**

1. **Add `TransectZone` enum** (new file or in grid.rs):
```rust
pub enum TransectZone {
    None, T1Natural, T2Rural, T3Suburban, T4Urban, T5Center, T6Core,
}
```

2. **Add transect field to `Cell`** (grid.rs):
```rust
pub struct Cell {
    // ... existing fields ...
    pub transect: TransectZone,  // NEW: form-based overlay
}
```

3. **Modify `building_spawner`** to check transect constraints:
   - `max_level_for_far()` limits building level based on transect
   - `TransectZone::max_stories()` caps visual height

4. **Add transect painting tool** (UI changes in `crates/ui/src/toolbar.rs`):
   - New toolbar section for transect zones
   - Visual overlay showing transect boundaries
   - Auto-suggestion: "Your downtown has T6 density but no T5 buffer zone"

5. **Update rendering** (`crates/rendering/src/overlay.rs`):
   - New overlay mode showing transect zones with color coding
   - T1=green, T2=light green, T3=yellow, T4=orange, T5=red, T6=dark red

**Estimated effort**: 3-5 days
**Impact**: High (enables mixed density neighborhoods, realistic urban gradients)
**Risk**: Medium (new field on Cell increases memory, save format change)

**Memory impact**: Adding `TransectZone` (1 byte) to `Cell` increases grid memory by 256*256 = 65,536 bytes = 64 KB. Negligible.

### 7.3 Phase 3: Advanced Mechanics

**Goal**: Implement NIMBY, parking, inclusionary zoning, and building variety systems.

**Sub-phases:**

**3a. Building Variety Pool (1-2 days)**
- Create `BuildingTemplate` struct with visual variant info
- Create `BuildingPool` with weighted random selection
- Modify `building_spawner` to select from pools instead of fixed capacity
- Add `BuildingAppearance` component for rendering variety
- Update `crates/rendering/src/building_meshes.rs` to use appearance data

**3b. NIMBY/YIMBY System (2-3 days)**
- Add `NimbyPressure` resource
- Add `PoliticalCapital` resource
- Create `compute_nimby_intensity` system (runs when zone changes happen)
- Add zone change delay mechanic (proposals sit for 30-90 days before taking effect)
- UI: Show NIMBY resistance level when hovering over upzone action
- Event system: "Neighborhood opposition blocks rezoning" / "Community supports new housing"

**3c. Parking Policy (1-2 days)**
- Add `ParkingPolicy` resource to `Policies`
- Modify `building_spawner` to account for parking land consumption
- Three parking modes: Minimums (default), Maximums, Eliminated
- Effect on effective density: Minimums reduce capacity by 30%, Eliminated increases by 30%

**3d. Inclusionary Zoning (1-2 days)**
- Add `InclusionaryZoningPolicy` resource
- Add `AffordableHousing` component
- Modify `building_spawner` to apply IZ to buildings above threshold size
- Track affordable vs. market-rate occupancy separately
- UI: Show affordable unit count in building info panel

**Estimated effort**: 6-9 days total
**Impact**: Very High (adds depth, player decisions, emergent narrative)
**Risk**: Medium (new systems interact with existing happiness, demand, economy)

### 7.4 Phase 4: Neighborhood Scoring

**Goal**: Implement Walk Score, 15-Minute City scoring, and Neighborhood Quality Index.

**Changes Required:**

1. **Walk Score System (2-3 days)**
   - Add `WalkScore` resource (per-district scores)
   - BFS walking distance calculation from district center
   - 9-category scoring with distance decay
   - Intersection density modifier
   - Update every 200 ticks (performance-safe with district sampling)

2. **15-Minute City Score (1-2 days)**
   - Add `FifteenMinuteScore` resource (per-district)
   - 6-category scoring (work, supply, care, learn, enjoy, transit)
   - Reuse BFS distances from Walk Score
   - Display as overlay map (green = 15-min city, red = car-dependent)

3. **Neighborhood Quality Index (1 day)**
   - Combine Walk Score + 15-Min + service coverage + safety + environment
   - Add to district info panel (`crates/ui/src/info_panel.rs`)
   - Rating bands with descriptive labels

4. **NQI Effects on Game Systems (1-2 days)**
   - Wire NQI to land value computation (`land_value.rs`)
   - Wire NQI to demand computation (`zones.rs`)
   - Wire NQI to immigration rates (`immigration.rs`)
   - Wire NQI to happiness computation (`happiness.rs`)

**Estimated effort**: 5-8 days
**Impact**: Very High (gives player actionable feedback on neighborhood design)
**Risk**: Medium (BFS computation cost, but mitigated by district sampling)

---

## Appendix A: Density Reference Table

Quick reference for converting between common density metrics and Megacity grid cells.

```
1 grid cell = 16m x 16m = 256 sq m = 2,755 sq ft = 0.063 acres
1 acre = 4,047 sq m = 15.8 cells
1 hectare = 10,000 sq m = 39.1 cells
1 city block (US standard) = 100m x 100m = 6.25 x 6.25 cells = ~39 cells
1 Manhattan block = 274m x 76m = 17.1 x 4.75 cells = ~81 cells
1 Portland short block = 61m x 61m = 3.8 x 3.8 cells = ~14.5 cells

Full Megacity grid:
  256 x 256 cells = 65,536 cells
  = 4,096m x 4,096m = 4.1 km x 4.1 km = 2.55 mi x 2.55 mi
  = 16.78 sq km = 6.48 sq mi = 4,145 acres
  = ~1,672 US standard city blocks
```

## Appendix B: Real-World City Density Comparisons

| City / Neighborhood | Population Density | Units/Acre | FAR | Megacity Equivalent (per cell) |
|---------------------|-------------------|------------|-----|-------------------------------|
| US Suburban | 2,000/sq mi | 4 | 0.3 | 2 residents |
| US Small City | 4,000/sq mi | 8 | 0.6 | 4 residents |
| Portland, OR | 4,800/sq mi | 10 | 0.8 | 5 residents |
| Seattle | 8,400/sq mi | 15 | 1.2 | 8 residents |
| San Francisco | 18,600/sq mi | 25 | 2.0 | 18 residents |
| Brooklyn, NY | 37,000/sq mi | 50 | 3.5 | 37 residents |
| Manhattan (avg) | 72,000/sq mi | 100 | 6.0 | 72 residents |
| Midtown Manhattan | 150,000+/sq mi | 200+ | 12.0 | 150 residents (workers, not residents) |
| Hong Kong (Mong Kok) | 340,000/sq mi | 400+ | 15.0+ | 340 residents |
| Dhaka, Bangladesh | 115,000/sq mi | 150 | 5.0+ | 115 residents |
| Tokyo (Shinjuku) | 45,000/sq mi | 80 | 8.0 | 45 residents |
| Paris (intra-muros) | 55,000/sq mi | 70 | 4.0-5.0 | 55 residents |
| Barcelona (Eixample) | 90,000/sq mi | 120 | 5.0-6.0 | 90 residents |

**Scaling Note:** Megacity's full map (4.1 km x 4.1 km) is roughly the size of:
- Manhattan below 96th Street (4.0 km x 3.4 km)
- Paris 1st-7th arrondissements
- Central Tokyo (Chiyoda + Chuo + Minato wards)
- Barcelona's Eixample district (3.5 km x 2.5 km)

At Manhattan-level density (72 residents/cell average), the full Megacity map could theoretically house 72 * 65,536 = 4.7 million residents. At suburban density (2 residents/cell), it would house 131,000.

## Appendix C: Zoning Code Quick Reference (US Standard)

```
Zone Code    Name                          FAR     Height    Use Summary
---------    ----                          ---     ------    -----------
R-1          Single Family Residential     0.3-0.5 35 ft    Detached houses only
R-2          Two-Family Residential        0.5-0.8 35 ft    R-1 + duplexes
R-3          Multi-Family Low              0.8-1.5 45 ft    R-2 + small apartments
R-4          Multi-Family Medium           1.5-3.0 65 ft    R-3 + mid-rise apartments
R-5          Multi-Family High             3.0-10.0 none    R-4 + towers
C-1          Neighborhood Commercial       0.5-1.5 35 ft    Small retail, services
C-2          Community Commercial          1.0-3.0 65 ft    General retail, offices
C-3          Regional Commercial           2.0-5.0 none     Malls, big box, auto dealers
C-4          Central Business District     5.0-15.0 none    Office towers, hotels
C-5          Commercial Recreation         0.5-1.0 35 ft    Entertainment, sports
M-1          Light Industrial              0.5-2.0 45 ft    Warehouses, clean manufacturing
M-2          Heavy Industrial              0.5-2.0 65 ft    Factories, processing plants
M-3          Extractive Industrial         0.1-0.3 none     Mining, quarries
PD           Planned Development           varies  varies   Custom plan (negotiated)
OS           Open Space                    0.0     none     Parks, recreation, conservation
AG           Agricultural                  0.05    35 ft    Farming, rural residential
MU-1         Mixed Use Low                 1.0-2.5 45 ft    Retail ground floor + apartments
MU-2         Mixed Use High                3.0-8.0 none     Retail + office + residential
TOD          Transit Overlay District      +2.0    +30 ft   Extra density near transit
HP           Historic Preservation Overlay varies  existing  Cannot exceed existing height
```

## Appendix D: Japanese Zone Comparison Table

```
Zone    Name                    Residential  Small Shop  Office  Factory  Height  FAR
----    ----                    -----------  ----------  ------  -------  ------  ---
1       Excl. Low-Res 1         Yes          <50 sq m    No      No       10m     0.5-1.0
2       Excl. Low-Res 2         Yes          <150 sq m   No      No       10m     0.5-1.0
3       Mid/High-Res 1          Yes          <500 sq m   Yes*    No       Slant   1.0-2.0
4       Mid/High-Res 2          Yes          <1500 sq m  Yes     No       Slant   1.0-2.0
5       Residential 1           Yes          <3000 sq m  Yes     <50 sq m Slant   1.0-3.0
6       Residential 2           Yes          <10000 sq m Yes     <50 sq m Slant   1.0-3.0
7       Neighborhood Comm.      Yes          No limit    Yes     <50 sq m None*   1.5-4.0
8       Commercial              Yes          No limit    Yes     Light    None    2.0-13.0
9       Quasi-Industrial        Yes          No limit    Yes     Yes      None    2.0-4.0
10      Industrial              Limited**    No limit    Yes     Yes      None    1.0-4.0
11      Excl. Industrial        No           No limit    Yes     Yes      None    1.0-4.0

*  Slant = shadow-angle restriction (buildings must step back to avoid casting shadows)
** Limited = no schools, hospitals, or hotels
```

## Appendix E: Data Structures Summary

All proposed new data structures and their estimated memory costs at full grid size (256x256 = 65,536 cells):

```
Structure                    Per-Cell Size    Total Memory    Purpose
---------                    -------------    ------------    -------
TransectZone (on Cell)       1 byte           64 KB           Form-based overlay
WalkScore (per district)     4 bytes          1 KB (256 districts) Walkability index
FifteenMinuteScore           28 bytes         7 KB (256 districts) 15-min city score
NimbyPressure (per district) 4 bytes          1 KB            NIMBY intensity
ParkingCells (per building)  4 bytes          ~100 KB (25K bldgs) Parking land use
BuildingQuality              12 bytes         ~300 KB (25K bldgs) Quality score + trend
BuildingAppearance           16 bytes         ~400 KB (25K bldgs) Visual variation
AffordableHousing            16 bytes         ~100 KB (6K bldgs)  IZ tracking
HistoricDesignation          16 bytes         ~20 KB (1K bldgs)   Preservation
UrbanGrowthBoundary          1 bit/cell       8 KB             UGB mask

Total estimated additional memory: ~1.0 MB
(Negligible compared to existing grid: 65,536 cells * ~40 bytes/cell = 2.5 MB)
```

---

*This document was produced through analysis of real-world urban planning systems, zoning codes, and existing city builder implementations. All dimensional data is based on standard North American and international urban planning references. Code examples are written for Megacity's Bevy ECS architecture with Rust.*
