# MOD-012: Mod Loader with Manifest System

## Priority: T5 (Stretch)
## Effort: Medium (3-5 days)
## Source: modding_architecture.md -- Mod Loading Architecture

## Description
Implement mod discovery and loading. Scan `mods/` directory, read `mod.toml` manifests (id, name, version, author, dependencies, mod_type), validate, and load in dependency order.

## Acceptance Criteria
- [ ] `mods/` directory scanned at startup
- [ ] `mod.toml` manifest format defined and documented
- [ ] Dependency resolution with topological sort
- [ ] Circular dependency detection with error
- [ ] Version compatibility checking
- [ ] Mod type classification: data-only, script, native
