# END-004: NIMBYism as Gameplay Mechanic

**Category:** Endgame / Late-Game Challenge
**Priority:** T3
**Source:** endgame_replayability.md -- NIMBYism as Gameplay

## Summary

When the player places certain facilities (waste processing, homeless shelters, affordable housing, transit stations, power plants), nearby property owners generate opposition. Opposition strength is proportional to property value, length of residence, and faction organization. Creates genuine moral dimension to facility placement.

## Details

- Triggered by placement of "undesirable" facilities near residential areas
- Opposition strength = f(property_value, resident_tenure, faction_strength)
- Manifests as: reduced approval, political events, construction delays, legal challenges
- Rich neighborhoods generate strong opposition, poor neighborhoods generate weak opposition
- Placing everything in poor neighborhoods is inequitable and concentrates negative effects
- Player must balance "where it's needed" vs "where it won't generate opposition"

## Dependencies

- END-003 (Political Faction System)
- Land Value system
- Service building placement

## Acceptance Criteria

- [ ] Facility placement triggers NIMBY opposition from nearby residents
- [ ] Opposition strength correlates with property values and tenure
- [ ] Opposition can delay or block construction
- [ ] Player can spend political capital to override
