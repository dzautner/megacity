# END-002: Demographic Shifts and Population Dynamics

**Category:** Endgame / Late-Game Challenge
**Priority:** T3
**Source:** endgame_replayability.md -- Demographic Shifts and Population Dynamics

## Summary

Implement age-distribution tracking and dependency ratio mechanics. City passes through demographic phases: Young City (years 0-15), Family Formation (10-25), School Bulge (15-30), Retention Crisis (25-40), Aging in Place (30-50), Pension Cliff (40-60), Renewal or Decline (50+). Each phase changes service demands and budget pressures.

## Details

- Track city-wide age distribution histogram
- Dependency ratio (non-working / working-age) as key metric
- School enrollment surges and declines
- Youth retention: do young adults stay or leave?
- Aging-in-place: elderly resist change, need different services
- Pension cliff: retired city employees create massive budget pressure
- Display age projections ("in 10 years, 30% will be over 65")
- Citizens track: age, years_in_city, household_type, income_bracket

## Dependencies

- Citizen lifecycle system (exists)
- Economy/Budget (pension obligations)
- Service system (shifting demands)

## Acceptance Criteria

- [ ] Age distribution tracked and displayed
- [ ] Dependency ratio calculated and affects budget pressure
- [ ] Service demands shift with demographic composition
- [ ] Pension obligations grow as city ages
- [ ] Young adult retention mechanic functional
