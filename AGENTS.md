# Megacity - AI Agent Contribution Guide

This document is for any AI agent (Claude, Codex, Gemini, Copilot, Devin, etc.) working on the Megacity codebase. Read this before writing any code.

## Repository

- **GitHub**: `dzautner/megacity`
- **Language**: Rust
- **Engine**: Bevy ECS
- **Workspace**: `crates/` with sub-crates: `simulation`, `rendering`, `save`, `ui`, `app`

## Golden Rules

1. **NEVER run `cargo build`, `cargo test`, or `cargo clippy` locally.** Let GitHub CI validate. Local builds eat CPU and are redundant.
2. **NEVER modify `lib.rs`** in simulation, rendering, or ui crates. Modules are auto-discovered via `automod_dir::dir!()`.
3. **NEVER add tests to someone else's test file.** Create a new file in `integration_tests/`.
4. **ONE plugin per line** in `plugin_registration.rs`. No tuples, no grouping.
5. **500 line hard limit** per file. Target 200-400 lines.
6. **Always run `cargo fmt --all`** before committing.
7. **ALWAYS use a git worktree.** Never work directly in the main repo checkout. See "Set Up a Worktree" below — follow the steps EXACTLY.
8. **NEVER run `git checkout` or `git switch` in the main repo.** This breaks other worktrees. Only use these inside your own worktree.
9. **SEE EVERY PR THROUGH TO MERGE.** Your job is NOT done when the code is written. You MUST monitor CI, fix failures, rebase when behind, and keep watching until `gh pr view <N> --json state` returns `MERGED`. Do NOT stop after pushing. Do NOT stop after creating the PR. Do NOT stop after CI passes. Only stop when the PR state is `MERGED`.
10. **Post to the Agent Chat Log** frequently. See "Agent Chat Log" section below.

## How to Pick Up Work

### Finding Issues

```bash
# List open issues by category
gh issue list --repo dzautner/megacity --state open --limit 50

# View a specific issue
gh issue view <NUMBER> --repo dzautner/megacity

# Filter by label or title prefix
gh issue list --repo dzautner/megacity --state open --search "AGENT-" --limit 20
```

### Issue Prefixes Explained

| Prefix | Meaning |
|--------|---------|
| `AGENT-` | Agent mode / canonical action API (high priority) |
| `CR-` | Codex review fixes |
| `STAB-` | Stability / reliability |
| `CS2-` | Cities: Skylines 2 parity features |
| `P0-` through `P3-` | Priority fixes (P0 = critical) |
| `FEAT-` | General features |
| `INFRA-` | Infrastructure / roads / terrain |
| `CIT-` | Citizen simulation |
| `ECON-` | Economy |
| `AUDIO-` | Sound / music |
| `BLDG-` | Buildings |
| `CAM-` | Camera |

### Claiming an Issue

Comment on the issue before starting:
```bash
gh issue comment <NUMBER> --repo dzautner/megacity --body "Working on this."
```

## Workflow: From Issue to Merged PR

### 1. Set Up a Worktree (MANDATORY — READ CAREFULLY)

**You MUST use a git worktree.** Do NOT work directly in the main repo checkout. Do NOT clone the repo again. Do NOT create a branch in the main checkout. Worktrees give each agent an isolated working directory so multiple agents can work simultaneously without stepping on each other.

**Step by step — follow EXACTLY:**

```bash
# Step 1: cd into the MAIN repo checkout (not a worktree, not a clone)
cd /path/to/megacity

# Step 2: Fetch latest main
git fetch origin main

# Step 3: Create the worktree. This creates a NEW directory with a NEW branch.
# The -b flag creates the branch. origin/main is the starting point.
git worktree add /tmp/worktree-<short-name> -b <agent>/issue-<NUMBER>-<short-desc> origin/main

# Step 4: cd into the worktree. ALL your work happens here.
cd /tmp/worktree-<short-name>

# Step 5: Verify you're in the right place
git branch    # Should show your new branch as active
pwd           # Should show /tmp/worktree-<short-name>
ls crates/    # Should show: app, rendering, save, simulation, ui
```

**Naming conventions:**
- Branch: `<agent>/issue-<NUMBER>-<short-desc>` (e.g., `gemini/issue-1843-grid-road-mode`, `codex/issue-1850-zone-depth`)
- Worktree directory: `/tmp/worktree-<short-name>` (e.g., `/tmp/worktree-grid-road`, `/tmp/worktree-zone-depth`)

**Common mistakes that WILL break things:**
- Working in the main repo directory instead of the worktree → conflicts with other agents
- Running `git checkout` or `git switch` in the main repo → detaches other worktrees
- Creating a worktree from another worktree → broken refs
- Forgetting `origin/main` at the end → branch starts from wrong commit
- Not cd-ing into the worktree before editing files → edits go to main repo

**After your PR is merged, clean up:**
```bash
cd /path/to/megacity    # Go back to main repo
git worktree remove /tmp/worktree-<short-name>
```

**If the worktree already exists** (from a previous failed run):
```bash
git worktree remove /tmp/worktree-<short-name> --force
# Then create it fresh
```

### 2. Understand the Architecture

```
crates/
  simulation/   # Game logic. Pure ECS. No rendering, no input, no OS calls.
    src/
      grid.rs              # WorldGrid, Cell, RoadType, ZoneType (256x256)
      economy.rs           # CityBudget, collect_taxes()
      budget.rs            # ExtendedBudget, income/expense breakdown
      road_segments.rs     # RoadSegmentStore (Bezier curves)
      road_graph_csr.rs    # CSR A* pathfinding
      movement.rs          # Citizen state machine
      sim_rng.rs           # SimRng (ChaCha8Rng, deterministic, saveable)
      simulation_sets.rs   # PreSim -> Simulation -> PostSim ordering
      plugin_registration.rs  # Central plugin registry
      integration_tests/   # Test files (auto-discovered)
      test_harness/        # TestCity builder for headless tests

  rendering/    # Visual presentation, input handling, camera
    src/
      input/               # Mouse/keyboard -> game actions
        placement.rs       # Building/utility placement
        road_drawing.rs    # Road drawing tool
        tool_handler.rs    # Tool routing

  ui/           # egui-based UI panels, toolbar, menus
    src/
      toolbar/             # Main toolbar and catalog
      info_panel/          # Budget, stats, keybinds
      main_menu.rs         # Main menu screen
      pause_menu.rs        # Pause menu

  save/         # Save/load system
    src/
      save_plugin.rs       # Save orchestration
      exclusive_load.rs    # World load system

  app/          # Binary entry point
    src/
      main.rs              # App startup, plugin registration
```

### 3. Add Your Feature

**New simulation feature:**
```
1. Create: crates/simulation/src/my_feature.rs
   - Define `pub struct MyFeaturePlugin;`
   - impl Plugin for MyFeaturePlugin { fn build(&self, app: &mut App) { ... } }

2. Register: add ONE line to crates/simulation/src/plugin_registration.rs
   app.add_plugins(MyFeaturePlugin);

3. For saveable state: implement Saveable trait, call app.register_saveable::<T>()

4. Tests: create crates/simulation/src/integration_tests/my_feature_tests.rs
   - Use TestCity::new() builder
   - Auto-discovered, no mod.rs edit needed
```

**New UI feature:**
```
1. Create: crates/ui/src/my_panel.rs
   - Define plugin, auto-discovered

2. Register in: crates/ui/src/plugin_registration.rs
```

**Key patterns to follow:**
- `RoadType::half_width()` for road widths (never hardcode)
- `SimRng` for randomness (never `thread_rng()`)
- `SimulationSet::Simulation` for main logic, `PostSim` for cleanup/stats
- `Saveable` trait + `register_saveable` for persistence

### 4. Common Clippy/Bevy Pitfalls

- Bevy system functions: max **16 parameters**. Combine `Res` params into tuples.
- `add_systems` tuples: max **~12 elements**. Split into multiple calls.
- Clippy `too_many_arguments`: max 7 args. Add `#[allow(clippy::too_many_arguments)]`.
- Clippy `derivable_impls`: use `#[derive(Default)]` instead of manual impl.
- Clippy `manual_clamp`: use `.clamp(min, max)` not `.max(min).min(max)`.
- Clippy `dead_code`: remove unused constants/fields, don't leave them.
- Clippy `type_complexity`: use type aliases for complex `Box<dyn Fn(...)>` types.

### 5. Format, Commit, Push, PR

```bash
# Format
cargo fmt --all

# Commit
git add -A
git commit -m "Short description of change

Closes #<NUMBER>

Co-Authored-By: <Your Agent Name> <noreply@anthropic.com>"

# Push
git push -u origin <branch-name>

# Create PR
gh pr create --repo dzautner/megacity \
  --title "<PREFIX>: Short title under 70 chars" \
  --body "$(cat <<'EOF'
## Summary
- Bullet points describing changes

Closes #<NUMBER>

## Test plan
- [ ] What to verify

Generated with [Your Agent Name]
EOF
)"

# Enable auto-merge
gh pr merge --auto --squash
```

### 6. Monitor CI (CRITICAL)

Branch protection requires ALL checks to pass AND branch must be up-to-date with main.

```bash
# Watch CI checks
gh pr checks <PR_NUMBER> --watch

# If a check fails, read the logs:
gh run view <RUN_ID> --log-failed

# Fix the issue, push again, re-watch
git add -A && git commit -m "Fix CI failure" && git push
gh pr checks <PR_NUMBER> --watch
```

### 7. Stay Up-to-Date with Main (CRITICAL)

Other agents' PRs merge frequently, making your branch stale. Auto-merge requires being up-to-date.

```bash
# Check merge state
gh pr view <PR_NUMBER> --json mergeStateStatus

# If BEHIND, rebase:
git fetch origin main
git rebase origin/main
git push --force-with-lease

# Then re-watch CI
gh pr checks <PR_NUMBER> --watch
```

**Repeat rebase + CI watch until the PR is merged.** This is the most common reason PRs get stuck.

### 8. Verify Merge and Clean Up (DO NOT SKIP THIS)

**Your task is NOT complete until the PR is merged.** Writing code is only half the job. You MUST stay alive and loop through steps 6-7 until the PR reaches MERGED state.

```bash
# Loop: check state, rebase if BEHIND, watch CI again
while true; do
  STATE=$(gh pr view <PR_NUMBER> --json state -q '.state')
  if [ "$STATE" = "MERGED" ]; then
    echo "PR merged!"
    break
  fi
  MERGE_STATUS=$(gh pr view <PR_NUMBER> --json mergeStateStatus -q '.mergeStateStatus')
  if [ "$MERGE_STATUS" = "BEHIND" ]; then
    git fetch origin main && git rebase origin/main && git push --force-with-lease
  fi
  gh pr checks <PR_NUMBER> --watch || true
  sleep 30
done

# Clean up worktree
cd /path/to/megacity
git worktree remove /tmp/worktree-<short-name>
```

**Common failure mode**: Agent writes code, creates PR, sees CI start, then stops. CI passes but branch is BEHIND. Auto-merge can't proceed. The PR sits there forever. **DON'T BE THAT AGENT.** Stay until MERGED.

## Parallel Agent Coordination

### How Worktrees Prevent Conflicts

Each agent works in its own worktree (a separate checkout of the repo). This means:
- Multiple agents can edit different files simultaneously
- Each agent has its own branch
- Git handles merging when PRs land on main
- No file locking, no coordination needed for non-overlapping changes

### What CAN Conflict

These files are touched by many features. Minimize changes to them:

| File | Risk | Mitigation |
|------|------|------------|
| `plugin_registration.rs` | Medium | One line per plugin, append at end |
| `grid.rs` (enums) | Medium | Add variants on separate lines |
| `toolbar/catalog/*.rs` | Medium | Add items at end of lists |
| `keybindings/bindings.rs` | Low-Medium | Add new bindings at end |

### What CANNOT Conflict

These are safe for parallel work:
- New `.rs` files (auto-discovered)
- New test files in `integration_tests/`
- New directories with `mod.rs`
- Changes within a single feature file that only you touch

### Dependency Chains

Some tickets have explicit dependencies (noted in issue body). Check before starting:
- `Depends On: #XXXX` means that ticket must merge first
- `Can Parallel With: #XXXX` means safe to work alongside
- `Blocks: #XXXX` means other tickets are waiting on yours

## Integration Testing

```rust
// crates/simulation/src/integration_tests/my_feature_tests.rs
use crate::test_harness::TestCity;

#[test]
fn test_my_feature_does_something() {
    let mut city = TestCity::new()
        .with_road_line((50, 128), (200, 128))     // place a road
        .with_zone_rect((50, 129), (100, 135), ZoneType::ResidentialLow)
        .build();

    city.tick_slow_cycles(5);  // advance simulation

    // Assert outcomes
    let budget = city.resource::<CityBudget>();
    assert!(budget.treasury > 0.0);
}
```

Key test methods:
- `TestCity::new()` — empty city builder
- `.with_road_line(start, end)` — place a road
- `.with_zone_rect(min, max, zone_type)` — zone an area
- `.build()` — finalize and return testable city
- `.tick()` — advance one simulation tick
- `.tick_slow_cycle()` — advance enough ticks for slow systems
- `.tick_slow_cycles(n)` — advance n slow cycles
- `.resource::<T>()` — read a resource
- `TestCity::with_tel_aviv()` — full pre-built map for smoke tests

## Determinism Requirements

The simulation is fully deterministic. Preserve this:
- Use `SimRng` (never `thread_rng()` or `rand::random()`)
- Use `FixedUpdate` schedule (never `Time<Real>`)
- Iterate collections in deterministic order (Vec, BTreeMap — never unsorted HashMap)
- Hash floats via `.to_bits()` (not direct comparison)

## Save System

For persistent state, implement `Saveable`:

```rust
impl Saveable for MyState {
    fn save_key() -> &'static str { "my_state" }
    fn save(&self) -> Vec<u8> { bitcode::encode(self).unwrap() }
    fn load(data: &[u8]) -> Self { bitcode::decode(data).unwrap_or_default() }
    fn reset(&mut self) { *self = Self::default(); }
}
```

Then in your plugin:
```rust
app.register_saveable::<MyState>();
```

Do NOT modify `save_types.rs`, `serialization.rs`, `save_restore.rs`, or `save_helpers.rs`.

## Agent Chat Log (IMPORTANT — USE THIS)

All agents share a communication log at **`.agent-chat.log`** in the main repo root (`/Users/danielzautner/cities/.agent-chat.log`). This is your team's informal chat channel. **Post frequently.** The human maintainer watches this file with `tail -f` to stay informed.

### How to Post

Append a line to the log. Always include a timestamp, your name, and your message:

```bash
echo "[$(date -u '+%Y-%m-%d %H:%M:%S')] <your-agent-name>: <message>" >> /Users/danielzautner/cities/.agent-chat.log
```

Example:
```bash
echo "[$(date -u '+%Y-%m-%d %H:%M:%S')] claude-agent-42: Starting work on #1843 (grid road mode)" >> /Users/danielzautner/cities/.agent-chat.log
```

### When to Post (Post Often!)

Post a message for **every significant event**. Be chatty. Think of this as a team Slack channel. Examples:

- **Starting work**: `"Starting work on #1843 — setting up worktree"`
- **Understanding the task**: `"Read the issue, looks like I need to add RoadDrawMode to grid.rs and a new input handler"`
- **Design decisions**: `"Going with an enum-based approach for road modes rather than booleans — cleaner for future extension"`
- **Progress updates**: `"Code written, running cargo fmt, about to push"`
- **PR created**: `"PR #1899 created, waiting for CI"`
- **CI status**: `"CI passed on PR #1899, auto-merge enabled"`
- **CI failures**: `"Clippy failed on PR #1899 — too_many_arguments on line 42, fixing now"`
- **Discoveries**: `"Found that road_segments.rs already has a curve_mode field — can reuse it instead of adding a new one"`
- **Warnings for others**: `"Heads up: I'm modifying plugin_registration.rs — if you're also touching it, expect a merge conflict"`
- **Merge conflicts**: `"Rebasing on main due to BEHIND status, new commits from other agents"`
- **Done**: `"PR #1899 merged! Issue #1843 closed. Cleaning up worktree."`
- **Ideas/suggestions**: `"While working on this I noticed the economy module could use a tax bracket feature — might be worth a ticket"`
- **Blockers**: `"Blocked on #1843 — depends on #1871 which hasn't merged yet"`

### How to Read

Read the last N lines to see what other agents have been up to:

```bash
tail -20 /Users/danielzautner/cities/.agent-chat.log
```

**Check the log before starting work** to see if anyone else is working on related files or might conflict with you.

### Rules

1. **Always use the absolute path** `/Users/danielzautner/cities/.agent-chat.log` — this works from any worktree
2. **Never delete or truncate** the log file — only append
3. **Keep messages on one line** — no multi-line messages
4. **Use your agent name consistently** so we can track who said what (e.g., `claude-cr02`, `gemini-grid-road`, `codex-zone-depth`)
5. **The log is .gitignored** — it stays local, never committed
6. **Post at least 5 messages per task**: start, design, progress, PR, done

## Quick Reference

| Task | Command |
|------|---------|
| List issues | `gh issue list --repo dzautner/megacity --state open` |
| View issue | `gh issue view <N> --repo dzautner/megacity` |
| Create worktree | `git worktree add /tmp/worktree-X -b branch origin/main` |
| Format code | `cargo fmt --all` |
| Create PR | `gh pr create --repo dzautner/megacity --title "..." --body "..."` |
| Auto-merge | `gh pr merge --auto --squash` |
| Watch CI | `gh pr checks <N> --watch` |
| Read CI failure | `gh run view <RUN_ID> --log-failed` |
| Check merge state | `gh pr view <N> --json mergeStateStatus` |
| Rebase on main | `git fetch origin main && git rebase origin/main && git push --force-with-lease` |
| Clean worktree | `git worktree remove /tmp/worktree-X` |
| Post to chat | `echo "[$(date -u '+%Y-%m-%d %H:%M:%S')] name: msg" >> /Users/danielzautner/cities/.agent-chat.log` |
| Read chat | `tail -20 /Users/danielzautner/cities/.agent-chat.log` |
