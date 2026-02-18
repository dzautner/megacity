# SAVE-018: Implement Save-Local Entity ID Mapping

## Priority: T1 (Medium-Term)
## Effort: Medium (2-3 days)
## Source: save_system_architecture.md -- Entity Reference Remapping

## Description
Replace grid-coordinate-based entity remapping with save-local IDs. Assign sequential integers to entities during save (buildings 0..N, citizens 0..M). Build a `Vec<Entity>` mapping on load to resolve references.

## Acceptance Criteria
- [ ] Entities assigned sequential save-local IDs during serialization
- [ ] All entity references serialized as save-local IDs
- [ ] Load builds mapping table from save-local to new Entity
- [ ] Works for both building and citizen-to-citizen references
- [ ] `Entity::PLACEHOLDER` no longer used for unresolvable references
