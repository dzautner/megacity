# DISASTER-020: Disaster Event Notification and Warning UI

## Priority: T1 (Core)

## Description
Implement a disaster notification system with visual and audio alerts. When a disaster is imminent or occurring, display a prominent warning banner, pause option, and affected area highlight. Include evacuation status for warnings with lead time.

## Current State
- Disasters trigger with a simple event and text message.
- No warning for events with lead time.
- No visual highlight of affected area.
- No evacuation mechanic.

## Definition of Done
- [ ] `DisasterStartEvent` Bevy event with disaster type, severity, epicenter.
- [ ] Warning banner UI: large, prominent, with disaster icon and severity.
- [ ] Auto-pause option: game pauses when disaster starts (configurable).
- [ ] Affected area: highlighted cells on the map showing damage zone.
- [ ] Warning lead time: for floods (12-48h), tornadoes (15-30min), tsunamis (10min-8h).
- [ ] Evacuation timer: shows countdown before impact for warned disasters.
- [ ] Camera snap: auto-zoom to disaster epicenter.
- [ ] Sound alert: different sounds per disaster type.

## Test Plan
- [ ] UI test: disaster banner appears prominently.
- [ ] UI test: camera snaps to earthquake epicenter.
- [ ] UI test: flood warning shows countdown timer.
- [ ] Integration test: warned disaster allows player to prepare.

## Pitfalls
- Auto-pause may disrupt flow for minor events.
- Sound system may not be implemented yet.
- Warning lead time for evacuation needs citizen pathfinding to shelters.

## Code References
- `crates/simulation/src/disasters.rs`: disaster events
- `crates/ui/src/lib.rs`: UI systems
- Research: `environment_climate.md` section 8.4
