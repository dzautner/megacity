# END-019: Procedural Events from Simulation State

**Category:** Endgame / Events
**Priority:** T3
**Source:** endgame_replayability.md -- Procedural Events from Simulation State

## Summary

Replace random events with state-driven event generation. Events are consequences of conditions, not dice rolls. Pipeline: City State -> Condition Evaluator -> Event Pool -> Selection -> Presentation -> Player Choice -> Consequence. Each event has preconditions, multiple response options, and cascading effects.

## Details

- Economic events: Factory Closure Chain (unemployment > 8%), Tech Boom (GDP growth > 5%), Affordability Crisis
- Social events: Civil Unrest (Gini > 0.45 + high crime), Graying City (elderly > 25%)
- Environmental events: Toxic Discovery (cumulative pollution + residential), Water Crisis
- Political events: Development Opposition, Budget Battle, Recall Election
- Each event: 2-4 response options with different cost/benefit tradeoffs
- Event chains: one event response can trigger follow-up events
- Events reference actual city data (district names, citizen names, buildings)

## Dependencies

- All simulation systems (state evaluation)
- Events system (exists, needs overhaul)

## Acceptance Criteria

- [ ] Events triggered by specific simulation state conditions
- [ ] Each event has multiple response options
- [ ] Responses modify simulation state
- [ ] Event chains functional
- [ ] Events reference actual city data contextually
