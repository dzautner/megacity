# Performance Budgets

Megacity targets **60 FPS** (16.67ms per frame) with a simulation tick rate
of 10 Hz. The frame-time budget is split between simulation and rendering so
that neither starves the other.

## Performance Targets

| Metric | Budget | Measured By |
|---|---|---|
| Simulation tick (`FixedUpdate`) | < 5 ms | `ecs_tick/tel_aviv_fixed_update` |
| Render update (`Update` schedule) | < 5 ms | `rendering/full_update_schedule` |
| Full frame (sim + render) | < 16 ms | `sim_frame/fixed_plus_update` |

These budgets assume the Tel Aviv reference map with ~10K citizens and all
simulation systems active (weather, economy, happiness, traffic, etc.).

## Running Benchmarks Locally

### Prerequisites

Benchmarks are gated behind the `bench` Cargo feature to avoid polluting
normal builds.

### Simulation benchmarks

```bash
cargo bench -p simulation --bench city_perf --features simulation/bench
```

### Frame benchmarks (simulation + rendering)

```bash
cargo bench -p megacity --bench frame_perf --features megacity/bench
```

### All benchmarks

```bash
cargo bench --workspace --features megacity/bench
```

### Filtering

Run a single benchmark group:

```bash
cargo bench -p simulation --bench city_perf --features simulation/bench -- pathfinding
```

### Viewing reports

Criterion generates HTML reports in `target/criterion/`. Open
`target/criterion/report/index.html` in a browser to see interactive plots
with historical comparisons.

## CI Benchmark Workflow

The benchmark suite runs as a **separate GitHub Actions workflow** (not part
of the PR gate) to avoid flaky CI failures from noisy cloud runners.

### Automatic runs

Benchmarks run automatically on every push to `main`, producing a results
artifact and a summary posted to the GitHub Actions job summary.

### Manual runs

Trigger benchmarks on any branch from the GitHub Actions UI:

1. Go to **Actions > Benchmarks**
2. Click **Run workflow**
3. Select the branch and optionally set a baseline ref
4. Results appear as workflow artifacts and in the job summary

### Regression detection

The workflow reads Criterion's JSON estimates and checks each budgeted
benchmark against its target. If any benchmark exceeds its budget, the
summary includes a warning. Full Criterion HTML reports are uploaded as
artifacts for detailed analysis.

## Benchmark Inventory

### Simulation (`crates/simulation/benches/city_perf.rs`)

| Group | What it measures |
|---|---|
| `pathfinding` | A* on HashMap-based `RoadNetwork` (short/medium/long paths) |
| `csr_pathfinding` | A* on CSR graph (same distances + graph build time) |
| `nearest_road` | Linear scan vs grid-accelerated nearest-road lookup |
| `spatial_grid` | Full rebuild and rect queries at 10K/25K/50K entities |
| `grid_operations` | Full grid scan, neighbor lookups, traffic clear, radius writes |
| `happiness` | Per-citizen coverage + pollution + land-value lookups |
| `service_coverage` | Boolean grid clear + radius stamp for service buildings |
| `commute_burst` | Batch pathfinding (100/500/1000 queries per tick) |
| `road_network` | Grid and realistic road layout construction |
| `memory_footprint` | Allocation cost of WorldGrid, CSR, TrafficGrid, coverage grids |
| `full_tick_estimate` | Synthetic tick: traffic clear + spatial rebuild + happiness + paths |
| `ecs_tick` | Real Bevy `FixedUpdate` on the Tel Aviv map with all systems |

### App (`crates/app/benches/frame_perf.rs`)

| Group | What it measures |
|---|---|
| `rendering/full_update_schedule` | CPU-side `Update` schedule (all rendering systems) |
| `sim_frame/fixed_plus_update` | Combined `FixedUpdate` + `Update` (one full game frame) |

## Adding New Benchmarks

1. Add your benchmark function in the appropriate bench file
2. Register it in the `criterion_group!` macro
3. If the benchmark has a performance budget, add a check in
   `.github/workflows/bench.yml` under the regression-check case statement
4. Update the tables above
