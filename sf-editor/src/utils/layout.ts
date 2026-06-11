/**
 * Layout computation utilities for the Sailfish Studio block editor.
 * Handles block sizing, positioning, and hit testing.
 */

/** Standard block height in pixels */
export const BLOCK_HEIGHT = 40;

/** Horizontal padding inside blocks in pixels */
export const BLOCK_PADDING = 12;

/** Distance within which blocks snap together, in pixels */
export const SNAP_DISTANCE = 20;

/** Approximate character width for block label measurement */
const CHAR_WIDTH = 8;

/** Width of each input slot in a block */
const INPUT_SLOT_WIDTH = 60;

/**
 * Compute the width of a block based on its label text and number of input slots.
 */
export function computeBlockWidth(label: string, inputCount: number): number {
  const labelWidth = label.length * CHAR_WIDTH;
  const inputSlotsWidth = inputCount * INPUT_SLOT_WIDTH;
  const totalPadding = BLOCK_PADDING * 2;
  return Math.max(labelWidth + inputSlotsWidth + totalPadding, 60);
}

/**
 * Compute the position of a block in a vertical stack.
 * Each block is placed at x=0 and y is based on index * BLOCK_HEIGHT + offset.
 */
export function computeBlockPosition(index: number, yOffset: number): { x: number; y: number } {
  return {
    x: 0,
    y: index * BLOCK_HEIGHT + yOffset,
  };
}

/**
 * Hit test: determine whether a point (px, py) is inside a block rectangle.
 */
export function isPointInBlock(
  px: number,
  py: number,
  blockX: number,
  blockY: number,
  blockWidth: number,
  blockHeight: number
): boolean {
  return (
    px >= blockX &&
    px <= blockX + blockWidth &&
    py >= blockY &&
    py <= blockY + blockHeight
  );
}
