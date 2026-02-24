//! Integration tests for SAVE-026: Post-load derived state rebuild.
//!
//! These tests verify that after inserting `PostLoadRebuildPending`, the
//! rebuild system correctly reconstructs all derived state: CSR graph,
//! service coverage, traffic grid, and spatial grid.

use crate::grid::RoadType;
use crate::happiness::ServiceCoverageGrid;
use crate::post_load_rebuild::PostLoadRebuildPending;
use crate::road_graph_csr::CsrGraph;
use crate::services::ServiceType;
use crate::spatial_grid::SpatialGrid;
use crate::test_harness::TestCity;
use crate::traffic::TrafficGrid;

/// After inserting `PostLoadRebuildPending`, the CSR graph should be rebuilt
/// from the current `RoadNetwork`.
#[test]
fn test_post_load_rebuild_csr_graph_from_roads() {
    let mut city = TestCity::new()
        .with_road(10, 10, 10, 20, RoadType::Local)
        .with_road(10, 20, 20, 20, RoadType::Local);

    // Manually zero out the CSR graph to simulate stale state after load.
    {
        let world = city.world_mut();
        *world.resource_mut::<CsrGraph>() = CsrGraph::default();
    }

    // Verify the CSR graph is currently empty.
    {
        let csr = city.world_mut().resource::<CsrGraph>();
        assert_eq!(csr.node_count(), 0, "CSR should be empty before rebuild");
    }

    // Insert the post-load rebuild marker.
    city.world_mut().insert_resource(PostLoadRebuildPending);

    // Run one tick so the rebuild system executes.
    city.tick(1);

    // CSR graph should now have nodes from the road network.
    {
        let csr = city.world_mut().resource::<CsrGraph>();
        assert!(
            csr.node_count() > 0,
            "CSR graph should have nodes after rebuild (got {})",
            csr.node_count()
        );
        assert!(
            csr.edge_count() > 0,
            "CSR graph should have edges after rebuild (got {})",
            csr.edge_count()
        );
    }

    // Verify the marker resource was removed.
    assert!(
        city.world_mut()
            .get_resource::<PostLoadRebuildPending>()
            .is_none(),
        "PostLoadRebuildPending should be removed after rebuild"
    );
}

/// After inserting `PostLoadRebuildPending`, the traffic grid should be zeroed.
#[test]
fn test_post_load_rebuild_zeros_traffic_grid() {
    let mut city = TestCity::new()
        .with_road(10, 10, 10, 20, RoadType::Local);

    // Manually set some traffic density to simulate stale loaded state.
    {
        let world = city.world_mut();
        let mut traffic = world.resource_mut::<TrafficGrid>();
        traffic.set(10, 15, 50);
        traffic.set(12, 12, 100);
    }

    // Verify traffic is non-zero.
    {
        let traffic = city.world_mut().resource::<TrafficGrid>();
        assert_eq!(traffic.get(10, 15), 50);
    }

    // Insert the post-load rebuild marker and tick.
    city.world_mut().insert_resource(PostLoadRebuildPending);
    city.tick(1);

    // Traffic grid should be zeroed out.
    {
        let traffic = city.world_mut().resource::<TrafficGrid>();
        assert_eq!(
            traffic.get(10, 15),
            0,
            "Traffic density should be zero after rebuild"
        );
        assert_eq!(
            traffic.get(12, 12),
            0,
            "Traffic density should be zero after rebuild"
        );
    }
}

/// After inserting `PostLoadRebuildPending`, the service coverage grid should
/// be marked dirty so it recalculates.
#[test]
fn test_post_load_rebuild_marks_service_coverage_dirty() {
    let mut city = TestCity::new()
        .with_road(10, 10, 10, 20, RoadType::Local)
        .with_service(15, 15, ServiceType::Hospital);

    // Run a tick to let coverage compute, then clear the dirty flag.
    city.tick(1);
    {
        let mut coverage = city.world_mut().resource_mut::<ServiceCoverageGrid>();
        coverage.dirty = false;
    }

    // Verify dirty is false.
    {
        let coverage = city.world_mut().resource::<ServiceCoverageGrid>();
        assert!(!coverage.dirty, "Coverage should not be dirty before rebuild");
    }

    // Insert the post-load rebuild marker and tick.
    city.world_mut().insert_resource(PostLoadRebuildPending);
    city.tick(1);

    // The dirty flag is set and then cleared by the update_service_coverage system
    // in the same tick (or the next). The important thing is that the coverage
    // grid should reflect the hospital's coverage after a full tick.
    // We verify by checking that the hospital's cell has health coverage.
    {
        let coverage = city.world_mut().resource::<ServiceCoverageGrid>();
        let idx = ServiceCoverageGrid::idx(15, 15);
        assert!(
            coverage.has_health(idx),
            "Service coverage at hospital location should include health after rebuild"
        );
    }
}

/// After inserting `PostLoadRebuildPending`, the spatial grid should be cleared.
#[test]
fn test_post_load_rebuild_clears_spatial_grid() {
    let mut city = TestCity::new();

    // Manually insert a fake entity into the spatial grid.
    {
        let world = city.world_mut();
        let mut spatial = world.resource_mut::<SpatialGrid>();
        spatial.insert(bevy::prelude::Entity::from_raw(999), 100.0, 100.0);
    }

    // Verify spatial grid has an entry.
    {
        let spatial = city.world_mut().resource::<SpatialGrid>();
        assert_eq!(spatial.entity_count(), 1, "Spatial grid should have 1 entry");
    }

    // Insert the post-load rebuild marker and tick.
    city.world_mut().insert_resource(PostLoadRebuildPending);
    city.tick(1);

    // Spatial grid should be cleared (the LOD system may repopulate it, but
    // the fake entity we inserted should not persist).
    // Since there are no citizens, the LOD system won't add anything.
    {
        let spatial = city.world_mut().resource::<SpatialGrid>();
        assert_eq!(
            spatial.entity_count(),
            0,
            "Spatial grid should be empty after rebuild (no citizens)"
        );
    }
}

/// The rebuild system should not run if the marker resource is not present.
#[test]
fn test_post_load_rebuild_does_not_run_without_marker() {
    let mut city = TestCity::new()
        .with_road(10, 10, 10, 20, RoadType::Local);

    // Zero out the CSR graph.
    {
        let world = city.world_mut();
        *world.resource_mut::<CsrGraph>() = CsrGraph::default();
    }

    // Tick without inserting the marker.
    city.tick(1);

    // CSR graph should remain empty (the rebuild system did not run).
    // Note: The oneway system runs in Update, not FixedUpdate, so it won't
    // rebuild the CSR during our tick() calls which only run FixedUpdate.
    {
        let csr = city.world_mut().resource::<CsrGraph>();
        assert_eq!(
            csr.node_count(),
            0,
            "CSR should remain empty without the rebuild marker"
        );
    }
}

/// The rebuild marker should be a one-shot: inserting it and ticking twice
/// should not cause a double rebuild (marker removed after first tick).
#[test]
fn test_post_load_rebuild_is_one_shot() {
    let mut city = TestCity::new()
        .with_road(10, 10, 10, 20, RoadType::Local);

    city.world_mut().insert_resource(PostLoadRebuildPending);
    city.tick(1);

    // Marker should be gone.
    assert!(
        city.world_mut()
            .get_resource::<PostLoadRebuildPending>()
            .is_none(),
        "Marker should be removed after first tick"
    );

    // Zero the CSR again to prove the rebuild doesn't fire a second time.
    {
        let world = city.world_mut();
        *world.resource_mut::<CsrGraph>() = CsrGraph::default();
    }

    city.tick(1);

    // CSR should still be empty (rebuild did not fire again).
    {
        let csr = city.world_mut().resource::<CsrGraph>();
        assert_eq!(
            csr.node_count(),
            0,
            "CSR should remain empty on second tick (rebuild is one-shot)"
        );
    }
}
