/**
 * Toast Item Component
 * Individual toast notification with animation
 */

import { Component, createEffect, onCleanup } from 'solid-js';
import type { Toast } from '../../types';
import './ToastItem.css';

interface ToastItemProps {
  toast: Toast;
  onClose: () => void;
}

export const ToastItem: Component<ToastItemProps> = (props) => {
  let progressRef: HTMLDivElement | undefined;

  createEffect(() => {
    if (props.toast.duration && props.toast.duration > 0) {
      const startTime = Date.now();
      const animationDuration = props.toast.duration;

      const updateProgress = () => {
        const elapsed = Date.now() - startTime;
        const progress = Math.min((elapsed / animationDuration) * 100, 100);

        if (progressRef) {
          progressRef.style.width = `${100 - progress}%`;
        }

        if (progress < 100) {
          requestAnimationFrame(updateProgress);
        }
      };

      const animationId = requestAnimationFrame(updateProgress);

      onCleanup(() => cancelAnimationFrame(animationId));
    }
  });

  const getSeverityIcon = () => {
    switch (props.toast.severity) {
      case 'success':
        return '✓';
      case 'error':
        return '✕';
      case 'warning':
        return '⚠';
      case 'info':
      default:
        return 'ℹ';
    }
  };

  return (
    <div
      class={`toast-item toast-${props.toast.severity}`}
      role="alert"
      aria-live="polite"
    >
      <div class="toast-icon">{getSeverityIcon()}</div>
      <div class="toast-message">{props.toast.message}</div>
      <button
        class="toast-close"
        onClick={props.onClose}
        aria-label="Close notification"
      >
        ×
      </button>
      {props.toast.duration && props.toast.duration > 0 && (
        <div class="toast-progress">
          <div ref={progressRef} class="toast-progress-bar" />
        </div>
      )}
    </div>
  );
};
