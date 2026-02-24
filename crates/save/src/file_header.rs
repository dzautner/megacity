// ---------------------------------------------------------------------------
// file_header – Save file header with magic bytes, version, and checksum
// ---------------------------------------------------------------------------
//
// Header format (28 bytes, fixed-size, little-endian):
//   [0..4]   Magic bytes: "MEGA" (0x4D454741)
//   [4..8]   Format version (u32)
//   [8..12]  Flags (u32: bit 0 = compressed, bit 1 = delta save)
//   [12..20] Timestamp (Unix epoch, u64)
//   [20..24] Uncompressed data size (u32)
//   [24..28] xxHash32 checksum of the data payload (everything after the header)
//
// On save: encode SaveData -> prepend header (with checksum of encoded data)
// On load: check magic -> validate checksum -> strip header -> decode SaveData
// Legacy: if first 4 bytes != "MEGA", treat as raw bitcode (headerless save)

use xxhash_rust::xxh32::xxh32;

/// Magic bytes identifying a Megacity save file.
pub const MAGIC: [u8; 4] = [0x4D, 0x45, 0x47, 0x41]; // "MEGA"

/// Size of the file header in bytes.
pub const HEADER_SIZE: usize = 28;

/// Current file header format version. This is distinct from the SaveData
/// version (which tracks schema changes). The header format version tracks
/// changes to the header layout itself.
pub const HEADER_FORMAT_VERSION: u32 = 1;

/// Seed for xxHash32 checksum.
const XXHASH_SEED: u32 = 0;

/// Parsed file header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileHeader {
    pub format_version: u32,
    pub flags: u32,
    pub timestamp: u64,
    pub uncompressed_size: u32,
    pub checksum: u32,
}

impl FileHeader {
    /// Create a new header for the given data payload.
    pub fn new(data: &[u8]) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            format_version: HEADER_FORMAT_VERSION,
            flags: 0,
            timestamp,
            uncompressed_size: data.len() as u32,
            checksum: xxh32(data, XXHASH_SEED),
        }
    }
}

/// Wrap encoded save data with a file header.
///
/// Returns bytes: [header (28 bytes)] ++ [data payload].
pub fn wrap_with_header(data: &[u8]) -> Vec<u8> {
    let header = FileHeader::new(data);
    let mut out = Vec::with_capacity(HEADER_SIZE + data.len());

    // Magic
    out.extend_from_slice(&MAGIC);
    // Format version
    out.extend_from_slice(&header.format_version.to_le_bytes());
    // Flags
    out.extend_from_slice(&header.flags.to_le_bytes());
    // Timestamp
    out.extend_from_slice(&header.timestamp.to_le_bytes());
    // Uncompressed data size
    out.extend_from_slice(&header.uncompressed_size.to_le_bytes());
    // Checksum
    out.extend_from_slice(&header.checksum.to_le_bytes());

    out.extend_from_slice(data);
    out
}

/// Result of unwrapping a save file's bytes.
pub enum UnwrapResult<'a> {
    /// File has a valid header; the payload bytes follow.
    WithHeader {
        header: FileHeader,
        payload: &'a [u8],
    },
    /// File has no header (legacy save); the entire buffer is the payload.
    Legacy(&'a [u8]),
}

/// Parse and validate the file header from raw bytes.
///
/// - If the file starts with "MEGA", parse the header, verify the checksum,
///   and return `UnwrapResult::WithHeader`.
/// - If the file does NOT start with "MEGA", return `UnwrapResult::Legacy`
///   so callers can attempt to decode it as a legacy headerless save.
///
/// # Errors
///
/// Returns an error if:
/// - The header is present but the file is too short
/// - The header format version is from a newer game build
/// - The checksum does not match (data corruption)
pub fn unwrap_header(bytes: &[u8]) -> Result<UnwrapResult<'_>, String> {
    // Legacy detection: if first 4 bytes aren't MEGA, treat as raw bitcode.
    if bytes.len() < 4 || bytes[..4] != MAGIC {
        return Ok(UnwrapResult::Legacy(bytes));
    }

    // We have magic bytes; now we need at least HEADER_SIZE bytes.
    if bytes.len() < HEADER_SIZE {
        return Err(format!(
            "Save file has MEGA magic bytes but is too short ({} bytes, \
             need at least {} for header)",
            bytes.len(),
            HEADER_SIZE
        ));
    }

    // Parse header fields (all little-endian).
    let format_version = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    let flags = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
    let timestamp = u64::from_le_bytes([
        bytes[12], bytes[13], bytes[14], bytes[15], bytes[16], bytes[17], bytes[18], bytes[19],
    ]);
    let uncompressed_size = u32::from_le_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
    let checksum = u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]);

    // Reject saves from a newer header format version.
    if format_version > HEADER_FORMAT_VERSION {
        return Err(format!(
            "Save file uses header format version {}, but this build only supports \
             up to version {}. Please update the game to load this save.",
            format_version, HEADER_FORMAT_VERSION,
        ));
    }

    let payload = &bytes[HEADER_SIZE..];

    // Verify checksum.
    let computed = xxh32(payload, XXHASH_SEED);
    if computed != checksum {
        return Err(format!(
            "Save file is corrupted: checksum mismatch \
             (expected {:#010X}, got {:#010X}). The file may have been \
             modified or damaged.",
            checksum, computed,
        ));
    }

    Ok(UnwrapResult::WithHeader {
        header: FileHeader {
            format_version,
            flags,
            timestamp,
            uncompressed_size,
            checksum,
        },
        payload,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_and_unwrap_roundtrip() {
        let data = b"hello world save data";
        let wrapped = wrap_with_header(data);

        // Should start with MEGA magic.
        assert_eq!(&wrapped[..4], &MAGIC);
        assert_eq!(wrapped.len(), HEADER_SIZE + data.len());

        // Unwrap should succeed.
        let result = unwrap_header(&wrapped).expect("unwrap should succeed");
        match result {
            UnwrapResult::WithHeader { header, payload } => {
                assert_eq!(header.format_version, HEADER_FORMAT_VERSION);
                assert_eq!(header.flags, 0);
                assert_eq!(header.uncompressed_size, data.len() as u32);
                assert_eq!(payload, data);
            }
            UnwrapResult::Legacy(_) => panic!("expected WithHeader, got Legacy"),
        }
    }

    #[test]
    fn test_legacy_detection() {
        // Data that doesn't start with MEGA should be treated as legacy.
        let data = b"\x00\x01\x02\x03some old save data";
        let result = unwrap_header(data).expect("unwrap should succeed");
        match result {
            UnwrapResult::Legacy(payload) => {
                assert_eq!(payload, data.as_slice());
            }
            UnwrapResult::WithHeader { .. } => panic!("expected Legacy, got WithHeader"),
        }
    }

    #[test]
    fn test_empty_data_is_legacy() {
        let result = unwrap_header(b"").expect("unwrap should succeed");
        match result {
            UnwrapResult::Legacy(payload) => {
                assert!(payload.is_empty());
            }
            UnwrapResult::WithHeader { .. } => panic!("expected Legacy"),
        }
    }

    #[test]
    fn test_corrupted_checksum_detected() {
        let data = b"test payload";
        let mut wrapped = wrap_with_header(data);

        // Corrupt one byte of the payload.
        let last = wrapped.len() - 1;
        wrapped[last] ^= 0xFF;

        let result = unwrap_header(&wrapped);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("corrupted"),
            "Error should mention corruption: {err}"
        );
        assert!(
            err.contains("checksum mismatch"),
            "Error should mention checksum: {err}"
        );
    }

    #[test]
    fn test_future_header_version_rejected() {
        let data = b"test payload";
        let mut wrapped = wrap_with_header(data);

        // Set format_version to a future value (999).
        let future_ver = 999u32.to_le_bytes();
        wrapped[4..8].copy_from_slice(&future_ver);

        let result = unwrap_header(&wrapped);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("header format version 999"),
            "Error should mention the version: {err}"
        );
    }

    #[test]
    fn test_truncated_header_detected() {
        // Only MEGA + a few bytes (less than HEADER_SIZE).
        let data = b"MEGA\x01\x00";
        let result = unwrap_header(data);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("too short"),
            "Error should mention too short: {err}"
        );
    }

    #[test]
    fn test_checksum_deterministic() {
        let data = b"deterministic test";
        let c1 = xxh32(data, XXHASH_SEED);
        let c2 = xxh32(data, XXHASH_SEED);
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_different_data_different_checksum() {
        let c1 = xxh32(b"data A", XXHASH_SEED);
        let c2 = xxh32(b"data B", XXHASH_SEED);
        assert_ne!(c1, c2);
    }

    #[test]
    fn test_empty_payload_roundtrip() {
        let data: &[u8] = b"";
        let wrapped = wrap_with_header(data);
        assert_eq!(wrapped.len(), HEADER_SIZE);

        let result = unwrap_header(&wrapped).expect("unwrap should succeed");
        match result {
            UnwrapResult::WithHeader { header, payload } => {
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
            UnwrapResult::WithHeader { header, payload } => {
                assert_eq!(header.uncompressed_size, 100_000);
                assert_eq!(payload, data.as_slice());
            }
            UnwrapResult::Legacy(_) => panic!("expected WithHeader"),
        }
    }
}
