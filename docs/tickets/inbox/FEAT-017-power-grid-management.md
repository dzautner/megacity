# FEAT-017: Power Grid Demand/Supply Management

**Category:** Feature / Infrastructure
**Priority:** T2
**Source:** community_wishlists.md -- Section 5.2, master_architecture.md

## Summary

Energy demand per building, time-of-day demand curves, seasonal variation. Generation types (coal, gas, nuclear, solar, wind, hydro, geothermal). Grid balancing: supply must meet demand or brownouts/blackouts. Renewable intermittency with battery storage. Cascading failure effects.

## Details

- Per-building demand profiles (residential peak evening, commercial peak day)
- Generation types with capacity, fuel cost, pollution, reliability
- Grid balance: demand > supply = brownout/blackout
- Blackout cascading: traffic lights fail, hospitals on backup, citizen panic
- Energy storage for renewable intermittency
- Smart grid features for late-game tech

## Acceptance Criteria

- [ ] Per-building energy demand calculated
- [ ] Supply/demand balance tracked
- [ ] Blackouts when supply < demand
- [ ] Multiple generation types with tradeoffs
- [ ] Battery storage for renewables
