/**
 * Toast Container Component
 * Displays toast notifications at the configured position
 */

import { Component, For } from 'solid-js';
import { toastStore } from '../../stores/toast.store';
import './ToastContainer.css';
import { ToastItem } from './ToastItem';

export const ToastContainer: Component = () => {
  return (
    <div class="toast-container">
      <For each={toastStore.toasts()}>
        {(toast) => (
          <ToastItem
            toast={toast}
            onClose={() => toastStore.removeToast(toast.id)}
          />
        )}
      </For>
    </div>
  );
};
