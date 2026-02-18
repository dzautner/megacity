# MOD-013: Mod Load Ordering and Dependency Resolution

## Priority: T5 (Stretch)
## Effort: Medium (2-3 days)
## Source: modding_architecture.md -- Mod Loading Architecture

## Description
Mods must load in dependency order. Implement topological sort of mod dependency graph. Handle conflicts, missing dependencies, and version mismatches.

## Acceptance Criteria
- [ ] Topological sort of mod dependency DAG
- [ ] Missing dependency error with helpful message
- [ ] Version constraint matching (semver compatible)
- [ ] Conflict detection when two mods modify same data
- [ ] Load order displayed in mod manager UI
