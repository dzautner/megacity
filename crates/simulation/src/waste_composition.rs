/// Waste composition model (WASTE-002).
///
/// Breaks municipal solid waste (MSW) into material categories and provides
/// methods to compute the recyclable fraction, compostable fraction, and
/// energy content (BTU/lb) of the waste stream.
///
/// Default percentages reflect the average US MSW composition (EPA data):
///   paper/cardboard 25%, food waste 22%, yard waste 12%, plastics 13%,
///   metals 9%, glass 4%, wood 6%, textiles 6%, other 3%.

/// Per-material energy content in BTU per pound (higher heating value).
/// Sources: EPA / US DOE waste-to-energy reference data.
const BTU_PAPER: f32 = 7_200.0;
const BTU_FOOD: f32 = 2_400.0;
const BTU_YARD: f32 = 2_800.0;
const BTU_PLASTICS: f32 = 14_000.0;
const BTU_METALS: f32 = 300.0;
const BTU_GLASS: f32 = 60.0;
const BTU_WOOD: f32 = 6_500.0;
const BTU_TEXTILES: f32 = 7_500.0;
const BTU_OTHER: f32 = 3_000.0;

/// Waste composition expressed as fractional percentages (each 0.0..=1.0,
/// summing to 1.0) for nine material categories.
#[derive(Debug, Clone, PartialEq)]
pub struct WasteComposition {
    /// Paper and cardboard fraction.
    pub paper: f32,
    /// Food waste fraction.
    pub food: f32,
    /// Yard waste / trimmings fraction.
    pub yard: f32,
    /// Plastics fraction.
    pub plastics: f32,
    /// Metals (ferrous + non-ferrous) fraction.
    pub metals: f32,
    /// Glass fraction.
    pub glass: f32,
    /// Wood fraction.
    pub wood: f32,
    /// Textiles fraction.
    pub textiles: f32,
    /// Other / miscellaneous fraction.
    pub other: f32,
}

impl Default for WasteComposition {
    /// Average US municipal solid waste composition (EPA data).
    fn default() -> Self {
        Self {
            paper: 0.25,
            food: 0.22,
            yard: 0.12,
            plastics: 0.13,
            metals: 0.09,
            glass: 0.04,
            wood: 0.06,
            textiles: 0.06,
            other: 0.03,
        }
    }
}

impl WasteComposition {
    // =========================================================================
    // Factory methods for building-type-specific compositions
    // =========================================================================

    /// Typical residential waste composition (close to national average).
    pub fn residential() -> Self {
        Self::default()
    }

    /// General commercial / office waste composition.
    /// Higher paper fraction, less food and yard waste than residential.
    pub fn commercial() -> Self {
        Self {
            paper: 0.30,
            food: 0.15,
            yard: 0.05,
            plastics: 0.15,
            metals: 0.08,
            glass: 0.04,
            wood: 0.08,
            textiles: 0.05,
            other: 0.10,
        }
    }

    /// Restaurant / food-service waste composition.
    /// Dominated by food waste; very little yard waste or metals.
    pub fn restaurant() -> Self {
        Self {
            paper: 0.18,
            food: 0.45,
            yard: 0.02,
            plastics: 0.14,
            metals: 0.04,
            glass: 0.06,
            wood: 0.03,
            textiles: 0.02,
            other: 0.06,
        }
    }

    /// Industrial waste composition.
    /// Heavier on wood, metals, and plastics (packaging / pallets).
    pub fn industrial() -> Self {
        Self {
            paper: 0.15,
            food: 0.05,
            yard: 0.02,
            plastics: 0.18,
            metals: 0.20,
            glass: 0.03,
            wood: 0.20,
            textiles: 0.04,
            other: 0.13,
        }
    }

    // =========================================================================
    // Derived metrics
    // =========================================================================

    /// Fraction of the waste stream that is recyclable (by weight).
    ///
    /// Recyclability rates per material:
    ///   paper 80%, plastics 30%, metals 95%, glass 90%, wood 20%, textiles 15%.
    pub fn recyclable_fraction(&self) -> f32 {
        self.paper * 0.80
            + self.plastics * 0.30
            + self.metals * 0.95
            + self.glass * 0.90
            + self.wood * 0.20
            + self.textiles * 0.15
    }

    /// Fraction of the waste stream that is compostable (by weight).
    ///
    /// Compostability rates per material:
    ///   food 95%, yard 98%, paper 10%, wood 30%.
    pub fn compostable_fraction(&self) -> f32 {
        self.food * 0.95 + self.yard * 0.98 + self.paper * 0.10 + self.wood * 0.30
    }

    /// Weighted-average energy content of the waste stream in BTU per pound.
    ///
    /// For average US MSW this should be approximately 4,500 BTU/lb.
    pub fn energy_content_btu_per_lb(&self) -> f32 {
        self.paper * BTU_PAPER
            + self.food * BTU_FOOD
            + self.yard * BTU_YARD
            + self.plastics * BTU_PLASTICS
            + self.metals * BTU_METALS
            + self.glass * BTU_GLASS
            + self.wood * BTU_WOOD
            + self.textiles * BTU_TEXTILES
            + self.other * BTU_OTHER
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: assert that composition fractions sum to 1.0 (within tolerance).
    fn assert_sums_to_one(c: &WasteComposition) {
        let sum = c.paper
            + c.food
            + c.yard
            + c.plastics
            + c.metals
            + c.glass
            + c.wood
            + c.textiles
            + c.other;
        assert!(
            (sum - 1.0).abs() < 0.001,
            "composition fractions sum to {sum}, expected 1.0"
        );
    }

    // =========================================================================
    // Composition validity
    // =========================================================================

    #[test]
    fn default_composition_sums_to_one() {
        assert_sums_to_one(&WasteComposition::default());
    }

    #[test]
    fn residential_composition_sums_to_one() {
        assert_sums_to_one(&WasteComposition::residential());
    }

    #[test]
    fn commercial_composition_sums_to_one() {
        assert_sums_to_one(&WasteComposition::commercial());
    }

    #[test]
    fn restaurant_composition_sums_to_one() {
        assert_sums_to_one(&WasteComposition::restaurant());
    }

    #[test]
    fn industrial_composition_sums_to_one() {
        assert_sums_to_one(&WasteComposition::industrial());
    }

    // =========================================================================
    // Recyclable fraction
    // =========================================================================

    #[test]
    fn recyclable_fraction_default_about_040() {
        let c = WasteComposition::default();
        let r = c.recyclable_fraction();
        // Expected: 0.25*0.80 + 0.13*0.30 + 0.09*0.95 + 0.04*0.90 + 0.06*0.20 + 0.06*0.15
        // = 0.200 + 0.039 + 0.0855 + 0.036 + 0.012 + 0.009 = 0.3815
        assert!(
            (r - 0.38).abs() < 0.05,
            "recyclable_fraction() = {r}, expected ~0.38..0.42"
        );
    }

    // =========================================================================
    // Compostable fraction
    // =========================================================================

    #[test]
    fn compostable_fraction_default_about_034() {
        let c = WasteComposition::default();
        let f = c.compostable_fraction();
        // Expected: 0.22*0.95 + 0.12*0.98 + 0.25*0.10 + 0.06*0.30
        // = 0.209 + 0.1176 + 0.025 + 0.018 = 0.3696
        assert!(
            (f - 0.37).abs() < 0.05,
            "compostable_fraction() = {f}, expected ~0.34..0.39"
        );
    }

    // =========================================================================
    // Energy content
    // =========================================================================

    #[test]
    fn energy_content_default_about_4500_btu() {
        let c = WasteComposition::default();
        let e = c.energy_content_btu_per_lb();
        assert!(
            (e - 4500.0).abs() < 500.0,
            "energy_content_btu_per_lb() = {e}, expected ~4000..5000"
        );
    }

    // =========================================================================
    // Building-type-specific compositions
    // =========================================================================

    #[test]
    fn restaurant_has_higher_food_fraction() {
        let residential = WasteComposition::residential();
        let restaurant = WasteComposition::restaurant();
        assert!(
            restaurant.food > residential.food,
            "restaurant food fraction ({}) should exceed residential ({})",
            restaurant.food,
            residential.food,
        );
    }

    #[test]
    fn industrial_has_higher_metals_fraction() {
        let residential = WasteComposition::residential();
        let industrial = WasteComposition::industrial();
        assert!(
            industrial.metals > residential.metals,
            "industrial metals fraction ({}) should exceed residential ({})",
            industrial.metals,
            residential.metals,
        );
    }

    #[test]
    fn commercial_has_higher_paper_fraction() {
        let residential = WasteComposition::residential();
        let commercial = WasteComposition::commercial();
        assert!(
            commercial.paper > residential.paper,
            "commercial paper fraction ({}) should exceed residential ({})",
            commercial.paper,
            residential.paper,
        );
    }

    #[test]
    fn restaurant_energy_content_lower_than_default() {
        // More food waste (low BTU) means lower energy content overall.
        let default_e = WasteComposition::default().energy_content_btu_per_lb();
        let restaurant_e = WasteComposition::restaurant().energy_content_btu_per_lb();
        assert!(
            restaurant_e < default_e,
            "restaurant energy ({restaurant_e}) should be lower than default ({default_e})"
        );
    }
}
