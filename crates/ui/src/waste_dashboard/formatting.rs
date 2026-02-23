//! Formatting helpers for the waste management dashboard.
//!
//! Provides human-readable display of tonnage, percentage, and dollar values.

/// Formats a tonnage value for human-readable display.
pub fn fmt_tons(tons: f64) -> String {
    if tons >= 1_000_000.0 {
        format!("{:.1}M", tons / 1_000_000.0)
    } else if tons >= 1_000.0 {
        format!("{:.1}K", tons / 1_000.0)
    } else if tons >= 1.0 {
        format!("{:.1}", tons)
    } else {
        format!("{:.2}", tons)
    }
}

/// Formats a percentage (0.0..1.0) for display as "XX.X%".
pub fn fmt_pct(fraction: f64) -> String {
    format!("{:.1}%", fraction * 100.0)
}

/// Formats a dollar amount for display.
pub fn fmt_dollars(amount: f64) -> String {
    if amount.abs() >= 1_000_000.0 {
        format!("${:.1}M", amount / 1_000_000.0)
    } else if amount.abs() >= 1_000.0 {
        format!("${:.1}K", amount / 1_000.0)
    } else {
        format!("${:.0}", amount)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Formatting tests
    // =========================================================================

    #[test]
    fn test_fmt_tons_small() {
        assert_eq!(fmt_tons(0.5), "0.50");
        assert_eq!(fmt_tons(0.99), "0.99");
    }

    #[test]
    fn test_fmt_tons_medium() {
        assert_eq!(fmt_tons(1.0), "1.0");
        assert_eq!(fmt_tons(42.7), "42.7");
        assert_eq!(fmt_tons(999.9), "999.9");
    }

    #[test]
    fn test_fmt_tons_thousands() {
        assert_eq!(fmt_tons(1_000.0), "1.0K");
        assert_eq!(fmt_tons(5_500.0), "5.5K");
        assert_eq!(fmt_tons(999_999.0), "1000.0K");
    }

    #[test]
    fn test_fmt_tons_millions() {
        assert_eq!(fmt_tons(1_000_000.0), "1.0M");
        assert_eq!(fmt_tons(2_500_000.0), "2.5M");
    }

    #[test]
    fn test_fmt_pct() {
        assert_eq!(fmt_pct(0.0), "0.0%");
        assert_eq!(fmt_pct(0.5), "50.0%");
        assert_eq!(fmt_pct(1.0), "100.0%");
        assert_eq!(fmt_pct(0.857), "85.7%");
    }

    #[test]
    fn test_fmt_dollars_small() {
        assert_eq!(fmt_dollars(50.0), "$50");
        assert_eq!(fmt_dollars(999.0), "$999");
    }

    #[test]
    fn test_fmt_dollars_thousands() {
        assert_eq!(fmt_dollars(1_000.0), "$1.0K");
        assert_eq!(fmt_dollars(5_500.0), "$5.5K");
    }

    #[test]
    fn test_fmt_dollars_millions() {
        assert_eq!(fmt_dollars(1_000_000.0), "$1.0M");
    }

    #[test]
    fn test_fmt_dollars_negative() {
        assert_eq!(fmt_dollars(-5_000.0), "$-5.0K");
    }
}
