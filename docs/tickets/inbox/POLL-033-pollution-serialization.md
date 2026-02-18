# POLL-033: Environmental Grid Save/Load Serialization

## Priority: T1 (Core)

## Description
Implement serialization for all environmental grid resources so they persist through save/load cycles. Currently some grids are serialized but many new grids (soil, UHI, flood, stormwater, energy, waste) need serialization support.

## Current State
- `PollutionGrid`, `NoisePollutionGrid`, `WaterPollutionGrid` use `u8` and may be serialized.
- `GroundwaterGrid`, `Weather`, `HeatingGrid` are serialized.
- `ForestFireGrid`, `TreeGrid` are not serialized (untracked files).
- No serialization for future grids (soil, UHI, energy, waste, flood zones).

## Definition of Done
- [ ] All environmental grids implement `Serialize`/`Deserialize`.
- [ ] Save format handles grid type changes (u8 to f32 migration).
- [ ] Backward compatibility: old saves without new grids get defaults.
- [ ] Forward compatibility: new saves include version number for migration.
- [ ] Grid compression: repeated values compressed (e.g., all-zero grids stored as flag).
- [ ] `DisasterHistory`, `WasteSystem`, `EnergyGrid` serialized.

## Test Plan
- [ ] Unit test: grid round-trips through serialize/deserialize without data loss.
- [ ] Integration test: save game with all environmental data, load, verify identical.
- [ ] Integration test: old save file loads with default values for new grids.
- [ ] Performance test: serialization of all grids completes within 100ms.

## Pitfalls
- Grid type changes (u8 to f32) break binary compatibility; need migration code.
- Many grids: serialization size may grow significantly.
- `ForestFireGrid` and `TreeGrid` are new untracked files; must be added to save.

## Code References
- `crates/save/src/serialization.rs`: existing save/load
- `crates/save/src/lib.rs`: save plugin
