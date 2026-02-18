# TEST-028: CI Performance Regression Detection

## Priority: T2 (Depth)
## Effort: Medium (2-3 days)
## Source: testing_strategy.md -- Section 5.3: CI Regression Detection

## Description
GitHub Actions workflow that runs benchmarks nightly, compares against baseline, and alerts on >10% regression. Uses benchmark-action/github-action-benchmark.

## Acceptance Criteria
- [ ] `.github/workflows/benchmarks.yml` nightly job
- [ ] Runs `cargo bench -p simulation` with bencher output
- [ ] Compares against stored baseline
- [ ] Alerts on >10% regression
- [ ] Fails job on regression
