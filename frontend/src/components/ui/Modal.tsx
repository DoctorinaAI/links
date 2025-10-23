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
  let mouseDownTarget: EventTarget | null = null;

  const handleMouseDown = (e: MouseEvent) => {
    mouseDownTarget = e.target;
  };

  const handleMouseUp = (e: MouseEvent) => {
    // Close only if both mousedown and mouseup happened on the backdrop
    if (mouseDownTarget === e.currentTarget && e.target === e.currentTarget) {
      props.onClose();
    }
    mouseDownTarget = null;
  };

  return (
    <Show when={props.isOpen}>
      <div
        class="modal-backdrop"
        onMouseDown={handleMouseDown}
        onMouseUp={handleMouseUp}
      >
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
