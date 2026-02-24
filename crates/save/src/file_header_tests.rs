use super::*;
use xxhash_rust::xxh32::xxh32;

const XXHASH_SEED_TEST: u32 = 0;

#[test]
fn test_wrap_and_unwrap_roundtrip() {
    let data = b"hello world save data";
    let metadata = SaveMetadata {
        city_name: "Test City".to_string(),
        population: 42_000,
        treasury: 123_456.78,
        day: 100,
        hour: 14.5,
        play_time_seconds: 3600.0,
    };
    let wrapped = wrap_with_header_and_metadata(data, &metadata);
    assert_eq!(&wrapped[..4], &MAGIC);

    let result = unwrap_header(&wrapped).expect("unwrap should succeed");
    match result {
        UnwrapResult::WithHeader {
            header,
            metadata: meta,
            payload,
        } => {
            assert_eq!(header.format_version, HEADER_FORMAT_VERSION);
            assert_eq!(header.flags, 0);
            assert_eq!(header.uncompressed_size, data.len() as u32);
            assert!(header.metadata_size > 0);
            assert_eq!(payload, data);
            let meta = meta.expect("metadata should be present");
            assert_eq!(meta.city_name, "Test City");
            assert_eq!(meta.population, 42_000);
            assert!((meta.treasury - 123_456.78).abs() < 0.01);
            assert_eq!(meta.day, 100);
        }
        UnwrapResult::Legacy(_) => panic!("expected WithHeader"),
    }
}

#[test]
fn test_wrap_without_explicit_metadata() {
    let data = b"test payload";
    let wrapped = wrap_with_header(data);
    let result = unwrap_header(&wrapped).expect("unwrap should succeed");
    match result {
        UnwrapResult::WithHeader {
            header,
            metadata,
            payload,
        } => {
            assert_eq!(header.format_version, HEADER_FORMAT_VERSION);
            assert_eq!(payload, data);
            let meta = metadata.expect("default metadata should be present");
            assert_eq!(meta.city_name, "Settlement");
            assert_eq!(meta.population, 0);
        }
        UnwrapResult::Legacy(_) => panic!("expected WithHeader"),
    }
}

#[test]
fn test_legacy_detection() {
    let data = b"\x00\x01\x02\x03some old save data";
    let result = unwrap_header(data).expect("unwrap should succeed");
    match result {
        UnwrapResult::Legacy(payload) => assert_eq!(payload, data.as_slice()),
        UnwrapResult::WithHeader { .. } => panic!("expected Legacy"),
    }
}

#[test]
fn test_empty_data_is_legacy() {
    let result = unwrap_header(b"").expect("unwrap should succeed");
    match result {
        UnwrapResult::Legacy(payload) => assert!(payload.is_empty()),
        UnwrapResult::WithHeader { .. } => panic!("expected Legacy"),
    }
}

#[test]
fn test_corrupted_checksum_detected() {
    let data = b"test payload";
    let mut wrapped = wrap_with_header(data);
    let last = wrapped.len() - 1;
    wrapped[last] ^= 0xFF;
    let result = unwrap_header(&wrapped);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("checksum mismatch"));
}

#[test]
fn test_future_header_version_rejected() {
    let data = b"test payload";
    let mut wrapped = wrap_with_header(data);
    let future_ver = 999u32.to_le_bytes();
    wrapped[4..8].copy_from_slice(&future_ver);
    let result = unwrap_header(&wrapped);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("header format version 999"));
}

#[test]
fn test_truncated_header_detected() {
    let data = b"MEGA\x01\x00";
    let result = unwrap_header(data);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("too short"));
}

#[test]
fn test_checksum_deterministic() {
    let c1 = xxh32(b"deterministic test", XXHASH_SEED_TEST);
    let c2 = xxh32(b"deterministic test", XXHASH_SEED_TEST);
    assert_eq!(c1, c2);
}

#[test]
fn test_different_data_different_checksum() {
    let c1 = xxh32(b"data A", XXHASH_SEED_TEST);
    let c2 = xxh32(b"data B", XXHASH_SEED_TEST);
    assert_ne!(c1, c2);
}

#[test]
fn test_empty_payload_roundtrip() {
    let wrapped = wrap_with_header(b"");
    let result = unwrap_header(&wrapped).expect("unwrap should succeed");
    match result {
        UnwrapResult::WithHeader {
            header, payload, ..
        } => {
            assert_eq!(header.uncompressed_size, 0);
            assert!(payload.is_empty());
        }
        UnwrapResult::Legacy(_) => panic!("expected WithHeader"),
    }
}

#[test]
fn test_large_payload_roundtrip() {
    let data: Vec<u8> = (0..100_000).map(|i| (i % 256) as u8).collect();
    let wrapped = wrap_with_header(&data);
    let result = unwrap_header(&wrapped).expect("unwrap should succeed");
    match result {
        UnwrapResult::WithHeader {
            header, payload, ..
        } => {
            assert_eq!(header.uncompressed_size, 100_000);
            assert_eq!(payload, data.as_slice());
        }
        UnwrapResult::Legacy(_) => panic!("expected WithHeader"),
    }
}

#[test]
fn test_read_metadata_only() {
    let data = b"some save data";
    let metadata = SaveMetadata {
        city_name: "Mega Metropolis".to_string(),
        population: 500_000,
        treasury: 999_999.99,
        day: 365,
        hour: 18.0,
        play_time_seconds: 72_000.0,
    };
    let wrapped = wrap_with_header_and_metadata(data, &metadata);
    let meta = read_metadata_only(&wrapped)
        .expect("should succeed")
        .expect("metadata should be present");
    assert_eq!(meta.city_name, "Mega Metropolis");
    assert_eq!(meta.population, 500_000);
}

#[test]
fn test_read_metadata_only_legacy() {
    let result = read_metadata_only(b"\x00\x01legacy save").expect("should succeed");
    assert!(result.is_none());
}

#[test]
fn test_v1_header_backward_compat() {
    let data = b"test payload for v1";
    let checksum = xxh32(data, XXHASH_SEED_TEST);
    let mut v1_bytes = Vec::with_capacity(HEADER_SIZE_V1 + data.len());
    v1_bytes.extend_from_slice(&MAGIC);
    v1_bytes.extend_from_slice(&1u32.to_le_bytes());
    v1_bytes.extend_from_slice(&0u32.to_le_bytes());
    v1_bytes.extend_from_slice(&1_700_000_000u64.to_le_bytes());
    v1_bytes.extend_from_slice(&(data.len() as u32).to_le_bytes());
    v1_bytes.extend_from_slice(&checksum.to_le_bytes());
    v1_bytes.extend_from_slice(data);

    let result = unwrap_header(&v1_bytes).expect("should parse V1 header");
    match result {
        UnwrapResult::WithHeader {
            header,
            metadata,
            payload,
        } => {
            assert_eq!(header.format_version, 1);
            assert_eq!(header.metadata_size, 0);
            assert!(metadata.is_none());
            assert_eq!(payload, data);
        }
        UnwrapResult::Legacy(_) => panic!("expected WithHeader"),
    }
}

#[test]
fn test_compressed_with_metadata_roundtrip() {
    let data = b"some data that will be compressed with LZ4";
    let metadata = SaveMetadata {
        city_name: "Compressed City".to_string(),
        population: 10_000,
        treasury: 50_000.0,
        day: 50,
        hour: 12.0,
        play_time_seconds: 1800.0,
    };
    let wrapped = wrap_with_header_compressed(data, &metadata);
    assert_eq!(&wrapped[..4], &MAGIC);

    let result = unwrap_header(&wrapped).expect("unwrap should succeed");
    match result {
        UnwrapResult::WithHeader {
            header,
            metadata: meta,
            payload,
        } => {
            assert!(header.is_compressed());
            assert_eq!(header.uncompressed_size, data.len() as u32);
            let meta = meta.expect("metadata should be present");
            assert_eq!(meta.city_name, "Compressed City");
            // Decompress and verify
            let decompressed = decompress_payload(payload).expect("decompression should succeed");
            assert_eq!(decompressed, data);
        }
        UnwrapResult::Legacy(_) => panic!("expected WithHeader"),
    }
}
