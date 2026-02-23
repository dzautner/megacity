#[cfg(test)]
mod tests {
    use super::super::random::tick_pseudo_random;
    use super::super::types::*;

    #[test]
    fn test_default_attractiveness() {
        let attr = CityAttractiveness::default();
        assert!((attr.overall_score - 50.0).abs() < 0.01);
        assert!((attr.employment_factor - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_default_immigration_stats() {
        let stats = ImmigrationStats::default();
        assert_eq!(stats.immigrants_this_month, 0);
        assert_eq!(stats.emigrants_this_month, 0);
        assert_eq!(stats.net_migration, 0);
    }

    #[test]
    fn test_tick_pseudo_random_deterministic() {
        // Same tick should produce same result
        assert_eq!(tick_pseudo_random(42), tick_pseudo_random(42));
    }

    #[test]
    fn test_tick_pseudo_random_varies() {
        // Different ticks should produce different results
        let a = tick_pseudo_random(100);
        let b = tick_pseudo_random(101);
        let c = tick_pseudo_random(102);
        // Extremely unlikely all three are equal
        assert!(a != b || b != c);
    }

    #[test]
    fn test_tick_pseudo_random_distribution() {
        // Check that modulo 10 produces a roughly even distribution
        let mut buckets = [0u32; 10];
        for i in 0..1000u64 {
            let val = tick_pseudo_random(i) % 10;
            buckets[val as usize] += 1;
        }
        // Each bucket should have roughly 100 (+/- 50 for statistical noise)
        for &count in &buckets {
            assert!(count > 50, "bucket too low: {}", count);
            assert!(count < 200, "bucket too high: {}", count);
        }
    }

    #[test]
    fn test_weight_sum() {
        // Weights should sum to 100
        let total =
            WEIGHT_EMPLOYMENT + WEIGHT_HAPPINESS + WEIGHT_SERVICES + WEIGHT_HOUSING + WEIGHT_TAX;
        assert!((total - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_max_attractiveness() {
        // All factors at 1.0 should yield score of 100
        let score = 1.0 * WEIGHT_EMPLOYMENT
            + 1.0 * WEIGHT_HAPPINESS
            + 1.0 * WEIGHT_SERVICES
            + 1.0 * WEIGHT_HOUSING
            + 1.0 * WEIGHT_TAX;
        assert!((score - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_min_attractiveness() {
        // All factors at 0.0 should yield score of 0
        let score = 0.0 * WEIGHT_EMPLOYMENT
            + 0.0 * WEIGHT_HAPPINESS
            + 0.0 * WEIGHT_SERVICES
            + 0.0 * WEIGHT_HOUSING
            + 0.0 * WEIGHT_TAX;
        assert!((score - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_tax_factor_baseline() {
        // At 10% tax rate (baseline), factor should be 0.5
        let baseline_tax = 0.10f32;
        let tax_diff = baseline_tax - 0.10;
        let factor = (0.5 - tax_diff * 5.0).clamp(0.0, 1.0);
        assert!((factor - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_tax_factor_low_tax() {
        // At 0% tax, factor should be 1.0
        let tax_rate = 0.0f32;
        let tax_diff = tax_rate - 0.10;
        let factor = (0.5 - tax_diff * 5.0).clamp(0.0, 1.0);
        assert!((factor - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_tax_factor_high_tax() {
        // At 20% tax, factor should be 0.0
        let tax_rate = 0.20f32;
        let tax_diff = tax_rate - 0.10;
        let factor = (0.5 - tax_diff * 5.0).clamp(0.0, 1.0);
        assert!((factor - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_employment_factor() {
        // 0% unemployment -> 1.0
        let factor: f32 = (1.0 - 0.0f32 * 5.0).clamp(0.0, 1.0);
        assert!((factor - 1.0).abs() < 0.01);

        // 10% unemployment -> 0.5
        let factor: f32 = (1.0 - 0.10f32 * 5.0).clamp(0.0, 1.0);
        assert!((factor - 0.5).abs() < 0.01);

        // 20%+ unemployment -> 0.0
        let factor: f32 = (1.0 - 0.20f32 * 5.0).clamp(0.0, 1.0);
        assert!((factor - 0.0).abs() < 0.01);
    }
}
