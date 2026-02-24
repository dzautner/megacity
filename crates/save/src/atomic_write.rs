//! Atomic file write using the write-rename pattern.
//!
//! Writes data to a temporary file (`{path}.tmp`), calls `sync_all()` to
//! ensure bytes are flushed to persistent storage, then atomically renames
//! the temp file to the final path.  This guarantees that a crash during
//! write cannot corrupt the existing save file.

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

/// Atomically writes `data` to `path` using the write-rename pattern.
///
/// 1. Write to `{path}.tmp`
/// 2. `sync_all()` to flush to disk
/// 3. `rename` temp to final path (atomic on POSIX; near-atomic on Windows)
///
/// If the process crashes during step 1 or 2, the original file at `path`
/// remains untouched.
pub fn atomic_write(path: &str, data: &[u8]) -> std::io::Result<()> {
    let final_path = Path::new(path);
    let tmp_path = format!("{}.tmp", path);

    // Ensure parent directory exists.
    if let Some(parent) = final_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    // Step 1: Write to temporary file.
    let mut file = File::create(&tmp_path)?;
    file.write_all(data)?;

    // Step 2: Flush to persistent storage.
    file.sync_all()?;

    // Step 3: Atomically rename temp file to final path.
    fs::rename(&tmp_path, final_path)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Helper to create a unique temp directory for each test.
    fn test_dir(name: &str) -> String {
        let dir = format!("/tmp/megacity_atomic_write_test_{}", name);
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_atomic_write_creates_file() {
        let dir = test_dir("creates_file");
        let path = format!("{}/save.bin", dir);

        atomic_write(&path, b"hello world").unwrap();

        let contents = fs::read(&path).unwrap();
        assert_eq!(contents, b"hello world");

        // Temp file should not remain.
        assert!(!Path::new(&format!("{}.tmp", path)).exists());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_atomic_write_overwrites_existing() {
        let dir = test_dir("overwrites");
        let path = format!("{}/save.bin", dir);

        // Write initial content.
        atomic_write(&path, b"version 1").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"version 1");

        // Overwrite with new content.
        atomic_write(&path, b"version 2").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"version 2");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_atomic_write_no_temp_file_left_behind() {
        let dir = test_dir("no_temp");
        let path = format!("{}/save.bin", dir);

        atomic_write(&path, b"data").unwrap();

        let tmp_path = format!("{}.tmp", path);
        assert!(
            !Path::new(&tmp_path).exists(),
            "Temp file should be removed after successful write"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_atomic_write_creates_parent_dirs() {
        let dir = test_dir("parent_dirs");
        let path = format!("{}/nested/deep/save.bin", dir);

        atomic_write(&path, b"nested data").unwrap();

        let contents = fs::read(&path).unwrap();
        assert_eq!(contents, b"nested data");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_atomic_write_preserves_original_on_tmp_present() {
        // Simulate the scenario: if a .tmp file exists from a previous
        // failed write, a new atomic write should succeed and clean up.
        let dir = test_dir("preserves_original");
        let path = format!("{}/save.bin", dir);
        let tmp_path = format!("{}.tmp", path);

        // Create an initial save file.
        fs::write(&path, b"original").unwrap();

        // Simulate a leftover .tmp from a crashed write.
        fs::write(&tmp_path, b"partial garbage").unwrap();

        // A new atomic write should succeed normally.
        atomic_write(&path, b"new save").unwrap();

        assert_eq!(fs::read(&path).unwrap(), b"new save");
        assert!(!Path::new(&tmp_path).exists());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_atomic_write_large_data() {
        let dir = test_dir("large_data");
        let path = format!("{}/save.bin", dir);

        // Write 1MB of data.
        let data = vec![0xAB_u8; 1024 * 1024];
        atomic_write(&path, &data).unwrap();

        let contents = fs::read(&path).unwrap();
        assert_eq!(contents.len(), 1024 * 1024);
        assert!(contents.iter().all(|&b| b == 0xAB));

        let _ = fs::remove_dir_all(&dir);
    }
}
