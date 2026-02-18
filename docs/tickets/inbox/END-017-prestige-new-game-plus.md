# END-017: Prestige and New Game Plus System

**Category:** Endgame / Replayability
**Priority:** T4
**Source:** endgame_replayability.md -- Prestige and New Game Plus

## Summary

After completing a sandbox city, player can "prestige" it: city is scored across 5 dimensions (Population, Economic, Quality of Life, Infrastructure, Environmental), archived, and player starts new city with bonuses + harder challenges. 5+ prestige levels with escalating difficulty and leaderboard eligibility.

## Details

- Scoring: 0-100 per dimension, total 0-500+
- Bonuses: starting budget, construction speed, starting knowledge, faction goodwill
- Challenges escalate: more disasters, stricter regulations, aggressive factions, climate acceleration
- Prestige 5+ = Legendary with random "curse" modifiers
- Global leaderboards (total prestige, single-city, fastest, by difficulty)

## Dependencies

- Core simulation (all metrics needed for scoring)
- Save system (archiving cities)

## Acceptance Criteria

- [ ] City scoring across 5 dimensions
- [ ] Prestige trigger and city archiving
- [ ] NG+ bonuses and escalating challenges
- [ ] Leaderboard integration
