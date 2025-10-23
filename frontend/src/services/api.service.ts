/**
 * API Service
 * Centralized API calls with type safety
 */

import type {
    AboutInfo,
    ApiResponse,
    ClickStats,
    CreateShortLinkRequest,
    HealthStatus,
    ShortLink,
    UpdateShortLinkRequest,
} from '../types';
import { http } from '../utils/http.client';

/**
 * Health check
 */
export async function checkHealth(): Promise<ApiResponse<HealthStatus>> {
  return http.get<HealthStatus>('/api/health');
}

/**
 * Get about information
 */
export async function getAbout(): Promise<ApiResponse<AboutInfo>> {
  return http.get<AboutInfo>('/api/about');
}

/**
 * Create short link
 */
export async function createShortLink(
  data: CreateShortLinkRequest
): Promise<ApiResponse<ShortLink>> {
  return http.post<ShortLink>('/api/links', data);
}

/**
 * Get all short links
 */
export async function getShortLinks(params?: {
  page?: number;
  perPage?: number;
}): Promise<ApiResponse<{ links: ShortLink[]; total: number }>> {
  return http.get<{ links: ShortLink[]; total: number }>('/api/links', { params });
}

/**
 * Get single short link
 */
export async function getShortLink(id: string): Promise<ApiResponse<ShortLink>> {
  return http.get<ShortLink>(`/api/links/${id}`);
}

/**
 * Update short link
 */
export async function updateShortLink(
  id: string,
  data: UpdateShortLinkRequest
): Promise<ApiResponse<ShortLink>> {
  return http.put<ShortLink>(`/api/links/${id}`, data);
}

/**
 * Delete short link
 */
export async function deleteShortLink(id: string): Promise<ApiResponse<void>> {
  return http.delete<void>(`/api/links/${id}`);
}

/**
 * Get click statistics
 */
export async function getClickStats(id: string): Promise<ApiResponse<ClickStats>> {
  return http.get<ClickStats>(`/api/links/${id}/stats`);
}

/**
 * Resolve short link (public)
 */
export async function resolveShortLink(
  code: string
): Promise<ApiResponse<{ url: string }>> {
  return http.get<{ url: string }>(`/api/resolve/${code}`);
}
