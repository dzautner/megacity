# Economic Simulation: Deep Implementation Reference

This document provides implementation-ready detail for Megacity's economic simulation layer.
It covers municipal finance, land value modeling, supply/demand dynamics, labor markets,
and economic cycles -- with real-world data, formulas, pseudocode, and lessons from
other city-builder and economy games.

**Cross-references to existing codebase:**
- `crates/simulation/src/economy.rs` -- CityBudget, tax collection
- `crates/simulation/src/budget.rs` -- ExtendedBudget, Loans, ZoneTaxRates, ServiceBudgets
- `crates/simulation/src/land_value.rs` -- LandValueGrid (u8 per cell)
- `crates/simulation/src/market.rs` -- MarketPrices, MarketEvents, supply/demand cycles
- `crates/simulation/src/production.rs` -- CityGoods, production chains

---

## 1. Municipal Finance

### 1.1 Property Tax Assessment

Property tax is the single largest revenue source for American municipalities, typically
accounting for 30-50% of total local government revenue. In a city builder, it is the
primary lever the player controls.

#### Real-World Assessment Process

1. **Market value determination.** An assessor estimates the fair market value (FMV) of
   each parcel. Methods include:
   - **Sales comparison approach:** Compare to recent sales of comparable properties.
   - **Cost approach:** Land value + depreciated replacement cost of improvements.
   - **Income approach:** For commercial properties, capitalize net operating income.

2. **Assessment ratio.** Most jurisdictions assess at a fraction of market value.
   Common ratios:
   - Residential: 10-33% of FMV (e.g., New York City: 6% for Class 1 residential)
   - Commercial: 25-45% of FMV (e.g., NYC: 45% for Class 4 commercial)
   - Industrial: 25-45% of FMV
   - Vacant land: Often assessed at 100% of FMV to discourage speculation

3. **Assessed value = Market Value x Assessment Ratio**

#### Millage Rates

A "mill" = $1 of tax per $1,000 of assessed value (= 0.1%).

Real millage rates by city type:

| City Type              | Typical Millage | Effective Rate on FMV | Example               |
|------------------------|----------------:|----------------------:|------------------------|
| Low-tax suburb         |       15-25     |        0.5-1.0%       | Scottsdale, AZ         |
| Average suburb         |       25-50     |        1.0-2.0%       | Naperville, IL         |
| Midsized city          |       40-80     |        1.5-3.0%       | Charlotte, NC          |
| Large city             |       60-120    |        2.0-4.0%       | Philadelphia, PA       |
| Distressed city        |      100-200+   |        3.5-7.0%       | Detroit, MI            |
| Special taxing district|       +5-30     |        +0.2-1.0%      | Fire, library, parks   |

**Note:** High millage on low assessed values can produce the same revenue as low millage
on high assessed values. Detroit has very high millage (~67 mills) but extremely low
property values, resulting in relatively low per-parcel revenue.

#### Implementation for Megacity

```
// Current system: flat tax_rate on citizen count
// Proposed: property tax on assessed building value

struct PropertyTaxConfig {
    residential_millage: f32,   // default 30.0 mills
    commercial_millage: f32,    // default 40.0 mills
    industrial_millage: f32,    // default 35.0 mills
    assessment_ratio: f32,      // default 0.25 (25% of market value)
}

fn calculate_property_tax(building: &Building, land_value: u8, config: &PropertyTaxConfig) -> f64 {
    // Step 1: Estimate market value
    // Base value from building type + quality + size
    let improvement_value = building.construction_cost as f64 *
        building.quality_multiplier() *
        (1.0 - building.age_depreciation());

    // Land value component (from LandValueGrid, scaled to dollar equivalent)
    let land_dollar_value = land_value as f64 * 100.0; // $0-$25,500 per cell

    let market_value = improvement_value + land_dollar_value;

    // Step 2: Apply assessment ratio
    let assessed_value = market_value * config.assessment_ratio as f64;

    // Step 3: Apply millage rate
    let millage = match building.zone_type {
        ZoneType::ResidentialLow | ZoneType::ResidentialMed | ZoneType::ResidentialHigh
            => config.residential_millage,
        ZoneType::CommercialLow | ZoneType::CommercialHigh
            => config.commercial_millage,
        ZoneType::Industrial
            => config.industrial_millage,
        ZoneType::Office
            => config.commercial_millage, // same as commercial
    };

    // Tax = assessed_value * millage / 1000
    assessed_value * millage as f64 / 1000.0
}
```

#### Tax Rate Response Curve

Tax rates have diminishing and eventually negative returns. Higher rates drive out
residents and businesses. Real-world research suggests the Laffer curve peaks
for property taxes around 3-4% effective rate for residential and 5-6% for commercial.

```
fn tax_satisfaction_modifier(effective_rate: f32) -> f32 {
    // Citizens become unhappy as rates rise
    // Below 1%: slight happiness bonus (low taxes!)
    // 1-2%: neutral
    // 2-3%: mild unhappiness
    // 3-5%: significant unhappiness, emigration pressure
    // 5%+: severe unhappiness, rapid population loss
    if effective_rate < 0.01 {
        0.05  // small bonus
    } else if effective_rate < 0.02 {
        0.0   // neutral
    } else if effective_rate < 0.03 {
        -0.05 * ((effective_rate - 0.02) / 0.01)
    } else if effective_rate < 0.05 {
        -0.05 - 0.15 * ((effective_rate - 0.03) / 0.02)
    } else {
        -0.20 - 0.30 * ((effective_rate - 0.05) / 0.05).min(1.0)
    }
}
```

**SimCity's tax curve for reference:**
SimCity 4 used a piecewise function where residential demand dropped by roughly
1 point per 0.5% above the 7% rate, and commercial/industrial dropped faster at
2 points per 0.5% above 7%. Below 7%, demand increases, creating a natural
equilibrium. The "sweet spot" was around 7-9% depending on city services.

### 1.2 Municipal Bonds

Municipal bonds ("munis") are how real cities fund capital projects. Two main types:

#### General Obligation (GO) Bonds

- Backed by the **full faith and credit** (taxing power) of the issuer
- Used for: schools, city halls, parks, general infrastructure
- Voter approval typically required
- Lower interest rates because they are safer (backed by tax revenue)
- Real rates: 2.0-5.5% depending on credit rating and term

#### Revenue Bonds

- Backed only by revenue from the specific project they fund
- Used for: toll roads, water systems, airports, stadiums, parking garages
- No voter approval needed (no general tax pledge)
- Higher interest rates (riskier -- if the project fails, bondholders lose)
- Real rates: 3.0-7.5%

#### Credit Ratings and Interest Rates

| Rating (S&P) | Description          | Typical GO Bond Rate | Revenue Bond Rate |
|---------------|----------------------|---------------------:|------------------:|
| AAA           | Highest quality      |           2.5-3.0%   |        3.0-3.5%   |
| AA+/AA/AA-    | High quality         |           2.8-3.5%   |        3.5-4.5%   |
| A+/A/A-       | Upper medium         |           3.5-4.5%   |        4.5-5.5%   |
| BBB+/BBB/BBB- | Medium (investment)  |           4.5-5.5%   |        5.5-7.0%   |
| BB+ and below | Speculative ("junk") |           6.0-9.0%   |        7.0-12.0%  |

#### What Determines Credit Rating in a Game

```
fn calculate_credit_rating(city: &CityStats) -> CreditRating {
    let mut score: f32 = 50.0; // Start at midpoint (BBB range)

    // Debt-to-revenue ratio (most important factor)
    // Under 1.0 is excellent, over 3.0 is dangerous
    let debt_ratio = city.total_debt / city.annual_revenue.max(1.0);
    score += match debt_ratio {
        r if r < 0.5 => 20.0,
        r if r < 1.0 => 15.0,
        r if r < 1.5 => 10.0,
        r if r < 2.0 => 0.0,
        r if r < 3.0 => -10.0,
        r if r < 5.0 => -20.0,
        _ => -30.0,
    };

    // Debt service coverage ratio (DSCR)
    // Revenue available for debt service / annual debt service
    // Above 2.0 is strong, below 1.0 means can't cover payments
    let dscr = (city.annual_revenue - city.operating_expenses) /
               city.annual_debt_service.max(1.0);
    score += match dscr {
        r if r > 3.0 => 15.0,
        r if r > 2.0 => 10.0,
        r if r > 1.5 => 5.0,
        r if r > 1.0 => 0.0,
        r if r > 0.8 => -10.0,
        _ => -25.0,
    };

    // Fund balance (reserves as % of expenditures)
    // 15-20% is considered healthy
    let reserve_ratio = city.treasury / city.annual_expenses.max(1.0);
    score += match reserve_ratio {
        r if r > 0.25 => 10.0,
        r if r > 0.15 => 5.0,
        r if r > 0.08 => 0.0,
        r if r > 0.03 => -5.0,
        _ => -15.0,
    };

    // Population trend (growing = good, shrinking = bad)
    score += city.population_growth_rate * 50.0; // +/- a few points

    // Economic diversity (single-industry = risky)
    score += city.economic_diversity_index * 5.0; // 0-1 scale

    // Convert score to rating
    match score as i32 {
        s if s >= 85 => CreditRating::AAA,
        s if s >= 75 => CreditRating::AAPlus,
        s if s >= 65 => CreditRating::AA,
        s if s >= 55 => CreditRating::AAMinus,
        s if s >= 48 => CreditRating::APlus,
        s if s >= 42 => CreditRating::A,
        s if s >= 36 => CreditRating::AMinus,
        s if s >= 30 => CreditRating::BBBPlus,
        s if s >= 24 => CreditRating::BBB,
        s if s >= 18 => CreditRating::BBBMinus,
        s if s >= 10 => CreditRating::BBPlus,
        _ => CreditRating::BB,
    }
}
```

#### Bankruptcy: Real-World Cases

**Chapter 9 municipal bankruptcy** is extremely rare. Key cases:

**Detroit, Michigan (2013):**
- Population fell from 1.85M (1950) to 700K (2013)
- Lost 60% of its tax base over 60 years
- $18 billion in debt/unfunded liabilities
- Trigger: Auto industry collapse, white flight, decades of mismanagement
- Pensioners received 95.5 cents on the dollar (after "Grand Bargain")
- GO bondholders received 74 cents on the dollar
- Recovery took 17 months in bankruptcy
- **Game lesson:** Population loss spirals. Fewer people -> less revenue -> cut services
  -> more people leave -> less revenue. This is the "death spiral" mechanic.

**Stockton, California (2012):**
- Population 300K, growing fast during housing boom
- City gave lavish pension/health benefits during boom years
- Housing crash destroyed property tax revenue (values fell 60%)
- $900M in debt (much in unfunded pension obligations)
- Emerged 2015 with restructured pension obligations
- **Game lesson:** Over-leveraging during booms. Revenue projections that assume
  continuous growth are dangerous.

**San Bernardino, California (2012):**
- Similar to Stockton: pension obligations + housing crash
- Added factor: Walmart and big-box retail drew sales tax away from downtown
- **Game lesson:** Economic base matters. Single-source dependency is fragile.

#### Implementation: Bond System

```
enum BondType {
    GeneralObligation {
        purpose: String,
        // Requires voter approval (player confirms)
    },
    Revenue {
        revenue_source: Entity, // The infrastructure generating revenue
        projected_annual_revenue: f64,
    },
}

struct MunicipalBond {
    bond_type: BondType,
    face_value: f64,        // Total amount raised
    coupon_rate: f32,       // Annual interest rate (set by credit rating)
    maturity_years: u32,    // 10, 20, or 30 year terms
    remaining_principal: f64,
    annual_payment: f64,    // Calculated at issuance
    issued_day: u32,        // Game day of issuance
}

// Interest rate = base_rate_for_rating + term_premium + type_premium
fn bond_interest_rate(rating: CreditRating, bond_type: &BondType, term_years: u32) -> f32 {
    let base = match rating {
        CreditRating::AAA => 2.5,
        CreditRating::AA => 3.0,
        CreditRating::A => 3.8,
        CreditRating::BBB => 5.0,
        CreditRating::BB => 7.0,
        _ => 9.0,
    };

    // Longer terms = higher rates (yield curve)
    let term_premium = match term_years {
        t if t <= 10 => 0.0,
        t if t <= 20 => 0.5,
        _ => 1.0,
    };

    // Revenue bonds carry a premium
    let type_premium = match bond_type {
        BondType::GeneralObligation { .. } => 0.0,
        BondType::Revenue { .. } => 1.0,
    };

    (base + term_premium + type_premium) / 100.0
}
```

### 1.3 Tax Increment Financing (TIF) Districts

TIF is the most common local economic development tool in the US. It is controversial
but widespread (all 50 states authorize some form of TIF). Here is exactly how it works:

#### Step-by-Step TIF Process

1. **Designation:** The city designates a geographic area as a TIF district. The area
   must typically meet a "blight" test (deteriorated, underutilized, or lacking
   infrastructure). In practice, many cities define blight broadly.

2. **Freeze the base:** At the moment of designation, the total assessed value of all
   property in the district is frozen. This is the **"base assessed value"** (BAV).

3. **Development occurs:** The city (or private developers with city incentives) invest
   in the district -- new roads, utilities, buildings, streetscaping. Property values
   rise as a result.

4. **Increment captured:** Any increase in assessed value above the BAV is the
   **"tax increment."** Property taxes on the increment flow to the TIF fund, not the
   general fund or other taxing bodies (school districts, counties, etc.).

5. **Increment funds projects:** The captured increment pays back the infrastructure
   investment, typically through bonds issued against projected future increment revenue.

6. **Expiration:** TIF districts expire after 15-30 years (varies by state).
   When expired, all property tax revenue flows back to normal taxing bodies at the
   new (higher) assessed values.

#### TIF Financials Example

```
Suppose a blighted 4-block area:
  - BAV at designation:         $5,000,000
  - Millage rate:               40 mills
  - Base year tax revenue:      $5,000,000 * 40/1000 = $200,000/year

After TIF investment of $3M in infrastructure:
  - Year 5 assessed value:     $12,000,000
  - Increment:                  $12,000,000 - $5,000,000 = $7,000,000
  - Increment revenue:          $7,000,000 * 40/1000 = $280,000/year
  - Base revenue (to general fund): $200,000/year (unchanged)
  - TIF captures: $280,000/year

At this rate, $3M investment is repaid in ~11 years.
After TIF expires, entire $480,000/year flows to general fund.
```

#### TIF Implementation for Megacity

```
struct TIFDistrict {
    cells: Vec<(usize, usize)>,   // Grid cells in the district
    base_assessed_value: f64,     // Frozen at creation
    creation_day: u32,
    expiration_day: u32,          // creation + (20 years * 365)
    total_investment: f64,        // Infrastructure spent
    remaining_debt: f64,          // Investment not yet repaid
    cumulative_increment: f64,    // Total increment captured so far
    is_active: bool,
}

fn update_tif_districts(
    tif_districts: &mut Vec<TIFDistrict>,
    land_value: &LandValueGrid,
    grid: &WorldGrid,
    millage: f32,
    current_day: u32,
    treasury: &mut f64,
) {
    for district in tif_districts.iter_mut() {
        if current_day >= district.expiration_day {
            district.is_active = false;
            continue;
        }

        // Calculate current total assessed value of district cells
        let current_av: f64 = district.cells.iter()
            .map(|&(x, y)| {
                let lv = land_value.get(x, y) as f64 * 100.0;
                let building_value = grid.get(x, y).building_value();
                (lv + building_value) * 0.25 // assessment ratio
            })
            .sum();

        // Increment = current - base (only positive increments matter)
        let increment = (current_av - district.base_assessed_value).max(0.0);
        let increment_revenue = increment * millage as f64 / 1000.0;

        // Apply increment to debt repayment
        let payment = increment_revenue.min(district.remaining_debt);
        district.remaining_debt -= payment;
        district.cumulative_increment += increment_revenue;

        // Any excess increment after debt repayment goes to general fund
        let excess = increment_revenue - payment;
        *treasury += excess;
    }
}
```

#### TIF Controversy (Design Consideration)

In reality, TIF often draws criticism because:
- It diverts revenue from schools and other taxing bodies
- The "but-for" test is often a fiction (development might have happened anyway)
- Can be used as a slush fund for politically connected developers

For the game, TIF should be a powerful but double-edged tool:
- **Pro:** Dramatically accelerates development in blighted areas
- **Con:** Reduces general fund revenue for the TIF duration
- **Risk:** If the area doesn't develop (wrong location, bad economy), you've invested
  infrastructure money you can't recoup

### 1.4 Impact Fees and Permit Fees

#### Impact Fees

Charged to developers when new construction generates demand for public infrastructure.
The legal standard is "rational nexus" -- the fee must be proportional to the impact.

| Impact Fee Type          | Typical Range per Unit | What It Funds                    |
|--------------------------|----------------------:|----------------------------------|
| Water connection         |         $2,000-8,000  | Water treatment plant capacity   |
| Sewer connection         |         $2,000-6,000  | Wastewater treatment capacity    |
| Transportation           |         $1,500-8,000  | Road widening, intersections     |
| Parks and recreation     |           $500-4,000  | Park land acquisition, facilities|
| Fire protection          |           $300-1,500  | Fire station capacity            |
| Schools                  |         $2,000-15,000 | School construction              |
| Storm drainage           |           $500-3,000  | Drainage infrastructure          |

**Total impact fees** in high-growth areas can reach $20,000-$60,000 per residential unit.
This is a significant cost that gets passed to homebuyers, increasing housing prices.

#### Permit Fees

| Permit Type              | Typical Fee    | Notes                                |
|--------------------------|---------------:|--------------------------------------|
| Building permit (res)    |   $500-2,000   | Based on construction value          |
| Building permit (comm)   | $2,000-15,000  | Based on construction value & sqft   |
| Zoning variance          |   $500-2,500   | Request to deviate from zoning code  |
| Rezoning application     | $1,000-5,000   | Change of zone designation           |
| Site plan review         |   $500-3,000   | Engineering review of plans          |
| Demolition permit        |   $200-1,000   | Environmental review included        |
| Grading permit           |   $500-2,000   | Earthwork approval                   |
| Certificate of occupancy |   $100-500     | Final inspection                     |

#### Implementation

```
fn calculate_impact_fee(building: &Building, city: &CityStats) -> f64 {
    let base_fee = match building.zone_type {
        ZoneType::ResidentialLow => 8_000.0,
        ZoneType::ResidentialMed => 6_000.0,  // Per unit, but more units
        ZoneType::ResidentialHigh => 4_500.0, // Per unit, many units
        ZoneType::CommercialLow => 12_000.0,
        ZoneType::CommercialHigh => 20_000.0,
        ZoneType::Industrial => 15_000.0,
        ZoneType::Office => 18_000.0,
    };

    // Multiply by number of units/capacity
    let units = building.capacity as f64;
    let total = base_fee * units;

    // Reduce in high-growth mode to encourage development
    // Increase in infrastructure-constrained cities
    let modifier = if city.infrastructure_utilization > 0.9 {
        1.5 // Infrastructure strained, charge more
    } else if city.population_growth_rate > 0.03 {
        0.7 // Fast growth, reduce to keep attracting development
    } else {
        1.0
    };

    total * modifier
}

fn calculate_permit_fee(construction_value: f64) -> f64 {
    // Typical formula: base + per-$1000 of construction value
    let base = 250.0;
    let per_thousand = 8.50;
    base + (construction_value / 1000.0) * per_thousand
}
```

### 1.5 Other Revenue Sources

Real municipalities rely on a diversified revenue base:

| Revenue Source          | % of Typical Budget | Notes                              |
|-------------------------|--------------------:|------------------------------------|
| Property tax            |            30-50%   | Primary, most stable               |
| Sales tax               |            10-25%   | Volatile, tracks consumer spending |
| Income/wage tax         |             5-15%   | Only some cities (NYC, Phila, etc) |
| Utility charges         |            10-20%   | Water, sewer, trash, electric      |
| Intergovernmental       |            10-20%   | State/federal grants               |
| Fees and permits        |             3-8%    | Building permits, impact fees      |
| Fines and forfeitures   |             1-3%    | Parking tickets, code violations   |
| Franchise fees          |             1-3%    | Cable, telecom, gas utilities      |
| Hotel/lodging tax       |             1-5%    | Tourism-dependent cities higher    |
| Real estate transfer tax|             1-3%    | Charged on property sales          |

For Megacity, the most impactful to implement:
1. Property tax (already exists, needs depth)
2. Sales tax (tied to commercial zone revenue)
3. Utility charges (tied to water/power infrastructure)
4. Impact/permit fees (one-time on construction)
5. Fines (from police, code enforcement -- ties to services)

---

## 2. Land Value: The Core Mechanic

Land value is arguably the single most important variable in a city builder. It determines
property tax revenue, building density, citizen wealth distribution, neighborhood character,
and -- critically -- what kind of development the market will produce. Every major city
builder (SimCity, Cities: Skylines, Tropico) has land value at its core, though most
implement it simplistically.

The existing `LandValueGrid` in `land_value.rs` uses a `u8` per cell with additive modifiers
from water proximity, industrial zones, pollution, parks, and services. This section
describes how to make land value a rich, hedonic pricing-based system.

### 2.1 Hedonic Pricing Model

The hedonic pricing model (Rosen, 1974) decomposes property value into the implicit
prices of its individual characteristics. The standard form is:

```
ln(Price) = beta_0
          + beta_1 * Structural_Characteristics
          + beta_2 * Neighborhood_Characteristics
          + beta_3 * Accessibility
          + beta_4 * Environmental_Quality
          + epsilon
```

In practice, researchers use semi-log models where the dependent variable is the
natural log of price, making coefficients interpretable as percentage effects.

#### Approximate Weights from Literature

The following weights are synthesized from hundreds of hedonic studies across US
metropolitan areas. These are approximate percentage impacts on property value:

**Structural Characteristics (explain ~40-50% of value variance):**

| Factor                      | Impact on Value    | Notes                           |
|-----------------------------|-------------------:|---------------------------------|
| Living area (per 100 sqft)  |         +3 to +5%  | Diminishing returns above 3000sf|
| Lot size (per 1000 sqft)    |         +1 to +3%  | Stronger in suburbs             |
| Bedrooms (each)             |         +2 to +5%  | Diminishing after 4             |
| Bathrooms (each)            |         +5 to +8%  | Strongest marginal factor       |
| Age of building (per year)  |        -0.3 to -1% | Nonlinear, stabilizes after 50yr|
| Garage (2-car)              |         +5 to +10% | Regional variation              |
| Building quality (grade)    |        +10 to +30%  | Excellent vs average            |
| Stories (each additional)   |         +3 to +8%  | In high-density zones           |

**Neighborhood Characteristics (explain ~25-35% of variance):**

| Factor                      | Impact on Value    | Notes                           |
|-----------------------------|-------------------:|---------------------------------|
| School quality (per 1 SD)   |         +5 to +10% | Strongest neighborhood effect   |
| Crime rate (per 1 SD)       |        -3 to -10%  | Violent crime stronger than prop.|
| Median neighborhood income  |         +5 to +15% | Proxy for many amenities        |
| % owner-occupied            |         +3 to +8%  | Stability signal                |
| Tree canopy coverage        |         +2 to +5%  | Visual amenity                  |
| Historic district           |         +5 to +15% | Prestige + zoning protection    |

**Accessibility (explain ~15-25% of variance):**

| Factor                        | Impact on Value    | Notes                         |
|-------------------------------|-------------------:|-------------------------------|
| Distance to CBD (per mile)    |        -1 to -5%   | Stronger in transit cities    |
| Distance to rail station      |    +5 to +25%      | Within 1/4 mile               |
| Highway access (within 1 mi)  |    +3 to +8%       | But noise disamenity if <500ft|
| Bus frequency (per route)     |    +1 to +3%       | Weaker than rail              |
| Walk score (per 10 points)    |    +1 to +5%       | Strongest in urban areas      |
| Commute time (per 10 min)     |   -3 to -8%        | Wage-commute tradeoff         |

**Environmental Quality (explain ~5-15% of variance):**

| Factor                        | Impact on Value    | Notes                         |
|-------------------------------|-------------------:|-------------------------------|
| Water view (ocean/lake)       |   +20 to +100%     | Extremely location-dependent  |
| Water proximity (< 500ft)     |   +10 to +25%      | Without view premium          |
| Park adjacency                |    +5 to +15%      | Within 1/4 mile               |
| Air quality (per 1 SD worse)  |   -2 to -6%        | PM2.5, ozone                  |
| Noise (per 10 dB above 55)    |   -2 to -5%        | Highway, airport, industrial  |
| Power plant/landfill (< 1mi)  |  -10 to -25%       | LULU (locally unwanted land use)|
| Flood zone (100-year)         |   -5 to -15%       | After flood events, larger    |

### 2.2 Implementation: Hedonic Land Value Calculator

The key insight for a game implementation is that we do NOT need to simulate individual
property transactions. Instead, we compute an **equilibrium land value** for each cell
based on its hedonic characteristics, then let that value influence what gets built.

```
// Replace the u8 LandValueGrid with a richer model
struct LandValueCell {
    raw_value: f32,        // Computed hedonic value (arbitrary units, 0-1000+)
    smoothed_value: f32,   // After spatial smoothing
    trend: f32,            // Change per tick (for UI display)
    zoned_value: f32,      // Value considering highest/best use under current zoning
}

struct HedonicWeights {
    // Accessibility
    w_cbd_distance: f32,       // -0.03 per cell of distance (log decay)
    w_transit_proximity: f32,  //  0.15 within radius, decaying
    w_road_access: f32,        //  0.05 adjacent to road
    w_highway_access: f32,     //  0.08 within 10 cells of highway

    // Amenities
    w_park: f32,               //  0.10 per park within radius 8
    w_water: f32,              //  0.20 adjacent, 0.10 within 3 cells
    w_school: f32,             //  0.08 per school within radius 12
    w_commercial_prox: f32,    //  0.05 per commercial building within radius 6

    // Disamenities
    w_pollution: f32,          // -0.003 per unit of pollution
    w_industrial_prox: f32,    // -0.08 per industrial building within radius 5
    w_noise: f32,              // -0.02 per noise unit
    w_crime: f32,              // -0.05 per crime index point

    // Density context
    w_density_bonus: f32,      //  0.02 per occupied building within radius 4
                               //  (agglomeration effect for commercial)
    w_crowding_penalty: f32,   // -0.01 per occupied building within radius 3
                               //  (congestion for residential)
}

fn compute_hedonic_value(
    x: usize, y: usize,
    grid: &WorldGrid,
    services: &[ServiceBuilding],
    pollution: &PollutionGrid,
    noise: &NoiseGrid,
    crime: &CrimeGrid,
    cbd_center: (usize, usize),
    transit_stops: &[(usize, usize)],
    weights: &HedonicWeights,
) -> f32 {
    let mut value: f32 = 50.0; // Baseline

    // --- Accessibility ---

    // Distance to CBD (Central Business District)
    let cbd_dist = manhattan_distance(x, y, cbd_center.0, cbd_center.1) as f32;
    // Log decay: value drops fast near CBD, slower further out
    value += weights.w_cbd_distance * (1.0 + cbd_dist).ln().max(0.0) * -10.0;

    // Transit proximity (check nearest transit stop)
    let min_transit_dist = transit_stops.iter()
        .map(|&(tx, ty)| manhattan_distance(x, y, tx, ty))
        .min()
        .unwrap_or(999) as f32;
    if min_transit_dist <= 3.0 {
        value += weights.w_transit_proximity * (1.0 - min_transit_dist / 4.0) * 100.0;
    }

    // Road access
    let (n4, n4c) = grid.neighbors4(x, y);
    let has_road = (0..n4c).any(|i| grid.get(n4[i].0, n4[i].1).cell_type == CellType::Road);
    if has_road {
        value += weights.w_road_access * 100.0;
    }

    // --- Amenities ---

    // Parks within radius 8
    let park_count = count_services_in_radius(services, x, y, 8, |s| s.is_park());
    value += weights.w_park * park_count as f32 * 100.0;

    // Water proximity
    let water_dist = find_nearest_water(grid, x, y, 10);
    if water_dist <= 1 {
        value += weights.w_water * 100.0;
    } else if water_dist <= 3 {
        value += weights.w_water * 50.0;
    } else if water_dist <= 6 {
        value += weights.w_water * 20.0;
    }

    // Schools within radius 12
    let school_count = count_services_in_radius(services, x, y, 12, |s| s.is_school());
    value += weights.w_school * school_count as f32 * 100.0;

    // --- Disamenities ---

    // Pollution
    let poll = pollution.get(x, y) as f32;
    value += weights.w_pollution * poll * 100.0; // w_pollution is negative

    // Industrial proximity
    let industrial_count = count_zone_in_radius(grid, x, y, 5, ZoneType::Industrial);
    value += weights.w_industrial_prox * industrial_count as f32 * 100.0;

    // Crime
    let crime_level = crime.get(x, y);
    value += weights.w_crime * crime_level * 100.0;

    // --- Agglomeration / Density ---
    let nearby_buildings = count_occupied_buildings_in_radius(grid, x, y, 4);
    let cell = grid.get(x, y);
    if cell.zone.is_commercial() || cell.zone == ZoneType::Office {
        value += weights.w_density_bonus * nearby_buildings as f32 * 100.0;
    } else if cell.zone.is_residential() {
        // Residential prefers moderate density but not overcrowding
        let density_effect = if nearby_buildings < 8 {
            weights.w_density_bonus * nearby_buildings as f32 * 50.0
        } else {
            weights.w_crowding_penalty * (nearby_buildings - 8) as f32 * 100.0
        };
        value += density_effect;
    }

    value.max(1.0) // Floor at 1.0 (land always has some value)
}
```

### 2.3 Grid Propagation: Spatial Smoothing

Raw hedonic values are per-cell, but land value in reality is spatially autocorrelated --
a high-value cell raises the value of its neighbors. We need a smoothing step.

#### Method 1: Iterative Diffusion (Simple, O(n) per pass)

```
fn smooth_land_values(grid: &mut LandValueGrid, iterations: u32, alpha: f32) {
    // alpha = smoothing factor (0.1 to 0.3 works well)
    // Each iteration, each cell pulls toward its neighbors' average
    let w = grid.width;
    let h = grid.height;
    let mut temp = vec![0.0f32; w * h];

    for _ in 0..iterations {
        for y in 0..h {
            for x in 0..w {
                let center = grid.get_f32(x, y);
                let mut neighbor_sum = 0.0;
                let mut neighbor_count = 0;

                // 8-connectivity for smoother results
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        if dx == 0 && dy == 0 { continue; }
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx >= 0 && ny >= 0 && (nx as usize) < w && (ny as usize) < h {
                            neighbor_sum += grid.get_f32(nx as usize, ny as usize);
                            neighbor_count += 1;
                        }
                    }
                }

                let neighbor_avg = if neighbor_count > 0 {
                    neighbor_sum / neighbor_count as f32
                } else {
                    center
                };

                // Blend: new = (1-alpha)*center + alpha*neighbor_avg
                temp[y * w + x] = (1.0 - alpha) * center + alpha * neighbor_avg;
            }
        }

        // Copy back
        for i in 0..temp.len() {
            grid.values_f32[i] = temp[i];
        }
    }
}
```

3-5 iterations with alpha=0.2 gives a good balance of spatial correlation without
over-smoothing. Run on the slow tick timer (every 100 game ticks).

#### Method 2: Kernel Convolution (Higher quality, still O(n))

For higher-quality results, convolve with a Gaussian-like kernel:

```
// Discrete 5x5 approximation of Gaussian with sigma=1.0
const KERNEL_5X5: [[f32; 5]; 5] = [
    [0.003, 0.013, 0.022, 0.013, 0.003],
    [0.013, 0.059, 0.097, 0.059, 0.013],
    [0.022, 0.097, 0.159, 0.097, 0.022],
    [0.013, 0.059, 0.097, 0.059, 0.013],
    [0.003, 0.013, 0.022, 0.013, 0.003],
];
// Sum = 1.0 (normalized)

fn convolve_land_values(grid: &mut LandValueGrid) {
    let w = grid.width;
    let h = grid.height;
    let mut output = vec![0.0f32; w * h];

    for y in 0..h {
        for x in 0..w {
            let mut sum = 0.0;
            let mut weight_sum = 0.0;
            for ky in 0..5i32 {
                for kx in 0..5i32 {
                    let nx = x as i32 + kx - 2;
                    let ny = y as i32 + ky - 2;
                    if nx >= 0 && ny >= 0 && (nx as usize) < w && (ny as usize) < h {
                        let kernel_weight = KERNEL_5X5[ky as usize][kx as usize];
                        sum += grid.get_f32(nx as usize, ny as usize) * kernel_weight;
                        weight_sum += kernel_weight;
                    }
                }
            }
            output[y * w + x] = sum / weight_sum; // Normalize at edges
        }
    }

    grid.values_f32 = output;
}
```

For the 256x256 grid (65,536 cells), a 5x5 kernel convolution takes ~1.6M multiplications,
which is trivially fast even on a single core.

#### Method 3: Separable Gaussian (Optimization for large kernels)

If we ever need larger blur radii (e.g., for city-wide value gradients), use separable
convolution -- a 2D Gaussian can be decomposed into two 1D passes:

```
// 1D Gaussian kernel, radius 7 (15 taps)
fn gaussian_1d(sigma: f32, radius: usize) -> Vec<f32> {
    let mut kernel = Vec::with_capacity(2 * radius + 1);
    let mut sum = 0.0;
    for i in 0..=(2 * radius) {
        let x = i as f32 - radius as f32;
        let val = (-x * x / (2.0 * sigma * sigma)).exp();
        kernel.push(val);
        sum += val;
    }
    // Normalize
    for v in &mut kernel { *v /= sum; }
    kernel
}

// Two-pass separable convolution
// Horizontal pass, then vertical pass
// Cost: 2 * n * (2*radius+1) instead of n * (2*radius+1)^2
```

### 2.4 Real-World Land Value Examples

Understanding real land value ranges helps calibrate the simulation:

#### Manhattan, New York

| Location                  | Land Value per sqft | Land Value per acre | Notes                    |
|---------------------------|--------------------:|--------------------:|--------------------------|
| Midtown Manhattan (5th Ave)| $1,500-3,000       |  $65M-$130M         | Peak commercial          |
| SoHo/Chelsea              | $800-1,500          |  $35M-$65M          | Mixed-use trendy         |
| Upper East Side (res)     | $400-800            |  $17M-$35M          | Premium residential      |
| Harlem (gentrifying)      | $150-400            |  $6.5M-$17M         | Rapidly rising           |
| Far Rockaway              | $30-80              |  $1.3M-$3.5M        | Peripheral, low-access   |

**Gradient:** Values drop by roughly 50% for each 2 miles from the CBD, with jumps
at transit stations. The "subway premium" is real: properties within 1/4 mile of a
subway station sell for 10-25% more than otherwise identical properties.

#### Suburban Values (Chicago metro example)

| Location                    | Land Value per sqft | Median Home Price | Notes                  |
|-----------------------------|--------------------:|------------------:|------------------------|
| Near North Side (downtown)  | $200-500            |  $450,000         | Urban core             |
| Evanston (inner suburb)     | $40-100             |  $350,000         | College town, transit  |
| Naperville (outer suburb)   | $15-35              |  $400,000         | Good schools premium   |
| Joliet (exurb)              | $5-15               |  $220,000         | Lower access, lower SES|
| Rural Will County           | $1-3                |  $180,000         | Agricultural fringe    |

#### Subway Station Effect: Empirical Data

Studies of transit-oriented development consistently find:

- **Within 1/4 mile (400m, ~25 grid cells at 16m/cell):**
  - Residential: +10 to +25% value premium
  - Commercial: +15 to +35% value premium
  - Office: +20 to +40% value premium

- **1/4 to 1/2 mile:**
  - Residential: +5 to +12%
  - Commercial: +8 to +20%

- **Beyond 1/2 mile:** Effect diminishes to near zero

- **Disamenity zone (within 100m / 6 cells):**
  - If above-ground rail: -3 to -8% (noise, visual blight)
  - If underground: no disamenity
  - Bus stops: minimal positive or negative effect

```
fn transit_premium(distance_cells: usize, transit_type: TransitType) -> f32 {
    let base_premium = match transit_type {
        TransitType::SubwayStation => 0.25,
        TransitType::TrainStation => 0.20,
        TransitType::BusRapidTransit => 0.10,
        TransitType::BusStop => 0.03,
    };

    // Disamenity for very close, above-ground
    let noise_penalty = match transit_type {
        TransitType::TrainStation if distance_cells <= 3 => -0.05,
        _ => 0.0,
    };

    // Distance decay (quarter-mile ~ 25 cells at 16m)
    let distance_factor = if distance_cells <= 6 {
        1.0 + noise_penalty / base_premium
    } else if distance_cells <= 25 {
        // Linear decay from 1.0 at 6 cells to 0.4 at 25 cells
        1.0 - 0.6 * ((distance_cells - 6) as f32 / 19.0)
    } else if distance_cells <= 50 {
        // Slow decay
        0.4 - 0.4 * ((distance_cells - 25) as f32 / 25.0)
    } else {
        0.0
    };

    base_premium * distance_factor.max(0.0)
}
```

### 2.5 Land Value Feedback Loops

Land value is not just an output -- it creates feedback loops that are central to
urban dynamics:

#### Positive Feedback (Gentrification Spiral)

```
High land value
  -> Attracts high-income residents/businesses
    -> More spending at local businesses
      -> More commercial investment
        -> Better services, amenities
          -> Even higher land value
```

This is realistic. In the game, allow land value to grow without bound (in theory)
but with natural caps:
- Traffic congestion reduces accessibility, capping downtown values
- Crime tends to increase with density, partially offsetting
- Infrastructure capacity limits density

#### Negative Feedback (Blight Spiral)

```
Low land value / declining value
  -> Higher-income residents leave (filtering down)
    -> Less commercial activity, businesses close
      -> Reduced tax revenue, service cuts
        -> Crime increases, infrastructure deteriorates
          -> Even lower land value
```

This is the "death spiral" that affected Detroit, Cleveland, St. Louis. In the game,
it should be possible but not irreversible -- targeted investment (TIF districts,
parks, transit) can interrupt the cycle.

#### Implementation: Value Momentum

```
fn update_land_value_with_momentum(
    cell: &mut LandValueCell,
    new_hedonic_value: f32,
    dt: f32,
) {
    let old_value = cell.smoothed_value;

    // Value adjusts toward hedonic equilibrium with inertia
    // (property values don't change overnight)
    let adjustment_speed = if new_hedonic_value > old_value {
        0.02 // Values rise slowly (development takes time)
    } else {
        0.05 // Values fall faster than they rise (realistic)
    };

    cell.smoothed_value += (new_hedonic_value - old_value) * adjustment_speed * dt;
    cell.trend = cell.smoothed_value - old_value;
}
```

### 2.6 Highest and Best Use

In real estate, "highest and best use" (HBU) determines what should be built on a
parcel. It considers:

1. **Legally permissible** (what does zoning allow?)
2. **Physically possible** (can the site support it?)
3. **Financially feasible** (will it make money?)
4. **Maximally productive** (which feasible use produces the highest land value?)

In the game, this translates to: **the land value under different zone types differs.**
Commercial land in a high-traffic area is worth more than residential. Residential
land near a school with low crime is worth more than commercial.

```
fn highest_and_best_use(
    x: usize, y: usize,
    grid: &WorldGrid,
    context: &HedonicContext,
) -> (ZoneType, f32) {
    let zone_types = [
        ZoneType::ResidentialLow,
        ZoneType::ResidentialMed,
        ZoneType::ResidentialHigh,
        ZoneType::CommercialLow,
        ZoneType::CommercialHigh,
        ZoneType::Office,
        ZoneType::Industrial,
    ];

    let mut best_type = ZoneType::ResidentialLow;
    let mut best_value = 0.0f32;

    for &zt in &zone_types {
        let value = compute_hedonic_value_for_zone(x, y, zt, context);
        if value > best_value {
            best_value = value;
            best_type = zt;
        }
    }

    (best_type, best_value)
}

fn compute_hedonic_value_for_zone(
    x: usize, y: usize,
    zone_type: ZoneType,
    ctx: &HedonicContext,
) -> f32 {
    let base = compute_hedonic_value(x, y, ctx);

    // Zone-specific modifiers
    match zone_type {
        ZoneType::CommercialHigh | ZoneType::CommercialLow => {
            // Commercial values foot traffic, visibility, accessibility
            let traffic_bonus = ctx.pedestrian_traffic.get(x, y) * 0.5;
            let intersection_bonus = if ctx.is_intersection(x, y) { 20.0 } else { 0.0 };
            base + traffic_bonus + intersection_bonus
        }
        ZoneType::Office => {
            // Offices want CBD proximity, transit, prestige
            let cbd_bonus = (100.0 - ctx.cbd_distance(x, y) as f32 * 2.0).max(0.0);
            let transit_bonus = if ctx.near_transit(x, y, 15) { 30.0 } else { 0.0 };
            base + cbd_bonus + transit_bonus
        }
        ZoneType::ResidentialHigh => {
            // High-rise residential wants transit, tolerates density
            let transit_bonus = if ctx.near_transit(x, y, 20) { 20.0 } else { 0.0 };
            base + transit_bonus
        }
        ZoneType::ResidentialLow => {
            // Suburban residential wants space, schools, low crime
            let school_bonus = ctx.school_quality_radius(x, y, 15) * 15.0;
            let crime_penalty = ctx.crime_level(x, y) * -20.0;
            let space_bonus = if ctx.density_radius(x, y, 5) < 3 { 15.0 } else { 0.0 };
            base + school_bonus + crime_penalty + space_bonus
        }
        ZoneType::Industrial => {
            // Industrial wants highway access, cheap land, doesn't care about amenities
            let highway_bonus = if ctx.near_highway(x, y, 8) { 25.0 } else { 0.0 };
            let cheap_land_bonus = (50.0 - base).max(0.0) * 0.3; // prefers cheap areas
            highway_bonus + cheap_land_bonus + 20.0 // baseline industrial value
        }
        _ => base,
    }
}
```

### 2.7 Georgist Land Value Tax (Advanced Mechanic)

Henry George's "Progress and Poverty" (1879) proposed taxing land value rather than
improvements. This is economically efficient because land supply is fixed -- you can't
"produce less land" in response to taxation.

A **Land Value Tax (LVT)** option for Megacity:

- Tax the land component only (not buildings)
- Incentivizes development: an empty lot downtown pays the same tax as a skyscraper
  on the same land
- Discourages land speculation / land banking
- Revenue-neutral: can replace some property tax

```
fn georgist_land_tax(
    land_value: f32,       // From hedonic model
    improvement_value: f32, // Building value
    lvt_rate: f32,         // e.g., 0.05 = 5% of land value
    property_tax_rate: f32, // Traditional rate on improvements
) -> f64 {
    let land_tax = land_value as f64 * lvt_rate as f64;
    let improvement_tax = improvement_value as f64 * property_tax_rate as f64;
    land_tax + improvement_tax
}
```

In practice, cities like Pittsburgh, PA experimented with split-rate taxation (higher
rate on land, lower on improvements) from 1913-2001. Harrisburg, PA used it to
encourage downtown development with measurable success in the 1980s-90s.

For the game, this could be a policy toggle that:
- Increases development density in high-value areas
- Reduces land speculation (empty lots get built on faster)
- Potentially reduces overall revenue if land values are low
- Creates interesting gameplay tension between encouraging development and
  maintaining tax revenue

---

## 3. Supply and Demand

### 3.1 Housing Filtering Theory

Housing filtering is the process by which housing units transition from higher-income
to lower-income occupants over time as they age and newer, more desirable units are
built. This is the primary mechanism through which the private market provides affordable
housing (or fails to).

#### The Filtering Chain

```
New luxury construction (Year 0)
  -> Upper-income households move in
    -> Their previous homes (10-20 years old) become available
      -> Upper-middle income moves into those
        -> Their previous homes (20-40 years old) become available
          -> Middle income moves into those
            -> Continue down the chain...
              -> Eventually: 50-80 year old buildings house lower-income residents
                -> Very old buildings: demolition or renovation
```

**Key parameters:**
- Average filtering time: 20-30 years per step
- Building quality depreciates ~1-2% per year without maintenance
- Renovation resets the clock (gentrification = reverse filtering)
- Demolition removes supply from the bottom

#### Implementation

```
struct HousingUnit {
    building_entity: Entity,
    construction_year: u32,
    last_renovation_year: u32,
    quality: f32,            // 0.0 to 1.0
    target_income_bracket: IncomeBracket,
    rent: f64,
    occupied: bool,
}

enum IncomeBracket {
    Luxury,       // Top 10%
    UpperMiddle,  // Top 30%
    Middle,       // 30-70%
    LowerMiddle,  // 70-90%
    Low,          // Bottom 10%
}

fn filter_housing(unit: &mut HousingUnit, current_year: u32) {
    let age = current_year - unit.last_renovation_year;

    // Quality degrades over time
    // With no maintenance: loses 1.5% quality per year
    // Minimum quality of 0.05 (uninhabitable below that -> abandoned)
    let annual_depreciation = 0.015;
    unit.quality = (1.0 - annual_depreciation * age as f32).max(0.05);

    // Bracket assignment based on quality
    unit.target_income_bracket = match unit.quality {
        q if q >= 0.85 => IncomeBracket::Luxury,
        q if q >= 0.65 => IncomeBracket::UpperMiddle,
        q if q >= 0.45 => IncomeBracket::Middle,
        q if q >= 0.25 => IncomeBracket::LowerMiddle,
        _ => IncomeBracket::Low,
    };

    // Rent tracks quality (but also location via land value)
    // Higher land value slows filtering because owners invest in maintenance
    unit.rent = base_rent_for_bracket(unit.target_income_bracket) *
                unit.land_value_multiplier();
}
```

### 3.2 Vacancy Rate Dynamics

The **natural vacancy rate** is the rate at which markets are in equilibrium -- neither
tight (rents rising) nor loose (rents falling). This is one of the most important
equilibrium concepts in real estate economics.

#### Real-World Vacancy Rates

| Market Type          | Natural Vacancy | Current Tight | Current Loose |
|----------------------|----------------:|--------------:|--------------:|
| Residential (rental) |          5-7%   |       2-3%    |      8-12%    |
| Residential (owned)  |          1-2%   |       0.5-1%  |      3-5%     |
| Office (Class A)     |          8-12%  |       4-6%    |     15-25%    |
| Retail               |          5-8%   |       3-4%    |     12-20%    |
| Industrial/warehouse |          5-8%   |       2-4%    |     10-15%    |

**What vacancy rates mean for rent/price adjustment:**

```
fn vacancy_rent_adjustment(vacancy_rate: f32, natural_rate: f32) -> f32 {
    // If vacancy < natural: market is tight, rents rise
    // If vacancy > natural: market is loose, rents fall
    // The relationship is approximately linear near equilibrium
    // but nonlinear at extremes

    let deviation = vacancy_rate - natural_rate;

    if deviation < -0.03 {
        // Very tight market: rents rise fast (bidding wars)
        // +5 to +15% per year
        0.10 + (-deviation - 0.03) * 2.0
    } else if deviation < 0.0 {
        // Moderately tight: gradual rent increases
        // +1 to +5% per year
        -deviation * 1.5
    } else if deviation < 0.03 {
        // At or near equilibrium: stable to slight decline
        // -1 to +1% per year
        -deviation * 0.5
    } else if deviation < 0.08 {
        // Loose market: rents declining
        // -2 to -5% per year
        -deviation * 1.0
    } else {
        // Very loose: significant rent declines, potential abandonment
        // -5 to -15% per year
        -0.08 - (deviation - 0.08) * 1.5
    }
}
```

#### Vacancy in the Game

```
struct HousingMarket {
    total_residential_units: u32,
    occupied_residential_units: u32,
    total_commercial_sqft: u32,
    occupied_commercial_sqft: u32,
    total_office_sqft: u32,
    occupied_office_sqft: u32,

    // Derived
    residential_vacancy: f32,
    commercial_vacancy: f32,
    office_vacancy: f32,

    // Price indices (100 = baseline)
    residential_price_index: f32,
    commercial_rent_index: f32,
    office_rent_index: f32,
}

fn update_housing_market(market: &mut HousingMarket) {
    // Calculate vacancy rates
    market.residential_vacancy = if market.total_residential_units > 0 {
        1.0 - (market.occupied_residential_units as f32 /
               market.total_residential_units as f32)
    } else {
        0.0
    };

    market.commercial_vacancy = if market.total_commercial_sqft > 0 {
        1.0 - (market.occupied_commercial_sqft as f32 /
               market.total_commercial_sqft as f32)
    } else {
        0.0
    };

    market.office_vacancy = if market.total_office_sqft > 0 {
        1.0 - (market.occupied_office_sqft as f32 /
               market.total_office_sqft as f32)
    } else {
        0.0
    };

    // Adjust prices based on vacancy
    let res_adj = vacancy_rent_adjustment(market.residential_vacancy, 0.05);
    let com_adj = vacancy_rent_adjustment(market.commercial_vacancy, 0.07);
    let off_adj = vacancy_rent_adjustment(market.office_vacancy, 0.10);

    // Apply adjustment per tick (scale by time step)
    let dt = 1.0 / 365.0; // Daily adjustment
    market.residential_price_index *= 1.0 + res_adj * dt;
    market.commercial_rent_index *= 1.0 + com_adj * dt;
    market.office_rent_index *= 1.0 + off_adj * dt;

    // Clamp to prevent extreme values
    market.residential_price_index = market.residential_price_index.clamp(20.0, 500.0);
    market.commercial_rent_index = market.commercial_rent_index.clamp(20.0, 500.0);
    market.office_rent_index = market.office_rent_index.clamp(20.0, 500.0);
}
```

### 3.3 Commercial Rent per Square Foot

Real-world commercial rents vary enormously by type and location:

#### Retail Rent per Square Foot (Annual, NNN)

| Location / Type              | Low End | Typical | High End | Notes                     |
|------------------------------|--------:|--------:|---------:|---------------------------|
| Strip mall, suburban         |   $8    |   $15   |   $25    | Dominated by national chains|
| Neighborhood retail          |  $12    |   $22   |   $40    | Walkable, mixed-use         |
| Grocery-anchored center      |  $10    |   $18   |   $30    | Stable anchor tenant        |
| Regional mall, inline        |  $20    |   $40   |   $80    | Declining format            |
| Urban high street             |  $30    |   $75   |  $200    | SoHo, Rodeo Drive, etc.    |
| Class A mall, premium         |  $50    |  $100   |  $300    | Short Hills, Galleria, etc. |
| Times Square (peak)          | $300    | $1,500  | $3,000   | Exceptional outlier         |

(NNN = "Triple Net" -- tenant pays property tax, insurance, maintenance on top)

#### Office Rent per Square Foot (Annual, Full Service)

| Market                        | Class B  | Class A  | Trophy/Class A+  |
|-------------------------------|--------:|---------:|-----------------:|
| Small metro secondary         |  $12-18 |  $20-28  |       N/A        |
| Midsized metro (Charlotte)    |  $18-24 |  $28-38  |    $40-50        |
| Major metro suburban (Dallas) |  $20-28 |  $30-42  |    $45-55        |
| Major metro CBD (Chicago)     |  $25-35 |  $38-55  |    $60-80        |
| Premium CBD (Boston, DC)      |  $35-50 |  $55-75  |    $80-100       |
| Manhattan Midtown             |  $45-60 |  $70-95  |   $100-200       |

#### Industrial Rent per Square Foot (Annual, NNN)

| Type                          | Typical Range | Notes                        |
|-------------------------------|-------------:|------------------------------|
| Bulk warehouse                |    $4-8      | Large footprint, low ceiling |
| Distribution center           |    $6-12     | Higher ceiling, dock doors   |
| Light industrial / flex       |    $8-15     | Office/warehouse hybrid      |
| Cold storage                  |   $12-20     | Specialized refrigeration    |
| Last-mile logistics           |   $10-18     | Urban locations, expensive   |

### 3.4 Developer Pro Forma Analysis

In reality, buildings only get built when the numbers work. A developer performs a
"pro forma" analysis to decide whether to proceed. This is the financial feasibility
test that determines what gets built and where.

#### Simplified Pro Forma

```
-- RESIDENTIAL DEVELOPMENT PRO FORMA --

Revenue Side:
  Total units:                    100
  Average size:                   900 sqft
  Market rent per sqft/month:     $2.50
  Gross potential rent:           100 * 900 * $2.50 * 12 = $2,700,000/year
  Vacancy & collection loss (5%): -$135,000
  Effective gross income:         $2,565,000/year

Expense Side:
  Operating expenses (35-45% of EGI):
    Property management (5%):     $128,250
    Maintenance & repairs (8%):   $205,200
    Property taxes (15%):         $384,750
    Insurance (3%):               $76,950
    Utilities (5%):               $128,250
    Administrative (2%):          $51,300
  Total operating expenses:       $974,700/year

Net Operating Income (NOI):       $1,590,300/year

Development Costs:
  Land acquisition:               $2,000,000
  Hard costs (construction):      $18,000,000 ($200/sqft * 90,000 sqft)
  Soft costs (15% of hard):      $2,700,000
  Financing costs (8% of total):  $1,816,000
  Total development cost:         $24,516,000

Key Metrics:
  Yield on cost (NOI / total cost):  6.49%
  Cap rate market (from comparable sales): 5.5%
  Estimated value (NOI / cap rate):  $28,914,545
  Developer profit:                  $4,398,545
  Return on cost:                    17.9%
  IRR (assuming 24-month build + 5-year hold): ~15%
```

#### Cap Rates by Property Type (Real-World)

The **capitalization rate** is NOI / Property Value. Lower cap rates mean higher
values relative to income (perceived as safer/more desirable):

| Property Type               | Low Cap | Typical Cap | High Cap | Notes              |
|-----------------------------|--------:|------------:|---------:|---------------------|
| Multifamily Class A (urban) |   3.5%  |     4.5%    |    5.5%  | Most compressed      |
| Multifamily Class B         |   4.5%  |     5.5%    |    6.5%  |                      |
| Multifamily Class C         |   5.5%  |     6.5%    |    8.0%  | Higher risk          |
| Office Class A (CBD)        |   4.0%  |     5.5%    |    7.0%  | Post-COVID widened   |
| Office Suburban              |   5.5%  |     7.0%    |    9.0%  |                      |
| Retail (grocery-anchored)   |   5.0%  |     6.0%    |    7.0%  | Stable anchors       |
| Retail (unanchored)         |   6.0%  |     7.5%    |    9.5%  |                      |
| Industrial/Logistics        |   3.5%  |     5.0%    |    6.5%  | Very compressed      |

#### Implementation: Development Feasibility

```
struct DeveloperProForma {
    zone_type: ZoneType,
    land_cost: f64,
    construction_cost_per_sqft: f64,
    total_sqft: f64,
    units: u32,
    market_rent_per_sqft_month: f64,
    vacancy_rate: f32,
    operating_expense_ratio: f32,
    cap_rate: f32,
    financing_rate: f32,
    construction_months: u32,
}

impl DeveloperProForma {
    fn gross_potential_revenue(&self) -> f64 {
        self.total_sqft * self.market_rent_per_sqft_month * 12.0
    }

    fn effective_gross_income(&self) -> f64 {
        self.gross_potential_revenue() * (1.0 - self.vacancy_rate as f64)
    }

    fn net_operating_income(&self) -> f64 {
        self.effective_gross_income() * (1.0 - self.operating_expense_ratio as f64)
    }

    fn total_development_cost(&self) -> f64 {
        let hard_costs = self.total_sqft * self.construction_cost_per_sqft;
        let soft_costs = hard_costs * 0.15;
        let financing = (self.land_cost + hard_costs + soft_costs) *
                        self.financing_rate as f64 *
                        (self.construction_months as f64 / 12.0);
        self.land_cost + hard_costs + soft_costs + financing
    }

    fn stabilized_value(&self) -> f64 {
        self.net_operating_income() / self.cap_rate as f64
    }

    fn developer_profit(&self) -> f64 {
        self.stabilized_value() - self.total_development_cost()
    }

    fn yield_on_cost(&self) -> f32 {
        (self.net_operating_income() / self.total_development_cost()) as f32
    }

    fn is_feasible(&self) -> bool {
        // Developers typically require:
        // 1. Yield on cost > market cap rate + 100-200 bps
        // 2. Positive profit
        // 3. Profit margin > 10%
        let yoc = self.yield_on_cost();
        let profit_margin = self.developer_profit() / self.total_development_cost();

        yoc > self.cap_rate + 0.01 &&
        self.developer_profit() > 0.0 &&
        profit_margin > 0.10
    }
}

fn should_developer_build(
    cell: (usize, usize),
    zone: ZoneType,
    land_value: f32,
    market: &HousingMarket,
    city_costs: &CityConstructionCosts,
) -> bool {
    let pro_forma = DeveloperProForma {
        zone_type: zone,
        land_cost: land_value as f64 * 100.0, // Scale to dollar equivalent
        construction_cost_per_sqft: city_costs.cost_per_sqft(zone),
        total_sqft: zone.typical_building_sqft(),
        units: zone.typical_units(),
        market_rent_per_sqft_month: market.rent_for_zone(zone),
        vacancy_rate: market.vacancy_for_zone(zone),
        operating_expense_ratio: zone.operating_expense_ratio(),
        cap_rate: market.cap_rate_for_zone(zone),
        financing_rate: 0.06, // Could tie to credit rating
        construction_months: zone.construction_months(),
    };

    pro_forma.is_feasible()
}
```

### 3.5 Zoning Constraining Supply

Zoning is the primary mechanism through which cities restrict what can be built.
In a city builder, the player IS the zoning authority. The constraint manifests as:

1. **Use restrictions:** Only certain zone types allowed in each cell
2. **Density limits:** Floor Area Ratio (FAR), building height, lot coverage
3. **Setback requirements:** Distance from property lines
4. **Parking minimums:** Required off-street parking (huge impact on density)

#### Floor Area Ratio (FAR)

FAR = Total building floor area / Lot area

| Zone Type          | Typical FAR | Building Character              |
|--------------------|------------:|----------------------------------|
| Single family (R1) |    0.3-0.5  | One house per lot               |
| Low residential    |    0.5-1.0  | Duplexes, small apartments      |
| Medium residential |    1.0-3.0  | 3-6 story apartments            |
| High residential   |    3.0-10.0 | High-rise towers                |
| Neighborhood comm. |    0.5-2.0  | Strip retail, small offices     |
| Downtown commercial|    3.0-15.0 | Towers, mixed-use               |
| Industrial         |    0.3-1.0  | Low, spread-out buildings       |

#### Zoning's Effect on Supply

**The fundamental tension in city builders:**

- Player zones land (determines supply of buildable area)
- Market demand drives what gets built within zones
- Over-zoning: too much land zoned for one use -> low density, scattered development
- Under-zoning: artificial scarcity -> high rents, housing crisis, unhappy citizens

```
fn calculate_zoning_constraint(
    demand: &ZoneDemand,
    supply: &ZoneSupply,
) -> ZoningPressure {
    let zones = [
        ZoneType::ResidentialLow,
        ZoneType::ResidentialMed,
        ZoneType::ResidentialHigh,
        ZoneType::CommercialLow,
        ZoneType::CommercialHigh,
        ZoneType::Industrial,
        ZoneType::Office,
    ];

    let mut pressures = HashMap::new();

    for &zone in &zones {
        let zone_demand = demand.get(zone); // Units demanded
        let zone_supply = supply.get(zone); // Units available (built + buildable)
        let zone_built = supply.built(zone); // Currently occupied

        // Supply ratio: how much of demand is being met
        let supply_ratio = if zone_demand > 0.0 {
            zone_built / zone_demand
        } else {
            1.0
        };

        let pressure = if supply_ratio < 0.6 {
            ZoningPressure::SevereShortage // Rents spiking, citizens angry
        } else if supply_ratio < 0.85 {
            ZoningPressure::Shortage       // Rents rising, development active
        } else if supply_ratio < 1.1 {
            ZoningPressure::Balanced        // Healthy market
        } else if supply_ratio < 1.5 {
            ZoningPressure::Oversupply      // Rents flat, some vacancy
        } else {
            ZoningPressure::GlutOversupply  // Rents falling, abandonment risk
        };

        pressures.insert(zone, pressure);
    }

    ZoningPressure::from(pressures)
}
```

#### Upzoning and Downzoning Effects

- **Upzoning** (e.g., R1 -> R3, allowing more density): Immediately increases land
  value because more profitable development is now permitted. But it takes time for
  development to actually occur.
- **Downzoning** (reducing allowed density): Reduces land value. Existing buildings
  that exceed new limits are "legally nonconforming" (grandfathered) but cannot
  expand.

```
fn upzone_effect(cell: &mut GridCell, new_zone: ZoneType) {
    let old_max_far = cell.zone.max_far();
    let new_max_far = new_zone.max_far();

    if new_max_far > old_max_far {
        // Upzoning: land value increases proportional to additional development potential
        let far_increase_ratio = new_max_far / old_max_far;
        cell.land_value_modifier *= 1.0 + (far_increase_ratio - 1.0) * 0.3;
        // (not dollar-for-dollar; accounts for risk, time value, etc.)
    } else if new_max_far < old_max_far {
        // Downzoning: land value decreases
        let far_decrease_ratio = new_max_far / old_max_far;
        cell.land_value_modifier *= far_decrease_ratio.powf(0.5);
        // (less-than-proportional decrease because existing use is grandfathered)
    }
}
```

---

## 4. Labor Markets

### 4.1 Worker-Job Matching

In a city economy, labor markets determine wages, employment, and commuting patterns.
The fundamental model is: workers seek the job that maximizes their utility
(wage minus commuting cost), and employers seek workers that maximize their productivity.

#### Matching Function

Labor economics uses the **matching function** to describe how workers and jobs find
each other:

```
M = A * U^alpha * V^(1-alpha)

Where:
  M = number of matches per period
  A = matching efficiency (how well the labor market works)
  U = number of unemployed workers seeking jobs
  V = number of vacant positions
  alpha = elasticity of matching w.r.t. unemployment (typically 0.5-0.7)
```

This is the Cobb-Douglas matching function used in Diamond-Mortensen-Pissarides
(DMP) search models (2010 Nobel Prize in Economics).

#### Implementation for Megacity

```
struct LaborMarket {
    // By skill level / industry sector
    workers_by_sector: HashMap<Sector, WorkerPool>,
    jobs_by_sector: HashMap<Sector, JobPool>,

    // Aggregate stats
    total_employed: u32,
    total_unemployed: u32,
    total_vacancies: u32,
    unemployment_rate: f32,

    // Matching parameters
    matching_efficiency: f32,  // A in the matching function (0.5-1.0)
    matching_alpha: f32,       // elasticity (typically 0.6)
}

#[derive(Clone)]
struct WorkerPool {
    count: u32,
    average_skill: f32,     // 0.0 to 1.0
    average_wage: f64,
    unemployed: u32,
}

#[derive(Clone)]
struct JobPool {
    count: u32,
    required_skill: f32,    // Minimum skill level
    offered_wage: f64,
    vacancies: u32,
}

enum Sector {
    Agriculture,
    Manufacturing,
    Retail,
    Professional,
    Technology,
    Healthcare,
    Education,
    Government,
    Construction,
    Hospitality,
}

fn match_workers_to_jobs(market: &mut LaborMarket, dt: f32) {
    for sector in Sector::all() {
        let workers = market.workers_by_sector.get(&sector);
        let jobs = market.jobs_by_sector.get(&sector);

        if let (Some(w), Some(j)) = (workers, jobs) {
            let u = w.unemployed as f32;
            let v = j.vacancies as f32;

            if u <= 0.0 || v <= 0.0 { continue; }

            // DMP matching function
            let matches_per_period = market.matching_efficiency *
                u.powf(market.matching_alpha) *
                v.powf(1.0 - market.matching_alpha);

            // Scale by time step and cap
            let actual_matches = (matches_per_period * dt)
                .min(u)
                .min(v) as u32;

            // Skill mismatch reduces effective matches
            let skill_match_rate = if w.average_skill >= j.required_skill {
                1.0
            } else {
                // Penalty for under-skilled workers
                (w.average_skill / j.required_skill).powf(2.0)
            };

            let effective_matches = (actual_matches as f32 * skill_match_rate) as u32;

            // Apply matches
            if let Some(w) = market.workers_by_sector.get_mut(&sector) {
                w.unemployed = w.unemployed.saturating_sub(effective_matches);
            }
            if let Some(j) = market.jobs_by_sector.get_mut(&sector) {
                j.vacancies = j.vacancies.saturating_sub(effective_matches);
            }
        }
    }

    // Update aggregate stats
    market.total_unemployed = market.workers_by_sector.values()
        .map(|w| w.unemployed).sum();
    market.total_employed = market.workers_by_sector.values()
        .map(|w| w.count - w.unemployed).sum();
    market.total_vacancies = market.jobs_by_sector.values()
        .map(|j| j.vacancies).sum();
    market.unemployment_rate = if market.total_employed + market.total_unemployed > 0 {
        market.total_unemployed as f32 /
        (market.total_employed + market.total_unemployed) as f32
    } else {
        0.0
    };
}
```

### 4.2 Wage Curves

The **wage curve** (Blanchflower & Oswald, 1994) describes the empirical relationship
between wages and local unemployment:

```
ln(wage) = -0.1 * ln(unemployment_rate) + controls

"A doubling of the unemployment rate is associated with a 10% decline in wages."
```

This relationship is remarkably stable across countries and time periods. It emerges
from bargaining power: when unemployment is low, workers can demand higher wages.

#### Wage Determination

```
fn calculate_wage(
    sector: Sector,
    skill_level: f32,
    local_unemployment: f32,
    cost_of_living: f32,
    city_productivity: f32,
) -> f64 {
    // Base wage by sector (annual, in game dollars)
    let base_wage = match sector {
        Sector::Agriculture => 28_000.0,
        Sector::Manufacturing => 42_000.0,
        Sector::Retail => 30_000.0,
        Sector::Professional => 65_000.0,
        Sector::Technology => 85_000.0,
        Sector::Healthcare => 60_000.0,
        Sector::Education => 48_000.0,
        Sector::Government => 52_000.0,
        Sector::Construction => 45_000.0,
        Sector::Hospitality => 28_000.0,
    };

    // Skill premium (exponential: higher skills command disproportionately more)
    let skill_premium = 1.0 + (skill_level as f64 - 0.5) * 1.5;
    // At skill 0.0: 0.25x, skill 0.5: 1.0x, skill 1.0: 1.75x

    // Wage curve effect: -0.1 elasticity
    // At 5% unemployment (natural rate), multiplier = 1.0
    // At 10% unemployment, multiplier ~ 0.93
    // At 2% unemployment, multiplier ~ 1.10
    let natural_rate = 0.05;
    let unemployment_effect = (natural_rate / local_unemployment.max(0.01)) as f64;
    let wage_curve_mult = unemployment_effect.powf(0.1);

    // Cost of living adjustment
    let col_mult = cost_of_living as f64;

    // City productivity (agglomeration economies)
    let productivity_mult = city_productivity as f64;

    base_wage * skill_premium * wage_curve_mult * col_mult * productivity_mult
}
```

### 4.3 Unemployment Simulation

Unemployment in a city builder should reflect real-world dynamics:

#### Types of Unemployment

1. **Frictional** (2-3%): People between jobs. Always exists in a healthy economy.
   Reduced by better job matching (employment services, internet age).

2. **Structural** (1-3%): Mismatch between worker skills and available jobs.
   Manufacturing workers when factories close. Reduced by education/retraining.

3. **Cyclical** (0-10%+): Due to economic downturns. The portion that varies with
   the business cycle.

4. **Seasonal** (varies): Agriculture, tourism, construction. Not usually modeled
   in city builders but could add realism.

```
struct UnemploymentBreakdown {
    frictional: f32,    // Always present, ~3%
    structural: f32,    // From skill mismatch
    cyclical: f32,      // From economic cycle
    total: f32,
}

fn calculate_unemployment(
    labor_market: &LaborMarket,
    economic_cycle: f32,      // -1.0 (recession) to 1.0 (boom)
    skill_mismatch: f32,      // 0.0 (perfect match) to 1.0 (total mismatch)
    population: u32,
) -> UnemploymentBreakdown {
    // Frictional: always ~3%, slightly lower with better services
    let frictional = 0.03;

    // Structural: driven by skill mismatch between workers and available jobs
    // Range: 0-8%
    let structural = skill_mismatch * 0.08;

    // Cyclical: driven by economic cycle
    // In a boom (1.0): 0% cyclical (may even push below natural rate)
    // In a recession (-1.0): up to 8% cyclical unemployment
    let cyclical = ((-economic_cycle + 1.0) / 2.0 * 0.08).max(0.0);

    let total = (frictional + structural + cyclical).min(0.30); // Cap at 30%

    UnemploymentBreakdown {
        frictional,
        structural,
        cyclical,
        total,
    }
}
```

#### Unemployment Effects on the City

| Unemployment Rate | Effect                                           |
|------------------:|--------------------------------------------------|
|          < 3%     | Labor shortage: wages rise fast, hard to attract business |
|        3-5%       | Healthy: "full employment," balanced market       |
|        5-7%       | Mild concern: some household stress               |
|        7-10%      | Significant: crime rises, happiness drops, emigration starts |
|       10-15%      | Severe: major social problems, political instability |
|         > 15%     | Crisis: widespread poverty, potential for unrest   |

### 4.4 Commuting Elasticity

Commuting is the glue between where people live and where they work. The willingness
to commute is captured by **commuting elasticity** -- how much further people will
commute for higher wages or lower housing costs.

#### Real-World Commuting Data

| Commute Type        | Average Time | Mode Share | Willingness to Pay |
|---------------------|-------------:|-----------:|-------------------:|
| Drive alone         |     26 min   |     76%    | ~$5-15/day         |
| Carpool             |     28 min   |      9%    | ~$3-8/day          |
| Public transit      |     48 min   |      5%    | ~$2-7/day          |
| Walk                |     12 min   |      3%    | ~$0 (prefer it)    |
| Work from home      |      0 min   |      7%    | Premium varies     |

**Value of time in commuting:** Research consistently finds that commuters value
commute time at approximately **50-60% of their hourly wage**. A worker earning
$30/hour values each minute of commute at about $0.25-0.30.

```
fn commute_utility_cost(
    distance_cells: usize,
    road_congestion: f32,     // 1.0 = free flow, 2.0 = double time
    transit_available: bool,
    hourly_wage: f64,
) -> f64 {
    // Convert distance to travel time (minutes)
    let base_speed_cells_per_minute = 2.0; // ~32m/min = ~1.2mph (grid cells)
    let car_speed = base_speed_cells_per_minute * 3.0 / road_congestion as f64;
    let transit_speed = base_speed_cells_per_minute * 2.5; // Faster than walking, slower than car

    let travel_time_car = distance_cells as f64 / car_speed;
    let travel_time_transit = if transit_available {
        distance_cells as f64 / transit_speed + 10.0 // 10 min wait/transfer
    } else {
        f64::MAX
    };

    let travel_time = travel_time_car.min(travel_time_transit);

    // Monetary cost of commuting
    let gas_cost_per_cell = 0.30; // ~$0.30 per cell driven
    let transit_fare = if transit_available { 2.75 } else { 0.0 };

    let monetary_cost = if travel_time_transit < travel_time_car {
        transit_fare
    } else {
        distance_cells as f64 * gas_cost_per_cell
    };

    // Time cost (value of time = 55% of hourly wage)
    let time_cost = travel_time * (hourly_wage * 0.55 / 60.0);

    // Total daily commute cost (round trip)
    (monetary_cost + time_cost) * 2.0
}

fn residential_location_utility(
    housing_cost: f64,       // Monthly rent
    commute_cost: f64,       // Daily commute cost * 22 workdays
    neighborhood_quality: f32, // 0-1
    school_quality: f32,     // 0-1
    income: f64,             // Monthly
) -> f64 {
    let disposable = income - housing_cost - commute_cost;

    // Utility = disposable income + quality-of-life bonuses
    // (simplified; real models use Cobb-Douglas utility functions)
    let quality_bonus = neighborhood_quality as f64 * income * 0.1
                      + school_quality as f64 * income * 0.08;

    disposable + quality_bonus
}
```

#### Commuting Patterns Shape Cities

The **monocentric city model** (Alonso-Muth-Mills) predicts that housing prices
decrease with distance from the CBD, and this gradient equals the marginal commuting
cost per mile. In equilibrium, all workers are indifferent between locations because
higher rents near the center are exactly offset by lower commuting costs.

```
Rent(d) = Rent(0) - t * d

Where:
  Rent(d) = rent at distance d from CBD
  Rent(0) = rent at CBD
  t = commuting cost per unit distance per period
  d = distance from CBD
```

The slope of this gradient determines city shape:
- **Steep gradient (high t):** Compact city, high density near center (pre-automobile)
- **Flat gradient (low t):** Sprawling city, suburban (automobile era)
- **Multi-centered:** Multiple employment nodes, complex gradient

### 4.5 Education and Skill Formation

Skills determine what jobs workers can fill and what wages they earn. The education
system is the primary mechanism for skill formation.

```
struct WorkerSkills {
    education_level: EducationLevel,
    skill_points: f32,       // 0.0 to 1.0
    experience_years: u32,
}

enum EducationLevel {
    NoHighSchool,     // ~10% of US adults
    HighSchool,       // ~27% of US adults
    SomeCollege,      // ~20% of US adults
    Associates,       // ~9% of US adults
    Bachelors,        // ~22% of US adults
    Graduate,         // ~12% of US adults
}

impl WorkerSkills {
    fn effective_skill(&self) -> f32 {
        let education_base = match self.education_level {
            EducationLevel::NoHighSchool => 0.10,
            EducationLevel::HighSchool => 0.25,
            EducationLevel::SomeCollege => 0.35,
            EducationLevel::Associates => 0.40,
            EducationLevel::Bachelors => 0.55,
            EducationLevel::Graduate => 0.75,
        };

        // Experience adds skill (diminishing returns)
        let experience_bonus = (self.experience_years as f32 / 30.0)
            .min(1.0)
            .powf(0.5) * 0.25;
        // 0 years: +0.00, 10 years: +0.14, 30+ years: +0.25

        (education_base + experience_bonus).min(1.0)
    }

    fn sector_match(&self, sector: Sector) -> f32 {
        // How well this worker's education matches the sector
        let required = sector.required_education();
        let has = self.education_level as u32;
        let req = required as u32;

        if has >= req {
            1.0 // Meets or exceeds requirement
        } else {
            // Penalty for under-qualification
            0.5_f32.powf((req - has) as f32) // 0.5 per level short
        }
    }
}
```

---

## 5. Economic Cycles

### 5.1 What Causes City Booms and Busts

Cities experience economic cycles driven by multiple interacting factors:

#### Boom Triggers

1. **Industry arrival:** A major employer establishes operations (e.g., Tesla in Austin,
   Amazon HQ2 in Arlington). Creates direct jobs + multiplier effect.

2. **Infrastructure investment:** New highway, transit line, airport. Reduces
   transportation costs, increases accessibility.

3. **Resource discovery:** Oil boom (Williston, ND), gold rush (San Francisco).
   Massive but potentially temporary wealth.

4. **Educational/research hub:** University + venture capital ecosystem
   (Stanford -> Silicon Valley, MIT -> Route 128).

5. **Government spending:** Military bases, federal facilities, state capitals.
   Stable but creates dependency.

6. **Speculative bubble:** Asset prices rise above fundamental value, attracting
   more investment, creating a self-reinforcing cycle until correction.

#### Bust Triggers

1. **Industry departure/collapse:** Auto industry in Detroit, steel in Pittsburgh,
   textiles in New England mill towns.

2. **Resource depletion:** Mining towns that exhaust their resource. "Ghost towns."

3. **Demographic shift:** White flight, suburbanization, birth rate decline.
   Population loss reduces the tax base.

4. **Competition from other cities:** Race to the bottom on taxes, incentives.
   Zero-sum game where one city's gain is another's loss.

5. **Natural disaster:** Hurricane Katrina (New Orleans lost 50% of population
   temporarily), earthquake, flood.

6. **Policy failure:** Corruption, mismanagement, over-borrowing, pension crisis.

#### The Multiplier Effect

When a "base" job is created (one that brings money into the city from outside --
manufacturing, tech, federal government), it creates additional "local" jobs:

```
Total jobs created = Base jobs * Local multiplier

Multiplier by industry:
  Technology:        4.5-5.0x  (1 tech job creates 4-5 local service jobs)
  Manufacturing:     1.5-2.5x  (declining from historical 3.0+)
  Finance:           3.0-4.0x
  Federal government: 1.5-2.0x
  Local services:    0.5-1.0x  (redistributive, not base)
  Construction:      1.2-1.5x  (temporary during building period)

fn economic_multiplier(sector: Sector) -> f32 {
    match sector {
        Sector::Technology => 4.5,
        Sector::Professional => 3.5,
        Sector::Manufacturing => 2.0,
        Sector::Healthcare => 2.5,
        Sector::Education => 2.0,
        Sector::Government => 1.8,
        Sector::Construction => 1.3,
        Sector::Retail => 0.8,
        Sector::Hospitality => 0.7,
        Sector::Agriculture => 1.2,
    }
}
```

### 5.2 Dutch Disease

"Dutch disease" occurs when a resource boom (or any single dominant industry)
crowds out other economic activity by raising costs:

```
Oil boom in a city:
  -> Oil sector pays high wages
    -> Workers leave other sectors for oil jobs
      -> Other sectors must raise wages to compete
        -> Other sectors become uncompetitive
          -> Economic base narrows to oil dependency
            -> When oil prices fall, entire economy crashes
```

**Historical examples:**
- Netherlands (1960s): Natural gas discovery -> manufacturing decline
- Houston (1980s): Oil boom followed by devastating oil price collapse
- Fort McMurray, Canada: Oil sands boom/bust cycle
- Equatorial Guinea: Oil wealth, no economic diversification

#### Implementation

```
fn check_dutch_disease(
    economy: &CityEconomy,
    sectors: &HashMap<Sector, SectorStats>,
) -> DutchDiseaseStatus {
    // Find the dominant sector
    let total_employment: u32 = sectors.values().map(|s| s.employment).sum();
    let max_sector = sectors.iter()
        .max_by_key(|(_, s)| s.employment)
        .map(|(sector, stats)| (*sector, stats.employment));

    if let Some((dominant, employment)) = max_sector {
        let concentration = employment as f32 / total_employment as f32;

        // Herfindahl-Hirschman Index for economic concentration
        let hhi: f32 = sectors.values()
            .map(|s| {
                let share = s.employment as f32 / total_employment as f32;
                share * share
            })
            .sum();

        // HHI ranges:
        // 0.0-0.10: Highly diversified
        // 0.10-0.18: Moderately concentrated
        // 0.18-0.25: Concentrated
        // 0.25+: Highly concentrated (Dutch disease risk)

        if hhi > 0.25 && concentration > 0.35 {
            DutchDiseaseStatus::Severe {
                dominant_sector: dominant,
                concentration,
                hhi,
            }
        } else if hhi > 0.18 {
            DutchDiseaseStatus::Moderate { hhi }
        } else {
            DutchDiseaseStatus::Healthy { hhi }
        }
    } else {
        DutchDiseaseStatus::Healthy { hhi: 0.0 }
    }
}

fn apply_dutch_disease_effects(
    status: &DutchDiseaseStatus,
    sectors: &mut HashMap<Sector, SectorStats>,
    happiness: &mut f32,
) {
    if let DutchDiseaseStatus::Severe { dominant_sector, concentration, .. } = status {
        // Non-dominant sectors lose competitiveness
        let dominant_wage = sectors.get(dominant_sector)
            .map(|s| s.average_wage).unwrap_or(50_000.0);

        for (sector, stats) in sectors.iter_mut() {
            if sector != dominant_sector {
                // Wage pressure: must match portion of dominant sector wages
                let wage_pressure = dominant_wage * 0.3 * *concentration as f64;
                let cost_increase = wage_pressure / stats.average_wage;

                // This makes other sectors less profitable
                stats.profitability *= (1.0 - cost_increase as f32 * 0.1).max(0.3);

                // Attrition: workers leave for dominant sector
                let attrition_rate = 0.02 * *concentration;
                let lost_workers = (stats.employment as f32 * attrition_rate) as u32;
                stats.employment = stats.employment.saturating_sub(lost_workers);
            }
        }

        // Citizens may be happy (high wages) or unhappy (lack of diversity, instability)
        *happiness += 0.05 * *concentration; // Short-term happiness from high wages
        *happiness -= 0.03; // Long-term anxiety about concentration
    }
}
```

### 5.3 Agglomeration Economies

Agglomeration economies are the benefits firms and workers receive from being located
near each other. This is why cities exist -- the productivity advantages of proximity
outweigh the costs of crowding.

#### Three Types of Agglomeration

1. **Sharing:** Firms share infrastructure, specialized labor pools, and intermediate
   inputs. A single factory can't justify a specialized machine shop; 50 factories can.

2. **Matching:** Larger labor markets produce better worker-job matches. Workers find
   jobs that better fit their skills; employers find workers that better fit their needs.

3. **Learning:** Knowledge spillovers. Being near other firms in the same industry
   accelerates innovation. This is why tech clusters (Silicon Valley), finance clusters
   (Wall Street), and entertainment clusters (Hollywood) exist.

#### Quantifying Agglomeration

Research (Combes & Gobillon, 2015; Rosenthal & Strange, 2004) finds:

- **Doubling city size increases productivity by 3-8%**
- The effect is strongest for knowledge-intensive industries (+8-15%)
- Manufacturing sees smaller benefits (+2-5%)
- The effect decays with distance: half-life of ~5-8 km
- Within-industry clustering (localization) is stronger than cross-industry (urbanization)

```
fn agglomeration_productivity_bonus(
    city_population: u32,
    sector: Sector,
    same_sector_firms_nearby: u32,
    total_firms_nearby: u32,
) -> f32 {
    // Urbanization economies (city size)
    let size_elasticity = match sector {
        Sector::Technology => 0.08,
        Sector::Professional => 0.06,
        Sector::Healthcare => 0.04,
        Sector::Manufacturing => 0.03,
        Sector::Retail => 0.02,
        _ => 0.03,
    };

    // ln(population) * elasticity
    // Baseline at 100K population
    let urbanization = if city_population > 100_000 {
        ((city_population as f32 / 100_000.0).ln() * size_elasticity)
    } else {
        // Small cities get a slight penalty relative to baseline
        (city_population as f32 / 100_000.0).ln() * size_elasticity * 0.5
    };

    // Localization economies (same-sector clustering)
    let localization_elasticity = match sector {
        Sector::Technology => 0.12,  // Very strong cluster effects
        Sector::Professional => 0.08,
        Sector::Manufacturing => 0.06,
        Sector::Healthcare => 0.05,
        _ => 0.03,
    };

    let localization = if same_sector_firms_nearby > 5 {
        (same_sector_firms_nearby as f32 / 5.0).ln() * localization_elasticity
    } else {
        0.0
    };

    // Total bonus: typically 5-25% in a well-developed city
    (urbanization + localization).clamp(-0.10, 0.30)
}
```

#### Congestion Diseconomies

Agglomeration benefits are partially offset by congestion costs:

- Traffic congestion
- Housing costs
- Crime
- Pollution
- Competition for workers (wage inflation)

```
fn congestion_cost(
    city_population: u32,
    road_capacity_utilization: f32,  // 0.0 to 1.5+ (can exceed capacity)
    average_commute_time: f32,       // minutes
    housing_cost_index: f32,         // 100 = national average
) -> f32 {
    // Congestion scales superlinearly with population
    let pop_congestion = if city_population > 100_000 {
        ((city_population as f32 / 100_000.0).ln()).powf(1.5) * 0.02
    } else {
        0.0
    };

    // Road congestion
    let road_congestion = if road_capacity_utilization > 0.8 {
        (road_capacity_utilization - 0.8).powf(2.0) * 0.5
    } else {
        0.0
    };

    // Commute time penalty
    let commute_penalty = if average_commute_time > 25.0 {
        (average_commute_time - 25.0) * 0.002
    } else {
        0.0
    };

    // Housing cost penalty (makes it hard to attract workers)
    let housing_penalty = if housing_cost_index > 130.0 {
        (housing_cost_index - 130.0) * 0.001
    } else {
        0.0
    };

    pop_congestion + road_congestion + commute_penalty + housing_penalty
}

// Net agglomeration benefit = agglomeration_bonus - congestion_cost
// Cities grow until net benefit approaches zero (equilibrium size)
```

### 5.4 How Games Model Economies

#### SimCity's Tax Curve (SimCity 4, 2003)

SimCity uses a simple demand model where tax rates directly affect zone demand
(RCI -- Residential, Commercial, Industrial):

```
// SimCity 4 approximate tax-demand relationship:
// (Reverse-engineered from game behavior)

fn simcity_demand_from_tax(tax_rate: f32) -> f32 {
    // tax_rate is 0-20% (0.0 to 0.20)
    // Returns demand modifier: positive = growth, negative = decline

    // Sweet spot is around 7-9%
    // Below 7%: strong demand but low revenue
    // At 7%: good balance
    // 9%: still positive demand
    // Above 9%: demand drops sharply
    // At 20%: maximum negative demand

    if tax_rate <= 0.07 {
        // Linear increase in demand as taxes decrease
        // At 0%: +3.0 demand boost
        // At 7%: +0.5 demand boost
        3.0 - (tax_rate / 0.07) * 2.5
    } else if tax_rate <= 0.09 {
        // Slight positive demand
        0.5 - (tax_rate - 0.07) / 0.02 * 0.5
    } else if tax_rate <= 0.12 {
        // Demand turns negative
        0.0 - (tax_rate - 0.09) / 0.03 * 2.0
    } else {
        // Sharply negative demand
        -2.0 - (tax_rate - 0.12) / 0.08 * 4.0
    }
}

// SimCity also factors in:
// - Desirability (land value equivalent)
// - Capacity (infrastructure limits)
// - Neighboring city demand (regional)
// - Mayor rating (citizen happiness with services)
```

SimCity 4's key insight: **separate tax rates per density level.** Players could
set low taxes for low-density residential to encourage suburban growth while
taxing high-density commercial higher. This created a natural tension between
revenue and growth.

#### Cities: Skylines (2015)

Cities: Skylines uses a simpler system:

```
// Approximate Cities: Skylines economy:
//
// Tax rates: 1-29% per zone type (default 12%)
// Revenue = population * tax_rate * base_income_per_type
//
// Demand is three bars (R, C, I) influenced by:
// - Available workers (R -> drives C and I demand)
// - Available shoppers (R -> drives C demand)
// - Available goods (I -> drives C supply)
// - Educated workers (drives I and office demand)
// - Tax rates (higher = lower demand, roughly linear)
// - Land value (higher = more desirable = more demand)
// - Services coverage (police, fire, health, education, leisure)
//
// The system is relatively simple:
// - No real property values (just "land value" for visual/happiness)
// - No real rents or housing market
// - No unemployment (citizens always find work or leave)
// - No business cycles
// - Tax response is nearly linear (no sweet spot or Laffer curve)
```

#### Anno Series (Anno 1800)

Anno games have the most detailed economic simulation among city builders:

```
// Anno 1800 economy model:
//
// Production chains are the core mechanic:
//   Farm -> Grain -> Mill -> Flour -> Bakery -> Bread -> Citizens
//
// Each population tier (Farmer, Worker, Artisan, Engineer, Investor)
// has specific needs:
//   Basic needs: Must be met or population declines
//   Luxury needs: Meeting these upgrades citizens to next tier
//
// Key formulas:
//   Consumption per minute = population * consumption_rate_per_person
//   Production per minute = 1 / production_time * workforce_efficiency
//
// Example: Bread production
//   1 Grain Farm produces 1 ton grain / minute
//   1 Mill processes 1 ton grain / minute -> 1 ton flour / minute
//   1 Bakery processes 1 ton flour / minute -> 1 ton bread / minute
//   1 ton bread serves 600 Farmers per minute
//   So 1000 Farmers need ~1.67 bakeries (and supporting chain)
//
// Trade:
//   Import price = base_price * distance_modifier * supply_demand_modifier
//   Export price = base_price * 0.8 (20% less than import)
//   Passive trade generates income from excess goods
//
// Taxes:
//   Revenue per citizen = base_rate * tax_level_multiplier * happiness_modifier
//   Tax levels: Very Low (0.5x, +happiness), Low (0.75x), Medium (1.0x),
//               High (1.5x, -happiness), Very High (2.0x, --happiness)
```

#### Tropico (Tropico 6, 2019)

Tropico models a small island economy with detailed citizen simulation:

```
// Tropico economy model:
//
// Every citizen has:
//   - A job (or unemployment)
//   - A home (or homeless)
//   - Individual needs (food, entertainment, healthcare, religion, etc.)
//   - Political faction membership
//   - Individual happiness score
//
// Wages:
//   Each building has a wage slider: Budget Cut / Economy / Normal / Rich / Filthy Rich
//   Higher wages -> better workers -> higher productivity -> more revenue
//   But also: higher wages -> higher costs
//
// Building Revenue:
//   Revenue = base_production * worker_quality * upgrade_bonus * manager_bonus
//   Cost = wages + maintenance
//   Profit = Revenue - Cost
//
// The "Swiss bank account" mechanic:
//   President can skim from treasury -> personal wealth
//   But drains city funds and angers citizens
//
// Trade prices fluctuate with world events and trade agreements
// Citizens vote based on individual happiness
// Factions have policy preferences (Capitalists want low taxes,
//   Communists want equality, etc.)
```

#### Victoria 3 (Paradox Interactive, 2022)

Victoria 3 has the most sophisticated economy in grand strategy:

```
// Victoria 3 economy model (simplified):
//
// Market system:
//   - Goods have buy/sell orders
//   - Price = base_price * (demand / supply)^(1/elasticity)
//   - Elasticity varies by good (food is inelastic, luxury goods elastic)
//
// Production method:
//   Each building has a production method determining:
//     Input goods, Output goods, Required workforce by type
//   Example: Iron Mine
//     Inputs: none (or tools at higher tech)
//     Outputs: 20 Iron per level
//     Workers: 5000 laborers per level
//     Alternative method (modern): needs tools, outputs 40 iron, needs 2000 machinists
//
// Wage determination:
//   wage = goods_price_of_subsistence_basket / required_hours
//   Workers negotiate based on: standard of living expectations,
//     political power (unions/suffrage), labor market tightness
//
// Standard of Living:
//   SoL = actual_consumption / expected_consumption (by wealth tier)
//   Pops with SoL > 1.0 are content
//   Pops with SoL < 0.5 face radicalization
//
// Key formulas:
//   GDP = sum(all_goods_produced * market_price)
//   Tax revenue = GDP * effective_tax_rate * collection_efficiency
//   Throughput = base_output * infrastructure_modifier * technology_modifier
//
// Price formula:
//   price = base_price * (demand_orders / supply_orders)^0.25
//   Note: exponent 0.25 means prices are quite inelastic
//   Doubling demand only increases price by ~19%
//
// Victoria 3 models:
//   - Individual pops with wealth, needs, political alignment
//   - Multi-good markets with partial equilibrium pricing
//   - Technology unlocking new production methods
//   - Trade routes with transport costs
//   - Interest groups (political factions) influencing policy
```

#### Offworld Trading Company (2016, Mohawk Games)

The purest economy game -- no military, pure market competition:

```
// Offworld Trading Company economy:
//
// Real-time market simulation:
//   Each good has a price determined by supply/demand
//   Price adjusts every few seconds
//   All players share the same market
//
// Price formula:
//   price = base_price * demand_multiplier / supply_multiplier
//   When you sell, price decreases
//   When you buy, price increases
//   The effect scales with quantity traded
//
// Key goods and base prices (approximate):
//   Water: $10     Iron: $15    Steel: $40    Electronics: $60
//   Food: $12      Carbon: $20  Glass: $35    Chemicals: $50
//   Oxygen: $8     Silicon: $18 Fuel: $45
//
// Market manipulation:
//   Buying large quantities drives price up (good if you have stockpile)
//   Selling large quantities crashes price (good if you want to bankrupt rivals)
//   "Black market" actions: EMP (disable buildings), mutiny (steal workers),
//     power surge (destroy power grid), underground nuke (claim land)
//
// The stock market:
//   Each company has a stock price based on: revenue, debt, assets
//   Players can buy each other's stock
//   Buying 100% of shares = hostile takeover = elimination
//   Stock price formula:
//     stock_price = (revenue * 4 + total_asset_value - debt) / shares_outstanding
//
// Brilliant design lesson: the market IS the game.
// No combat, no city services. Pure economic warfare.
//
// Applicable to Megacity:
// - Use real supply/demand pricing (already partially in market.rs)
// - Make economic information visible and important to player decisions
// - Economic volatility creates interesting decisions
```

### 5.5 Economic Cycle Implementation

The business cycle in a city builder should create waves of growth and contraction
that force the player to adapt:

```
struct EconomicCycle {
    phase: CyclePhase,
    phase_progress: f32,      // 0.0 to 1.0 within current phase
    gdp_growth_rate: f32,     // Current growth rate
    confidence_index: f32,    // 0-100 (consumer/business confidence)
    inflation_rate: f32,      // Annual rate
    interest_rate_environment: f32, // Central bank rate (affects bond costs)
}

enum CyclePhase {
    Expansion,    // Growth, falling unemployment, rising prices
    Peak,         // Maximum output, overheating risks
    Contraction,  // Falling output, rising unemployment
    Trough,       // Minimum output, recovery beginning
}

fn update_economic_cycle(
    cycle: &mut EconomicCycle,
    city: &CityStats,
    external_shocks: &[ExternalShock],
    dt: f32,
) {
    // Base cycle: sinusoidal with 4-8 year period
    // But modified by endogenous and exogenous factors

    let cycle_length_years = 6.0; // Average business cycle length
    let natural_progress_per_tick = dt / (cycle_length_years * 365.0);

    // Advance through phases
    cycle.phase_progress += natural_progress_per_tick;

    // External shocks can accelerate or reverse the cycle
    for shock in external_shocks {
        match shock {
            ExternalShock::RecessionTrigger => {
                cycle.phase = CyclePhase::Contraction;
                cycle.phase_progress = 0.0;
                cycle.confidence_index -= 20.0;
            }
            ExternalShock::BoomTrigger => {
                cycle.phase = CyclePhase::Expansion;
                cycle.phase_progress = 0.0;
                cycle.confidence_index += 15.0;
            }
            ExternalShock::IndustryCollapse { sector, magnitude } => {
                cycle.confidence_index -= magnitude * 30.0;
                cycle.gdp_growth_rate -= magnitude * 0.05;
            }
            ExternalShock::TechBoom { sector } => {
                cycle.confidence_index += 10.0;
                cycle.gdp_growth_rate += 0.02;
            }
        }
    }

    // Phase transitions
    if cycle.phase_progress >= 1.0 {
        cycle.phase_progress = 0.0;
        cycle.phase = match cycle.phase {
            CyclePhase::Expansion => CyclePhase::Peak,
            CyclePhase::Peak => CyclePhase::Contraction,
            CyclePhase::Contraction => CyclePhase::Trough,
            CyclePhase::Trough => CyclePhase::Expansion,
        };
    }

    // GDP growth rate by phase
    cycle.gdp_growth_rate = match cycle.phase {
        CyclePhase::Expansion => {
            // Growth accelerates through expansion
            0.01 + cycle.phase_progress * 0.03 // 1% to 4%
        }
        CyclePhase::Peak => {
            // Growth peaks then slows
            0.04 - cycle.phase_progress * 0.03 // 4% to 1%
        }
        CyclePhase::Contraction => {
            // Growth turns negative
            0.01 - cycle.phase_progress * 0.06 // 1% to -5%
        }
        CyclePhase::Trough => {
            // Recovery begins
            -0.05 + cycle.phase_progress * 0.06 // -5% to 1%
        }
    };

    // Confidence follows with a lag
    let target_confidence = match cycle.phase {
        CyclePhase::Expansion => 60.0 + cycle.phase_progress * 30.0,
        CyclePhase::Peak => 85.0 + cycle.phase_progress * 10.0,
        CyclePhase::Contraction => 80.0 - cycle.phase_progress * 50.0,
        CyclePhase::Trough => 30.0 + cycle.phase_progress * 20.0,
    };

    // Smooth toward target (inertia)
    cycle.confidence_index += (target_confidence - cycle.confidence_index) * 0.05 * dt;
    cycle.confidence_index = cycle.confidence_index.clamp(5.0, 100.0);

    // Inflation
    cycle.inflation_rate = match cycle.phase {
        CyclePhase::Expansion => 0.015 + cycle.phase_progress * 0.02,   // 1.5-3.5%
        CyclePhase::Peak => 0.03 + cycle.phase_progress * 0.02,         // 3-5%
        CyclePhase::Contraction => 0.03 - cycle.phase_progress * 0.03,  // 3% to 0%
        CyclePhase::Trough => -0.01 + cycle.phase_progress * 0.02,      // -1% to 1%
    };

    // Interest rate environment (follows inflation with a lag)
    let target_rate = (cycle.inflation_rate + 0.01).clamp(0.005, 0.08);
    cycle.interest_rate_environment += (target_rate - cycle.interest_rate_environment) * 0.02;
}
```

#### Effects of Economic Cycle on City Systems

```
fn apply_cycle_effects(
    cycle: &EconomicCycle,
    budget: &mut CityBudget,
    market: &mut HousingMarket,
    labor: &mut LaborMarket,
    immigration: &mut ImmigrationPressure,
) {
    let growth = cycle.gdp_growth_rate;
    let confidence = cycle.confidence_index / 100.0;

    // Tax revenue scales with economic cycle
    budget.monthly_income *= (1.0 + growth) as f64;

    // Housing demand
    if growth > 0.02 {
        // Boom: high demand, prices rise
        market.residential_price_index *= 1.0 + growth * 0.5;
        market.commercial_rent_index *= 1.0 + growth * 0.3;
    } else if growth < -0.01 {
        // Contraction: demand falls
        market.residential_price_index *= 1.0 + growth * 0.3;
        market.commercial_rent_index *= 1.0 + growth * 0.5;
    }

    // Labor market
    // Unemployment inversely tracks growth (Okun's law: 1% GDP = 2% unemployment)
    let okun_coefficient = 2.0;
    let natural_rate = 0.05;
    let target_unemployment = (natural_rate - growth * okun_coefficient).clamp(0.02, 0.25);

    // Smooth transition
    labor.unemployment_rate += (target_unemployment - labor.unemployment_rate) * 0.1;

    // Immigration
    // People move to cities during booms, leave during busts
    immigration.push_factor = growth * 0.5 + confidence * 0.3;

    // Development activity
    // New construction starts track confidence with a lag
    // This is implemented in the building_spawner: it should check
    // confidence before spawning new buildings
}
```

### 5.6 City-Specific Economic Models

#### The Export Base Model

Cities grow when their "export base" (goods/services sold outside the city) grows.
Local services (restaurants, dry cleaners) only redistribute money within the city.

```
fn calculate_export_base(sectors: &HashMap<Sector, SectorStats>) -> ExportBase {
    let mut export_employment = 0u32;
    let mut local_employment = 0u32;

    for (sector, stats) in sectors {
        let export_share = match sector {
            Sector::Manufacturing => 0.80,  // Mostly export
            Sector::Technology => 0.70,     // Mostly export (software, etc.)
            Sector::Professional => 0.40,   // Mix of local and export clients
            Sector::Healthcare => 0.15,     // Mostly local (some medical tourism)
            Sector::Education => 0.30,      // Universities attract outside students
            Sector::Government => 0.20,     // Federal/state = export; local = not
            Sector::Retail => 0.05,         // Mostly local
            Sector::Hospitality => 0.30,    // Tourism = export
            Sector::Construction => 0.05,   // Mostly local
            Sector::Agriculture => 0.90,    // Mostly export
        };

        export_employment += (stats.employment as f32 * export_share) as u32;
        local_employment += (stats.employment as f32 * (1.0 - export_share)) as u32;
    }

    ExportBase {
        export_employment,
        local_employment,
        base_ratio: export_employment as f32 / local_employment.max(1) as f32,
    }
}
```

#### Growth Machine Theory

Harvey Molotch's "growth machine" theory (1976): Cities are organized around
the goal of increasing land values. The "growth coalition" (real estate developers,
construction unions, local media, utility companies) pushes for policies that
promote growth because they profit from rising land values.

In Megacity, this translates to:
- **Developers** pressure for upzoning, tax breaks, and infrastructure investment
- **Citizens** want affordable housing and low taxes
- **Businesses** want low costs and skilled workers
- These groups have conflicting interests that create interesting gameplay

### 5.7 Real Estate Cycles (Separate from Business Cycles)

Real estate has its own cycle, typically 15-20 years, only loosely correlated with
the general business cycle:

```
Phase 1: Recovery (2-4 years)
  - Vacancy declining from peak
  - No new construction
  - Rents stabilizing
  - Investors begin acquiring distressed properties

Phase 2: Expansion (3-6 years)
  - Vacancy below natural rate
  - Rents rising
  - New construction begins
  - Easy financing available
  - Development becomes speculative

Phase 3: Hypersupply (2-4 years)
  - New construction continues from Phase 2 commitments (2-3 year lag)
  - But demand growth slows
  - Vacancy starts rising
  - Rent growth slows, then stops
  - Overbuilding becomes apparent

Phase 4: Recession (2-4 years)
  - Vacancy rising rapidly
  - Rents declining
  - New construction stops
  - Property values fall
  - Foreclosures, distressed sales
  - Developers go bankrupt

struct RealEstateCycle {
    phase: REPhase,
    phase_progress: f32,
    construction_pipeline: Vec<PipelineProject>, // Projects under construction
    vacancy_trend: f32,
    rent_trend: f32,
}

fn real_estate_cycle_update(
    cycle: &mut RealEstateCycle,
    market: &HousingMarket,
    dt: f32,
) {
    // The critical mechanism: construction has a LAG
    // Decisions to build are made during expansion
    // Buildings complete 2-3 years later, potentially during hypersupply
    // This lag is what causes overbuilding

    let construction_lag_years = 2.5;

    // Phase detection based on market conditions
    let vacancy_direction = market.residential_vacancy - 0.05; // relative to natural
    let rent_direction = market.residential_price_index - 100.0; // relative to baseline

    let detected_phase = if vacancy_direction < -0.02 && rent_direction > 5.0 {
        REPhase::Expansion
    } else if vacancy_direction < 0.0 && rent_direction < 5.0 {
        REPhase::Recovery
    } else if vacancy_direction > 0.0 && rent_direction > 0.0 {
        REPhase::Hypersupply
    } else {
        REPhase::Recession
    };

    // Smooth transition
    if detected_phase != cycle.phase {
        cycle.phase = detected_phase;
        cycle.phase_progress = 0.0;
    }

    cycle.phase_progress += dt / (3.0 * 365.0); // ~3 years per phase

    // Developer construction decisions
    let should_build = match cycle.phase {
        REPhase::Recovery => cycle.phase_progress > 0.7, // Late recovery
        REPhase::Expansion => true,                       // Build aggressively
        REPhase::Hypersupply => cycle.phase_progress < 0.3, // Still completing
        REPhase::Recession => false,                      // Stop everything
    };

    // This feeds into the building_spawner system's decision to spawn new buildings
}
```

---

## 6. Integration: Connecting All Systems

### 6.1 The Full Economic Loop

All the systems described above interconnect in a feedback loop:

```
Economic Cycle (macro)
  |
  +--> Labor Market
  |      |
  |      +--> Wages --> Consumer spending --> Commercial revenue
  |      |
  |      +--> Unemployment --> Crime, happiness, emigration
  |
  +--> Housing Market
  |      |
  |      +--> Vacancy rates --> Rent levels --> Cost of living
  |      |
  |      +--> Construction activity --> Jobs, impact fees
  |
  +--> Land Value
  |      |
  |      +--> Property tax revenue --> City services
  |      |
  |      +--> Development decisions --> What gets built
  |      |
  |      +--> Neighborhood character --> Who lives/works there
  |
  +--> Municipal Finance
         |
         +--> Tax rates --> Demand modifier (SimCity curve)
         |
         +--> Service funding --> Service quality --> Happiness, land value
         |
         +--> Debt/bond capacity --> Infrastructure investment capacity
         |
         +--> Budget surplus/deficit --> Credit rating --> Borrowing cost
```

### 6.2 Performance-Optimized Tick Schedule

Not all systems need to update every frame. A tiered schedule:

```
Every game tick (60/sec):
  - Citizen movement/pathfinding (already in movement.rs)
  - Traffic flow updates
  - Building occupancy changes

Every 10 ticks (6/sec):
  - Labor market matching (incremental)
  - Vacancy rate calculation

Every 100 ticks (SlowTickTimer, ~0.6/sec):
  - Land value recalculation (hedonic model)
  - Spatial smoothing pass
  - Market price updates (already in market.rs)
  - Economic cycle phase update
  - Housing market rent adjustment

Every 1000 ticks (~0.06/sec, roughly every 15 seconds):
  - Tax collection (monthly)
  - Service budget allocation
  - Credit rating recalculation
  - Bond interest payments
  - TIF district updates
  - Developer pro forma evaluation (what to build)
  - Real estate cycle phase check
  - Agglomeration economy recalculation
  - Export base analysis
```

### 6.3 Data Structures Summary

```
// Core economic resources (Bevy ECS Resources)

#[derive(Resource)]
struct Economy {
    cycle: EconomicCycle,
    labor_market: LaborMarket,
    housing_market: HousingMarket,
    real_estate_cycle: RealEstateCycle,
    export_base: ExportBase,
    dutch_disease_status: DutchDiseaseStatus,
}

#[derive(Resource)]
struct MunicipalFinance {
    budget: CityBudget,               // Existing
    extended: ExtendedBudget,          // Existing
    bonds: Vec<MunicipalBond>,         // GO and revenue bonds
    credit_rating: CreditRating,
    tif_districts: Vec<TIFDistrict>,
    property_tax_config: PropertyTaxConfig,
    impact_fee_schedule: ImpactFeeSchedule,
}

#[derive(Resource)]
struct EnhancedLandValue {
    grid: Vec<LandValueCell>,          // f32 per cell instead of u8
    hedonic_weights: HedonicWeights,
    cbd_center: (usize, usize),       // Could be dynamically detected
    transit_stops: Vec<(usize, usize)>,
}
```

### 6.4 Existing Code Integration Points

The current codebase has these hooks where enhanced economics should plug in:

1. **`economy.rs` :: `collect_taxes()`** -- Replace flat per-citizen tax with
   property tax assessment using `PropertyTaxConfig` and per-building value from
   `EnhancedLandValue`.

2. **`budget.rs` :: `ExtendedBudget`** -- Extend with `MunicipalBond` support
   alongside existing `Loan` system. Add `CreditRating` tracking.

3. **`land_value.rs` :: `update_land_value()`** -- Replace additive modifier system
   with hedonic pricing model. Change from `u8` to `f32` grid. Add spatial smoothing.

4. **`market.rs` :: `update_market_prices()`** -- Already has supply/demand pricing
   and market events. Extend with economic cycle modifiers and sector-specific
   supply/demand balancing.

5. **`buildings.rs` :: building spawner** -- Add developer pro forma check before
   spawning. Buildings should only appear when financially feasible.

6. **`citizen_spawner.rs`** -- Immigration/emigration should respond to economic
   conditions (unemployment rate, housing availability, wage levels).

7. **`happiness.rs`** -- Economic factors (employment, housing cost burden,
   commute time) should feed into happiness calculation.

### 6.5 Calibration Guidelines

Getting the numbers right is critical for game feel. Guidelines:

| Metric                        | "Feels Right" Range | Too Low Breaks...      | Too High Breaks...     |
|-------------------------------|--------------------:|------------------------|------------------------|
| Tax rate default              |              7-10%  | Revenue (bankruptcy)   | Growth (empty city)    |
| Unemployment                  |              3-7%   | Wage spiral            | Social collapse        |
| Vacancy rate (res)            |              3-8%   | Housing crisis         | Abandonment            |
| GDP growth                    |           1-4%/year | Stagnation feel        | Unrealistic boom       |
| Inflation                     |           1-3%/year | Deflationary spiral    | Prices feel unstable   |
| Bond interest                 |             3-6%    | Free money (too easy)  | Can't build anything   |
| Land value range              |           1-500+    | No differentiation     | UI unreadable          |
| Property tax as % of budget   |            30-50%   | Tax irrelevant         | Nothing else matters   |
| Agglomeration bonus           |             5-25%   | Cities don't matter    | Unrealistic snowball   |
| Business cycle length         |          4-8 years  | Whiplash               | Boring, never changes  |

### 6.6 UI Requirements

The economic system generates data that must be surfaced to the player:

1. **Budget panel** (existing in `ui/src/toolbar.rs`): Expand with:
   - Income/expense breakdown by category
   - Bond management interface
   - Credit rating display
   - TIF district management

2. **Overlay map** (existing in `rendering/src/overlay.rs`): Add:
   - Land value heatmap (enhanced, f32 resolution)
   - Vacancy rate overlay
   - Unemployment by district
   - Economic cycle indicator

3. **Graphs/charts**: Time series for:
   - Tax revenue history
   - Land value trends
   - Unemployment rate
   - Housing price index
   - GDP growth rate

4. **Advisor notifications**: Alert player when:
   - Credit rating changes
   - Economic cycle phase shifts
   - Housing shortage or glut detected
   - Budget deficit exceeds threshold
   - Bond payment approaching
   - Dutch disease risk detected

---

## 7. Appendix: Reference Data Tables

### 7.1 US Median Home Prices by City Size (2023 approximate)

| City Size Category       | Median Home Price | Monthly Property Tax | Annual Growth |
|--------------------------|------------------:|---------------------:|--------------:|
| Rural (< 10K)            |       $180,000    |              $150    |         2-3%  |
| Small city (10-50K)      |       $220,000    |              $200    |         3-4%  |
| Midsized city (50-250K)  |       $280,000    |              $280    |         3-5%  |
| Large city (250K-1M)     |       $350,000    |              $350    |         4-6%  |
| Major metro (1M+)        |       $450,000    |              $450    |         4-8%  |
| Superstar city (NYC, SF) |       $800,000+   |              $700+   |         5-10% |

### 7.2 Construction Costs per Square Foot (2023 approximate)

| Building Type               | Low Market | Average | High Market | Premium Market |
|-----------------------------|----------:|---------:|------------:|---------------:|
| Single family residential   |      $120 |    $180  |       $250  |          $400+ |
| Multifamily (wood frame)    |      $150 |    $200  |       $280  |          $400  |
| Multifamily (concrete/steel)|      $200 |    $300  |       $450  |          $600+ |
| Retail (strip)              |      $100 |    $150  |       $200  |          $300  |
| Retail (mall/specialty)     |      $150 |    $250  |       $400  |          $600  |
| Office (Class B)            |      $150 |    $250  |       $350  |          $500  |
| Office (Class A)            |      $250 |    $400  |       $600  |          $800+ |
| Industrial/warehouse        |       $60 |    $100  |       $150  |          $200  |
| Hospital                    |      $350 |    $550  |       $750  |        $1,000  |
| School                      |      $200 |    $350  |       $500  |          $650  |

### 7.3 Operating Expense Ratios by Property Type

| Property Type              | Operating Expense as % of Revenue |
|----------------------------|----------------------------------:|
| Single family (owner)      |                         15-25%    |
| Multifamily (rental)       |                         35-45%    |
| Office (full service)      |                         40-50%    |
| Retail (NNN)               |                         10-15%    |
| Industrial (NNN)           |                          8-12%    |
| Hotel                      |                         55-70%    |

### 7.4 Service Cost per Capita (Annual, 2023 approximate)

| Service                    | Low-Cost City | Average City | High-Cost City |
|----------------------------|-------------:|-------------:|---------------:|
| Police                     |         $200 |         $350 |           $600 |
| Fire/EMS                   |         $120 |         $200 |           $400 |
| Roads/transportation       |         $150 |         $300 |           $500 |
| Water/sewer                |         $100 |         $200 |           $350 |
| Trash/sanitation           |          $80 |         $150 |           $250 |
| Parks/recreation           |          $50 |         $120 |           $250 |
| Libraries                  |          $20 |          $40 |            $80 |
| General government         |         $100 |         $200 |           $400 |
| **Total per capita**       |       **$820**|     **$1,560**|       **$2,830**|

### 7.5 Employment Multipliers by Sector (Updated)

| Sector               | Direct Jobs | Indirect Jobs | Induced Jobs | Total Multiplier |
|----------------------|------------:|--------------:|-------------:|-----------------:|
| Software/IT          |        1.00 |          1.90 |         1.58 |             4.48 |
| Biotech/Pharma       |        1.00 |          2.10 |         1.42 |             4.52 |
| Finance/Insurance    |        1.00 |          1.50 |         1.30 |             3.80 |
| Advanced Manufacturing|       1.00 |          1.30 |         0.90 |             3.20 |
| Basic Manufacturing  |        1.00 |          0.80 |         0.70 |             2.50 |
| Healthcare           |        1.00 |          0.90 |         0.70 |             2.60 |
| Higher Education     |        1.00 |          0.70 |         0.80 |             2.50 |
| Retail               |        1.00 |          0.30 |         0.30 |             1.60 |
| Food Service         |        1.00 |          0.20 |         0.20 |             1.40 |

(Indirect = supplier chain jobs; Induced = jobs from worker spending)

---

*This document is intended as a reference for implementing deep economic simulation
in Megacity. Not all systems need to be implemented simultaneously -- prioritize
property tax assessment, enhanced land value, and vacancy dynamics as they have the
highest impact on gameplay depth with moderate implementation complexity.*
