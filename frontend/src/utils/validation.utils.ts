/**
 * Validation utilities for short links
 */

// Reserved slugs that cannot be used (system routes, common paths)
const RESERVED_SLUGS = new Set([
  'admin',
  'api',
  'auth',
  'login',
  'logout',
  'register',
  'dashboard',
  'settings',
  'profile',
  'user',
  'users',
  'health',
  'status',
  'metrics',
  'docs',
  'swagger',
  'graphql',
  'websocket',
  'ws',
  'static',
  'public',
  'assets',
  'images',
  'css',
  'js',
  'fonts',
  'favicon',
  'robots',
  'sitemap',
  'about',
  'contact',
  'help',
  'support',
  'terms',
  'privacy',
  'legal',
]);

// File extensions that are not allowed for slugs
const FORBIDDEN_EXTENSIONS = new Set([
  // Images
  'jpg', 'jpeg', 'png', 'gif', 'webp', 'svg', 'ico', 'bmp', 'tiff',
  // Documents
  'pdf', 'doc', 'docx', 'xls', 'xlsx', 'ppt', 'pptx', 'txt', 'csv',
  // Web
  'html', 'htm', 'css', 'js', 'json', 'xml', 'yaml', 'yml',
  // Archives
  'zip', 'rar', 'tar', 'gz', '7z',
  // Media
  'mp3', 'mp4', 'avi', 'mov', 'wmv', 'flv', 'wav',
  // Code
  'py', 'java', 'cpp', 'c', 'h', 'cs', 'rb', 'go', 'rs', 'php',
  'ts', 'tsx', 'jsx',
  // Other
  'exe', 'dll', 'so', 'dylib', 'bin', 'dat',
]);

/**
 * Validate slug format
 * Allowed: a-z, 0-9, -, _, ., ~
 * Restrictions:
 * - Not in reserved list
 * - Cannot end with file extension
 * - Cannot start/end with special chars
 * - Cannot contain consecutive dots
 */
export function validateSlug(slug: string): { valid: boolean; error?: string } {
  if (!slug || slug.trim().length === 0) {
    return { valid: false, error: 'Slug is required' };
  }

  const trimmed = slug.trim().toLowerCase();

  if (trimmed.length < 2) {
    return { valid: false, error: 'Slug must be at least 2 characters' };
  }

  if (trimmed.length > 50) {
    return { valid: false, error: 'Slug must be less than 50 characters' };
  }

  // Check basic character set
  const slugRegex = /^[a-z0-9\-_.~]+$/;
  if (!slugRegex.test(trimmed)) {
    return {
      valid: false,
      error: 'Slug can only contain: a-z, 0-9, -, _, ., ~'
    };
  }

  // Cannot start or end with special characters
  if (/^[-_.~]|[-_.~]$/.test(trimmed)) {
    return {
      valid: false,
      error: 'Slug cannot start or end with -, _, ., ~'
    };
  }

  // Cannot contain consecutive dots (security: path traversal)
  if (/\.\./.test(trimmed)) {
    return {
      valid: false,
      error: 'Slug cannot contain consecutive dots'
    };
  }

  // Check if slug is in reserved list
  if (RESERVED_SLUGS.has(trimmed)) {
    return {
      valid: false,
      error: 'This slug is reserved for system use'
    };
  }

  // Check for forbidden file extensions
  const parts = trimmed.split('.');
  if (parts.length > 1) {
    const extension = parts[parts.length - 1];
    if (FORBIDDEN_EXTENSIONS.has(extension)) {
      return {
        valid: false,
        error: `Slug cannot end with .${extension} extension`
      };
    }
  }

  // Additional security checks
  // Cannot be just dots and dashes
  if (/^[.\-~_]+$/.test(trimmed)) {
    return {
      valid: false,
      error: 'Slug must contain at least one alphanumeric character'
    };
  }

  return { valid: true };
}

/**
 * Validate parameter key
 * Stricter validation for analytics safety
 */
export function validateParamKey(key: string): { valid: boolean; error?: string } {
  if (!key || key.trim().length === 0) {
    return { valid: false, error: 'Key is required' };
  }

  if (key.length > 50) {
    return { valid: false, error: 'Key must be less than 50 characters' };
  }

  // Only allow alphanumeric and underscore for keys
  const keyRegex = /^[a-zA-Z_][a-zA-Z0-9_]*$/;
  if (!keyRegex.test(key)) {
    return {
      valid: false,
      error: 'Key must start with letter/underscore and contain only alphanumeric/underscore'
    };
  }

  return { valid: true };
}

/**
 * Validate parameter value
 * Allow more characters but still safe for analytics
 */
export function validateParamValue(value: string): { valid: boolean; error?: string } {
  if (value.length > 200) {
    return { valid: false, error: 'Value must be less than 200 characters' };
  }

  // Disallow control characters and some special chars that could break analytics
  const invalidChars = /[\x00-\x1F\x7F<>]/;
  if (invalidChars.test(value)) {
    return {
      valid: false,
      error: 'Value contains invalid characters'
    };
  }

  return { valid: true };
}

/**
 * Validate redirect URL
 * Must be valid HTTPS URL
 */
export function validateRedirect(url: string | null | undefined): { valid: boolean; error?: string } {
  if (!url || url.trim().length === 0) {
    return { valid: true }; // Redirect is optional
  }

  const trimmed = url.trim();

  if (!trimmed.startsWith('https://')) {
    return {
      valid: false,
      error: 'Redirect URL must start with https://'
    };
  }

  try {
    new URL(trimmed);
    return { valid: true };
  } catch {
    return {
      valid: false,
      error: 'Invalid URL format'
    };
  }
}

/**
 * Check if link can be deleted (< 30 minutes old)
 */
export function canDeleteLink(createdAt: string): boolean {
  const created = new Date(createdAt);
  const now = new Date();
  const diffMinutes = (now.getTime() - created.getTime()) / (1000 * 60);
  return diffMinutes < 30;
}

/**
 * Check if slug can be changed (< 30 minutes old)
 */
export function canChangeSlug(createdAt: string): boolean {
  return canDeleteLink(createdAt);
}
