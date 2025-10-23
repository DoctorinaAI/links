/**
 * Theme Configuration
 * Centralized color palette and theme settings
 */

export const theme = {
  colors: {
    // Primary brand colors
    primary: {
      main: '#6b7fd7',
      light: '#8c9ee8',
      dark: '#4a5fb5',
      gradient: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
    },

    // Background colors
    background: {
      primary: '#6b7fd7',
      secondary: '#f7fafc',
      white: '#ffffff',
      gray: '#f9fafb',
    },

    // Text colors
    text: {
      primary: '#2d3748',
      secondary: '#718096',
      tertiary: '#a0aec0',
      inverse: '#ffffff',
    },

    // Border colors
    border: {
      light: '#e2e8f0',
      medium: '#cbd5e0',
      dark: '#a0aec0',
    },

    // Status colors
    status: {
      success: '#48bb78',
      error: '#f56565',
      warning: '#ed8936',
      info: '#4299e1',
    },

    // Shadow colors
    shadow: {
      light: 'rgba(0, 0, 0, 0.1)',
      medium: 'rgba(0, 0, 0, 0.2)',
      heavy: 'rgba(0, 0, 0, 0.3)',
      primary: 'rgba(102, 126, 234, 0.35)',
    },
  },

  // Spacing scale
  spacing: {
    xs: '0.5rem',
    sm: '0.75rem',
    md: '1rem',
    lg: '1.5rem',
    xl: '2rem',
    xxl: '3rem',
  },

  // Border radius
  radius: {
    sm: '0.5rem',
    md: '0.75rem',
    lg: '1rem',
    xl: '1.5rem',
    xxl: '2rem',
    full: '9999px',
  },

  // Font sizes
  fontSize: {
    xs: '0.75rem',
    sm: '0.875rem',
    base: '1rem',
    lg: '1.125rem',
    xl: '1.25rem',
    '2xl': '1.5rem',
    '3xl': '1.875rem',
    '4xl': '2.25rem',
  },

  // Font weights
  fontWeight: {
    normal: '400',
    medium: '500',
    semibold: '600',
    bold: '700',
    extrabold: '800',
  },

  // Shadows
  shadow: {
    sm: '0 1px 2px 0 rgba(0, 0, 0, 0.05)',
    base: '0 1px 3px 0 rgba(0, 0, 0, 0.1), 0 1px 2px 0 rgba(0, 0, 0, 0.06)',
    md: '0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06)',
    lg: '0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05)',
    xl: '0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04)',
    '2xl': '0 25px 50px -12px rgba(0, 0, 0, 0.25)',
    card: '0 20px 60px rgba(0, 0, 0, 0.3), 0 0 0 1px rgba(255, 255, 255, 0.1) inset',
    primaryGlow: '0 12px 28px rgba(102, 126, 234, 0.35), 0 0 0 1px rgba(255, 255, 255, 0.2) inset',
  },

  // Z-index scale
  zIndex: {
    base: 0,
    dropdown: 1000,
    sticky: 1020,
    fixed: 1030,
    modal: 1040,
    popover: 1050,
    tooltip: 1060,
    toast: 10000,
  },
} as const;

export type Theme = typeof theme;
