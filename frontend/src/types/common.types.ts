/**
 * Common application types
 */

/**
 * Toast notification severity levels
 */
export type ToastSeverity = 'success' | 'error' | 'warning' | 'info';

/**
 * Toast notification
 */
export interface Toast {
  id: string;
  message: string;
  severity: ToastSeverity;
  duration?: number;
  timestamp: number;
}

/**
 * Loading state
 */
export interface LoadingState {
  isLoading: boolean;
  message?: string;
}

/**
 * Error state
 */
export interface ErrorState {
  hasError: boolean;
  error?: Error | ApiError;
  timestamp?: number;
}

/**
 * Pagination params
 */
export interface PaginationParams {
  page: number;
  perPage: number;
}

/**
 * Sort params
 */
export interface SortParams {
  field: string;
  order: 'asc' | 'desc';
}

/**
 * Filter params
 */
export interface FilterParams {
  search?: string;
  enabled?: boolean;
  dateFrom?: string;
  dateTo?: string;
}

import type { ApiError } from './api.types';
