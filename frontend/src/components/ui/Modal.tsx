/**
 * Modal Dialog Component
 */

import { Component, JSX, Show } from 'solid-js';
import './Modal.css';

interface ModalProps {
  isOpen: boolean;
  onClose: () => void;
  title: string;
  children: JSX.Element;
  maxWidth?: string;
}

const Modal: Component<ModalProps> = (props) => {
  const handleBackdropClick = (e: MouseEvent) => {
    if (e.target === e.currentTarget) {
      props.onClose();
    }
  };

  return (
    <Show when={props.isOpen}>
      <div class="modal-backdrop" onClick={handleBackdropClick}>
        <div
          class="modal-content"
          style={{ 'max-width': props.maxWidth || '600px' }}
        >
          <div class="modal-header">
            <h2 class="modal-title">{props.title}</h2>
            <button
              class="modal-close"
              onClick={props.onClose}
              aria-label="Close"
            >
              ×
            </button>
          </div>
          <div class="modal-body">
            {props.children}
          </div>
        </div>
      </div>
    </Show>
  );
};

export default Modal;
