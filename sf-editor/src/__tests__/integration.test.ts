/**
 * Integration test: full flow from project creation → add target → set variable → serialize
 */
import { describe, it, expect } from 'vitest';
import { ProjectStore, type Target, type Variable } from '../stores/project-store';
import { categoryColor, hexToRgb, rgbToHex } from '../utils/color';
import { computeBlockWidth, computeBlockPosition, BLOCK_HEIGHT } from '../utils/layout';

describe('Integration: full project lifecycle', () => {
  it('should create a project, add a sprite with variables, and serialize', () => {
    // 1. Create a new project
    const store = new ProjectStore();
    store.createProject('Integration Test');

    expect(store.projectName).toBe('Integration Test');
    expect(store.targets).toHaveLength(1);
    expect(store.dirty).toBe(true);

    // 2. Add a sprite target with blocks and variables
    const sprite: Target = {
      name: 'Cat',
      isStage: false,
      variables: {
        score: { name: 'score', value: 0, isCloud: false },
        lives: { name: 'lives', value: 3, isCloud: false },
      },
      blocks: {
        block1: {
          id: 'block1',
          opcode: 'event_whenflagclicked',
          next: 'block2',
          parent: null,
          inputs: {},
          fields: {},
        },
        block2: {
          id: 'block2',
          opcode: 'motion_gotoxy',
          next: null,
          parent: 'block1',
          inputs: { X: [1, 0], Y: [1, 0] },
          fields: {},
        },
      },
    };

    store.addTarget(sprite);
    expect(store.targets).toHaveLength(2);

    // 3. Switch to the sprite target
    store.setCurrentTarget(1);
    const current = store.getCurrentTarget();
    expect(current?.name).toBe('Cat');
    expect(current?.isStage).toBe(false);

    // 4. Update a variable value
    if (current) {
      (current.variables['score'] as Variable).value = 10;
    }

    // 5. Verify block layout computations using color and layout utils
    const motionColor = categoryColor('motion');
    expect(motionColor).toBe('#4C97FF');

    const { r, g, b } = hexToRgb(motionColor);
    expect(r).toBe(76);
    expect(g).toBe(151);
    expect(b).toBe(255);

    // Compute block widths
    const blockWidth1 = computeBlockWidth('when flag clicked', 0);
    const blockWidth2 = computeBlockWidth('go to x y', 2);
    expect(blockWidth1).toBeGreaterThan(0);
    expect(blockWidth2).toBeGreaterThan(blockWidth1);

    // Compute block positions in a stack
    const pos0 = computeBlockPosition(0, 0);
    const pos1 = computeBlockPosition(1, 0);
    expect(pos0.y).toBe(0);
    expect(pos1.y).toBe(BLOCK_HEIGHT);

    // 6. Serialize the project
    const json = store.toJSON();
    const parsed = JSON.parse(json);

    expect(parsed.projectName).toBe('Integration Test');
    expect(parsed.targets).toHaveLength(2);
    expect(parsed.targets[0].name).toBe('Stage');
    expect(parsed.targets[0].isStage).toBe(true);
    expect(parsed.targets[1].name).toBe('Cat');
    expect(parsed.targets[1].isStage).toBe(false);
    expect(parsed.targets[1].variables.score.value).toBe(10);
    expect(parsed.targets[1].variables.lives.value).toBe(3);
    expect(parsed.targets[1].blocks.block1.opcode).toBe('event_whenflagclicked');
    expect(parsed.targets[1].blocks.block2.opcode).toBe('motion_gotoxy');
  });

  it('should round-trip project through JSON serialization', () => {
    // Create and populate a project
    const original = new ProjectStore();
    original.createProject('Round Trip');
    original.addTarget({
      name: 'Sprite1',
      isStage: false,
      variables: {
        myVar: { name: 'myVar', value: 'hello', isCloud: true },
      },
      blocks: {
        b1: {
          id: 'b1',
          opcode: 'control_forever',
          next: null,
          parent: null,
          inputs: {},
          fields: {},
        },
      },
    });

    // Serialize
    const json = original.toJSON();

    // Deserialize
    const restored = ProjectStore.fromJSON(json);

    expect(restored.projectName).toBe('Round Trip');
    expect(restored.targets).toHaveLength(2);
    expect(restored.targets[1].name).toBe('Sprite1');
    expect(restored.targets[1].variables.myVar.value).toBe('hello');
    expect(restored.targets[1].variables.myVar.isCloud).toBe(true);
    expect(restored.targets[1].blocks.b1.opcode).toBe('control_forever');
    expect(restored.dirty).toBe(false);
  });

  it('should use layout and color utilities together for block rendering', () => {
    // Simulate rendering a stack of blocks from different categories
    const blockStack = [
      { label: 'when flag clicked', category: 'events', inputs: 0 },
      { label: 'go to x y', category: 'motion', inputs: 2 },
      { label: 'say Hello!', category: 'looks', inputs: 1 },
    ];

    const renderedBlocks = blockStack.map((block, index) => {
      const color = categoryColor(block.category);
      const rgb = hexToRgb(color);
      const width = computeBlockWidth(block.label, block.inputs);
      const position = computeBlockPosition(index, 50);

      return {
        ...block,
        color,
        rgb,
        width,
        position,
      };
    });

    // Verify each block has correct properties
    expect(renderedBlocks[0].color).toBe('#FFBF00'); // events
    expect(renderedBlocks[1].color).toBe('#4C97FF'); // motion
    expect(renderedBlocks[2].color).toBe('#9966FF'); // looks

    // Verify layout positions
    expect(renderedBlocks[0].position.y).toBe(50);
    expect(renderedBlocks[1].position.y).toBe(90);
    expect(renderedBlocks[2].position.y).toBe(130);

    // Verify widths are different based on content
    expect(renderedBlocks[1].width).toBeGreaterThan(renderedBlocks[0].width);

    // Verify we can convert back to hex
    for (const block of renderedBlocks) {
      const hexAgain = rgbToHex(block.rgb.r, block.rgb.g, block.rgb.b);
      expect(hexAgain).toBe(block.color.toUpperCase());
    }
  });
});
