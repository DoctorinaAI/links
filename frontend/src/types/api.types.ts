/**
 * API response types
 * Base interfaces for all API communications
 */

/**
 * Standard API response wrapper
 */
export interface ApiResponse<T = unknown> {
  success: boolean;
  data?: T;
  error?: ApiError;
  metadata?: ResponseMetadata;
}

/**
 * API error details
 */
export interface ApiError {
  code: string;
  message: string;
  details?: Record<string, unknown>;
  field?: string;
}

/**
 * Response metadata (pagination, etc.)
 */
export interface ResponseMetadata {
  page?: number;
  perPage?: number;
  total?: number;
  timestamp?: string;
}

/**
 * Short link entity
 */
export interface ShortLink {
  id: string;
  code: string;
  url: string;
  title?: string;
  description?: string;
  clicks: number;
  createdAt: string;
  updatedAt: string;
  expiresAt?: string;
  enabled: boolean;
}

/**
 * Click statistics
 */
export interface ClickStats {
  total: number;
  unique: number;
  byDate: Record<string, number>;
  byCountry?: Record<string, number>;
  byDevice?: Record<string, number>;
}

/**
 * Create short link request
 */
export interface CreateShortLinkRequest {
  url: string;
  code?: string;
  title?: string;
  description?: string;
  expiresAt?: string;
}

/**
 * Update short link request
 */
export interface UpdateShortLinkRequest {
  url?: string;
  title?: string;
  description?: string;
  expiresAt?: string;
  enabled?: boolean;
}

/**
 * Health check response
 */
export interface HealthStatus {
  status: 'OK' | 'ERROR';
  database?: string;
  uptime?: number;
}

/**
 * About information
 */
export interface AboutInfo {
  name: string;
  version: string;
  description: string;
  author?: string;
  license?: string;
  repository?: string;
}
