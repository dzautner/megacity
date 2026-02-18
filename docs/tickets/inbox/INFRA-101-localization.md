# INFRA-101: Localization Infrastructure
**Priority:** T4
**Complexity:** L (3-5 days)
**Dependencies:** none
**Source:** master_architecture.md, M5

## Description
Implement localization system with string tables. All user-facing text referenced by key, looked up from current locale file. Support for at least English + 2 other languages. String tables in TOML or JSON. Number/currency formatting per locale. Date formatting per locale. UI layout must handle text length variation (German text is 30% longer than English).

## Definition of Done
- [ ] String table system with locale files
- [ ] All UI text uses string keys (no hardcoded strings)
- [ ] Number/currency formatting per locale
- [ ] Language selection in settings
- [ ] At least English + 1 additional language
- [ ] UI handles text overflow gracefully
- [ ] Tests pass

## Test Plan
- Unit: String lookup returns correct translation for locale
- Unit: Missing translation falls back to English
- Integration: Switching locale changes all visible text

## Pitfalls
- Retroactively extracting all hardcoded strings is tedious
- Text expansion (German, Russian) may break UI layouts
- Right-to-left languages (Arabic, Hebrew) need additional UI work

## Relevant Code
- `crates/ui/src/` -- all UI text
- `crates/rendering/src/overlay.rs` -- overlay labels
