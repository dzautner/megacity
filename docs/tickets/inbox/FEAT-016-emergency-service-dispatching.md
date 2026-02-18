# FEAT-016: Emergency Service Vehicle Dispatching

**Category:** Feature / Services
**Priority:** T2
**Source:** community_wishlists.md -- Section 4.4, 8.2 (VERY HIGH frequency)

## Summary

Fire trucks, ambulances, police cars dispatch from nearest station, not random locations. Response time based on road distance and traffic conditions. Service quality degrades when over-capacity.

## Details

- Vehicle dispatch from nearest available station
- Response time = road distance / speed * traffic factor
- Service capacity (hospital beds, fire truck count) limits simultaneous responses
- Quality degrades gracefully when over-capacity
- Mutual aid from neighboring districts when local is overwhelmed

## Acceptance Criteria

- [ ] Vehicles dispatch from nearest station
- [ ] Response time calculated from road distance + traffic
- [ ] Capacity limits enforced
- [ ] Over-capacity degradation visible
