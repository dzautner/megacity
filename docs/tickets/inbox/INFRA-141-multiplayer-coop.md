# INFRA-141: Multiplayer Cooperative City Building
**Priority:** T5
**Complexity:** XL (1-2 weeks)
**Dependencies:** INFRA-109
**Source:** master_architecture.md, M6

## Description
Enable cooperative multiplayer where 2-4 players build cities in a shared region. Each player controls their own city. Shared road/rail connections between cities. Trade agreements. Competitive metrics (population, happiness, budget). Networking with state synchronization. Turn-based or real-time with speed negotiation.

## Definition of Done
- [ ] Networking layer (peer-to-peer or client-server)
- [ ] State synchronization for shared region
- [ ] Per-player city control
- [ ] Inter-city connections and trade
- [ ] Lobby and matchmaking
- [ ] Tests pass

## Test Plan
- Unit: Two clients see consistent shared state
- Integration: Two players build connected cities with trade

## Pitfalls
- Networking is extremely complex for simulation games
- Desync issues with deterministic simulation
- Network latency affects real-time play; consider turn-based

## Relevant Code
- New crate: `crates/networking/`
