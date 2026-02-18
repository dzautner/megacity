# FEAT-035: Larger Buildable Maps

**Category:** Feature / Terrain
**Priority:** T2
**Source:** community_wishlists.md -- Section 14.2 (EXTREMELY HIGH frequency)

## Summary

Expand beyond 256x256 grid. Support for 512x512 or larger with streaming/chunked loading. Terrain sculpting during gameplay. Waterfront development (quays, boardwalks, marinas). Island/archipelago map types.

## Details

- Current: 256x256 (CELL_SIZE=16, CHUNK_SIZE=8)
- Target: at least 512x512 or configurable map sizes
- Chunked loading to maintain performance
- Terrain modification tools (raise, lower, flatten, create water)
- Multiple landmass support with bridges/ferries

## Acceptance Criteria

- [ ] Map sizes larger than 256x256 supported
- [ ] Chunked loading maintains performance
- [ ] Terrain sculpting tools available
- [ ] Island/water maps functional
