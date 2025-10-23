/**
 * Application configuration
 * Centralized configuration management using environment variables
 */

/**
 * API configuration
 */
export const API_CONFIG = {
  baseURL: import.meta.env.VITE_API_BASE_URL || 'http://localhost:8000',
  timeout: parseInt(import.meta.env.VITE_API_TIMEOUT || '30000', 10),
  headers: {
    'Content-Type': 'application/json',
  },
} as const;

/**
 * Application configuration
 */
export const APP_CONFIG = {
  name: 'Links',
  version: import.meta.env.VITE_APP_VERSION || '0.0.1',
  environment: import.meta.env.MODE || 'development',
  isDevelopment: import.meta.env.DEV,
  isProduction: import.meta.env.PROD,
  googleClientId: import.meta.env.VITE_GOOGLE_CLIENT_ID || '',
} as const;

/**
 * Toast notification configuration
 */
export const TOAST_CONFIG = {
  defaultDuration: 5000,
  maxToasts: 5,
  position: 'top-right',
} as const;

/**
 * Pagination configuration
 */
export const PAGINATION_CONFIG = {
  defaultPage: 1,
  defaultPerPage: 20,
  pageSizeOptions: [10, 20, 50, 100],
} as const;

/**
 * PWA configuration
 */
export const PWA_CONFIG = {
  enabled: import.meta.env.PROD || import.meta.env.VITE_PWA_DEV === 'true',
  swPath: '/sw.js',
} as const;

/**
 * Google OAuth configuration
 */
export const GOOGLE_CONFIG = {
  clientId: import.meta.env.VITE_GOOGLE_CLIENT_ID || '',
  enabled: !!import.meta.env.VITE_GOOGLE_CLIENT_ID,
} as const;
