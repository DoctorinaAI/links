/**
 * Authentication types
 */

/**
 * User information
 */
export interface User {
  id: string;
  email: string;
  name: string;
  picture?: string;
}

/**
 * Authentication state
 */
export interface AuthState {
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
}

/**
 * Google JWT credential response
 */
export interface GoogleCredentialResponse {
  credential: string;
  select_by?: string;
}

/**
 * Decoded JWT payload from Google
 */
export interface GoogleJWTPayload {
  iss: string;
  sub: string;
  email: string;
  email_verified: boolean;
  name: string;
  picture?: string;
  given_name?: string;
  family_name?: string;
  iat: number;
  exp: number;
}

/**
 * Our internal JWT payload
 */
export interface InternalJWTPayload {
  sub: string;
  email: string;
  name: string;
  picture?: string;
  iat: number;
  exp: number;
}

/**
 * Token exchange response from backend
 */
export interface TokenExchangeResponse {
  token: string;
  user: User;
}
