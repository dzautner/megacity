// ---------------------------------------------------------------------------
// file_header – Save file header with magic bytes, version, and checksum
// ---------------------------------------------------------------------------
//
// Header format v2 (32 bytes, fixed-size, little-endian):
//   [0..4]   Magic bytes: "MEGA" (0x4D454741)
//   [4..8]   Format version (u32)
//   [8..12]  Flags (u32: bit 0 = compressed, bit 1 = delta save)
//   [12..20] Timestamp (Unix epoch, u64)
//   [20..24] Uncompressed data size (u32)
//   [24..28] xxHash32 checksum of the data payload (after header + metadata)
//   [28..32] Metadata size (u32): byte count of the SaveMetadata section
//
// Layout: [Header 32B] [Metadata (metadata_size bytes)] [Data payload]
//
// On save: encode SaveData -> compress -> encode metadata -> prepend header
// On load: parse header -> read metadata -> validate checksum -> decompress -> decode
// Legacy: if first 4 bytes != "MEGA", treat as raw bitcode (headerless save)
// V1 compat: if format_version == 1, header is 28 bytes with no metadata

use crate::save_metadata::SaveMetadata;
use xxhash_rust::xxh32::xxh32;

/// Magic bytes identifying a Megacity save file.
pub const MAGIC: [u8; 4] = [0x4D, 0x45, 0x47, 0x41]; // "MEGA"

/// Size of the V1 file header in bytes (no metadata field).
pub const HEADER_SIZE_V1: usize = 28;

/// Size of the V2 file header in bytes (with metadata_size field).
pub const HEADER_SIZE: usize = 32;

/// Current file header format version. Bumped to 2 for the metadata section.
pub const HEADER_FORMAT_VERSION: u32 = 2;

/// Flag bit 0: payload is LZ4-compressed.
pub const FLAG_COMPRESSED: u32 = 0x1;

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
    /// Size of the metadata section in bytes. Zero means no metadata.
    pub metadata_size: u32,
}

impl FileHeader {
    /// Returns `true` if the compressed flag (bit 0) is set.
    pub fn is_compressed(&self) -> bool {
        self.flags & FLAG_COMPRESSED != 0
    }
}

/// Compress, wrap with header and metadata.
///
/// Layout: [Header 32B] [Metadata] [LZ4-compressed payload]
pub fn wrap_with_header_compressed(data: &[u8], metadata: &SaveMetadata) -> Vec<u8> {
    let compressed = lz4_flex::compress_prepend_size(data);
    let metadata_bytes = metadata.encode();

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let header = FileHeader {
        format_version: HEADER_FORMAT_VERSION,
        flags: FLAG_COMPRESSED,
        timestamp,
        uncompressed_size: data.len() as u32,
        checksum: xxh32(&compressed, XXHASH_SEED),
        metadata_size: metadata_bytes.len() as u32,
    };

    let mut out = Vec::with_capacity(HEADER_SIZE + metadata_bytes.len() + compressed.len());
    write_header(&mut out, &header);
    out.extend_from_slice(&metadata_bytes);
    out.extend_from_slice(&compressed);
    out
}

/// Wrap encoded save data with a header and metadata (uncompressed, test-only).
#[cfg(test)]
pub fn wrap_with_header_and_metadata(data: &[u8], metadata: &SaveMetadata) -> Vec<u8> {
    let metadata_bytes = metadata.encode();

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let header = FileHeader {
        format_version: HEADER_FORMAT_VERSION,
        flags: 0,
        timestamp,
        uncompressed_size: data.len() as u32,
        checksum: xxh32(data, XXHASH_SEED),
        metadata_size: metadata_bytes.len() as u32,
    };

    let mut out = Vec::with_capacity(HEADER_SIZE + metadata_bytes.len() + data.len());
    write_header(&mut out, &header);
    out.extend_from_slice(&metadata_bytes);
    out.extend_from_slice(data);
    out
}

/// Wrap encoded save data with a header (uncompressed, no explicit metadata, test-only).
#[cfg(test)]
pub fn wrap_with_header(data: &[u8]) -> Vec<u8> {
    wrap_with_header_and_metadata(data, &SaveMetadata::default())
}

/// Write header bytes to buffer.
fn write_header(out: &mut Vec<u8>, header: &FileHeader) {
    out.extend_from_slice(&MAGIC);
    out.extend_from_slice(&header.format_version.to_le_bytes());
    out.extend_from_slice(&header.flags.to_le_bytes());
    out.extend_from_slice(&header.timestamp.to_le_bytes());
    out.extend_from_slice(&header.uncompressed_size.to_le_bytes());
    out.extend_from_slice(&header.checksum.to_le_bytes());
    out.extend_from_slice(&header.metadata_size.to_le_bytes());
}

/// Decompress an LZ4-compressed payload.
pub fn decompress_payload(compressed: &[u8]) -> Result<Vec<u8>, String> {
    lz4_flex::decompress_size_prepended(compressed).map_err(|e| {
        format!("Failed to decompress LZ4 payload: {e}. The save file may be corrupted.")
    })
}

/// Result of unwrapping a save file's bytes.
#[derive(Debug)]
pub enum UnwrapResult<'a> {
    /// File has a valid header; the payload bytes follow.
    WithHeader {
        header: FileHeader,
        metadata: Option<SaveMetadata>,
        payload: &'a [u8],
    },
    /// File has no header (legacy save); the entire buffer is the payload.
    Legacy(&'a [u8]),
}

/// Parse and validate the file header from raw bytes.
pub fn unwrap_header(bytes: &[u8]) -> Result<UnwrapResult<'_>, String> {
    if bytes.len() < 4 || bytes[..4] != MAGIC {
        return Ok(UnwrapResult::Legacy(bytes));
    }

    if bytes.len() < HEADER_SIZE_V1 {
        return Err(format!(
            "Save file has MEGA magic bytes but is too short ({} bytes, \
             need at least {} for header)",
            bytes.len(),
            HEADER_SIZE_V1
        ));
    }

    let format_version = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    let flags = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
    let timestamp = u64::from_le_bytes([
        bytes[12], bytes[13], bytes[14], bytes[15], bytes[16], bytes[17], bytes[18], bytes[19],
    ]);
    let uncompressed_size = u32::from_le_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
    let checksum = u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]);

    if format_version > HEADER_FORMAT_VERSION {
        return Err(format!(
            "Save file uses header format version {}, but this build only supports \
             up to version {}. Please update the game to load this save.",
            format_version, HEADER_FORMAT_VERSION,
        ));
    }

    // V1 headers: 28 bytes, no metadata
    if format_version <= 1 {
        let payload = &bytes[HEADER_SIZE_V1..];
        let computed = xxh32(payload, XXHASH_SEED);
        if computed != checksum {
            return Err(format!(
                "Save file is corrupted: checksum mismatch \
                 (expected {:#010X}, got {:#010X}).",
                checksum, computed,
            ));
        }
        return Ok(UnwrapResult::WithHeader {
            header: FileHeader {
                format_version,
                flags,
                timestamp,
                uncompressed_size,
                checksum,
                metadata_size: 0,
            },
            metadata: None,
            payload,
        });
    }

    // V2+ headers: 32 bytes with metadata_size field
    if bytes.len() < HEADER_SIZE {
        return Err(format!(
            "Save file v{} too short ({} bytes, need at least {})",
            format_version,
            bytes.len(),
            HEADER_SIZE
        ));
    }

    let metadata_size = u32::from_le_bytes([bytes[28], bytes[29], bytes[30], bytes[31]]);
    let metadata_end = HEADER_SIZE + metadata_size as usize;

    if bytes.len() < metadata_end {
        return Err(format!(
            "Save file claims {} bytes of metadata but only {} bytes remain",
            metadata_size,
            bytes.len() - HEADER_SIZE,
        ));
    }

    let metadata_bytes = &bytes[HEADER_SIZE..metadata_end];
    let payload = &bytes[metadata_end..];

    let computed = xxh32(payload, XXHASH_SEED);
    if computed != checksum {
        return Err(format!(
            "Save file is corrupted: checksum mismatch \
             (expected {:#010X}, got {:#010X}).",
            checksum, computed,
        ));
    }

    let metadata = if metadata_size > 0 {
        match SaveMetadata::decode(metadata_bytes) {
            Ok(m) => Some(m),
            Err(e) => {
                eprintln!("Warning: failed to decode save metadata: {e}");
                None
            }
        }
    } else {
        None
    };

    Ok(UnwrapResult::WithHeader {
        header: FileHeader {
            format_version,
            flags,
            timestamp,
            uncompressed_size,
            checksum,
            metadata_size,
        },
        metadata,
        payload,
    })
}

/// Read only the metadata from a save file without decoding the full payload.
pub fn read_metadata_only(bytes: &[u8]) -> Result<Option<SaveMetadata>, String> {
    match unwrap_header(bytes)? {
        UnwrapResult::WithHeader { metadata, .. } => Ok(metadata),
        UnwrapResult::Legacy(_) => Ok(None),
    }
}

#[cfg(test)]
#[path = "file_header_tests.rs"]
mod file_header_tests;
