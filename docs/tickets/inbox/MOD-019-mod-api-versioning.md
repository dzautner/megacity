# MOD-019: Mod API Versioning and Backward Compatibility

## Priority: T5 (Stretch)
## Effort: Medium (2-3 days)
## Source: modding_architecture.md -- Backward Compatibility

## Description
Define a stable API version number. Mods declare minimum API version in manifest. Game checks compatibility and rejects incompatible mods. API changes follow semver.

## Acceptance Criteria
- [ ] API version number (major.minor) defined
- [ ] Mods declare `min_api_version` in manifest
- [ ] Game rejects mods targeting newer API than supported
- [ ] Minor version bumps are backward compatible
- [ ] Major version bumps require mod updates
- [ ] Deprecation warnings for old API usage
