# UX-008: Notification System with Priority and Navigation

## Priority: T2 (Depth)
## Effort: Medium (3-5 days)
## Source: camera_controls_ux.md -- Section 13.5: Notification System

## Description
City events generate categorized notifications. Scrolling ticker at top, color-coded by priority (red=emergency, green=positive). Clickable to jump to location. Auto-dismiss after 10-15s for low priority.

## Acceptance Criteria
- [ ] `Notification` struct: text, priority, location, timestamp
- [ ] Priority levels: Emergency, Warning, Attention, Info, Positive
- [ ] Scrolling ticker below HUD bar
- [ ] Color-coded by priority
- [ ] Click notification jumps camera to location
- [ ] Auto-dismiss timer (10-15s for low priority)
- [ ] Persist in event journal log
- [ ] Emergency notifications persist until dismissed
