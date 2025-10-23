/**
 * Authentication Service
 * Handles authentication flow with backend
 */

import * as authStore from '../stores/auth.store';
import type {
    GoogleJWTPayload,
    InternalJWTPayload,
    TokenExchangeResponse,
    User,
} from '../types';
import { httpClient } from '../utils/http.client';
import { decodeJWT } from './google.identity.service';

/**
 * Exchange Google JWT for internal token
 */
export const exchangeGoogleToken = async (
  googleToken: string
): Promise<TokenExchangeResponse> => {
  const response = await httpClient.post<TokenExchangeResponse>(
    '/api/v1/auth/google',
    { token: googleToken }
  );

  // Handle both 'success' and 'status' fields for backward compatibility
  const isSuccess = response.success === true || (response as any).status === 'ok';

  if (!isSuccess || !response.data) {
    throw new Error(response.error?.message || 'Token exchange failed');
  }

  return response.data;
};

/**
 * Validate and restore session from stored token
 */
export const validateStoredToken = async (): Promise<User | null> => {
  const token = authStore.getToken();
  if (!token) {
    return null;
  }

  try {
    // Decode token to get user info
    const payload = decodeJWT<InternalJWTPayload>(token);
    if (!payload) {
      authStore.logout();
      return null;
    }

    // Note: Our internal tokens are non-expiring, so no exp check needed
    // If exp exists and is expired, logout
    if (payload.exp) {
      const now = Math.floor(Date.now() / 1000);
      if (payload.exp < now) {
        authStore.logout();
        return null;
      }
    }

    // Token is valid, create user from payload
    const user: User = {
      id: payload.sub,
      email: payload.email,
      name: payload.name,
      picture: payload.picture,
    };

    // Return user (token is valid until server restart or secret change)
    return user;
  } catch (error) {
    console.error('Token validation failed:', error);
    authStore.logout();
    return null;
  }
};

/**
 * Handle Google Sign-In callback
 */
export const handleGoogleSignIn = async (
  googleToken: string
): Promise<void> => {
  try {
    authStore.setAuthLoading(true);

    // Decode Google JWT to get user info
    const googlePayload = decodeJWT<GoogleJWTPayload>(googleToken);
    if (!googlePayload) {
      throw new Error('Invalid Google token');
    }

    // Exchange Google token for our internal token
    const { token, user } = await exchangeGoogleToken(googleToken);

    // Save token and set user
    authStore.login(user, token);

    console.log('Authentication successful:', user.email);
  } catch (error) {
    console.error('Google sign-in failed:', error);
    authStore.logout();
    throw error;
  } finally {
    authStore.setAuthLoading(false);
  }
};

/**
 * Logout user
 */
export const logoutUser = (): void => {
  authStore.logout();
  console.log('User logged out');
};

/**
 * Get current auth token for requests
 */
export const getAuthToken = (): string | null => {
  return authStore.getToken();
};
