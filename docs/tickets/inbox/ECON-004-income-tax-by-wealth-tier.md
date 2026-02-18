# ECON-004: Income Tax by Wealth Tier
**Priority:** T2
**Complexity:** M
**Dependencies:** ECON-001, ECON-012
**Source:** economic_simulation.md, section 1.2; cities_skylines_analysis.md, section 3.2

## Description
Implement progressive income tax where citizens pay different rates based on their income bracket. This requires citizen wealth/income tracking (ECON-012 dependency) and provides a more realistic tax model.

- Income brackets: Low ($20K), LowerMiddle ($40K), Middle ($65K), UpperMiddle ($100K), High ($200K+)
- Tax rate per bracket independently adjustable (default: 5%, 7%, 9%, 11%, 13%)
- Income tax supplements property tax (not replaces)
- High tax on wealthy drives them to leave; low tax on wealthy attracts them but reduces revenue
- Income tax UI: slider per bracket in budget panel

## Definition of Done
- [ ] Citizens have income/wealth attribute
- [ ] Income tax calculated per citizen based on bracket
- [ ] Per-bracket tax rates adjustable by player
- [ ] Income tax revenue tracked in budget breakdown
- [ ] High tax rates reduce happiness for affected bracket

## Test Plan
- Unit: Citizen in High bracket pays more tax than Low bracket citizen
- Integration: Raise high bracket tax, verify wealthy citizens' happiness drops
- Integration: Total income tax revenue visible in budget

## Pitfalls
- Income tracking dependency on citizen wealth system
- Progressive tax creates incentive to attract wealthy -- may need balancing
- Too many tax sliders overwhelms player

## Relevant Code
- `crates/simulation/src/economy.rs:collect_taxes` -- add income tax component
- `crates/simulation/src/budget.rs` -- add income tax rate per bracket
- `crates/simulation/src/citizen.rs` -- income/wealth field
- `crates/ui/src/info_panel.rs` -- income tax sliders
