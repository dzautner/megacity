// ---------------------------------------------------------------------------
// save_fuzz_tests – Fuzz testing for save file decoder robustness (part 1)
// ---------------------------------------------------------------------------
//
// Tests random bytes, truncated files, header edge cases, and patterned data.
// All malformed inputs must produce errors, never panics.

#[cfg(test)]
mod tests {
    use crate::file_header::{unwrap_header, wrap_with_header, HEADER_SIZE, MAGIC};
    use crate::save_types::SaveData;

    /// Simple deterministic pseudo-random number generator (xorshift64).
    struct Rng(u64);

    impl Rng {
        fn new(seed: u64) -> Self {
            Self(seed)
        }

        fn next_u64(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0 = x;
            x
        }

        fn next_u8(&mut self) -> u8 {
            (self.next_u64() & 0xFF) as u8
        }

        fn fill_bytes(&mut self, buf: &mut [u8]) {
            for byte in buf.iter_mut() {
                *byte = self.next_u8();
            }
        }

        fn gen_range(&mut self, lo: usize, hi: usize) -> usize {
            if lo >= hi {
                return lo;
            }
            (self.next_u64() as usize) % (hi - lo) + lo
        }
    }

    fn make_valid_save_bytes() -> Vec<u8> {
        use simulation::grid::WorldGrid;

        let grid = WorldGrid::new(4, 4);
        let grid_stage = crate::serialization::collect_grid_stage(
            &grid,
            &simulation::roads::RoadNetwork::default(),
            None,
        );
        let save = SaveData {
            version: 1,
            grid: grid_stage.grid,
            roads: grid_stage.roads,
            clock: crate::serialization::SaveClock {
                elapsed_secs: 0.0,
                time_of_day: 0.0,
                day: 0,
                speed_multiplier: 1.0,
            },
            budget: crate::serialization::SaveBudget {
                funds: 10000.0,
                tax_rate: 0.1,
                income: 0.0,
                expenses: 0.0,
            },
            demand: crate::serialization::SaveDemand {
                residential: 0.5,
                commercial: 0.5,
                industrial: 0.5,
                residential_high: None,
                commercial_high: None,
                office: None,
                vacancy_residential: None,
                vacancy_commercial: None,
                vacancy_industrial: None,
            },
            buildings: vec![],
            citizens: vec![],
            utility_sources: vec![],
            service_buildings: vec![],
            road_segments: None,
            policies: None,
            weather: None,
            unlock_state: None,
            extended_budget: None,
            loan_book: None,
            lifecycle_timer: None,
            virtual_population: None,
            life_sim_timer: None,
            stormwater_grid: None,
            water_sources: None,
            degree_days: None,
            construction_modifiers: None,
            recycling_state: None,
            wind_damage_state: None,
            uhi_grid: None,
            drought_state: None,
            heat_wave_state: None,
            composting_state: None,
            cold_snap_state: None,
            water_treatment_state: None,
            groundwater_depletion_state: None,
            wastewater_state: None,
            hazardous_waste_state: None,
            storm_drainage_state: None,
            landfill_capacity_state: None,
            flood_state: None,
            reservoir_state: None,
            landfill_gas_state: None,
            cso_state: None,
            water_conservation_state: None,
            fog_state: None,
            urban_growth_boundary: None,
            snow_state: None,
            agriculture_state: None,
            extensions: Default::default(),
        };
        save.encode()
    }

    fn make_valid_save_file() -> Vec<u8> {
        let payload = make_valid_save_bytes();
        wrap_with_header(&payload)
    }

    // -- Random bytes tests ------------------------------------------------

    #[test]
    fn test_fuzz_random_bytes_decode() {
        let sizes = [0, 1, 2, 3, 4, 10, 27, 28, 29, 50, 100, 500, 1000, 10_000];
        let mut rng = Rng::new(0xDEAD_BEEF_CAFE_1234);

        for &size in &sizes {
            let mut buf = vec![0u8; size];
            rng.fill_bytes(&mut buf);

            let result = std::panic::catch_unwind(|| SaveData::decode(&buf));
            assert!(
                result.is_ok(),
                "SaveData::decode panicked on {size} random bytes"
            );
        }
    }

    #[test]
    fn test_fuzz_random_bytes_unwrap_header() {
        let sizes = [0, 1, 2, 3, 4, 10, 27, 28, 29, 50, 100, 500, 1000, 10_000];
        let mut rng = Rng::new(0xBAAD_F00D_1234_5678);

        for &size in &sizes {
            let mut buf = vec![0u8; size];
            rng.fill_bytes(&mut buf);

            let result = std::panic::catch_unwind(|| unwrap_header(&buf));
            assert!(
                result.is_ok(),
                "unwrap_header panicked on {size} random bytes"
            );
        }
    }

    #[test]
    fn test_fuzz_random_bytes_full_pipeline() {
        let sizes = [0, 1, 4, 28, 100, 1000, 5000];
        let mut rng = Rng::new(0x1234_5678_9ABC_DEF0);

        for &size in &sizes {
            let mut buf = vec![0u8; size];
            rng.fill_bytes(&mut buf);

            let result = std::panic::catch_unwind(|| match unwrap_header(&buf) {
                Ok(crate::file_header::UnwrapResult::WithHeader { payload, .. }) => {
                    let _ = SaveData::decode(payload);
                }
                Ok(crate::file_header::UnwrapResult::Legacy(payload)) => {
                    let _ = SaveData::decode(payload);
                }
                Err(_) => {}
            });
            assert!(
                result.is_ok(),
                "Full pipeline panicked on {size} random bytes"
            );
        }
    }

    #[test]
    fn test_fuzz_random_bytes_with_magic_prefix() {
        let mut rng = Rng::new(0xFEED_FACE_DEAD_BEEF);

        for trial in 0..50 {
            let size = rng.gen_range(4, 200);
            let mut buf = vec![0u8; size];
            rng.fill_bytes(&mut buf);
            buf[..4].copy_from_slice(&MAGIC);

            let result = std::panic::catch_unwind(|| {
                let header_result = unwrap_header(&buf);
                if let Ok(crate::file_header::UnwrapResult::WithHeader { payload, .. }) =
                    header_result
                {
                    let _ = SaveData::decode(payload);
                }
            });
            assert!(
                result.is_ok(),
                "Panicked on trial {trial} with MEGA-prefixed random bytes"
            );
        }
    }

    // -- Truncated file tests ----------------------------------------------

    #[test]
    fn test_fuzz_truncated_save_file() {
        let full_file = make_valid_save_file();

        let truncation_points = [
            0,
            1,
            2,
            3,
            4,
            HEADER_SIZE - 1,
            HEADER_SIZE,
            HEADER_SIZE + 1,
            full_file.len() / 2,
            full_file.len().saturating_sub(1),
        ];

        for &trunc in &truncation_points {
            if trunc > full_file.len() {
                continue;
            }
            let truncated = &full_file[..trunc];

            let result = std::panic::catch_unwind(|| match unwrap_header(truncated) {
                Ok(crate::file_header::UnwrapResult::WithHeader { payload, .. }) => {
                    let _ = SaveData::decode(payload);
                }
                Ok(crate::file_header::UnwrapResult::Legacy(payload)) => {
                    let _ = SaveData::decode(payload);
                }
                Err(_) => {}
            });
            assert!(
                result.is_ok(),
                "Panicked on truncation at byte {trunc}/{} of valid save",
                full_file.len()
            );
        }
    }

    #[test]
    fn test_fuzz_truncated_payload_only() {
        let valid_payload = make_valid_save_bytes();

        let truncation_points = [
            0,
            1,
            2,
            valid_payload.len() / 4,
            valid_payload.len() / 2,
            valid_payload.len() * 3 / 4,
            valid_payload.len().saturating_sub(1),
        ];

        for &trunc in &truncation_points {
            if trunc > valid_payload.len() {
                continue;
            }
            let truncated = &valid_payload[..trunc];

            let result = std::panic::catch_unwind(|| {
                let _ = SaveData::decode(truncated);
            });
            assert!(
                result.is_ok(),
                "SaveData::decode panicked on truncated payload at byte {trunc}/{}",
                valid_payload.len()
            );
        }
    }

    // -- Oversized file tests ----------------------------------------------

    #[test]
    fn test_fuzz_oversized_uncompressed_size_claim() {
        let small_payload = b"tiny";
        let mut file = Vec::with_capacity(HEADER_SIZE + small_payload.len());

        file.extend_from_slice(&MAGIC);
        file.extend_from_slice(&1u32.to_le_bytes());
        file.extend_from_slice(&0u32.to_le_bytes());
        file.extend_from_slice(&0u64.to_le_bytes());
        file.extend_from_slice(&u32::MAX.to_le_bytes());
        let checksum = xxhash_rust::xxh32::xxh32(small_payload, 0);
        file.extend_from_slice(&checksum.to_le_bytes());
        file.extend_from_slice(small_payload);

        let result = std::panic::catch_unwind(|| match unwrap_header(&file) {
            Ok(crate::file_header::UnwrapResult::WithHeader { payload, .. }) => {
                let _ = SaveData::decode(payload);
            }
            _ => {}
        });
        assert!(
            result.is_ok(),
            "Panicked on oversized uncompressed_size claim"
        );
    }

    #[test]
    fn test_fuzz_header_with_zero_length_payload() {
        let empty_payload: &[u8] = b"";
        let file = wrap_with_header(empty_payload);

        let result = std::panic::catch_unwind(|| match unwrap_header(&file) {
            Ok(crate::file_header::UnwrapResult::WithHeader { payload, .. }) => {
                assert!(payload.is_empty());
                let _ = SaveData::decode(payload);
            }
            _ => panic!("Expected WithHeader for wrapped empty payload"),
        });
        assert!(result.is_ok(), "Panicked on zero-length payload");
    }

    #[test]
    fn test_fuzz_oversized_random_payload() {
        let mut rng = Rng::new(0x0BAD_CAFE_0000_0001);
        let mut big_payload = vec![0u8; 100_000];
        rng.fill_bytes(&mut big_payload);

        let file = wrap_with_header(&big_payload);

        let result = std::panic::catch_unwind(|| {
            if let Ok(crate::file_header::UnwrapResult::WithHeader { payload, .. }) =
                unwrap_header(&file)
            {
                let _ = SaveData::decode(payload);
            }
        });
        assert!(result.is_ok(), "Panicked on 100KB random payload");
    }

    // -- Header edge cases -------------------------------------------------

    #[test]
    fn test_fuzz_header_edge_case_versions() {
        let valid_payload = make_valid_save_bytes();
        let checksum = xxhash_rust::xxh32::xxh32(&valid_payload, 0);

        let versions: [u32; 6] = [0, 1, 2, u32::MAX, u32::MAX - 1, 999_999];

        for &version in &versions {
            let mut file = Vec::with_capacity(HEADER_SIZE + valid_payload.len());
            file.extend_from_slice(&MAGIC);
            file.extend_from_slice(&version.to_le_bytes());
            file.extend_from_slice(&0u32.to_le_bytes());
            file.extend_from_slice(&0u64.to_le_bytes());
            file.extend_from_slice(&(valid_payload.len() as u32).to_le_bytes());
            file.extend_from_slice(&checksum.to_le_bytes());
            file.extend_from_slice(&valid_payload);

            let result = std::panic::catch_unwind(|| match unwrap_header(&file) {
                Ok(crate::file_header::UnwrapResult::WithHeader { payload, .. }) => {
                    let _ = SaveData::decode(payload);
                }
                _ => {}
            });
            assert!(
                result.is_ok(),
                "Panicked on header format version {version}"
            );
        }
    }

    #[test]
    fn test_fuzz_header_edge_case_flags() {
        let valid_payload = make_valid_save_bytes();
        let checksum = xxhash_rust::xxh32::xxh32(&valid_payload, 0);

        let flag_values: [u32; 5] = [0, 1, 2, 3, u32::MAX];

        for &flags in &flag_values {
            let mut file = Vec::with_capacity(HEADER_SIZE + valid_payload.len());
            file.extend_from_slice(&MAGIC);
            file.extend_from_slice(&1u32.to_le_bytes());
            file.extend_from_slice(&flags.to_le_bytes());
            file.extend_from_slice(&0u64.to_le_bytes());
            file.extend_from_slice(&(valid_payload.len() as u32).to_le_bytes());
            file.extend_from_slice(&checksum.to_le_bytes());
            file.extend_from_slice(&valid_payload);

            let result = std::panic::catch_unwind(|| match unwrap_header(&file) {
                Ok(crate::file_header::UnwrapResult::WithHeader { payload, .. }) => {
                    let _ = SaveData::decode(payload);
                }
                _ => {}
            });
            assert!(result.is_ok(), "Panicked on header flags {flags:#X}");
        }
    }

    // -- Patterned data tests ----------------------------------------------

    #[test]
    fn test_fuzz_all_zeros() {
        for size in [0, 1, 28, 100, 1000] {
            let buf = vec![0u8; size];
            let result = std::panic::catch_unwind(|| {
                let _ = SaveData::decode(&buf);
                let _ = unwrap_header(&buf);
            });
            assert!(
                result.is_ok(),
                "Panicked on all-zeros buffer of size {size}"
            );
        }
    }

    #[test]
    fn test_fuzz_all_ones() {
        for size in [0, 1, 28, 100, 1000] {
            let buf = vec![0xFFu8; size];
            let result = std::panic::catch_unwind(|| {
                let _ = SaveData::decode(&buf);
                let _ = unwrap_header(&buf);
            });
            assert!(result.is_ok(), "Panicked on all-0xFF buffer of size {size}");
        }
    }

    #[test]
    fn test_fuzz_ascending_bytes() {
        let buf: Vec<u8> = (0..1000).map(|i| (i % 256) as u8).collect();
        let result = std::panic::catch_unwind(|| {
            let _ = SaveData::decode(&buf);
            let _ = unwrap_header(&buf);
        });
        assert!(result.is_ok(), "Panicked on ascending-byte pattern");
    }

    // -- Checksum mismatch test --------------------------------------------

    #[test]
    fn test_fuzz_checksum_mismatch_detected() {
        let valid_file = make_valid_save_file();

        if valid_file.len() > HEADER_SIZE {
            let mut corrupted = valid_file.clone();
            let last = corrupted.len() - 1;
            corrupted[last] ^= 0x01;

            match unwrap_header(&corrupted) {
                Err(e) => {
                    assert!(
                        e.contains("checksum") || e.contains("corrupted"),
                        "Error should mention checksum: {e}"
                    );
                }
                Ok(_) => {
                    panic!("Expected checksum error for corrupted payload");
                }
            }
        }
    }
}
