/**
 * Error Boundary Component
 * Catches and handles errors in the component tree
 */

import { Component, JSX, ErrorBoundary as SolidErrorBoundary } from 'solid-js';
import './ErrorBoundary.css';

interface ErrorBoundaryProps {
  children: JSX.Element;
  fallback?: (error: Error, reset: () => void) => JSX.Element;
}

/**
 * Default error fallback UI
 */
const DefaultErrorFallback: Component<{ error: Error; reset: () => void }> = (props) => {
  const isDevelopment = import.meta.env.DEV;

  return (
    <div class="error-boundary">
      <div class="error-boundary-content">
        <div class="error-boundary-icon">⚠️</div>
        <h1 class="error-boundary-title">Something went wrong</h1>
        <p class="error-boundary-message">
          We're sorry, but something unexpected happened. Please try refreshing the page.
        </p>

        {isDevelopment && (
          <details class="error-boundary-details">
            <summary>Error Details (Development Only)</summary>
            <pre class="error-boundary-stack">
              {props.error.message}
              {'\n\n'}
              {props.error.stack}
            </pre>
          </details>
        )}

        <div class="error-boundary-actions">
          <button class="error-boundary-button" onClick={props.reset}>
            Try Again
          </button>
          <button
            class="error-boundary-button error-boundary-button-secondary"
            onClick={() => window.location.href = '/'}
          >
            Go Home
          </button>
        </div>
      </div>
    </div>
  );
};

/**
 * Error Boundary wrapper component
 */
export const ErrorBoundary: Component<ErrorBoundaryProps> = (props) => {
  return (
    <SolidErrorBoundary
      fallback={(error, reset) =>
        props.fallback
          ? props.fallback(error, reset)
          : <DefaultErrorFallback error={error} reset={reset} />
      }
    >
      {props.children}
    </SolidErrorBoundary>
  );
};
