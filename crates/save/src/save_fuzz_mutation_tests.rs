// ---------------------------------------------------------------------------
// save_fuzz_mutation_tests – Mutation-based fuzz testing for save decoder
// ---------------------------------------------------------------------------
//
// Tests corrupted bodies, bit-flip mutations, byte-level mutations, and
// stress testing with high trial counts. All malformed inputs must produce
// errors, never panics.

#[cfg(test)]
mod tests {
    use crate::file_header::{unwrap_header, wrap_with_header};
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

    // -- Valid header + corrupted body -------------------------------------

    #[test]
    fn test_fuzz_valid_header_corrupted_body() {
        let valid_payload = make_valid_save_bytes();
        let mut rng = Rng::new(0xCAFE_BABE_0000_0001);

        for trial in 0..20 {
            let mut garbage_body = vec![0u8; valid_payload.len()];
            rng.fill_bytes(&mut garbage_body);

            let file_bytes = wrap_with_header(&garbage_body);

            let result = std::panic::catch_unwind(|| match unwrap_header(&file_bytes) {
                Ok(crate::file_header::UnwrapResult::WithHeader { payload, .. }) => {
                    let decode_result = SaveData::decode(payload);
                    assert!(
                        decode_result.is_err(),
                        "Trial {trial}: garbage body decoded without error"
                    );
                }
                other => {
                    panic!("Trial {trial}: expected WithHeader, got {other:?}");
                }
            });
            assert!(
                result.is_ok(),
                "Panicked on trial {trial} with valid header + corrupted body"
            );
        }
    }

    #[test]
    fn test_fuzz_valid_header_partially_corrupted_body() {
        let valid_payload = make_valid_save_bytes();
        let mut rng = Rng::new(0xABCD_EF01_2345_6789);

        for trial in 0..20 {
            let mut corrupted = valid_payload.clone();
            let num_corruptions = rng.gen_range(1, 11);
            for _ in 0..num_corruptions {
                if !corrupted.is_empty() {
                    let idx = rng.gen_range(0, corrupted.len());
                    corrupted[idx] ^= rng.next_u8() | 1;
                }
            }

            let file_bytes = wrap_with_header(&corrupted);

            let result = std::panic::catch_unwind(|| {
                if let Ok(crate::file_header::UnwrapResult::WithHeader { payload, .. }) =
                    unwrap_header(&file_bytes)
                {
                    let _ = SaveData::decode(payload);
                }
            });
            assert!(
                result.is_ok(),
                "Panicked on trial {trial} with partially corrupted body"
            );
        }
    }

    // -- Bit-flip mutations ------------------------------------------------

    #[test]
    fn test_fuzz_single_bit_flips_on_payload() {
        let valid_payload = make_valid_save_bytes();
        if valid_payload.is_empty() {
            return;
        }

        let mut rng = Rng::new(0xB17F_1100_0000_0001);

        for trial in 0..50 {
            let mut mutated = valid_payload.clone();
            let byte_idx = rng.gen_range(0, mutated.len());
            let bit_idx = rng.gen_range(0, 8);
            mutated[byte_idx] ^= 1 << bit_idx;

            let result = std::panic::catch_unwind(|| {
                let _ = SaveData::decode(&mutated);
            });
            assert!(
                result.is_ok(),
                "SaveData::decode panicked on single-bit flip trial {trial} \
                 (byte {byte_idx}, bit {bit_idx})"
            );
        }
    }

    #[test]
    fn test_fuzz_multi_bit_flips_on_file() {
        let valid_file = make_valid_save_file();
        let mut rng = Rng::new(0xB17F_1100_0000_0002);

        for trial in 0..30 {
            let mut mutated = valid_file.clone();
            let num_flips = rng.gen_range(1, 20);
            for _ in 0..num_flips {
                let byte_idx = rng.gen_range(0, mutated.len());
                let bit_idx = rng.gen_range(0, 8);
                mutated[byte_idx] ^= 1 << bit_idx;
            }

            let result = std::panic::catch_unwind(|| match unwrap_header(&mutated) {
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
                "Full pipeline panicked on multi-bit-flip trial {trial}"
            );
        }
    }

    // -- Byte-level mutations (overwrite, insert, delete) ------------------

    #[test]
    fn test_fuzz_byte_overwrite_mutations() {
        let valid_payload = make_valid_save_bytes();
        if valid_payload.is_empty() {
            return;
        }
        let mut rng = Rng::new(0x00E2_0000_0000_0001);

        for trial in 0..30 {
            let mut mutated = valid_payload.clone();
            let count = rng.gen_range(1, 6);
            for _ in 0..count {
                let idx = rng.gen_range(0, mutated.len());
                mutated[idx] = rng.next_u8();
            }

            let result = std::panic::catch_unwind(|| {
                let _ = SaveData::decode(&mutated);
            });
            assert!(
                result.is_ok(),
                "SaveData::decode panicked on byte-overwrite trial {trial}"
            );
        }
    }

    #[test]
    fn test_fuzz_byte_insertion_mutations() {
        let valid_payload = make_valid_save_bytes();
        let mut rng = Rng::new(0x1115_3270_0000_0001);

        for trial in 0..20 {
            let mut mutated = valid_payload.clone();
            let pos = rng.gen_range(0, mutated.len().saturating_add(1));
            let count = rng.gen_range(1, 11);
            for _ in 0..count {
                let byte = rng.next_u8();
                if pos <= mutated.len() {
                    mutated.insert(pos, byte);
                }
            }

            let result = std::panic::catch_unwind(|| {
                let _ = SaveData::decode(&mutated);
            });
            assert!(
                result.is_ok(),
                "SaveData::decode panicked on byte-insertion trial {trial}"
            );
        }
    }

    #[test]
    fn test_fuzz_byte_deletion_mutations() {
        let valid_payload = make_valid_save_bytes();
        if valid_payload.is_empty() {
            return;
        }
        let mut rng = Rng::new(0xDE1E_7E00_0000_0001);

        for trial in 0..20 {
            let mut mutated = valid_payload.clone();
            let count = rng.gen_range(1, 11).min(mutated.len());
            for _ in 0..count {
                if mutated.is_empty() {
                    break;
                }
                let idx = rng.gen_range(0, mutated.len());
                mutated.remove(idx);
            }

            let result = std::panic::catch_unwind(|| {
                let _ = SaveData::decode(&mutated);
            });
            assert!(
                result.is_ok(),
                "SaveData::decode panicked on byte-deletion trial {trial}"
            );
        }
    }

    // -- Stress test with high trial count ---------------------------------

    #[test]
    fn test_fuzz_stress_random_inputs() {
        let mut rng = Rng::new(0x5700_5500_0000_0001);

        for trial in 0..200 {
            let size = rng.gen_range(0, 2000);
            let mut buf = vec![0u8; size];
            rng.fill_bytes(&mut buf);

            let result = std::panic::catch_unwind(|| {
                let _ = SaveData::decode(&buf);
                match unwrap_header(&buf) {
                    Ok(crate::file_header::UnwrapResult::WithHeader { payload, .. }) => {
                        let _ = SaveData::decode(payload);
                    }
                    Ok(crate::file_header::UnwrapResult::Legacy(payload)) => {
                        let _ = SaveData::decode(payload);
                    }
                    Err(_) => {}
                }
            });
            assert!(result.is_ok(), "Stress test panicked on trial {trial}");
        }
    }
}
