import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import {
  OfflineSyncManager,
  SyncState,
  MAX_LOG_HOURS,
  MAX_LOG_ENTRIES,
  type LogEntry,
  type Conflict,
} from './offline-sync';

describe('OfflineSyncManager', () => {
  let sync: OfflineSyncManager;

  beforeEach(() => {
    vi.useFakeTimers();
    sync = new OfflineSyncManager();
  });

  afterEach(() => {
    sync.destroy();
    vi.useRealTimers();
  });

  // ---- Constants ----

  describe('constants', () => {
    it('should have MAX_LOG_HOURS = 72', () => {
      expect(MAX_LOG_HOURS).toBe(72);
    });

    it('should have MAX_LOG_ENTRIES = 10000', () => {
      expect(MAX_LOG_ENTRIES).toBe(10000);
    });
  });

  // ---- Operation Recording ----

  describe('operation recording', () => {
    it('should record an operation', () => {
      const entry = sync.recordOperation('block-insert', { blockId: 'b1', x: 10, y: 20 });
      expect(entry.id).toBeDefined();
      expect(entry.operationType).toBe('block-insert');
      expect(entry.operationData).toEqual({ blockId: 'b1', x: 10, y: 20 });
      expect(entry.synced).toBe(false);
      expect(entry.timestamp).toBe(Date.now());
    });

    it('should record multiple operations', () => {
      sync.recordOperation('block-insert', { blockId: 'b1' });
      sync.recordOperation('block-delete', { blockId: 'b2' });
      sync.recordOperation('variable-set', { name: 'x', value: 42 });
      expect(sync.getLogSize()).toBe(3);
    });

    it('should generate unique IDs for entries', () => {
      const e1 = sync.recordOperation('op1', {});
      const e2 = sync.recordOperation('op2', {});
      expect(e1.id).not.toBe(e2.id);
    });

    it('should record operation with complex data', () => {
      const data = {
        blocks: [
          { id: 'b1', opcode: 'motion_gotoxy', x: 100, y: 200 },
          { id: 'b2', opcode: 'looks_show' },
        ],
        metadata: { source: 'user', timestamp: 12345 },
      };
      const entry = sync.recordOperation('block-batch', data);
      expect(entry.operationData).toEqual(data);
    });

    it('should return entries in order', () => {
      sync.recordOperation('op1', {});
      vi.advanceTimersByTime(1);
      sync.recordOperation('op2', {});
      vi.advanceTimersByTime(1);
      sync.recordOperation('op3', {});
      const log = sync.getLog();
      expect(log[0].operationType).toBe('op1');
      expect(log[1].operationType).toBe('op2');
      expect(log[2].operationType).toBe('op3');
    });
  });

  // ---- Unsynced Operations ----

  describe('unsynced operations', () => {
    it('should return all unsynced operations', () => {
      sync.recordOperation('op1', {});
      sync.recordOperation('op2', {});
      const unsynced = sync.getUnsyncedOperations();
      expect(unsynced).toHaveLength(2);
    });

    it('should exclude synced operations', () => {
      const e1 = sync.recordOperation('op1', {});
      sync.recordOperation('op2', {});
      sync.markSynced([e1.id]);
      const unsynced = sync.getUnsyncedOperations();
      expect(unsynced).toHaveLength(1);
      expect(unsynced[0].operationType).toBe('op2');
    });

    it('should return empty array when all synced', () => {
      const e1 = sync.recordOperation('op1', {});
      const e2 = sync.recordOperation('op2', {});
      sync.markSynced([e1.id, e2.id]);
      expect(sync.getUnsyncedOperations()).toHaveLength(0);
    });

    it('should return empty array when log is empty', () => {
      expect(sync.getUnsyncedOperations()).toHaveLength(0);
    });
  });

  // ---- Mark as Synced ----

  describe('mark as synced', () => {
    it('should mark entries as synced', () => {
      const e1 = sync.recordOperation('op1', {});
      sync.recordOperation('op2', {});
      sync.markSynced([e1.id]);
      expect(e1.synced).toBe(true);
    });

    it('should update lastSyncTimestamp after marking synced', () => {
      const e1 = sync.recordOperation('op1', {});
      sync.markSynced([e1.id]);
      expect(sync.getLastSyncTimestamp()).toBe(Date.now());
    });

    it('should not update lastSyncTimestamp with empty list', () => {
      sync.markSynced([]);
      expect(sync.getLastSyncTimestamp()).toBeNull();
    });

    it('should ignore unknown entry IDs', () => {
      sync.recordOperation('op1', {});
      sync.markSynced(['nonexistent']);
      const log = sync.getLog();
      expect(log[0].synced).toBe(false);
    });
  });

  // ---- Offline/Online State ----

  describe('offline/online transitions', () => {
    it('should start in Synced state', () => {
      expect(sync.getState()).toBe(SyncState.Synced);
    });

    it('should transition to Offline', () => {
      sync.goOffline();
      expect(sync.getState()).toBe(SyncState.Offline);
    });

    it('should transition from Offline to Syncing when unsynced ops exist', () => {
      sync.recordOperation('op1', {});
      sync.goOffline();
      sync.goOnline();
      expect(sync.getState()).toBe(SyncState.Syncing);
    });

    it('should transition from Offline to Synced when no unsynced ops', () => {
      sync.goOffline();
      sync.goOnline();
      expect(sync.getState()).toBe(SyncState.Synced);
    });

    it('should transition from Offline to ConflictPending when conflicts exist', () => {
      sync.goOffline();
      // Manually add a conflict
      const local = sync.recordOperation('block-update', { targetId: 'b1' });
      const remote: LogEntry = {
        id: 'remote_1',
        timestamp: Date.now(),
        operationType: 'block-update',
        operationData: { targetId: 'b1' },
        synced: false,
      };
      sync.detectConflicts([local], [remote]);
      sync.goOnline();
      expect(sync.getState()).toBe(SyncState.ConflictPending);
    });
  });

  // ---- Sync Status ----

  describe('sync status', () => {
    it('should return current sync status', () => {
      sync.recordOperation('op1', {});
      const status = sync.getSyncStatus();
      expect(status.state).toBe(SyncState.Synced);
      expect(status.pendingCount).toBe(1);
      expect(status.conflictCount).toBe(0);
    });

    it('should report correct state when offline', () => {
      sync.recordOperation('op1', {});
      sync.goOffline();
      const status = sync.getSyncStatus();
      expect(status.state).toBe(SyncState.Offline);
      expect(status.pendingCount).toBe(1);
    });

    it('should report conflict count', () => {
      const local = sync.recordOperation('block-update', { targetId: 'b1' });
      const remote: LogEntry = {
        id: 'remote_1',
        timestamp: Date.now(),
        operationType: 'block-update',
        operationData: { targetId: 'b1' },
        synced: false,
      };
      sync.detectConflicts([local], [remote]);
      const status = sync.getSyncStatus();
      expect(status.conflictCount).toBe(1);
    });

    it('should report last sync timestamp', () => {
      const e1 = sync.recordOperation('op1', {});
      sync.markSynced([e1.id]);
      const status = sync.getSyncStatus();
      expect(status.lastSyncTimestamp).toBe(Date.now());
    });
  });

  // ---- Conflict Detection ----

  describe('conflict detection', () => {
    it('should detect conflict on same operation type and targetId', () => {
      const local = sync.recordOperation('block-update', { targetId: 'b1', value: 'local' });
      const remote: LogEntry = {
        id: 'remote_1',
        timestamp: Date.now(),
        operationType: 'block-update',
        operationData: { targetId: 'b1', value: 'remote' },
        synced: false,
      };
      const conflicts = sync.detectConflicts([local], [remote]);
      expect(conflicts).toHaveLength(1);
      expect(conflicts[0].localEntry).toBe(local);
      expect(conflicts[0].remoteEntry).toBe(remote);
      expect(conflicts[0].resolved).toBe(false);
    });

    it('should not detect conflict on different targetIds', () => {
      const local = sync.recordOperation('block-update', { targetId: 'b1' });
      const remote: LogEntry = {
        id: 'remote_1',
        timestamp: Date.now(),
        operationType: 'block-update',
        operationData: { targetId: 'b2' },
        synced: false,
      };
      const conflicts = sync.detectConflicts([local], [remote]);
      expect(conflicts).toHaveLength(0);
    });

    it('should detect conflict when same type but no targetId (non-object data)', () => {
      const local = sync.recordOperation('variable-set', 'simple-data');
      const remote: LogEntry = {
        id: 'remote_1',
        timestamp: Date.now(),
        operationType: 'variable-set',
        operationData: 'other-data',
        synced: false,
      };
      const conflicts = sync.detectConflicts([local], [remote]);
      expect(conflicts).toHaveLength(1);
    });

    it('should detect conflict on different operation types with same targetId', () => {
      const local = sync.recordOperation('block-update', { targetId: 'b1' });
      const remote: LogEntry = {
        id: 'remote_1',
        timestamp: Date.now(),
        operationType: 'block-delete',
        operationData: { targetId: 'b1' },
        synced: false,
      };
      const conflicts = sync.detectConflicts([local], [remote]);
      expect(conflicts).toHaveLength(1);
    });

    it('should not detect conflict on different types without targetId', () => {
      const local = sync.recordOperation('block-insert', { x: 10 });
      const remote: LogEntry = {
        id: 'remote_1',
        timestamp: Date.now(),
        operationType: 'variable-set',
        operationData: { name: 'x' },
        synced: false,
      };
      const conflicts = sync.detectConflicts([local], [remote]);
      expect(conflicts).toHaveLength(0);
    });

    it('should detect multiple conflicts', () => {
      const local1 = sync.recordOperation('block-update', { targetId: 'b1' });
      const local2 = sync.recordOperation('block-update', { targetId: 'b2' });
      const remote1: LogEntry = {
        id: 'remote_1',
        timestamp: Date.now(),
        operationType: 'block-update',
        operationData: { targetId: 'b1' },
        synced: false,
      };
      const remote2: LogEntry = {
        id: 'remote_2',
        timestamp: Date.now(),
        operationType: 'block-update',
        operationData: { targetId: 'b2' },
        synced: false,
      };
      const conflicts = sync.detectConflicts([local1, local2], [remote1, remote2]);
      expect(conflicts).toHaveLength(2);
    });

    it('should transition to ConflictPending when conflicts detected', () => {
      const local = sync.recordOperation('block-update', { targetId: 'b1' });
      const remote: LogEntry = {
        id: 'remote_1',
        timestamp: Date.now(),
        operationType: 'block-update',
        operationData: { targetId: 'b1' },
        synced: false,
      };
      sync.detectConflicts([local], [remote]);
      expect(sync.getState()).toBe(SyncState.ConflictPending);
    });
  });

  // ---- Conflict Resolution ----

  describe('conflict resolution', () => {
    it('should resolve with local wins', () => {
      const local = sync.recordOperation('block-update', { targetId: 'b1', value: 'local' });
      const remote: LogEntry = {
        id: 'remote_1',
        timestamp: Date.now(),
        operationType: 'block-update',
        operationData: { targetId: 'b1', value: 'remote' },
        synced: false,
      };
      const [conflict] = sync.detectConflicts([local], [remote]);
      sync.resolveConflict(conflict.id, 'local');
      expect(conflict.resolved).toBe(true);
      expect(conflict.resolution).toBe('local');
    });

    it('should resolve with remote wins', () => {
      const local = sync.recordOperation('block-update', { targetId: 'b1', value: 'local' });
      const remote: LogEntry = {
        id: 'remote_1',
        timestamp: Date.now(),
        operationType: 'block-update',
        operationData: { targetId: 'b1', value: 'remote' },
        synced: false,
      };
      const [conflict] = sync.detectConflicts([local], [remote]);
      sync.resolveConflict(conflict.id, 'remote');
      expect(conflict.resolved).toBe(true);
      expect(conflict.resolution).toBe('remote');
      expect(conflict.localEntry.synced).toBe(true);
    });

    it('should resolve with manual data', () => {
      const local = sync.recordOperation('block-update', { targetId: 'b1', value: 'local' });
      const remote: LogEntry = {
        id: 'remote_1',
        timestamp: Date.now(),
        operationType: 'block-update',
        operationData: { targetId: 'b1', value: 'remote' },
        synced: false,
      };
      const [conflict] = sync.detectConflicts([local], [remote]);
      const manualData = { targetId: 'b1', value: 'merged' };
      sync.resolveConflict(conflict.id, 'manual', manualData);
      expect(conflict.resolved).toBe(true);
      expect(conflict.resolution).toBe('manual');
      expect(conflict.manualData).toEqual(manualData);
    });

    it('should throw on unknown conflict ID', () => {
      expect(() => sync.resolveConflict('nonexistent', 'local')).toThrow(
        'Conflict not found: nonexistent'
      );
    });

    it('should transition to Synced when all conflicts resolved', () => {
      const local = sync.recordOperation('block-update', { targetId: 'b1' });
      const remote: LogEntry = {
        id: 'remote_1',
        timestamp: Date.now(),
        operationType: 'block-update',
        operationData: { targetId: 'b1' },
        synced: false,
      };
      const [conflict] = sync.detectConflicts([local], [remote]);
      expect(sync.getState()).toBe(SyncState.ConflictPending);
      sync.resolveConflict(conflict.id, 'local');
      expect(sync.getState()).toBe(SyncState.Synced);
    });

    it('should stay in ConflictPending when some conflicts unresolved', () => {
      const local1 = sync.recordOperation('block-update', { targetId: 'b1' });
      vi.advanceTimersByTime(1);
      const local2 = sync.recordOperation('block-update', { targetId: 'b2' });
      const remote1: LogEntry = {
        id: 'remote_1',
        timestamp: Date.now(),
        operationType: 'block-update',
        operationData: { targetId: 'b1' },
        synced: false,
      };
      const remote2: LogEntry = {
        id: 'remote_2',
        timestamp: Date.now(),
        operationType: 'block-update',
        operationData: { targetId: 'b2' },
        synced: false,
      };
      const conflicts = sync.detectConflicts([local1, local2], [remote1, remote2]);
      expect(conflicts).toHaveLength(2);
      sync.resolveConflict(conflicts[0].id, 'local');
      expect(sync.getState()).toBe(SyncState.ConflictPending);
    });

    it('should resolve multiple conflicts', () => {
      const local1 = sync.recordOperation('block-update', { targetId: 'b1' });
      vi.advanceTimersByTime(1);
      const local2 = sync.recordOperation('block-update', { targetId: 'b2' });
      const remote1: LogEntry = {
        id: 'remote_1',
        timestamp: Date.now(),
        operationType: 'block-update',
        operationData: { targetId: 'b1' },
        synced: false,
      };
      const remote2: LogEntry = {
        id: 'remote_2',
        timestamp: Date.now(),
        operationType: 'block-update',
        operationData: { targetId: 'b2' },
        synced: false,
      };
      const conflicts = sync.detectConflicts([local1, local2], [remote1, remote2]);
      sync.resolveConflict(conflicts[0].id, 'local');
      sync.resolveConflict(conflicts[1].id, 'remote');
      expect(sync.getState()).toBe(SyncState.Synced);
    });
  });

  // ---- Conflict List ----

  describe('conflict list', () => {
    it('should list all conflicts', () => {
      const local = sync.recordOperation('block-update', { targetId: 'b1' });
      const remote: LogEntry = {
        id: 'remote_1',
        timestamp: Date.now(),
        operationType: 'block-update',
        operationData: { targetId: 'b1' },
        synced: false,
      };
      sync.detectConflicts([local], [remote]);
      expect(sync.getConflictList()).toHaveLength(1);
    });

    it('should get a specific conflict by ID', () => {
      const local = sync.recordOperation('block-update', { targetId: 'b1' });
      const remote: LogEntry = {
        id: 'remote_1',
        timestamp: Date.now(),
        operationType: 'block-update',
        operationData: { targetId: 'b1' },
        synced: false,
      };
      const [conflict] = sync.detectConflicts([local], [remote]);
      const found = sync.getConflict(conflict.id);
      expect(found).toBeDefined();
      expect(found!.id).toBe(conflict.id);
    });

    it('should return undefined for unknown conflict ID', () => {
      expect(sync.getConflict('nonexistent')).toBeUndefined();
    });

    it('should clear all conflicts', () => {
      const local = sync.recordOperation('block-update', { targetId: 'b1' });
      const remote: LogEntry = {
        id: 'remote_1',
        timestamp: Date.now(),
        operationType: 'block-update',
        operationData: { targetId: 'b1' },
        synced: false,
      };
      sync.detectConflicts([local], [remote]);
      sync.clearConflicts();
      expect(sync.getConflictList()).toHaveLength(0);
      expect(sync.getState()).toBe(SyncState.Synced);
    });
  });

  // ---- Expired Log Cleanup ----

  describe('expired log cleanup', () => {
    it('should clear entries older than MAX_LOG_HOURS', () => {
      const now = Date.now();
      // Add an old entry (beyond 72 hours)
      sync.recordOperation('old-op', {});
      // Manually set timestamp to be old
      const log = sync.getLog();
      log[0].timestamp = now - (MAX_LOG_HOURS + 1) * 60 * 60 * 1000;

      // Add a recent entry
      sync.recordOperation('new-op', {});

      const removed = sync.clearExpiredLogs(now);
      expect(removed).toBe(1);
      expect(sync.getLogSize()).toBe(1);
      expect(sync.getLog()[0].operationType).toBe('new-op');
    });

    it('should not clear entries within MAX_LOG_HOURS', () => {
      sync.recordOperation('op1', {});
      const removed = sync.clearExpiredLogs();
      expect(removed).toBe(0);
      expect(sync.getLogSize()).toBe(1);
    });

    it('should clear all entries if all are expired', () => {
      const now = Date.now();
      sync.recordOperation('op1', {});
      sync.recordOperation('op2', {});
      const log = sync.getLog();
      log[0].timestamp = now - (MAX_LOG_HOURS + 1) * 60 * 60 * 1000;
      log[1].timestamp = now - (MAX_LOG_HOURS + 2) * 60 * 60 * 1000;
      const removed = sync.clearExpiredLogs(now);
      expect(removed).toBe(2);
      expect(sync.getLogSize()).toBe(0);
    });

    it('should support custom max hours', () => {
      const now = Date.now();
      sync.recordOperation('op1', {});
      const log = sync.getLog();
      // Set to 2 hours old
      log[0].timestamp = now - 2 * 60 * 60 * 1000;
      // Clear with 1 hour max
      const removed = sync.clearExpiredLogsWithMaxHours(1, now);
      expect(removed).toBe(1);
    });

    it('should not remove synced entries that are still within window', () => {
      const now = Date.now();
      const e = sync.recordOperation('op1', {});
      sync.markSynced([e.id]);
      const removed = sync.clearExpiredLogs(now);
      expect(removed).toBe(0);
      expect(sync.getLogSize()).toBe(1);
    });
  });

  // ---- Max Log Entries ----

  describe('max log entries', () => {
    it('should enforce MAX_LOG_ENTRIES', () => {
      // Add more than max entries
      for (let i = 0; i < MAX_LOG_ENTRIES + 100; i++) {
        sync.recordOperation(`op-${i}`, { index: i });
      }
      expect(sync.getLogSize()).toBeLessThanOrEqual(MAX_LOG_ENTRIES);
    });

    it('should keep the most recent entries when over max', () => {
      for (let i = 0; i < MAX_LOG_ENTRIES + 10; i++) {
        sync.recordOperation(`op-${i}`, { index: i });
      }
      const log = sync.getLog();
      // The oldest entries should have been trimmed
      const firstOp = log[0].operationType;
      expect(firstOp).not.toBe('op-0');
      // The newest entries should be present
      const lastOp = log[log.length - 1].operationType;
      expect(lastOp).toContain('op-');
    });
  });

  // ---- Drain Pending Operations ----

  describe('drain pending operations', () => {
    it('should return all unsynced operations', () => {
      sync.recordOperation('op1', {});
      sync.recordOperation('op2', {});
      const drained = sync.drainPendingOperations();
      expect(drained).toHaveLength(2);
    });

    it('should clear drained operations from log', () => {
      sync.recordOperation('op1', {});
      sync.recordOperation('op2', {});
      sync.drainPendingOperations();
      expect(sync.getUnsyncedOperations()).toHaveLength(0);
    });

    it('should not drain synced operations', () => {
      const e1 = sync.recordOperation('op1', {});
      sync.recordOperation('op2', {});
      sync.markSynced([e1.id]);
      const drained = sync.drainPendingOperations();
      expect(drained).toHaveLength(1);
      expect(drained[0].operationType).toBe('op2');
      // Synced entry should still be in log
      expect(sync.getLogSize()).toBe(1);
    });

    it('should return empty array when no unsynced ops', () => {
      expect(sync.drainPendingOperations()).toHaveLength(0);
    });
  });

  // ---- Complete Sync ----

  describe('complete sync', () => {
    it('should transition to Synced when no conflicts remain', () => {
      sync.recordOperation('op1', {});
      sync.goOffline();
      sync.goOnline();
      expect(sync.getState()).toBe(SyncState.Syncing);
      sync.completeSync();
      expect(sync.getState()).toBe(SyncState.Synced);
    });

    it('should transition to ConflictPending when conflicts exist', () => {
      const local = sync.recordOperation('block-update', { targetId: 'b1' });
      const remote: LogEntry = {
        id: 'remote_1',
        timestamp: Date.now(),
        operationType: 'block-update',
        operationData: { targetId: 'b1' },
        synced: false,
      };
      sync.detectConflicts([local], [remote]);
      sync.completeSync();
      expect(sync.getState()).toBe(SyncState.ConflictPending);
    });
  });

  // ---- Full Offline→Online Cycle ----

  describe('full offline→online cycle', () => {
    it('should handle offline recording and online sync', () => {
      // Start online
      expect(sync.getState()).toBe(SyncState.Synced);

      // Go offline
      sync.goOffline();
      expect(sync.getState()).toBe(SyncState.Offline);

      // Record operations while offline
      const e1 = sync.recordOperation('block-insert', { blockId: 'b1' });
      const e2 = sync.recordOperation('variable-set', { name: 'score', value: 0 });
      expect(sync.getUnsyncedOperations()).toHaveLength(2);

      // Go back online
      sync.goOnline();
      expect(sync.getState()).toBe(SyncState.Syncing);

      // Mark operations as synced
      sync.markSynced([e1.id, e2.id]);
      sync.completeSync();
      expect(sync.getState()).toBe(SyncState.Synced);
      expect(sync.getUnsyncedOperations()).toHaveLength(0);
    });

    it('should handle conflict resolution during sync', () => {
      sync.goOffline();
      const local = sync.recordOperation('block-update', { targetId: 'b1', value: 'local' });
      const remote: LogEntry = {
        id: 'remote_1',
        timestamp: Date.now(),
        operationType: 'block-update',
        operationData: { targetId: 'b1', value: 'remote' },
        synced: false,
      };
      sync.goOnline();
      const [conflict] = sync.detectConflicts([local], [remote]);
      expect(sync.getState()).toBe(SyncState.ConflictPending);

      sync.resolveConflict(conflict.id, 'remote');
      expect(sync.getState()).toBe(SyncState.Synced);
    });
  });

  // ---- Log Access ----

  describe('log access', () => {
    it('should return a copy of the log', () => {
      sync.recordOperation('op1', {});
      const log = sync.getLog();
      log.push({
        id: 'fake',
        timestamp: Date.now(),
        operationType: 'fake',
        operationData: {},
        synced: false,
      });
      expect(sync.getLogSize()).toBe(1); // Original not modified
    });

    it('should clear the entire log', () => {
      sync.recordOperation('op1', {});
      sync.recordOperation('op2', {});
      sync.clearLog();
      expect(sync.getLogSize()).toBe(0);
    });
  });

  // ---- Destroy ----

  describe('destroy', () => {
    it('should clear all data on destroy', () => {
      sync.recordOperation('op1', {});
      const local = sync.recordOperation('block-update', { targetId: 'b1' });
      const remote: LogEntry = {
        id: 'remote_1',
        timestamp: Date.now(),
        operationType: 'block-update',
        operationData: { targetId: 'b1' },
        synced: false,
      };
      sync.detectConflicts([local], [remote]);
      sync.destroy();
      expect(sync.getLogSize()).toBe(0);
      expect(sync.getConflictList()).toHaveLength(0);
    });
  });
});
