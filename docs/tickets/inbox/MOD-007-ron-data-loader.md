# MOD-007: RON Data File Loader and Validator

## Priority: T2 (Depth)
## Effort: Medium (3-4 days)
## Source: modding_architecture.md -- Data-Driven Architecture

## Description
Implement a generic data file loading system using the RON (Rusty Object Notation) crate. Validate data at load time (ranges, required fields, references). Provide clear error messages with file/line info.

## Acceptance Criteria
- [ ] `ron` crate added as dependency
- [ ] Generic `load_data_file<T: DeserializeOwned>()` function
- [ ] Validation framework: range checks, required fields, enum matching
- [ ] Error messages include file path and field name
- [ ] All data files loaded at startup before game loop begins
