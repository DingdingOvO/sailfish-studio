//! Drag & snap system — detects when a dragged block is close enough to a
//! candidate snap point and returns the snap result.

use serde::{Deserialize, Serialize};

// ── Constants ────────────────────────────────────────────────────────────────

/// Maximum distance (in pixels) for a block to magnetically snap.
pub const MAGNETIC_SNAP_DISTANCE: f64 = 20.0;

// ── Types ────────────────────────────────────────────────────────────────────

/// Current state of a block being dragged.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DragState {
    pub is_dragging: bool,
    /// Current position of the dragged block's origin.
    pub drag_x: f64,
    pub drag_y: f64,
    /// Offset from the mouse to the block origin (so the block doesn't jump).
    pub offset_x: f64,
    pub offset_y: f64,
    /// ID of the block currently being dragged.
    pub dragged_block_id: Option<String>,
    /// ID of the block we would snap to (if within range).
    pub snap_target_id: Option<String>,
}

impl Default for DragState {
    fn default() -> Self {
        Self {
            is_dragging: false,
            drag_x: 0.0,
            drag_y: 0.0,
            offset_x: 0.0,
            offset_y: 0.0,
            dragged_block_id: None,
            snap_target_id: None,
        }
    }
}

impl DragState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Begin dragging a block.
    pub fn start_drag(&mut self, block_id: &str, mouse_x: f64, mouse_y: f64, block_x: f64, block_y: f64) {
        self.is_dragging = true;
        self.dragged_block_id = Some(block_id.to_string());
        self.offset_x = block_x - mouse_x;
        self.offset_y = block_y - mouse_y;
        self.drag_x = block_x;
        self.drag_y = block_y;
        self.snap_target_id = None;
    }

    /// Update position while dragging.
    pub fn update_drag(&mut self, mouse_x: f64, mouse_y: f64) {
        if self.is_dragging {
            self.drag_x = mouse_x + self.offset_x;
            self.drag_y = mouse_y + self.offset_y;
        }
    }

    /// Stop dragging.
    pub fn end_drag(&mut self) {
        self.is_dragging = false;
        self.dragged_block_id = None;
        self.snap_target_id = None;
    }
}

/// Direction of a snap connector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SnapDirection {
    Top,
    Bottom,
}

/// A potential snap attachment point on an existing block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapPoint {
    pub x: f64,
    pub y: f64,
    pub block_id: String,
    pub direction: SnapDirection,
}

/// The result of a successful snap detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapResult {
    pub target_block_id: String,
    pub snap_offset_x: f64,
    pub snap_offset_y: f64,
}

// ── Snap logic ───────────────────────────────────────────────────────────────

/// Check whether the currently dragged block should snap to any of the
/// candidate snap points. Returns the best (nearest) snap if within range.
pub fn check_snap(drag: &DragState, candidates: &[SnapPoint]) -> Option<SnapResult> {
    if !drag.is_dragging {
        return None;
    }
    find_nearest_snap(drag.drag_x, drag.drag_y, candidates, MAGNETIC_SNAP_DISTANCE)
}

/// Find the nearest snap point within `max_distance`.
///
/// Returns `None` if no candidate is close enough.
pub fn find_nearest_snap(
    x: f64,
    y: f64,
    candidates: &[SnapPoint],
    max_distance: f64,
) -> Option<SnapResult> {
    let mut best: Option<(f64, &SnapPoint)> = None;

    for sp in candidates {
        let dx = x - sp.x;
        let dy = y - sp.y;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist <= max_distance {
            match best {
                Some((best_dist, _)) if dist >= best_dist => {}
                _ => best = Some((dist, sp)),
            }
        }
    }

    best.map(|(_, sp)| SnapResult {
        target_block_id: sp.block_id.clone(),
        snap_offset_x: sp.x - x,
        snap_offset_y: sp.y - y,
    })
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snap_within_range() {
        let candidates = vec![SnapPoint {
            x: 100.0,
            y: 200.0,
            block_id: "block_a".to_string(),
            direction: SnapDirection::Bottom,
        }];
        let result = find_nearest_snap(105.0, 205.0, &candidates, MAGNETIC_SNAP_DISTANCE);
        assert!(result.is_some(), "should find a snap within range");
        let r = result.unwrap();
        assert_eq!(r.target_block_id, "block_a");
        // offset should point from drag position toward snap point
        assert!(
            (r.snap_offset_x - (-5.0)).abs() < 0.01,
            "snap_offset_x: got {}",
            r.snap_offset_x
        );
        assert!(
            (r.snap_offset_y - (-5.0)).abs() < 0.01,
            "snap_offset_y: got {}",
            r.snap_offset_y
        );
    }

    #[test]
    fn snap_out_of_range() {
        let candidates = vec![SnapPoint {
            x: 100.0,
            y: 200.0,
            block_id: "block_a".to_string(),
            direction: SnapDirection::Bottom,
        }];
        let result = find_nearest_snap(150.0, 250.0, &candidates, MAGNETIC_SNAP_DISTANCE);
        assert!(result.is_none(), "should not snap when too far away");
    }

    #[test]
    fn nearest_snap_selected_among_multiple() {
        let candidates = vec![
            SnapPoint {
                x: 50.0,
                y: 50.0,
                block_id: "far".to_string(),
                direction: SnapDirection::Bottom,
            },
            SnapPoint {
                x: 102.0,
                y: 103.0,
                block_id: "near".to_string(),
                direction: SnapDirection::Top,
            },
            SnapPoint {
                x: 200.0,
                y: 200.0,
                block_id: "far2".to_string(),
                direction: SnapDirection::Bottom,
            },
        ];
        let result = find_nearest_snap(100.0, 100.0, &candidates, MAGNETIC_SNAP_DISTANCE);
        assert!(result.is_some());
        assert_eq!(result.unwrap().target_block_id, "near");
    }

    #[test]
    fn check_snap_requires_dragging() {
        let drag = DragState {
            is_dragging: false,
            drag_x: 100.0,
            drag_y: 200.0,
            offset_x: 0.0,
            offset_y: 0.0,
            dragged_block_id: None,
            snap_target_id: None,
        };
        let candidates = vec![SnapPoint {
            x: 100.0,
            y: 200.0,
            block_id: "exact".to_string(),
            direction: SnapDirection::Bottom,
        }];
        assert!(
            check_snap(&drag, &candidates).is_none(),
            "should not snap when not dragging"
        );
    }

    #[test]
    fn drag_state_lifecycle() {
        let mut drag = DragState::new();
        assert!(!drag.is_dragging);

        drag.start_drag("block1", 50.0, 60.0, 100.0, 120.0);
        assert!(drag.is_dragging);
        assert_eq!(drag.dragged_block_id.as_deref(), Some("block1"));
        assert!(
            (drag.offset_x - 50.0).abs() < 0.01,
            "offset_x = drag_origin - mouse"
        );
        assert!(
            (drag.offset_y - 60.0).abs() < 0.01,
            "offset_y = drag_origin - mouse"
        );

        drag.update_drag(60.0, 70.0);
        assert!(
            (drag.drag_x - 110.0).abs() < 0.01,
            "drag_x should update"
        );
        assert!(
            (drag.drag_y - 130.0).abs() < 0.01,
            "drag_y should update"
        );

        drag.end_drag();
        assert!(!drag.is_dragging);
        assert!(drag.dragged_block_id.is_none());
    }

    #[test]
    fn check_snap_with_active_drag() {
        let mut drag = DragState::new();
        drag.start_drag("dragged", 100.0, 100.0, 100.0, 100.0);
        drag.update_drag(105.0, 105.0);

        let candidates = vec![SnapPoint {
            x: 106.0,
            y: 106.0,
            block_id: "target".to_string(),
            direction: SnapDirection::Top,
        }];
        let result = check_snap(&drag, &candidates);
        assert!(result.is_some(), "should snap during active drag");
        assert_eq!(result.unwrap().target_block_id, "target");
    }

    #[test]
    fn empty_candidates_returns_none() {
        let result = find_nearest_snap(100.0, 100.0, &[], MAGNETIC_SNAP_DISTANCE);
        assert!(result.is_none(), "empty candidates should return None");
    }
}
