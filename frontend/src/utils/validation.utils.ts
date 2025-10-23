/**
 * Validation utilities for short links
 */

/**
 * Validate slug format
 * Allowed: a-z, 0-9, -, _, ., ~
 */
export function validateSlug(slug: string): { valid: boolean; error?: string } {
  if (!slug || slug.trim().length === 0) {
    return { valid: false, error: 'Slug is required' };
  }

  if (slug.length < 2) {
    return { valid: false, error: 'Slug must be at least 2 characters' };
  }

  if (slug.length > 50) {
    return { valid: false, error: 'Slug must be less than 50 characters' };
  }

  const slugRegex = /^[a-z0-9\-_.~]+$/;
  if (!slugRegex.test(slug)) {
    return { 
      valid: false, 
      error: 'Slug can only contain: a-z, 0-9, -, _, ., ~' 
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
