# END-016: Roguelite Commissioner Mode

**Category:** Endgame / Replayability
**Priority:** T4
**Source:** endgame_replayability.md -- Roguelite Elements and Meta-Progression

## Summary

Against the Storm-inspired roguelite mode. Player is a "City Commissioner" taking contracts on procedurally generated sites. Each run lasts 2-6 hours with specific objectives. Earn Reputation (meta-currency) to unlock permanent Knowledge Tree upgrades.

## Details

- Procedural site generation (terrain, climate, resources, regional context)
- Primary + secondary objectives per run
- Commissioner Rank progression (Intern through Grand Commissioner)
- Knowledge Tree with 4 branches: Urban Planning, Economic, Social, Engineering
- Risk/reward modifiers before each run (harsh climate, limited resources, etc.)
- Integration with sandbox mode (knowledge unlocks shared)
- Run ends on completion, failure (bankruptcy/exodus), or abandonment

## Dependencies

- Procedural terrain generation
- Core simulation systems
- Save system (meta-progression persistence)

## Acceptance Criteria

- [ ] Roguelite mode selectable from main menu
- [ ] Procedural site generation functional
- [ ] Objectives evaluated during run
- [ ] Meta-progression persists between runs
- [ ] Knowledge Tree functional with 4 branches
