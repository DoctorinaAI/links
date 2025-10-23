/**
 * Error handling utilities
 */

import { toastStore } from '../stores/toast.store';
import type { ApiError } from '../types';

/**
 * Format error message for display
 */
export function formatErrorMessage(error: unknown): string {
  if (typeof error === 'string') {
    return error;
  }

  if (error instanceof Error) {
    return error.message;
  }

  if (isApiError(error)) {
    return error.message || 'An error occurred';
  }

  return 'An unexpected error occurred';
}

/**
 * Check if error is an API error
 */
export function isApiError(error: unknown): error is ApiError {
  return (
    typeof error === 'object' &&
    error !== null &&
    'code' in error &&
    'message' in error
  );
}

/**
 * Handle error with toast notification
 */
export function handleError(error: unknown, customMessage?: string): void {
  const message = customMessage || formatErrorMessage(error);
  toastStore.showError(message);

  // Log error in development
  if (import.meta.env.DEV) {
    console.error('Error:', error);
  }
}

/**
 * Handle API error specifically
 */
export function handleApiError(error: ApiError): void {
  const message = error.message || 'API request failed';
  toastStore.showError(message);

  // Log error details in development
  if (import.meta.env.DEV) {
    console.error('API Error:', {
      code: error.code,
      message: error.message,
      details: error.details,
      field: error.field,
    });
  }
}

/**
 * Async error wrapper
 * Wraps async functions with automatic error handling
 */
export function withErrorHandling<T extends (...args: any[]) => Promise<any>>(
  fn: T,
  customErrorMessage?: string
): T {
  return (async (...args: Parameters<T>) => {
    try {
      return await fn(...args);
    } catch (error) {
      handleError(error, customErrorMessage);
      throw error;
    }
  }) as T;
}

/**
 * Create error object
 */
export function createError(code: string, message: string, details?: Record<string, unknown>): ApiError {
  return {
    code,
    message,
    details,
  };
}

/**
 * Network error checker
 */
export function isNetworkError(error: unknown): boolean {
  if (error instanceof TypeError) {
    return error.message.includes('fetch') || error.message.includes('network');
  }
  return false;
}

/**
 * Timeout error checker
 */
export function isTimeoutError(error: unknown): boolean {
  if (error instanceof Error) {
    return error.name === 'AbortError' || error.message.includes('timeout');
  }
  return false;
}

/**
 * Get user-friendly error message
 */
export function getUserFriendlyError(error: unknown): string {
  if (isNetworkError(error)) {
    return 'Network connection failed. Please check your internet connection.';
  }

  if (isTimeoutError(error)) {
    return 'Request timed out. Please try again.';
  }

  return formatErrorMessage(error);
}
