# UX-073: Traffic LOS Grading with Road Color Feedback

## Priority: T1 (Core -- M2)
## Effort: Medium (2-3 days)
## Source: master_architecture.md M2

## Description
Color-code road surfaces based on traffic Level of Service (LOS A-F). Green = free flow (A), yellow = stable (C), red = congested (F). Visible in traffic overlay and optionally in normal view.

## Acceptance Criteria
- [ ] LOS grading: A (free flow) through F (gridlock)
- [ ] Road surface color changes based on LOS
- [ ] Color ramp: green -> yellow -> orange -> red
- [ ] Visible in traffic overlay mode
- [ ] Optional: subtle tinting in normal view for congested roads
