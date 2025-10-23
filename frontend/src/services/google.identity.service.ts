/**
 * Google Identity Service
 * Handles Google Sign-In integration
 */

import type { GoogleCredentialResponse } from '../types';

declare global {
  interface Window {
    google?: {
      accounts: {
        id: {
          initialize: (config: GoogleInitConfig) => void;
          renderButton: (parent: HTMLElement, options: GoogleButtonConfig) => void;
          prompt: () => void;
        };
      };
    };
  }
}

interface GoogleInitConfig {
  client_id: string;
  callback: (response: GoogleCredentialResponse) => void;
  auto_select?: boolean;
  cancel_on_tap_outside?: boolean;
}

interface GoogleButtonConfig {
  type?: 'standard' | 'icon';
  theme?: 'outline' | 'filled_blue' | 'filled_black';
  size?: 'large' | 'medium' | 'small';
  text?: 'signin_with' | 'signup_with' | 'continue_with' | 'signin';
  shape?: 'rectangular' | 'pill' | 'circle' | 'square';
  logo_alignment?: 'left' | 'center';
  width?: number;
}

/**
 * Initialize Google Identity Services
 */
export const initializeGoogleIdentity = (
  clientId: string,
  callback: (response: GoogleCredentialResponse) => void
): void => {
  if (!window.google) {
    console.error('Google Identity Services not loaded');
    return;
  }

  window.google.accounts.id.initialize({
    client_id: clientId,
    callback,
    auto_select: false,
    cancel_on_tap_outside: true,
  });
};

/**
 * Render Google Sign-In button
 */
export const renderGoogleButton = (
  elementId: string,
  options?: GoogleButtonConfig
): void => {
  const element = document.getElementById(elementId);
  if (!element || !window.google) {
    console.error('Element not found or Google Identity Services not loaded');
    return;
  }

  const defaultOptions: GoogleButtonConfig = {
    type: 'standard',
    theme: 'outline',
    size: 'large',
    text: 'signin_with',
    shape: 'rectangular',
    logo_alignment: 'left',
    width: 250,
  };

  window.google.accounts.id.renderButton(element, {
    ...defaultOptions,
    ...options,
  });
};

/**
 * Decode JWT token (simple base64 decode)
 */
export const decodeJWT = <T = any>(token: string): T | null => {
  try {
    const base64Url = token.split('.')[1];
    const base64 = base64Url.replace(/-/g, '+').replace(/_/g, '/');
    const jsonPayload = decodeURIComponent(
      atob(base64)
        .split('')
        .map((c) => '%' + ('00' + c.charCodeAt(0).toString(16)).slice(-2))
        .join('')
    );
    return JSON.parse(jsonPayload);
  } catch (error) {
    console.error('Failed to decode JWT:', error);
    return null;
  }
};
