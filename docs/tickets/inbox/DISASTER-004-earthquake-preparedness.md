# DISASTER-004: Earthquake Preparedness Measures

## Priority: T3 (Differentiation)

## Description
Implement earthquake preparedness policies and infrastructure: seismic building codes, retrofit programs, early warning systems, emergency supplies caches, earthquake drills, flexible gas lines, and base isolation for critical facilities.

## Current State
- No preparedness measures exist.
- No early warning system.
- No building code policies for seismic safety.

## Definition of Done
- [ ] Seismic building codes policy: all new buildings seismic-designed, +20% build cost.
- [ ] Retrofit program policy: $10K/building to upgrade existing.
- [ ] Early warning system: $500K, provides 10-60 sec warning, auto gas shutoff.
- [ ] Emergency supplies cache: $200K, faster recovery, fewer casualties.
- [ ] Earthquake drills: $50K/year, -30% casualties.
- [ ] Flexible gas lines: $5K/building, -80% post-quake fire.
- [ ] Base isolation: $1M/facility, hospitals/fire stations immune below MMI IX.
- [ ] Each measure has a toggle and cost in the policy/disaster preparedness panel.

## Test Plan
- [ ] Unit test: building codes make new buildings SeismicDesigned.
- [ ] Unit test: earthquake drills reduce casualty rate by 30%.
- [ ] Integration test: city with full preparedness has much lower earthquake damage.

## Pitfalls
- Large number of policies may overwhelm the UI.
- Costs must be balanced so preparedness is worthwhile but not trivial.
- Early warning auto gas shutoff prevents post-quake fires automatically.

## Code References
- Research: `environment_climate.md` section 5.1.5
