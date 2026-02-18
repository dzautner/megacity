# INFRA-131: Ridged Multifractal Noise for Mountain Regions
**Priority:** T3
**Complexity:** S (hours)
**Dependencies:** INFRA-001
**Source:** procedural_terrain.md, Section 1.2 (Ridged Multifractal)

## Description
Add ridged multifractal noise blended with standard fBm for mountain regions. Use `FractalType::Ridged` from fastnoise-lite. Blend using a mountain mask: high elevation areas from continent-scale noise use ridged noise, flat areas use standard fBm. This creates sharp ridge lines and peaks in mountainous areas while keeping plains smooth.

## Definition of Done
- [ ] Ridged noise generator configured
- [ ] Mountain mask from low-frequency continent noise
- [ ] Blend: lerp(smooth_fbm, ridged, mountain_mask)
- [ ] Mountain areas have visible ridges and peaks
- [ ] Tests pass

## Test Plan
- Unit: Mountain mask selects high-elevation regions
- Unit: Ridged areas have sharper features than smooth areas
- Integration: Map has both smooth plains and ridged mountains

## Pitfalls
- Ridged noise for entire map looks unnatural; mask is essential
- Mountain mask threshold affects land/mountain ratio
- Ridge direction should look geologically plausible

## Relevant Code
- `crates/simulation/src/terrain.rs` -- noise blending
