# CIT-047: Social Network Model (Friends, Coworkers)

**Priority:** T3 (Differentiation)
**Complexity:** High (3-4 person-weeks)
**Dependencies:** CIT-023 (behavioral LOD)
**Source:** social_agent_simulation.md Section 10

## Description

Full LOD citizens maintain a social network of connections: neighbors (same chunk), coworkers (same workplace building), friends (leisure encounters). Network size: 5-15 connections per citizen. Social influence: connected citizens influence each other's political opinions, happiness, and behavior. Opinion diffusion: if 3/5 friends are unhappy, citizen's happiness drops (-5 sympathy penalty). Social capital: network size * connection quality affects resilience.

## Definition of Done

- [ ] `SocialNetwork` component (Vec<Entity>, max 15)
- [ ] Connections formed from: neighbors, coworkers, leisure encounters
- [ ] Connection quality (0-1, increases with interaction time)
- [ ] Opinion diffusion: friends influence happiness and politics
- [ ] Social capital metric per citizen
- [ ] Social network only for Full LOD tier
- [ ] Network visualization in citizen inspection (list of friends)

## Test Plan

- Unit test: coworkers form connections after working together
- Unit test: unhappy friends reduce citizen happiness
- Unit test: social capital correctly computed
- Integration test: opinion diffusion spreads through social networks

## Pitfalls

- O(n^2) potential connections; limit strictly to Full LOD and cap network size
- Entity references in network break on save/load; use entity remapping

## Relevant Code

- `crates/simulation/src/citizen.rs` (Family component as template)
- `crates/simulation/src/lod.rs`
