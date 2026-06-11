//! Block layout engine — computes the geometric layout of visual blocks.

use serde::{Deserialize, Serialize};

// ── Constants ────────────────────────────────────────────────────────────────

/// Standard block height in pixels.
pub const BLOCK_HEIGHT: f64 = 40.0;
/// Horizontal padding inside a block (on each side).
pub const BLOCK_PADDING: f64 = 12.0;
/// Width of the connector notch.
pub const NOTCH_WIDTH: f64 = 15.0;
/// Depth (height) of the connector notch.
pub const NOTCH_DEPTH: f64 = 4.0;
/// Width of the connector bump.
pub const BUMP_WIDTH: f64 = 15.0;
/// Depth (height) of the connector bump.
pub const BUMP_DEPTH: f64 = 4.0;
/// Estimated width of a single character in the block label.
const CHAR_WIDTH: f64 = 8.0;
/// Width reserved for each input slot.
const INPUT_SLOT_WIDTH: f64 = 30.0;
/// Minimum block width (even with an empty label).
const MIN_BLOCK_WIDTH: f64 = 60.0;

// ── Types ────────────────────────────────────────────────────────────────────

/// Describes the kind and shape of a block for layout purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockInfo {
    /// e.g. "motion_movesteps", "looks_say"
    pub opcode: String,
    /// Number of input slots on this block.
    pub inputs_count: usize,
    /// Category string: "motion", "looks", …
    pub category: String,
    /// Stack / statement block (hat + stack + cap).
    pub is_statement: bool,
    /// Reporter block (oval shape).
    pub is_reporter: bool,
    /// Boolean block (hexagonal shape).
    pub is_boolean: bool,
    /// Has a notch on top (can be connected from above).
    pub has_previous: bool,
    /// Has a bump on bottom (can connect to the next block below).
    pub has_next: bool,
}

impl BlockInfo {
    /// Convenience constructor for a statement block.
    pub fn statement(opcode: &str, category: &str, inputs_count: usize) -> Self {
        Self {
            opcode: opcode.to_string(),
            inputs_count,
            category: category.to_string(),
            is_statement: true,
            is_reporter: false,
            is_boolean: false,
            has_previous: true,
            has_next: true,
        }
    }

    /// Convenience constructor for a reporter block.
    pub fn reporter(opcode: &str, category: &str, inputs_count: usize) -> Self {
        Self {
            opcode: opcode.to_string(),
            inputs_count,
            category: category.to_string(),
            is_statement: false,
            is_reporter: true,
            is_boolean: false,
            has_previous: false,
            has_next: false,
        }
    }

    /// Convenience constructor for a boolean block.
    pub fn boolean(opcode: &str, category: &str, inputs_count: usize) -> Self {
        Self {
            opcode: opcode.to_string(),
            inputs_count,
            category: category.to_string(),
            is_statement: false,
            is_reporter: false,
            is_boolean: true,
            has_previous: false,
            has_next: false,
        }
    }

    /// Derive a label length from the opcode by stripping the category prefix.
    fn label_len(&self) -> usize {
        // Remove "category_" prefix to approximate the displayed label.
        if let Some(idx) = self.opcode.find('_') {
            self.opcode[idx + 1..].len()
        } else {
            self.opcode.len()
        }
    }
}

/// Computed geometric layout of a single block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockLayout {
    /// Left edge x-coordinate.
    pub x: f64,
    /// Top edge y-coordinate.
    pub y: f64,
    /// Total width of the block.
    pub width: f64,
    /// Total height of the block.
    pub height: f64,
    /// X-offset of the connector notch (top).
    pub notch_x: f64,
    /// Y-offset of the connector notch (top).
    pub notch_y: f64,
    /// X-offset of the connector bump (bottom).
    pub bump_x: f64,
    /// Y-offset of the connector bump (bottom).
    pub bump_y: f64,
}

// ── Layout computation ───────────────────────────────────────────────────────

/// Compute the layout for a generic block.
///
/// Dispatches to the appropriate shape-specific function based on
/// `is_reporter` / `is_boolean`, otherwise falls back to the statement layout.
pub fn compute_layout(block: &BlockInfo) -> BlockLayout {
    if block.is_reporter {
        compute_reporter_layout(block)
    } else if block.is_boolean {
        compute_boolean_layout(block)
    } else {
        compute_statement_layout(block)
    }
}

/// Layout for statement / stack blocks (rounded-rect with notch & bump).
fn compute_statement_layout(block: &BlockInfo) -> BlockLayout {
    let label_width = block.label_len() as f64 * CHAR_WIDTH;
    let inputs_width = block.inputs_count as f64 * INPUT_SLOT_WIDTH;
    let width = (BLOCK_PADDING * 2.0 + label_width + inputs_width).max(MIN_BLOCK_WIDTH);
    let height = BLOCK_HEIGHT;

    let notch_x = BLOCK_PADDING;
    let notch_y = 0.0;
    let bump_x = BLOCK_PADDING;
    let bump_y = height - BUMP_DEPTH;

    BlockLayout {
        x: 0.0,
        y: 0.0,
        width,
        height,
        notch_x,
        notch_y,
        bump_x,
        bump_y,
    }
}

/// Layout for reporter blocks (oval shape).
///
/// Reporters are slightly shorter and have an oval aspect ratio.
/// No notch or bump connectors.
pub fn compute_reporter_layout(block: &BlockInfo) -> BlockLayout {
    let label_width = block.label_len() as f64 * CHAR_WIDTH;
    let inputs_width = block.inputs_count as f64 * INPUT_SLOT_WIDTH;
    let width = (BLOCK_PADDING * 2.0 + label_width + inputs_width).max(MIN_BLOCK_WIDTH);
    // Reporter blocks are a bit shorter (oval).
    let height = BLOCK_HEIGHT * 0.75;

    BlockLayout {
        x: 0.0,
        y: 0.0,
        width,
        height,
        notch_x: 0.0,
        notch_y: 0.0,
        bump_x: 0.0,
        bump_y: 0.0,
    }
}

/// Layout for boolean blocks (hexagonal shape).
///
/// Boolean blocks are hexagonal and slightly taller than reporters.
/// No notch or bump connectors.
pub fn compute_boolean_layout(block: &BlockInfo) -> BlockLayout {
    let label_width = block.label_len() as f64 * CHAR_WIDTH;
    let inputs_width = block.inputs_count as f64 * INPUT_SLOT_WIDTH;
    let width = (BLOCK_PADDING * 2.0 + label_width + inputs_width).max(MIN_BLOCK_WIDTH);
    // Boolean blocks have a hexagonal shape; slightly shorter than statement.
    let height = BLOCK_HEIGHT * 0.85;

    BlockLayout {
        x: 0.0,
        y: 0.0,
        width,
        height,
        notch_x: 0.0,
        notch_y: 0.0,
        bump_x: 0.0,
        bump_y: 0.0,
    }
}

/// Calculate the minimum width a block would need given a label length and
/// number of input slots.
pub fn minimum_width(label_chars: usize, input_count: usize) -> f64 {
    (BLOCK_PADDING * 2.0 + label_chars as f64 * CHAR_WIDTH + input_count as f64 * INPUT_SLOT_WIDTH)
        .max(MIN_BLOCK_WIDTH)
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn statement_block_basic_dimensions() {
        let block = BlockInfo::statement("motion_movesteps", "motion", 1);
        let layout = compute_layout(&block);

        // "movesteps" = 9 chars × 8 = 72 + 2×12 padding + 1×30 input = 126
        let expected_width = (BLOCK_PADDING * 2.0 + 9.0 * CHAR_WIDTH + 1.0 * INPUT_SLOT_WIDTH)
            .max(MIN_BLOCK_WIDTH);
        assert!(
            (layout.width - expected_width).abs() < 0.01,
            "width: got {}, expected {}",
            layout.width,
            expected_width
        );
        assert!(
            (layout.height - BLOCK_HEIGHT).abs() < 0.01,
            "height should be BLOCK_HEIGHT"
        );
    }

    #[test]
    fn statement_block_notch_and_bump_positions() {
        let block = BlockInfo::statement("control_wait", "control", 1);
        let layout = compute_layout(&block);

        // Notch at top-left padding area.
        assert!(
            (layout.notch_x - BLOCK_PADDING).abs() < 0.01,
            "notch_x should be BLOCK_PADDING"
        );
        assert!(
            (layout.notch_y).abs() < 0.01,
            "notch_y should be 0"
        );

        // Bump at bottom.
        assert!(
            (layout.bump_x - BLOCK_PADDING).abs() < 0.01,
            "bump_x should be BLOCK_PADDING"
        );
        assert!(
            (layout.bump_y - (BLOCK_HEIGHT - BUMP_DEPTH)).abs() < 0.01,
            "bump_y should be height - BUMP_DEPTH"
        );
    }

    #[test]
    fn reporter_block_has_no_connectors() {
        let block = BlockInfo::reporter("motion_xposition", "motion", 0);
        let layout = compute_layout(&block);

        // Reporter blocks have no notch/bump.
        assert!(
            (layout.notch_x).abs() < 0.01 && (layout.notch_y).abs() < 0.01,
            "reporter should have no notch"
        );
        assert!(
            (layout.bump_x).abs() < 0.01 && (layout.bump_y).abs() < 0.01,
            "reporter should have no bump"
        );
        // Reporter height is 75% of statement height.
        assert!(
            (layout.height - BLOCK_HEIGHT * 0.75).abs() < 0.01,
            "reporter height should be 75% of BLOCK_HEIGHT"
        );
    }

    #[test]
    fn boolean_block_shape() {
        let block = BlockInfo::boolean("operators_gt", "operators", 2);
        let layout = compute_layout(&block);

        // Boolean blocks have no notch/bump.
        assert!(
            (layout.notch_x).abs() < 0.01,
            "boolean should have no notch"
        );
        assert!(
            (layout.bump_x).abs() < 0.01,
            "boolean should have no bump"
        );
        // Boolean height is 85% of statement height.
        assert!(
            (layout.height - BLOCK_HEIGHT * 0.85).abs() < 0.01,
            "boolean height should be 85% of BLOCK_HEIGHT"
        );
    }

    #[test]
    fn minimum_width_enforced() {
        // Very short label, no inputs → should still be at least MIN_BLOCK_WIDTH.
        let block = BlockInfo {
            opcode: "x".to_string(),
            inputs_count: 0,
            category: "motion".to_string(),
            is_statement: true,
            is_reporter: false,
            is_boolean: false,
            has_previous: true,
            has_next: true,
        };
        let layout = compute_layout(&block);
        assert!(
            layout.width >= MIN_BLOCK_WIDTH,
            "width should be at least MIN_BLOCK_WIDTH, got {}",
            layout.width
        );
    }

    #[test]
    fn more_inputs_increase_width() {
        let block1 = BlockInfo::statement("motion_movesteps", "motion", 0);
        let block2 = BlockInfo::statement("motion_movesteps", "motion", 3);
        let layout1 = compute_layout(&block1);
        let layout2 = compute_layout(&block2);
        assert!(
            layout2.width > layout1.width,
            "more inputs should produce wider block: {} vs {}",
            layout2.width,
            layout1.width
        );
    }

    #[test]
    fn minimum_width_utility_function() {
        let w = minimum_width(5, 2);
        let expected = (BLOCK_PADDING * 2.0 + 5.0 * CHAR_WIDTH + 2.0 * INPUT_SLOT_WIDTH)
            .max(MIN_BLOCK_WIDTH);
        assert!(
            (w - expected).abs() < 0.01,
            "minimum_width: got {}, expected {}",
            w,
            expected
        );

        // Edge case: zero label, zero inputs → MIN_BLOCK_WIDTH
        let w_min = minimum_width(0, 0);
        assert!(
            (w_min - MIN_BLOCK_WIDTH).abs() < 0.01,
            "minimum_width(0,0) should be MIN_BLOCK_WIDTH"
        );
    }

    #[test]
    fn compute_layout_dispatches_correctly() {
        let stmt = BlockInfo::statement("motion_movesteps", "motion", 0);
        let rep = BlockInfo::reporter("motion_xposition", "motion", 0);
        let boo = BlockInfo::boolean("operators_gt", "operators", 0);

        let layout_stmt = compute_layout(&stmt);
        let layout_rep = compute_layout(&rep);
        let layout_boo = compute_layout(&boo);

        // Statement is tallest, reporter is shortest.
        assert!(layout_stmt.height > layout_boo.height);
        assert!(layout_boo.height > layout_rep.height);
    }
}
