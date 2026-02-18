# SAVE-020: Add Save Metadata Section

## Priority: T1 (Medium-Term)
## Effort: Small (1-2 days)
## Source: save_system_architecture.md -- Proposed File Structure

## Description
Add a `SaveMetadata` section to the file header containing quick-access information for the load screen: city name, population, treasury, game day/hour, play time, and mod list. This can be read without fully decoding the save.

## Acceptance Criteria
- [ ] `SaveMetadata` struct with city name, population, treasury, day, hour, play time
- [ ] Metadata section encoded separately in file header
- [ ] Load screen reads metadata without full decode
- [ ] Metadata displayed in save/load UI
