//! Unit tests for the undo/redo system.

#[cfg(test)]
mod tests {
    use crate::grid::{RoadType, ZoneType};
    use crate::undo_redo::history::ActionHistory;
    use crate::undo_redo::types::{CityAction, MAX_HISTORY};

    #[test]
    fn test_action_history_push_and_undo() {
        let mut history = ActionHistory::default();
        let action = CityAction::PlaceGridRoad {
            x: 10,
            y: 20,
            road_type: RoadType::Local,
            cost: 10.0,
        };
        history.push(action);
        assert_eq!(history.undo_stack.len(), 1);
        assert!(history.redo_stack.is_empty());

        let undone = history.pop_undo();
        assert!(undone.is_some());
        assert!(history.undo_stack.is_empty());
    }

    #[test]
    fn test_push_clears_redo_stack() {
        let mut history = ActionHistory::default();
        // Push and undo to get something in the redo stack
        history.push(CityAction::PlaceGridRoad {
            x: 10,
            y: 20,
            road_type: RoadType::Local,
            cost: 10.0,
        });
        let action = history.pop_undo().unwrap();
        history.push_redo(action);
        assert_eq!(history.redo_stack.len(), 1);

        // New action should clear redo
        history.push(CityAction::PlaceGridRoad {
            x: 5,
            y: 5,
            road_type: RoadType::Avenue,
            cost: 20.0,
        });
        assert!(history.redo_stack.is_empty());
    }

    #[test]
    fn test_max_history_limit() {
        let mut history = ActionHistory::default();
        for i in 0..150 {
            history.push(CityAction::PlaceGridRoad {
                x: i,
                y: 0,
                road_type: RoadType::Local,
                cost: 10.0,
            });
        }
        assert_eq!(history.undo_stack.len(), MAX_HISTORY);
    }

    #[test]
    fn test_composite_action() {
        let mut history = ActionHistory::default();
        let composite = CityAction::Composite(vec![
            CityAction::PlaceGridRoad {
                x: 10,
                y: 10,
                road_type: RoadType::Local,
                cost: 10.0,
            },
            CityAction::PlaceGridRoad {
                x: 11,
                y: 10,
                road_type: RoadType::Local,
                cost: 10.0,
            },
        ]);
        history.push(composite);
        assert_eq!(history.undo_stack.len(), 1);
    }

    #[test]
    fn test_can_undo_can_redo() {
        let mut history = ActionHistory::default();
        assert!(!history.can_undo());
        assert!(!history.can_redo());

        history.push(CityAction::PlaceGridRoad {
            x: 0,
            y: 0,
            road_type: RoadType::Local,
            cost: 10.0,
        });
        assert!(history.can_undo());
        assert!(!history.can_redo());

        let action = history.pop_undo().unwrap();
        history.push_redo(action);
        assert!(!history.can_undo());
        assert!(history.can_redo());
    }

    #[test]
    fn test_push_undo_no_clear_preserves_redo() {
        let mut history = ActionHistory::default();
        history.push_redo(CityAction::PlaceGridRoad {
            x: 0,
            y: 0,
            road_type: RoadType::Local,
            cost: 10.0,
        });
        history.push_undo_no_clear(CityAction::PlaceGridRoad {
            x: 1,
            y: 1,
            road_type: RoadType::Avenue,
            cost: 20.0,
        });
        assert!(history.can_undo());
        assert!(history.can_redo());
    }

    #[test]
    fn test_place_zone_action() {
        let mut history = ActionHistory::default();
        let action = CityAction::PlaceZone {
            cells: vec![
                (10, 10, ZoneType::ResidentialLow),
                (10, 11, ZoneType::ResidentialLow),
            ],
            cost: 10.0,
        };
        history.push(action);
        assert_eq!(history.undo_stack.len(), 1);
    }
}
