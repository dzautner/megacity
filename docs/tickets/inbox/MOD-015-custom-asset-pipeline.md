# MOD-015: Custom Asset Pipeline for Mod Content

## Priority: T5 (Stretch)
## Effort: Large (1-2 weeks)
## Source: modding_architecture.md -- Asset Pipeline, master_architecture.md T5

## Description
Allow mods to provide custom building models, textures, and props. Define asset format (glTF for 3D, PNG for textures), validation rules, and loading pipeline. Integrate with Bevy asset system.

## Acceptance Criteria
- [ ] Asset format specification: glTF 2.0 for models, PNG for textures
- [ ] Asset validation: polygon count limits, texture size limits, required attributes
- [ ] Mod assets loaded from `mods/{mod_id}/assets/`
- [ ] Asset registry maps mod asset IDs to Bevy handles
- [ ] LOD requirements documented for modders
