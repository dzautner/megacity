# SAVE-005: Fix VirtualPopulation Serialization
**Priority:** T0
**Complexity:** S
**Dependencies:** none
**Source:** master_architecture.md, section M2 (known issue)

## Description
VirtualPopulation is not serialized, causing population count mismatch on load. Virtual citizens represent the scaled population beyond entity count (LOD abstraction).

- Serialize VirtualPopulation resource (virtual_count, demographic distribution)
- On load, restore virtual population to saved state
- Verify total population (entity + virtual) matches pre-save count

## Definition of Done
- [x] VirtualPopulation serialized
- [x] Population count matches after load
- [x] Virtual demographic distribution preserved

## Test Plan
- Integration: Save city with 100K pop (many virtual), load, verify total pop matches

## Pitfalls
- virtual_population.rs may have fields that are hard to serialize
- Backward compatibility with saves that don't have virtual pop data

## Relevant Code
- `crates/simulation/src/virtual_population.rs` -- VirtualPopulation resource
- `crates/save/src/serialization.rs` -- add to save/load

## Implementation Notes (completed)

### Changes Made

1. **`crates/simulation/src/virtual_population.rs`**
   - Added `VirtualPopulation::from_saved()` constructor that restores all fields
     from saved data while resetting `smoothed_frame_time` to its default (0.016)
     since it is a runtime metric.

2. **`crates/save/src/serialization.rs`**
   - Added `SaveVirtualPopulation` and `SaveDistrictStats` structs (with bitcode
     Encode/Decode and serde Serialize/Deserialize)
   - Added `#[serde(default)] pub virtual_population: Option<SaveVirtualPopulation>`
     field to `SaveData` for backward compatibility
   - Added `virtual_population: Option<&VirtualPopulation>` parameter to
     `create_save_data()`
   - Added serialization of all VirtualPopulation fields: `total_virtual`,
     `virtual_employed`, `district_stats` (full demographic distribution including
     age_brackets, commuters_out, tax_contribution, service_demand), and
     `max_real_citizens`
   - Added `restore_virtual_population()` function that reconstructs the resource
     via `VirtualPopulation::from_saved()`
   - Updated test calls to include the new parameter

3. **`crates/save/src/lib.rs`**
   - Added `VirtualPopulation` to `V2ResourcesRead` and `V2ResourcesWrite`
     SystemParam bundles
   - `handle_save`: passes `Some(&v2.virtual_population)` to `create_save_data`
   - `handle_load`: restores VirtualPopulation from save data, or defaults to
     empty VirtualPopulation for old saves (backward compat)
   - `handle_new_game`: resets VirtualPopulation to default

### Backward Compatibility
- Old saves without `virtual_population` field decode successfully (via
  `#[serde(default)]`) and default to 0 virtual population
- No migration needed; population will be recalculated organically on next
  simulation tick
