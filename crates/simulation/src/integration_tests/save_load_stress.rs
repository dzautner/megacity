use crate::grid::{RoadType, ZoneType};
use crate::services::ServiceType;
use crate::test_harness::TestCity;
use crate::utilities::UtilityType;

// ===========================================================================
// TEST-061: Stress Test: Rapid Save/Load Cycles (Issue #841)
// ===========================================================================

/// Rapidly save and load the SaveableRegistry 100 times.
/// Verifies no state corruption or resource leaks across cycles.
#[test]
fn test_rapid_save_load_100_cycles_saveable_registry() {
    use crate::SaveableRegistry;
    use std::collections::BTreeMap;

    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.init_resource::<SaveableRegistry>();

    #[derive(bevy::prelude::Resource, Default, Clone, Debug, PartialEq)]
    struct CounterResource {
        value: u64,
    }

    impl crate::Saveable for CounterResource {
        const SAVE_KEY: &'static str = "stress_counter";
        fn save_to_bytes(&self) -> Option<Vec<u8>> {
            serde_json::to_vec(&self.value).ok()
        }
        fn load_from_bytes(bytes: &[u8]) -> Self {
            Self {
                value: serde_json::from_slice(bytes).unwrap_or_default(),
            }
        }
    }

    #[derive(bevy::prelude::Resource, Default, Clone, Debug, PartialEq)]
    struct VecResource {
        items: Vec<String>,
    }

    impl crate::Saveable for VecResource {
        const SAVE_KEY: &'static str = "stress_vec";
        fn save_to_bytes(&self) -> Option<Vec<u8>> {
            if self.items.is_empty() {
                return None;
            }
            serde_json::to_vec(&self.items).ok()
        }
        fn load_from_bytes(bytes: &[u8]) -> Self {
            Self {
                items: serde_json::from_slice(bytes).unwrap_or_default(),
            }
        }
    }

    #[derive(bevy::prelude::Resource, Default, Clone, Debug, PartialEq)]
    struct NestedResource {
        mapping: BTreeMap<String, Vec<f64>>,
    }

    impl crate::Saveable for NestedResource {
        const SAVE_KEY: &'static str = "stress_nested";
        fn save_to_bytes(&self) -> Option<Vec<u8>> {
            if self.mapping.is_empty() {
                return None;
            }
            serde_json::to_vec(&self.mapping).ok()
        }
        fn load_from_bytes(bytes: &[u8]) -> Self {
            Self {
                mapping: serde_json::from_slice(bytes).unwrap_or_default(),
            }
        }
    }

    app.init_resource::<CounterResource>();
    app.init_resource::<VecResource>();
    app.init_resource::<NestedResource>();
    {
        let mut registry = app.world_mut().resource_mut::<SaveableRegistry>();
        registry.register::<CounterResource>();
        registry.register::<VecResource>();
        registry.register::<NestedResource>();
    }

    app.world_mut().resource_mut::<CounterResource>().value = 123_456_789;
    app.world_mut().resource_mut::<VecResource>().items =
        (0..50).map(|i| format!("item_{i}")).collect();
    {
        let mut nested = app.world_mut().resource_mut::<NestedResource>();
        for i in 0..10 {
            nested.mapping.insert(
                format!("key_{i}"),
                (0..20).map(|j| (i * 20 + j) as f64 * 0.1).collect(),
            );
        }
    }

    for cycle in 0..100 {
        let extensions = {
            let registry = app.world().resource::<SaveableRegistry>();
            registry.save_all(app.world())
        };
        assert!(
            extensions.contains_key("stress_counter"),
            "cycle {cycle}: counter missing"
        );
        assert!(
            extensions.contains_key("stress_vec"),
            "cycle {cycle}: vec missing"
        );
        assert!(
            extensions.contains_key("stress_nested"),
            "cycle {cycle}: nested missing"
        );

        app.world_mut().insert_resource(CounterResource::default());
        app.world_mut().insert_resource(VecResource::default());
        app.world_mut().insert_resource(NestedResource::default());

        {
            let registry = app
                .world_mut()
                .remove_resource::<SaveableRegistry>()
                .unwrap();
            registry.load_all(app.world_mut(), &extensions);
            app.world_mut().insert_resource(registry);
        }

        assert_eq!(
            app.world().resource::<CounterResource>().value,
            123_456_789,
            "cycle {cycle}: counter corrupted"
        );
        assert_eq!(
            app.world().resource::<VecResource>().items.len(),
            50,
            "cycle {cycle}: vec len changed"
        );
        assert_eq!(
            app.world().resource::<VecResource>().items[0],
            "item_0",
            "cycle {cycle}: first item corrupted"
        );
        assert_eq!(
            app.world().resource::<VecResource>().items[49],
            "item_49",
            "cycle {cycle}: last item corrupted"
        );
        assert_eq!(
            app.world().resource::<NestedResource>().mapping.len(),
            10,
            "cycle {cycle}: nested map size changed"
        );
        let key5 = app
            .world()
            .resource::<NestedResource>()
            .mapping
            .get("key_5")
            .expect("key_5")
            .clone();
        assert_eq!(key5.len(), 20, "cycle {cycle}: nested vec len changed");
        assert!(
            (key5[0] - 10.0).abs() < 1e-10,
            "cycle {cycle}: nested float corrupted"
        );
    }
}

/// Rapidly serialize/deserialize citizen data 100 times via serde_json.
#[test]
fn test_rapid_citizen_serde_100_roundtrips() {
    use crate::citizen::{
        CitizenDetails, Gender, Needs, PathCache, Personality, Position, Velocity,
    };
    use crate::roads::RoadNode;

    let mut details_json = serde_json::to_string(&CitizenDetails {
        age: 42,
        gender: Gender::Female,
        education: 3,
        happiness: 72.5,
        health: 88.3,
        salary: 6500.0,
        savings: 15000.0,
    })
    .unwrap();
    let mut pers_json = serde_json::to_string(&Personality {
        ambition: 0.85,
        sociability: 0.3,
        materialism: 0.65,
        resilience: 0.92,
    })
    .unwrap();
    let mut needs_json = serde_json::to_string(&Needs {
        hunger: 55.0,
        energy: 70.0,
        social: 42.0,
        fun: 38.0,
        comfort: 65.0,
    })
    .unwrap();
    let mut pos_json = serde_json::to_string(&Position { x: 200.5, y: 180.3 }).unwrap();
    let mut vel_json = serde_json::to_string(&Velocity { x: 1.5, y: -0.7 }).unwrap();
    let mut path_json = serde_json::to_string(&PathCache::new(vec![
        RoadNode(12, 10),
        RoadNode(15, 10),
        RoadNode(18, 10),
    ]))
    .unwrap();

    for cycle in 0..100 {
        let d: CitizenDetails =
            serde_json::from_str(&details_json).unwrap_or_else(|e| panic!("cycle {cycle}: {e}"));
        let p: Personality =
            serde_json::from_str(&pers_json).unwrap_or_else(|e| panic!("cycle {cycle}: {e}"));
        let n: Needs =
            serde_json::from_str(&needs_json).unwrap_or_else(|e| panic!("cycle {cycle}: {e}"));
        let po: Position =
            serde_json::from_str(&pos_json).unwrap_or_else(|e| panic!("cycle {cycle}: {e}"));
        let v: Velocity =
            serde_json::from_str(&vel_json).unwrap_or_else(|e| panic!("cycle {cycle}: {e}"));
        let pa: PathCache =
            serde_json::from_str(&path_json).unwrap_or_else(|e| panic!("cycle {cycle}: {e}"));
        details_json = serde_json::to_string(&d).unwrap();
        pers_json = serde_json::to_string(&p).unwrap();
        needs_json = serde_json::to_string(&n).unwrap();
        pos_json = serde_json::to_string(&po).unwrap();
        vel_json = serde_json::to_string(&v).unwrap();
        path_json = serde_json::to_string(&pa).unwrap();
    }

    let fd: CitizenDetails = serde_json::from_str(&details_json).unwrap();
    let fp: Personality = serde_json::from_str(&pers_json).unwrap();
    let fn_: Needs = serde_json::from_str(&needs_json).unwrap();
    let fpos: Position = serde_json::from_str(&pos_json).unwrap();
    let fv: Velocity = serde_json::from_str(&vel_json).unwrap();
    let fpa: PathCache = serde_json::from_str(&path_json).unwrap();

    assert_eq!(fd.age, 42, "age drifted");
    assert_eq!(fd.happiness, 72.5, "happiness drifted");
    assert_eq!(fd.salary, 6500.0, "salary drifted");
    assert_eq!(fp.ambition, 0.85, "ambition drifted");
    assert_eq!(fn_.hunger, 55.0, "hunger drifted");
    assert_eq!(fpos.x, 200.5, "pos.x drifted");
    assert_eq!(fv.x, 1.5, "vel.x drifted");
    assert_eq!(fpa.waypoints.len(), 3, "path len changed");
}

/// Stress test: entity count and treasury stability through 100 save/load cycles.
#[test]
fn test_rapid_save_load_entity_and_treasury_stability() {
    use crate::SaveableRegistry;

    let mut city = TestCity::new()
        .with_budget(50_000.0)
        .with_road(10, 10, 30, 10, RoadType::Local)
        .with_road(20, 5, 20, 15, RoadType::Local)
        .with_building(12, 11, ZoneType::ResidentialLow, 1)
        .with_building(14, 11, ZoneType::ResidentialLow, 1)
        .with_building(18, 11, ZoneType::CommercialLow, 1)
        .with_building(22, 11, ZoneType::Industrial, 1)
        .with_service(25, 11, ServiceType::PoliceStation)
        .with_utility(28, 11, UtilityType::PowerPlant)
        .with_citizen((12, 11), (18, 11))
        .with_citizen((14, 11), (22, 11))
        .with_citizen((12, 11), (22, 11));

    let ic = city.citizen_count();
    let ib = city.building_count();
    let it = city.budget().treasury;
    let ir = city.road_cell_count();

    for cycle in 0..100 {
        let ext = {
            let w = city.world_mut();
            let r = w.resource::<SaveableRegistry>();
            r.save_all(w)
        };
        {
            let w = city.world_mut();
            let r = w.remove_resource::<SaveableRegistry>().unwrap();
            r.load_all(w, &ext);
            w.insert_resource(r);
        }
        assert_eq!(city.citizen_count(), ic, "cycle {cycle}: citizens changed");
        assert_eq!(
            city.building_count(),
            ib,
            "cycle {cycle}: buildings changed"
        );
        assert!(
            (city.budget().treasury - it).abs() < 1e-6,
            "cycle {cycle}: treasury drifted"
        );
        assert_eq!(city.road_cell_count(), ir, "cycle {cycle}: roads changed");
    }
}

/// Stress test: save/load 100 times with interleaved simulation ticks.
#[test]
fn test_rapid_save_load_interleaved_with_ticks() {
    use crate::SaveableRegistry;

    let mut city = TestCity::new()
        .with_budget(50_000.0)
        .with_road(10, 10, 30, 10, RoadType::Local)
        .with_building(12, 11, ZoneType::ResidentialLow, 1)
        .with_building(18, 11, ZoneType::CommercialLow, 1)
        .with_citizen((12, 11), (18, 11));

    let ic = city.citizen_count();
    for cycle in 0..100 {
        city.tick(5);
        let ext = {
            let w = city.world_mut();
            let r = w.resource::<SaveableRegistry>();
            r.save_all(w)
        };
        {
            let w = city.world_mut();
            let r = w.remove_resource::<SaveableRegistry>().unwrap();
            r.load_all(w, &ext);
            w.insert_resource(r);
        }
        assert!(
            city.citizen_count() >= 1,
            "cycle {cycle}: all citizens gone"
        );
    }
    assert!(
        city.citizen_count() <= ic + 500,
        "citizen count exploded, possible leak"
    );
}

/// Stress test: verify save data bytes are deterministic across cycles.
#[test]
fn test_save_data_deterministic_across_cycles() {
    use crate::SaveableRegistry;

    let mut app = bevy::app::App::new();
    app.add_plugins(bevy::MinimalPlugins);
    app.init_resource::<SaveableRegistry>();

    #[derive(bevy::prelude::Resource, Default, Clone, Debug, PartialEq)]
    struct DeterminismTest {
        data: Vec<u32>,
        label: String,
    }

    impl crate::Saveable for DeterminismTest {
        const SAVE_KEY: &'static str = "stress_determinism";
        fn save_to_bytes(&self) -> Option<Vec<u8>> {
            if self.data.is_empty() {
                return None;
            }
            serde_json::to_vec(&(&self.data, &self.label)).ok()
        }
        fn load_from_bytes(bytes: &[u8]) -> Self {
            let (data, label): (Vec<u32>, String) =
                serde_json::from_slice(bytes).unwrap_or_default();
            Self { data, label }
        }
    }

    app.init_resource::<DeterminismTest>();
    {
        let mut r = app.world_mut().resource_mut::<SaveableRegistry>();
        r.register::<DeterminismTest>();
    }
    {
        let mut r = app.world_mut().resource_mut::<DeterminismTest>();
        r.data = (0..100).collect();
        r.label = "determinism_stress".to_string();
    }

    let ref_ext = {
        let r = app.world().resource::<SaveableRegistry>();
        r.save_all(app.world())
    };
    let ref_bytes = ref_ext.get("stress_determinism").unwrap().clone();

    for cycle in 0..100 {
        let ext = {
            let r = app.world().resource::<SaveableRegistry>();
            r.save_all(app.world())
        };
        assert_eq!(
            ext.get("stress_determinism").unwrap(),
            &ref_bytes,
            "cycle {cycle}: non-deterministic"
        );
    }
}
