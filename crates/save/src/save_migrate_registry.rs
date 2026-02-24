// ---------------------------------------------------------------------------
// Save migration registry: structured, validated migration chain
// ---------------------------------------------------------------------------
//
// Each migration step is a function `fn(&mut SaveData)` that transforms save
// data from version N to version N+1.  The registry validates at construction
// time that the chain is contiguous (no gaps, no duplicates).

use crate::save_error::SaveError;
use crate::save_types::SaveData;

/// A single migration step: transforms save data from `from_version` to `from_version + 1`.
pub(crate) struct MigrationStep {
    pub from_version: u32,
    pub description: &'static str,
    pub migrate_fn: fn(&mut SaveData),
}

/// Result of running the migration chain on a save file.
#[derive(Debug, Clone)]
pub struct MigrationReport {
    /// The version the save file was originally at.
    pub original_version: u32,
    /// The version the save file is now at (should equal `CURRENT_SAVE_VERSION`).
    pub final_version: u32,
    /// Number of migration steps that were applied.
    pub steps_applied: u32,
    /// Descriptions of each step that was applied, in order.
    pub step_descriptions: Vec<&'static str>,
}

/// Registry holding an ordered, validated chain of migration steps.
pub(crate) struct MigrationRegistry {
    steps: Vec<MigrationStep>,
    current_version: u32,
}

impl MigrationRegistry {
    /// Build a registry from a list of migration steps.
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - The chain has gaps (e.g., v2->v3 is missing)
    /// - The chain has duplicate source versions
    /// - The chain doesn't end at `current_version - 1`
    pub fn new(steps: Vec<MigrationStep>, current_version: u32) -> Self {
        // Validate: no duplicates
        let mut seen = std::collections::HashSet::new();
        for step in &steps {
            assert!(
                seen.insert(step.from_version),
                "Duplicate migration step for version {}",
                step.from_version
            );
        }

        // Validate: contiguous chain from 0 to current_version-1
        if current_version > 0 {
            for v in 0..current_version {
                assert!(
                    seen.contains(&v),
                    "Missing migration step from v{} to v{}. The migration chain must be \
                     contiguous from v0 to v{}.",
                    v,
                    v + 1,
                    current_version - 1
                );
            }
        }

        // Sort by from_version for deterministic application order
        let mut steps = steps;
        steps.sort_by_key(|s| s.from_version);

        Self {
            steps,
            current_version,
        }
    }

    /// Apply all necessary migration steps to bring a save from its current
    /// version up to `current_version`.
    ///
    /// # Errors
    ///
    /// Returns `SaveError::VersionMismatch` if the save is from a future version.
    pub fn migrate(&self, save: &mut SaveData) -> Result<MigrationReport, SaveError> {
        let original_version = save.version;

        if save.version > self.current_version {
            return Err(SaveError::VersionMismatch {
                expected_max: self.current_version,
                found: save.version,
            });
        }

        let mut steps_applied = 0u32;
        let mut step_descriptions = Vec::new();

        // Apply each step whose from_version matches the save's current version
        for step in &self.steps {
            if save.version >= self.current_version {
                break;
            }
            if step.from_version == save.version {
                (step.migrate_fn)(save);
                save.version = step.from_version + 1;
                steps_applied += 1;
                step_descriptions.push(step.description);
            }
        }

        debug_assert_eq!(save.version, self.current_version);

        Ok(MigrationReport {
            original_version,
            final_version: save.version,
            steps_applied,
            step_descriptions,
        })
    }

    /// Returns the number of registered migration steps.
    #[cfg(test)]
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    /// Returns the current (target) version.
    #[cfg(test)]
    pub fn current_version(&self) -> u32 {
        self.current_version
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::save_types::CURRENT_SAVE_VERSION;

    #[test]
    fn test_registry_step_count_matches_current_version() {
        let registry = super::super::save_migrate::build_migration_registry();
        assert_eq!(
            registry.step_count() as u32,
            CURRENT_SAVE_VERSION,
            "Registry should have exactly CURRENT_SAVE_VERSION steps \
             (one for each v0->v1, v1->v2, ..., v(N-1)->vN)"
        );
    }

    #[test]
    fn test_registry_target_version() {
        let registry = super::super::save_migrate::build_migration_registry();
        assert_eq!(registry.current_version(), CURRENT_SAVE_VERSION);
    }

    #[test]
    fn test_registry_rejects_future_version() {
        let registry = super::super::save_migrate::build_migration_registry();
        let mut save = crate::save_migrate::tests::minimal_save(CURRENT_SAVE_VERSION + 1);
        let result = registry.migrate(&mut save);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SaveError::VersionMismatch { .. }
        ));
    }

    #[test]
    fn test_registry_noop_for_current_version() {
        let registry = super::super::save_migrate::build_migration_registry();
        let mut save = crate::save_migrate::tests::minimal_save(CURRENT_SAVE_VERSION);
        let report = registry.migrate(&mut save).unwrap();
        assert_eq!(report.original_version, CURRENT_SAVE_VERSION);
        assert_eq!(report.final_version, CURRENT_SAVE_VERSION);
        assert_eq!(report.steps_applied, 0);
        assert!(report.step_descriptions.is_empty());
    }

    #[test]
    fn test_registry_migrates_from_v0() {
        let registry = super::super::save_migrate::build_migration_registry();
        let mut save = crate::save_migrate::tests::minimal_save(0);
        let report = registry.migrate(&mut save).unwrap();
        assert_eq!(report.original_version, 0);
        assert_eq!(report.final_version, CURRENT_SAVE_VERSION);
        assert_eq!(report.steps_applied, CURRENT_SAVE_VERSION);
        assert_eq!(
            report.step_descriptions.len(),
            CURRENT_SAVE_VERSION as usize
        );
    }

    #[test]
    #[should_panic(expected = "Duplicate migration step")]
    fn test_registry_rejects_duplicate_steps() {
        let steps = vec![
            MigrationStep {
                from_version: 0,
                description: "first",
                migrate_fn: |_| {},
            },
            MigrationStep {
                from_version: 0,
                description: "duplicate",
                migrate_fn: |_| {},
            },
        ];
        MigrationRegistry::new(steps, 1);
    }

    #[test]
    #[should_panic(expected = "Missing migration step")]
    fn test_registry_rejects_gaps() {
        let steps = vec![
            MigrationStep {
                from_version: 0,
                description: "v0->v1",
                migrate_fn: |_| {},
            },
            // gap: no v1->v2
            MigrationStep {
                from_version: 2,
                description: "v2->v3",
                migrate_fn: |_| {},
            },
        ];
        MigrationRegistry::new(steps, 3);
    }
}
