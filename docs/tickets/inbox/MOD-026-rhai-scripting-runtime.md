# MOD-026: Rhai Scripting Runtime (Rust-Native)

## Priority: T5 (Stretch)
## Effort: Large (2-3 weeks)
## Source: modding_architecture.md -- Section 2.3: Rhai

## Description
Alternative scripting via Rhai (Rust-native, no FFI). Good for simple event handlers and policy tweaks. Lower overhead than Lua/WASM for simple operations. Evaluate as complement or alternative to Lua/WASM.

## Acceptance Criteria
- [ ] `rhai` crate integrated
- [ ] `RhaiModRuntime` struct with sandboxed engine
- [ ] City, building, traffic APIs registered
- [ ] Performance benchmarked vs Lua and WASM
- [ ] Decision documented: use as primary or secondary scripting
