/**
 * Collaboration Presence service for Sailfish Studio editor.
 * Tracks user cursors, edit targets, and status for real-time collaboration.
 */

/** Predefined collaboration colors (8 colors, cycling) */
export const COLLAB_COLORS = [
  '#FF6B6B', // Red
  '#4ECDC4', // Teal
  '#45B7D1', // Sky Blue
  '#96CEB4', // Sage
  '#FFEAA7', // Yellow
  '#DDA0DD', // Plum
  '#98D8C8', // Mint
  '#F7DC6F', // Gold
] as const;

/** Maximum number of remote cursors to display */
export const MAX_VISIBLE_CURSORS = 8;

/** Throttle interval for cursor broadcasts (ms) */
export const CURSOR_THROTTLE_MS = 100;

/** Idle timeout in milliseconds (2 minutes) */
export const IDLE_TIMEOUT_MS = 2 * 60 * 1000;

/** Cursor area type */
export type CursorArea = 'blocks' | 'stage' | 'other';

/** Cursor position of a remote user */
export interface CursorPosition {
  userId: string;
  x: number;
  y: number;
  area: CursorArea;
}

/** Edit target type */
export type EditTargetType = 'block' | 'sprite' | 'costume';

/** Current edit state of a user */
export interface EditState {
  userId: string;
  targetId: string;
  type: EditTargetType;
}

/** User status */
export enum UserStatus {
  Active = 'active',
  Idle = 'idle',
  Offline = 'offline',
}

/** User info in the presence system */
export interface PresenceUser {
  userId: string;
  username: string;
  color: string;
  status: UserStatus;
  lastActivity: number;
}

/** Event types for the presence system */
export type PresenceEventType =
  | 'cursor-update'
  | 'edit-start'
  | 'edit-end'
  | 'user-status-change';

/** Event listener callback */
export type PresenceEventListener = (data: unknown) => void;

/**
 * PresenceManager tracks user cursors, edit targets, and status
 * for real-time collaboration in the editor.
 */
export class PresenceManager {
  private users: Map<string, PresenceUser> = new Map();
  private cursors: Map<string, CursorPosition> = new Map();
  private editTargets: Map<string, EditState> = new Map();
  private listeners: Map<PresenceEventType, Set<PresenceEventListener>> = new Map();
  private colorIndex = 0;
  private lastCursorBroadcast = 0;
  private pendingCursor: CursorPosition | null = null;
  private throttleTimer: ReturnType<typeof setTimeout> | null = null;
  private idleCheckInterval: ReturnType<typeof setInterval> | null = null;
  private localUserId: string;

  constructor(userId: string) {
    this.localUserId = userId;
    // Auto-add the local user
    this.addUser(userId, userId);
    this.startIdleCheck();
  }

  /** Get the local user ID */
  getLocalUserId(): string {
    return this.localUserId;
  }

  /** Assign a collaboration color (cycling through 8 colors) */
  assignColor(): string {
    const color = COLLAB_COLORS[this.colorIndex % COLLAB_COLORS.length];
    this.colorIndex++;
    return color;
  }

  /** Add a user to the presence system */
  addUser(userId: string, username: string): PresenceUser {
    const color = this.assignColor();
    const user: PresenceUser = {
      userId,
      username,
      color,
      status: UserStatus.Active,
      lastActivity: Date.now(),
    };
    this.users.set(userId, user);
    return user;
  }

  /** Remove a user from the presence system */
  removeUser(userId: string): boolean {
    this.cursors.delete(userId);
    this.editTargets.delete(userId);
    return this.users.delete(userId);
  }

  /** Update cursor position (with throttling) */
  updateCursor(x: number, y: number, area: CursorArea): void {
    const now = Date.now();
    const cursor: CursorPosition = {
      userId: this.localUserId,
      x,
      y,
      area,
    };

    if (now - this.lastCursorBroadcast >= CURSOR_THROTTLE_MS) {
      // Not throttled, broadcast immediately
      this.lastCursorBroadcast = now;
      this.cursors.set(this.localUserId, cursor);
      this.emit('cursor-update', cursor);
      this.pendingCursor = null;
    } else {
      // Throttled, store as pending
      this.pendingCursor = cursor;
      if (!this.throttleTimer) {
        this.throttleTimer = setTimeout(() => {
          this.flushPendingCursor();
        }, CURSOR_THROTTLE_MS - (now - this.lastCursorBroadcast));
      }
    }

    // Update user activity
    this.updateUserActivity(this.localUserId);
  }

  /** Flush any pending cursor update */
  private flushPendingCursor(): void {
    if (this.pendingCursor) {
      this.lastCursorBroadcast = Date.now();
      this.cursors.set(this.localUserId, this.pendingCursor);
      this.emit('cursor-update', this.pendingCursor);
      this.pendingCursor = null;
    }
    this.throttleTimer = null;
  }

  /** Update a remote user's cursor (no throttling for remote updates) */
  updateRemoteCursor(cursor: CursorPosition): void {
    this.cursors.set(cursor.userId, cursor);
    this.emit('cursor-update', cursor);
    this.updateUserActivity(cursor.userId);
  }

  /** Get remote cursors (excluding local user), limited to MAX_VISIBLE_CURSORS */
  getRemoteCursors(): CursorPosition[] {
    const remoteCursors: CursorPosition[] = [];
    for (const [userId, cursor] of this.cursors) {
      if (userId !== this.localUserId) {
        remoteCursors.push(cursor);
      }
    }
    return remoteCursors.slice(0, MAX_VISIBLE_CURSORS);
  }

  /** Set an edit target for a user */
  setEditTarget(userId: string, targetId: string, type: EditTargetType): void {
    const editState: EditState = { userId, targetId, type };
    this.editTargets.set(userId, editState);
    this.emit('edit-start', editState);
    this.updateUserActivity(userId);
  }

  /** Clear an edit target for a user */
  clearEditTarget(userId: string): void {
    const editState = this.editTargets.get(userId);
    if (editState) {
      this.editTargets.delete(userId);
      this.emit('edit-end', editState);
    }
  }

  /** Get all current edit targets */
  getEditTargets(): EditState[] {
    return Array.from(this.editTargets.values());
  }

  /** Update a user's status */
  updateUserStatus(userId: string, status: UserStatus): void {
    const user = this.users.get(userId);
    if (user) {
      const oldStatus = user.status;
      user.status = status;
      if (status === UserStatus.Active) {
        user.lastActivity = Date.now();
      }
      if (oldStatus !== status) {
        this.emit('user-status-change', { userId, oldStatus, newStatus: status });
      }
    }
  }

  /** Update a user's last activity timestamp */
  private updateUserActivity(userId: string): void {
    const user = this.users.get(userId);
    if (user && user.status !== UserStatus.Offline) {
      const wasIdle = user.status === UserStatus.Idle;
      user.lastActivity = Date.now();
      user.status = UserStatus.Active;
      if (wasIdle) {
        this.emit('user-status-change', { userId, oldStatus: UserStatus.Idle, newStatus: UserStatus.Active });
      }
    }
  }

  /** Get the full user list */
  getUserList(): PresenceUser[] {
    return Array.from(this.users.values());
  }

  /** Get only active users */
  getActiveUsers(): PresenceUser[] {
    return Array.from(this.users.values()).filter(
      (u) => u.status === UserStatus.Active
    );
  }

  /** Get a specific user */
  getUser(userId: string): PresenceUser | undefined {
    return this.users.get(userId);
  }

  /** Check idle users and transition them */
  checkIdleUsers(): void {
    const now = Date.now();
    for (const user of this.users.values()) {
      if (user.status === UserStatus.Active && now - user.lastActivity >= IDLE_TIMEOUT_MS) {
        this.updateUserStatus(user.userId, UserStatus.Idle);
      }
    }
  }

  /** Start the idle check interval */
  private startIdleCheck(): void {
    this.idleCheckInterval = setInterval(() => {
      this.checkIdleUsers();
    }, 30000); // Check every 30 seconds
  }

  /** Stop the idle check interval and clean up */
  destroy(): void {
    if (this.idleCheckInterval) {
      clearInterval(this.idleCheckInterval);
      this.idleCheckInterval = null;
    }
    if (this.throttleTimer) {
      clearTimeout(this.throttleTimer);
      this.throttleTimer = null;
    }
    this.users.clear();
    this.cursors.clear();
    this.editTargets.clear();
    this.listeners.clear();
  }

  /** Add an event listener */
  on(event: PresenceEventType, listener: PresenceEventListener): void {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, new Set());
    }
    this.listeners.get(event)!.add(listener);
  }

  /** Remove an event listener */
  off(event: PresenceEventType, listener: PresenceEventListener): void {
    this.listeners.get(event)?.delete(listener);
  }

  /** Emit an event to all registered listeners */
  emit(event: PresenceEventType, data: unknown): void {
    this.listeners.get(event)?.forEach((listener) => {
      try {
        listener(data);
      } catch {
        // Swallow listener errors
      }
    });
  }
}
