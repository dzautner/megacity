# INFR-003: Water/Sewage Pipe Network
**Priority:** T2
**Complexity:** L
**Dependencies:** INFR-002
**Source:** cities_skylines_analysis.md, section 10; master_architecture.md, section M3

## Description
Implement explicit water and sewage pipe networks. Currently utilities propagate via BFS from any powered/watered cell. Replace with explicit pipe network that follows roads (auto-placed) with manual override for off-road routing.

- Pipes auto-placed along roads (no player action needed for basic coverage)
- Player can manually place pipes to connect off-road areas
- Pipe capacity: small (residential), medium (commercial), large (industrial)
- Pipe aging: old pipes leak (water loss %) and eventually burst
- Pipe burst: water service lost for affected buildings, repair cost
- Underground view shows pipe network
- Separate water and sewage pipe layers

## Definition of Done
- [ ] Pipe network auto-follows roads
- [ ] Manual pipe placement for off-road areas
- [ ] Pipe capacity and aging system
- [ ] Underground pipe view overlay
- [ ] Water/sewage flow through pipe network

## Test Plan
- Integration: Build road, verify pipes auto-placed
- Integration: Build off-road area, verify no water until pipes manually placed

## Pitfalls
- Auto-placement must not overwhelm player (should be invisible until problems arise)
- Pipe network is a THIRD underground layer (adds complexity)
- Pipe aging and bursts need maintenance gameplay loop

## Relevant Code
- `crates/simulation/src/utilities.rs` -- pipe network implementation
- `crates/rendering/src/overlay.rs` -- underground pipe view
- `crates/rendering/src/input.rs` -- pipe placement tool
