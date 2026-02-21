use crate::grid::ZoneType;
use crate::test_harness::TestCity;

#[test]
fn zone_placement_sets_zone_type() {
    let city = TestCity::new().with_zone(100, 100, ZoneType::ResidentialLow);

    city.assert_zone(100, 100, ZoneType::ResidentialLow);
}

#[test]
fn zone_rect_sets_all_cells() {
    let city = TestCity::new().with_zone_rect(100, 100, 104, 104, ZoneType::CommercialHigh);

    for y in 100..=104 {
        for x in 100..=104 {
            city.assert_zone(x, y, ZoneType::CommercialHigh);
        }
    }
    city.assert_zone(99, 99, ZoneType::None);
    city.assert_zone(105, 105, ZoneType::None);
}

#[test]
fn zone_count_matches_rect_area() {
    let city = TestCity::new().with_zone_rect(50, 50, 54, 54, ZoneType::Industrial);

    let count = city.zoned_cell_count(ZoneType::Industrial);
    assert_eq!(
        count, 25,
        "5x5 rect should have 25 zoned cells, got {count}"
    );
}
