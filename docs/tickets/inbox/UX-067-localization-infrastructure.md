# UX-067: Localization Infrastructure (String Tables)

## Priority: T4 (Polish)
## Effort: Medium (3-5 days)
## Source: master_architecture.md M5

## Description
Replace all hardcoded UI strings with localization keys. Load string tables from data files. Support multiple languages. UI layout must accommodate varying text lengths.

## Acceptance Criteria
- [ ] All UI strings replaced with localization keys
- [ ] String table format: TOML/JSON with locale codes
- [ ] At least English string table complete
- [ ] Language selector in settings
- [ ] UI layout handles long/short text gracefully
- [ ] Number and date formatting per locale
