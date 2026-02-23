#[cfg(test)]
mod tests {
    use crate::natural_resources::ResourceGrid;
    use crate::production::systems::pick_industry_from_nearby_resources;
    use crate::production::types::{chain_for, CityGoods, GoodsType, IndustryType};

    #[test]
    fn test_industry_type_categories() {
        assert!(IndustryType::Agriculture.is_extraction());
        assert!(IndustryType::Forestry.is_extraction());
        assert!(IndustryType::Mining.is_extraction());
        assert!(IndustryType::OilExtraction.is_extraction());

        assert!(IndustryType::FoodProcessing.is_processing());
        assert!(IndustryType::SawMill.is_processing());
        assert!(IndustryType::Smelter.is_processing());
        assert!(IndustryType::Refinery.is_processing());

        assert!(IndustryType::Manufacturing.is_manufacturing());
        assert!(IndustryType::TechAssembly.is_manufacturing());
    }

    #[test]
    fn test_goods_prices() {
        for &g in GoodsType::all() {
            assert!(g.export_price() > 0.0);
            assert!(g.import_price() > g.export_price());
        }
    }

    #[test]
    fn test_city_goods_default() {
        let goods = CityGoods::default();
        for &g in GoodsType::all() {
            assert_eq!(goods.available[&g], 0.0);
            assert_eq!(goods.production_rate[&g], 0.0);
            assert_eq!(goods.consumption_rate[&g], 0.0);
        }
    }

    #[test]
    fn test_production_chain_extraction_has_no_inputs() {
        let chain = chain_for(IndustryType::Agriculture);
        assert!(chain.inputs.is_empty());
        assert!(!chain.outputs.is_empty());
    }

    #[test]
    fn test_production_chain_processing_has_inputs() {
        let chain = chain_for(IndustryType::FoodProcessing);
        assert!(!chain.inputs.is_empty());
        assert!(!chain.outputs.is_empty());
    }

    #[test]
    fn test_pick_industry_no_resources() {
        let grid = ResourceGrid::default();
        let industry = pick_industry_from_nearby_resources(128, 128, &grid);
        assert!(industry.is_manufacturing());
    }
}
