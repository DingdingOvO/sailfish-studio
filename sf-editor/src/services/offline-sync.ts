/**
 * Offline Sync service for Sailfish Studio editor.
 * Manages operation logging, conflict detection/resolution, and offline/online sync.
 */

/** Maximum hours to keep log entries */
export const MAX_LOG_HOURS = 72;

/** Maximum number of log entries */
export const MAX_LOG_ENTRIES = 10000;

/** Sync state */
export enum SyncState {
  Synced = 'synced',
  Syncing = 'syncing',
  Offline = 'offline',
  ConflictPending = 'conflict-pending',
}

/** A single log entry */
export interface LogEntry {
  id: string;
  timestamp: number;
  operationType: string;
  operationData: unknown;
  synced: boolean;
}

/** Conflict between local and remote operations */
export interface Conflict {
  id: string;
  localEntry: LogEntry;
  remoteEntry: LogEntry;
  resolved: boolean;
  resolution?: 'local' | 'remote' | 'manual';
  manualData?: unknown;
}

/** Sync status information */
export interface SyncStatus {
  state: SyncState;
  lastSyncTimestamp: number | null;
  pendingCount: number;
  conflictCount: number;
}

/** Generate a unique entry ID */
function generateEntryId(): string {
  return `entry_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`;
}

/**
 * OfflineSyncManager handles operation logging, conflict detection/resolution,
 * and offline/online state transitions for collaborative editing.
 */
export class OfflineSyncManager {
  private log: LogEntry[] = [];
  private conflicts: Map<string, Conflict> = new Map();
  private syncState: SyncState = SyncState.Synced;
  private lastSyncTimestamp: number | null = null;
  private entryCounter = 0;

  /** Record an operation to the local log */
  recordOperation(operationType: string, operationData: unknown): LogEntry {
    const entry: LogEntry = {
      id: generateEntryId(),
      timestamp: Date.now(),
      operationType,
      operationData,
      synced: false,
    };

    this.log.push(entry);
    this.entryCounter++;

    // Enforce max entries
    if (this.log.length > MAX_LOG_ENTRIES) {
      // Remove oldest unsynced entries first, then oldest synced
      this.log = this.log.slice(-MAX_LOG_ENTRIES);
    }

    return entry;
  }

  /** Get all unsynced operations */
  getUnsyncedOperations(): LogEntry[] {
    return this.log.filter((entry) => !entry.synced);
  }

  /** Mark operations as synced by their IDs */
  markSynced(entryIds: string[]): void {
    const idSet = new Set(entryIds);
    for (const entry of this.log) {
      if (idSet.has(entry.id)) {
        entry.synced = true;
      }
    }

    if (entryIds.length > 0) {
      this.lastSyncTimestamp = Date.now();
    }
  }

  /** Transition to offline state */
  goOffline(): void {
    this.syncState = SyncState.Offline;
  }

  /** Transition to online state and attempt sync */
  goOnline(): void {
    if (this.syncState === SyncState.Offline) {
      const unsynced = this.getUnsyncedOperations();
      if (unsynced.length > 0) {
        this.syncState = SyncState.Syncing;
      } else if (this.conflicts.size > 0) {
        this.syncState = SyncState.ConflictPending;
      } else {
        this.syncState = SyncState.Synced;
        this.lastSyncTimestamp = Date.now();
      }
    }
  }

  /** Complete the sync process */
  completeSync(): void {
    if (this.conflicts.size > 0) {
      this.syncState = SyncState.ConflictPending;
    } else {
      this.syncState = SyncState.Synced;
      this.lastSyncTimestamp = Date.now();
    }
  }

  /** Get current sync status */
  getSyncStatus(): SyncStatus {
    return {
      state: this.syncState,
      lastSyncTimestamp: this.lastSyncTimestamp,
      pendingCount: this.getUnsyncedOperations().length,
      conflictCount: this.conflicts.size,
    };
  }

  /** Get the current sync state */
  getState(): SyncState {
    return this.syncState;
  }

  /** Get the last sync timestamp */
  getLastSyncTimestamp(): number | null {
    return this.lastSyncTimestamp;
  }

  /**
   * Detect conflicts between local and remote operations.
   * A conflict occurs when both local and remote have unsynced operations
   * targeting the same resource (matching operationType + overlapping targetId in operationData).
   */
  detectConflicts(localOps: LogEntry[], remoteOps: LogEntry[]): Conflict[] {
    const newConflicts: Conflict[] = [];

    for (const local of localOps) {
      for (const remote of remoteOps) {
        if (this.isConflict(local, remote)) {
          const conflict: Conflict = {
            id: `conflict_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`,
            localEntry: local,
            remoteEntry: remote,
            resolved: false,
          };
          this.conflicts.set(conflict.id, conflict);
          newConflicts.push(conflict);
        }
      }
    }

    if (newConflicts.length > 0) {
      this.syncState = SyncState.ConflictPending;
    }

    return newConflicts;
  }

  /** Check if two operations conflict (same target) */
  private isConflict(local: LogEntry, remote: LogEntry): boolean {
    // Same operation type indicates potential conflict
    if (local.operationType !== remote.operationType) {
      // Also check if they target the same resource by looking at operationData
      const localData = local.operationData as Record<string, unknown> | null;
      const remoteData = remote.operationData as Record<string, unknown> | null;
      if (localData && remoteData && localData.targetId && remoteData.targetId) {
        return localData.targetId === remoteData.targetId;
      }
      return false;
    }

    // Same operation type: check if they target the same resource
    const localData = local.operationData as Record<string, unknown> | null;
    const remoteData = remote.operationData as Record<string, unknown> | null;

    if (localData && remoteData) {
      // If both have targetId, compare them
      if (localData.targetId !== undefined && remoteData.targetId !== undefined) {
        return localData.targetId === remoteData.targetId;
      }
      // If neither has targetId but same type, they conflict
      return true;
    }

    // Same type with non-object data: conflict
    return true;
  }

  /** Resolve a conflict */
  resolveConflict(
    conflictId: string,
    resolution: 'local' | 'remote' | 'manual',
    manualData?: unknown
  ): void {
    const conflict = this.conflicts.get(conflictId);
    if (!conflict) {
      throw new Error(`Conflict not found: ${conflictId}`);
    }

    conflict.resolved = true;
    conflict.resolution = resolution;
    conflict.manualData = manualData;

    switch (resolution) {
      case 'local':
        // Keep local, discard remote
        break;
      case 'remote':
        // Keep remote, mark local as synced (it will be replaced)
        conflict.localEntry.synced = true;
        break;
      case 'manual':
        // Manual resolution with provided data
        conflict.localEntry.synced = true;
        break;
    }

    // Check if all conflicts are resolved
    const unresolved = Array.from(this.conflicts.values()).filter((c) => !c.resolved);
    if (unresolved.length === 0) {
      // All conflicts resolved
      if (this.syncState === SyncState.ConflictPending) {
        this.syncState = SyncState.Synced;
        this.lastSyncTimestamp = Date.now();
      }
    }
  }

  /** Get list of pending (unresolved) conflicts */
  getConflictList(): Conflict[] {
    return Array.from(this.conflicts.values());
  }

  /** Get a specific conflict by ID */
  getConflict(conflictId: string): Conflict | undefined {
    return this.conflicts.get(conflictId);
  }

  /** Clear log entries older than MAX_LOG_HOURS */
  clearExpiredLogs(now?: number): number {
    const cutoff = (now ?? Date.now()) - MAX_LOG_HOURS * 60 * 60 * 1000;
    const before = this.log.length;
    this.log = this.log.filter((entry) => entry.timestamp >= cutoff);
    return before - this.log.length;
  }

  /** Clear expired logs with a custom max hours value */
  clearExpiredLogsWithMaxHours(maxHours: number, now?: number): number {
    const cutoff = (now ?? Date.now()) - maxHours * 60 * 60 * 1000;
    const before = this.log.length;
    this.log = this.log.filter((entry) => entry.timestamp >= cutoff);
    return before - this.log.length;
  }

  /** Get all unsynced operations and clear them from the log */
  drainPendingOperations(): LogEntry[] {
    const unsynced = this.getUnsyncedOperations();
    const unsyncedIds = new Set(unsynced.map((e) => e.id));
    this.log = this.log.filter((entry) => !unsyncedIds.has(entry.id));
    return unsynced;
  }

  /** Get the full log (for debugging/testing) */
  getLog(): LogEntry[] {
    return [...this.log];
  }

  /** Get the number of log entries */
  getLogSize(): number {
    return this.log.length;
  }

  /** Clear the entire log */
  clearLog(): void {
    this.log = [];
  }

  /** Clear all conflicts */
  clearConflicts(): void {
    this.conflicts.clear();
    if (this.syncState === SyncState.ConflictPending) {
      this.syncState = SyncState.Synced;
    }
  }

  /** Clean up resources */
  destroy(): void {
    this.log = [];
    this.conflicts.clear();
  }
}
