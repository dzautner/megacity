# CIT-044: Thought Stack System (Dwarf Fortress-Inspired)

**Priority:** T3 (Differentiation)
**Complexity:** Medium (2-3 person-weeks)
**Dependencies:** CIT-023 (behavioral LOD)
**Source:** social_agent_simulation.md Section 14.3 (Dwarf Fortress model)

## Description

Full LOD citizens (~500-2000 nearest to camera) track individual thoughts -- positive and negative memories with decay. Thoughts: "Nice park nearby" (+3, decay 30 days), "Long commute" (-8, persistent while true), "Got a raise" (+5, decay 60 days), "Witnessed crime" (-12, decay 90 days), "Beautiful waterfront" (+5, persistent). Overall mood = base_personality + sum(thought.value * recency_weight). Thoughts visible in citizen inspection panel, creating narrative ("unhappy because: crowded apartment -10, long commute -8, park nearby +3").

## Definition of Done

- [ ] `ThoughtStack` component (Vec<Thought>, max 20 entries)
- [ ] Thought struct: description, value, creation_day, decay_days, is_persistent
- [ ] Thoughts generated from events and conditions
- [ ] Mood computed from thought stack sum
- [ ] Thought decay over time (removed after decay_days)
- [ ] At least 30 thought types covering all happiness factors
- [ ] Citizen inspection UI shows thought list
- [ ] Only for Full LOD tier (not simplified or statistical)

## Test Plan

- Unit test: thought added and decays correctly
- Unit test: mood computed from stack sum
- Unit test: max 20 thoughts (oldest/weakest removed)
- Visual test: clicking citizen shows readable thought list

## Pitfalls

- ThoughtStack only for ~2000 citizens; must not be applied to all entities
- Thought generation must be rate-limited to avoid spam

## Relevant Code

- `crates/simulation/src/citizen.rs`
- `crates/simulation/src/happiness.rs`
- `crates/simulation/src/lod.rs`
