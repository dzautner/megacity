# EDU-006: Library System and Literacy Bonus

**Priority:** T2 (Depth)
**Complexity:** Low (0.5 person-weeks)
**Dependencies:** None
**Source:** historical_demographics_services.md Section 3.4

## Description

Libraries provide education coverage bonus (+10% education quality in radius), happiness bonus (+3), and land value boost (+5). Library capacity: 50,000-200,000 visitors/year. Libraries serve as supplementary education (improve graduation rates), community gathering space (social need), and literacy improvement. Library budget affects service hours and collection quality.

## Definition of Done

- [ ] Library service building with coverage radius
- [ ] Education quality bonus (+10%) in coverage area
- [ ] Happiness bonus (+3) for residents in coverage
- [ ] Land value bonus (+5) in coverage area
- [ ] Visitor capacity tracking
- [ ] Budget affects service quality

## Test Plan

- Unit test: library improves nearby education quality
- Unit test: library provides happiness bonus
- Integration test: district with library has higher education outcomes

## Pitfalls

- Library already exists in ServiceType; ensure it has functional effects beyond coverage bit

## Relevant Code

- `crates/simulation/src/services.rs` (ServiceType::Library)
- `crates/simulation/src/education.rs`
