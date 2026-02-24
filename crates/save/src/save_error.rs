// ---------------------------------------------------------------------------
// SaveError: proper error types for save/load operations
// ---------------------------------------------------------------------------

use std::fmt;

/// Errors that can occur during save/load operations.
///
/// Replaces ad-hoc `eprintln!` error swallowing with a typed error enum
/// that can be propagated, matched, and displayed to users.
#[derive(Debug)]
pub enum SaveError {
    /// I/O error (file not found, permission denied, disk full, etc.)
    Io(std::io::Error),
    /// Bitcode encoding failed.
    Encode(String),
    /// Bitcode decoding failed (corrupt or invalid save data).
    Decode(String),
    /// Save file version is newer than this build supports.
    VersionMismatch { expected_max: u32, found: u32 },
    /// Save migration failed for a reason other than version mismatch.
    MigrationFailed(String),
    /// No save data was available to load (e.g., no pending bytes).
    NoData,
    /// A required resource was missing from the ECS world.
    MissingResource(String),
}

impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SaveError::Io(e) => write!(f, "I/O error: {e}"),
            SaveError::Encode(msg) => write!(f, "Encoding error: {msg}"),
            SaveError::Decode(msg) => write!(f, "Decoding error: {msg}"),
            SaveError::VersionMismatch {
                expected_max,
                found,
            } => write!(
                f,
                "Version mismatch: save is v{found}, but this build only supports up to v{expected_max}"
            ),
            SaveError::MigrationFailed(msg) => write!(f, "Migration failed: {msg}"),
            SaveError::NoData => write!(f, "No save data available to load"),
            SaveError::MissingResource(name) => {
                write!(f, "Missing required resource: {name}")
            }
        }
    }
}

impl std::error::Error for SaveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SaveError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for SaveError {
    fn from(e: std::io::Error) -> Self {
        SaveError::Io(e)
    }
}

impl From<bitcode::Error> for SaveError {
    fn from(e: bitcode::Error) -> Self {
        SaveError::Decode(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_error_display_io() {
        let err = SaveError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        ));
        let msg = format!("{err}");
        assert!(msg.contains("I/O error"), "got: {msg}");
        assert!(msg.contains("file not found"), "got: {msg}");
    }

    #[test]
    fn test_save_error_display_decode() {
        let err = SaveError::Decode("invalid data".to_string());
        let msg = format!("{err}");
        assert!(msg.contains("Decoding error"), "got: {msg}");
        assert!(msg.contains("invalid data"), "got: {msg}");
    }

    #[test]
    fn test_save_error_display_version_mismatch() {
        let err = SaveError::VersionMismatch {
            expected_max: 32,
            found: 99,
        };
        let msg = format!("{err}");
        assert!(msg.contains("v99"), "got: {msg}");
        assert!(msg.contains("v32"), "got: {msg}");
    }

    #[test]
    fn test_save_error_display_no_data() {
        let err = SaveError::NoData;
        let msg = format!("{err}");
        assert!(msg.contains("No save data"), "got: {msg}");
    }

    #[test]
    fn test_save_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let save_err: SaveError = io_err.into();
        assert!(matches!(save_err, SaveError::Io(_)));
    }

    #[test]
    fn test_save_error_is_error_trait() {
        let err = SaveError::Io(std::io::Error::new(std::io::ErrorKind::Other, "test"));
        // Verify it implements std::error::Error by calling source()
        let source = std::error::Error::source(&err);
        assert!(source.is_some());
    }

    #[test]
    fn test_save_error_debug() {
        let err = SaveError::MigrationFailed("bad data".to_string());
        let debug = format!("{err:?}");
        assert!(debug.contains("MigrationFailed"), "got: {debug}");
    }
}
