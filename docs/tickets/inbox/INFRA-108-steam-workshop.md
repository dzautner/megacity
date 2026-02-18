# INFRA-108: Steam Workshop Integration
**Priority:** T5
**Complexity:** M (1-3 days)
**Dependencies:** INFRA-106
**Source:** master_architecture.md, M6

## Description
Integrate with Steam Workshop for mod upload, download, and management. Upload: package mod files + metadata + preview image. Download: subscribe to mods, auto-download on game start. Mod manager UI: enable/disable mods, set load order, view mod details. Workshop browsing within game.

## Definition of Done
- [ ] Upload mod to Steam Workshop
- [ ] Subscribe and download mods
- [ ] Mod manager UI (enable/disable/order)
- [ ] Workshop browsing in-game
- [ ] Mod update notifications
- [ ] Tests pass

## Test Plan
- Unit: Mod package creates valid Workshop item
- Integration: Subscribe to mod, restart game, mod is active

## Pitfalls
- Steam Workshop API requires Steamworks SDK
- Mod conflicts need user-facing resolution
- Workshop items need review/moderation pipeline

## Relevant Code
- `crates/app/src/main.rs` -- Workshop initialization
- `crates/ui/src/lib.rs` -- mod manager UI
