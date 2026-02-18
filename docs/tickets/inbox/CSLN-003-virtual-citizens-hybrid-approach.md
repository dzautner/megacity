# CSLN-003: Validate Virtual/Real Citizen Hybrid Model
**Priority:** T1
**Complexity:** M
**Dependencies:** SAVE-005
**Source:** cities_skylines_analysis.md, section 15.2, 16.7 (lesson 2)

## Description
CS2 tried to simulate every citizen at full fidelity and failed at scale. CS1's hybrid approach (mix of real and virtual citizens) was more sustainable. Validate that Megacity's LOD system (Full/Simplified/Abstract) correctly scales to 1M+ citizens without degradation.

- Verify LOD transitions are smooth (no visible pop-in)
- Verify virtual population statistics match entity population
- Verify aggregated behaviors (immigration, happiness, demand) work correctly with virtual pop
- Stress test: 1M virtual citizens + 10K entity citizens at 60fps
- Document LOD system behavior and limitations

## Definition of Done
- [ ] LOD system handles 1M citizens at target frame rate
- [ ] Virtual population statistics match entity averages
- [ ] No behavior discontinuities at LOD boundaries
- [ ] Stress test passes with documented results

## Test Plan
- Benchmark: 1M citizens at 60fps
- Integration: City statistics match between 10K real and 100K (with 90K virtual)

## Pitfalls
- VirtualPopulation serialization missing (SAVE-005 prerequisite)
- Virtual citizens must aggregate correctly for demand, happiness, tax calculations
- LOD transition zone may have brief statistical anomalies

## Relevant Code
- `crates/simulation/src/virtual_population.rs` -- virtual pop system
- `crates/simulation/src/lod.rs` -- LOD tier management
- `crates/simulation/src/citizen.rs` -- entity citizens
