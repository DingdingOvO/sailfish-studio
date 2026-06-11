/**
 * Authentication service for Sailfish Studio editor.
 * Handles user login/register, token management, and role-based access.
 */

/** User role in a project */
export type UserRole = 'owner' | 'editor' | 'viewer';

/** Authentication provider */
export enum AuthProvider {
  Local = 'local',
  GitHub = 'github',
  Google = 'google',
}

/** User information */
export interface User {
  id: string;
  username: string;
  email: string;
  avatar_url: string | null;
  role: UserRole;
}

/** Stored token data */
export interface TokenData {
  accessToken: string;
  refreshToken: string;
  expiresAt: number; // Unix timestamp in ms
}

/** Session data persisted in localStorage */
export interface SessionData {
  user: User;
  tokens: TokenData;
  provider: AuthProvider;
}

/** Auth event types */
export type AuthEventType = 'login' | 'logout' | 'token-refreshed' | 'session-expired';

/** Auth event listener callback */
export type AuthEventListener = (data: unknown) => void;

/** Token refresh buffer: refresh 5 minutes before expiry */
export const REFRESH_BEFORE_EXPIRY_MS = 5 * 60 * 1000;

/** Default token expiry: 1 hour */
export const DEFAULT_TOKEN_EXPIRY_MS = 60 * 60 * 1000;

/** localStorage key for session persistence */
export const SESSION_STORAGE_KEY = 'sf-auth-session';

/**
 * AuthService manages user authentication, token lifecycle,
 * and role-based access control for the editor.
 */
export class AuthService {
  private currentUser: User | null = null;
  private tokens: TokenData | null = null;
  private authProvider: AuthProvider = AuthProvider.Local;
  private listeners: Map<AuthEventType, Set<AuthEventListener>> = new Map();
  private refreshTimer: ReturnType<typeof setTimeout> | null = null;
  private storage: Storage | null;
  // Simulated user database for local auth
  private userDatabase: Map<string, { password: string; user: User }> = new Map();

  constructor(storage?: Storage) {
    this.storage = storage ?? null;
    this.restoreSession();
  }

  /** Register a new user account */
  register(username: string, email: string, password: string): User {
    if (this.userDatabase.has(username)) {
      throw new Error('Username already exists');
    }

    const user: User = {
      id: `user_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`,
      username,
      email,
      avatar_url: null,
      role: 'editor',
    };

    this.userDatabase.set(username, { password, user });
    return user;
  }

  /** Log in with username and password */
  login(username: string, password: string): User {
    const entry = this.userDatabase.get(username);
    if (!entry || entry.password !== password) {
      throw new Error('Invalid credentials');
    }

    this.currentUser = entry.user;
    this.authProvider = AuthProvider.Local;
    this.tokens = this.generateTokens();
    this.scheduleRefresh();
    this.persistSession();
    this.emit('login', { user: this.currentUser });

    return this.currentUser;
  }

  /** Log in with an OAuth provider */
  loginWithOAuth(provider: AuthProvider): string {
    // Returns a URL to redirect the user to for OAuth
    const urls: Record<AuthProvider, string> = {
      [AuthProvider.GitHub]: 'https://github.com/login/oauth/authorize',
      [AuthProvider.Google]: 'https://accounts.google.com/o/oauth2/v2/auth',
      [AuthProvider.Local]: '',
    };

    if (provider === AuthProvider.Local) {
      throw new Error('Local provider does not support OAuth');
    }

    this.authProvider = provider;
    return urls[provider];
  }

  /** Complete OAuth login (simulated - in real app called by OAuth callback) */
  completeOAuthLogin(provider: AuthProvider, user: User, tokens: TokenData): void {
    this.currentUser = user;
    this.authProvider = provider;
    this.tokens = tokens;
    this.scheduleRefresh();
    this.persistSession();
    this.emit('login', { user: this.currentUser });
  }

  /** Log out the current user */
  logout(): void {
    const hadUser = this.currentUser !== null;
    this.currentUser = null;
    this.tokens = null;
    this.authProvider = AuthProvider.Local;

    if (this.refreshTimer) {
      clearTimeout(this.refreshTimer);
      this.refreshTimer = null;
    }

    this.clearSession();

    if (hadUser) {
      this.emit('logout', {});
    }
  }

  /** Get the currently logged-in user */
  getCurrentUser(): User | null {
    return this.currentUser;
  }

  /** Get the current tokens */
  getTokens(): TokenData | null {
    return this.tokens;
  }

  /** Refresh the access token */
  refreshToken(): TokenData {
    if (!this.tokens || !this.currentUser) {
      throw new Error('Not authenticated');
    }

    this.tokens = this.generateTokens();
    this.scheduleRefresh();
    this.persistSession();
    this.emit('token-refreshed', { tokens: this.tokens });

    return this.tokens;
  }

  /** Check if the current user can edit */
  canEdit(): boolean {
    if (!this.currentUser) return false;
    return this.currentUser.role === 'owner' || this.currentUser.role === 'editor';
  }

  /** Check if the current user is the owner */
  isOwner(): boolean {
    if (!this.currentUser) return false;
    return this.currentUser.role === 'owner';
  }

  /** Check if the current user can view */
  canView(): boolean {
    return this.currentUser !== null;
  }

  /** Check if the current session is expired */
  isSessionExpired(): boolean {
    if (!this.tokens) return true;
    return Date.now() >= this.tokens.expiresAt;
  }

  /** Get the current auth provider */
  getAuthProvider(): AuthProvider {
    return this.authProvider;
  }

  /** Generate a new set of tokens */
  private generateTokens(): TokenData {
    return {
      accessToken: `at_${Date.now()}_${Math.random().toString(36).slice(2, 14)}`,
      refreshToken: `rt_${Date.now()}_${Math.random().toString(36).slice(2, 14)}`,
      expiresAt: Date.now() + DEFAULT_TOKEN_EXPIRY_MS,
    };
  }

  /** Schedule an automatic token refresh before expiry */
  private scheduleRefresh(): void {
    if (this.refreshTimer) {
      clearTimeout(this.refreshTimer);
    }

    if (!this.tokens) return;

    const timeUntilExpiry = this.tokens.expiresAt - Date.now();
    const refreshDelay = Math.max(0, timeUntilExpiry - REFRESH_BEFORE_EXPIRY_MS);

    if (timeUntilExpiry <= 0) {
      // Already expired
      this.emit('session-expired', {});
      return;
    }

    this.refreshTimer = setTimeout(() => {
      try {
        this.refreshToken();
      } catch {
        this.emit('session-expired', {});
      }
    }, refreshDelay);
  }

  /** Persist session to localStorage */
  private persistSession(): void {
    if (!this.storage || !this.currentUser || !this.tokens) return;

    const session: SessionData = {
      user: this.currentUser,
      tokens: this.tokens,
      provider: this.authProvider,
    };

    try {
      this.storage.setItem(SESSION_STORAGE_KEY, JSON.stringify(session));
    } catch {
      // localStorage might be full or unavailable
    }
  }

  /** Restore session from localStorage */
  private restoreSession(): void {
    if (!this.storage) return;

    try {
      const raw = this.storage.getItem(SESSION_STORAGE_KEY);
      if (!raw) return;

      const session: SessionData = JSON.parse(raw);
      this.currentUser = session.user;
      this.tokens = session.tokens;
      this.authProvider = session.provider;

      // Check if session is still valid
      if (this.isSessionExpired()) {
        this.clearSession();
        this.currentUser = null;
        this.tokens = null;
        this.authProvider = AuthProvider.Local;
        return;
      }

      this.scheduleRefresh();
    } catch {
      // Corrupted session data
      this.clearSession();
    }
  }

  /** Clear session from localStorage */
  private clearSession(): void {
    if (!this.storage) return;

    try {
      this.storage.removeItem(SESSION_STORAGE_KEY);
    } catch {
      // Ignore
    }
  }

  /** Add an event listener */
  on(event: AuthEventType, listener: AuthEventListener): void {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, new Set());
    }
    this.listeners.get(event)!.add(listener);
  }

  /** Remove an event listener */
  off(event: AuthEventType, listener: AuthEventListener): void {
    this.listeners.get(event)?.delete(listener);
  }

  /** Emit an event to all registered listeners */
  emit(event: AuthEventType, data: unknown): void {
    this.listeners.get(event)?.forEach((listener) => {
      try {
        listener(data);
      } catch {
        // Swallow listener errors
      }
    });
  }

  /** Clean up resources */
  destroy(): void {
    if (this.refreshTimer) {
      clearTimeout(this.refreshTimer);
      this.refreshTimer = null;
    }
    this.listeners.clear();
  }
}
