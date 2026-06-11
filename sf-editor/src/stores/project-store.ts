/**
 * Project state store for Sailfish Studio editor.
 * Plain TypeScript class with no framework dependencies.
 */

/** Represents a variable in a target */
export interface Variable {
  name: string;
  value: string | number | boolean;
  isCloud?: boolean;
}

/** Represents a block in a target */
export interface Block {
  id: string;
  opcode: string;
  next: string | null;
  parent: string | null;
  inputs: Record<string, unknown>;
  fields: Record<string, unknown>;
}

/** Represents a target (stage or sprite) in the project */
export interface Target {
  name: string;
  isStage: boolean;
  variables: Record<string, Variable>;
  blocks: Record<string, Block>;
}

/** Project state store */
export class ProjectStore {
  projectName: string;
  targets: Target[];
  currentTargetIndex: number;
  dirty: boolean;

  constructor() {
    this.projectName = '';
    this.targets = [];
    this.currentTargetIndex = 0;
    this.dirty = false;
  }

  /** Create a new project with a default stage target */
  createProject(name: string = 'Untitled'): void {
    this.projectName = name;
    this.targets = [
      {
        name: 'Stage',
        isStage: true,
        variables: {},
        blocks: {},
      },
    ];
    this.currentTargetIndex = 0;
    this.dirty = true;
  }

  /** Add a new sprite target to the project */
  addTarget(target: Target): void {
    this.targets.push(target);
    this.dirty = true;
  }

  /** Remove a target by name */
  removeTarget(name: string): boolean {
    const index = this.targets.findIndex((t) => t.name === name);
    if (index === -1) return false;

    // Cannot remove the stage
    if (this.targets[index].isStage) return false;

    this.targets.splice(index, 1);

    // Adjust current target index if needed
    if (this.currentTargetIndex >= this.targets.length) {
      this.currentTargetIndex = this.targets.length - 1;
    }

    this.dirty = true;
    return true;
  }

  /** Set the current target by index */
  setCurrentTarget(index: number): void {
    if (index < 0 || index >= this.targets.length) {
      throw new Error(`Target index out of range: ${index}`);
    }
    this.currentTargetIndex = index;
  }

  /** Get the current target */
  getCurrentTarget(): Target | undefined {
    return this.targets[this.currentTargetIndex];
  }

  /** Mark the project as having unsaved changes */
  markDirty(): void {
    this.dirty = true;
  }

  /** Mark the project as saved (clean) */
  markClean(): void {
    this.dirty = false;
  }

  /** Serialize the project to a JSON string */
  toJSON(): string {
    return JSON.stringify({
      projectName: this.projectName,
      targets: this.targets,
    }, null, 2);
  }

  /** Deserialize a project from a JSON string */
  static fromJSON(json: string): ProjectStore {
    const data = JSON.parse(json);
    const store = new ProjectStore();
    store.projectName = data.projectName ?? '';
    store.targets = data.targets ?? [];
    store.dirty = false;
    return store;
  }
}
