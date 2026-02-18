# CIT-045: Mood Threshold Behavioral Breaks

**Priority:** T3 (Differentiation)
**Complexity:** Low-Medium (1 person-week)
**Dependencies:** CIT-044 (thought stack)
**Source:** social_agent_simulation.md Section 14.4 (RimWorld model)

## Description

Mood thresholds trigger visible behavioral changes. Miserable (happiness < 15): emigration, crime, protests. Unhappy (15-25): reduced productivity -20%, complaints, graffiti. Stressed (25-35): social conflicts, minor issues. Neutral (35-65): baseline. Content (65-80): +5% productivity. Happy (80-100): inspired work (+15% productivity), community volunteering, positive events. Distribution of city across thresholds displayed as bar chart.

## Definition of Done

- [ ] Mood thresholds: Miserable/Unhappy/Stressed/Neutral/Content/Happy
- [ ] Miserable: emigration probability, crime probability increase
- [ ] Unhappy: productivity penalty -20%
- [ ] Content: productivity bonus +5%
- [ ] Happy: productivity bonus +15%, positive events
- [ ] City mood distribution bar chart in stats UI
- [ ] Mood icons above citizen sprites (Full LOD only)

## Test Plan

- Unit test: happiness < 15 = Miserable threshold
- Unit test: Miserable citizen has emigration probability
- Unit test: Happy citizen has productivity bonus
- Integration test: city mood distribution visible in UI

## Pitfalls

- Threshold effects must not create death spirals (miserable -> crime -> more miserable)
- Mood icons must be subtle (not obtrusive at normal zoom)

## Relevant Code

- `crates/simulation/src/happiness.rs`
- `crates/simulation/src/lifecycle.rs` (emigration)
- `crates/rendering/src/status_icons.rs`
