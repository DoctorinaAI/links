/**
 * HTTP Client Auth Setup
 * Configure authentication interceptors for HTTP client
 */

import { getToken, logout } from '../stores/auth.store';
import type { RequestOptions } from '../utils/http.client';
import { httpClient, HttpError } from '../utils/http.client';

/**
 * Public endpoints that don't require authentication
 * These endpoints should not have the Authorization header added
 */
const PUBLIC_ENDPOINTS = [
  '/api/v1/auth/google', // Google token exchange endpoint
  '/api/health',
  '/api/about',
];

/**
 * Check if endpoint is public (doesn't require auth)
 */
const isPublicEndpoint = (url: string): boolean => {
  return PUBLIC_ENDPOINTS.some(endpoint => url.includes(endpoint));
};

/**
 * Setup authentication interceptors
 */
export const setupAuthInterceptors = (): void => {
  // Request interceptor: Add Authorization header for private endpoints
  httpClient.addRequestInterceptor((config: RequestOptions) => {
    const endpoint = config.endpoint || '';

    // Skip auth header for public endpoints
    if (isPublicEndpoint(endpoint)) {
      console.debug(`[Auth] Public endpoint, skipping auth: ${endpoint}`);
      return config;
    }

    // Add Authorization header for all other requests
    const token = getToken();

    if (token) {
      console.debug(`[Auth] Adding auth header to: ${endpoint}`);

      // Ensure headers is an object
      const headers = config.headers as Record<string, string> || {};

      config.headers = {
        ...headers,
        Authorization: `Bearer ${token}`,
      } as HeadersInit;
    } else {
      console.warn(`[Auth] ⚠️ No token available for: ${endpoint}`);
    }

    return config;
  });

  // Error interceptor: Handle 401/403 by logging out
  httpClient.addErrorInterceptor(async (error: HttpError) => {
    if (error.statusCode === 401 || error.statusCode === 403) {
      console.warn('[Auth] Authentication failed, logging out...', error.statusCode);
      logout();
    }
  });
};