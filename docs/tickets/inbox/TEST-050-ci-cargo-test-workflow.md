# TEST-050: CI Workflow for cargo test on Every PR

## Priority: T1 (Core)
## Effort: Small (1 day)
## Source: testing_strategy.md -- CI Integration

## Description
GitHub Actions workflow that runs `cargo test --workspace` on every pull request. Includes clippy lint, format check, and test coverage report.

## Acceptance Criteria
- [ ] `.github/workflows/ci.yml` runs on every PR
- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy --workspace` passes (no warnings)
- [ ] `cargo fmt --check` passes
- [ ] Test results reported as PR check
