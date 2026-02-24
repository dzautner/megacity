// ---------------------------------------------------------------------------
// tests_lz4_compression – Tests for LZ4 compression in save/load pipeline
// ---------------------------------------------------------------------------

use crate::file_header::{
    decompress_payload, unwrap_header, wrap_with_header, wrap_with_header_compressed, UnwrapResult,
    FLAG_COMPRESSED, HEADER_FORMAT_VERSION, HEADER_SIZE, MAGIC,
};

#[test]
fn test_compressed_roundtrip() {
    let data = b"hello world save data for compression test";
    let wrapped = wrap_with_header_compressed(data, &crate::save_metadata::SaveMetadata::default());

    // Should start with MEGA magic.
    assert_eq!(&wrapped[..4], &MAGIC);

    // Unwrap the header.
    let result = unwrap_header(&wrapped).expect("unwrap should succeed");
    match result {
        UnwrapResult::WithHeader {
            header, payload, ..
        } => {
            assert_eq!(header.format_version, HEADER_FORMAT_VERSION);
            assert!(header.is_compressed());
            assert_eq!(header.flags & FLAG_COMPRESSED, FLAG_COMPRESSED);
            assert_eq!(header.uncompressed_size, data.len() as u32);

            // Payload is the compressed data (not the original).
            assert_ne!(payload, data.as_slice());

            // Decompress and verify.
            let decompressed = decompress_payload(payload).expect("decompression should succeed");
            assert_eq!(decompressed, data);
        }
        UnwrapResult::Legacy(_) => panic!("expected WithHeader, got Legacy"),
    }
}

#[test]
fn test_uncompressed_header_has_no_compressed_flag() {
    let data = b"uncompressed data";
    let wrapped = wrap_with_header(data);

    let result = unwrap_header(&wrapped).expect("unwrap should succeed");
    match result {
        UnwrapResult::WithHeader {
            header, payload, ..
        } => {
            assert!(!header.is_compressed());
            assert_eq!(header.flags & FLAG_COMPRESSED, 0);
            assert_eq!(payload, data.as_slice());
        }
        UnwrapResult::Legacy(_) => panic!("expected WithHeader, got Legacy"),
    }
}

#[test]
fn test_backward_compat_uncompressed_saves_still_load() {
    // Simulate an old uncompressed save: wrap with header (no compression).
    let data = b"old save data from before compression was added";
    let wrapped = wrap_with_header(data);

    // Load path: unwrap header, check flag, no decompression needed.
    let result = unwrap_header(&wrapped).expect("unwrap should succeed");
    match result {
        UnwrapResult::WithHeader {
            header, payload, ..
        } => {
            assert!(!header.is_compressed());
            // Payload should be the raw data since it's not compressed.
            assert_eq!(payload, data.as_slice());
        }
        UnwrapResult::Legacy(_) => panic!("expected WithHeader, got Legacy"),
    }
}

#[test]
fn test_legacy_saves_without_header_still_load() {
    // Legacy saves don't start with MEGA.
    let data = b"\x00\x01legacy bitcode data";
    let result = unwrap_header(data).expect("unwrap should succeed");
    match result {
        UnwrapResult::Legacy(payload) => {
            assert_eq!(payload, data.as_slice());
        }
        UnwrapResult::WithHeader { .. } => panic!("expected Legacy, got WithHeader"),
    }
}

#[test]
fn test_compressed_save_is_smaller_for_repetitive_data() {
    // Repetitive data should compress well.
    let data: Vec<u8> = "ABCDEFGH".repeat(10_000).into_bytes();
    let uncompressed_wrapped = wrap_with_header(&data);
    let compressed_wrapped =
        wrap_with_header_compressed(&data, &crate::save_metadata::SaveMetadata::default());

    // Compressed should be significantly smaller.
    assert!(
        compressed_wrapped.len() < uncompressed_wrapped.len() / 2,
        "Expected compressed ({}) to be less than half of uncompressed ({})",
        compressed_wrapped.len(),
        uncompressed_wrapped.len(),
    );
}

#[test]
fn test_compressed_empty_payload_roundtrip() {
    let data: &[u8] = b"";
    let wrapped = wrap_with_header_compressed(data, &crate::save_metadata::SaveMetadata::default());

    let result = unwrap_header(&wrapped).expect("unwrap should succeed");
    match result {
        UnwrapResult::WithHeader {
            header, payload, ..
        } => {
            assert!(header.is_compressed());
            assert_eq!(header.uncompressed_size, 0);

            let decompressed = decompress_payload(payload).expect("decompression should succeed");
            assert!(decompressed.is_empty());
        }
        UnwrapResult::Legacy(_) => panic!("expected WithHeader"),
    }
}

#[test]
fn test_compressed_large_payload_roundtrip() {
    let data: Vec<u8> = (0..100_000).map(|i| (i % 256) as u8).collect();
    let wrapped =
        wrap_with_header_compressed(&data, &crate::save_metadata::SaveMetadata::default());

    let result = unwrap_header(&wrapped).expect("unwrap should succeed");
    match result {
        UnwrapResult::WithHeader {
            header, payload, ..
        } => {
            assert!(header.is_compressed());
            assert_eq!(header.uncompressed_size, 100_000);

            let decompressed = decompress_payload(payload).expect("decompression should succeed");
            assert_eq!(decompressed, data);
        }
        UnwrapResult::Legacy(_) => panic!("expected WithHeader"),
    }
}

#[test]
fn test_checksum_covers_compressed_payload() {
    let data = b"test data for checksum verification";
    let mut wrapped =
        wrap_with_header_compressed(data, &crate::save_metadata::SaveMetadata::default());

    // Corrupt one byte of the compressed payload.
    let last = wrapped.len() - 1;
    wrapped[last] ^= 0xFF;

    let result = unwrap_header(&wrapped);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.contains("checksum mismatch"),
        "Error should mention checksum: {err}"
    );
}

#[test]
fn test_compressed_file_has_correct_header_structure() {
    let data = b"structure test data";
    let wrapped = wrap_with_header_compressed(data, &crate::save_metadata::SaveMetadata::default());

    // First 4 bytes: MEGA magic.
    assert_eq!(&wrapped[..4], &MAGIC);

    // Bytes 4..8: format version.
    let version = u32::from_le_bytes([wrapped[4], wrapped[5], wrapped[6], wrapped[7]]);
    assert_eq!(version, HEADER_FORMAT_VERSION);

    // Bytes 8..12: flags with compressed bit set.
    let flags = u32::from_le_bytes([wrapped[8], wrapped[9], wrapped[10], wrapped[11]]);
    assert_eq!(flags & FLAG_COMPRESSED, FLAG_COMPRESSED);

    // Bytes 20..24: uncompressed size matches original data.
    let uncompressed_size =
        u32::from_le_bytes([wrapped[20], wrapped[21], wrapped[22], wrapped[23]]);
    assert_eq!(uncompressed_size, data.len() as u32);

    // Total size should be header + compressed payload (smaller or larger depending on data).
    assert!(wrapped.len() >= HEADER_SIZE);
}
