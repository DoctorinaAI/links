/**
 * HTTP Client Auth Setup
 * Configure authentication interceptors for HTTP client
 */

import { getToken, logout } from '../stores/auth.store';
import type { RequestOptions } from '../utils/http.client';
import { httpClient, HttpError } from '../utils/http.client';

/**
 * Setup authentication interceptors
 */
export const setupAuthInterceptors = (): void => {
  // Request interceptor: Add Authorization header
  httpClient.addRequestInterceptor((config: RequestOptions) => {
    const token = getToken();
    if (token) {
      config.headers = {
        ...config.headers,
        Authorization: `Bearer ${token}`,
      };
    }
    return config;
  });

  // Error interceptor: Handle 401/403 by logging out
  httpClient.addErrorInterceptor(async (error: HttpError) => {
    if (error.statusCode === 401 || error.statusCode === 403) {
      console.warn('Authentication failed, logging out...', error.statusCode);
      logout();
    }
  });
};
