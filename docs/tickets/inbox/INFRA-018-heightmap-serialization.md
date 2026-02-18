# INFRA-018: Heightmap Serialization in Save Files
**Priority:** T1
**Complexity:** S (hours)
**Dependencies:** INFRA-001
**Source:** procedural_terrain.md, Section 8.4

## Description
Add terrain heightmap data to the save file format. Currently terrain may be regenerated from seed on load, but player-modified terrain needs to persist. Store the full 256x256 elevation grid in the save file. Use 16-bit quantization (u16) to reduce file size: `u16_value = (elevation * 65535.0) as u16`. This compresses the 256KB f32 grid to 128KB u16.

## Definition of Done
- [ ] Heightmap included in save file format
- [ ] 16-bit quantization for space efficiency
- [ ] Round-trip save/load preserves terrain to within quantization error
- [ ] Save version incremented with migration for old saves (regenerate from seed)
- [ ] Tests pass

## Test Plan
- Unit: Quantize/dequantize round-trip error < 0.00002
- Unit: Save/load preserves terrain elevation values
- Integration: Modified terrain survives save/load cycle

## Pitfalls
- Old save files without heightmap data need migration (regenerate from seed)
- Quantization error is 1/65535 ~= 0.000015, acceptable for gameplay but test edge cases

## Relevant Code
- `crates/save/src/serialization.rs` -- add heightmap field
- `crates/save/src/lib.rs` -- version migration
