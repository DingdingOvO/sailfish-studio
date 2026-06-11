import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import {
  PresenceManager,
  UserStatus,
  COLLAB_COLORS,
  MAX_VISIBLE_CURSORS,
  CURSOR_THROTTLE_MS,
  IDLE_TIMEOUT_MS,
  type CursorPosition,
  type EditState,
  type PresenceEventType,
} from './collab-presence';

describe('PresenceManager', () => {
  let pm: PresenceManager;

  beforeEach(() => {
    vi.useFakeTimers();
    pm = new PresenceManager('user1');
  });

  afterEach(() => {
    pm.destroy();
    vi.useRealTimers();
  });

  // ---- Constants ----

  describe('constants', () => {
    it('should have 8 collaboration colors', () => {
      expect(COLLAB_COLORS).toHaveLength(8);
    });

    it('should have MAX_VISIBLE_CURSORS = 8', () => {
      expect(MAX_VISIBLE_CURSORS).toBe(8);
    });

    it('should have CURSOR_THROTTLE_MS = 100', () => {
      expect(CURSOR_THROTTLE_MS).toBe(100);
    });

    it('should have IDLE_TIMEOUT_MS = 120000 (2 minutes)', () => {
      expect(IDLE_TIMEOUT_MS).toBe(2 * 60 * 1000);
    });
  });

  // ---- User Management ----

  describe('user management', () => {
    it('should add a user with a color', () => {
      // Local user (user1) already auto-added in constructor with color index 0
      const user = pm.addUser('user2', 'Alice');
      expect(user.userId).toBe('user2');
      expect(user.username).toBe('Alice');
      expect(user.color).toBe(COLLAB_COLORS[1]); // Index 0 taken by local user
      expect(user.status).toBe(UserStatus.Active);
    });

    it('should cycle through colors for multiple users', () => {
      // Local user (user1) already auto-added with color index 0
      const users = [];
      for (let i = 2; i < 12; i++) {
        users.push(pm.addUser(`user${i}`, `User${i}`));
      }
      // users[0] gets color index 1, users[6] gets color index 7, users[7] cycles to index 0
      expect(users[0].color).toBe(COLLAB_COLORS[1]);
      expect(users[6].color).toBe(COLLAB_COLORS[7]);
      expect(users[7].color).toBe(COLLAB_COLORS[0]); // Cycle
      expect(users[8].color).toBe(COLLAB_COLORS[1]); // Cycle
    });

    it('should remove a user', () => {
      pm.addUser('user2', 'Bob');
      expect(pm.getUserList()).toHaveLength(2);
      const removed = pm.removeUser('user2');
      expect(removed).toBe(true);
      expect(pm.getUserList()).toHaveLength(1);
    });

    it('should return false when removing non-existent user', () => {
      const removed = pm.removeUser('nobody');
      expect(removed).toBe(false);
    });

    it('should remove cursor and edit target when removing user', () => {
      pm.addUser('user2', 'Bob');
      pm.updateRemoteCursor({ userId: 'user2', x: 10, y: 20, area: 'blocks' });
      pm.setEditTarget('user2', 'block1', 'block');
      pm.removeUser('user2');
      // No remote cursors for user2
      const cursors = pm.getRemoteCursors();
      expect(cursors.find((c) => c.userId === 'user2')).toBeUndefined();
      // No edit targets for user2
      const targets = pm.getEditTargets();
      expect(targets.find((t) => t.userId === 'user2')).toBeUndefined();
    });

    it('should get user by ID', () => {
      pm.addUser('user2', 'Bob');
      const user = pm.getUser('user2');
      expect(user).toBeDefined();
      expect(user!.username).toBe('Bob');
    });

    it('should return undefined for unknown user ID', () => {
      expect(pm.getUser('unknown')).toBeUndefined();
    });
  });

  // ---- Cursor Updates ----

  describe('cursor updates', () => {
    it('should update local cursor', () => {
      pm.addUser('user1', 'Alice');
      pm.updateCursor(100, 200, 'blocks');
      const cursor = pm.getRemoteCursors();
      // Local user cursor is not in remote cursors
      expect(cursor).toHaveLength(0);
    });

    it('should update remote cursor', () => {
      pm.addUser('user2', 'Bob');
      pm.updateRemoteCursor({ userId: 'user2', x: 50, y: 75, area: 'stage' });
      const cursors = pm.getRemoteCursors();
      expect(cursors).toHaveLength(1);
      expect(cursors[0].userId).toBe('user2');
      expect(cursors[0].x).toBe(50);
      expect(cursors[0].y).toBe(75);
      expect(cursors[0].area).toBe('stage');
    });

    it('should emit cursor-update event on local cursor', () => {
      const listener = vi.fn();
      pm.on('cursor-update', listener);
      pm.addUser('user1', 'Alice');
      pm.updateCursor(100, 200, 'blocks');
      expect(listener).toHaveBeenCalledOnce();
    });

    it('should emit cursor-update event on remote cursor', () => {
      const listener = vi.fn();
      pm.on('cursor-update', listener);
      pm.addUser('user2', 'Bob');
      pm.updateRemoteCursor({ userId: 'user2', x: 10, y: 20, area: 'other' });
      expect(listener).toHaveBeenCalledOnce();
      expect((listener.mock.calls[0][0] as CursorPosition).userId).toBe('user2');
    });

    it('should respect MAX_VISIBLE_CURSORS for remote cursors', () => {
      // Add more than 8 remote users
      for (let i = 2; i <= 12; i++) {
        pm.addUser(`user${i}`, `User${i}`);
        pm.updateRemoteCursor({ userId: `user${i}`, x: i * 10, y: i * 20, area: 'blocks' });
      }
      const cursors = pm.getRemoteCursors();
      expect(cursors.length).toBeLessThanOrEqual(MAX_VISIBLE_CURSORS);
      expect(cursors).toHaveLength(8);
    });
  });

  // ---- Cursor Throttling ----

  describe('cursor throttling', () => {
    it('should broadcast cursor immediately on first update', () => {
      const listener = vi.fn();
      pm.on('cursor-update', listener);
      pm.addUser('user1', 'Alice');
      pm.updateCursor(10, 20, 'blocks');
      expect(listener).toHaveBeenCalledOnce();
    });

    it('should throttle rapid cursor updates', () => {
      const listener = vi.fn();
      pm.on('cursor-update', listener);
      pm.addUser('user1', 'Alice');

      pm.updateCursor(10, 20, 'blocks'); // Immediate
      expect(listener).toHaveBeenCalledTimes(1);

      pm.updateCursor(15, 25, 'blocks'); // Throttled
      expect(listener).toHaveBeenCalledTimes(1); // Still 1

      // Advance time past throttle
      vi.advanceTimersByTime(CURSOR_THROTTLE_MS);
      expect(listener).toHaveBeenCalledTimes(2); // Pending cursor flushed
    });

    it('should broadcast pending cursor after throttle period', () => {
      const listener = vi.fn();
      pm.on('cursor-update', listener);
      pm.addUser('user1', 'Alice');

      pm.updateCursor(10, 20, 'blocks');
      vi.advanceTimersByTime(50); // Half throttle period

      pm.updateCursor(30, 40, 'blocks'); // Throttled, pending updated
      vi.advanceTimersByTime(CURSOR_THROTTLE_MS); // Full throttle period

      // Should have flushed the pending cursor
      const lastCall = listener.mock.calls[listener.mock.calls.length - 1][0] as CursorPosition;
      expect(lastCall.x).toBe(30);
      expect(lastCall.y).toBe(40);
    });

    it('should only keep the latest pending cursor', () => {
      const listener = vi.fn();
      pm.on('cursor-update', listener);
      pm.addUser('user1', 'Alice');

      pm.updateCursor(10, 20, 'blocks'); // Immediate
      vi.advanceTimersByTime(50);

      pm.updateCursor(30, 40, 'blocks'); // Throttled, pending
      pm.updateCursor(50, 60, 'blocks'); // Throttled, replaces pending

      vi.advanceTimersByTime(CURSOR_THROTTLE_MS);

      // The flushed cursor should be the last one
      const lastCall = listener.mock.calls[listener.mock.calls.length - 1][0] as CursorPosition;
      expect(lastCall.x).toBe(50);
      expect(lastCall.y).toBe(60);
    });

    it('should not throttle if enough time has passed', () => {
      const listener = vi.fn();
      pm.on('cursor-update', listener);
      pm.addUser('user1', 'Alice');

      pm.updateCursor(10, 20, 'blocks');
      vi.advanceTimersByTime(CURSOR_THROTTLE_MS + 1);

      pm.updateCursor(30, 40, 'blocks'); // Should not be throttled
      expect(listener).toHaveBeenCalledTimes(2);
    });
  });

  // ---- Edit Targets ----

  describe('edit targets', () => {
    it('should set an edit target', () => {
      pm.setEditTarget('user1', 'block-abc', 'block');
      const targets = pm.getEditTargets();
      expect(targets).toHaveLength(1);
      expect(targets[0].userId).toBe('user1');
      expect(targets[0].targetId).toBe('block-abc');
      expect(targets[0].type).toBe('block');
    });

    it('should emit edit-start event when setting edit target', () => {
      const listener = vi.fn();
      pm.on('edit-start', listener);
      pm.setEditTarget('user1', 'sprite-1', 'sprite');
      expect(listener).toHaveBeenCalledOnce();
      const data = listener.mock.calls[0][0] as EditState;
      expect(data.targetId).toBe('sprite-1');
      expect(data.type).toBe('sprite');
    });

    it('should clear an edit target', () => {
      pm.setEditTarget('user1', 'costume-1', 'costume');
      pm.clearEditTarget('user1');
      expect(pm.getEditTargets()).toHaveLength(0);
    });

    it('should emit edit-end event when clearing edit target', () => {
      const listener = vi.fn();
      pm.on('edit-end', listener);
      pm.setEditTarget('user1', 'costume-1', 'costume');
      pm.clearEditTarget('user1');
      expect(listener).toHaveBeenCalledOnce();
    });

    it('should not emit edit-end when clearing non-existent target', () => {
      const listener = vi.fn();
      pm.on('edit-end', listener);
      pm.clearEditTarget('user1');
      expect(listener).not.toHaveBeenCalled();
    });

    it('should support multiple edit targets from different users', () => {
      pm.setEditTarget('user1', 'block-1', 'block');
      pm.setEditTarget('user2', 'sprite-1', 'sprite');
      pm.setEditTarget('user3', 'costume-1', 'costume');
      expect(pm.getEditTargets()).toHaveLength(3);
    });

    it('should overwrite edit target for same user', () => {
      pm.setEditTarget('user1', 'block-1', 'block');
      pm.setEditTarget('user1', 'block-2', 'block');
      expect(pm.getEditTargets()).toHaveLength(1);
      expect(pm.getEditTargets()[0].targetId).toBe('block-2');
    });
  });

  // ---- User Status ----

  describe('user status', () => {
    it('should start with Active status', () => {
      const user = pm.addUser('user2', 'Bob');
      expect(user.status).toBe(UserStatus.Active);
    });

    it('should transition from Active to Idle', () => {
      pm.addUser('user2', 'Bob');
      pm.updateUserStatus('user2', UserStatus.Idle);
      const user = pm.getUser('user2');
      expect(user!.status).toBe(UserStatus.Idle);
    });

    it('should transition from Idle to Offline', () => {
      pm.addUser('user2', 'Bob');
      pm.updateUserStatus('user2', UserStatus.Idle);
      pm.updateUserStatus('user2', UserStatus.Offline);
      const user = pm.getUser('user2');
      expect(user!.status).toBe(UserStatus.Offline);
    });

    it('should transition from Active to Offline directly', () => {
      pm.addUser('user2', 'Bob');
      pm.updateUserStatus('user2', UserStatus.Offline);
      const user = pm.getUser('user2');
      expect(user!.status).toBe(UserStatus.Offline);
    });

    it('should emit user-status-change on status transition', () => {
      const listener = vi.fn();
      pm.on('user-status-change', listener);
      pm.addUser('user2', 'Bob');
      pm.updateUserStatus('user2', UserStatus.Idle);
      expect(listener).toHaveBeenCalledOnce();
      expect(listener.mock.calls[0][0]).toEqual({
        userId: 'user2',
        oldStatus: UserStatus.Active,
        newStatus: UserStatus.Idle,
      });
    });

    it('should not emit user-status-change when status unchanged', () => {
      const listener = vi.fn();
      pm.on('user-status-change', listener);
      pm.addUser('user2', 'Bob');
      pm.updateUserStatus('user2', UserStatus.Active); // Same status
      expect(listener).not.toHaveBeenCalled();
    });

    it('should not change status for unknown user', () => {
      const listener = vi.fn();
      pm.on('user-status-change', listener);
      pm.updateUserStatus('unknown', UserStatus.Idle);
      expect(listener).not.toHaveBeenCalled();
    });

    it('should automatically transition idle users', () => {
      pm.addUser('user2', 'Bob');
      // Advance time past idle timeout
      vi.advanceTimersByTime(IDLE_TIMEOUT_MS + 1);
      pm.checkIdleUsers();
      const user = pm.getUser('user2');
      expect(user!.status).toBe(UserStatus.Idle);
    });

    it('should reactivate idle user on activity', () => {
      pm.addUser('user2', 'Bob');
      pm.updateUserStatus('user2', UserStatus.Idle);
      // Simulate activity via remote cursor update
      pm.updateRemoteCursor({ userId: 'user2', x: 1, y: 2, area: 'blocks' });
      const user = pm.getUser('user2');
      expect(user!.status).toBe(UserStatus.Active);
    });
  });

  // ---- User Lists ----

  describe('user lists', () => {
    it('should return all users', () => {
      pm.addUser('user2', 'Bob');
      pm.addUser('user3', 'Carol');
      expect(pm.getUserList()).toHaveLength(3); // user1 + user2 + user3
    });

    it('should return only active users', () => {
      pm.addUser('user2', 'Bob');
      pm.addUser('user3', 'Carol');
      pm.updateUserStatus('user2', UserStatus.Idle);
      pm.updateUserStatus('user3', UserStatus.Offline);
      const active = pm.getActiveUsers();
      expect(active).toHaveLength(1);
      expect(active[0].userId).toBe('user1');
    });

    it('should return empty active list when all idle/offline', () => {
      pm.addUser('user2', 'Bob');
      pm.updateUserStatus('user1', UserStatus.Idle);
      pm.updateUserStatus('user2', UserStatus.Offline);
      expect(pm.getActiveUsers()).toHaveLength(0);
    });
  });

  // ---- Event System ----

  describe('event system', () => {
    it('should register and call event listener', () => {
      const listener = vi.fn();
      pm.on('cursor-update', listener);
      pm.emit('cursor-update', { test: true });
      expect(listener).toHaveBeenCalledOnce();
    });

    it('should remove event listener', () => {
      const listener = vi.fn();
      pm.on('edit-start', listener);
      pm.off('edit-start', listener);
      pm.emit('edit-start', {});
      expect(listener).not.toHaveBeenCalled();
    });

    it('should support multiple listeners for same event', () => {
      const l1 = vi.fn();
      const l2 = vi.fn();
      pm.on('edit-end', l1);
      pm.on('edit-end', l2);
      pm.emit('edit-end', {});
      expect(l1).toHaveBeenCalledOnce();
      expect(l2).toHaveBeenCalledOnce();
    });

    it('should not cross-fire between events', () => {
      const cursorListener = vi.fn();
      const editListener = vi.fn();
      pm.on('cursor-update', cursorListener);
      pm.on('edit-start', editListener);
      pm.emit('cursor-update', {});
      expect(cursorListener).toHaveBeenCalledOnce();
      expect(editListener).not.toHaveBeenCalled();
    });

    it('should swallow errors from listeners', () => {
      const badListener = () => {
        throw new Error('Listener error');
      };
      const goodListener = vi.fn();
      pm.on('user-status-change', badListener);
      pm.on('user-status-change', goodListener);
      // Should not throw
      pm.emit('user-status-change', {});
      expect(goodListener).toHaveBeenCalledOnce();
    });
  });

  // ---- Color Assignment ----

  describe('color assignment', () => {
    it('should assign colors from COLLAB_COLORS', () => {
      const user = pm.addUser('user2', 'Bob');
      expect(COLLAB_COLORS).toContain(user.color);
    });

    it('should assign different colors to consecutive users', () => {
      const u1 = pm.addUser('u1', 'A');
      const u2 = pm.addUser('u2', 'B');
      expect(u1.color).not.toBe(u2.color);
    });

    it('should cycle colors after all 8 are used', () => {
      const colors: string[] = [];
      for (let i = 0; i < 16; i++) {
        const user = pm.addUser(`u${i}`, `User${i}`);
        colors.push(user.color);
      }
      // First 8 and second 8 should match
      for (let i = 0; i < 8; i++) {
        expect(colors[i]).toBe(colors[i + 8]);
      }
    });
  });

  // ---- Destroy / Cleanup ----

  describe('destroy', () => {
    it('should clear all data on destroy', () => {
      pm.addUser('user2', 'Bob');
      pm.updateRemoteCursor({ userId: 'user2', x: 1, y: 2, area: 'blocks' });
      pm.setEditTarget('user1', 'block-1', 'block');
      pm.destroy();
      expect(pm.getUserList()).toHaveLength(0);
      expect(pm.getRemoteCursors()).toHaveLength(0);
      expect(pm.getEditTargets()).toHaveLength(0);
    });

    it('should stop emitting events after destroy', () => {
      const listener = vi.fn();
      pm.on('cursor-update', listener);
      pm.destroy();
      // Emit after destroy should not throw (listeners cleared)
      expect(() => pm.emit('cursor-update', {})).not.toThrow();
    });
  });

  // ---- Integration ----

  describe('integration', () => {
    it('should handle full user lifecycle', () => {
      const statusListener = vi.fn();
      pm.on('user-status-change', statusListener);

      // Add user
      const user = pm.addUser('user2', 'Bob');
      expect(user.status).toBe(UserStatus.Active);

      // User goes idle
      pm.updateUserStatus('user2', UserStatus.Idle);
      expect(pm.getUser('user2')!.status).toBe(UserStatus.Idle);

      // User activity reactivates
      pm.updateRemoteCursor({ userId: 'user2', x: 100, y: 200, area: 'stage' });
      expect(pm.getUser('user2')!.status).toBe(UserStatus.Active);

      // User goes offline
      pm.updateUserStatus('user2', UserStatus.Offline);

      // Remove user
      pm.removeUser('user2');
      expect(pm.getUser('user2')).toBeUndefined();
    });

    it('should handle multiple users editing different targets', () => {
      pm.addUser('user2', 'Bob');
      pm.addUser('user3', 'Carol');

      pm.setEditTarget('user1', 'block-1', 'block');
      pm.setEditTarget('user2', 'sprite-1', 'sprite');
      pm.setEditTarget('user3', 'costume-1', 'costume');

      const targets = pm.getEditTargets();
      expect(targets).toHaveLength(3);

      const types = targets.map((t) => t.type).sort();
      expect(types).toEqual(['block', 'costume', 'sprite']);
    });
  });
});
