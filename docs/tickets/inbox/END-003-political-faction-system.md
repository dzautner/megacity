# END-003: Political Faction System

**Category:** Endgame / Late-Game Challenge
**Priority:** T3
**Source:** endgame_replayability.md -- Political Complexity and Faction Systems

## Summary

Implement interest-based political factions that form organically from city demographics: Homeowner Coalition, Renter/Affordable Housing Alliance, Business/Commercial Interests, Environmental Coalition, Labor/Workers Alliance, Development/Growth Coalition. Factions oppose/support player actions, form alliances, and generate political events.

## Details

- 6 factions with distinct interests, opposition triggers, and support triggers
- Faction strength grows based on city conditions (e.g., Homeowners grow as city ages)
- Alliance formation: Homeowners+Environmentalists vs Developers
- Political events: community opposition, budget battles, recall elections, scandals, strikes, protests
- Each faction has a satisfaction meter affecting city governance
- Political capital as abstract resource (earned by popular actions, spent to override opposition)

## Dependencies

- Citizen system (demographics drive faction membership)
- Economy/Budget (budget battles)
- District system (faction strength varies by district)

## Acceptance Criteria

- [ ] 6 factions implemented with satisfaction tracking
- [ ] Faction strength scales with relevant city conditions
- [ ] Political events generated from faction interactions
- [ ] Political capital resource functional
- [ ] Factions can block or delay player projects
