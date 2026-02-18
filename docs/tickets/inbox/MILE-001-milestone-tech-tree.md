# MILE-001: Milestone and Tech Tree System Overhaul
**Priority:** T2
**Complexity:** M
**Dependencies:** none
**Source:** cities_skylines_analysis.md, section 13; master_architecture.md, section 5.1

## Description
Review and improve the milestone/unlock system. Currently unlocks.rs has basic progression. Align milestones with CS1's proven structure while adding Megacity-specific unlocks.

Milestone structure:
- 0 pop: Basic roads, R/C/I zones, water, power
- 240: Healthcare, deathcare, garbage
- 1,200: Fire, police, elementary school
- 2,600: High school, parks, policies
- 5,000: Bus lines, unique buildings
- 7,500: High density zones, metro, office zones
- 12,000: University, train, cargo
- 20,000: Airport, ferry
- 36,000: Tax office, more unique buildings
- 50,000: Stock exchange, monument unlocks
- 65,000: Advanced monuments
- 80,000: All monuments, all tiles

## Definition of Done
- [ ] 12 milestones with specific unlock lists
- [ ] Unlocks gate building types, services, policies
- [ ] Milestone notification on achievement
- [ ] Progress toward next milestone visible
- [ ] Milestone panel in UI

## Test Plan
- Integration: Reach 1200 pop, verify fire station unlocked
- Integration: High density zones locked before 7500 pop

## Pitfalls
- unlocks.rs already exists with partial implementation
- Must not make early game too restricted (boring) or late game have nothing to unlock
- Each unlock should feel meaningful and change available strategies

## Relevant Code
- `crates/simulation/src/unlocks.rs` -- milestone definitions
- `crates/ui/src/toolbar.rs` -- unlock gating on tools
- `crates/ui/src/info_panel.rs` -- milestone panel
