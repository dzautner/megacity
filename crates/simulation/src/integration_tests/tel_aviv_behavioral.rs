use crate::grid::CellType;
use crate::test_harness::TestCity;

// ---------------------------------------------------------------------------
// Tel Aviv smoke test: behavioral invariants on the full map
// ---------------------------------------------------------------------------

/// Run the Tel Aviv map and verify marriage reciprocity and road/grid
/// consistency invariants hold.
#[test]
fn test_tel_aviv_behavioral_invariants_after_simulation() {
    use crate::citizen::Family;
    use std::collections::HashMap;

    let mut city = TestCity::with_tel_aviv();
    city.tick_slow_cycles(5);

    // Invariant 1: Marriage reciprocity
    {
        let world = city.world_mut();
        let mut query = world.query::<(bevy::prelude::Entity, &Family)>();
        let pairs: Vec<_> = query.iter(world).map(|(e, f)| (e, f.partner)).collect();
        let family_map: HashMap<_, _> = pairs.iter().map(|(e, p)| (*e, *p)).collect();

        for (entity, partner_opt) in &family_map {
            if let Some(partner) = partner_opt {
                let partner_partner = family_map.get(partner).and_then(|p| *p);
                assert_eq!(
                    partner_partner,
                    Some(*entity),
                    "Tel Aviv reciprocity violated: {:?} -> {:?}, but {:?} -> {:?}",
                    entity,
                    partner,
                    partner,
                    partner_partner
                );
            }
        }
    }

    // Invariant 2: Road segment / grid consistency (soft check).
    // On the Tel Aviv map, a few segment cells may overlap with water/terrain,
    // so we check >95% consistency rather than strict equality.
    {
        let grid = city.grid();
        let segments = city.road_segments();
        let mut total_cells = 0usize;
        let mut mismatch_cells = 0usize;
        for seg in &segments.segments {
            for &(cx, cy) in &seg.rasterized_cells {
                if grid.in_bounds(cx, cy) {
                    total_cells += 1;
                    if grid.get(cx, cy).cell_type != CellType::Road {
                        mismatch_cells += 1;
                    }
                }
            }
        }
        if total_cells > 0 {
            let match_rate = 1.0 - (mismatch_cells as f64 / total_cells as f64);
            assert!(
                match_rate > 0.95,
                "Tel Aviv: only {:.1}% of segment cells match grid roads ({}/{} mismatched)",
                match_rate * 100.0,
                mismatch_cells,
                total_cells,
            );
        }
    }
}
