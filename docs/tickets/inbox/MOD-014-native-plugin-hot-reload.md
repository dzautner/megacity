# MOD-014: Hot-Reloading Native Plugins via Dynamic Library Loading

## Priority: T5 (Stretch)
## Effort: Large (2-3 weeks)
## Source: modding_architecture.md -- Native Plugin System

## Description
Support loading `.so`/`.dll`/`.dylib` native plugins. Implement hot-reload for development: watch for file changes, unload old plugin, load new version. Uses `libloading` crate.

## Acceptance Criteria
- [ ] `libloading` crate integration
- [ ] Plugin trait with `init()`, `tick()`, `shutdown()` entry points
- [ ] File watcher for hot-reload during development
- [ ] Symbol resolution and version checking
- [ ] Graceful error handling for ABI mismatches
- [ ] Security warning for native plugins
