# WASTE-001: Per-Building Waste Generation Rates

## Priority: T1 (Core)

## Description
Implement per-building waste generation with rates varying by building type, income level, and building category. The research doc specifies 4.5 lbs/person/day US average with variations from 3.0 (low-income) to 6.0 (high-income) for residential, and much higher rates for commercial, industrial, hospitals, and construction.

## Current State
- No waste generation system exists.
- No waste tracking per building.
- No total waste production metric.

## Definition of Done
- [ ] `WasteProducer` component with `waste_lbs_per_day: f32`, `recycling_participation: bool`.
- [ ] Residential rates: low-income=3.0, middle=4.5, high=6.0 lbs/person/day.
- [ ] Commercial rates: small=50, large=300, restaurant=200 lbs/building/day.
- [ ] Industrial rates: light=500, heavy=2,000 lbs/building/day.
- [ ] Service rates: hospital=1,500, school=100 lbs/facility/day.
- [ ] Construction sites: 5,000 lbs/day (active), demolition=50,000 lbs one-time.
- [ ] `WasteSystem` resource with `total_generated_tons`, updated daily.
- [ ] Per-capita waste metric for dashboard display.

## Test Plan
- [ ] Unit test: residential building with 10 occupants generates 45 lbs/day.
- [ ] Unit test: hospital generates 1,500 lbs/day.
- [ ] Integration test: city of 10,000 produces ~22.5 tons/day.
- [ ] Integration test: waste total increases with city growth.

## Pitfalls
- Income level per residential building may not be tracked; could default to middle.
- Construction waste is a one-time burst that may overwhelm the system.
- Must handle buildings without `WasteProducer` component (legacy buildings).

## Code References
- `crates/simulation/src/buildings.rs`: `Building` component
- Research: `environment_climate.md` section 6.1.1
