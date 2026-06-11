import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import {
  AuthService,
  AuthProvider,
  REFRESH_BEFORE_EXPIRY_MS,
  DEFAULT_TOKEN_EXPIRY_MS,
  SESSION_STORAGE_KEY,
  type User,
  type TokenData,
  type SessionData,
} from './auth';

/** Create a mock localStorage */
function createMockStorage(): Storage {
  let store: Record<string, string> = {};
  return {
    getItem: vi.fn((key: string) => store[key] ?? null),
    setItem: vi.fn((key: string, value: string) => {
      store[key] = value;
    }),
    removeItem: vi.fn((key: string) => {
      delete store[key];
    }),
    clear: vi.fn(() => {
      store = {};
    }),
    get length() {
      return Object.keys(store).length;
    },
    key: vi.fn((index: number) => Object.keys(store)[index] ?? null),
  };
}

describe('AuthService', () => {
  let auth: AuthService;
  let storage: Storage;

  beforeEach(() => {
    vi.useFakeTimers();
    storage = createMockStorage();
    auth = new AuthService(storage);
  });

  afterEach(() => {
    auth.destroy();
    vi.useRealTimers();
  });

  // ---- Constants ----

  describe('constants', () => {
    it('should have REFRESH_BEFORE_EXPIRY_MS = 5 minutes', () => {
      expect(REFRESH_BEFORE_EXPIRY_MS).toBe(5 * 60 * 1000);
    });

    it('should have DEFAULT_TOKEN_EXPIRY_MS = 1 hour', () => {
      expect(DEFAULT_TOKEN_EXPIRY_MS).toBe(60 * 60 * 1000);
    });

    it('should have SESSION_STORAGE_KEY defined', () => {
      expect(SESSION_STORAGE_KEY).toBe('sf-auth-session');
    });
  });

  // ---- Registration ----

  describe('register', () => {
    it('should register a new user', () => {
      const user = auth.register('alice', 'alice@example.com', 'password123');
      expect(user.username).toBe('alice');
      expect(user.email).toBe('alice@example.com');
      expect(user.id).toBeDefined();
      expect(user.role).toBe('editor');
    });

    it('should throw on duplicate username', () => {
      auth.register('alice', 'alice@example.com', 'password123');
      expect(() => auth.register('alice', 'alice2@example.com', 'other')).toThrow(
        'Username already exists'
      );
    });

    it('should assign null avatar_url by default', () => {
      const user = auth.register('bob', 'bob@example.com', 'pass');
      expect(user.avatar_url).toBeNull();
    });
  });

  // ---- Login ----

  describe('login', () => {
    it('should login with valid credentials', () => {
      auth.register('alice', 'alice@example.com', 'password123');
      const user = auth.login('alice', 'password123');
      expect(user.username).toBe('alice');
      expect(auth.getCurrentUser()).toEqual(user);
    });

    it('should throw on invalid username', () => {
      expect(() => auth.login('nobody', 'password')).toThrow('Invalid credentials');
    });

    it('should throw on wrong password', () => {
      auth.register('alice', 'alice@example.com', 'password123');
      expect(() => auth.login('alice', 'wrong')).toThrow('Invalid credentials');
    });

    it('should set tokens after login', () => {
      auth.register('alice', 'alice@example.com', 'password123');
      auth.login('alice', 'password123');
      const tokens = auth.getTokens();
      expect(tokens).not.toBeNull();
      expect(tokens!.accessToken).toBeDefined();
      expect(tokens!.refreshToken).toBeDefined();
      expect(tokens!.expiresAt).toBeGreaterThan(Date.now());
    });

    it('should emit login event', () => {
      const listener = vi.fn();
      auth.on('login', listener);
      auth.register('alice', 'alice@example.com', 'password123');
      auth.login('alice', 'password123');
      expect(listener).toHaveBeenCalledOnce();
    });
  });

  // ---- Logout ----

  describe('logout', () => {
    it('should logout the current user', () => {
      auth.register('alice', 'alice@example.com', 'password123');
      auth.login('alice', 'password123');
      auth.logout();
      expect(auth.getCurrentUser()).toBeNull();
      expect(auth.getTokens()).toBeNull();
    });

    it('should emit logout event', () => {
      const listener = vi.fn();
      auth.on('logout', listener);
      auth.register('alice', 'alice@example.com', 'password123');
      auth.login('alice', 'password123');
      auth.logout();
      expect(listener).toHaveBeenCalledOnce();
    });

    it('should not emit logout when not logged in', () => {
      const listener = vi.fn();
      auth.on('logout', listener);
      auth.logout();
      expect(listener).not.toHaveBeenCalled();
    });

    it('should clear session from storage on logout', () => {
      auth.register('alice', 'alice@example.com', 'password123');
      auth.login('alice', 'password123');
      auth.logout();
      expect(storage.removeItem).toHaveBeenCalledWith(SESSION_STORAGE_KEY);
    });
  });

  // ---- Token Management ----

  describe('token management', () => {
    it('should generate tokens with correct expiry', () => {
      auth.register('alice', 'alice@example.com', 'password123');
      auth.login('alice', 'password123');
      const tokens = auth.getTokens()!;
      expect(tokens.expiresAt).toBe(Date.now() + DEFAULT_TOKEN_EXPIRY_MS);
    });

    it('should refresh tokens', () => {
      auth.register('alice', 'alice@example.com', 'password123');
      auth.login('alice', 'password123');
      const oldTokens = auth.getTokens()!;
      const newTokens = auth.refreshToken();
      expect(newTokens.accessToken).not.toBe(oldTokens.accessToken);
      expect(newTokens.refreshToken).not.toBe(oldTokens.refreshToken);
    });

    it('should throw when refreshing without auth', () => {
      expect(() => auth.refreshToken()).toThrow('Not authenticated');
    });

    it('should emit token-refreshed event', () => {
      const listener = vi.fn();
      auth.on('token-refreshed', listener);
      auth.register('alice', 'alice@example.com', 'password123');
      auth.login('alice', 'password123');
      auth.refreshToken();
      expect(listener).toHaveBeenCalledOnce();
    });

    it('should detect expired session', () => {
      auth.register('alice', 'alice@example.com', 'password123');
      auth.login('alice', 'password123');
      expect(auth.isSessionExpired()).toBe(false);
      // Destroy to stop auto-refresh, then advance time
      auth.destroy();
      vi.advanceTimersByTime(DEFAULT_TOKEN_EXPIRY_MS + 1);
      expect(auth.isSessionExpired()).toBe(true);
    });

    it('should detect non-expired session', () => {
      auth.register('alice', 'alice@example.com', 'password123');
      auth.login('alice', 'password123');
      expect(auth.isSessionExpired()).toBe(false);
    });
  });

  // ---- Auto Refresh ----

  describe('auto refresh', () => {
    it('should auto-refresh 5 minutes before expiry', () => {
      const listener = vi.fn();
      auth.on('token-refreshed', listener);

      auth.register('alice', 'alice@example.com', 'password123');
      auth.login('alice', 'password123');

      // Advance to just before refresh time
      vi.advanceTimersByTime(DEFAULT_TOKEN_EXPIRY_MS - REFRESH_BEFORE_EXPIRY_MS);
      expect(listener).toHaveBeenCalledOnce();
    });

    it('should emit session-expired when token is already expired on refresh attempt', () => {
      const expiredListener = vi.fn();
      auth.register('alice', 'alice@example.com', 'password123');
      auth.login('alice', 'password123');
      auth.on('session-expired', expiredListener);

      // Destroy to stop auto-refresh, then advance past expiry
      auth.destroy();
      vi.advanceTimersByTime(DEFAULT_TOKEN_EXPIRY_MS + 1);
      expect(auth.isSessionExpired()).toBe(true);
    });
  });

  // ---- Role-Based Access ----

  describe('role-based access', () => {
    it('owner can edit, is owner, can view', () => {
      auth.register('owner', 'owner@test.com', 'pass');
      const user = auth.login('owner', 'pass');
      // Override role for testing
      (user as User).role = 'owner';
      expect(auth.canEdit()).toBe(true);
      expect(auth.isOwner()).toBe(true);
      expect(auth.canView()).toBe(true);
    });

    it('editor can edit, is not owner, can view', () => {
      auth.register('editor', 'editor@test.com', 'pass');
      auth.login('editor', 'pass');
      // Default role is 'editor'
      expect(auth.canEdit()).toBe(true);
      expect(auth.isOwner()).toBe(false);
      expect(auth.canView()).toBe(true);
    });

    it('viewer cannot edit, is not owner, can view', () => {
      auth.register('viewer', 'viewer@test.com', 'pass');
      const user = auth.login('viewer', 'pass');
      (user as User).role = 'viewer';
      expect(auth.canEdit()).toBe(false);
      expect(auth.isOwner()).toBe(false);
      expect(auth.canView()).toBe(true);
    });

    it('unauthenticated user cannot do anything', () => {
      expect(auth.canEdit()).toBe(false);
      expect(auth.isOwner()).toBe(false);
      expect(auth.canView()).toBe(false);
    });
  });

  // ---- Session Persistence ----

  describe('session persistence', () => {
    it('should persist session to localStorage on login', () => {
      auth.register('alice', 'alice@example.com', 'password123');
      auth.login('alice', 'password123');
      expect(storage.setItem).toHaveBeenCalledWith(
        SESSION_STORAGE_KEY,
        expect.any(String)
      );
    });

    it('should restore session from localStorage', () => {
      auth.register('alice', 'alice@example.com', 'password123');
      auth.login('alice', 'password123');

      // Create new AuthService with same storage
      const auth2 = new AuthService(storage);
      expect(auth2.getCurrentUser()).not.toBeNull();
      expect(auth2.getCurrentUser()!.username).toBe('alice');
      auth2.destroy();
    });

    it('should not restore expired session', () => {
      auth.register('alice', 'alice@example.com', 'password123');
      auth.login('alice', 'password123');

      // Destroy to stop auto-refresh, then advance past expiry
      auth.destroy();
      vi.advanceTimersByTime(DEFAULT_TOKEN_EXPIRY_MS + 1);

      // Create new AuthService with same storage
      const auth2 = new AuthService(storage);
      expect(auth2.getCurrentUser()).toBeNull();
      auth2.destroy();
    });

    it('should clear session from storage on logout', () => {
      auth.register('alice', 'alice@example.com', 'password123');
      auth.login('alice', 'password123');
      auth.logout();
      expect(storage.removeItem).toHaveBeenCalledWith(SESSION_STORAGE_KEY);
    });

    it('should handle corrupted session data gracefully', () => {
      (storage.getItem as ReturnType<typeof vi.fn>).mockReturnValueOnce('not-valid-json');
      const auth2 = new AuthService(storage);
      expect(auth2.getCurrentUser()).toBeNull();
      auth2.destroy();
    });
  });

  // ---- OAuth ----

  describe('OAuth', () => {
    it('should return GitHub OAuth URL', () => {
      const url = auth.loginWithOAuth(AuthProvider.GitHub);
      expect(url).toContain('github.com');
    });

    it('should return Google OAuth URL', () => {
      const url = auth.loginWithOAuth(AuthProvider.Google);
      expect(url).toContain('google');
    });

    it('should throw for Local provider OAuth', () => {
      expect(() => auth.loginWithOAuth(AuthProvider.Local)).toThrow(
        'Local provider does not support OAuth'
      );
    });

    it('should complete OAuth login', () => {
      const user: User = {
        id: 'gh_123',
        username: 'github_user',
        email: 'gh@example.com',
        avatar_url: 'https://avatar.url',
        role: 'editor',
      };
      const tokens: TokenData = {
        accessToken: 'at_gh',
        refreshToken: 'rt_gh',
        expiresAt: Date.now() + DEFAULT_TOKEN_EXPIRY_MS,
      };
      auth.completeOAuthLogin(AuthProvider.GitHub, user, tokens);
      expect(auth.getCurrentUser()).toEqual(user);
      expect(auth.getAuthProvider()).toBe(AuthProvider.GitHub);
    });

    it('should emit login event on OAuth completion', () => {
      const listener = vi.fn();
      auth.on('login', listener);
      const user: User = {
        id: 'go_456',
        username: 'google_user',
        email: 'go@example.com',
        avatar_url: null,
        role: 'viewer',
      };
      const tokens: TokenData = {
        accessToken: 'at_go',
        refreshToken: 'rt_go',
        expiresAt: Date.now() + DEFAULT_TOKEN_EXPIRY_MS,
      };
      auth.completeOAuthLogin(AuthProvider.Google, user, tokens);
      expect(listener).toHaveBeenCalledOnce();
    });
  });

  // ---- Auth Provider ----

  describe('auth provider', () => {
    it('should default to Local provider', () => {
      expect(auth.getAuthProvider()).toBe(AuthProvider.Local);
    });

    it('should update provider after login', () => {
      auth.register('alice', 'alice@test.com', 'pass');
      auth.login('alice', 'pass');
      expect(auth.getAuthProvider()).toBe(AuthProvider.Local);
    });

    it('should reset provider on logout', () => {
      const user: User = {
        id: 'gh_1',
        username: 'gh_user',
        email: 'gh@test.com',
        avatar_url: null,
        role: 'editor',
      };
      const tokens: TokenData = {
        accessToken: 'at',
        refreshToken: 'rt',
        expiresAt: Date.now() + DEFAULT_TOKEN_EXPIRY_MS,
      };
      auth.completeOAuthLogin(AuthProvider.GitHub, user, tokens);
      auth.logout();
      expect(auth.getAuthProvider()).toBe(AuthProvider.Local);
    });
  });

  // ---- Event System ----

  describe('event system', () => {
    it('should support multiple listeners', () => {
      const l1 = vi.fn();
      const l2 = vi.fn();
      auth.on('login', l1);
      auth.on('login', l2);
      auth.register('alice', 'alice@test.com', 'pass');
      auth.login('alice', 'pass');
      expect(l1).toHaveBeenCalledOnce();
      expect(l2).toHaveBeenCalledOnce();
    });

    it('should remove specific listener', () => {
      const listener = vi.fn();
      auth.on('login', listener);
      auth.off('login', listener);
      auth.register('alice', 'alice@test.com', 'pass');
      auth.login('alice', 'pass');
      expect(listener).not.toHaveBeenCalled();
    });

    it('should not cross-fire between events', () => {
      const loginListener = vi.fn();
      const logoutListener = vi.fn();
      auth.on('login', loginListener);
      auth.on('logout', logoutListener);
      auth.register('alice', 'alice@test.com', 'pass');
      auth.login('alice', 'pass');
      expect(loginListener).toHaveBeenCalledOnce();
      expect(logoutListener).not.toHaveBeenCalled();
    });

    it('should swallow listener errors', () => {
      const badListener = () => { throw new Error('oops'); };
      const goodListener = vi.fn();
      auth.on('login', badListener);
      auth.on('login', goodListener);
      auth.register('alice', 'alice@test.com', 'pass');
      auth.login('alice', 'pass');
      expect(goodListener).toHaveBeenCalledOnce();
    });
  });

  // ---- Full Flow ----

  describe('full flow', () => {
    it('should handle register → login → refresh → logout cycle', () => {
      auth.register('alice', 'alice@test.com', 'pass');
      const user = auth.login('alice', 'pass');
      expect(auth.getCurrentUser()).toEqual(user);

      const newTokens = auth.refreshToken();
      expect(auth.getTokens()).toEqual(newTokens);

      auth.logout();
      expect(auth.getCurrentUser()).toBeNull();
      expect(auth.getTokens()).toBeNull();
    });
  });
});
