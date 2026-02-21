# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added
- Version display in the Settings panel showing build version and save format version
- CHANGELOG.md with structured format for tracking changes
- Deterministic simulation phases via SystemSet ordering (#1277)
- Level of Service (LOS) A-F traffic grading (#1272)
- BPR travel time function in pathfinding (#1271)
- Priority-based enhanced click-to-select (#1267)
- Budget breakdown panel with bars, percentages, and trends (#1266)
- Service coverage detail panel (#1265)
- Startup assertion for Saveable registration drift (#1262)
- Bulldoze refunds to prevent economic soft-lock (#1260)
- Save/load roundtrip integration tests (#1216)
- Box selection with Shift+drag (#1217)
- Tabbed building info panel (#1215)
- Two-key tool shortcuts (#1214)
- Notification system with priority and navigation (#1213)
- Minimap with terrain, roads, buildings, and camera viewport (#1212)
- Income/expense indicator in toolbar (#1211)
- Zone brush preview with multi-cell area (#1210)
- Tool cost tooltips with maintenance, coverage, and descriptions (#1206)
- Enhanced road preview with full width and intersection markers (#1205)
- Right-click context menu (#1204)
- Color-coded speed indicator visual (#1203)
- Cell tooltips on hover (#1202)
- Road parallel snapping (#1201)
- Road intersection snapping (#1200)
- Road angle snapping with 15-degree increments (#1199)
- Real-time road cost display during placement (#1197)
- Overlay legend with color ramp and value range (#1196)
- Customizable keybindings with settings UI (#1195)
- Search/filter panel for buildings and citizens (#1193)
- Enhanced network visualization for power/water overlays (#1192)
- Wind direction streamlines overlay (#1191)

### Changed
- Migrate WASM saves from localStorage to IndexedDB (#1276)
- Move x11 behind desktop target gate to slim WASM deps (#1270)
- Scale road maintenance cost by RoadType (#1237)
- Move advisor dismiss persistence to simulation plugin (#1232)
- Cache toolbar catalog and coverage metrics for performance (#1261)
- Static quality gates: zero-warning policy (#1280)
- Bench CI gates made resilient to shared-runner noise (#1268)

### Fixed
- Load/new-game entity teardown race with exclusive systems (#1269)
- Tool popup positioning to reduce map obstruction (#1264)
- Family graph dropped on save/load (#1263)
- Road snapping sub-pixel misalignment for zoning (#1259)
- Road intersection mesh not invalidated on segment removal (#1257)
- Citizen render pipeline ordering ambiguity (#1256)
- Marriage matching to enforce one-to-one pairing per tick (#1254)
- Extension-map load preserving stale state across saves (#1253)
- Job-seeking overfilling building capacity in single tick (#1252)
- Saveable key collisions with duplicate-key guard (#1233)
- HeatMitigationState not persisting across save/load (#1231)
- Keybinding conflicts resolved (#1198)
- Future save versions rejected at runtime (#1255)
