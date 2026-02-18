# SAVE-001: Save File Versioning and Migration
**Priority:** T1
**Complexity:** M
**Dependencies:** none
**Source:** master_architecture.md, section M2

## Description
Implement save file versioning so that new game versions can load old saves. Each save file gets a version number. Migration functions transform old format to current format.

- Version number stored in save header
- Migration chain: v1->v2->v3->... (each migration handles one version bump)
- If save version < current, apply all intermediate migrations
- If save version > current, reject with error message
- Default values for new fields added in migrations
- Test: every released save version must be loadable by current code

## Definition of Done
- [ ] Save version number in file header
- [ ] Migration functions for each version transition
- [ ] Old saves load successfully with default values for new fields
- [ ] Future-version saves rejected gracefully
- [ ] Migration tested with sample save files

## Test Plan
- Unit: v1 save migrates to v2 with correct defaults
- Integration: Load pre-migration save, verify all data preserved
- Integration: Attempt to load future version, verify error message

## Pitfalls
- Must define migration for EVERY new field added to save format
- Binary save format makes field-by-field migration harder than JSON
- Need to keep sample save files from each version for testing

## Relevant Code
- `crates/save/src/lib.rs` -- version handling
- `crates/save/src/serialization.rs` -- migration functions
