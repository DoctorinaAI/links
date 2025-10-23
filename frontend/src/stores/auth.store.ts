/**
 * Authentication store
 * Global state management for user authentication
 */

import { createSignal } from 'solid-js';
import type { User } from '../types';

const TOKEN_KEY = 'auth_token';

// Signals
const [user, setUser] = createSignal<User | null>(null);
const [isLoading, setIsLoading] = createSignal<boolean>(false);
const [token, setToken] = createSignal<string | null>(null);

/**
 * Initialize token from localStorage on app start
 * Should be called once during app initialization
 */
export const initializeAuth = (): void => {
  const storedToken = localStorage.getItem(TOKEN_KEY);
  if (storedToken) {
    setToken(storedToken);
    console.debug('[Auth] Token restored from storage');
  }
};

/**
 * Check if user is authenticated
 */
export const isAuthenticated = (): boolean => {
  return user() !== null;
};

/**
 * Get current user
 */
export const getUser = () => user;

/**
 * Set user and mark as authenticated
 */
export const login = (userData: User, authToken: string): void => {
  // Set in memory first (synchronous, instant)
  setToken(authToken);
  setUser(userData);

  // Save to localStorage asynchronously (non-blocking)
  queueMicrotask(() => {
    try {
      localStorage.setItem(TOKEN_KEY, authToken);
      console.debug('[Auth] Token saved to storage');
    } catch (error) {
      console.error('[Auth] Failed to save token to storage:', error);
    }
  });
};

/**
 * Clear user and remove token
 */
export const logout = (): void => {
  // Clear memory first (synchronous)
  setToken(null);
  setUser(null);

  // Remove from localStorage asynchronously (non-blocking)
  queueMicrotask(() => {
    try {
      localStorage.removeItem(TOKEN_KEY);
      console.debug('[Auth] Token removed from storage');
    } catch (error) {
      console.error('[Auth] Failed to remove token from storage:', error);
    }
  });
};

/**
 * Get current auth token from memory
 * Always returns the in-memory token (fast, synchronous)
 */
export const getToken = (): string | null => {
  return token();
};

/**
 * Set loading state
 */
export const setAuthLoading = (loading: boolean): void => {
  setIsLoading(loading);
};

/**
 * Get loading state
 */
export const getAuthLoading = () => isLoading;

/**
 * Check if token exists in storage
 */
export const hasStoredToken = (): boolean => {
  return getToken() !== null;
};
