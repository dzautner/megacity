# INFRA-077: Data-Driven Game Parameters (Config Files)
**Priority:** T2
**Complexity:** L (3-5 days)
**Dependencies:** none
**Source:** master_architecture.md, M3

## Description
Externalize hardcoded game parameters into data files (RON, TOML, or JSON). Building stats (capacity, cost, dimensions), road configurations (speed, capacity, cost), service parameters (radius, capacity, cost), economy parameters (tax rates, thresholds), all should be loadable from config files. This enables modding and rapid balance iteration without recompilation.

## Definition of Done
- [ ] Config file format chosen (RON recommended for Bevy ecosystem)
- [ ] Building parameters in config files
- [ ] Road parameters in config files
- [ ] Service parameters in config files
- [ ] Economy parameters in config files
- [ ] Config files hot-reloadable in dev mode
- [ ] Tests pass

## Test Plan
- Unit: Changing building capacity in config changes in-game capacity
- Unit: Missing config file falls back to compiled defaults
- Integration: Game plays correctly with default configs and modified configs

## Pitfalls
- Too many config files is hard to manage; group logically
- Config validation needed (negative capacity, etc.)
- Hot reload in production builds needs feature flag

## Relevant Code
- `crates/simulation/src/config.rs` -- current hardcoded constants
- `crates/simulation/src/buildings.rs` -- building parameters
- `crates/simulation/src/grid.rs` -- road type parameters
