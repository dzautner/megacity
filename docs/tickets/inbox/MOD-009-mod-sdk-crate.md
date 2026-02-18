# MOD-009: Create Mod SDK Crate with Stable API

## Priority: T5 (Stretch)
## Effort: Large (2-3 weeks)
## Source: modding_architecture.md -- Mod SDK Architecture

## Description
Create `crates/mod-sdk` with a stable public API that does not expose Bevy internals. Provides trait-based extension points (ModPlugin, ModSystem, ModEvent) that modders implement.

## Acceptance Criteria
- [ ] `crates/mod-sdk` crate with public API
- [ ] `ModPlugin` trait for mod entry points
- [ ] Stable API types (no Bevy types exposed to modders)
- [ ] API versioning with semver
- [ ] Documentation with examples
- [ ] Backward compatibility commitment
