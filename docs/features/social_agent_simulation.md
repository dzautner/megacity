# Social Agent Simulation: Deep Implementation Guide

## Table of Contents

1. [Agent Demographics and Identity](#1-agent-demographics-and-identity)
2. [Life Stages and Transition Probabilities](#2-life-stages-and-transition-probabilities)
3. [Daily Schedule Generation](#3-daily-schedule-generation)
4. [Decision Making Models](#4-decision-making-models)
5. [Happiness Modeling](#5-happiness-modeling)
6. [Social Dynamics: Segregation](#6-social-dynamics-segregation)
7. [Crime Simulation](#7-crime-simulation)
8. [Health and Disease](#8-health-and-disease)
9. [Education Pipeline](#9-education-pipeline)
10. [Immigration and Migration](#10-immigration-and-migration)
11. [Homelessness](#11-homelessness)
12. [Social Mobility](#12-social-mobility)
13. [Governance and Politics](#13-governance-and-politics)
14. [How Commercial Games Do It](#14-how-commercial-games-do-it)
15. [Performance: ECS Patterns for 100K+ Agents](#15-performance-ecs-patterns-for-100k-agents)
16. [Integration with Existing Megacity Systems](#16-integration-with-existing-megacity-systems)

---

## 1. Agent Demographics and Identity

### 1.1 Demographic Attributes

Every citizen agent carries a demographic profile that drives all downstream behavior -- where they choose to live, what jobs they seek, how they vote, and what makes them happy or miserable. The existing `CitizenDetails` component stores age, gender, education (0-3), happiness, health, salary, and savings. A full social simulation extends this substantially.

#### Core Demographic Component

```rust
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Demographics {
    // Identity
    pub age: u8,                    // 0-100
    pub gender: Gender,             // Male, Female
    pub ethnicity: EthnicGroup,     // For Schelling segregation model
    pub religion: Religion,         // Affects value priorities, voting

    // Socioeconomic
    pub education_level: EducationLevel,  // None, Elementary, HighSchool, Bachelors, Masters, Doctorate
    pub income_class: IncomeClass,        // Poverty, LowIncome, LowerMiddle, UpperMiddle, HighIncome, Wealthy
    pub occupation: Occupation,           // Unemployed, BlueCollar, WhiteCollar, Professional, Executive, Retired
    pub years_of_experience: u8,          // Affects salary within occupation band

    // Family
    pub marital_status: MaritalStatus,    // Single, Partnered, Married, Divorced, Widowed
    pub household_size: u8,               // 1-8 typical
    pub num_children: u8,                 // Dependent children
    pub num_dependents: u8,               // Elderly dependents

    // Financial
    pub monthly_income: f32,              // Gross salary
    pub monthly_expenses: f32,            // Rent + food + transport + childcare + ...
    pub net_worth: f32,                   // Assets minus debts (can be negative)
    pub debt: f32,                        // Outstanding loans
    pub rent_burden: f32,                 // Rent as fraction of income (>0.3 = burdened)
}
```

#### Education Level Distribution

Real-world distributions (US Census 2020, adapted for game balance):

| Education Level | Real % | Game % | Base Salary Multiplier |
|----------------|--------|--------|----------------------|
| None/Some Elementary | 5% | 5% | 0.5x |
| Elementary Complete | 10% | 10% | 0.7x |
| High School | 35% | 35% | 1.0x |
| Bachelors | 25% | 25% | 1.8x |
| Masters | 15% | 15% | 2.5x |
| Doctorate/Professional | 10% | 10% | 3.5x |

The existing system uses education 0-3. Extending to 0-5 provides better granularity for job matching and income stratification.

#### Income Distribution

Income follows a log-normal distribution, which produces the characteristic right-skewed shape seen in real economies. The Gini coefficient controls inequality:

```rust
/// Generate income from log-normal distribution
fn generate_income(education: EducationLevel, experience: u8, rng: &mut impl Rng) -> f32 {
    let base = education.salary_multiplier() * CITY_MEDIAN_INCOME;
    let experience_bonus = 1.0 + (experience as f32 * 0.02); // 2% per year
    let mu = (base * experience_bonus).ln();
    let sigma = 0.4; // Controls spread; higher = more inequality
    let log_normal: f64 = (mu as f64 + sigma * rng.sample::<f64, _>(StandardNormal)).exp();
    log_normal as f32
}

/// City-level Gini coefficient calculation
/// Gini = 0.0 (perfect equality) to 1.0 (one person has everything)
/// US cities typically 0.40-0.55, Scandinavian ~0.25-0.30
fn compute_gini(incomes: &[f32]) -> f32 {
    let n = incomes.len() as f32;
    if n < 2.0 { return 0.0; }
    let mut sorted = incomes.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mean = sorted.iter().sum::<f32>() / n;
    if mean == 0.0 { return 0.0; }
    let mut sum = 0.0;
    for (i, &income) in sorted.iter().enumerate() {
        sum += (2.0 * (i + 1) as f32 - n - 1.0) * income;
    }
    sum / (n * n * mean)
}
```

#### Ethnic Groups and Cultural Identity

For the Schelling segregation model (see Section 6), citizens need an ethnic/cultural group attribute. This should be handled sensitively -- using abstract groups avoids real-world mapping while still enabling meaningful simulation:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EthnicGroup {
    GroupA,    // "Founders" - initial settlement population
    GroupB,    // "Northern immigrants"
    GroupC,    // "Coastal traders"
    GroupD,    // "Mountain folk"
    GroupE,    // "Southern settlers"
    Mixed,     // Children of mixed-group parents
}
```

These abstract groups carry no real-world connotations but produce authentic segregation dynamics when combined with in-group preference parameters.

### 1.2 Household Composition

Individual agents are grouped into households that share a dwelling unit. This is critical for realistic housing demand (a city of 100K people is ~40K households, not 100K housing units).

```rust
#[derive(Component, Debug, Clone)]
pub struct Household {
    pub members: Vec<Entity>,           // All citizen entities in this household
    pub head_of_household: Entity,      // Primary decision-maker
    pub household_type: HouseholdType,
    pub combined_income: f32,
    pub dwelling: Entity,               // Building entity
    pub vehicles: u8,                   // For mode choice
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HouseholdType {
    SinglePerson,           // 28% of US households
    CoupleNoChildren,       // 25%
    NuclearFamily,          // 20% (2 parents + children)
    SingleParent,           // 9%
    ExtendedFamily,         // 5% (3+ generations)
    Roommates,              // 8% (unrelated adults)
    Elderly,                // 5% (1-2 seniors, 65+)
}
```

Household type determines housing demand (a family of 4 needs 2+ bedrooms), car ownership (suburban families: 2 cars; urban singles: 0-1), school enrollment, and service needs. The household acts as the economic unit for rent affordability calculations:

```
Affordable monthly rent = Combined household income * 0.30
```

When rent exceeds 30% of household income, the `rent_burden` flag triggers stress, reduced savings, and eventual relocation or homelessness.

### 1.3 Agent Generation at City Founding

When a new city starts, the initial population should reflect a plausible founding demographic rather than random noise:

```rust
fn generate_founding_population(count: u32, rng: &mut impl Rng) -> Vec<CitizenTemplate> {
    let mut citizens = Vec::with_capacity(count as usize);
    for _ in 0..count {
        // Founding populations skew young-adult (pioneers, young families)
        let age = weighted_random_age(rng, &[
            (18..=25, 0.25),  // Young adults seeking opportunity
            (26..=35, 0.35),  // Prime working age
            (36..=45, 0.20),  // Established workers
            (46..=55, 0.10),  // Experienced professionals
            (56..=65, 0.07),  // Pre-retirement
            (66..=80, 0.03),  // Retirees (rare in new cities)
        ]);
        // Education skews practical for a new settlement
        let education = weighted_random(rng, &[
            (EducationLevel::HighSchool, 0.40),
            (EducationLevel::Bachelors, 0.30),
            (EducationLevel::Elementary, 0.15),
            (EducationLevel::Masters, 0.10),
            (EducationLevel::Doctorate, 0.05),
        ]);
        citizens.push(CitizenTemplate { age, education, /* ... */ });
    }
    citizens
}
```

As the city matures, the age pyramid naturally evolves through births, aging, death, immigration, and emigration.

---

## 2. Life Stages and Transition Probabilities

### 2.1 Life Stage State Machine

The existing `LifeStage` enum defines six stages: Child (0-5), SchoolAge (6-17), YoungAdult (18-25), Adult (26-54), Senior (55-64), Retired (65+). A deeper simulation adds probabilistic transitions and life events at each stage boundary.

```
                    Birth
                      |
                      v
    +----------+    age 6    +------------+    age 18    +-------------+
    |  Child   | ----------> | SchoolAge  | ----------> | YoungAdult  |
    | (0-5)    |             | (6-17)     |             | (18-25)     |
    +----------+             +------------+             +-------------+
                                   |                          |
                              dropout (2-8%)              age 26 or
                                   |                    life event
                                   v                          |
                            [Enter workforce               v
                             unskilled]            +-------------+
                                                   |   Adult     |
                                                   |  (26-54)    |
                                                   +-------------+
                                                         |
                                                    age 55 or
                                                   early retirement
                                                         |
                                                         v
                                                   +-------------+
                                                   |   Senior    |
                                                   |  (55-64)    |
                                                   +-------------+
                                                         |
                                                    age 65 or
                                                   forced retirement
                                                         |
                                                         v
                                                   +-------------+
                                                   |  Retired    |
                                                   |  (65+)      |
                                                   +-------------+
                                                         |
                                                    death (probabilistic)
                                                         |
                                                         v
                                                      [Despawn]
```

### 2.2 Life Event Probabilities

Each year (game-year = AGING_INTERVAL_DAYS ticks), agents roll for life events. These probabilities are calibrated against real demographic data:

#### Education Transitions

```rust
struct EducationTransitions {
    // Probability of progressing to next education level
    // Conditional on current stage and city's education quality
    fn advance_probability(
        current: EducationLevel,
        age: u8,
        city_education_quality: f32,  // 0.0-1.0, from school coverage/budget
        family_income: IncomeClass,
        personality_ambition: f32,
    ) -> f32 {
        let base = match (current, age) {
            // Elementary -> High School: nearly universal
            (EducationLevel::Elementary, 14..=18) => 0.92,
            // High School -> Bachelors: varies widely by income
            (EducationLevel::HighSchool, 18..=22) => match family_income {
                IncomeClass::Poverty => 0.15,
                IncomeClass::LowIncome => 0.25,
                IncomeClass::LowerMiddle => 0.40,
                IncomeClass::UpperMiddle => 0.65,
                IncomeClass::HighIncome => 0.80,
                IncomeClass::Wealthy => 0.90,
            },
            // Bachelors -> Masters: depends on field and ambition
            (EducationLevel::Bachelors, 22..=30) => 0.15 + personality_ambition * 0.15,
            // Masters -> Doctorate: rare
            (EducationLevel::Masters, 24..=35) => 0.05 + personality_ambition * 0.10,
            _ => 0.0,
        };
        // City education quality modifies probability by +/- 20%
        (base * (0.8 + 0.4 * city_education_quality)).clamp(0.0, 1.0)
    }
}
```

#### Marriage and Partnership

```rust
fn marriage_probability(age: u8, gender: Gender, is_single: bool) -> f32 {
    if !is_single { return 0.0; }
    // Probabilities per year, calibrated to median marriage age of ~28-30
    match age {
        18..=21 => 0.02,
        22..=25 => 0.06,
        26..=29 => 0.10,   // Peak marriage years
        30..=34 => 0.08,
        35..=39 => 0.05,
        40..=49 => 0.03,
        50..=64 => 0.01,
        65..=100 => 0.005,
        _ => 0.0,
    }
}

fn divorce_probability(years_married: u8) -> f32 {
    // Annual divorce probability peaks in years 5-8
    // Overall ~40-50% of marriages end in divorce (real US data)
    match years_married {
        0..=2 => 0.02,
        3..=5 => 0.035,
        6..=8 => 0.04,    // Peak risk
        9..=15 => 0.025,
        16..=25 => 0.015,
        _ => 0.008,        // Long marriages are stable
    }
}
```

#### Fertility

```rust
fn birth_probability(
    mother_age: u8,
    existing_children: u8,
    income_class: IncomeClass,
    housing_size: u8,   // Available bedrooms
) -> f32 {
    if existing_children as u8 >= housing_size { return 0.0; } // No room

    let age_factor = match mother_age {
        15..=19 => 0.02,   // Teen pregnancy (reduced in game)
        20..=24 => 0.08,
        25..=29 => 0.12,   // Peak fertility window
        30..=34 => 0.10,
        35..=39 => 0.05,
        40..=44 => 0.01,
        _ => 0.0,
    };

    // Higher income = fewer children (demographic transition)
    let income_modifier = match income_class {
        IncomeClass::Poverty => 1.4,
        IncomeClass::LowIncome => 1.2,
        IncomeClass::LowerMiddle => 1.0,
        IncomeClass::UpperMiddle => 0.8,
        IncomeClass::HighIncome => 0.6,
        IncomeClass::Wealthy => 0.5,
    };

    // Diminishing desire for additional children
    let parity_modifier = match existing_children {
        0 => 1.0,
        1 => 0.8,
        2 => 0.3,
        3 => 0.1,
        _ => 0.02,
    };

    age_factor * income_modifier * parity_modifier
}
```

This produces total fertility rates (TFR) of roughly 1.5-2.5 depending on city income distribution, matching real-world developed-nation ranges. The player can influence TFR through childcare services, housing availability, and economic conditions.

#### Mortality

The existing system applies death probability after age 70 with a linear ramp. A more realistic model uses the Gompertz-Makeham mortality law:

```rust
/// Gompertz-Makeham mortality: hazard rate = a * exp(b * age) + c
/// Parameters fitted to modern developed-nation life tables:
///   a = 0.00002 (baseline senescent mortality)
///   b = 0.085   (exponential aging rate)
///   c = 0.0005  (age-independent background mortality: accidents, etc.)
fn annual_death_probability(age: u8, health: f32) -> f32 {
    let a = 0.00002_f32;
    let b = 0.085_f32;
    let c = 0.0005_f32;

    let base_hazard = a * (b * age as f32).exp() + c;

    // Health modifier: perfect health (100) = 0.5x, critical health (0) = 3.0x
    let health_modifier = 3.0 - 2.5 * (health / 100.0);

    // Clamp to reasonable range
    (base_hazard * health_modifier).clamp(0.0, 0.95)
}

// Example outputs:
//   age 20, health 80: 0.00056 (1 in 1786 per year)
//   age 40, health 80: 0.00122 (1 in 820)
//   age 60, health 80: 0.00853 (1 in 117)
//   age 70, health 60: 0.0462  (1 in 22)
//   age 80, health 40: 0.394   (1 in 2.5)
//   age 90, health 30: 0.95    (almost certain)
```

This produces a life expectancy of roughly 78-82 years at perfect health with healthcare access, dropping to ~65-70 without healthcare -- matching real-world patterns and giving players a clear incentive to build hospitals.

### 2.3 Career Progression

Workers advance through career stages that affect income and job satisfaction:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CareerStage {
    EntryLevel,     // 0-3 years experience
    Junior,         // 3-7 years
    MidCareer,      // 7-15 years
    Senior,         // 15-25 years
    Executive,      // 25+ years (only top 5-10%)
}

impl CareerStage {
    pub fn salary_multiplier(&self) -> f32 {
        match self {
            Self::EntryLevel => 1.0,
            Self::Junior => 1.3,
            Self::MidCareer => 1.7,
            Self::Senior => 2.2,
            Self::Executive => 3.5,
        }
    }

    /// Probability of promotion per year
    pub fn promotion_probability(&self, performance: f32, ambition: f32) -> f32 {
        let base = match self {
            Self::EntryLevel => 0.20,    // Fast early promotions
            Self::Junior => 0.12,
            Self::MidCareer => 0.06,
            Self::Senior => 0.02,        // Very competitive
            Self::Executive => 0.0,      // Already at top
        };
        base * (0.5 + performance * 0.5) * (0.7 + ambition * 0.3)
    }
}
```

### 2.4 Life Events Summary Table

| Event | Age Range | Annual Probability | Dependencies |
|-------|-----------|-------------------|--------------|
| Start school | 6 | 1.0 (automatic) | Elementary school exists |
| Drop out of school | 14-17 | 0.02-0.08 | Low income, low ambition |
| Graduate high school | 18 | 0.92 (if enrolled) | School quality |
| Enter university | 18-22 | 0.15-0.90 | Income, ambition, university exists |
| Graduate university | 22-26 | 0.75 (if enrolled) | GPA proxy, university quality |
| First job | 16-25 | 0.8+ | Jobs available, education matches |
| Marriage | 22-40 | 0.02-0.10 | Partner available, income stable |
| First child | 22-35 | 0.02-0.12 | Married/partnered, housing |
| Divorce | Any married | 0.008-0.04 | Happiness, years married |
| Job loss | Any working | 0.02-0.10 | Economic conditions, firm closure |
| Career change | 25-50 | 0.03 | Unhappiness at work, ambition |
| Early retirement | 55-64 | 0.05-0.15 | Savings sufficient, health |
| Retirement | 65 | 0.90+ | Pension system |
| Widowhood | 60+ | varies | Partner death probability |
| Death | Any | Gompertz curve | Age, health, healthcare access |

---

## 3. Daily Schedule Generation

### 3.1 Activity-Based Travel Demand Models

The existing citizen state machine (`CitizenState` enum) handles basic transitions: AtHome -> CommutingToWork -> Working -> CommutingHome, with branching to Shopping and Leisure. A full activity-based model generates a complete daily itinerary that determines *when*, *where*, and *how long* each citizen does each activity.

Real transportation planning uses Activity-Based Models (ABMs) like TRANSIMS, MATSim, and CEMDAP. The key insight is that travel is *derived demand* -- people don't travel for its own sake; they travel to perform activities. Modeling activities first, then deriving trips, produces more realistic traffic patterns than trip-generation models.

#### Daily Activity Chain

```rust
#[derive(Debug, Clone)]
pub struct DailySchedule {
    pub activities: Vec<ScheduledActivity>,
    pub generated_day: u32,      // Game day this schedule was generated for
}

#[derive(Debug, Clone)]
pub struct ScheduledActivity {
    pub activity_type: ActivityType,
    pub start_hour: f32,         // 0.0-24.0 (fractional hours)
    pub duration_hours: f32,     // Expected duration
    pub location: ActivityLocation,
    pub flexibility: f32,        // 0.0 = rigid (work), 1.0 = fully flexible (leisure)
    pub priority: u8,            // Higher = harder to skip
}

#[derive(Debug, Clone, Copy)]
pub enum ActivityType {
    Sleep,
    Work,
    School,
    Shopping,           // Grocery, errands
    Leisure,            // Parks, entertainment
    Socializing,        // Visit friends/family
    Healthcare,         // Doctor visits
    ChildCare,          // Drop off / pick up children
    PersonalBusiness,   // Government offices, banking
    Exercise,           // Gym, running
    Worship,            // Religious services (weekly)
    Eating,             // Restaurant, cafe
}

#[derive(Debug, Clone)]
pub enum ActivityLocation {
    Home,
    Workplace,
    School(Entity),
    Specific(usize, usize),    // Grid coordinates of a known destination
    NearestOfType(ServiceType), // Find closest matching service
    SocialContact(Entity),      // Visit another citizen
}
```

### 3.2 Schedule Generation Algorithm

Schedule generation uses a hierarchical approach: mandatory activities (work, school) are placed first as "skeleton" events, then discretionary activities (shopping, leisure) fill remaining time slots.

```rust
fn generate_daily_schedule(
    citizen: &CitizenDetails,
    personality: &Personality,
    needs: &Needs,
    household: &Household,
    day_of_week: DayOfWeek,
    rng: &mut impl Rng,
) -> DailySchedule {
    let mut activities = Vec::with_capacity(8);
    let life_stage = citizen.life_stage();

    // === LAYER 1: Mandatory activities ===

    // Sleep (everyone)
    let wake_time = generate_wake_time(life_stage, personality, rng);
    let sleep_time = generate_sleep_time(life_stage, personality, rng);
    activities.push(ScheduledActivity {
        activity_type: ActivityType::Sleep,
        start_hour: sleep_time,
        duration_hours: (24.0 + wake_time - sleep_time) % 24.0,
        location: ActivityLocation::Home,
        flexibility: 0.1,
        priority: 10,
    });

    // Work (if employed and workday)
    if life_stage.can_work() && citizen.occupation != Occupation::Unemployed
        && day_of_week.is_workday()
    {
        let (work_start, work_duration) = generate_work_hours(
            citizen.occupation,
            personality.ambition,
            rng,
        );
        activities.push(ScheduledActivity {
            activity_type: ActivityType::Work,
            start_hour: work_start,
            duration_hours: work_duration,
            location: ActivityLocation::Workplace,
            flexibility: 0.1,
            priority: 9,
        });
    }

    // School (if school-age and weekday)
    if life_stage.should_attend_school() && day_of_week.is_workday() {
        activities.push(ScheduledActivity {
            activity_type: ActivityType::School,
            start_hour: 8.0,
            duration_hours: 7.0,
            location: ActivityLocation::NearestOfType(ServiceType::ElementarySchool),
            flexibility: 0.0,
            priority: 9,
        });
    }

    // === LAYER 2: Maintenance activities ===

    // Childcare (if parent with young children)
    if citizen.num_children > 0 && has_school_age_children(household) {
        activities.push(ScheduledActivity {
            activity_type: ActivityType::ChildCare,
            start_hour: 7.5,
            duration_hours: 0.5,
            location: ActivityLocation::NearestOfType(ServiceType::Kindergarten),
            flexibility: 0.2,
            priority: 8,
        });
    }

    // Shopping (probability-based, 2-3 times per week)
    if rng.gen::<f32>() < shopping_probability(needs.hunger, day_of_week) {
        let shop_time = find_free_slot(&activities, 1.0, wake_time, sleep_time, rng);
        if let Some(start) = shop_time {
            activities.push(ScheduledActivity {
                activity_type: ActivityType::Shopping,
                start_hour: start,
                duration_hours: 0.75,
                location: ActivityLocation::NearestOfType(ServiceType::Commercial),
                flexibility: 0.8,
                priority: 5,
            });
        }
    }

    // === LAYER 3: Discretionary activities ===

    // Leisure (driven by fun need)
    if needs.fun < 50.0 || (rng.gen::<f32>() < personality.sociability * 0.3) {
        let leisure_time = find_free_slot(&activities, 1.5, wake_time, sleep_time, rng);
        if let Some(start) = leisure_time {
            activities.push(ScheduledActivity {
                activity_type: ActivityType::Leisure,
                start_hour: start,
                duration_hours: 1.5,
                location: ActivityLocation::NearestOfType(ServiceType::SmallPark),
                flexibility: 1.0,
                priority: 3,
            });
        }
    }

    // Socializing (driven by social need)
    if needs.social < 40.0 && rng.gen::<f32>() < personality.sociability * 0.5 {
        let social_time = find_free_slot(&activities, 2.0, wake_time, sleep_time, rng);
        if let Some(start) = social_time {
            activities.push(ScheduledActivity {
                activity_type: ActivityType::Socializing,
                start_hour: start,
                duration_hours: 2.0,
                location: ActivityLocation::NearestOfType(ServiceType::Plaza),
                flexibility: 1.0,
                priority: 2,
            });
        }
    }

    // Sort by start time
    activities.sort_by(|a, b| a.start_hour.partial_cmp(&b.start_hour).unwrap());

    DailySchedule {
        activities,
        generated_day: 0, // Set by caller
    }
}
```

### 3.3 Time-Use Survey Calibration Data

Real-world time-use surveys (American Time Use Survey, Eurostat HETUS) provide calibration targets:

#### Average Daily Time Allocation (hours, working-age adults)

| Activity | Employed Weekday | Employed Weekend | Unemployed | Retired |
|----------|-----------------|-----------------|------------|---------|
| Sleep | 7.5-8.5 | 8.5-9.5 | 9.0-10.0 | 8.5-9.5 |
| Work | 7.5-9.0 | 0-2.0 | 0 | 0 |
| Commute | 0.5-1.5 | 0-0.5 | 0 | 0 |
| Eating | 1.0-1.5 | 1.0-1.5 | 1.0-1.5 | 1.5-2.0 |
| Housework | 0.5-1.0 | 1.5-3.0 | 2.0-3.0 | 2.0-3.0 |
| Shopping | 0.3-0.5 | 0.5-1.5 | 0.5-1.0 | 0.5-1.0 |
| Childcare | 0.5-1.5 | 1.0-3.0 | 1.0-2.0 | 0-0.5 |
| Leisure/TV | 2.0-3.0 | 3.0-5.0 | 4.0-6.0 | 4.0-6.0 |
| Socializing | 0.5-1.0 | 1.0-2.0 | 1.0-2.0 | 1.0-2.0 |
| Exercise | 0.2-0.5 | 0.3-1.0 | 0.3-0.5 | 0.5-1.0 |
| Education | 0-0.5 | 0-0.5 | 0-2.0 | 0-0.5 |

#### Work Start Time Distribution

Not everyone starts at 9 AM. Real work start times follow a bimodal distribution:

```rust
fn generate_work_start_hour(occupation: Occupation, rng: &mut impl Rng) -> f32 {
    match occupation {
        // Office workers: peaked at 8-9 AM with variance
        Occupation::WhiteCollar | Occupation::Professional | Occupation::Executive => {
            // Normal distribution, mean=8.5, std=0.75
            let normal = rng.sample::<f32, _>(StandardNormal);
            (8.5 + normal * 0.75).clamp(6.0, 11.0)
        }
        // Blue collar: earlier start, often 6-7 AM
        Occupation::BlueCollar => {
            let normal = rng.sample::<f32, _>(StandardNormal);
            (6.5 + normal * 1.0).clamp(4.0, 10.0)
        }
        // Service/retail: bimodal (morning shift or afternoon shift)
        Occupation::Service => {
            if rng.gen::<f32>() < 0.6 {
                // Morning shift
                (7.0 + rng.gen::<f32>() * 2.0) // 7-9 AM
            } else {
                // Afternoon/evening shift
                (14.0 + rng.gen::<f32>() * 3.0) // 2-5 PM
            }
        }
        _ => 8.0,
    }
}
```

This produces staggered commute peaks that create realistic morning and evening rush hours rather than a single spike at 8 AM and 5 PM.

### 3.4 Markov Chain Schedule Perturbation

Rather than generating a completely new schedule each day (expensive), use a Markov chain to perturb the previous day's schedule. The transition matrix encodes how likely a citizen is to change each activity:

```rust
/// Markov chain for day-to-day schedule stability
/// Most people have very similar schedules Monday-Friday
struct ScheduleMarkov {
    /// Probability of keeping the same activity at the same time tomorrow
    inertia: f32,  // Typically 0.85-0.95 for mandatory, 0.3-0.5 for discretionary
}

fn perturb_schedule(
    previous: &DailySchedule,
    citizen: &CitizenDetails,
    needs: &Needs,
    rng: &mut impl Rng,
) -> DailySchedule {
    let mut new_activities = Vec::with_capacity(previous.activities.len());

    for activity in &previous.activities {
        let keep_probability = match activity.activity_type {
            ActivityType::Work | ActivityType::School => 0.95,   // Very stable
            ActivityType::Sleep => 0.90,                         // Slight drift
            ActivityType::Shopping => 0.40,                      // Variable
            ActivityType::Leisure => 0.30,                       // Spontaneous
            ActivityType::Socializing => 0.25,                   // Very variable
            ActivityType::Healthcare => 0.10,                    // Rare, scheduled
            _ => 0.50,
        };

        if rng.gen::<f32>() < keep_probability {
            // Keep with small time perturbation
            let time_jitter = rng.gen::<f32>() * 0.5 - 0.25; // +/- 15 min
            let mut a = activity.clone();
            a.start_hour = (a.start_hour + time_jitter).clamp(0.0, 23.5);
            new_activities.push(a);
        }
        // Otherwise, activity is dropped and may be replaced by need-driven generation
    }

    // Fill gaps with need-driven activities (similar to layer 2 & 3 above)
    fill_discretionary_gaps(&mut new_activities, citizen, needs, rng);

    DailySchedule {
        activities: new_activities,
        generated_day: previous.generated_day + 1,
    }
}
```

This Markov approach means that on most workdays, a citizen follows essentially the same routine (leave at 7:45, work until 5:15, shop on Tuesday and Thursday, gym on Wednesday). This is both computationally cheap and behaviorally realistic -- humans are creatures of habit.

### 3.5 Weekend and Special Day Handling

```rust
#[derive(Debug, Clone, Copy)]
pub enum DayOfWeek {
    Monday, Tuesday, Wednesday, Thursday, Friday, Saturday, Sunday,
}

impl DayOfWeek {
    pub fn is_workday(&self) -> bool {
        !matches!(self, Self::Saturday | Self::Sunday)
    }

    pub fn from_game_day(day: u32) -> Self {
        match day % 7 {
            0 => Self::Monday,
            1 => Self::Tuesday,
            2 => Self::Wednesday,
            3 => Self::Thursday,
            4 => Self::Friday,
            5 => Self::Saturday,
            _ => Self::Sunday,
        }
    }
}
```

Weekend schedules differ substantially:
- No work or school for most citizens
- Wake time shifts 1-2 hours later
- Shopping probability increases 2x (Saturday is peak retail)
- Leisure and socializing time doubles
- Family activities (childcare, household tasks) fill former work hours
- Religious services on Sunday for religious citizens (weekly worship pattern)
- Recreational travel (parks, stadiums) peaks on weekends

### 3.6 Schedule Execution in the State Machine

The existing `citizen_state_machine` system transitions citizens through states based on the `GameClock`. Integrating generated schedules:

```rust
fn execute_schedule(
    clock: &GameClock,
    schedule: &DailySchedule,
    current_state: &CitizenStateComp,
    needs: &Needs,
) -> Option<CitizenState> {
    let current_hour = clock.hour_of_day();

    // Find the activity that should be active right now
    let current_activity = schedule.activities.iter()
        .find(|a| {
            let end = a.start_hour + a.duration_hours;
            current_hour >= a.start_hour && current_hour < end
        });

    match current_activity {
        Some(activity) => {
            let target_state = match activity.activity_type {
                ActivityType::Sleep => CitizenState::AtHome,
                ActivityType::Work => CitizenState::Working,
                ActivityType::School => CitizenState::AtSchool,
                ActivityType::Shopping => CitizenState::Shopping,
                ActivityType::Leisure | ActivityType::Socializing
                    | ActivityType::Exercise => CitizenState::AtLeisure,
                _ => CitizenState::AtHome,
            };

            // If we're not already in the target state, start commuting
            if current_state.0 != target_state && !current_state.0.is_commuting() {
                Some(commuting_state_for(target_state))
            } else {
                None // Already where we need to be
            }
        }
        None => {
            // Between activities, head home
            if current_state.0 != CitizenState::AtHome && !current_state.0.is_commuting() {
                Some(CitizenState::CommutingHome)
            } else {
                None
            }
        }
    }
}
```

---

## 4. Decision Making Models

### 4.1 Residential Location Choice (Where to Live)

Where citizens choose to live is arguably the most important decision in a city simulation -- it drives demand for housing, determines commute patterns, shapes neighborhood demographics, and feeds back into land values, school quality, and crime rates.

The standard approach in urban economics is the **discrete choice model** (specifically, the multinomial logit model), where each citizen evaluates every available dwelling and selects probabilistically based on a utility function.

#### Utility Function for Housing Choice

```rust
/// Compute the utility of a dwelling for a given household.
/// Higher utility = more attractive location.
fn housing_utility(
    dwelling: &Dwelling,
    household: &Household,
    citizen: &Demographics,
    city_data: &CityData,
) -> f32 {
    // Weight vector (sums to 1.0, varies by household type and income)
    let weights = housing_weights(citizen.income_class, household.household_type);

    // --- Factor 1: Affordability (weight: 0.25 for low-income, 0.10 for high-income) ---
    // rent_burden = monthly_rent / household_income
    // Utility peaks when rent is ~25% of income, drops sharply above 35%
    let rent_burden = dwelling.monthly_rent / household.combined_income.max(1.0);
    let affordability = if rent_burden < 0.20 {
        0.9  // Easily affordable but maybe low quality
    } else if rent_burden < 0.30 {
        1.0  // Sweet spot
    } else if rent_burden < 0.40 {
        0.5  // Strained
    } else if rent_burden < 0.50 {
        0.2  // Severely burdened
    } else {
        0.0  // Unaffordable
    };

    // --- Factor 2: Commute Distance (weight: 0.20) ---
    // Manhattan distance from dwelling to workplace
    // People strongly dislike commutes > 30 min (in grid cells: ~30 cells)
    let commute_dist = if let Some(work) = &household.primary_workplace {
        let dx = (dwelling.grid_x as i32 - work.grid_x as i32).abs();
        let dy = (dwelling.grid_y as i32 - work.grid_y as i32).abs();
        (dx + dy) as f32
    } else {
        0.0  // Unemployed/retired, commute doesn't matter
    };
    let commute_utility = (-0.03 * commute_dist).exp(); // Exponential decay
    // commute 0  -> 1.0
    // commute 10 -> 0.74
    // commute 20 -> 0.55
    // commute 30 -> 0.41
    // commute 50 -> 0.22

    // --- Factor 3: School Quality (weight: 0.15 for families, 0.0 for singles) ---
    let school_quality = if household.has_children() {
        city_data.school_quality_at(dwelling.grid_x, dwelling.grid_y)
    } else {
        0.5 // Neutral
    };

    // --- Factor 4: Safety/Crime (weight: 0.15) ---
    let crime_level = city_data.crime_grid.get(dwelling.grid_x, dwelling.grid_y) as f32;
    let safety = 1.0 - (crime_level / 100.0); // 0=dangerous, 1=safe

    // --- Factor 5: Environmental Quality (weight: 0.10) ---
    let pollution = city_data.pollution_grid.get(dwelling.grid_x, dwelling.grid_y) as f32;
    let noise = city_data.noise_grid.get(dwelling.grid_x, dwelling.grid_y) as f32;
    let parks_nearby = city_data.park_coverage_at(dwelling.grid_x, dwelling.grid_y);
    let environment = (1.0 - pollution / 100.0) * 0.4
                    + (1.0 - noise / 100.0) * 0.3
                    + parks_nearby * 0.3;

    // --- Factor 6: Neighborhood Composition (weight: 0.05-0.15) ---
    // Schelling preference: utility from similar neighbors
    let similar_fraction = city_data.neighborhood_similarity(
        dwelling.grid_x, dwelling.grid_y,
        citizen.ethnicity,
        8, // 8-cell radius
    );
    // Most people prefer ~30-70% similar neighbors (not fully segregated or isolated)
    let composition_utility = if similar_fraction < 0.2 {
        0.3 + similar_fraction * 2.0  // Very isolated from in-group
    } else if similar_fraction < 0.7 {
        0.9 + similar_fraction * 0.1  // Comfortable range
    } else {
        1.0 - (similar_fraction - 0.7) * 0.3  // Too homogeneous (slightly less desirable)
    };

    // --- Factor 7: Services Access (weight: 0.10) ---
    let services = city_data.service_score_at(dwelling.grid_x, dwelling.grid_y);

    // --- Weighted sum ---
    weights.affordability * affordability
        + weights.commute * commute_utility
        + weights.schools * school_quality
        + weights.safety * safety
        + weights.environment * environment
        + weights.composition * composition_utility
        + weights.services * services
}
```

#### Weight Profiles by Household Type

```rust
fn housing_weights(income: IncomeClass, household_type: HouseholdType) -> HousingWeights {
    match (income, household_type) {
        // Low-income single: affordability dominates everything
        (IncomeClass::Poverty | IncomeClass::LowIncome, HouseholdType::SinglePerson) =>
            HousingWeights {
                affordability: 0.40, commute: 0.20, schools: 0.0,
                safety: 0.15, environment: 0.05, composition: 0.10, services: 0.10,
            },
        // Middle-income family: balanced with school emphasis
        (IncomeClass::LowerMiddle | IncomeClass::UpperMiddle, HouseholdType::NuclearFamily) =>
            HousingWeights {
                affordability: 0.20, commute: 0.15, schools: 0.20,
                safety: 0.15, environment: 0.10, composition: 0.10, services: 0.10,
            },
        // High-income couple: environment and services matter most
        (IncomeClass::HighIncome | IncomeClass::Wealthy, HouseholdType::CoupleNoChildren) =>
            HousingWeights {
                affordability: 0.05, commute: 0.15, schools: 0.0,
                safety: 0.15, environment: 0.25, composition: 0.15, services: 0.25,
            },
        // Single parent: safety and affordability critical
        (_, HouseholdType::SingleParent) =>
            HousingWeights {
                affordability: 0.30, commute: 0.15, schools: 0.15,
                safety: 0.20, environment: 0.05, composition: 0.10, services: 0.05,
            },
        // Elderly: services access critical (healthcare)
        (_, HouseholdType::Elderly) =>
            HousingWeights {
                affordability: 0.20, commute: 0.0, schools: 0.0,
                safety: 0.20, environment: 0.15, composition: 0.10, services: 0.35,
            },
        // Default: balanced
        _ => HousingWeights {
            affordability: 0.25, commute: 0.15, schools: 0.10,
            safety: 0.15, environment: 0.10, composition: 0.10, services: 0.15,
        },
    }
}
```

#### Multinomial Logit Selection

Rather than always choosing the highest-utility option (which creates unrealistic clustering), use the multinomial logit model where probability is proportional to exponentiated utility:

```rust
/// Select a dwelling probabilistically based on utility scores.
/// P(dwelling_i) = exp(beta * U_i) / sum(exp(beta * U_j))
/// beta controls "rationality": beta=0 -> random, beta=inf -> always best
fn select_dwelling_logit(
    utilities: &[(Entity, f32)],
    beta: f32,  // Typically 3.0-8.0
    rng: &mut impl Rng,
) -> Option<Entity> {
    if utilities.is_empty() { return None; }

    // Compute logit probabilities (with numerical stability trick)
    let max_u = utilities.iter().map(|&(_, u)| u).fold(f32::NEG_INFINITY, f32::max);
    let exp_sum: f32 = utilities.iter()
        .map(|&(_, u)| (beta * (u - max_u)).exp())
        .sum();

    if exp_sum == 0.0 { return Some(utilities[0].0); }

    let roll = rng.gen::<f32>() * exp_sum;
    let mut cumulative = 0.0;
    for &(entity, u) in utilities {
        cumulative += (beta * (u - max_u)).exp();
        if roll <= cumulative {
            return Some(entity);
        }
    }
    Some(utilities.last().unwrap().0)
}
```

The `beta` parameter represents the agent's "rationality" or information quality. In practice, beta=5.0 means the best option is chosen ~70% of the time in a set of 10 options, while beta=2.0 produces much more diffuse choices. Low-income citizens with less flexibility might have higher beta (they must optimize harder), while wealthy citizens with more options can afford suboptimal choices (lower beta).

### 4.2 Job Choice (Where to Work)

Employment decisions follow a similar utility-maximization framework:

```rust
fn job_utility(
    job: &JobOpening,
    citizen: &Demographics,
    home: &HomeLocation,
    personality: &Personality,
) -> f32 {
    // --- Wage utility (logarithmic -- diminishing returns on income) ---
    // A $10K raise matters more to someone making $30K than someone making $200K
    let wage_util = (job.salary / 1000.0).ln().max(0.0);

    // --- Commute disutility ---
    let commute_cells = manhattan_distance(home, job);
    let commute_disutil = -0.04 * commute_cells as f32;

    // --- Skill match (education vs. job requirements) ---
    // Overqualified: slight penalty (boredom, lower pay than potential)
    // Underqualified: large penalty (won't get hired, or will struggle)
    let skill_gap = citizen.education_level as i32 - job.required_education as i32;
    let skill_match = match skill_gap {
        i if i < -1 => 0.0,    // Severely underqualified, won't be hired
        -1 => 0.3,             // Slightly underqualified
        0 => 1.0,              // Perfect match
        1 => 0.8,              // Slightly overqualified
        _ => 0.5,              // Very overqualified (taking a step down)
    };

    // --- Industry preference (personality-driven) ---
    let industry_preference = match (job.industry, personality.ambition) {
        (Industry::Technology, a) if a > 0.7 => 1.2,   // Ambitious love tech
        (Industry::Government, _) => 0.9,                // Stable but boring
        (Industry::Creative, a) if a < 0.5 => 1.1,     // Less ambitious like creative
        _ => 1.0,
    };

    // --- Wage vs commute tradeoff (the key insight) ---
    // A typical commuter values travel time at 50-70% of their wage rate.
    // So a 1-hour commute "costs" ~$15-25 in perceived disutility.
    // This creates realistic spatial equilibrium: higher-paying downtown jobs
    // attract workers from further away, while local jobs serve nearby residents.

    let value_of_time = citizen.monthly_income / 160.0 * 0.6; // 60% of hourly wage
    let commute_time_hours = commute_cells as f32 * 0.05;     // ~3 min per cell
    let commute_cost = value_of_time * commute_time_hours * 20.0; // 20 workdays/month
    let net_monthly_income = job.salary - commute_cost;
    let net_income_util = (net_monthly_income / 1000.0).ln().max(0.0);

    // Weighted combination
    0.50 * net_income_util
        + 0.20 * skill_match
        + 0.15 * industry_preference
        + 0.15 * job.prestige
}
```

The critical insight is the **wage-commute tradeoff**: workers accept longer commutes only when compensated by higher wages. This naturally produces the monocentric city model (Alonso-Muth-Mills) where land prices and density decrease with distance from employment centers, and it breaks down into the polycentric model when suburban employment centers emerge.

### 4.3 Mode Choice (How to Travel)

Transport mode choice determines whether a citizen walks, drives, takes transit, or bikes. This is essential for generating realistic traffic loads.

```rust
#[derive(Debug, Clone, Copy)]
pub enum TransportMode {
    Walking,
    Bicycle,
    Car,
    Bus,
    Subway,
    Tram,
}

fn choose_transport_mode(
    origin: (usize, usize),
    destination: (usize, usize),
    citizen: &Demographics,
    household: &Household,
    city_data: &CityData,
    rng: &mut impl Rng,
) -> TransportMode {
    let distance = manhattan_distance_f32(origin, destination);

    // Available modes depend on infrastructure and ownership
    let has_car = household.vehicles > 0;
    let has_transit = city_data.has_transit_coverage(origin)
                   && city_data.has_transit_coverage(destination);
    let has_bike_lane = city_data.has_bike_infrastructure(origin, destination);

    // Generalized cost for each mode (time + money + comfort, in "utility units")
    let mut mode_utilities: Vec<(TransportMode, f32)> = Vec::new();

    // Walking: free, slow, distance-limited
    if distance < 15.0 {
        let walk_time = distance * 3.0;  // 3 min per cell
        let walk_util = -0.1 * walk_time; // Small time penalty
        mode_utilities.push((TransportMode::Walking, walk_util));
    }

    // Bicycle: cheap, moderate speed, requires bike lanes for safety bonus
    if distance < 40.0 {
        let bike_time = distance * 1.5;
        let safety_bonus = if has_bike_lane { 0.3 } else { -0.2 };
        let bike_util = -0.08 * bike_time + safety_bonus;
        mode_utilities.push((TransportMode::Bicycle, bike_util));
    }

    // Car: fast but expensive, affected by congestion
    if has_car {
        let congestion = city_data.average_congestion_on_route(origin, destination);
        let car_time = distance * 0.8 * (1.0 + congestion * 2.0); // Congestion doubles time
        let car_cost = distance * 0.5; // Fuel + wear
        let parking_cost = city_data.parking_cost_at(destination);
        let car_util = -0.05 * car_time - 0.01 * (car_cost + parking_cost)
                     + 0.5; // Comfort/convenience bonus
        mode_utilities.push((TransportMode::Car, car_util));
    }

    // Bus: cheap, slow (stops frequently), requires coverage
    if has_transit {
        let bus_time = distance * 2.0;  // Slower than car due to stops
        let wait_time = 5.0;            // Average wait at stop (min)
        let bus_cost = 2.5;             // Fixed fare
        let bus_util = -0.08 * (bus_time + wait_time) - 0.01 * bus_cost;
        mode_utilities.push((TransportMode::Bus, bus_util));

        // Subway (if exists): fast, reliable, requires stations nearby
        if city_data.has_subway_at(origin) && city_data.has_subway_at(destination) {
            let subway_time = distance * 0.6;
            let subway_cost = 3.0;
            let subway_util = -0.05 * subway_time - 0.01 * subway_cost + 0.2;
            mode_utilities.push((TransportMode::Subway, subway_util));
        }
    }

    // Income affects mode choice (wealthy prefer car comfort, poor prefer cheap options)
    for (mode, util) in &mut mode_utilities {
        match (mode, citizen.income_class) {
            (TransportMode::Car, IncomeClass::HighIncome | IncomeClass::Wealthy) => {
                *util += 0.3; // Wealthy value car convenience more
            }
            (TransportMode::Bus, IncomeClass::HighIncome | IncomeClass::Wealthy) => {
                *util -= 0.3; // Wealthy avoid bus stigma
            }
            (TransportMode::Walking | TransportMode::Bicycle,
             IncomeClass::Poverty | IncomeClass::LowIncome) => {
                *util += 0.2; // Low-income prefer free transport
            }
            _ => {}
        }
    }

    // Logit selection with moderate rationality
    select_mode_logit(&mode_utilities, 3.0, rng)
}
```

This mode choice model produces emergent phenomena:
- **Congestion pricing**: When roads congest, car utility drops, pushing citizens to transit (if available) or changing departure times.
- **Transit ridership**: Building subway lines increases subway utility, pulling riders from cars, reducing congestion.
- **Car dependency**: In cities without transit, citizens are forced into cars regardless of income, leading to universal congestion.
- **Gentrification signal**: When low-income citizens in a neighborhood start switching from walking to cars, it signals rising incomes (gentrification in progress).

### 4.4 Consumption and Spending Decisions

Citizens allocate income across spending categories, which feeds back into the economy:

```rust
fn monthly_budget_allocation(
    income: f32,
    household: &Household,
    personality: &Personality,
) -> SpendingBreakdown {
    // Engel's law: food share decreases as income rises
    let food_share = (0.35 - 0.20 * (income / 10000.0).min(1.0)).max(0.10);

    // Housing: constrained by actual rent
    let housing_share = (household.rent / income).min(0.50);

    // Transport: depends on mode
    let transport_share = if household.vehicles > 0 { 0.15 } else { 0.05 };

    // Remaining income allocated by personality
    let remaining = (1.0 - food_share - housing_share - transport_share).max(0.0);

    SpendingBreakdown {
        food: income * food_share,
        housing: household.rent,
        transport: income * transport_share,
        healthcare: income * remaining * 0.10,
        education: income * remaining * (0.05 + personality.ambition * 0.10),
        entertainment: income * remaining * (0.10 + (1.0 - personality.materialism) * 0.10),
        savings: income * remaining * (personality.materialism * 0.30),
        other: income * remaining * 0.15,
    }
}
```

Spending patterns drive commercial district demand (shopping trips), entertainment venue usage (leisure trips), and the overall economic multiplier effect.

---

## 5. Happiness Modeling

### 5.1 Current System Analysis

The existing `update_happiness` system in Megacity computes happiness as a linear sum of bonuses and penalties starting from `BASE_HAPPINESS = 50.0`. Positive factors include employment (+15), short commute (+10), power (+5), water (+5), various service coverages (+3 to +8), land value bonus, policy bonuses, and weather. Negative factors include missing utilities (-20 to -25), high taxes, crime, pollution, noise, garbage, congestion, poor road conditions, homelessness (-10 to -30), and low health/needs.

This works and is performant (already uses `par_iter_mut()` for parallel evaluation), but the model can be deepened with three important psychological phenomena: multi-factor weighting with wealth-dependent profiles, hedonic adaptation, and relative deprivation.

### 5.2 Multi-Factor Weighted Happiness Model

A robust happiness model decomposes satisfaction into weighted domains. The existing `WealthHappinessWeights` struct already adjusts per wealth tier -- this extends the concept to a full framework.

#### Domain Weights

| Domain | Weight | Sub-factors | Notes |
|--------|--------|-------------|-------|
| Housing Quality | 20% | Size adequacy, condition, rent burden, overcrowding | Maslow's hierarchy: shelter is foundational |
| Commute | 15% | Travel time, mode comfort, congestion exposure | Research: every 10 min of commute = same unhappiness as 19% pay cut |
| Safety | 15% | Crime rate at home, crime trend, victimization history | Asymmetric: one crime event destroys years of safety |
| Health | 10% | Personal health, healthcare access, air quality | Health is baseline; only noticed when poor |
| Environment | 10% | Pollution, noise, green space, aesthetic quality | Wealthy citizens weight this 2x (environmental gentrification) |
| Social | 10% | Social connections, community belonging, neighbor similarity | Isolation is as harmful as smoking 15 cigarettes/day |
| Services | 10% | Education, fire, police, utilities, internet, postal | Threshold effect: zero services = misery, full coverage = modest boost |
| Affordability | 10% | Rent burden, disposable income, savings trajectory | Separate from housing quality -- can have nice house but unaffordable |

#### Implementation with Diminishing Returns

Instead of linear bonuses, use logarithmic or sigmoid curves that model diminishing returns -- the first hospital in a neighborhood matters enormously, the fifth one barely registers:

```rust
/// Sigmoid satisfaction function
/// Maps a raw score (0.0-1.0) to a satisfaction contribution (0.0-1.0)
/// with configurable steepness and midpoint
fn sigmoid_satisfaction(raw: f32, midpoint: f32, steepness: f32) -> f32 {
    1.0 / (1.0 + (-steepness * (raw - midpoint)).exp())
}

/// Compute domain satisfaction for housing quality
fn housing_quality_satisfaction(
    dwelling: &Dwelling,
    household: &Household,
    citizen: &Demographics,
) -> f32 {
    // Sub-factor 1: Space adequacy (people per room)
    let people_per_room = household.size as f32 / dwelling.rooms.max(1) as f32;
    let space_score = match people_per_room {
        x if x <= 0.5 => 1.0,   // Spacious
        x if x <= 1.0 => 0.9,   // Comfortable
        x if x <= 1.5 => 0.6,   // Adequate
        x if x <= 2.0 => 0.3,   // Crowded
        _ => 0.1,                // Overcrowded
    };

    // Sub-factor 2: Building condition
    let condition_score = dwelling.condition as f32 / 100.0;

    // Sub-factor 3: Utilities (binary -- massive impact when missing)
    let utilities_score = if dwelling.has_power && dwelling.has_water && dwelling.has_sewage {
        1.0
    } else if dwelling.has_power && dwelling.has_water {
        0.6
    } else if dwelling.has_power {
        0.3
    } else {
        0.05
    };

    // Sub-factor 4: Heating (seasonal)
    let heating_score = if dwelling.needs_heating && !dwelling.has_heating {
        0.3 // Cold and miserable
    } else {
        1.0
    };

    // Weighted combination with diminishing returns
    let raw = space_score * 0.30
            + condition_score * 0.25
            + utilities_score * 0.30
            + heating_score * 0.15;

    sigmoid_satisfaction(raw, 0.5, 6.0) // Sigmoid with midpoint at 0.5, moderate steepness
}
```

#### Wealth-Tier Weight Modulation

The existing `WealthHappinessWeights` system adjusts per tier. Here is the full matrix:

```rust
fn happiness_weight_profile(income: IncomeClass) -> DomainWeights {
    match income {
        IncomeClass::Poverty => DomainWeights {
            housing: 0.25,     // Critical -- shelter insecurity
            commute: 0.10,     // Less relevant (walk/bus anyway)
            safety: 0.20,      // High-crime neighborhoods
            health: 0.10,
            environment: 0.05, // Luxury concern
            social: 0.10,
            services: 0.10,
            affordability: 0.10,
        },
        IncomeClass::LowIncome => DomainWeights {
            housing: 0.22, commute: 0.12, safety: 0.18,
            health: 0.10, environment: 0.05, social: 0.10,
            services: 0.10, affordability: 0.13,
        },
        IncomeClass::LowerMiddle => DomainWeights {
            housing: 0.20, commute: 0.15, safety: 0.15,
            health: 0.10, environment: 0.08, social: 0.10,
            services: 0.10, affordability: 0.12,
        },
        IncomeClass::UpperMiddle => DomainWeights {
            housing: 0.18, commute: 0.15, safety: 0.12,
            health: 0.10, environment: 0.12, social: 0.10,
            services: 0.12, affordability: 0.11,
        },
        IncomeClass::HighIncome => DomainWeights {
            housing: 0.15, commute: 0.15, safety: 0.10,
            health: 0.10, environment: 0.18, social: 0.10,
            services: 0.15, affordability: 0.07,
        },
        IncomeClass::Wealthy => DomainWeights {
            housing: 0.12, commute: 0.12, safety: 0.08,
            health: 0.10, environment: 0.20, social: 0.12,
            services: 0.18, affordability: 0.08,
        },
    }
}
```

Key pattern: As income rises, affordability and safety concerns decrease while environment, services, and social concerns increase. This matches Maslow's hierarchy of needs -- once basic survival needs are met, higher-order needs dominate.

### 5.3 Hedonic Adaptation (The Hedonic Treadmill)

One of the most robust findings in happiness research is hedonic adaptation: people quickly return to a baseline happiness level after both positive and negative events. A lottery winner is ecstatic for weeks but returns to baseline within months. A person who loses their job is devastated initially but adapts within a year.

In a city simulation, this means that building a new park produces a happiness spike that fades, requiring the player to continuously improve to maintain satisfaction.

```rust
#[derive(Component, Debug, Clone)]
pub struct HappinessAdaptation {
    /// The citizen's personal baseline (set point), determined by personality.
    /// Most people: 60-70. Resilient individuals: 70-80. Neurotic: 40-55.
    pub set_point: f32,

    /// How quickly the citizen adapts to changes (0.0 = never, 1.0 = instantly).
    /// Typical value: 0.03-0.10 per game-day (~30-100 day half-life).
    pub adaptation_rate: f32,

    /// Running average that the current happiness is pulled toward.
    pub adapted_level: f32,

    /// Recent shock events that resist adaptation for a fixed duration.
    pub shocks: Vec<HappinessShock>,
}

#[derive(Debug, Clone)]
pub struct HappinessShock {
    pub magnitude: f32,         // Positive or negative
    pub remaining_ticks: u32,   // How long before adaptation begins
    pub category: ShockCategory,
}

#[derive(Debug, Clone, Copy)]
pub enum ShockCategory {
    /// Adapts quickly (weeks): New park, new shop, road improvement
    EnvironmentalImprovement,
    /// Adapts moderately (months): Pay raise, new home
    SocioeconomicGain,
    /// Adapts slowly (months-year): Job loss, home foreclosure
    SocioeconomicLoss,
    /// Barely adapts: Death of family member, violent crime victimization
    TraumaticEvent,
    /// Never adapts: Chronic pain, permanent disability
    ChronicCondition,
}

impl ShockCategory {
    pub fn adaptation_half_life_ticks(&self) -> u32 {
        match self {
            Self::EnvironmentalImprovement => 200,   // ~20 game-days
            Self::SocioeconomicGain => 600,          // ~60 game-days
            Self::SocioeconomicLoss => 1000,         // ~100 game-days
            Self::TraumaticEvent => 3000,            // ~300 game-days (nearly permanent)
            Self::ChronicCondition => u32::MAX,      // Permanent
        }
    }
}

fn apply_hedonic_adaptation(
    adaptation: &mut HappinessAdaptation,
    current_happiness: &mut f32,
    delta_ticks: u32,
) {
    // Decay shocks
    adaptation.shocks.retain_mut(|shock| {
        if shock.remaining_ticks > delta_ticks {
            shock.remaining_ticks -= delta_ticks;
            true
        } else {
            false
        }
    });

    // Compute total active shock effect
    let shock_effect: f32 = adaptation.shocks.iter()
        .map(|s| s.magnitude * (s.remaining_ticks as f32
            / s.category.adaptation_half_life_ticks() as f32).min(1.0))
        .sum();

    // Pull current happiness toward set_point + shock_effect
    let target = adaptation.set_point + shock_effect;
    let blend = adaptation.adaptation_rate * delta_ticks as f32;
    *current_happiness = *current_happiness * (1.0 - blend) + target * blend;
    *current_happiness = current_happiness.clamp(0.0, 100.0);
}
```

**Game design implications:** Without hedonic adaptation, players can build one good neighborhood and ignore it forever. With adaptation, citizens gradually expect more -- the player must continuously invest in improvements to maintain satisfaction. This creates the "treadmill" effect that drives ongoing gameplay engagement.

However, pure treadmill is frustrating. The key is that the *set point itself* can shift slowly upward (by 0.1-0.5 per game-year) when a citizen experiences sustained positive conditions, modeling genuine life satisfaction improvements. A city that goes from terrible to good genuinely makes its citizens happier in the long run -- but the marginal gain from each incremental improvement shrinks.

### 5.4 Relative Deprivation and Social Comparison

People don't evaluate their circumstances in absolute terms -- they compare to their reference group. A person earning $50K is happy in a neighborhood where the median is $40K but miserable where the median is $100K. This is the **relative deprivation** effect (Runciman 1966, Yitzhaki 1979).

```rust
/// Compute relative deprivation for a citizen.
/// Returns a value from -1.0 (much better off than neighbors) to +1.0 (much worse off).
/// Positive values reduce happiness; negative values boost it slightly.
fn relative_deprivation(
    citizen_income: f32,
    neighborhood_incomes: &[f32],
) -> f32 {
    if neighborhood_incomes.is_empty() { return 0.0; }

    let n = neighborhood_incomes.len() as f32;
    let citizen_income = citizen_income.max(1.0);

    // Yitzhaki's index: sum of income differences with richer neighbors
    // divided by (n * mean_income)
    let richer_sum: f32 = neighborhood_incomes.iter()
        .filter(|&&inc| inc > citizen_income)
        .map(|&inc| inc - citizen_income)
        .sum();
    let mean_income = neighborhood_incomes.iter().sum::<f32>() / n;

    if mean_income == 0.0 { return 0.0; }

    let deprivation_index = richer_sum / (n * mean_income);

    // Also compute "relative advantage" (being richer than neighbors)
    // This provides a smaller boost (people are loss-averse)
    let poorer_sum: f32 = neighborhood_incomes.iter()
        .filter(|&&inc| inc < citizen_income)
        .map(|&inc| citizen_income - inc)
        .sum();
    let advantage_index = poorer_sum / (n * mean_income);

    // Net effect: deprivation hurts more than advantage helps (prospect theory)
    // Loss aversion ratio ~2.0 (Kahneman & Tversky)
    let net = 2.0 * deprivation_index - 0.5 * advantage_index;
    net.clamp(-1.0, 1.0)
}
```

**Gameplay effect:** This creates a pressure toward either homogeneous neighborhoods (Schelling segregation) or mixed-income policies. If a player builds luxury condos next to low-income housing, the low-income residents become *less* happy despite their conditions not changing -- because they now see what they don't have. This is realistic and creates interesting policy dilemmas: mixed-income neighborhoods are socially better but harder to maintain.

### 5.5 Asymmetric Loss Aversion

People feel losses roughly twice as intensely as equivalent gains (Kahneman & Tversky, prospect theory). In the happiness model:

```rust
fn happiness_change_with_loss_aversion(
    old_happiness: f32,
    new_raw_happiness: f32,
    resilience: f32,  // From Personality, 0.0-1.0
) -> f32 {
    let delta = new_raw_happiness - old_happiness;

    if delta >= 0.0 {
        // Gains: apply normally (possibly with diminishing returns)
        old_happiness + delta * 0.8  // 80% of raw gain
    } else {
        // Losses: amplified, modulated by resilience
        let loss_multiplier = 2.0 - resilience * 0.8; // 1.2x (resilient) to 2.0x (fragile)
        old_happiness + delta * loss_multiplier
    }
}
```

This means demolishing a park in a neighborhood hits happiness harder than building one helps. Players learn that service disruptions are very costly and plan accordingly.

### 5.6 Mood Thresholds and Behavioral Triggers

Similar to RimWorld's mental break thresholds, certain happiness levels trigger behavioral changes:

```rust
#[derive(Debug, Clone, Copy)]
pub enum MoodState {
    Ecstatic,      // 90-100: Bonus productivity, helps neighbors, won't emigrate
    Happy,         // 70-89:  Normal positive behavior
    Content,       // 50-69:  Neutral
    Unhappy,       // 30-49:  Reduced productivity, starts job searching
    Miserable,     // 15-29:  Emigration candidate, crime increase, protest
    Crisis,        // 0-14:   Active harm: vandalism, riots, immediate emigration
}

impl MoodState {
    pub fn from_happiness(h: f32) -> Self {
        match h as u32 {
            90..=100 => Self::Ecstatic,
            70..=89 => Self::Happy,
            50..=69 => Self::Content,
            30..=49 => Self::Unhappy,
            15..=29 => Self::Miserable,
            _ => Self::Crisis,
        }
    }

    pub fn productivity_modifier(&self) -> f32 {
        match self {
            Self::Ecstatic => 1.15,
            Self::Happy => 1.05,
            Self::Content => 1.0,
            Self::Unhappy => 0.85,
            Self::Miserable => 0.60,
            Self::Crisis => 0.30,
        }
    }

    pub fn crime_propensity_modifier(&self) -> f32 {
        match self {
            Self::Ecstatic => 0.5,
            Self::Happy => 0.8,
            Self::Content => 1.0,
            Self::Unhappy => 1.5,
            Self::Miserable => 2.5,
            Self::Crisis => 5.0,
        }
    }

    pub fn emigration_probability_per_month(&self) -> f32 {
        match self {
            Self::Ecstatic | Self::Happy | Self::Content => 0.0,
            Self::Unhappy => 0.02,
            Self::Miserable => 0.10,
            Self::Crisis => 0.40,
        }
    }
}
```

---

## 6. Social Dynamics: Segregation

### 6.1 The Schelling Model

Thomas Schelling's segregation model (1971) demonstrates that even mild individual preferences for similar neighbors produce stark macro-level segregation. The key insight: if each agent requires only 30-40% similar neighbors to stay put, the result is near-total spatial separation of groups. This emergent property makes it a compelling game mechanic.

### 6.2 Grid-Based Implementation for Megacity

Since Megacity already operates on a 256x256 grid with buildings and occupants, the Schelling model maps directly to the existing data structure. Each residential building's occupants have an ethnic/cultural group, and the neighborhood composition is computed from surrounding cells.

```rust
/// Configuration for the Schelling segregation dynamics
#[derive(Resource)]
pub struct SegregationConfig {
    /// Minimum fraction of similar neighbors for comfort (0.0-1.0).
    /// Schelling's finding: even 0.33 produces strong segregation.
    pub tolerance_threshold: f32,

    /// Radius (in grid cells) to consider as "neighborhood".
    pub neighborhood_radius: i32,

    /// How often to re-evaluate (in ticks). Residential relocation is slow.
    pub evaluation_interval: u64,

    /// Maximum relocations per tick (performance cap).
    pub max_relocations_per_tick: u32,
}

impl Default for SegregationConfig {
    fn default() -> Self {
        Self {
            tolerance_threshold: 0.35,
            neighborhood_radius: 5,
            evaluation_interval: 200,
            max_relocations_per_tick: 50,
        }
    }
}

/// Per-cell neighborhood composition cache (avoids recomputation per citizen)
#[derive(Resource)]
pub struct NeighborhoodComposition {
    /// For each cell, the fraction of each ethnic group in the neighborhood.
    /// Stored as a flat array: [cell_0_groupA, cell_0_groupB, ..., cell_1_groupA, ...]
    /// With 5 groups and 256*256 cells = 327,680 f32 values (~1.25 MB)
    pub fractions: Vec<f32>,
    pub num_groups: usize,
    pub width: usize,
    pub height: usize,
}

impl NeighborhoodComposition {
    pub fn new(width: usize, height: usize, num_groups: usize) -> Self {
        Self {
            fractions: vec![0.0; width * height * num_groups],
            num_groups,
            width,
            height,
        }
    }

    pub fn get_fraction(&self, x: usize, y: usize, group: usize) -> f32 {
        self.fractions[(y * self.width + x) * self.num_groups + group]
    }

    pub fn set_fraction(&mut self, x: usize, y: usize, group: usize, value: f32) {
        self.fractions[(y * self.width + x) * self.num_groups + group] = value;
    }
}

/// System: Compute neighborhood composition grid
/// Runs infrequently because residential composition changes slowly
fn update_neighborhood_composition(
    config: Res<SegregationConfig>,
    grid: Res<WorldGrid>,
    buildings: Query<(&Building, &BuildingOccupantGroups)>,
    mut composition: ResMut<NeighborhoodComposition>,
) {
    let radius = config.neighborhood_radius;
    let w = composition.width;
    let h = composition.height;
    let ng = composition.num_groups;

    // Reset
    composition.fractions.fill(0.0);

    // Build per-cell group counts
    // First pass: accumulate raw counts per cell from building occupants
    let mut cell_counts = vec![0u16; w * h * ng];
    for (building, groups) in &buildings {
        let idx = building.grid_y * w + building.grid_x;
        for g in 0..ng {
            cell_counts[idx * ng + g] += groups.counts[g];
        }
    }

    // Second pass: for each cell, sum neighborhood counts within radius
    // Using a sliding window for O(w*h*radius) instead of O(w*h*radius^2)
    for y in 0..h {
        for x in 0..w {
            let mut total = 0u32;
            let mut group_totals = [0u32; 6]; // Max 6 groups

            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                        continue;
                    }
                    let nidx = ny as usize * w + nx as usize;
                    for g in 0..ng {
                        let count = cell_counts[nidx * ng + g] as u32;
                        group_totals[g] += count;
                        total += count;
                    }
                }
            }

            if total > 0 {
                let cell_base = (y * w + x) * ng;
                for g in 0..ng {
                    composition.fractions[cell_base + g] =
                        group_totals[g] as f32 / total as f32;
                }
            }
        }
    }
}
```

### 6.3 Relocation Decision

When a citizen evaluates whether to move, they compare their current neighborhood's similar-neighbor fraction against their tolerance threshold:

```rust
/// System: evaluate neighborhood satisfaction and trigger relocations
fn evaluate_neighborhood_satisfaction(
    config: Res<SegregationConfig>,
    tick: Res<TickCounter>,
    composition: Res<NeighborhoodComposition>,
    mut relocation_events: EventWriter<RelocationRequest>,
    citizens: Query<(Entity, &HomeLocation, &Demographics, &CitizenDetails), With<Citizen>>,
) {
    if tick.0 % config.evaluation_interval != 0 { return; }

    let mut relocations = 0u32;

    for (entity, home, demographics, details) in &citizens {
        if relocations >= config.max_relocations_per_tick { break; }

        let group_idx = demographics.ethnicity as usize;
        let similar_fraction = composition.get_fraction(
            home.grid_x, home.grid_y, group_idx
        );

        // Tolerance varies by personality and life stage
        let personal_tolerance = config.tolerance_threshold
            * (0.8 + demographics.personality_openness * 0.4);
            // Open personalities tolerate more diversity

        if similar_fraction < personal_tolerance {
            // Discomfort level scales with how far below threshold
            let discomfort = (personal_tolerance - similar_fraction) / personal_tolerance;

            // Probability of seeking to relocate depends on discomfort + other factors
            // Even dissatisfied people don't immediately move (moving costs, inertia)
            let move_probability = discomfort * 0.3
                * (1.0 + (details.savings / 5000.0).min(1.0)); // Need savings to move

            // Use tick-based pseudo-random (deterministic, no thread-local RNG)
            let hash = tick_pseudo_random(tick.0 ^ entity.index() as u64);
            if (hash % 1000) < (move_probability * 1000.0) as u32 {
                relocation_events.send(RelocationRequest {
                    citizen: entity,
                    reason: RelocationReason::NeighborhoodComposition,
                    preferred_similar_fraction: personal_tolerance,
                    budget: details.savings * 0.5,
                });
                relocations += 1;
            }
        }
    }
}
```

### 6.4 Segregation Metrics

Track segregation for the player as a city-wide statistic:

```rust
/// The Dissimilarity Index: most common segregation metric.
/// D = 0.0 (perfectly integrated) to 1.0 (perfectly segregated)
/// D > 0.6 is considered "hyper-segregated" (US cities like Chicago: D~0.75)
fn compute_dissimilarity_index(
    composition: &NeighborhoodComposition,
    group_a: usize,
    group_b: usize,
) -> f32 {
    let w = composition.width;
    let h = composition.height;
    let ng = composition.num_groups;

    let mut total_a = 0.0f32;
    let mut total_b = 0.0f32;
    let mut sum = 0.0f32;

    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) * ng;
            let a = composition.fractions[idx + group_a];
            let b = composition.fractions[idx + group_b];
            total_a += a;
            total_b += b;
        }
    }

    if total_a == 0.0 || total_b == 0.0 { return 0.0; }

    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) * ng;
            let a_frac = composition.fractions[idx + group_a] / total_a;
            let b_frac = composition.fractions[idx + group_b] / total_b;
            sum += (a_frac - b_frac).abs();
        }
    }

    sum * 0.5
}
```

### 6.5 Policy Interventions Against Segregation

The player can attempt to counteract segregation through:

1. **Inclusionary zoning**: Require new developments to include affordable units. This increases housing utility for low-income groups in high-value areas but may reduce developer profitability (fewer high-end buildings spawn).

2. **Public housing placement**: Strategically placing public housing in affluent neighborhoods breaks up homogeneous zones but triggers NIMBY resistance (see Section 13).

3. **Transit investment**: Better transit between segregated areas reduces the "friction of distance" and allows more mixed commute patterns.

4. **School integration**: Busing programs (controversial, high political cost) can force school integration, improving education equity but reducing parent satisfaction with long commutes.

```rust
struct SegregationPolicy {
    inclusionary_zoning: bool,       // Forced mixed-income in new developments
    inclusionary_percentage: f32,    // Required affordable units (10-30%)
    public_housing_integration: bool, // Place public housing in affluent areas
    school_busing: bool,             // Cross-district school enrollment
    affirmative_housing_vouchers: bool, // Subsidize moves to integrated areas
}
```

Each policy has a *political cost* (reduces approval among affected groups) and a *time lag* (effects take 5-20 game-years to manifest). This creates interesting strategic dilemmas for the player.

---

## 7. Crime Simulation

### 7.1 Current System Analysis

The existing `CrimeGrid` in Megacity computes crime as an inverse function of land value, reduced by police coverage within a radius. This is a reasonable starting point but lacks crime *types*, *agent-level crime decisions*, and *feedback loops* that make crime simulation interesting.

### 7.2 Crime Type Taxonomy

Real crime analysis distinguishes categories with very different causes, spatial patterns, and responses:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrimeType {
    // Property crimes (60-70% of total crime)
    Theft,              // Opportunity-driven, peaks in commercial areas
    Burglary,           // Residential, peaks in low-density affluent areas
    Vandalism,          // Youth-driven, peaks in neglected areas
    AutoTheft,          // Peaks in parking areas with poor lighting

    // Violent crimes (15-20%)
    Assault,            // Alcohol-correlated, peaks near entertainment
    Robbery,            // Intersection of poverty and opportunity
    Homicide,           // Rare, concentrated in specific neighborhoods

    // White-collar crimes (5-10%)
    Fraud,              // Correlates with commercial activity
    Embezzlement,       // Rare, correlates with office density

    // Drug crimes (10-15%)
    DrugPossession,     // Correlates with poverty and youth
    DrugTrafficking,    // Concentrated in specific transit corridors

    // Quality-of-life (not counted in statistics but affects perception)
    Graffiti,           // Youth, neglect
    PublicIntoxication, // Entertainment areas
    Loitering,          // Commercial areas, perceived disorder
}

impl CrimeType {
    /// Severity weight for overall crime index (FBI UCR weights)
    pub fn severity_weight(&self) -> f32 {
        match self {
            Self::Homicide => 10.0,
            Self::Robbery => 4.0,
            Self::Assault => 3.0,
            Self::Burglary => 2.0,
            Self::AutoTheft => 1.5,
            Self::Theft => 1.0,
            Self::DrugTrafficking => 2.0,
            Self::DrugPossession => 0.5,
            Self::Fraud | Self::Embezzlement => 1.5,
            Self::Vandalism => 0.5,
            Self::Graffiti | Self::PublicIntoxication | Self::Loitering => 0.1,
        }
    }
}
```

### 7.3 Crime Causation Model

Crime is driven by the interaction of **motivation** (why someone commits crime), **opportunity** (where/when crime can occur), and **guardianship** (what prevents crime). This is the Routine Activity Theory (Cohen & Felson, 1979).

```rust
/// Compute crime probability at a grid cell
fn cell_crime_probability(
    x: usize,
    y: usize,
    city_data: &CityData,
    crime_type: CrimeType,
) -> f32 {
    // === MOTIVATION ===
    // Driven by unemployment, poverty, inequality, youth population
    let unemployment = city_data.unemployment_rate_at(x, y);
    let poverty_rate = city_data.poverty_rate_at(x, y);
    let youth_fraction = city_data.youth_fraction_at(x, y); // ages 15-24
    let inequality = city_data.local_gini_at(x, y);

    let motivation = match crime_type {
        CrimeType::Theft | CrimeType::Burglary | CrimeType::Robbery => {
            // Economic crimes: poverty and unemployment drive these
            unemployment * 0.3 + poverty_rate * 0.4 + inequality * 0.3
        }
        CrimeType::Vandalism | CrimeType::Graffiti => {
            // Youth disorder: bored, unemployed youth
            youth_fraction * 0.5 + unemployment * 0.3 + 0.2 * (1.0 - city_data.fun_coverage_at(x, y))
        }
        CrimeType::Assault => {
            // Violence: inequality, alcohol, crowding
            inequality * 0.3 + city_data.alcohol_density_at(x, y) * 0.4 + 0.3 * unemployment
        }
        CrimeType::DrugPossession | CrimeType::DrugTrafficking => {
            poverty_rate * 0.4 + unemployment * 0.3 + youth_fraction * 0.3
        }
        _ => 0.1,
    };

    // === OPPORTUNITY ===
    // Available targets and suitable conditions
    let opportunity = match crime_type {
        CrimeType::Theft => {
            // Commercial areas with foot traffic
            city_data.commercial_density_at(x, y) * 0.6 + city_data.foot_traffic_at(x, y) * 0.4
        }
        CrimeType::Burglary => {
            // Residential with high land value, low density (fewer witnesses)
            let value = city_data.land_value_at(x, y) / 100.0;
            let low_density = 1.0 - city_data.population_density_at(x, y);
            value * 0.5 + low_density * 0.5
        }
        CrimeType::AutoTheft => {
            city_data.parking_density_at(x, y) * 0.7
                + (1.0 - city_data.lighting_at(x, y)) * 0.3
        }
        CrimeType::Assault | CrimeType::Robbery => {
            // Dark areas near population centers
            let darkness = 1.0 - city_data.lighting_at(x, y);
            let foot_traffic = city_data.foot_traffic_at(x, y);
            darkness * 0.4 + foot_traffic * 0.3 + city_data.alcohol_density_at(x, y) * 0.3
        }
        _ => 0.3,
    };

    // === GUARDIANSHIP (Absence of) ===
    // Police presence, community surveillance, environmental design
    let police_presence = city_data.police_coverage_at(x, y);
    let community_cohesion = city_data.social_cohesion_at(x, y); // Engaged neighbors
    let environmental_design = city_data.cpted_score_at(x, y);   // CPTED principles
    let lighting = city_data.lighting_at(x, y);

    let guardianship = police_presence * 0.40
                     + community_cohesion * 0.25
                     + environmental_design * 0.20
                     + lighting * 0.15;

    // === Final crime probability ===
    // Crime = Motivation * Opportunity * (1 - Guardianship)
    let base_rate = motivation * opportunity * (1.0 - guardianship);

    // Scale to per-tick probability and clamp
    (base_rate * 0.01).clamp(0.0, 0.05) // Max 5% per tick per cell
}
```

### 7.4 Broken Windows Theory

The broken windows theory (Wilson & Kelling, 1982) argues that visible signs of disorder (graffiti, litter, broken windows) signal that nobody cares about the area, encouraging escalation to more serious crime. Regardless of academic debate about its validity, it makes excellent game mechanics.

```rust
/// Disorder index for a grid cell (0.0 = pristine, 1.0 = total disorder)
fn compute_disorder(
    x: usize, y: usize,
    graffiti_grid: &DisorderGrid,
    garbage_grid: &GarbageGrid,
    road_condition: &RoadConditionGrid,
    building_condition: &BuildingConditionGrid,
    abandoned_buildings: &AbandonmentGrid,
) -> f32 {
    let graffiti = graffiti_grid.get(x, y) as f32 / 255.0;
    let garbage = (garbage_grid.get(x, y) as f32 / 50.0).min(1.0);
    let poor_roads = 1.0 - (road_condition.get(x, y) as f32 / 100.0);
    let poor_buildings = 1.0 - (building_condition.get(x, y) as f32 / 100.0);
    let abandoned = abandoned_buildings.get(x, y) as f32 / 5.0; // 5+ = full disorder

    graffiti * 0.20
        + garbage * 0.25
        + poor_roads * 0.15
        + poor_buildings * 0.20
        + abandoned * 0.20
}

/// Disorder feeds back into crime motivation
/// Each 0.1 increase in disorder raises crime probability by ~5%
fn disorder_crime_multiplier(disorder: f32) -> f32 {
    1.0 + disorder * 0.5 // 1.0x at zero disorder, 1.5x at full disorder
}
```

**Gameplay loop:** Neglecting an area -> garbage accumulates, road conditions degrade -> disorder rises -> crime increases -> citizens flee -> more abandonment -> more disorder. The player must break this cycle with active investment (cleanup, repair, policing) or watch neighborhoods collapse.

### 7.5 Policing Models

Different policing strategies have different effectiveness profiles:

```rust
#[derive(Debug, Clone, Copy)]
pub enum PolicingStrategy {
    /// Standard patrol: uniform coverage, moderate effectiveness
    StandardPatrol,
    /// Hot-spot policing: concentrate in high-crime areas
    /// Research shows 20-30% crime reduction in targeted areas
    /// but may displace crime to adjacent areas
    HotSpotPolicing,
    /// Community policing: build relationships, increase social cohesion
    /// Slow to take effect but reduces motivation long-term
    CommunityPolicing,
    /// Zero tolerance: aggressive enforcement of all infractions
    /// Quick results but damages community trust, expensive
    ZeroTolerance,
    /// Problem-oriented: address root causes
    /// Most effective long-term but requires investment
    ProblemOriented,
}

impl PolicingStrategy {
    /// Immediate crime reduction effectiveness
    pub fn immediate_effectiveness(&self) -> f32 {
        match self {
            Self::StandardPatrol => 0.15,
            Self::HotSpotPolicing => 0.30,
            Self::CommunityPolicing => 0.05,
            Self::ZeroTolerance => 0.35,
            Self::ProblemOriented => 0.10,
        }
    }

    /// Long-term crime reduction per game-year of sustained application
    pub fn annual_structural_reduction(&self) -> f32 {
        match self {
            Self::StandardPatrol => 0.02,
            Self::HotSpotPolicing => 0.03,
            Self::CommunityPolicing => 0.08,    // Best long-term
            Self::ZeroTolerance => -0.01,       // Actually increases crime long-term
            Self::ProblemOriented => 0.10,      // Best overall but expensive
        }
    }

    /// Impact on community trust (affects social cohesion)
    pub fn trust_impact(&self) -> f32 {
        match self {
            Self::StandardPatrol => 0.0,
            Self::HotSpotPolicing => -0.05,     // Over-policing concerns
            Self::CommunityPolicing => 0.10,    // Builds trust
            Self::ZeroTolerance => -0.15,       // Destroys trust
            Self::ProblemOriented => 0.05,
        }
    }

    /// Monthly cost multiplier relative to standard patrol
    pub fn cost_multiplier(&self) -> f32 {
        match self {
            Self::StandardPatrol => 1.0,
            Self::HotSpotPolicing => 1.1,
            Self::CommunityPolicing => 1.2,
            Self::ZeroTolerance => 1.5,
            Self::ProblemOriented => 1.3,
        }
    }
}
```

### 7.6 Crime Displacement and Diffusion

When policing increases in one area, crime does not simply disappear -- it partially displaces to adjacent areas. Research suggests approximately 25-50% displacement with a "diffusion of benefits" effect where adjacent areas also see some reduction.

```rust
fn apply_crime_displacement(
    crime_grid: &mut CrimeGrid,
    policed_x: usize,
    policed_y: usize,
    reduction: f32,
    displacement_rate: f32,  // 0.25-0.50
    diffusion_rate: f32,     // 0.10-0.20 (benefit spreading)
) {
    let width = crime_grid.width;
    let height = crime_grid.height;

    // Reduce crime in policed area
    let original = crime_grid.get(policed_x, policed_y) as f32;
    let reduced = original * (1.0 - reduction);
    crime_grid.set(policed_x, policed_y, reduced as u8);

    let displaced_amount = (original - reduced) * displacement_rate;
    let benefit_amount = (original - reduced) * diffusion_rate;

    // Distribute displaced crime to adjacent cells (weighted by inverse distance)
    let displacement_radius = 5;
    for dy in -displacement_radius..=displacement_radius {
        for dx in -displacement_radius..=displacement_radius {
            if dx == 0 && dy == 0 { continue; }
            let nx = policed_x as i32 + dx;
            let ny = policed_y as i32 + dy;
            if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 { continue; }

            let dist = (dx.abs() + dy.abs()) as f32;
            let weight = 1.0 / (dist * dist);

            let cell_val = crime_grid.get(nx as usize, ny as usize) as f32;

            if dist <= 3.0 {
                // Near the policed area: receive displaced crime AND benefit
                let net = displaced_amount * weight * 0.1 - benefit_amount * weight * 0.15;
                crime_grid.set(nx as usize, ny as usize,
                    (cell_val + net).clamp(0.0, 255.0) as u8);
            } else {
                // Further out: only receive displacement
                let added = displaced_amount * weight * 0.05;
                crime_grid.set(nx as usize, ny as usize,
                    (cell_val + added).clamp(0.0, 255.0) as u8);
            }
        }
    }
}
```

### 7.7 Incarceration and Recidivism

Prison capacity affects crime dynamics. Without sufficient prison capacity, arrested criminals return quickly, but over-incarceration is expensive and counterproductive:

```rust
struct IncarcerationModel {
    prison_capacity: u32,
    current_inmates: u32,
    annual_cost_per_inmate: f32,  // $30K-$50K per year

    /// Recidivism: probability of re-offending within 3 years
    /// US average: ~67%. Can be reduced with rehabilitation programs.
    base_recidivism_rate: f32,  // 0.67

    /// Rehabilitation program effectiveness (reduces recidivism)
    rehabilitation_investment: f32, // 0.0-1.0
}

impl IncarcerationModel {
    fn effective_recidivism(&self) -> f32 {
        // Rehabilitation can cut recidivism from 67% to ~30%
        self.base_recidivism_rate * (1.0 - self.rehabilitation_investment * 0.55)
    }

    fn crime_reduction_from_incarceration(&self) -> f32 {
        // Incapacitation effect: each inmate prevented ~15 crimes/year
        // But diminishing returns: mass incarceration removes
        // low-level offenders who would desist anyway
        let incarceration_rate = self.current_inmates as f32 / 100000.0; // Per 100K pop
        if incarceration_rate < 200.0 {
            incarceration_rate * 0.001  // Linear below 200/100K
        } else {
            // Diminishing returns above 200/100K (US is at ~650/100K)
            0.2 + (incarceration_rate - 200.0) * 0.0002
        }
    }
}
```

---

## 8. Health and Disease

### 8.1 Individual Health Model

The existing `CitizenDetails` tracks health as a single f32 (0-100). A deeper model decomposes health into components that respond to different city conditions:

```rust
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct HealthProfile {
    // Physical health (0-100)
    pub physical: f32,
    // Mental health (0-100)
    pub mental: f32,
    // Chronic conditions (persistent, permanent or slowly improving)
    pub chronic_conditions: Vec<ChronicCondition>,
    // Acute conditions (temporary, from disease or injury)
    pub acute_condition: Option<AcuteCondition>,
    // Composite score for backward compatibility
    pub overall: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ChronicCondition {
    Asthma,           // Worsened by air pollution
    HeartDisease,     // Worsened by stress, pollution, sedentary lifestyle
    Diabetes,         // Correlated with poverty (food deserts)
    Depression,       // Correlated with isolation, unemployment, noise
    Obesity,          // Correlated with car dependency, lack of parks
    RespiratoryDisease, // Worsened by air pollution
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AcuteCondition {
    pub disease: DiseaseType,
    pub severity: f32,         // 0.0 = mild, 1.0 = critical
    pub days_remaining: u32,   // Natural course duration
    pub contagious: bool,
}
```

#### Environmental Health Effects

```rust
fn update_citizen_health(
    citizen: &mut HealthProfile,
    home_x: usize, home_y: usize,
    city_data: &CityData,
    has_healthcare: bool,
    is_employed: bool,
    commute_mode: TransportMode,
) {
    // --- Air pollution -> respiratory health ---
    let pollution = city_data.pollution_at(home_x, home_y);
    let pollution_damage = pollution as f32 / 255.0 * 0.5; // Up to 0.5 health/day
    citizen.physical -= pollution_damage;

    // Chronic conditions worsen faster in polluted areas
    for condition in &citizen.chronic_conditions {
        match condition {
            ChronicCondition::Asthma | ChronicCondition::RespiratoryDisease => {
                citizen.physical -= pollution_damage * 0.3; // Extra damage
            }
            _ => {}
        }
    }

    // --- Noise -> mental health ---
    let noise = city_data.noise_at(home_x, home_y);
    if noise > 50 { // Above 50 dB(A) chronic exposure
        citizen.mental -= (noise - 50) as f32 / 255.0 * 0.3;
    }

    // --- Green space -> health recovery ---
    let park_access = city_data.park_coverage_at(home_x, home_y);
    citizen.physical += park_access * 0.2; // Parks promote physical activity
    citizen.mental += park_access * 0.3;   // Nature reduces stress

    // --- Active transport -> fitness ---
    match commute_mode {
        TransportMode::Walking => citizen.physical += 0.15,
        TransportMode::Bicycle => citizen.physical += 0.20,
        TransportMode::Car => citizen.physical -= 0.05, // Sedentary
        _ => {}
    }

    // --- Employment -> mental health ---
    if !is_employed {
        citizen.mental -= 0.3; // Unemployment is devastating for mental health
    }

    // --- Healthcare access -> recovery ---
    if has_healthcare {
        // Healthcare slows chronic condition progression
        citizen.physical += 0.1;
        citizen.mental += 0.05;
        // Faster recovery from acute conditions
        if let Some(ref mut acute) = citizen.acute_condition {
            acute.days_remaining = acute.days_remaining.saturating_sub(1);
            acute.severity *= 0.95; // 5% severity reduction per day with care
        }
    }

    citizen.overall = (citizen.physical * 0.6 + citizen.mental * 0.4).clamp(0.0, 100.0);
    citizen.physical = citizen.physical.clamp(0.0, 100.0);
    citizen.mental = citizen.mental.clamp(0.0, 100.0);
}
```

### 8.2 SIR Disease Spread Model

The SIR (Susceptible-Infected-Recovered) model is the foundation of epidemiology. Citizens transition between states:

```
 S (Susceptible) --[infection]--> I (Infected) --[recovery]--> R (Recovered/Immune)
                                      |
                                      +--[death]-->  D (Dead)
```

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiseaseState {
    Susceptible,
    Exposed,      // Infected but not yet symptomatic (incubation)
    Infected,     // Symptomatic and contagious
    Recovered,    // Immune (temporarily or permanently)
    Dead,
}

#[derive(Resource)]
pub struct DiseaseOutbreak {
    pub disease: DiseaseType,
    pub state: OutbreakState,

    // SIR parameters
    pub beta: f32,    // Transmission rate (infections per contact per day)
    pub gamma: f32,   // Recovery rate (1/duration in days)
    pub mu: f32,      // Mortality rate (fraction of infected who die)
    pub sigma: f32,   // 1/incubation period (for SEIR model)

    // Tracking
    pub total_susceptible: u32,
    pub total_exposed: u32,
    pub total_infected: u32,
    pub total_recovered: u32,
    pub total_dead: u32,
}

#[derive(Debug, Clone, Copy)]
pub enum DiseaseType {
    Flu,            // beta=0.3, gamma=0.14 (7-day illness), mu=0.001
    Pandemic,       // beta=0.5, gamma=0.07 (14-day illness), mu=0.02
    Waterborne,     // Spreads through water system, not person-to-person
    FoodPoisoning,  // Localized to specific commercial buildings
}

impl DiseaseType {
    pub fn sir_parameters(&self) -> (f32, f32, f32, f32) {
        // (beta, gamma, mu, sigma)
        match self {
            Self::Flu => (0.3, 0.14, 0.001, 0.5),
            Self::Pandemic => (0.5, 0.07, 0.02, 0.2),
            Self::Waterborne => (0.0, 0.10, 0.005, 0.33),
            Self::FoodPoisoning => (0.0, 0.33, 0.001, 1.0),
        }
    }

    /// R0: basic reproduction number (average infections per case)
    /// R0 = beta / gamma
    /// Flu: R0 = 2.1, Pandemic: R0 = 7.1
    /// Disease dies out when R0 < 1 (herd immunity threshold = 1 - 1/R0)
    pub fn r0(&self) -> f32 {
        let (beta, gamma, _, _) = self.sir_parameters();
        beta / gamma
    }

    /// Herd immunity threshold: fraction that must be immune to stop spread
    pub fn herd_immunity_threshold(&self) -> f32 {
        1.0 - 1.0 / self.r0()
    }
}
```

#### Spatial SIR on the Grid

Disease spreads through spatial proximity. Rather than tracking every contact (too expensive at 100K+ agents), use a grid-based SIR where each cell tracks aggregate disease states:

```rust
#[derive(Resource)]
pub struct DiseaseGrid {
    /// Per-cell infected count
    pub infected: Vec<u16>,
    /// Per-cell recovered (immune) count
    pub recovered: Vec<u16>,
    pub width: usize,
    pub height: usize,
}

/// System: spread disease each tick
fn spread_disease(
    mut disease: ResMut<DiseaseOutbreak>,
    mut disease_grid: ResMut<DiseaseGrid>,
    population_grid: &PopulationGrid,
    healthcare_coverage: &ServiceCoverageGrid,
) {
    let (beta, gamma, mu, _sigma) = disease.disease.sir_parameters();
    let w = disease_grid.width;
    let h = disease_grid.height;

    let mut new_infections = vec![0u16; w * h];
    let mut new_recoveries = vec![0u16; w * h];

    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            let pop = population_grid.get(x, y) as f32;
            if pop == 0.0 { continue; }

            let infected = disease_grid.infected[idx] as f32;
            let recovered = disease_grid.recovered[idx] as f32;
            let susceptible = (pop - infected - recovered).max(0.0);

            // Force of infection: beta * I/N, with spatial coupling to neighbors
            let mut force = beta * infected / pop;

            // Add infection pressure from neighboring cells (spatial coupling)
            let coupling = 0.1; // 10% of neighbor's infection contributes
            for (nx, ny) in neighbors4(x, y, w, h) {
                let nidx = ny * w + nx;
                let n_pop = population_grid.get(nx, ny) as f32;
                if n_pop > 0.0 {
                    force += coupling * beta * disease_grid.infected[nidx] as f32 / n_pop;
                }
            }

            // Healthcare reduces mortality by 50-80%
            let healthcare_modifier = if healthcare_coverage.has_health(idx) {
                0.3 // 70% reduction in mortality
            } else {
                1.0
            };

            // New infections this tick
            let new_inf = (force * susceptible * 0.1).min(susceptible);
            new_infections[idx] = new_inf as u16;

            // Recoveries
            let new_rec = (gamma * infected * 0.1) as u16;
            new_recoveries[idx] = new_rec;

            // Deaths (from infected pool)
            let _new_dead = (mu * healthcare_modifier * infected * 0.1) as u16;
        }
    }

    // Apply updates
    for idx in 0..(w * h) {
        disease_grid.infected[idx] = disease_grid.infected[idx]
            .saturating_add(new_infections[idx])
            .saturating_sub(new_recoveries[idx]);
        disease_grid.recovered[idx] =
            disease_grid.recovered[idx].saturating_add(new_recoveries[idx]);
    }

    // Update outbreak totals
    disease.total_infected = disease_grid.infected.iter().map(|&x| x as u32).sum();
    disease.total_recovered = disease_grid.recovered.iter().map(|&x| x as u32).sum();
}
```

### 8.3 Healthcare System Modeling

Healthcare access creates a feedback loop: good healthcare -> healthier population -> higher productivity -> more tax revenue -> more healthcare investment.

```rust
/// Healthcare demand vs. supply ratio
fn healthcare_pressure(
    population_over_65: u32,
    total_population: u32,
    hospital_beds: u32,
    clinic_capacity: u32,
    active_disease: bool,
) -> f32 {
    // Elderly use 3-5x more healthcare than young adults
    let effective_demand = (total_population - population_over_65) as f32
                         + population_over_65 as f32 * 4.0;

    // Disease outbreaks multiply demand
    let disease_multiplier = if active_disease { 2.5 } else { 1.0 };

    let supply = (hospital_beds + clinic_capacity) as f32;
    let demand = effective_demand * disease_multiplier;

    if supply == 0.0 { return 10.0; }
    (demand / supply).max(0.1)
}
```

### 8.4 Aging Population Cost Curve

As the city ages, healthcare costs rise non-linearly:

| Age Group | % of Pop (young city) | % of Pop (mature city) | Healthcare cost/capita |
|-----------|----------------------|----------------------|----------------------|
| 0-17 | 25% | 18% | $3,000 |
| 18-44 | 35% | 30% | $4,500 |
| 45-64 | 25% | 25% | $8,500 |
| 65-74 | 10% | 15% | $15,000 |
| 75-84 | 4% | 8% | $22,000 |
| 85+ | 1% | 4% | $32,000 |

A city that attracted young workers in its early years will face a healthcare cost explosion 30-40 game-years later as that cohort ages. This creates a long-term strategic challenge that rewards planning: building hospitals before the crisis hits is much cheaper than scrambling during it.

---

## 9. Education Pipeline

### 9.1 Education System Architecture

Education is one of the most impactful feedback loops in a city simulation. School quality affects property values, which affects tax revenue, which funds schools. Breaking or reinforcing this loop is one of the most consequential decisions a player makes.

```rust
#[derive(Resource)]
pub struct EducationSystem {
    pub schools: Vec<SchoolData>,
    pub city_education_index: f32,      // 0.0-1.0, aggregate quality
    pub enrollment_rates: EnrollmentRates,
    pub graduation_rates: GraduationRates,
    pub teacher_student_ratio: f32,     // Target: 1:15 to 1:25
    pub annual_per_pupil_spending: f32, // US average: ~$14K
}

#[derive(Debug, Clone)]
pub struct SchoolData {
    pub entity: Entity,
    pub school_type: SchoolType,
    pub grid_x: usize,
    pub grid_y: usize,
    pub capacity: u32,
    pub current_enrollment: u32,
    pub quality: f32,           // 0.0-1.0
    pub funding_level: f32,     // Budget allocation relative to baseline
    pub teacher_quality: f32,   // Attracts/retains good teachers based on salary
    pub catchment_radius: i32,  // Grid cells served
}

#[derive(Debug, Clone, Copy)]
pub enum SchoolType {
    Kindergarten,      // Ages 3-5, capacity ~60
    Elementary,        // Ages 6-10, capacity ~400
    MiddleSchool,      // Ages 11-13, capacity ~600
    HighSchool,        // Ages 14-17, capacity ~1200
    University,        // Ages 18-22+, capacity ~5000
    CommunityCollege,  // Ages 18+, capacity ~2000
    VocationalSchool,  // Ages 16+, capacity ~500
}

impl SchoolType {
    pub fn age_range(&self) -> (u8, u8) {
        match self {
            Self::Kindergarten => (3, 5),
            Self::Elementary => (6, 10),
            Self::MiddleSchool => (11, 13),
            Self::HighSchool => (14, 17),
            Self::University | Self::CommunityCollege | Self::VocationalSchool => (18, 30),
        }
    }

    pub fn graduation_duration_years(&self) -> u8 {
        match self {
            Self::Kindergarten => 2,
            Self::Elementary => 5,
            Self::MiddleSchool => 3,
            Self::HighSchool => 4,
            Self::University => 4,
            Self::CommunityCollege => 2,
            Self::VocationalSchool => 2,
        }
    }
}
```

### 9.2 School Quality Model

School quality is determined by multiple factors, mirroring real education research:

```rust
fn compute_school_quality(
    school: &SchoolData,
    budget: &ExtendedBudget,
    neighborhood: &NeighborhoodData,
) -> f32 {
    // Factor 1: Funding (30%)
    // Measured as per-pupil spending relative to city average
    let funding_ratio = school.funding_level;
    let funding_score = sigmoid_satisfaction(funding_ratio, 1.0, 3.0);

    // Factor 2: Teacher quality (25%)
    // Teacher salary relative to median -> attracts better teachers
    // Research: teacher quality is the #1 in-school factor for student outcomes
    let teacher_score = school.teacher_quality;

    // Factor 3: Class size (15%)
    // Optimal: 15-20 students per teacher
    let class_size = school.current_enrollment as f32
        / (school.capacity as f32 * school.teacher_quality * 0.05).max(1.0);
    let class_size_score = if class_size <= 15.0 {
        1.0
    } else if class_size <= 20.0 {
        0.9
    } else if class_size <= 25.0 {
        0.7
    } else if class_size <= 30.0 {
        0.5
    } else {
        0.3
    };

    // Factor 4: Overcrowding (15%)
    let utilization = school.current_enrollment as f32 / school.capacity.max(1) as f32;
    let overcrowding_score = if utilization <= 0.85 {
        1.0
    } else if utilization <= 1.0 {
        0.8
    } else if utilization <= 1.15 {
        0.5 // Over capacity
    } else {
        0.2 // Severely overcrowded
    };

    // Factor 5: Neighborhood socioeconomic composition (15%)
    // Research: peer effects are significant -- high-poverty schools underperform
    // even with equal funding (Coleman Report, 1966)
    let neighborhood_score = 1.0 - neighborhood.poverty_rate * 0.8;

    // Weighted combination
    funding_score * 0.30
        + teacher_score * 0.25
        + class_size_score * 0.15
        + overcrowding_score * 0.15
        + neighborhood_score * 0.15
}
```

### 9.3 Enrollment Rate Model

Not all eligible children attend school. Enrollment depends on access, family economics, and school quality:

```rust
#[derive(Debug, Clone, Default)]
pub struct EnrollmentRates {
    pub kindergarten: f32,   // 60-85% (not mandatory in many places)
    pub elementary: f32,     // 95-99%
    pub middle_school: f32,  // 90-98%
    pub high_school: f32,    // 80-95%
    pub university: f32,     // 25-70% (highly variable by income)
}

fn enrollment_probability(
    child_age: u8,
    school_type: SchoolType,
    family_income: IncomeClass,
    nearest_school_distance: f32,
    school_quality: f32,
) -> f32 {
    let base_rate = match school_type {
        SchoolType::Kindergarten => 0.70,
        SchoolType::Elementary => 0.97,
        SchoolType::MiddleSchool => 0.95,
        SchoolType::HighSchool => 0.88,
        SchoolType::University => match family_income {
            IncomeClass::Poverty => 0.15,
            IncomeClass::LowIncome => 0.25,
            IncomeClass::LowerMiddle => 0.40,
            IncomeClass::UpperMiddle => 0.60,
            IncomeClass::HighIncome => 0.75,
            IncomeClass::Wealthy => 0.85,
        },
        SchoolType::CommunityCollege => 0.20,
        SchoolType::VocationalSchool => 0.10,
    };

    // Distance penalty: each 10 cells of distance reduces enrollment by 5%
    let distance_penalty = (nearest_school_distance / 10.0 * 0.05).min(0.30);

    // Quality bonus: good schools attract more enrollment
    let quality_modifier = 0.8 + school_quality * 0.4; // 0.8x to 1.2x

    (base_rate - distance_penalty) * quality_modifier
}
```

### 9.4 Education -> Property Value Feedback Loop

This is one of the most powerful dynamics in real cities and should be central to gameplay:

```
Good schools
    |
    v
Families move to area (housing demand increases)
    |
    v
Property values rise
    |
    v
Property tax revenue increases
    |
    v
School funding increases
    |
    v
School quality improves (feedback to top)
```

The inverse loop is equally powerful and destructive:

```
Poor schools
    |
    v
Families with means leave (white flight / income flight)
    |
    v
Property values fall
    |
    v
Tax revenue decreases
    |
    v
School funding cut
    |
    v
School quality declines further (death spiral)
```

```rust
/// System: Update land values based on school quality (runs monthly)
fn school_quality_affects_land_value(
    schools: Query<&SchoolData>,
    mut land_value: ResMut<LandValueGrid>,
) {
    for school in &schools {
        let quality = school.quality;
        let radius = school.catchment_radius;

        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let nx = school.grid_x as i32 + dx;
                let ny = school.grid_y as i32 + dy;
                if nx < 0 || ny < 0 { continue; }
                let (nx, ny) = (nx as usize, ny as usize);

                let dist = (dx.abs() + dy.abs()) as f32;
                let distance_decay = 1.0 - dist / radius as f32;

                // School quality premium: top schools add up to 20% to land value
                // Poor schools reduce land value by up to 10%
                let quality_premium = (quality - 0.5) * 0.40; // -0.20 to +0.20
                let effect = quality_premium * distance_decay;

                let current = land_value.get(nx, ny) as f32;
                let new_val = (current * (1.0 + effect)).clamp(0.0, 255.0);
                land_value.set(nx, ny, new_val as u8);
            }
        }
    }
}
```

### 9.5 Dropout and Graduation Rates

```rust
fn dropout_probability(
    student_age: u8,
    school_type: SchoolType,
    family_income: IncomeClass,
    student_happiness: f32,
    school_quality: f32,
) -> f32 {
    // Base dropout rates (annual) -- US national averages
    let base = match school_type {
        SchoolType::Elementary => 0.001,    // Almost zero
        SchoolType::MiddleSchool => 0.005,  // Very rare
        SchoolType::HighSchool => 0.04,     // ~4% per year (varies 2-10%)
        SchoolType::University => 0.08,     // ~8% per year (~40% don't finish in 6 years)
        _ => 0.02,
    };

    // Income modifier: poverty quadruples dropout risk
    let income_modifier = match family_income {
        IncomeClass::Poverty => 4.0,
        IncomeClass::LowIncome => 2.5,
        IncomeClass::LowerMiddle => 1.5,
        IncomeClass::UpperMiddle => 0.8,
        IncomeClass::HighIncome => 0.4,
        IncomeClass::Wealthy => 0.2,
    };

    // Unhappy students drop out more
    let happiness_modifier = if student_happiness < 30.0 {
        3.0
    } else if student_happiness < 50.0 {
        1.5
    } else {
        1.0
    };

    // Poor school quality increases dropout
    let quality_modifier = 2.0 - school_quality; // 1.0 at quality=1.0, 2.0 at quality=0.0

    (base * income_modifier * happiness_modifier * quality_modifier).min(0.30)
}
```

### 9.6 Education Outcomes and Workforce Quality

Education feeds directly into the labor market and economic productivity:

```rust
/// How education level affects job matching and productivity
fn education_workforce_impact(
    city_education_index: f32,
    workforce_education_distribution: &[f32; 6], // Fraction at each education level
) -> WorkforceQuality {
    // Human capital index (0.0-1.0)
    // Weighted by education level's productivity contribution
    let productivity_weights = [0.3, 0.5, 0.7, 1.0, 1.3, 1.5]; // None through Doctorate
    let human_capital: f32 = workforce_education_distribution.iter()
        .zip(productivity_weights.iter())
        .map(|(&frac, &weight)| frac * weight)
        .sum();

    // Innovation capacity: driven by university-educated workforce
    let innovation = (workforce_education_distribution[3]  // Bachelors
                    + workforce_education_distribution[4]   // Masters
                    + workforce_education_distribution[5])  // Doctorate
                    * 2.0; // Amplified because innovation is non-linear

    // Skill mismatch: penalizes cities where education doesn't match job requirements
    // High education + low-skill jobs = frustration; low education + high-skill jobs = unfilled positions

    WorkforceQuality {
        productivity_index: human_capital.clamp(0.0, 1.5),
        innovation_index: innovation.clamp(0.0, 1.0),
        education_index: city_education_index,
    }
}
```

### 9.7 Library System

Libraries serve as education infrastructure with unique benefits:

- **Children**: Improves school readiness (+0.05 school quality bonus within radius)
- **Adults**: Enables self-education (university-equivalent at 20% efficiency, free)
- **Elderly**: Social gathering point, reduces isolation
- **Unemployment**: Job search assistance, computer access, resume help
- **Digital divide**: Provides internet access to low-income residents

Libraries are extremely cost-effective per capita served ($40-60/person/year vs. $14K/student/year for schools) but politically vulnerable because their benefits are diffuse and hard to measure.

---

## 10. Immigration and Migration

### 10.1 Current System Analysis

The existing `immigration.rs` computes `CityAttractiveness` as a weighted sum of employment, happiness, services, housing availability, and tax rates. Scores above 60 trigger immigration waves; scores below 30 trigger emigration. This is a solid foundation that can be extended with push-pull factor modeling, demographic selectivity, and chain migration effects.

### 10.2 Push-Pull Utility Model

The Todaro model of migration (1969) says that people migrate when the expected utility at the destination exceeds the expected utility at the origin, minus the cost of moving:

```
Migrate if: E[U_destination] - E[U_origin] > C_migration
```

For a city builder, the "origin" is an abstract outside world, and the "destination" is the player's city. The outside world has a baseline utility that represents competing cities.

```rust
#[derive(Resource)]
pub struct MigrationModel {
    /// Baseline utility of the "outside world" -- competing cities
    /// Higher = harder to attract immigrants (late-game challenge)
    /// Starts at 40, rises with game year (simulates global development)
    pub outside_world_utility: f32,

    /// Migration cost (in utility units). Higher = less migration, more inertia.
    /// Physical moving cost + psychological cost of leaving social network
    pub migration_cost: f32,

    /// Maximum immigrants per wave (based on outside connection infrastructure)
    pub max_immigration_rate: u32,

    /// Demographic preferences of potential immigrants
    pub immigrant_pool: ImmigrantPool,
}

#[derive(Debug, Clone)]
pub struct ImmigrantPool {
    /// Age distribution of potential immigrants (peak at 25-35)
    pub age_distribution: [(u8, u8, f32); 6],

    /// Education distribution (varies by city's reputation)
    pub education_distribution: [f32; 6],

    /// Ethnic composition (varies by city's existing composition -- chain migration)
    pub ethnic_distribution: [f32; 6],
}

/// Compute city utility for a potential immigrant of a given profile
fn city_utility_for_immigrant(
    profile: &ImmigrantProfile,
    city: &CityData,
    attractiveness: &CityAttractiveness,
) -> f32 {
    // === PULL FACTORS (make city attractive) ===

    // Employment opportunity: probability of finding a job matching education
    let job_match = city.job_availability_for_education(profile.education);
    let expected_wage = city.median_wage_for_education(profile.education);
    let wage_utility = (expected_wage / 3000.0).ln().max(0.0); // Log utility of income

    // Housing availability: can they afford to live there?
    let affordable_housing = city.affordable_housing_fraction(expected_wage);
    let housing_utility = sigmoid_satisfaction(affordable_housing, 0.3, 5.0);

    // Services quality
    let services_utility = attractiveness.services_factor;

    // Safety
    let safety_utility = 1.0 - city.average_crime_level() / 100.0;

    // Network effects: are similar people already there? (chain migration)
    let chain_migration_bonus = city.ethnic_fraction(profile.ethnicity) * 0.3;
    // Chain migration is powerful: people follow existing communities
    // In the US, 60%+ of immigrants have family already in-country

    // === PUSH FACTORS (from outside, make anywhere attractive) ===
    // These are constant for all cities, representing global conditions
    let push_factor = 0.0; // Baseline; could be event-driven (war, famine)

    // === WEIGHTED UTILITY ===
    let utility = job_match * 0.30 * wage_utility
                + housing_utility * 0.20
                + services_utility * 0.15
                + safety_utility * 0.15
                + chain_migration_bonus * 0.10
                + push_factor * 0.10;

    utility * 100.0 // Scale to 0-100 for comparison with outside world
}
```

### 10.3 Immigrant Selectivity

Not all immigrants are the same. The composition of immigration depends on the city's characteristics:

```rust
fn generate_immigrant_profile(
    city: &CityData,
    rng: &mut impl Rng,
) -> ImmigrantProfile {
    // Age: immigrants skew young (25-35 peak)
    let age = weighted_random_age(rng, &[
        (18..=24, 0.20),
        (25..=34, 0.35),  // Peak immigration age
        (35..=44, 0.20),
        (45..=54, 0.10),
        (55..=64, 0.08),
        (65..=80, 0.07),  // Retiree migration (to amenity-rich cities)
    ]);

    // Education: depends on city's economy
    // High-tech cities attract educated immigrants
    // Industrial cities attract blue-collar immigrants
    let education = if city.tech_sector_fraction() > 0.3 {
        weighted_random(rng, &[
            (EducationLevel::HighSchool, 0.15),
            (EducationLevel::Bachelors, 0.35),
            (EducationLevel::Masters, 0.30),
            (EducationLevel::Doctorate, 0.20),
        ])
    } else if city.manufacturing_fraction() > 0.3 {
        weighted_random(rng, &[
            (EducationLevel::Elementary, 0.20),
            (EducationLevel::HighSchool, 0.45),
            (EducationLevel::Bachelors, 0.25),
            (EducationLevel::Masters, 0.10),
        ])
    } else {
        // Balanced economy
        weighted_random(rng, &[
            (EducationLevel::Elementary, 0.10),
            (EducationLevel::HighSchool, 0.35),
            (EducationLevel::Bachelors, 0.30),
            (EducationLevel::Masters, 0.15),
            (EducationLevel::Doctorate, 0.10),
        ])
    };

    // Ethnicity: chain migration -- 60% probability of matching existing majority
    let ethnicity = if rng.gen::<f32>() < 0.6 {
        city.largest_ethnic_group()
    } else {
        random_ethnic_group(rng)
    };

    ImmigrantProfile { age, education, ethnicity, /* ... */ }
}
```

### 10.4 Internal Migration (Intra-City)

Besides inter-city migration, citizens relocate *within* the city. This is far more common (Americans move every 5-7 years on average) and drives neighborhood change:

```rust
/// Probability that a household considers moving this year
fn annual_relocation_probability(
    household: &Household,
    citizen: &CitizenDetails,
    home_satisfaction: f32,
) -> f32 {
    let base = match citizen.life_stage() {
        LifeStage::YoungAdult => 0.25,  // Very mobile (apartment, first job changes)
        LifeStage::Adult => 0.12,       // Life events (family formation, job change)
        LifeStage::Senior => 0.05,      // Downsizing, retirement
        LifeStage::Retired => 0.03,     // Very low mobility
        _ => 0.05,
    };

    // Dissatisfaction increases mobility
    let satisfaction_modifier = if home_satisfaction < 0.3 {
        3.0 // Very unhappy -> actively looking
    } else if home_satisfaction < 0.5 {
        1.5
    } else if home_satisfaction > 0.8 {
        0.5 // Very happy -> unlikely to move
    } else {
        1.0
    };

    // Life events trigger moves
    let life_event_modifier = 1.0; // Multiplied by events (marriage=3x, baby=2x, divorce=4x)

    // Renters are more mobile than owners
    let tenure_modifier = if household.is_renter { 1.5 } else { 0.7 };

    (base * satisfaction_modifier * life_event_modifier * tenure_modifier).min(0.80)
}
```

### 10.5 Emigration Triggers

The existing system emigrates citizens with happiness below 20. A richer model considers multiple factors:

```rust
fn emigration_utility(
    citizen: &CitizenDetails,
    city_attractiveness: f32,
    outside_world_utility: f32,
    social_ties: u32,         // Number of family/friends in city
    years_lived_here: u32,
) -> f32 {
    // Leaving utility = outside opportunity - current satisfaction - social cost - inertia
    let current_utility = citizen.happiness;
    let opportunity_elsewhere = outside_world_utility;

    // Social ties create "stickiness" -- each close tie adds ~5 utility points
    let social_anchor = (social_ties as f32 * 5.0).min(30.0);

    // Duration of residence adds inertia (sunk cost, familiarity)
    let inertia = (years_lived_here as f32 * 2.0).min(20.0);

    // Moving cost
    let moving_cost = 10.0 + citizen.household_size as f32 * 5.0;

    let net_gain = opportunity_elsewhere - current_utility - social_anchor - inertia - moving_cost;

    // Only consider emigrating if net gain is significantly positive
    if net_gain > 10.0 {
        // Probability scales with net gain
        ((net_gain - 10.0) / 50.0).clamp(0.0, 0.5)
    } else {
        0.0
    }
}
```

This model explains why people stay in bad situations: social ties, familiarity, and moving costs create substantial inertia. Only when conditions deteriorate severely or outside opportunities dramatically improve do emigration waves occur.

### 10.6 Brain Drain and Talent Wars

When a city loses its educated population, it enters a "brain drain" spiral:

```
Educated workers leave
    |
    v
Workforce quality drops
    |
    v
High-skill businesses can't fill positions -> leave or don't locate
    |
    v
Tax revenue drops (high earners gone)
    |
    v
Services decline (budget cuts)
    |
    v
More educated workers leave (feedback to top)
```

The player counters brain drain by investing in universities, creating high-skill jobs (tech parks, research centers), and maintaining quality of life. The model tracks a "talent balance" metric:

```rust
struct TalentBalance {
    highly_educated_immigrants: u32,  // This year
    highly_educated_emigrants: u32,   // This year
    net_talent: i32,                  // Difference
    brain_drain_index: f32,           // Running average, 0.0=severe drain, 1.0=talent magnet
}
```

---

## 11. Homelessness

### 11.1 Current System Analysis

The existing `homelessness.rs` implements a three-stage pipeline: `check_homelessness` (detect citizens who lost housing or cannot afford rent), `seek_shelter` (find shelter capacity), and `recover_from_homelessness` (attempt to find permanent housing). This is a strong foundation. The deeper model adds causation pathways, intervention effectiveness data, and the distinction between temporary and chronic homelessness.

### 11.2 Pathways into Homelessness

Homelessness research identifies several distinct pathways, each requiring different interventions:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HomelessnessCause {
    // Economic (50-60% of cases)
    Eviction,              // Rent increased beyond affordability
    JobLoss,               // Lost income -> can't pay rent within savings buffer
    MedicalBankruptcy,     // Healthcare costs consumed savings

    // Structural (15-20%)
    HousingShortage,       // No affordable units available at any price
    NaturalDisaster,       // Home destroyed by fire, flood, earthquake
    Demolition,            // City demolished their building

    // Personal crisis (20-25%)
    MentalHealthCrisis,    // Severe depression, PTSD, psychosis
    SubstanceAbuse,        // Addiction consumed resources and relationships
    DomesticViolence,      // Fled unsafe home
    FamilyBreakdown,       // Divorce, family rejection (esp. youth)

    // Systemic (5-10%)
    AgingOutOfFosterCare,  // Aged out at 18 with no support
    ReentryFromPrison,     // Released with no housing plan
}

impl HomelessnessCause {
    /// Probability of transitioning from housed to homeless, per year
    /// Given the risk factors are present
    pub fn annual_risk(&self) -> f32 {
        match self {
            Self::Eviction => 0.15,           // 15% of evicted become homeless
            Self::JobLoss => 0.05,            // 5% if no savings buffer
            Self::MedicalBankruptcy => 0.08,
            Self::HousingShortage => 0.10,    // If vacancy rate < 2%
            Self::NaturalDisaster => 0.80,    // Most disaster victims temporarily homeless
            Self::Demolition => 0.95,         // Almost certain if no relocation plan
            Self::MentalHealthCrisis => 0.12,
            Self::SubstanceAbuse => 0.15,
            Self::DomesticViolence => 0.20,
            Self::FamilyBreakdown => 0.10,
            Self::AgingOutOfFosterCare => 0.25, // 25% of foster youth experience homelessness
            Self::ReentryFromPrison => 0.30,    // 30% of released inmates
        }
    }

    /// Average duration of homelessness episode (in game-days) without intervention
    pub fn expected_duration_days(&self) -> u32 {
        match self {
            Self::Eviction | Self::JobLoss => 60,              // Transitional
            Self::NaturalDisaster | Self::Demolition => 30,    // Usually temporary
            Self::MedicalBankruptcy => 120,
            Self::HousingShortage => 180,
            Self::MentalHealthCrisis => 365,                   // Can become chronic
            Self::SubstanceAbuse => 730,                       // Often chronic
            Self::DomesticViolence => 45,                      // Need safe housing fast
            Self::FamilyBreakdown => 90,
            Self::AgingOutOfFosterCare => 180,
            Self::ReentryFromPrison => 120,
        }
    }

    /// Required intervention type
    pub fn effective_intervention(&self) -> InterventionType {
        match self {
            Self::Eviction | Self::JobLoss | Self::HousingShortage => {
                InterventionType::HousingFirst  // Just need affordable housing
            }
            Self::MedicalBankruptcy => InterventionType::FinancialAssistance,
            Self::MentalHealthCrisis | Self::SubstanceAbuse => {
                InterventionType::SupportiveHousing  // Housing + wraparound services
            }
            Self::DomesticViolence => InterventionType::EmergencyShelter,
            Self::NaturalDisaster | Self::Demolition => InterventionType::TemporaryShelter,
            Self::AgingOutOfFosterCare | Self::ReentryFromPrison => {
                InterventionType::TransitionalHousing
            }
            Self::FamilyBreakdown => InterventionType::HousingFirst,
        }
    }
}
```

### 11.3 Intervention Effectiveness

Research data on intervention success rates, which can be used directly as game parameters:

```rust
#[derive(Debug, Clone, Copy)]
pub enum InterventionType {
    /// Emergency shelter: temporary beds, no services
    /// Prevents death from exposure but doesn't solve homelessness
    EmergencyShelter,

    /// Transitional housing: 6-24 month housing with case management
    /// 60-65% success rate (housed after 2 years)
    TransitionalHousing,

    /// Housing First: permanent housing with no preconditions
    /// 80-90% success rate (the most effective approach)
    /// But expensive: ~$20K/person/year
    HousingFirst,

    /// Supportive housing: permanent housing with mental health/addiction services
    /// 85% success rate for chronically homeless
    /// Most expensive: ~$30K/person/year, but saves $40K+ in emergency services
    SupportiveHousing,

    /// Financial assistance: one-time rent payment, deposit assistance
    /// 80% success rate for economically homeless
    /// Cheapest: ~$3K/person
    FinancialAssistance,

    /// Rapid rehousing: short-term rental assistance + case management
    /// 75% success rate
    /// $5-8K/person
    RapidRehousing,

    /// Temporary shelter: FEMA-style disaster relief
    TemporaryShelter,
}

impl InterventionType {
    /// Probability that a homeless person becomes permanently housed
    /// when this intervention is applied
    pub fn success_rate(&self) -> f32 {
        match self {
            Self::EmergencyShelter => 0.20,        // Shelter alone rarely solves it
            Self::TransitionalHousing => 0.62,
            Self::HousingFirst => 0.85,
            Self::SupportiveHousing => 0.87,
            Self::FinancialAssistance => 0.80,     // For economic causes only
            Self::RapidRehousing => 0.75,
            Self::TemporaryShelter => 0.15,
        }
    }

    /// Monthly cost per person served
    pub fn monthly_cost(&self) -> f32 {
        match self {
            Self::EmergencyShelter => 800.0,
            Self::TransitionalHousing => 1200.0,
            Self::HousingFirst => 1700.0,
            Self::SupportiveHousing => 2500.0,
            Self::FinancialAssistance => 250.0, // One-time, amortized
            Self::RapidRehousing => 600.0,
            Self::TemporaryShelter => 500.0,
        }
    }

    /// Cost savings from prevented emergency services (ER visits, police, jail)
    /// Chronically homeless people cost cities ~$35K-$100K/year in emergency services
    pub fn monthly_savings(&self) -> f32 {
        match self {
            Self::EmergencyShelter => 200.0,
            Self::TransitionalHousing => 800.0,
            Self::HousingFirst => 2500.0,       // Huge savings from prevented ER visits
            Self::SupportiveHousing => 3000.0,  // Even larger savings
            Self::FinancialAssistance => 100.0,
            Self::RapidRehousing => 500.0,
            Self::TemporaryShelter => 100.0,
        }
    }
}
```

### 11.4 Chronic vs. Transitional Homelessness

A critical distinction that affects policy:

```rust
fn classify_homelessness(homeless: &Homeless, cause: HomelessnessCause) -> HomelessnessType {
    if homeless.ticks_homeless > 365 * 10 { // More than ~1 game-year
        HomelessnessType::Chronic
    } else if homeless.episodes > 3 {
        HomelessnessType::Episodic // Repeatedly cycles in and out
    } else {
        HomelessnessType::Transitional // First episode, likely temporary
    }
}
```

Distribution in real cities:
- **Transitional** (75%): Short episodes, usually economic causes, self-resolving with minimal assistance
- **Episodic** (10%): Cycles between housed and homeless, often mental health or substance abuse related
- **Chronic** (15%): Long-term, most visible, consumes 50% of homeless services budget

The game should model this distribution: most homeless citizens recover on their own when housing becomes available, but a persistent core requires active intervention (shelters, supportive housing, mental health services).

### 11.5 Visible Impact on City

Homelessness creates visible effects that affect other citizens and the player:

```rust
fn homelessness_neighborhood_impact(
    homeless_count_in_area: u32,
    area_population: u32,
) -> NeighborhoodImpact {
    let concentration = homeless_count_in_area as f32 / area_population.max(1) as f32;

    NeighborhoodImpact {
        // Visible homelessness reduces property values
        land_value_modifier: 1.0 - (concentration * 3.0).min(0.30),

        // Increases perceived disorder (broken windows effect)
        disorder_increase: concentration * 0.5,

        // Reduces commercial activity (shoppers avoid areas with visible homelessness)
        commercial_traffic_modifier: 1.0 - (concentration * 2.0).min(0.40),

        // Political pressure increases (citizens demand action)
        political_pressure: (concentration * 100.0).min(30.0),

        // Compassion fatigue: initial sympathy fades with prolonged exposure
        sympathy_factor: (-concentration * 5.0).exp(), // Decays exponentially
    }
}
```

---

## 12. Social Mobility

### 12.1 Intergenerational Elasticity

Social mobility measures whether children end up in a different economic class than their parents. The intergenerational elasticity of income (IGE) is the key metric:

- **IGE = 0.0**: Perfect mobility (parent income doesn't predict child income)
- **IGE = 1.0**: Perfect immobility (children earn exactly what parents earned)
- **US reality**: IGE ~ 0.47 (moderate immobility)
- **Scandinavian countries**: IGE ~ 0.15-0.27 (high mobility)
- **Developing nations**: IGE ~ 0.6-0.7 (low mobility)

In the game, the player's city should have an IGE that responds to policy:

```rust
#[derive(Resource)]
pub struct SocialMobilityStats {
    /// Intergenerational income elasticity (lower = more mobility)
    pub ige: f32,

    /// Fraction of children born in bottom quintile who reach top quintile
    pub rags_to_riches_rate: f32,  // US: ~7.5%, Denmark: ~11.7%

    /// Fraction of children born in top quintile who fall to bottom quintile
    pub riches_to_rags_rate: f32,  // US: ~9.5%, Denmark: ~12.0%

    /// The "Great Gatsby Curve": correlation between inequality and immobility
    pub gatsby_curve_prediction: f32,
}

fn compute_ige(
    city: &CityData,
    education_system: &EducationSystem,
    housing_market: &HousingMarket,
) -> f32 {
    // Factors that INCREASE mobility (lower IGE):
    let education_quality = education_system.city_education_index;
    let education_access = education_system.enrollment_rates.university;
    let social_spending = city.social_spending_fraction(); // % of budget on social services
    let healthcare_access = city.healthcare_coverage();
    let neighborhood_integration = 1.0 - city.segregation_index();

    // Factors that DECREASE mobility (higher IGE):
    let inequality = city.gini_coefficient();
    let housing_cost_burden = city.median_rent_burden();
    let school_quality_variance = city.school_quality_standard_deviation();

    // The Great Gatsby Curve: inequality predicts immobility
    // IGE ~ 0.1 + 0.5 * Gini (rough linear fit across countries)
    let gatsby_baseline = 0.1 + 0.5 * inequality;

    // Policy effects modify the baseline
    let policy_modifier = 1.0
        - education_quality * 0.15      // Good schools reduce IGE by up to 15%
        - education_access * 0.10       // University access reduces IGE by up to 10%
        - social_spending * 0.10        // Social safety net reduces IGE
        - healthcare_access * 0.05      // Healthcare prevents poverty traps
        - neighborhood_integration * 0.10  // Integrated neighborhoods help
        + housing_cost_burden * 0.10    // High housing costs increase IGE
        + school_quality_variance * 0.10; // Unequal schools increase IGE

    (gatsby_baseline * policy_modifier).clamp(0.10, 0.80)
}
```

### 12.2 Child Outcome Model

When a child "grows up" (transitions from SchoolAge to YoungAdult), their outcomes are determined by a combination of family background and city-level factors:

```rust
fn determine_child_outcome(
    child: &CitizenDetails,
    parent_income: IncomeClass,
    parent_education: EducationLevel,
    school_quality: f32,
    neighborhood_quality: f32,
    city_ige: f32,
    rng: &mut impl Rng,
) -> ChildOutcome {
    // Parent advantage: higher income/education -> better starting position
    let parent_advantage = parent_income.rank() as f32 / 5.0 * 0.5
                         + parent_education.rank() as f32 / 5.0 * 0.5;

    // Meritocratic component: school quality + neighborhood + luck
    let merit = school_quality * 0.3
              + neighborhood_quality * 0.2
              + rng.gen::<f32>() * 0.5; // Substantial random component (life is uncertain)

    // Blend between parent advantage and merit, weighted by IGE
    // High IGE = parent advantage dominates; low IGE = merit dominates
    let blended = city_ige * parent_advantage + (1.0 - city_ige) * merit;

    // Map blended score to education outcome
    let education = if blended > 0.8 {
        EducationLevel::Masters  // Top 20% -> graduate degree
    } else if blended > 0.55 {
        EducationLevel::Bachelors
    } else if blended > 0.30 {
        EducationLevel::HighSchool
    } else {
        EducationLevel::Elementary // Dropout
    };

    // Income class at first job (correlated with education but with noise)
    let initial_income = match education {
        EducationLevel::Masters | EducationLevel::Doctorate => {
            if rng.gen::<f32>() < 0.7 { IncomeClass::UpperMiddle } else { IncomeClass::LowerMiddle }
        }
        EducationLevel::Bachelors => {
            if rng.gen::<f32>() < 0.5 { IncomeClass::LowerMiddle } else { IncomeClass::LowIncome }
        }
        EducationLevel::HighSchool => {
            if rng.gen::<f32>() < 0.4 { IncomeClass::LowIncome } else { IncomeClass::Poverty }
        }
        _ => IncomeClass::Poverty,
    };

    ChildOutcome { education, initial_income }
}
```

### 12.3 Opportunity Zones

Certain neighborhoods have systematically better outcomes for children. Raj Chetty's research shows that moving a child from a bottom-quartile neighborhood to a top-quartile neighborhood at age 8 increases adult earnings by ~10%.

```rust
/// Neighborhood opportunity score (0.0-1.0)
/// Based on the Chetty-Hendren neighborhood effect research
fn neighborhood_opportunity_score(
    x: usize, y: usize,
    school_quality: f32,
    crime: f32,
    poverty_rate: f32,
    two_parent_household_rate: f32,
    social_capital: f32,  // Community organizations, voter turnout
) -> f32 {
    // Weights from Chetty et al. (2018) "The Opportunity Atlas"
    let score = school_quality * 0.25
              + (1.0 - crime) * 0.20
              + (1.0 - poverty_rate) * 0.20
              + two_parent_household_rate * 0.15
              + social_capital * 0.20;

    score.clamp(0.0, 1.0)
}
```

The player can visualize opportunity scores as an overlay, identifying neighborhoods where children have the best and worst chances of upward mobility. This provides a clear policy target: improving low-opportunity neighborhoods has the highest long-term return on investment.

---

## 13. Governance and Politics

### 13.1 Approval Rating System

The player's actions have political consequences. Citizens form opinions about the mayor (player) based on their personal experience:

```rust
#[derive(Resource)]
pub struct MayoralApproval {
    pub overall_approval: f32,       // 0-100%
    pub economy_approval: f32,
    pub services_approval: f32,
    pub safety_approval: f32,
    pub environment_approval: f32,
    pub housing_approval: f32,

    /// Recent actions and their approval impact
    pub recent_actions: Vec<PolicyAction>,

    /// Election timer
    pub next_election_day: u32,
    pub terms_served: u32,
}

fn compute_approval(
    citizens: &Query<(&CitizenDetails, &Demographics)>,
    city_stats: &CityStats,
    recent_policies: &[PolicyAction],
) -> f32 {
    // Base approval = average happiness rescaled to approval
    let base = city_stats.average_happiness;

    // Recent policy effects (recency-weighted)
    let mut policy_effect = 0.0f32;
    for action in recent_policies {
        let recency_weight = (-action.days_ago as f32 / 180.0).exp(); // Half-life ~180 days
        policy_effect += action.approval_impact * recency_weight;
    }

    // Economic conditions: "Are you better off than 4 years ago?"
    let economic_trend = city_stats.gdp_growth_rate * 10.0; // +/- 10% per 1% GDP growth

    // Negativity bias: scandals and failures weigh 2x more than successes
    let net_effect = if policy_effect < 0.0 {
        policy_effect * 2.0
    } else {
        policy_effect
    };

    (base + net_effect + economic_trend).clamp(0.0, 100.0)
}
```

### 13.2 NIMBY Politics (Not In My Backyard)

One of the most realistic and frustrating political dynamics: citizens support public goods in theory but oppose them near their homes.

```rust
#[derive(Debug, Clone)]
pub struct NimbyReaction {
    pub facility_type: FacilityType,
    pub opposition_radius: i32,      // Grid cells affected
    pub opposition_intensity: f32,   // 0.0-1.0
    pub affected_citizens: u32,
    pub approval_impact: f32,        // Negative impact on mayor approval
}

#[derive(Debug, Clone, Copy)]
pub enum FacilityType {
    // Strong NIMBY opposition
    Prison,              // -30 approval in 10-cell radius
    Landfill,            // -25 approval in 15-cell radius
    WasteTreatment,      // -20 approval in 10-cell radius
    HomelessShelter,     // -15 approval in 5-cell radius
    HalfwayHouse,        // -15 approval in 5-cell radius
    HighDensityHousing,  // -10 approval in 3-cell radius
    WindTurbine,         // -8 approval in 5-cell radius (noise + visual)
    CellTower,           // -5 approval in 3-cell radius

    // Mild NIMBY (traffic/noise concerns)
    Stadium,             // -5 approval in 5-cell radius (event days)
    NightClub,           // -5 approval in 3-cell radius
    Highway,             // -10 approval in 2-cell radius

    // YIMBY (generally welcome)
    Park,                // +5 approval in 5-cell radius
    Library,             // +3 approval in 5-cell radius
    School,              // +5 approval in 5-cell radius (families)
    FireStation,         // +3 approval in 8-cell radius

    // Mixed reactions
    PublicTransit,       // +5 approval city-wide, -5 near construction
    Hospital,            // +10 approval wide, -3 immediate neighbors (ambulance noise)
}

impl FacilityType {
    pub fn nimby_score(&self) -> (f32, i32) {
        // (opposition_intensity, radius)
        match self {
            Self::Prison => (0.90, 10),
            Self::Landfill => (0.85, 15),
            Self::WasteTreatment => (0.70, 10),
            Self::HomelessShelter => (0.60, 5),
            Self::HalfwayHouse => (0.65, 5),
            Self::HighDensityHousing => (0.40, 3),
            Self::WindTurbine => (0.30, 5),
            Self::CellTower => (0.20, 3),
            Self::Stadium => (0.20, 5),
            Self::NightClub => (0.25, 3),
            Self::Highway => (0.50, 2),
            Self::Park => (-0.30, 5),          // Positive
            Self::Library => (-0.15, 5),
            Self::School => (-0.20, 5),
            Self::FireStation => (-0.15, 8),
            Self::PublicTransit => (-0.10, 3),
            Self::Hospital => (-0.20, 8),
        }
    }
}

/// Compute NIMBY opposition when a facility is proposed
fn compute_nimby_opposition(
    facility: FacilityType,
    location: (usize, usize),
    citizens: &Query<(&CitizenDetails, &Demographics, &HomeLocation)>,
) -> NimbyReaction {
    let (intensity, radius) = facility.nimby_score();
    let mut affected = 0u32;
    let mut total_opposition = 0.0f32;

    for (details, demographics, home) in citizens.iter() {
        let dx = (home.grid_x as i32 - location.0 as i32).abs();
        let dy = (home.grid_y as i32 - location.1 as i32).abs();
        let dist = dx + dy;

        if dist <= radius {
            affected += 1;
            let distance_decay = 1.0 - dist as f32 / radius as f32;

            // Wealthy citizens oppose more (more political influence, more to lose)
            let wealth_modifier = match demographics.income_class {
                IncomeClass::HighIncome | IncomeClass::Wealthy => 1.5,
                IncomeClass::UpperMiddle => 1.2,
                _ => 1.0,
            };

            // Property owners oppose more than renters (property values at stake)
            let tenure_modifier = if demographics.is_homeowner { 1.3 } else { 0.8 };

            total_opposition += intensity * distance_decay * wealth_modifier * tenure_modifier;
        }
    }

    let approval_impact = if affected > 0 {
        -(total_opposition / affected as f32) * 10.0
    } else {
        0.0
    };

    NimbyReaction {
        facility_type: facility,
        opposition_radius: radius,
        opposition_intensity: intensity,
        affected_citizens: affected,
        approval_impact,
    }
}
```

### 13.3 Election System

Periodic elections create stakes for the player's decisions:

```rust
#[derive(Resource)]
pub struct ElectionSystem {
    pub election_cycle_days: u32,     // 365 * 4 = every 4 game-years
    pub days_until_election: u32,
    pub campaign_season: bool,        // Last 90 days before election
    pub incumbent_approval: f32,
    pub challenger_strength: f32,     // 0.0-1.0, random challenger quality
}

fn election_result(
    approval: f32,
    challenger_strength: f32,
    economic_trend: f32,    // Positive = good for incumbent
    rng: &mut impl Rng,
) -> ElectionOutcome {
    // Incumbent advantage: +5% baseline
    let incumbent_score = approval * 0.6
                        + economic_trend * 20.0  // Economy is strongest predictor
                        + 5.0;                    // Incumbency advantage

    let challenger_score = (100.0 - approval) * 0.5
                         + challenger_strength * 30.0
                         + rng.gen::<f32>() * 10.0; // Random campaign events

    if incumbent_score > challenger_score {
        ElectionOutcome::Reelected
    } else {
        ElectionOutcome::Defeated // Game over? Or reduced authority? Policy resets?
    }
}
```

### 13.4 Protest System

When citizen grievances reach critical mass, protests emerge:

```rust
#[derive(Debug, Clone)]
pub struct Protest {
    pub location: (usize, usize),
    pub cause: ProtestCause,
    pub participants: u32,
    pub intensity: f32,         // 0.0=peaceful vigil, 1.0=riot
    pub days_active: u32,
    pub media_attention: f32,   // Affects approval impact
}

#[derive(Debug, Clone, Copy)]
pub enum ProtestCause {
    HighTaxes,
    PoorServices,
    Pollution,
    Homelessness,
    PoliceBrutality,
    HousingCosts,
    TrafficCongestion,
    CorruptionScandal,
    FacilityPlacement,  // NIMBY protest
    LaborDispute,
}

fn should_protest_spawn(
    cause: ProtestCause,
    affected_citizens: u32,
    grievance_level: f32,  // 0.0-1.0
    existing_protests: u32,
) -> bool {
    // Threshold model: protests start when grievance exceeds threshold
    // AND a critical mass of affected citizens exists
    let threshold = match cause {
        ProtestCause::HighTaxes => 0.6,
        ProtestCause::PoorServices => 0.7,
        ProtestCause::PoliceBrutality => 0.4,  // Low threshold (emotional)
        ProtestCause::HousingCosts => 0.65,
        ProtestCause::FacilityPlacement => 0.5,
        _ => 0.6,
    };

    let critical_mass = 50; // Minimum protesters to form visible protest

    // Contagion: existing protests lower the threshold for new ones
    let contagion_modifier = 1.0 - existing_protests as f32 * 0.05;

    grievance_level > threshold * contagion_modifier && affected_citizens > critical_mass
}
```

### 13.5 Policy Effects and Implementation Lag

Real policies don't take effect instantly. Implementation lag creates a gap between policy announcement and measurable results, during which approval may drop:

```rust
struct PolicyImplementation {
    policy: PolicyType,
    announced_day: u32,
    implementation_progress: f32, // 0.0-1.0
    full_effect_day: u32,
    immediate_cost: f32,         // Budget impact from day 1
    steady_state_effect: f32,    // Long-run benefit (may take years)
}

/// Policy lag times (game-days until full effect)
fn policy_lag(policy: PolicyType) -> u32 {
    match policy {
        PolicyType::TaxChange => 30,                // Quick to implement
        PolicyType::ZoningChange => 180,            // 6 months bureaucracy
        PolicyType::NewSchool => 730,               // 2 years construction + staffing
        PolicyType::NewHospital => 1095,            // 3 years
        PolicyType::PublicTransitLine => 1825,      // 5 years (or more)
        PolicyType::InclusionaryZoning => 1095,     // 3 years for units to appear
        PolicyType::PoliceReform => 365,            // 1 year training + culture shift
        PolicyType::EnvironmentalRegulation => 730, // 2 years for compliance
        PolicyType::RentControl => 60,              // Quick but contentious
    }
}
```

The player must plan ahead: policies started now produce results years from now. This rewards strategic thinking and punishes reactive governance.

---

## 14. How Commercial Games Do It

### 14.1 Victoria 3 -- Population Groups (Pops)

Victoria 3 (Paradox, 2022) uses the most sophisticated population simulation in commercial games. Rather than simulating individuals, it groups citizens into "Pops" -- aggregated population units sharing key attributes.

#### Pop Grouping

Each Pop represents a group sharing:
- **Culture** (e.g., British, French, Chinese)
- **Religion** (e.g., Protestant, Catholic, Buddhist)
- **Profession/Strata** (Aristocrats, Capitalists, Professionals, Clerks, Laborers, Peasants, Slaves)
- **State** (geographic location within the country)

A state with 5 cultures, 4 religions, and 7 professions could have up to 140 pops, but most combinations are sparsely populated so actual pop count is lower. The global pop count is ~10K-30K, each representing thousands to millions of people.

#### Standard of Living Formula

Victoria 3 computes Standard of Living per pop:

```
SoL = base_from_profession
    + wealth_contribution (from pop savings)
    + goods_satisfaction (are they consuming enough food, clothes, luxury?)
    - taxation_burden
    + political_freedoms_bonus
    + social_security_bonus
```

Each pop has needs broken into tiers:
1. **Subsistence needs**: grain, meat, fish, clothes (must be met or starvation)
2. **Basic needs**: furniture, heating, alcohol (improve SoL by 5-10)
3. **Comfort needs**: tea, coffee, tobacco, luxury clothes (improve SoL by 3-5)
4. **Luxury needs**: fine art, automobiles, telephones (improve SoL by 2-3 each)

Pops buy goods from the market based on their budget. If prices are too high, they go unsatisfied. If trade brings in cheap goods, everyone benefits. This creates realistic economic feedback: industrialization makes goods cheap, raising living standards, increasing population, creating labor supply, enabling more industrialization.

#### Political Interest Groups

Pops join Interest Groups (IGs) based on their profession and values:
- **Industrialists**: Capitalists, want free trade, low taxes on business
- **Landowners**: Aristocrats, want traditional economy, no land reform
- **Trade Unions**: Laborers, want workers' rights, higher wages
- **Intelligentsia**: Professionals, want education, political reform
- **Armed Forces**: Military, want prestige, defense spending
- **Devout**: Religious, want traditional values, religious education

Each IG has a **clout** value (political power) proportional to the wealth and population of its member pops. Clout determines policy influence, and pops shift between IGs based on their material conditions.

#### Key Formulas

```
// Pop political strength
pop_political_strength = pop_size * literacy_factor * wealth_factor * civil_rights_modifier

// Interest Group clout
ig_clout = sum(member_pop_political_strength) / total_political_strength * 100

// Revolution risk
radicalism = max(0, (expected_SoL - actual_SoL) * 2.0)
// Expected SoL rises with education, cultural exposure, neighboring nations' SoL

// Loyalty to government
loyalty = base
    + (approves_of_laws * 0.3)
    - (disapproves_of_laws * 0.5)
    + (ig_in_government * 20)
    - (ig_excluded * 10)
```

#### Applicability to Megacity

Megacity could use a pop-like system for the "statistical far-away" LOD tier. Instead of tracking 100K individual agents, track ~500-2000 population groups defined by (neighborhood, income_class, education, ethnicity). Each pop group stores aggregate statistics (average happiness, employment rate, political alignment) and produces aggregate behaviors (demand for housing, services, transit). Only the ~5K-10K "nearby" citizens are tracked individually.

### 14.2 Tropico -- Faction Loyalty System

The Tropico series (Haemimont Games / Limbic Entertainment) models citizens through faction alignment. Each citizen has loyalty to multiple factions simultaneously:

#### Factions and Loyalty

Tropico uses 6-8 factions:
- **Militarists**: Want military buildings, police state
- **Religious**: Want churches, cathedral, traditional values
- **Capitalists**: Want industry, free trade, low taxes
- **Communists**: Want equality, state housing, social programs
- **Intellectuals**: Want education, libraries, free press
- **Environmentalists**: Want parks, clean energy, nature
- **Nationalists**: Want sovereignty, heritage buildings
- **Loyalists**: Support you no matter what (bought with favors)

#### Loyalty Calculation

```
// Per-citizen faction loyalty
loyalty_to_faction = base_affinity (from traits)
    + housing_quality * faction_housing_weight
    + job_satisfaction * faction_job_weight
    + church_attendance * faction_religion_weight
    + environment_quality * faction_environment_weight
    + freedom_level * faction_freedom_weight
    + military_strength * faction_military_weight
    + equality_index * faction_equality_weight
    + specific_building_bonuses

// Citizen's overall political alignment
dominant_faction = argmax(faction_loyalties)

// Faction support = count(citizens where dominant_faction == this_faction) / total_citizens

// Approval of El Presidente
citizen_approval = overall_happiness * 0.5
    + faction_specific_satisfaction * 0.3
    + personal_bribe_effect * 0.1
    + fear_from_military * 0.1
```

The genius of Tropico's system is that factions create **impossible tradeoffs**: building a church pleases Religious but angers Intellectuals. Building a factory pleases Capitalists but angers Environmentalists. The player must balance competing demands, which is the core political gameplay loop.

#### Applicability to Megacity

Faction-style grouping works well as a UI layer. Even if citizens are individually simulated, displaying aggregate faction satisfaction gives the player actionable information: "Environmentalists: 35% approval (low -- build parks or reduce pollution)."

### 14.3 Dwarf Fortress -- Individual Simulation

Dwarf Fortress (Bay 12 Games) represents the opposite extreme: every dwarf is fully individually simulated with detailed needs, preferences, relationships, memories, beliefs, and emotional states.

#### Individual Attributes

Each dwarf has:
- **100+ personality traits** (facets like anxiety, anger, creativity, orderliness)
- **Social relationships** with every other dwarf (friendship, grudge, family, mentor)
- **Memories** of events (saw a beautiful statue, witnessed a death, ate a nice meal)
- **Skills** in 100+ categories with experience gain
- **Needs** (practice a martial art, pray, drink, create art, socialize, etc.)
- **Goals/dreams** (become a legendary smith, start a family, collect wealth)

#### Thought System

Dwarves accumulate "thoughts" -- positive and negative memories with decay:

```
// Simplified Dwarf Fortress thought model
thought_stack = [
    ("Admired a masterwork statue", +30, decay=120_days),
    ("Rained on recently", -5, decay=30_days),
    ("Had a fantastic meal", +10, decay=60_days),
    ("Forced to work without a break", -20, decay=90_days),
    ("Hasn't talked to a friend", -15, decay=ongoing),
    ("Witnessed death of a friend", -80, decay=365_days),
]

overall_mood = base_personality + sum(thought.value * thought.recency_weight)
```

When mood drops below thresholds, dwarves have breakdowns:
- **Unhappy**: Reduced work speed, negative social interactions
- **Very unhappy**: Tantrums, destroying items, picking fights
- **Miserable**: Full mental break, berserk rampage or suicidal behavior
- **Spiraling**: The "tantrum spiral" -- one dwarf's breakdown traumatizes others, cascading

#### Applicability to Megacity

Dwarf Fortress's approach is too expensive for 100K+ agents but incredibly compelling for the "nearby" LOD tier. The 500-2000 visible citizens closest to the camera could track individual thoughts, relationships, and memories. When the player clicks a citizen, they see a rich narrative: "Unhappy because: crowded apartment (-10), long commute (-8), recently got a raise (+5), neighborhood park is nice (+3), worried about crime (-12)."

### 14.4 RimWorld -- Mood Threshold Mental Breaks

RimWorld (Ludeon Studios) uses a simpler version of Dwarf Fortress's system, focused on mood thresholds that trigger behavioral changes:

#### Mood System

```
// RimWorld mood calculation
mood = base_50
    + thought_stack_sum  // -50 to +50 range
    + trait_modifiers     // Optimist: +6, Pessimist: -6
    + health_modifiers    // Injured: -5 per injury
    + comfort_bonus       // Nice room: +5
    + outdoor_bonus       // Time outside: +3
    - expectations_penalty // High expectations from wealth: -10 to -15
```

#### Threshold Breaks

```
Mental break thresholds (mood 0-100):
  0-5:   Extreme break (berserk, catatonic, give up entirely)
  5-15:  Major break (wander aimlessly, start fires, binge eat)
  15-25: Minor break (insult spree, food binge, hide in room)
  25-35: Stressed (work speed -10%, social fights more likely)
  35-65: Neutral
  65-80: Content (work speed +5%)
  80-100: Happy (inspired creativity, inspired work speed +35%)
```

The key insight is **expectations scaling**: wealthy colonists have higher expectations, so the same conditions that make a poor colonist happy make a wealthy one miserable. This maps perfectly to relative deprivation theory (Section 5.4).

#### RimWorld's "Expectations" system

```
Expectations by wealth level:
  Extremely low (total wealth < $4K): No penalty
  Low ($4K-$15K): -3 mood
  Moderate ($15K-$75K): -6 mood
  High ($75K-$300K): -10 mood
  Extremely high ($300K+): -14 mood
  Sky high ($1M+): -18 mood
```

This is a simple implementation of the hedonic treadmill: as wealth increases, the baseline mood shifts downward, requiring better conditions to maintain the same happiness level.

#### Applicability to Megacity

The threshold system maps directly. Define mood thresholds for citizens that trigger visible behavioral changes: protests at miserable, crime at very unhappy, emigration at unhappy, volunteering at happy, community events at ecstatic. This gives the player clear feedback about population mood distribution.

### 14.5 Frostpunk -- Hope and Discontent

Frostpunk (11 bit studios) uses a dual-track system: **Hope** (long-term motivation) and **Discontent** (short-term frustration).

#### Dual Track System

```
Hope (0-100):
  Sources: New laws passed, successful expeditions, construction, speeches
  Drains: Starvation, death, harsh conditions, broken promises
  Effect: Hope=0 triggers game over (citizens abandon city)
  Mechanic: Asymmetric -- hope is hard to build and easy to lose

Discontent (0-100):
  Sources: Hunger, cold, overwork, crime, unfair laws, death
  Drains: Food distribution, heating, days off, good housing, entertainment
  Effect: Discontent=100 triggers revolution (game over)
  Mechanic: Constantly rising unless actively managed
```

The brilliance of Frostpunk's design is the **tension between hope and discontent**: some actions that raise hope (authoritarian laws, propaganda) also raise discontent. Some actions that lower discontent (days off, extra rations) also cost resources that could prevent future crises, lowering hope long-term.

#### Applicability to Megacity

A dual-track system could work for Megacity's political layer:
- **Civic Trust**: Long-term confidence in the city's future (similar to Hope). Built by infrastructure, consistent governance, economic growth. Destroyed by scandals, broken promises, decline.
- **Grievance**: Short-term frustration (similar to Discontent). Caused by daily problems (traffic, crime, pollution). Reduced by responsive action.

When Civic Trust falls below 20% and Grievance exceeds 80%, an election is called early (vote of no confidence).

### 14.6 Comparison Matrix

| Feature | Victoria 3 | Tropico | Dwarf Fortress | RimWorld | Frostpunk | Megacity (proposed) |
|---------|-----------|---------|---------------|----------|-----------|-------------------|
| Agent granularity | Pop groups (~10K) | Individual (300-1000) | Individual (200-300) | Individual (3-30) | Aggregate only | Individual + LOD |
| Max population | Millions (aggregated) | ~1000 | ~300 | ~30 | ~600 | 100K+ |
| Demographics | Culture/religion/job | Basic traits | Deep personality | Traits + backstory | None | Full demographic |
| Economic model | Supply/demand market | Simple needs | Craft economy | Barter/trade | Survival | Market + zones |
| Political model | Interest groups + laws | Factions + elections | Nobles + mandates | None | Hope/Discontent | Approval + elections |
| Happiness model | Standard of Living | Multi-factor | Thought stack | Mood + breaks | Hope/Discontent | Multi-factor + adaptation |
| Segregation | By culture/religion | By faction | By caste | N/A | N/A | Schelling model |
| Crime | None | Basic | Individual acts | Individual acts | None | Spatial + causal |
| Disease | Epidemics (aggregate) | None | Individual (infections) | Individual (diseases) | Frostbite/illness | SIR spatial |
| Education | Literacy rate | Basic | Skill training | Skill learning | None | Full pipeline |
| Performance trick | Pop aggregation | Small population | Small population | Small population | Aggregate | LOD tiers |

---

## 15. Performance: ECS Patterns for 100K+ Agents

### 15.1 The Fundamental Challenge

At 100K citizens with the full demographic model described in this document, the naive approach is untenable:
- 100K entities x ~20 components each = 2M component accesses per system
- If 30 systems run per tick at 10 Hz = 60M component accesses per second
- Each citizen happiness update touches 15+ resources = 1.5M resource reads per tick

The existing Megacity codebase already uses several performance techniques (parallel iteration, tick staggering, bitflag service coverage, spatial grids). This section describes additional ECS patterns specifically for social simulation at scale.

### 15.2 LOD for Behavior (The Key Insight)

Just as rendering uses Level-of-Detail to simplify distant geometry, social simulation should use behavioral LOD to simplify distant agents. The existing LOD system has three tiers: Full, Simplified, Abstract. Extending this to social behavior:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BehaviorLOD {
    /// Full individual simulation (closest to camera, ~500-2000 citizens)
    /// - Full pathfinding with A* on road graph
    /// - Individual decision making (utility functions)
    /// - Thought stack with memories
    /// - Full schedule generation and execution
    /// - Visible movement on screen
    /// Per-citizen cost: ~50us/tick
    Full,

    /// Simplified simulation (visible but distant, ~5000-15000 citizens)
    /// - Simplified movement (lerp between waypoints, no A*)
    /// - Statistical decision making (probability tables instead of utility)
    /// - No thought stack, only current mood
    /// - Schedule = template for demographic group
    /// - Visible as dots/sprites on screen
    /// Per-citizen cost: ~5us/tick
    Simplified,

    /// Statistical simulation (off-screen, ~80000+ citizens)
    /// - No individual movement or pathfinding
    /// - Pop-group level simulation (Victoria 3 style)
    /// - Aggregate happiness from neighborhood conditions
    /// - Demographic transitions via probability tables
    /// - Invisible, only statistics
    /// Per-citizen cost: ~0.1us/tick (aggregated)
    Statistical,
}
```

#### LOD Transition System

```rust
/// System: Assign behavior LOD based on camera distance
fn assign_behavior_lod(
    camera: Query<&Transform, With<Camera>>,
    mut citizens: Query<(&Position, &mut BehaviorLodComp), With<Citizen>>,
) {
    let cam = camera.single();
    let cam_pos = cam.translation.truncate();

    // LOD boundaries (in world units)
    const FULL_RADIUS: f32 = 512.0;        // ~32 cells
    const SIMPLIFIED_RADIUS: f32 = 2048.0;  // ~128 cells
    // Beyond simplified = Statistical

    citizens.par_iter_mut().for_each(|(pos, mut lod)| {
        let dist_sq = (pos.x - cam_pos.x).powi(2) + (pos.y - cam_pos.y).powi(2);

        let new_lod = if dist_sq < FULL_RADIUS * FULL_RADIUS {
            BehaviorLOD::Full
        } else if dist_sq < SIMPLIFIED_RADIUS * SIMPLIFIED_RADIUS {
            BehaviorLOD::Simplified
        } else {
            BehaviorLOD::Statistical
        };

        if lod.0 != new_lod {
            lod.0 = new_lod;
        }
    });
}
```

#### Statistical Tier: Pop-Group Aggregation

For the 80K+ citizens in the Statistical tier, don't iterate per-entity at all. Instead, aggregate into ~200-1000 "pop groups" defined by (neighborhood_chunk, income_class, education_level):

```rust
#[derive(Resource)]
pub struct PopulationGroups {
    pub groups: Vec<PopGroup>,
    /// Map from (chunk_x, chunk_y, income, education) -> group index
    pub index: HashMap<PopGroupKey, usize>,
    pub dirty: bool,
}

#[derive(Debug, Clone)]
pub struct PopGroup {
    pub key: PopGroupKey,
    pub count: u32,
    pub average_happiness: f32,
    pub average_health: f32,
    pub employment_rate: f32,
    pub average_income: f32,
    pub average_age: f32,
    pub crime_exposure: f32,
    pub service_coverage: f32,
    pub emigration_pressure: f32,

    /// Entity list for on-demand individual access
    pub members: Vec<Entity>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PopGroupKey {
    pub chunk_x: u8,       // 256/8 = 32 chunks
    pub chunk_y: u8,
    pub income_class: u8,  // 6 levels
    pub education: u8,     // 6 levels
}
// Max groups: 32 * 32 * 6 * 6 = 36,864, but most are empty
// Typical: 200-1000 active groups
```

The Statistical tier happiness system runs on pop groups, not individuals:

```rust
fn update_statistical_happiness(
    mut groups: ResMut<PopulationGroups>,
    crime_grid: Res<CrimeGrid>,
    pollution_grid: Res<PollutionGrid>,
    coverage: Res<ServiceCoverageGrid>,
    land_value: Res<LandValueGrid>,
) {
    for group in &mut groups.groups {
        if group.count == 0 { continue; }

        let cx = group.key.chunk_x as usize * 8 + 4; // Chunk center
        let cy = group.key.chunk_y as usize * 8 + 4;

        // Sample conditions at chunk center (not per-citizen!)
        let crime = crime_grid.get(cx, cy) as f32 / 255.0;
        let pollution = pollution_grid.get(cx, cy) as f32 / 255.0;
        let idx = ServiceCoverageGrid::idx(cx, cy);
        let has_services = coverage.flags[idx].count_ones() as f32 / 8.0;
        let lv = land_value.get(cx, cy) as f32 / 255.0;

        // Compute group-level happiness (same formula, but once per group)
        let mut h = 50.0;
        h += group.employment_rate * 15.0;
        h += has_services * 10.0;
        h -= crime * 15.0;
        h -= pollution * 5.0;
        h += lv * 5.0;

        group.average_happiness = h.clamp(0.0, 100.0);
    }
}
// Cost: ~200-1000 iterations instead of 80,000+
```

### 15.3 Tick Staggering and Temporal Distribution

Not all systems need to run every tick. The existing codebase uses `TickCounter` and `SlowTickTimer` for this. Extend with a formal staggering framework:

```rust
/// System execution frequencies
/// At 10 Hz base tick rate:
const MOVEMENT_INTERVAL: u64 = 1;      // Every tick (10 Hz) -- must be smooth
const NEEDS_INTERVAL: u64 = 5;          // Every 0.5s -- gradual changes
const HAPPINESS_INTERVAL: u64 = 10;     // Every 1s -- existing behavior
const SCHEDULE_INTERVAL: u64 = 100;     // Every 10s -- daily schedule gen
const CRIME_INTERVAL: u64 = 50;         // Every 5s -- slow process
const HEALTH_INTERVAL: u64 = 30;        // Every 3s -- gradual changes
const IMMIGRATION_INTERVAL: u64 = 100;  // Every 10s -- existing behavior
const SEGREGATION_INTERVAL: u64 = 200;  // Every 20s -- very slow process
const EDUCATION_INTERVAL: u64 = 500;    // Every 50s -- monthly/yearly
const LIFECYCLE_INTERVAL: u64 = 3650;   // Every 365s -- annual events
const POP_REGROUP_INTERVAL: u64 = 500;  // Every 50s -- aggregate rebuild
```

Additionally, distribute citizens across tick slots to avoid spike frames:

```rust
/// Distribute citizens across N slots so only 1/N are processed per tick
/// The existing system uses entity index modulo -- formalize this:
fn citizen_tick_slot(entity: Entity, num_slots: u64, tick: u64) -> bool {
    (entity.index() as u64 % num_slots) == (tick % num_slots)
}

// Example: Update happiness for 1/10th of citizens each tick
// Instead of: 100K citizens every 10 ticks (spike)
// Do: 10K citizens every tick (smooth)
fn update_happiness_staggered(
    tick: Res<TickCounter>,
    mut citizens: Query<(&mut CitizenDetails, &HomeLocation, &BehaviorLodComp), With<Citizen>>,
    // ... resources ...
) {
    const SLOTS: u64 = 10;

    citizens.par_iter_mut().for_each(|(mut details, home, lod)| {
        // Only process citizens in the current slot
        if !citizen_tick_slot(details.entity_id, SLOTS, tick.0) {
            return;
        }
        // Skip statistical tier (handled by pop-group system)
        if lod.0 == BehaviorLOD::Statistical {
            return;
        }
        // ... happiness computation ...
    });
}
```

### 15.4 Spatial Hashing for Neighbor Queries

Many social systems require querying nearby citizens (Schelling model, disease spread, social networks). A spatial hash provides O(1) lookups:

```rust
/// Spatial hash grid for fast neighbor lookups
/// Each cell stores a list of citizen entities in that area
#[derive(Resource)]
pub struct SpatialHash {
    /// Buckets: one Vec<Entity> per spatial cell
    /// Cell size should be tuned to expected query radius
    pub cells: Vec<Vec<Entity>>,
    pub cell_size: f32,       // World units per cell (e.g., 64.0 = 4x4 grid cells)
    pub grid_width: usize,    // Number of hash cells wide
    pub grid_height: usize,
}

impl SpatialHash {
    pub fn new(world_width: f32, world_height: f32, cell_size: f32) -> Self {
        let grid_width = (world_width / cell_size).ceil() as usize;
        let grid_height = (world_height / cell_size).ceil() as usize;
        Self {
            cells: vec![Vec::new(); grid_width * grid_height],
            cell_size,
            grid_width,
            grid_height,
        }
    }

    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            cell.clear();
        }
    }

    pub fn insert(&mut self, entity: Entity, x: f32, y: f32) {
        let cx = (x / self.cell_size) as usize;
        let cy = (y / self.cell_size) as usize;
        if cx < self.grid_width && cy < self.grid_height {
            self.cells[cy * self.grid_width + cx].push(entity);
        }
    }

    /// Query all entities within radius of a point
    pub fn query_radius(&self, x: f32, y: f32, radius: f32) -> Vec<Entity> {
        let min_cx = ((x - radius) / self.cell_size).max(0.0) as usize;
        let max_cx = ((x + radius) / self.cell_size).min(self.grid_width as f32 - 1.0) as usize;
        let min_cy = ((y - radius) / self.cell_size).max(0.0) as usize;
        let max_cy = ((y + radius) / self.cell_size).min(self.grid_height as f32 - 1.0) as usize;

        let r2 = radius * radius;
        let mut result = Vec::new();
        for cy in min_cy..=max_cy {
            for cx in min_cx..=max_cx {
                for &entity in &self.cells[cy * self.grid_width + cx] {
                    // Fine-grained distance check would require position lookup
                    // For most use cases, cell-level accuracy is sufficient
                    result.push(entity);
                }
            }
        }
        result
    }
}

/// System: rebuild spatial hash each tick (only for Full LOD citizens)
fn rebuild_spatial_hash(
    mut hash: ResMut<SpatialHash>,
    citizens: Query<(Entity, &Position, &BehaviorLodComp), With<Citizen>>,
) {
    hash.clear();
    for (entity, pos, lod) in &citizens {
        if lod.0 == BehaviorLOD::Full {
            hash.insert(entity, pos.x, pos.y);
        }
    }
}
```

### 15.5 Batch Processing with Archetype Optimization

Bevy's ECS is archetype-based: entities with the same set of components are stored contiguously in memory. This means queries that access only a subset of components are cache-friendly. Design components to minimize archetype fragmentation:

```rust
// BAD: Optional components create archetype explosion
// Each combination of present/absent optional components = new archetype
// 5 optional components = 32 archetypes

// GOOD: Pack optional data into a single component with Option fields
#[derive(Component)]
pub struct SocialData {
    pub household_id: Option<Entity>,
    pub school_id: Option<Entity>,
    pub church_id: Option<Entity>,
    pub social_group: Option<u32>,
}
// One archetype for all citizens, regardless of which fields are Some

// GOOD: Use marker components sparingly for true behavioral branching
// These create only 2 archetypes (with and without marker)
#[derive(Component)]
pub struct IsHomeless;  // Marker -- only added to homeless citizens

#[derive(Component)]
pub struct IsStudent;   // Marker -- only for school-age citizens
```

### 15.6 Data-Oriented Grid Processing

For grid-based social simulation (crime, disease, segregation), process grids with SIMD-friendly patterns:

```rust
/// Process crime and disease grids together (they often share data)
/// Fused loop reduces memory traffic by reading each cell's data once
fn fused_grid_update(
    mut crime: ResMut<CrimeGrid>,
    mut disease: ResMut<DiseaseGrid>,
    population: Res<PopulationGrid>,
    coverage: Res<ServiceCoverageGrid>,
) {
    let n = crime.levels.len();

    // Process in cache-friendly order (row by row)
    // Each cell's data is read once, used for multiple computations
    for i in 0..n {
        let pop = population.density[i] as f32;
        if pop == 0.0 { continue; }

        let flags = coverage.flags[i];
        let has_police = flags & COVERAGE_POLICE != 0;
        let has_health = flags & COVERAGE_HEALTH != 0;

        // Crime update (reuses pop and police data)
        if !has_police {
            crime.levels[i] = crime.levels[i].saturating_add(1);
        }

        // Disease update (reuses pop and health data)
        let infected = disease.infected[i] as f32;
        if infected > 0.0 && !has_health {
            // Spread to neighbors (handled separately to avoid write conflicts)
        }
    }
}
```

### 15.7 Memory Layout and Cache Optimization

At 100K citizens, memory layout matters enormously:

```rust
// Component sizes (approximate):
// CitizenDetails:    32 bytes (age:1, gender:1, edu:1, padding:1, happiness:4, health:4, salary:4, savings:4, etc.)
// Position:          8 bytes (x:4, y:4)
// HomeLocation:      16 bytes (grid_x:8, grid_y:8, building:4, padding:4)
// WorkLocation:      16 bytes
// CitizenStateComp:  1 byte + padding
// Needs:             20 bytes (5 f32)
// Personality:       16 bytes (4 f32)
// BehaviorLod:       1 byte + padding
//
// Total per citizen: ~120-160 bytes
// 100K citizens: ~12-16 MB
// Fits comfortably in L3 cache of modern CPUs (32-64 MB)
//
// But: if we add the full Demographics struct from Section 1.1:
// Demographics:      ~80 bytes
// HealthProfile:     ~60 bytes (with Vec for chronic conditions)
// DailySchedule:     ~200 bytes (Vec of activities)
// HappinessAdaptation: ~100 bytes (shocks Vec)
// SocialData:        ~40 bytes
//
// Extended total: ~500-600 bytes per citizen
// 100K citizens: ~50-60 MB
// Exceeds L3 cache -- must be careful about access patterns

// Solution: Hot/Cold data split
// "Hot" data (accessed every tick): Position, State, Velocity, LOD = ~32 bytes
// "Warm" data (accessed every 10-50 ticks): Happiness, Needs, Health = ~60 bytes
// "Cold" data (accessed every 100+ ticks): Demographics, Schedule, Social = ~400 bytes
//
// Bevy naturally provides this if components are separate structs
// (different queries access different archetype columns)
```

### 15.8 Parallel Processing Patterns

The existing `par_iter_mut()` is the primary parallelism tool. Additional patterns:

```rust
/// Pattern 1: Chunk-parallel grid processing
/// Instead of parallel iteration over entities, process grid chunks independently
fn parallel_crime_update(
    crime: &mut CrimeGrid,
    // ...
) {
    let chunk_size = 32; // Process 32x32 cell chunks
    let chunks_x = (crime.width + chunk_size - 1) / chunk_size;
    let chunks_y = (crime.height + chunk_size - 1) / chunk_size;

    // Each chunk is independent (no write conflicts)
    // Use rayon or Bevy's parallel scope
    (0..chunks_x * chunks_y).into_par_iter().for_each(|chunk_idx| {
        let cx = (chunk_idx % chunks_x) * chunk_size;
        let cy = (chunk_idx / chunks_x) * chunk_size;
        // Process cells (cx..cx+chunk_size, cy..cy+chunk_size)
        // No atomic operations needed -- each chunk writes to its own cells
    });
}

/// Pattern 2: Double-buffered grids for read-write separation
/// Avoids the problem of reading neighbors while writing current cell
struct DoubleBufferedGrid {
    read: Vec<u8>,
    write: Vec<u8>,
}

impl DoubleBufferedGrid {
    fn swap(&mut self) {
        std::mem::swap(&mut self.read, &mut self.write);
    }
}

/// Pattern 3: Deferred command batching
/// Collect entity modifications, then apply in batch
/// Avoids the overhead of per-entity Commands in tight loops
fn batch_emigration(
    citizens: Query<(Entity, &CitizenDetails), With<Citizen>>,
    mut commands: Commands,
) {
    // Collect despawn candidates first (read-only pass)
    let to_despawn: Vec<Entity> = citizens.iter()
        .filter(|(_, d)| d.happiness < 10.0)
        .map(|(e, _)| e)
        .collect();

    // Batch despawn (single write pass)
    for entity in to_despawn {
        commands.entity(entity).despawn();
    }
}
```

### 15.9 Performance Budget

Target frame budget at 10 Hz (100ms per tick):

| System | Budget (ms) | Technique |
|--------|------------|-----------|
| Movement (Full LOD) | 15 | par_iter, A* capped at 64/tick |
| Movement (Simplified) | 5 | par_iter, lerp only |
| Happiness | 8 | par_iter, 1/10 citizens per tick |
| Needs decay | 3 | par_iter, every 5th tick |
| Crime grid | 2 | Every 50th tick, grid-parallel |
| Disease grid | 2 | Every 30th tick, grid-parallel |
| Health | 3 | par_iter, every 30th tick |
| Immigration | 1 | Every 100th tick |
| Schedule gen | 3 | Every 100th tick, 1/100 citizens |
| Pop-group stats | 2 | Every 50th tick |
| LOD assignment | 1 | par_iter, every 10th tick |
| Segregation | 2 | Every 200th tick, grid-parallel |
| Education | 1 | Every 500th tick |
| Lifecycle events | 1 | Every 3650th tick |
| Rendering | 30 | Fixed |
| UI | 5 | Fixed |
| **Buffer** | **16** | Headroom for spikes |
| **Total** | **100** | 10 Hz target |

The key insight is that most social systems are *slow processes* -- crime doesn't change every millisecond. By running expensive systems infrequently and distributing citizen processing across ticks, the amortized cost per tick stays well within budget even at 100K+ citizens.

---

## 16. Integration with Existing Megacity Systems

### 16.1 Mapping to Current Components

The proposed social simulation features map onto existing Megacity components:

| Proposed Feature | Existing Component | Status | Integration Path |
|-----------------|-------------------|--------|-----------------|
| Demographics | `CitizenDetails` (age, gender, education, salary, savings) | Partial | Extend struct with income_class, occupation, marital_status |
| Life stages | `LifeStage` enum (6 stages) | Complete | Add transition probabilities per stage |
| Personality | `Personality` (ambition, sociability, materialism, resilience) | Complete | Use in decision models |
| Needs | `Needs` (hunger, energy, social, fun, comfort) | Complete | Drive schedule generation |
| Family | `Family` (partner, children, parent) | Complete | Use for household formation |
| Happiness | `update_happiness` system | Complete | Add adaptation, relative deprivation |
| Wealth tiers | `WealthTier` enum + `WealthHappinessWeights` | Complete | Map to weight profiles |
| Crime | `CrimeGrid` + `update_crime` | Basic | Add crime types, broken windows |
| Immigration | `CityAttractiveness` + immigration_wave | Complete | Add push-pull utility model |
| Homelessness | `Homeless` component + 3-system pipeline | Complete | Add causation pathways |
| Health | `CitizenDetails.health` (f32) | Basic | Decompose into physical/mental |
| State machine | `CitizenState` (10 states) | Complete | Drive from generated schedules |
| Pathfinding | `PathCache` + CSR A* | Complete | LOD-aware (skip for Statistical tier) |
| Service coverage | `ServiceCoverageGrid` (bitflags) | Complete | Input to decision models |
| LOD | `LodTier` (Full/Simplified/Abstract) | Complete | Extend to behavior LOD |
| Education | Service buildings (schools, universities) | Partial | Add quality model + pipeline |
| Weather | `Weather` resource | Complete | Input to health model |
| Pollution | `PollutionGrid` | Complete | Input to health + happiness |
| Noise | `NoisePollutionGrid` | Complete | Input to mental health |
| Road condition | `RoadConditionGrid` | Complete | Input to disorder score |
| Land value | `LandValueGrid` | Complete | Feedback from school quality |
| Budget | `CityBudget` + `ExtendedBudget` | Complete | Fund services, education, police |

### 16.2 Incremental Implementation Order

The features described in this document represent a massive scope. Implement incrementally in this order, where each phase builds on the previous:

#### Phase 1: Decision Framework (2-3 weeks)
1. Extend `CitizenDetails` with income_class and occupation
2. Implement housing utility function (Section 4.1)
3. Add residential relocation when citizens are unhappy with housing
4. Implement mode choice basics (walk/drive/transit)

#### Phase 2: Happiness Depth (1-2 weeks)
1. Add hedonic adaptation (Section 5.3)
2. Add mood state thresholds (Section 5.6)
3. Implement relative deprivation using neighborhood income sampling

#### Phase 3: Crime Depth (1-2 weeks)
1. Add crime types to CrimeGrid (currently single u8)
2. Implement broken windows disorder feedback
3. Add policing strategy options

#### Phase 4: Schedule System (2-3 weeks)
1. Implement daily schedule generation (Section 3.2)
2. Integrate schedules with existing state machine
3. Add weekend/weekday differentiation
4. Use Markov perturbation for day-to-day variation

#### Phase 5: Education Pipeline (1-2 weeks)
1. Add school quality computation
2. Implement enrollment rates and dropout probabilities
3. Add school quality -> land value feedback loop

#### Phase 6: Health and Disease (1-2 weeks)
1. Decompose health into physical/mental
2. Add environmental health effects
3. Implement basic SIR disease model

#### Phase 7: Social Dynamics (2-3 weeks)
1. Implement Schelling segregation model
2. Add neighborhood composition grid
3. Implement governance: approval ratings, protests
4. Add NIMBY reactions to facility placement

#### Phase 8: Performance LOD (1-2 weeks)
1. Extend LOD system to behavior
2. Implement pop-group aggregation for Statistical tier
3. Add tick staggering framework

#### Phase 9: Polish and Emergent Dynamics (ongoing)
1. Implement social mobility tracking
2. Add election system
3. Tune all parameters for gameplay feel
4. Add homelessness causation pathways
5. Immigration demographic selectivity

### 16.3 Data Dependencies and System Ordering

The social simulation systems form a directed acyclic graph of data dependencies:

```
Weather ──────────────────────────┐
                                  |
ServiceCoverage ──┐               |
                  v               v
LandValue <── SchoolQuality   HealthUpdate
    |              |              |
    v              v              v
CrimeUpdate   EducationPipeline  DiseaseSpread
    |              |              |
    v              v              v
DisorderScore  LifecycleEvents   MortalityCheck
    |              |              |
    v              v              v
HappinessUpdate <─┘───────────────┘
    |
    v
MoodStateThresholds
    |
    ├──> EmigrationCheck
    ├──> CrimePropensity
    ├──> ProtestGeneration
    └──> ProductivityModifier
              |
              v
         EconomicOutput
              |
              v
         TaxRevenue ──> BudgetAllocation ──> ServiceFunding ──> (loop to top)
```

In Bevy, this ordering is enforced through system sets and ordering constraints:

```rust
app.configure_sets(FixedUpdate, (
    SimulationSet::Environment,     // Weather, pollution, noise
    SimulationSet::Services,        // Service coverage, school quality
    SimulationSet::LandValue,       // Land value (depends on services)
    SimulationSet::CrimeHealth,     // Crime, health, disease
    SimulationSet::Happiness,       // Happiness (depends on crime, health, services)
    SimulationSet::Behavior,        // Mood states, decisions, schedule execution
    SimulationSet::Demographics,    // Immigration, emigration, lifecycle
    SimulationSet::Economy,         // Revenue, budget, spending
    SimulationSet::Statistics,      // Pop groups, city stats, metrics
).chain());
```

### 16.4 Testing Strategy

Each social system should have unit tests for correctness and benchmark tests for performance:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_housing_utility_affordability_dominates_for_poor() {
        let cheap_far = housing_utility(
            &Dwelling { rent: 500.0, grid_x: 100, grid_y: 100, .. },
            &Household { combined_income: 2000.0, .. },
            &Demographics { income_class: IncomeClass::Poverty, .. },
            &CityData::default(),
        );
        let expensive_near = housing_utility(
            &Dwelling { rent: 1500.0, grid_x: 10, grid_y: 10, .. },
            &Household { combined_income: 2000.0, .. },
            &Demographics { income_class: IncomeClass::Poverty, .. },
            &CityData::default(),
        );
        assert!(cheap_far > expensive_near,
            "Low-income should prefer affordable distant housing");
    }

    #[test]
    fn test_gompertz_mortality_increases_with_age() {
        let young = annual_death_probability(25, 80.0);
        let middle = annual_death_probability(50, 80.0);
        let old = annual_death_probability(75, 80.0);
        assert!(young < middle);
        assert!(middle < old);
        assert!(young < 0.01, "Young healthy person should have <1% annual mortality");
        assert!(old > 0.01, "75-year-old should have >1% annual mortality");
    }

    #[test]
    fn test_hedonic_adaptation_returns_to_set_point() {
        let mut adaptation = HappinessAdaptation {
            set_point: 65.0,
            adaptation_rate: 0.05,
            adapted_level: 65.0,
            shocks: vec![],
        };
        let mut happiness = 90.0; // Sudden happiness spike
        for _ in 0..100 {
            apply_hedonic_adaptation(&mut adaptation, &mut happiness, 1);
        }
        // After 100 ticks of adaptation, should be close to set point
        assert!((happiness - 65.0).abs() < 5.0,
            "Happiness {} should adapt toward set_point 65.0", happiness);
    }

    #[test]
    fn test_schelling_threshold_produces_segregation() {
        // Simulate a small grid and verify that even low tolerance
        // (0.33) produces segregation (dissimilarity index > 0.5)
        // ... integration test ...
    }

    #[bench]
    fn bench_happiness_100k_citizens(b: &mut Bencher) {
        // Setup 100K citizens with components
        // Benchmark happiness update system
        b.iter(|| {
            // Should complete in < 10ms for 100K citizens
        });
    }
}
```

### 16.5 Configuration and Tuning

All numerical parameters should be exposed in a configuration resource for gameplay tuning:

```rust
#[derive(Resource, Serialize, Deserialize)]
pub struct SocialSimConfig {
    // Demographics
    pub founding_age_distribution: Vec<(u8, u8, f32)>,
    pub birth_rate_modifier: f32,        // 1.0 = realistic, 2.0 = fast growth
    pub death_rate_modifier: f32,
    pub aging_speed: f32,                // Game-years per real second

    // Happiness
    pub hedonic_adaptation_rate: f32,    // 0.03-0.10
    pub loss_aversion_ratio: f32,        // 1.5-2.5
    pub relative_deprivation_weight: f32,

    // Crime
    pub base_crime_multiplier: f32,
    pub broken_windows_strength: f32,
    pub displacement_rate: f32,

    // Health
    pub disease_probability_per_year: f32,
    pub pandemic_probability_per_year: f32,

    // Education
    pub school_quality_land_value_weight: f32,
    pub dropout_base_rate: f32,

    // Migration
    pub outside_world_utility: f32,
    pub migration_cost: f32,
    pub chain_migration_strength: f32,

    // Political
    pub election_cycle_days: u32,
    pub nimby_sensitivity: f32,
    pub protest_threshold: f32,

    // Performance
    pub full_lod_radius: f32,
    pub simplified_lod_radius: f32,
    pub tick_stagger_slots: u64,
    pub max_pathfind_per_tick: usize,
}
```

This configuration can be loaded from a TOML/RON file, allowing rapid iteration on game feel without recompilation.
