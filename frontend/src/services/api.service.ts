/**
 * API Service
 * Centralized API calls with type safety
 */

import type {
    ApiResponse,
    ClickStatsResponse,
    CreateShortLinkRequest,
    HealthStatus,
    ShortLink,
    ShortLinksListResponse,
    UpdateShortLinkRequest,
} from '../types';
import { http } from '../utils/http.client';

/**
 * Health check
 */
export async function checkHealth(): Promise<ApiResponse<HealthStatus>> {
  return http.get<HealthStatus>('/api/v1/health');
}

/**
 * Get all short links
 */
export async function getShortLinks(): Promise<ApiResponse<ShortLinksListResponse>> {
  return http.get<ShortLinksListResponse>('/api/v1/admin/links');
}

/**
 * Get single short link by slug
 */
export async function getShortLink(slug: string): Promise<ApiResponse<ShortLink>> {
  return http.get<ShortLink>(`/api/v1/admin/links/${slug}`);
}

/**
 * Create short link
 */
export async function createShortLink(
  data: CreateShortLinkRequest
): Promise<ApiResponse<ShortLink>> {
  return http.post<ShortLink>('/api/v1/admin/links', data);
}

/**
 * Update short link
 */
export async function updateShortLink(
  slug: string,
  data: UpdateShortLinkRequest
): Promise<ApiResponse<ShortLink>> {
  return http.put<ShortLink>(`/api/v1/admin/links/${slug}`, data);
}

/**
 * Delete short link
 */
export async function deleteShortLink(slug: string): Promise<ApiResponse<void>> {
  return http.delete<void>(`/api/v1/admin/links/${slug}`);
}

/**
 * Get click statistics
 */
export async function getClickStats(slug: string): Promise<ApiResponse<ClickStatsResponse>> {
  return http.get<ClickStatsResponse>(`/api/v1/admin/links/${slug}/stats`);
}
