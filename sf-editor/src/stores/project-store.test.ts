import { describe, it, expect } from 'vitest';
import { ProjectStore, type Target, type Variable } from './project-store';

describe('ProjectStore', () => {
  it('should initialize with default values', () => {
    const store = new ProjectStore();
    expect(store.projectName).toBe('');
    expect(store.targets).toEqual([]);
    expect(store.currentTargetIndex).toBe(0);
    expect(store.dirty).toBe(false);
  });

  it('should create a project with a default stage', () => {
    const store = new ProjectStore();
    store.createProject('My Project');

    expect(store.projectName).toBe('My Project');
    expect(store.targets).toHaveLength(1);
    expect(store.targets[0].name).toBe('Stage');
    expect(store.targets[0].isStage).toBe(true);
    expect(store.dirty).toBe(true);
  });

  it('should create a project with default name', () => {
    const store = new ProjectStore();
    store.createProject();

    expect(store.projectName).toBe('Untitled');
  });

  it('should add a target to the project', () => {
    const store = new ProjectStore();
    store.createProject();

    const sprite: Target = {
      name: 'Sprite1',
      isStage: false,
      variables: {},
      blocks: {},
    };
    store.addTarget(sprite);

    expect(store.targets).toHaveLength(2);
    expect(store.targets[1].name).toBe('Sprite1');
    expect(store.dirty).toBe(true);
  });

  it('should remove a non-stage target', () => {
    const store = new ProjectStore();
    store.createProject();
    store.addTarget({ name: 'Sprite1', isStage: false, variables: {}, blocks: {} });

    const result = store.removeTarget('Sprite1');

    expect(result).toBe(true);
    expect(store.targets).toHaveLength(1);
  });

  it('should not remove the stage target', () => {
    const store = new ProjectStore();
    store.createProject();

    const result = store.removeTarget('Stage');

    expect(result).toBe(false);
    expect(store.targets).toHaveLength(1);
  });

  it('should return false when removing a non-existent target', () => {
    const store = new ProjectStore();
    store.createProject();

    const result = store.removeTarget('NonExistent');

    expect(result).toBe(false);
  });

  it('should set and get the current target', () => {
    const store = new ProjectStore();
    store.createProject();
    store.addTarget({ name: 'Sprite1', isStage: false, variables: {}, blocks: {} });

    store.setCurrentTarget(1);
    expect(store.currentTargetIndex).toBe(1);
    expect(store.getCurrentTarget()?.name).toBe('Sprite1');
  });

  it('should throw when setting an invalid target index', () => {
    const store = new ProjectStore();
    store.createProject();

    expect(() => store.setCurrentTarget(5)).toThrow('Target index out of range');
    expect(() => store.setCurrentTarget(-1)).toThrow('Target index out of range');
  });

  it('should adjust currentTargetIndex when current target is removed', () => {
    const store = new ProjectStore();
    store.createProject();
    store.addTarget({ name: 'Sprite1', isStage: false, variables: {}, blocks: {} });
    store.addTarget({ name: 'Sprite2', isStage: false, variables: {}, blocks: {} });
    store.setCurrentTarget(2);

    store.removeTarget('Sprite2');

    expect(store.currentTargetIndex).toBe(1);
  });

  it('should mark dirty and clean', () => {
    const store = new ProjectStore();
    store.createProject();
    expect(store.dirty).toBe(true);

    store.markClean();
    expect(store.dirty).toBe(false);

    store.markDirty();
    expect(store.dirty).toBe(true);
  });

  it('should serialize to JSON', () => {
    const store = new ProjectStore();
    store.createProject('Test');
    store.markClean();

    const json = store.toJSON();
    const parsed = JSON.parse(json);

    expect(parsed.projectName).toBe('Test');
    expect(parsed.targets).toHaveLength(1);
    expect(parsed.targets[0].name).toBe('Stage');
  });

  it('should deserialize from JSON', () => {
    const store = new ProjectStore();
    store.createProject('DeserTest');
    store.addTarget({ name: 'Sprite1', isStage: false, variables: {}, blocks: {} });

    const json = store.toJSON();
    const restored = ProjectStore.fromJSON(json);

    expect(restored.projectName).toBe('DeserTest');
    expect(restored.targets).toHaveLength(2);
    expect(restored.dirty).toBe(false);
  });
});
