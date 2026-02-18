# END-023: Congestion Ceiling and Transport Scaling

**Category:** Endgame / Late-Game Challenge
**Priority:** T2
**Source:** endgame_replayability.md -- Congestion Ceiling

## Summary

Implement induced demand (building more roads generates more traffic). Traffic solutions that work at 10K fail at 50K, those at 50K fail at 200K. Force mode shift at scale. Transport mode hierarchy with capacity ceilings per mode.

## Details

- Induced demand: new highway capacity fills within a few game years
- Transport mode capacity: cars 800-1600 persons/hr/m, BRT 5K-15K, metro 30K-80K
- Population-based congestion phases: intersection (10K-25K), arterial (25K-50K), network (50K-100K), mode shift imperative (100K-200K), system integration (200K-500K), demand management (500K+)
- Last-mile problem: transit works station-to-station but needs first/last mile solutions
- Congestion pricing as late-game tool

## Dependencies

- Traffic system (exists)
- Public transit system (needed)
- Economy (congestion pricing)

## Acceptance Criteria

- [ ] Induced demand modeled (new capacity fills over time)
- [ ] Traffic solutions have population-dependent effectiveness
- [ ] Mode shift required at higher populations
- [ ] Congestion pricing mechanic available
