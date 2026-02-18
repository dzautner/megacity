# AUDIO-016: Day/Night Cycle Audio

**Category:** Audio / Environmental
**Priority:** T4
**Source:** sound_design.md -- Section 4.4

## Summary

24-hour ambient volume envelope driven by GameClock. Dawn chorus (5-7am, seasonal intensity), daytime activity (sirens, aircraft, bells), evening wind-down (crickets begin, traffic fades), night sounds (owl, distant dogs, insects, near-silence). Ambient volume peaks at midday (0.9), bottoms at 2-3am (0.1).

## Details

- Dawn chorus bell curve centered at 5:45 AM, strongest in spring, absent in winter
- Daytime: distant sirens (scales with crime), aircraft flyovers (if airport), hourly bells (if civic building)
- Evening: crickets (summer/autumn 8pm+), domestic sounds
- Night: owl hooting (random 120-600s), distant dog bark (300-900s), wind more noticeable
- Overall 24-hour volume envelope applied to ambience bus

## Dependencies

- AUDIO-001
- GameClock, Weather

## Acceptance Criteria

- [ ] 24-hour volume envelope functional
- [ ] Dawn chorus audible in spring/summer
- [ ] Night sounds distinct from day
- [ ] Time passage conveyed through audio
