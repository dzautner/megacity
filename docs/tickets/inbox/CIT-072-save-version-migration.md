# CIT-072: Save File Version Migration System

**Priority:** T2 (Depth)
**Complexity:** Medium (3-4 person-weeks)
**Dependencies:** None
**Source:** master_architecture.md Section 1.17

## Description

Save files need a version number and migration functions. Current saves break when format changes. Add: save_version field to SaveData, migration function registry (v1->v2, v2->v3, etc.), chain migrations for multi-version jumps, default values for new fields, removed field handling. Each new feature that adds serialized state needs a migration entry. Test with save files from previous versions.

## Definition of Done

- [ ] Save version number in SaveData header
- [ ] Migration function registry (HashMap<(from_ver, to_ver), MigrationFn>)
- [ ] Chain migration for multi-version jumps
- [ ] Default values for new fields in migration
- [ ] Warning when loading old save (migration applied)
- [ ] Migration test suite with fixture save files
- [ ] Migration log in event journal
- [ ] Save format documentation

## Test Plan

- Unit test: v1 save migrates to v3 through v2 step
- Unit test: new field gets default value in migration
- Integration test: save from current version loads after migration code added
- Regression test: new game still produces latest version save

## Pitfalls

- Must not lose data during migration; conservative defaults
- Binary format (bitcode) makes manual inspection difficult

## Relevant Code

- `crates/save/src/serialization.rs` (SaveData)
- `crates/save/src/lib.rs` (handle_save, handle_load)
