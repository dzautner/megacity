# SAVE-022: Delta/Incremental Save System

## Priority: T4 (Polish)
## Effort: Large (1-2 weeks)
## Source: save_system_architecture.md -- Future Architecture Recommendations

## Description
Only save changed entities since last save. Track dirty flags on components. Save delta = changed entities only + generation counter. Full save every Nth autosave as baseline.

## Acceptance Criteria
- [ ] Dirty flag tracking on entity components
- [ ] Delta save encodes only changed entities
- [ ] Full save every 5th autosave (configurable)
- [ ] Delta files resolve against baseline
- [ ] Autosave with delta takes <100ms for typical changes
