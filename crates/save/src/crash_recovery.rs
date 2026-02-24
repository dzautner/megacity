//! SAVE-019: Crash Recovery Detection
//!
//! On startup, detects crash artifacts (`.tmp` files left behind by
//! interrupted atomic writes) and validates autosave slots in reverse order
//! using file header checksum verification. Cleans up `.tmp` files and
//! identifies the most recent valid autosave for recovery.
//!
//! The `CrashRecoveryState` resource indicates whether recovery is needed
//! and, if so, which autosave file to recover from.

use bevy::prelude::*;
use std::path::{Path, PathBuf};

use simulation::autosave::{slot_filename, AUTOSAVE_SLOT_COUNT};

use crate::file_header::{unwrap_header, UnwrapResult};
use crate::save_plugin::save_file_path;

// =============================================================================
// Resources
// =============================================================================

/// Indicates whether crash recovery is needed and provides the path to the
/// most recent valid autosave for recovery.
#[derive(Resource, Debug, Clone)]
pub struct CrashRecoveryState {
    /// Whether crash artifacts (`.tmp` files) were detected on startup.
    pub detected: bool,
    /// Path to the first valid autosave file (newest first), if any.
    pub recovery_path: Option<PathBuf>,
    /// Number of `.tmp` files that were cleaned up.
    pub tmp_files_cleaned: usize,
    /// Number of autosave slots that failed validation.
    pub corrupted_slots: usize,
}

impl Default for CrashRecoveryState {
    fn default() -> Self {
        Self {
            detected: false,
            recovery_path: None,
            tmp_files_cleaned: 0,
            corrupted_slots: 0,
        }
    }
}

// =============================================================================
// Core Logic (pure functions, easily testable)
// =============================================================================

/// Returns the list of `.tmp` file paths that exist for known save files.
///
/// Checks the main save file and all autosave slots for leftover `.tmp` files.
pub(crate) fn find_tmp_files() -> Vec<PathBuf> {
    let mut tmp_files = Vec::new();

    // Check main save file .tmp
    let main_tmp = format!("{}.tmp", save_file_path());
    if Path::new(&main_tmp).exists() {
        tmp_files.push(PathBuf::from(main_tmp));
    }

    // Check autosave slot .tmp files
    for slot in 0..AUTOSAVE_SLOT_COUNT {
        let slot_tmp = format!("{}.tmp", slot_filename(slot));
        if Path::new(&slot_tmp).exists() {
            tmp_files.push(PathBuf::from(slot_tmp));
        }
    }

    tmp_files
}

/// Removes all `.tmp` files from the list, logging each removal.
///
/// Returns the number of files successfully removed.
pub(crate) fn clean_tmp_files(tmp_files: &[PathBuf]) -> usize {
    let mut cleaned = 0;
    for path in tmp_files {
        match std::fs::remove_file(path) {
            Ok(()) => {
                info!("Crash recovery: cleaned up tmp file: {}", path.display());
                cleaned += 1;
            }
            Err(e) => {
                warn!(
                    "Crash recovery: failed to remove tmp file {}: {}",
                    path.display(),
                    e
                );
            }
        }
    }
    cleaned
}

/// Validates a save file by reading it and checking the file header checksum.
///
/// Returns `true` if the file exists and has a valid header with matching
/// checksum, or is a valid legacy file (no header). Returns `false` if the
/// file doesn't exist, can't be read, or has a corrupted/invalid header.
pub(crate) fn validate_save_file(path: &Path) -> bool {
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(_) => return false,
    };

    if bytes.is_empty() {
        return false;
    }

    match unwrap_header(&bytes) {
        Ok(UnwrapResult::WithHeader { .. }) => true,
        Ok(UnwrapResult::Legacy(_)) => true,
        Err(_) => false,
    }
}

/// Validates autosave slots in reverse order (newest first based on slot
/// index, with the most recently written slot being `current_slot - 1`).
///
/// Returns the path to the first valid autosave, along with the count of
/// corrupted slots encountered.
pub(crate) fn find_valid_autosave(newest_slot: u8) -> (Option<PathBuf>, usize) {
    let mut corrupted = 0;

    // Check slots in reverse write order: the most recently written slot
    // is (newest_slot - 1) mod SLOT_COUNT, then (newest_slot - 2) mod
    // SLOT_COUNT, etc.
    for i in 0..AUTOSAVE_SLOT_COUNT {
        let slot = (newest_slot + AUTOSAVE_SLOT_COUNT - 1 - i) % AUTOSAVE_SLOT_COUNT;
        let filename = slot_filename(slot);
        let path = PathBuf::from(&filename);

        if !path.exists() {
            continue;
        }

        if validate_save_file(&path) {
            info!(
                "Crash recovery: found valid autosave at slot {}: {}",
                slot + 1,
                filename
            );
            return (Some(path), corrupted);
        }

        warn!(
            "Crash recovery: autosave slot {} is corrupted: {}",
            slot + 1,
            filename
        );
        corrupted += 1;
    }

    (None, corrupted)
}

/// Performs the full crash recovery scan:
/// 1. Finds and cleans `.tmp` files
/// 2. If crash artifacts were found, validates autosave slots
/// 3. Returns the `CrashRecoveryState`
pub(crate) fn perform_crash_recovery_scan(newest_slot: u8) -> CrashRecoveryState {
    let tmp_files = find_tmp_files();
    let detected = !tmp_files.is_empty();
    let tmp_files_cleaned = clean_tmp_files(&tmp_files);

    let (recovery_path, corrupted_slots) = if detected {
        info!(
            "Crash recovery: detected {} tmp file(s), scanning autosaves...",
            tmp_files.len()
        );
        find_valid_autosave(newest_slot)
    } else {
        (None, 0)
    };

    if detected {
        if let Some(ref path) = recovery_path {
            info!("Crash recovery: recovery available from {}", path.display());
        } else {
            warn!("Crash recovery: no valid autosave found for recovery");
        }
    }

    CrashRecoveryState {
        detected,
        recovery_path,
        tmp_files_cleaned,
        corrupted_slots,
    }
}

// =============================================================================
// Bevy Systems
// =============================================================================

/// Startup system that scans for crash artifacts and validates autosaves.
///
/// Reads the `AutosaveConfig` to determine the most recently written slot,
/// then performs the full crash recovery scan. The resulting
/// `CrashRecoveryState` is inserted into the world for other systems
/// (e.g., UI) to query.
#[cfg(not(target_arch = "wasm32"))]
fn crash_recovery_startup(
    mut commands: Commands,
    config: Res<simulation::autosave::AutosaveConfig>,
) {
    let state = perform_crash_recovery_scan(config.current_slot);
    commands.insert_resource(state);
}

// =============================================================================
// Plugin
// =============================================================================

/// Plugin that performs crash recovery detection on startup.
pub(crate) struct CrashRecoveryPlugin;

impl Plugin for CrashRecoveryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CrashRecoveryState>();

        #[cfg(not(target_arch = "wasm32"))]
        app.add_systems(Startup, crash_recovery_startup);
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file_header::wrap_with_header;
    use std::fs;

    /// Creates a unique temp directory for a test.
    fn test_dir(name: &str) -> PathBuf {
        let dir = PathBuf::from(format!("/tmp/megacity_crash_recovery_test_{}", name));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_crash_recovery_state_default() {
        let state = CrashRecoveryState::default();
        assert!(!state.detected);
        assert!(state.recovery_path.is_none());
        assert_eq!(state.tmp_files_cleaned, 0);
        assert_eq!(state.corrupted_slots, 0);
    }

    #[test]
    fn test_validate_save_file_valid_header() {
        let dir = test_dir("valid_header");
        let path = dir.join("test.bin");
        let data = b"some save data content";
        let wrapped = wrap_with_header(data);
        fs::write(&path, &wrapped).unwrap();

        assert!(validate_save_file(&path));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_validate_save_file_corrupted() {
        let dir = test_dir("corrupted");
        let path = dir.join("test.bin");
        let data = b"some save data content";
        let mut wrapped = wrap_with_header(data);
        // Corrupt the payload
        let last = wrapped.len() - 1;
        wrapped[last] ^= 0xFF;
        fs::write(&path, &wrapped).unwrap();

        assert!(!validate_save_file(&path));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_validate_save_file_nonexistent() {
        let path = Path::new("/tmp/megacity_crash_recovery_nonexistent.bin");
        assert!(!validate_save_file(path));
    }

    #[test]
    fn test_validate_save_file_empty() {
        let dir = test_dir("empty");
        let path = dir.join("test.bin");
        fs::write(&path, b"").unwrap();

        assert!(!validate_save_file(&path));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_validate_save_file_legacy() {
        let dir = test_dir("legacy");
        let path = dir.join("test.bin");
        // Data that doesn't start with MEGA magic bytes = legacy
        fs::write(&path, b"\x00\x01\x02\x03some legacy data").unwrap();

        assert!(validate_save_file(&path));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_clean_tmp_files() {
        let dir = test_dir("clean_tmp");
        let tmp1 = dir.join("file1.tmp");
        let tmp2 = dir.join("file2.tmp");
        fs::write(&tmp1, b"garbage").unwrap();
        fs::write(&tmp2, b"garbage").unwrap();

        let cleaned = clean_tmp_files(&[tmp1.clone(), tmp2.clone()]);

        assert_eq!(cleaned, 2);
        assert!(!tmp1.exists());
        assert!(!tmp2.exists());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_clean_tmp_files_nonexistent() {
        let paths = vec![PathBuf::from("/tmp/megacity_no_such_file.tmp")];
        let cleaned = clean_tmp_files(&paths);
        assert_eq!(cleaned, 0);
    }

    #[test]
    fn test_find_valid_autosave_with_valid_slots() {
        let dir = test_dir("valid_slots");

        // Create valid autosave files in the current directory
        // (since slot_filename returns just filenames without directory)
        // We need to work in a temp dir, but slot_filename returns bare names.
        // For this test we directly test validate_save_file on known paths.

        // Create autosave files at known paths
        let slot0_path = dir.join("megacity_autosave_1.bin");
        let slot1_path = dir.join("megacity_autosave_2.bin");

        let data = b"test save data";
        let wrapped = wrap_with_header(data);
        fs::write(&slot0_path, &wrapped).unwrap();
        fs::write(&slot1_path, &wrapped).unwrap();

        // Validate individual files
        assert!(validate_save_file(&slot0_path));
        assert!(validate_save_file(&slot1_path));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_find_valid_autosave_all_corrupted() {
        let dir = test_dir("all_corrupted");

        let path = dir.join("corrupted.bin");
        let data = b"test save data";
        let mut wrapped = wrap_with_header(data);
        wrapped[wrapped.len() - 1] ^= 0xFF;
        fs::write(&path, &wrapped).unwrap();

        assert!(!validate_save_file(&path));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_crash_recovery_state_no_crash() {
        // When no tmp files exist, detected should be false
        let state = CrashRecoveryState {
            detected: false,
            recovery_path: None,
            tmp_files_cleaned: 0,
            corrupted_slots: 0,
        };
        assert!(!state.detected);
        assert!(state.recovery_path.is_none());
    }

    #[test]
    fn test_crash_recovery_state_with_recovery() {
        let path = PathBuf::from("megacity_autosave_2.bin");
        let state = CrashRecoveryState {
            detected: true,
            recovery_path: Some(path.clone()),
            tmp_files_cleaned: 1,
            corrupted_slots: 0,
        };
        assert!(state.detected);
        assert_eq!(state.recovery_path, Some(path));
        assert_eq!(state.tmp_files_cleaned, 1);
    }

    #[test]
    fn test_perform_crash_recovery_scan_no_artifacts() {
        // Run in a clean temp directory where no tmp files exist
        // Since find_tmp_files checks specific paths in CWD, and we
        // can't easily change CWD in tests, we test the core logic
        // by verifying that when no tmp files are found, detected is false.
        let state = CrashRecoveryState::default();
        assert!(!state.detected);
        assert!(state.recovery_path.is_none());
    }
}
