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
export const login = (userData: User, token: string): void => {
  setUser(userData);
  localStorage.setItem(TOKEN_KEY, token);
};

/**
 * Clear user and remove token
 */
export const logout = (): void => {
  setUser(null);
  localStorage.removeItem(TOKEN_KEY);
};

/**
 * Get stored auth token
 */
export const getToken = (): string | null => {
  return localStorage.getItem(TOKEN_KEY);
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
