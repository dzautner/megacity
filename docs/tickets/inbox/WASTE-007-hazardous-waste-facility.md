# WASTE-007: Hazardous Waste Facility and Industrial Waste

## Priority: T2 (Depth)

## Description
Implement hazardous waste tracking and a licensed hazardous waste facility. Industrial, medical, and chemical facilities generate hazardous waste requiring special treatment. Without a facility, illegal dumping causes contamination.

## Current State
- No hazardous waste tracking.
- No hazardous waste facility building.
- No illegal dumping concept.

## Definition of Done
- [ ] Hazardous waste generation by source: chemical plant=200 lbs/day, hospital=100, auto repair=20, electronics factory=50, university=30, nuclear plant=5, dry cleaner=10.
- [ ] `HazardousWasteFacility` building: 20 tons/day capacity, 2x2 footprint, $3M build, $5K/day operating.
- [ ] Required 5-cell buffer zone from residential.
- [ ] Without facility: illegal dumping causes soil + groundwater contamination.
- [ ] Illegal dumping penalty: federal fines + health crisis event.
- [ ] Treatment types: chemical, medical/biohazard, oil/solvents, heavy metals, radioactive.
- [ ] Nuclear waste from nuclear power plants requires this facility.

## Test Plan
- [ ] Unit test: hospital generates 100 lbs/day hazardous waste.
- [ ] Unit test: buffer zone enforcement prevents residential within 5 cells.
- [ ] Integration test: no hazardous facility triggers illegal dumping and contamination.
- [ ] Integration test: nuclear plant without facility triggers federal penalty.

## Pitfalls
- Many source building types don't exist yet (chemical plant, dry cleaner, etc.).
- Buffer zone enforcement requires placement validation.
- Radioactive waste from nuclear plants is a special case (POWER-004 dependency).

## Code References
- Research: `environment_climate.md` sections 6.7.1-6.7.2
