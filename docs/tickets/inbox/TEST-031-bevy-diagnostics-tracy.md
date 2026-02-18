# TEST-031: Bevy Diagnostics and Tracy Integration

## Priority: T2 (Depth)
## Effort: Small (1 day)
## Source: testing_strategy.md -- Section 5.6: Bevy Diagnostics

## Description
Add FrameTimeDiagnosticsPlugin and EntityCountDiagnosticsPlugin for dev builds. Add `#[cfg(feature = "trace")]` spans to critical systems for Tracy profiling.

## Acceptance Criteria
- [ ] FrameTimeDiagnosticsPlugin added for debug builds
- [ ] EntityCountDiagnosticsPlugin added for debug builds
- [ ] `trace` feature flag defined in Cargo.toml
- [ ] info_span! added to: update_happiness, move_citizens, building_spawner, update_traffic
- [ ] Tracy integration documented in dev docs
