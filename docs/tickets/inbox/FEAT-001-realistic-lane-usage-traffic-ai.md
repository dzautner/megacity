# FEAT-001: Realistic Lane Usage and Traffic AI

**Category:** Feature / Transportation
**Priority:** T1
**Source:** community_wishlists.md -- Section 1.1 (EXTREMELY HIGH frequency)

## Summary

Vehicles should use all available lanes, not funnel into one lane. Traffic-aware routing: vehicles dynamically reroute based on congestion, not just shortest distance. Proper merging behavior at highway on-ramps. Lane mathematics for multi-lane transitions.

## Details

- Multi-lane pathfinding where vehicles distribute across lanes
- Congestion-aware routing (already partially in `csr_find_path_with_traffic`)
- Proper merge behavior at on-ramps with acceleration lanes
- Lane reduction transitions handled without gridlock
- Different vehicle speeds (trucks slower than cars)
- No-despawn option (vehicles persist in traffic, toggleable)

## Acceptance Criteria

- [ ] Vehicles distributed across all available lanes
- [ ] Route choice considers congestion
- [ ] Lane merging functions without deadlock
- [ ] Speed variation by vehicle type
