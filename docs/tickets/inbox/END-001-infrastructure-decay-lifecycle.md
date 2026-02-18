# END-001: Infrastructure Decay and Lifecycle System

**Category:** Endgame / Late-Game Challenge
**Priority:** T3
**Source:** endgame_replayability.md -- Infrastructure Decay and Rebuild Cycles

## Summary

Implement infrastructure aging with bathtub-curve reliability. Each infrastructure element tracks age, condition (0.0-1.0), maintenance_level, capacity_remaining, and last_major_repair. Condition decay uses: `condition_loss_per_year = base_rate * usage_factor * weather_factor * (1.0 / maintenance_level)`. Below 0.7 = reduced capacity, below 0.5 = frequent disruptions, below 0.3 = critical failures.

## Details

- Roads: 15-20yr surface, 30-40yr base, bridges 50-75yr
- Water/sewer pipes: 50-100yr lifespan
- Power lines/transformers: 25-50yr
- Buildings: 20-100yr depending on type
- Track per-infrastructure-element: age, condition, maintenance_level, capacity_remaining
- Replacement wave problem: infrastructure built together ages together, creating simultaneous replacement crises
- Three repair options: Repair (cheap, partial restore), Rebuild (moderate, full reset), Upgrade (expensive, improved)
- Visual aging: roads darken, crack, show potholes over time

## Dependencies

- Economy/Budget (maintenance costs)
- Road maintenance system (already partially exists in `road_maintenance.rs`)
- Building system

## Acceptance Criteria

- [ ] Infrastructure elements track age and condition
- [ ] Condition degrades based on usage, weather, and maintenance level
- [ ] Capacity reduction at low condition thresholds
- [ ] Player can repair, rebuild, or upgrade aging infrastructure
- [ ] Visual aging visible on roads and buildings
