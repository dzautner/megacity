# META-001: Data-Driven Architecture (Extract Hardcoded Values)

**Category:** Meta / Architecture
**Priority:** T2
**Source:** master_architecture.md -- Section 1.16

## Summary

Extract all hardcoded game parameters into external data files (RON/JSON/TOML). Building stats, road parameters, service radii, policy effects, zone settings, economy constants. Foundation for modding. Override hierarchy: base game -> mod -> user.

## Details

- Currently all parameters are hardcoded in Rust source
- Extract to data files loadable at runtime
- Building stats: cost, size, capacity, effects
- Service radii per service type
- Economy constants: tax rates, growth factors, costs
- Zone settings: density limits, building heights
- Road parameters: speeds, capacities, costs

## Acceptance Criteria

- [ ] Building stats in external data files
- [ ] Service parameters in data files
- [ ] Economy constants configurable
- [ ] Data files loadable at runtime
- [ ] Override hierarchy functional
