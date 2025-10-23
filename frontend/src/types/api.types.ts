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
  slug: string;
  params: Record<string, string>;
  author: string;
  redirect: string | null;
  description?: string | null;
  created_at: string;
  updated_at: string;
}

/**
 * List of short links response
 */
export interface ShortLinksListResponse {
  links: ShortLink[];
  count: number;
}

/**
 * Click statistics response
 */
export interface ClickStatsResponse {
  slug: string;
  total_clicks: number;
  recent_clicks: string[];
}

/**
 * Create short link request
 */
export interface CreateShortLinkRequest {
  slug: string;
  params: Record<string, string>;
  author: string;
  redirect?: string | null;
  description?: string | null;
}

/**
 * Update short link request
 */
export interface UpdateShortLinkRequest {
  slug: string;
  params: Record<string, string>;
  author: string;
  redirect?: string | null;
  description?: string | null;
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
