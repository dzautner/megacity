//! Unit tests for the energy dashboard types.

#[cfg(test)]
mod tests {
    use crate::energy_dashboard::types::{
        EnergyDashboardVisible, EnergyHistory, GenerationMix, HISTORY_CAPACITY,
    };

    #[test]
    fn test_visible_default_is_hidden() {
        let visible = EnergyDashboardVisible::default();
        assert!(!visible.0);
    }

    #[test]
    fn test_visible_toggle() {
        let mut visible = EnergyDashboardVisible::default();
        visible.0 = true;
        assert!(visible.0);
    }

    #[test]
    fn test_history_default_empty() {
        let history = EnergyHistory::default();
        assert_eq!(history.valid_count(), 0);
        assert_eq!(history.write_idx, 0);
        assert_eq!(history.sample_count, 0);
    }

    #[test]
    fn test_history_push_and_count() {
        let mut history = EnergyHistory::default();
        history.push(10.0, 12.0);
        assert_eq!(history.valid_count(), 1);
        history.push(20.0, 22.0);
        assert_eq!(history.valid_count(), 2);
    }

    #[test]
    fn test_history_ordered_before_full() {
        let mut history = EnergyHistory::default();
        history.push(1.0, 10.0);
        history.push(2.0, 20.0);
        history.push(3.0, 30.0);

        let demand = history.ordered_demand();
        assert_eq!(demand.len(), 3);
        assert!((demand[0] - 1.0).abs() < f32::EPSILON);
        assert!((demand[1] - 2.0).abs() < f32::EPSILON);
        assert!((demand[2] - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_history_wraps_around() {
        let mut history = EnergyHistory::default();
        for i in 0..HISTORY_CAPACITY + 5 {
            history.push(i as f32, i as f32 * 2.0);
        }

        assert_eq!(history.valid_count(), HISTORY_CAPACITY);
        let demand = history.ordered_demand();
        assert_eq!(demand.len(), HISTORY_CAPACITY);
        // Oldest sample should be index 5 (since we wrote 29 samples, oldest surviving = 5)
        assert!((demand[0] - 5.0).abs() < f32::EPSILON);
        // Newest should be 28
        assert!((demand[HISTORY_CAPACITY - 1] - (HISTORY_CAPACITY + 4) as f32).abs() < f32::EPSILON);
    }

    #[test]
    fn test_generation_mix_total() {
        let mix = GenerationMix {
            coal_mw: 100.0,
            gas_mw: 200.0,
            wind_mw: 50.0,
            battery_mw: 10.0,
        };
        assert!((mix.total() - 360.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_generation_mix_default_zero() {
        let mix = GenerationMix::default();
        assert!((mix.total()).abs() < f32::EPSILON);
    }
}
