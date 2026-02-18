# INFRA-123: Cascading Infrastructure Failures
**Priority:** T2
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-021, INFRA-019
**Source:** infrastructure_engineering.md, Cross-Cutting Themes

## Description
Model cascading failures between interdependent infrastructure systems. Power outage -> water pumps fail -> no water pressure -> no firefighting ability. Road failure -> waste collection disrupted -> public health crisis. Sewer failure -> water contamination -> disease outbreak. Each system failure can trigger downstream failures in dependent systems. Dependency chain visualization.

## Definition of Done
- [ ] Infrastructure dependency graph (power -> water -> fire, etc.)
- [ ] Failure in one system triggers cascading effects
- [ ] Power outage disables water pumps (no water in areas without gravity feed)
- [ ] Water failure reduces firefighting effectiveness
- [ ] Cascade chain shown in disaster notification
- [ ] Tests pass

## Test Plan
- Unit: Power outage in area -> water pressure drops -> fire damage increased
- Unit: Road closure -> garbage collection disrupted in affected area
- Integration: Major power plant failure creates visible cascade

## Pitfalls
- Too many cascades can feel unfair; provide warning and mitigation options
- Redundancy (backup generators, multiple water sources) should break cascades
- Performance: cascade evaluation should be efficient

## Relevant Code
- `crates/simulation/src/utilities.rs` -- power/water systems
- `crates/simulation/src/fire.rs` -- firefighting water dependency
- `crates/simulation/src/disasters.rs` -- failure events
