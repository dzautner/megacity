//! Integration tests for SaveMetadata encoding/decoding and file header integration.

use crate::file_header::{
    read_metadata_only, unwrap_header, wrap_with_header, wrap_with_header_and_metadata,
    UnwrapResult, HEADER_FORMAT_VERSION,
};
use crate::save_metadata::SaveMetadata;

#[test]
fn test_metadata_roundtrip_with_real_save_data() {
    let fake_payload = vec![0xDE_u8; 1024];
    let metadata = SaveMetadata {
        city_name: "Metropolis".to_string(),
        population: 75_000,
        treasury: 50_000.50,
        day: 42,
        hour: 15.75,
        play_time_seconds: 7200.0,
    };

    let wrapped = wrap_with_header_and_metadata(&fake_payload, &metadata);
    let result = unwrap_header(&wrapped).expect("unwrap should succeed");
    match result {
        UnwrapResult::WithHeader {
            header,
            metadata: meta,
            payload,
        } => {
            assert_eq!(header.format_version, HEADER_FORMAT_VERSION);
            assert_eq!(payload, fake_payload.as_slice());
            let meta = meta.expect("metadata should be present");
            assert_eq!(meta.city_name, "Metropolis");
            assert_eq!(meta.population, 75_000);
            assert!((meta.treasury - 50_000.50).abs() < 0.01);
            assert_eq!(meta.day, 42);
        }
        UnwrapResult::Legacy(_) => panic!("expected WithHeader"),
    }
}

#[test]
fn test_read_metadata_only_skips_payload_decode() {
    let large_payload = vec![0xFF; 1_000_000];
    let metadata = SaveMetadata {
        city_name: "Megacity".to_string(),
        population: 250_000,
        treasury: 999_999.0,
        day: 365,
        hour: 0.0,
        play_time_seconds: 360_000.0,
    };

    let wrapped = wrap_with_header_and_metadata(&large_payload, &metadata);
    let meta = read_metadata_only(&wrapped)
        .expect("should succeed")
        .expect("metadata should be present");

    assert_eq!(meta.city_name, "Megacity");
    assert_eq!(meta.population, 250_000);
    assert_eq!(meta.day, 365);
}

#[test]
fn test_metadata_default_values() {
    let meta = SaveMetadata::default();
    assert_eq!(meta.city_name, "Settlement");
    assert_eq!(meta.population, 0);
    assert_eq!(meta.treasury, 0.0);
    assert_eq!(meta.day, 1);
    assert_eq!(meta.hour, 6.0);
    assert_eq!(meta.play_time_seconds, 0.0);
}

#[test]
fn test_metadata_encode_decode_standalone() {
    let original = SaveMetadata {
        city_name: "Large City".to_string(),
        population: 30_000,
        treasury: -500.0,
        day: 200,
        hour: 23.99,
        play_time_seconds: 86400.0,
    };

    let bytes = original.encode();
    let decoded = SaveMetadata::decode(&bytes).expect("decode should succeed");
    assert_eq!(decoded.city_name, original.city_name);
    assert_eq!(decoded.population, original.population);
    assert!((decoded.treasury - original.treasury).abs() < 0.01);
    assert_eq!(decoded.day, original.day);
}

#[test]
fn test_wrap_with_header_includes_default_metadata() {
    let payload = b"some data";
    let wrapped = wrap_with_header(payload);
    let meta = read_metadata_only(&wrapped)
        .expect("should succeed")
        .expect("default metadata should be present");
    assert_eq!(meta.city_name, "Settlement");
    assert_eq!(meta.population, 0);
}

#[test]
fn test_metadata_with_extreme_values() {
    let metadata = SaveMetadata {
        city_name: "World Capital".to_string(),
        population: u32::MAX,
        treasury: f64::MAX,
        day: u32::MAX,
        hour: 23.999,
        play_time_seconds: f64::MAX,
    };
    let bytes = metadata.encode();
    let decoded = SaveMetadata::decode(&bytes).expect("decode should succeed");
    assert_eq!(decoded.population, u32::MAX);
    assert_eq!(decoded.day, u32::MAX);
}

#[test]
fn test_legacy_save_has_no_metadata() {
    let legacy_bytes = b"\x00\x01\x02\x03some old save data";
    let result = read_metadata_only(legacy_bytes).expect("should succeed");
    assert!(result.is_none());
}
