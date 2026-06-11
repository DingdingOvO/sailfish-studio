//! Canvas 2D block renderer — produces a command list that can be replayed
//! on an HTML5 Canvas or inspected in tests.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::layout::{BlockInfo, BlockLayout};

// ── Category colours ─────────────────────────────────────────────────────────

/// Map from category name to hex colour string (same palette as Scratch).
pub const CATEGORY_COLORS: &[(&str, &str)] = &[
    ("motion", "#4C97FF"),
    ("looks", "#9966FF"),
    ("sound", "#CF63CF"),
    ("events", "#FFBF00"),
    ("control", "#FFAB19"),
    ("sensing", "#5CB1D6"),
    ("operators", "#59C059"),
    ("variables", "#FF8C1A"),
    ("pen", "#0fBD8C"),
];

/// Build a `HashMap` from the category colour list.
pub fn category_color_map() -> HashMap<&'static str, &'static str> {
    CATEGORY_COLORS.iter().copied().collect()
}

/// Look up the colour for a category. Returns a grey fallback for unknown.
pub fn color_for_category(category: &str) -> &'static str {
    CATEGORY_COLORS
        .iter()
        .find(|(cat, _)| *cat == category)
        .map(|(_, color)| *color)
        .unwrap_or("#AAAAAA")
}

// ── Render primitives ────────────────────────────────────────────────────────

/// A single drawing command emitted by the renderer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RenderCommand {
    /// Rounded rectangle background.
    RoundedRect {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        radius: f64,
        color: String,
    },
    /// Top connector notch.
    Notch {
        x: f64,
        y: f64,
        width: f64,
        depth: f64,
    },
    /// Bottom connector bump.
    Bump {
        x: f64,
        y: f64,
        width: f64,
        depth: f64,
    },
    /// Label text.
    Text {
        x: f64,
        y: f64,
        content: String,
        color: String,
    },
    /// Input value slot (rounded-rect placeholder).
    InputSlot {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    },
}

// ── Render context ───────────────────────────────────────────────────────────

/// Collects render commands without needing an actual canvas.
#[derive(Debug, Clone, Default)]
pub struct RenderContext {
    commands: Vec<RenderCommand>,
}

impl RenderContext {
    /// Create an empty context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Push a single command.
    pub fn push(&mut self, cmd: RenderCommand) {
        self.commands.push(cmd);
    }

    /// Return the full command list.
    pub fn commands(&self) -> &[RenderCommand] {
        &self.commands
    }

    /// Count commands of a specific variant name.
    pub fn count_commands_of_type(&self, name: &str) -> usize {
        self.commands
            .iter()
            .filter(|c| match c {
                RenderCommand::RoundedRect { .. } => name == "RoundedRect",
                RenderCommand::Notch { .. } => name == "Notch",
                RenderCommand::Bump { .. } => name == "Bump",
                RenderCommand::Text { .. } => name == "Text",
                RenderCommand::InputSlot { .. } => name == "InputSlot",
            })
            .count()
    }

    /// Clear all commands.
    pub fn clear(&mut self) {
        self.commands.clear();
    }
}

// ── Block renderer ───────────────────────────────────────────────────────────

/// Stateless renderer that converts block info + layout into draw commands.
pub struct BlockRenderer;

impl BlockRenderer {
    /// Render a single block into the given context.
    pub fn render_block(ctx: &mut RenderContext, layout: &BlockLayout, block: &BlockInfo) {
        let color = color_for_category(&block.category).to_string();

        // 1. Background rounded rectangle.
        ctx.push(RenderCommand::RoundedRect {
            x: layout.x,
            y: layout.y,
            width: layout.width,
            height: layout.height,
            radius: 4.0,
            color: color.clone(),
        });

        // 2. Notch (top connector) — only for statement blocks with has_previous.
        if block.is_statement && block.has_previous {
            ctx.push(RenderCommand::Notch {
                x: layout.x + layout.notch_x,
                y: layout.y + layout.notch_y,
                width: crate::layout::NOTCH_WIDTH,
                depth: crate::layout::NOTCH_DEPTH,
            });
        }

        // 3. Bump (bottom connector) — only for statement blocks with has_next.
        if block.is_statement && block.has_next {
            ctx.push(RenderCommand::Bump {
                x: layout.x + layout.bump_x,
                y: layout.y + layout.bump_y,
                width: crate::layout::BUMP_WIDTH,
                depth: crate::layout::BUMP_DEPTH,
            });
        }

        // 4. Label text — derive from opcode.
        let label = if let Some(idx) = block.opcode.find('_') {
            block.opcode[idx + 1..].to_string()
        } else {
            block.opcode.clone()
        };
        ctx.push(RenderCommand::Text {
            x: layout.x + crate::layout::BLOCK_PADDING,
            y: layout.y + layout.height / 2.0,
            content: label,
            color: "#FFFFFF".to_string(),
        });

        // 5. Input slots.
        let slot_width = 30.0;
        let slot_height = layout.height * 0.55;
        for i in 0..block.inputs_count {
            ctx.push(RenderCommand::InputSlot {
                x: layout.x + layout.width - crate::layout::BLOCK_PADDING - (block.inputs_count - i) as f64 * slot_width,
                y: layout.y + (layout.height - slot_height) / 2.0,
                width: slot_width,
                height: slot_height,
            });
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{compute_layout, BlockInfo};

    #[test]
    fn category_colors_complete() {
        let map = category_color_map();
        assert_eq!(map.get("motion"), Some(&"#4C97FF"));
        assert_eq!(map.get("looks"), Some(&"#9966FF"));
        assert_eq!(map.get("sound"), Some(&"#CF63CF"));
        assert_eq!(map.get("events"), Some(&"#FFBF00"));
        assert_eq!(map.get("control"), Some(&"#FFAB19"));
        assert_eq!(map.get("sensing"), Some(&"#5CB1D6"));
        assert_eq!(map.get("operators"), Some(&"#59C059"));
        assert_eq!(map.get("variables"), Some(&"#FF8C1A"));
        assert_eq!(map.get("pen"), Some(&"#0fBD8C"));
    }

    #[test]
    fn color_for_unknown_category_returns_grey() {
        assert_eq!(color_for_category("nonexistent"), "#AAAAAA");
    }

    #[test]
    fn render_statement_block_generates_all_commands() {
        let block = BlockInfo::statement("motion_movesteps", "motion", 2);
        let layout = compute_layout(&block);
        let mut ctx = RenderContext::new();

        BlockRenderer::render_block(&mut ctx, &layout, &block);

        // Expect: 1 RoundedRect + 1 Notch + 1 Bump + 1 Text + 2 InputSlots = 6
        assert_eq!(ctx.commands().len(), 6, "should generate 6 commands");
        assert_eq!(ctx.count_commands_of_type("RoundedRect"), 1);
        assert_eq!(ctx.count_commands_of_type("Notch"), 1);
        assert_eq!(ctx.count_commands_of_type("Bump"), 1);
        assert_eq!(ctx.count_commands_of_type("Text"), 1);
        assert_eq!(ctx.count_commands_of_type("InputSlot"), 2);
    }

    #[test]
    fn render_reporter_block_no_connectors() {
        let block = BlockInfo::reporter("motion_xposition", "motion", 0);
        let layout = compute_layout(&block);
        let mut ctx = RenderContext::new();

        BlockRenderer::render_block(&mut ctx, &layout, &block);

        assert_eq!(ctx.count_commands_of_type("Notch"), 0, "reporter has no notch");
        assert_eq!(ctx.count_commands_of_type("Bump"), 0, "reporter has no bump");
        assert_eq!(ctx.count_commands_of_type("RoundedRect"), 1);
        assert_eq!(ctx.count_commands_of_type("Text"), 1);
    }

    #[test]
    fn render_block_uses_correct_category_color() {
        let block = BlockInfo::statement("looks_say", "looks", 1);
        let layout = compute_layout(&block);
        let mut ctx = RenderContext::new();

        BlockRenderer::render_block(&mut ctx, &layout, &block);

        // Find the RoundedRect command and check its color.
        let rect = ctx
            .commands()
            .iter()
            .find_map(|c| match c {
                RenderCommand::RoundedRect { color, .. } => Some(color.clone()),
                _ => None,
            })
            .expect("should have a RoundedRect");
        assert_eq!(rect, "#9966FF", "looks category should be purple");
    }

    #[test]
    fn render_block_label_derived_from_opcode() {
        let block = BlockInfo::statement("control_wait", "control", 1);
        let layout = compute_layout(&block);
        let mut ctx = RenderContext::new();

        BlockRenderer::render_block(&mut ctx, &layout, &block);

        let text = ctx
            .commands()
            .iter()
            .find_map(|c| match c {
                RenderCommand::Text { content, .. } => Some(content.clone()),
                _ => None,
            })
            .expect("should have Text command");
        assert_eq!(text, "wait", "label should be derived from opcode after '_'");
    }

    #[test]
    fn render_context_clear_works() {
        let mut ctx = RenderContext::new();
        ctx.push(RenderCommand::Notch {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            depth: 2.0,
        });
        assert!(!ctx.commands().is_empty());
        ctx.clear();
        assert!(ctx.commands().is_empty());
    }
}
