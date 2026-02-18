# SAVE-017: Serialize Family Relationships

## Priority: T1 (Medium-Term)
## Effort: Medium (2-3 days)
## Source: save_system_architecture.md -- Issue 4: Family Not Serialized

## Description
Family (partner, children) resets to default on load, destroying all family bonds. Requires entity remapping via save-local IDs. Serialize family as citizen indices (position in citizens array), then resolve to entities after all citizens are spawned.

## Acceptance Criteria
- [ ] Family relationships serialized as citizen indices
- [ ] Two-pass load: spawn citizens first, resolve family references second
- [ ] Partner and children relationships survive save/load
- [ ] Old saves load with empty family (default)
