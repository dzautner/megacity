# INFRA-063: Save File Versioning with Migration System
**Priority:** T0
**Complexity:** M (1-3 days)
**Dependencies:** none
**Source:** master_architecture.md, M2

## Description
Add version numbers to save files and implement a migration system that upgrades old save formats to current. Each save file includes a version integer. On load, if version < current, run migration functions in sequence (v1->v2, v2->v3, etc.). Migrations can add new fields with defaults, rename fields, or restructure data. This ensures saves survive code updates.

## Definition of Done
- [ ] Save files include version number
- [ ] `migrate_save(data, from_version, to_version)` function
- [ ] Chained migration from any old version to current
- [ ] Migration for each new field added to save format
- [ ] Old save files load correctly after migration
- [ ] Tests pass

## Test Plan
- Unit: V1 save migrates to V3 with correct defaults for new fields
- Unit: Current version save loads without migration
- Integration: Old save file from before versioning loads (version 0 -> current)

## Pitfalls
- Must define version 0 for existing unversioned saves
- Migration must handle missing fields gracefully (Option<T> or defaults)
- Binary format changes need careful offset handling

## Relevant Code
- `crates/save/src/lib.rs` -- save/load entry points
- `crates/save/src/serialization.rs` -- format definition
