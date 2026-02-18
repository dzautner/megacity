# INFRA-061: Transit Accessibility in Land Value
**Priority:** T2
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-037, INFRA-060
**Source:** master_architecture.md, Section 6.2

## Description
Add transit accessibility as a land value factor. Cells within walking distance of transit stops get a land value bonus proportional to service frequency. Metro station proximity provides larger bonus than bus stop. Accessibility = weighted sum of (1/headway) for all transit stops within 800m. This creates the value capture feedback loop: transit investment -> land value increase -> property tax revenue -> transit funding.

## Definition of Done
- [ ] Transit accessibility score computed per cell
- [ ] Score based on nearby stop frequency and mode (bus < BRT < rail < metro)
- [ ] Accessibility added as factor in land value computation
- [ ] Land value overlay shows transit-influenced value increases
- [ ] Tests pass

## Test Plan
- Unit: Cell 200m from metro station with 3-min headway gets high accessibility score
- Unit: Cell with no transit stops nearby gets zero accessibility bonus
- Integration: Metro station construction raises surrounding land values

## Pitfalls
- Accessibility must be recomputed when transit routes change
- Double-counting: transit reduces commute time (already a happiness factor) AND adds land value
- Capitalization rate: how much land value increase per unit of accessibility?

## Relevant Code
- `crates/simulation/src/land_value.rs` -- add accessibility factor
- `crates/simulation/src/services.rs` -- transit stop locations
