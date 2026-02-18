# MOD-028: Hot-Reload Data Files During Development

## Priority: T2 (Depth)
## Effort: Small (1-2 days)
## Source: modding_architecture.md -- Data-Driven Architecture

## Description
Watch data files for changes during development. When a data file is modified, reload and apply new values without restarting the game. Speeds up balancing iteration.

## Acceptance Criteria
- [ ] File watcher on `assets/data/` directory
- [ ] Changed files auto-reloaded
- [ ] New values applied to simulation immediately
- [ ] Status message: "Reloaded buildings.ron"
- [ ] Dev-only feature (disabled in release builds)
