# INFRA-025: Underground-Specific Overlay Modes
**Priority:** T2
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-022, INFRA-019, INFRA-020
**Source:** underground_infrastructure.md, Overlay Modes section

## Description
Add new overlay modes specific to underground infrastructure: WaterPressure (heatmap blue=high, red=low), SewerCapacity (green=ok, red=overloaded), StormwaterRisk (flood risk from drainage capacity), PowerLoad (load vs substation capacity), MetroRidership (station catchment and ridership), UndergroundConflicts (cells where infrastructure overlaps at same depth). Integrate with existing `OverlayMode` enum.

## Definition of Done
- [ ] 6 new overlay mode variants added to `OverlayMode` enum
- [ ] Each overlay renders appropriate heatmap
- [ ] Overlay toggleable from UI toolbar
- [ ] Legend/scale shown for each overlay
- [ ] Tests pass

## Test Plan
- Unit: WaterPressure overlay shows lower values far from pump stations
- Integration: Overlay visually identifies problem areas

## Pitfalls
- Too many overlay modes clutters the UI; may need a submenu
- Some overlays require data that only exists when underground systems are built

## Relevant Code
- `crates/rendering/src/overlay.rs` -- `OverlayMode` enum, overlay rendering
- `crates/ui/src/toolbar.rs` -- overlay selection UI
