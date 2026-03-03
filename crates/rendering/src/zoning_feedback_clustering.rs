//! Flood-fill clustering for zone utility feedback.
//!
//! Groups 4-connected cells of the same missing-utility kind into clusters,
//! each with a centroid position and cell count. Used by `zoning_feedback`
//! to aggregate many cells into a single floating badge.

use std::collections::VecDeque;

use simulation::config::{GRID_HEIGHT, GRID_WIDTH};

use crate::colorblind_palette::UtilityIconKind;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A zoned cell that is missing utilities and has no building.
#[derive(Clone, Copy)]
pub(crate) struct ZoneFeedbackCell {
    pub gx: usize,
    pub gy: usize,
    pub kind: UtilityIconKind,
}

/// A connected group of same-kind missing-utility cells.
pub(crate) struct ZoneCluster {
    pub kind: UtilityIconKind,
    pub centroid_gx: f32,
    pub centroid_gy: f32,
    pub cell_count: usize,
}

// ---------------------------------------------------------------------------
// Clustering algorithm
// ---------------------------------------------------------------------------

/// Flood-fill 4-connected clustering of feedback cells by kind.
///
/// Returns clusters sorted by `cell_count` descending (largest first).
pub(crate) fn cluster_cells(cells: &[ZoneFeedbackCell]) -> Vec<ZoneCluster> {
    if cells.is_empty() {
        return Vec::new();
    }

    // Build a sparse lookup: (gx, gy) -> kind
    let mut cell_map: std::collections::HashMap<(usize, usize), UtilityIconKind> =
        std::collections::HashMap::with_capacity(cells.len());
    for c in cells {
        cell_map.insert((c.gx, c.gy), c.kind);
    }

    let mut visited: std::collections::HashSet<(usize, usize)> =
        std::collections::HashSet::with_capacity(cells.len());
    let mut clusters = Vec::new();

    for c in cells {
        let start = (c.gx, c.gy);
        if visited.contains(&start) {
            continue;
        }

        let kind = c.kind;
        let mut queue = VecDeque::new();
        queue.push_back(start);
        visited.insert(start);

        let mut sum_x: f64 = 0.0;
        let mut sum_y: f64 = 0.0;
        let mut count: usize = 0;

        while let Some((cx, cy)) = queue.pop_front() {
            sum_x += cx as f64;
            sum_y += cy as f64;
            count += 1;

            // 4-connected neighbors
            let neighbors = [
                (cx.wrapping_sub(1), cy),
                (cx + 1, cy),
                (cx, cy.wrapping_sub(1)),
                (cx, cy + 1),
            ];
            for n in neighbors {
                if n.0 < GRID_WIDTH && n.1 < GRID_HEIGHT && !visited.contains(&n) {
                    if let Some(&nk) = cell_map.get(&n) {
                        if nk == kind {
                            visited.insert(n);
                            queue.push_back(n);
                        }
                    }
                }
            }
        }

        clusters.push(ZoneCluster {
            kind,
            centroid_gx: (sum_x / count as f64) as f32,
            centroid_gy: (sum_y / count as f64) as f32,
            cell_count: count,
        });
    }

    // Sort by cell_count descending so we can take the largest
    clusters.sort_by(|a, b| b.cell_count.cmp(&a.cell_count));
    clusters
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_empty() {
        let clusters = cluster_cells(&[]);
        assert!(clusters.is_empty());
    }

    #[test]
    fn test_cluster_single_cell() {
        let cells = vec![ZoneFeedbackCell {
            gx: 5,
            gy: 5,
            kind: UtilityIconKind::NoPower,
        }];
        let clusters = cluster_cells(&cells);
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].cell_count, 1);
        assert_eq!(clusters[0].kind, UtilityIconKind::NoPower);
    }

    #[test]
    fn test_cluster_adjacent_same_kind_merge() {
        let cells = vec![
            ZoneFeedbackCell { gx: 5, gy: 5, kind: UtilityIconKind::NoPower },
            ZoneFeedbackCell { gx: 6, gy: 5, kind: UtilityIconKind::NoPower },
            ZoneFeedbackCell { gx: 7, gy: 5, kind: UtilityIconKind::NoPower },
        ];
        let clusters = cluster_cells(&cells);
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].cell_count, 3);
        assert!((clusters[0].centroid_gx - 6.0).abs() < 0.01);
    }

    #[test]
    fn test_cluster_different_kinds_separate() {
        let cells = vec![
            ZoneFeedbackCell { gx: 5, gy: 5, kind: UtilityIconKind::NoPower },
            ZoneFeedbackCell { gx: 6, gy: 5, kind: UtilityIconKind::NoWater },
        ];
        let clusters = cluster_cells(&cells);
        assert_eq!(clusters.len(), 2);
    }

    #[test]
    fn test_cluster_diagonal_not_connected() {
        let cells = vec![
            ZoneFeedbackCell { gx: 5, gy: 5, kind: UtilityIconKind::NoPower },
            ZoneFeedbackCell { gx: 6, gy: 6, kind: UtilityIconKind::NoPower },
        ];
        let clusters = cluster_cells(&cells);
        assert_eq!(clusters.len(), 2);
    }

    #[test]
    fn test_cluster_sorted_by_size() {
        let cells = vec![
            ZoneFeedbackCell { gx: 0, gy: 0, kind: UtilityIconKind::NoPower },
            ZoneFeedbackCell { gx: 10, gy: 10, kind: UtilityIconKind::NoPower },
            ZoneFeedbackCell { gx: 11, gy: 10, kind: UtilityIconKind::NoPower },
            ZoneFeedbackCell { gx: 12, gy: 10, kind: UtilityIconKind::NoPower },
        ];
        let clusters = cluster_cells(&cells);
        assert_eq!(clusters.len(), 2);
        assert_eq!(clusters[0].cell_count, 3);
        assert_eq!(clusters[1].cell_count, 1);
    }

    #[test]
    fn test_cluster_l_shape() {
        // L-shaped cluster should be one connected component
        let cells = vec![
            ZoneFeedbackCell { gx: 0, gy: 0, kind: UtilityIconKind::NoWater },
            ZoneFeedbackCell { gx: 1, gy: 0, kind: UtilityIconKind::NoWater },
            ZoneFeedbackCell { gx: 2, gy: 0, kind: UtilityIconKind::NoWater },
            ZoneFeedbackCell { gx: 0, gy: 1, kind: UtilityIconKind::NoWater },
            ZoneFeedbackCell { gx: 0, gy: 2, kind: UtilityIconKind::NoWater },
        ];
        let clusters = cluster_cells(&cells);
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].cell_count, 5);
    }
}
