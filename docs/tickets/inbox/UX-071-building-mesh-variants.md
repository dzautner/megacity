# UX-071: Building Mesh Variants (2-3 Per Zone Per Level)

## Priority: T1 (Core -- M2)
## Effort: Medium (3-5 days per zone type)
## Source: master_architecture.md M2

## Description
Currently buildings within the same zone type and level look identical. Add 2-3 mesh variants per zone type per level, randomly selected on spawn.

## Acceptance Criteria
- [ ] At least 2-3 mesh variants for ResidentialLow levels 1-3
- [ ] At least 2-3 mesh variants for CommercialLow levels 1-3
- [ ] At least 2 mesh variants for Industrial
- [ ] Random variant selection on building spawn (deterministic with seeded RNG)
- [ ] Visual variety noticeable in neighborhood view
