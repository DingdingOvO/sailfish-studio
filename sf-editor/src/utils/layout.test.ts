import { describe, it, expect } from 'vitest';
import {
  BLOCK_HEIGHT,
  BLOCK_PADDING,
  SNAP_DISTANCE,
  computeBlockWidth,
  computeBlockPosition,
  isPointInBlock,
} from './layout';

describe('constants', () => {
  it('should have BLOCK_HEIGHT = 40', () => {
    expect(BLOCK_HEIGHT).toBe(40);
  });

  it('should have BLOCK_PADDING = 12', () => {
    expect(BLOCK_PADDING).toBe(12);
  });

  it('should have SNAP_DISTANCE = 20', () => {
    expect(SNAP_DISTANCE).toBe(20);
  });
});

describe('computeBlockWidth', () => {
  it('should compute width for a simple label with no inputs', () => {
    // "move" = 4 chars * 8 = 32 + padding 24 = 56 → min 60
    expect(computeBlockWidth('move', 0)).toBe(60);
  });

  it('should compute width for a label with inputs', () => {
    // "move steps" = 10 chars * 8 = 80 + 1 input * 60 + padding 24 = 164
    expect(computeBlockWidth('move steps', 1)).toBe(164);
  });

  it('should compute width for empty label with inputs', () => {
    // 0 chars + 2 inputs * 60 + padding 24 = 144
    expect(computeBlockWidth('', 2)).toBe(144);
  });

  it('should respect minimum width of 60', () => {
    // "a" = 1 char * 8 = 8 + padding 24 = 32 → min 60
    expect(computeBlockWidth('a', 0)).toBe(60);
  });

  it('should compute width for long label', () => {
    // "say Hello World!" = 16 chars * 8 = 128 + padding 24 = 152
    expect(computeBlockWidth('say Hello World!', 0)).toBe(152);
  });

  it('should compute width for label with multiple inputs', () => {
    // "set var to" = 10 * 8 = 80 + 2 * 60 + 24 = 224
    expect(computeBlockWidth('set var to', 2)).toBe(224);
  });
});

describe('computeBlockPosition', () => {
  it('should return position for first block (index 0)', () => {
    expect(computeBlockPosition(0, 0)).toEqual({ x: 0, y: 0 });
  });

  it('should return position for second block (index 1)', () => {
    expect(computeBlockPosition(1, 0)).toEqual({ x: 0, y: 40 });
  });

  it('should include yOffset', () => {
    expect(computeBlockPosition(0, 100)).toEqual({ x: 0, y: 100 });
  });

  it('should combine index and yOffset', () => {
    expect(computeBlockPosition(3, 50)).toEqual({ x: 0, y: 170 });
  });

  it('should handle large index values', () => {
    expect(computeBlockPosition(10, 0)).toEqual({ x: 0, y: 400 });
  });
});

describe('isPointInBlock', () => {
  it('should return true for point inside block', () => {
    expect(isPointInBlock(50, 20, 0, 0, 100, 40)).toBe(true);
  });

  it('should return false for point outside block (right)', () => {
    expect(isPointInBlock(150, 20, 0, 0, 100, 40)).toBe(false);
  });

  it('should return false for point outside block (below)', () => {
    expect(isPointInBlock(50, 50, 0, 0, 100, 40)).toBe(false);
  });

  it('should return true for point on the edge', () => {
    expect(isPointInBlock(0, 0, 0, 0, 100, 40)).toBe(true);
    expect(isPointInBlock(100, 40, 0, 0, 100, 40)).toBe(true);
  });

  it('should work with offset block position', () => {
    expect(isPointInBlock(150, 70, 100, 50, 100, 40)).toBe(true);
    expect(isPointInBlock(90, 70, 100, 50, 100, 40)).toBe(false);
  });

  it('should return false for negative coordinates outside block', () => {
    expect(isPointInBlock(-1, -1, 0, 0, 100, 40)).toBe(false);
  });
});
