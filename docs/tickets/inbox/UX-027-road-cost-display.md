# UX-027: Real-Time Road Cost Display During Placement

## Priority: T1 (Core Polish)
## Effort: Small (0.5 day)
## Source: camera_controls_ux.md -- Section 12.7: Road Cost Display

## Description
During road placement, show estimated cost in real-time near cursor. Calculate from segment length and cost_per_cell. Green if affordable, red if over budget.

## Acceptance Criteria
- [ ] Cost text displayed near cursor during road drawing
- [ ] Updates in real-time as cursor moves
- [ ] Green text if budget sufficient, red if insufficient
- [ ] Shows total cost for full segment
