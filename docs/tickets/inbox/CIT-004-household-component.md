# CIT-004: Household Component and Formation

**Priority:** T3 (Differentiation)
**Complexity:** Medium-High (3-4 person-weeks)
**Dependencies:** CIT-001 (demographics)
**Source:** social_agent_simulation.md Section 1.2

## Description

Group individual citizen agents into Household entities. Each Household has members (Vec<Entity>), head_of_household, household_type (7 types: SinglePerson 28%, CoupleNoChildren 25%, NuclearFamily 20%, SingleParent 9%, ExtendedFamily 5%, Roommates 8%, Elderly 5%), combined_income, dwelling entity, and vehicle count. Households are the economic unit for rent affordability: affordable_rent = combined_income * 0.30. When rent exceeds 30% of income, rent_burden triggers stress and eventual relocation/homelessness.

## Definition of Done

- [ ] `Household` component with all fields
- [ ] `HouseholdType` enum with 7 variants and distribution percentages
- [ ] Household formation at citizen spawn (group related citizens)
- [ ] Combined income calculation
- [ ] Rent burden calculation (rent / combined_income)
- [ ] Rent burden > 0.3 flags stress on members
- [ ] Housing demand based on household size, not individual count
- [ ] Serialization of Household relationships

## Test Plan

- Unit test: household type distribution matches target percentages
- Unit test: combined income correctly sums member incomes
- Unit test: rent burden correctly calculated
- Integration test: family of 4 seeks 2+ bedroom housing

## Pitfalls

- Entity references in Household break on save/load; need remapping
- Household dissolution on death/divorce requires careful member reassignment
- Performance: iterating households adds another query layer

## Relevant Code

- `crates/simulation/src/citizen.rs` (Family component, line 282-287)
- `crates/simulation/src/life_simulation.rs` (life_events: marriage, children)
- `crates/simulation/src/homelessness.rs` (housing loss)
