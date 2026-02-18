# WASTE-008: Waste Reduction Policies (Plastic Ban, Deposit/Return, Composting Mandate)

## Priority: T3 (Differentiation)

## Description
Implement waste management policies that push the waste hierarchy (reduce > reuse > recycle > energy recovery > landfill). Policies include plastic bag bans, deposit/return programs, composting mandates, and WTE requirements.

## Current State
- No waste-related policies.
- No waste hierarchy concept.

## Definition of Done
- [ ] Plastic bag ban: -5% overall waste generation, minor citizen convenience impact.
- [ ] Deposit/return program: +10% recycling rate, $500K infrastructure cost.
- [ ] Composting mandate: +15% diversion to composting, happiness -2 (inconvenience), $1M enforcement.
- [ ] WTE mandate: waste diverted from landfill to WTE when available.
- [ ] Each policy toggleable in policy panel with cost/benefit summary.
- [ ] Policies reduce `WasteSystem.total_generated_tons` or redirect waste streams.

## Test Plan
- [ ] Unit test: plastic bag ban reduces waste by 5%.
- [ ] Unit test: composting mandate increases composting diversion by 15%.
- [ ] Integration test: combining multiple policies significantly reduces landfill input.

## Pitfalls
- Policy impacts on happiness need to be small enough to not deter usage.
- Composting mandate requires composting facility to exist (dependency).
- Players may not understand the waste hierarchy without UI guidance.

## Code References
- Research: `environment_climate.md` section 6.5.3
