/**
 * HTTP Client with error handling and interceptors
 * Modern fetch-based implementation with retry logic
 */

import { API_CONFIG } from '../config/app.config';
import type { ApiError, ApiResponse } from '../types';

/**
 * HTTP request options
 */
export interface RequestOptions extends RequestInit {
  params?: Record<string, string | number | boolean>;
  timeout?: number;
  retries?: number;
  retryDelay?: number;
}

/**
 * Custom HTTP error class
 */
export class HttpError extends Error {
  constructor(
    public statusCode: number,
    public apiError?: ApiError,
    message?: string
  ) {
    super(message || apiError?.message || `HTTP Error ${statusCode}`);
    this.name = 'HttpError';
  }
}

/**
 * HTTP Client class
 * Singleton pattern for centralized HTTP communications
 */
class HttpClient {
  private static instance: HttpClient;
  private baseURL: string;
  private defaultTimeout: number;
  private requestInterceptors: Array<(config: RequestOptions) => RequestOptions | Promise<RequestOptions>> = [];
  private responseInterceptors: Array<(response: Response) => Response | Promise<Response>> = [];
  private errorInterceptors: Array<(error: HttpError) => void | Promise<void>> = [];

  private constructor() {
    this.baseURL = API_CONFIG.baseURL;
    this.defaultTimeout = API_CONFIG.timeout;
  }

  public static getInstance(): HttpClient {
    if (!HttpClient.instance) {
      HttpClient.instance = new HttpClient();
    }
    return HttpClient.instance;
  }

  /**
   * Add request interceptor
   */
  public addRequestInterceptor(
    interceptor: (config: RequestOptions) => RequestOptions | Promise<RequestOptions>
  ): void {
    this.requestInterceptors.push(interceptor);
  }

  /**
   * Add response interceptor
   */
  public addResponseInterceptor(
    interceptor: (response: Response) => Response | Promise<Response>
  ): void {
    this.responseInterceptors.push(interceptor);
  }

  /**
   * Add error interceptor (called on HTTP errors)
   */
  public addErrorInterceptor(
    interceptor: (error: HttpError) => void | Promise<void>
  ): void {
    this.errorInterceptors.push(interceptor);
  }

  /**
   * Build URL with query parameters
   */
  private buildUrl(endpoint: string, params?: Record<string, string | number | boolean>): string {
    const url = new URL(endpoint, this.baseURL);

    if (params) {
      Object.entries(params).forEach(([key, value]) => {
        url.searchParams.append(key, String(value));
      });
    }

    return url.toString();
  }

  /**
   * Apply request interceptors
   */
  private async applyRequestInterceptors(config: RequestOptions): Promise<RequestOptions> {
    let modifiedConfig = config;

    for (const interceptor of this.requestInterceptors) {
      modifiedConfig = await interceptor(modifiedConfig);
    }

    return modifiedConfig;
  }

  /**
   * Apply response interceptors
   */
  private async applyResponseInterceptors(response: Response): Promise<Response> {
    let modifiedResponse = response;

    for (const interceptor of this.responseInterceptors) {
      modifiedResponse = await interceptor(modifiedResponse);
    }

    return modifiedResponse;
  }

  /**
   * Parse response body
   */
  private async parseResponse<T>(response: Response): Promise<ApiResponse<T>> {
    const contentType = response.headers.get('content-type');

    // Handle empty responses
    if (response.status === 204 || !contentType) {
      return { success: true, data: undefined as T };
    }

    // Parse JSON response
    if (contentType?.includes('application/json')) {
      const json = await response.json();

      // Handle API error response
      if (!response.ok) {
        throw new HttpError(response.status, json.error, json.error?.message);
      }

      return json as ApiResponse<T>;
    }

    // Handle text response
    const text = await response.text();
    return { success: true, data: text as T };
  }

  /**
   * Make HTTP request with retry logic
   */
  private async requestWithRetry<T>(
    url: string,
    options: RequestOptions,
    retries = 0,
    retryDelay = 1000
  ): Promise<ApiResponse<T>> {
    try {
      // Apply request interceptors
      const config = await this.applyRequestInterceptors(options);

      // Create abort controller for timeout
      const controller = new AbortController();
      const timeout = config.timeout || this.defaultTimeout;
      const timeoutId = setTimeout(() => controller.abort(), timeout);

      // Make request
      let response = await fetch(url, {
        ...config,
        signal: controller.signal,
        headers: {
          ...API_CONFIG.headers,
          ...config.headers,
        },
      });

      clearTimeout(timeoutId);

      // Apply response interceptors
      response = await this.applyResponseInterceptors(response);

      // Parse and return response
      return await this.parseResponse<T>(response);

    } catch (error) {
      // Retry on network errors
      if (retries > 0 && (error instanceof TypeError || (error as any).name === 'AbortError')) {
        await new Promise((resolve) => setTimeout(resolve, retryDelay));
        return this.requestWithRetry<T>(url, options, retries - 1, retryDelay * 2);
      }

      // Call error interceptors for HTTP errors
      if (error instanceof HttpError) {
        for (const interceptor of this.errorInterceptors) {
          await interceptor(error);
        }
        throw error;
      }

      // Wrap other errors
      throw new HttpError(
        0,
        {
          code: 'NETWORK_ERROR',
          message: error instanceof Error ? error.message : 'Network request failed',
        }
      );
    }
  }

  /**
   * Generic request method
   */
  private async request<T>(
    endpoint: string,
    options: RequestOptions = {}
  ): Promise<ApiResponse<T>> {
    const { params, retries = 2, retryDelay = 1000, ...fetchOptions } = options;
    const url = this.buildUrl(endpoint, params);

    return this.requestWithRetry<T>(url, fetchOptions, retries, retryDelay);
  }

  /**
   * GET request
   */
  public async get<T>(endpoint: string, options?: RequestOptions): Promise<ApiResponse<T>> {
    return this.request<T>(endpoint, {
      ...options,
      method: 'GET',
    });
  }

  /**
   * POST request
   */
  public async post<T>(
    endpoint: string,
    data?: unknown,
    options?: RequestOptions
  ): Promise<ApiResponse<T>> {
    return this.request<T>(endpoint, {
      ...options,
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  /**
   * PUT request
   */
  public async put<T>(
    endpoint: string,
    data?: unknown,
    options?: RequestOptions
  ): Promise<ApiResponse<T>> {
    return this.request<T>(endpoint, {
      ...options,
      method: 'PUT',
      body: JSON.stringify(data),
    });
  }

  /**
   * DELETE request
   */
  public async delete<T>(endpoint: string, options?: RequestOptions): Promise<ApiResponse<T>> {
    return this.request<T>(endpoint, {
      ...options,
      method: 'DELETE',
    });
  }

  /**
   * PATCH request
   */
  public async patch<T>(
    endpoint: string,
    data?: unknown,
    options?: RequestOptions
  ): Promise<ApiResponse<T>> {
    return this.request<T>(endpoint, {
      ...options,
      method: 'PATCH',
      body: JSON.stringify(data),
    });
  }
}

// Export singleton instance
export const httpClient = HttpClient.getInstance();

// Export convenience methods
export const http = {
  get: <T>(endpoint: string, options?: RequestOptions) => httpClient.get<T>(endpoint, options),
  post: <T>(endpoint: string, data?: unknown, options?: RequestOptions) =>
    httpClient.post<T>(endpoint, data, options),
  put: <T>(endpoint: string, data?: unknown, options?: RequestOptions) =>
    httpClient.put<T>(endpoint, data, options),
  delete: <T>(endpoint: string, options?: RequestOptions) => httpClient.delete<T>(endpoint, options),
  patch: <T>(endpoint: string, data?: unknown, options?: RequestOptions) =>
    httpClient.patch<T>(endpoint, data, options),
  addRequestInterceptor: (interceptor: (config: RequestOptions) => RequestOptions | Promise<RequestOptions>) =>
    httpClient.addRequestInterceptor(interceptor),
  addResponseInterceptor: (interceptor: (response: Response) => Response | Promise<Response>) =>
    httpClient.addResponseInterceptor(interceptor),
};
