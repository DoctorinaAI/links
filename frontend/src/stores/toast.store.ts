/**
 * Toast Notification Store
 * Global state management for toast notifications
 */

import { createSignal } from 'solid-js';
import { TOAST_CONFIG } from '../config/app.config';
import type { Toast, ToastSeverity } from '../types';
import { generateId } from '../utils/helpers';

// Toast store state
const [toasts, setToasts] = createSignal<Toast[]>([]);

/**
 * Add a new toast notification
 */
function addToast(
  message: string,
  severity: ToastSeverity = 'info',
  duration?: number
): string {
  const id = generateId();
  const toast: Toast = {
    id,
    message,
    severity,
    duration: duration ?? TOAST_CONFIG.defaultDuration,
    timestamp: Date.now(),
  };

  setToasts((prev) => {
    const updated = [...prev, toast];
    // Limit max toasts
    if (updated.length > TOAST_CONFIG.maxToasts) {
      return updated.slice(-TOAST_CONFIG.maxToasts);
    }
    return updated;
  });

  // Auto-remove toast after duration
  const finalDuration = duration ?? TOAST_CONFIG.defaultDuration;
  if (finalDuration > 0) {
    setTimeout(() => removeToast(id), finalDuration);
  }

  return id;
}

/**
 * Remove a toast notification by ID
 */
function removeToast(id: string): void {
  setToasts((prev) => prev.filter((toast) => toast.id !== id));
}

/**
 * Clear all toasts
 */
function clearAllToasts(): void {
  setToasts([]);
}

/**
 * Show success toast
 */
function showSuccess(message: string, duration?: number): string {
  return addToast(message, 'success', duration);
}

/**
 * Show error toast
 */
function showError(message: string, duration?: number): string {
  return addToast(message, 'error', duration);
}

/**
 * Show warning toast
 */
function showWarning(message: string, duration?: number): string {
  return addToast(message, 'warning', duration);
}

/**
 * Show info toast
 */
function showInfo(message: string, duration?: number): string {
  return addToast(message, 'info', duration);
}

// Export toast store
export const toastStore = {
  toasts,
  addToast,
  removeToast,
  clearAllToasts,
  showSuccess,
  showError,
  showWarning,
  showInfo,
};
