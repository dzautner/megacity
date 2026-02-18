# TEST-014: Integration Test: Road -> Zone -> Building Chain

## Priority: T1 (Core)
## Effort: Medium (2-3 days)
## Source: testing_strategy.md -- Section 3.2: Bevy Integration Tests

## Description
End-to-end test: place roads, zone adjacent cells, provide utilities, run 120 ticks, verify buildings spawn. Uses full Bevy App with SimulationPlugin.

## Acceptance Criteria
- [ ] `test_app()` helper creates App with MinimalPlugins + SimulationPlugin
- [ ] `run_ticks()` helper runs N app updates
- [ ] Test places roads at known coordinates
- [ ] Test zones adjacent cells with ResidentialLow
- [ ] Test sets has_power and has_water
- [ ] After 120 ticks, building_count > 0
