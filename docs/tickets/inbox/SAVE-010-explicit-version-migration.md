# SAVE-010: Explicit Version Numbers with Migration Chain

## Priority: T1 (Medium-Term)
## Effort: Medium (3-4 days)
## Source: save_system_architecture.md -- Versioning and Migration

## Description
Replace implicit `#[serde(default)]` versioning with explicit version numbers. Each save version has its own struct and a pure migration function. The serde defaults give a false sense of backward compatibility that bitcode does not actually provide.

## Acceptance Criteria
- [ ] File header contains monotonic version number
- [ ] `mod v1`, `mod v2` struct definitions
- [ ] Migration functions: `migrate_v1_to_v2()`, etc.
- [ ] Old saves load via migration chain
- [ ] Tests for each migration function
- [ ] Future version detection with graceful rejection
