# SAVE-013: Implement Rotating Autosave Slots

## Priority: T1 (Medium-Term)
## Effort: Small (1-2 days)
## Source: save_system_architecture.md -- Autosave Design

## Description
Add rotating autosave with configurable interval (default 5 minutes) and 3 slots. `AutosaveConfig` resource with `interval_minutes`, `slot_count`, `current_slot`, `last_save_time`.

## Acceptance Criteria
- [ ] `AutosaveConfig` resource with configurable interval (1-30 min)
- [ ] 3 rotating slots: `autosave_1.bin`, `autosave_2.bin`, `autosave_3.bin`
- [ ] Slot counter cycles after each autosave
- [ ] Player always has 2 good autosaves if latest is corrupted
- [ ] Autosave can be disabled in settings
