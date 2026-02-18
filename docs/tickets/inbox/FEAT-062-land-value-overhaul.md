# FEAT-062: Land Value System Overhaul

**Category:** Feature / Economy
**Priority:** T2
**Source:** master_architecture.md -- Section 1.9

## Summary

Land value as central integrating variable. Hedonic pricing: accessibility + amenities - negative externalities. Neighborhood spillover diffusion. View corridors (water/park views). Historical tracking. Land speculation near planned infrastructure. Replace u8 range with higher precision.

## Details

- Accessibility component: distance to jobs, transit, highway
- Amenity component: parks, schools, cultural facilities, views
- Negative: pollution, noise, crime, industrial adjacency
- Spillover: high-value buildings raise neighbors
- Speculation: value rises near planned infrastructure
- Land value determines building upgrade, rent, property tax

## Acceptance Criteria

- [ ] Accessibility component in land value calculation
- [ ] View corridors provide bonus
- [ ] Neighborhood spillover functional
- [ ] Land value determines property tax revenue
- [ ] Historical land value tracking
