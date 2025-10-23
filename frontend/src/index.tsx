/**
 * Application Entry Point
 * Initializes the SolidJS application with PWA support
 */

/* @refresh reload */
import { render } from 'solid-js/web';
import App from './App';
import { PWA_CONFIG } from './config/app.config';
import { pwaService } from './services/pwa.service';

const root = document.getElementById('root');

if (import.meta.env.DEV && !(root instanceof HTMLElement)) {
  throw new Error(
    'Root element not found. Did you forget to add it to your index.html? ' +
    'Or maybe the id attribute got misspelled?',
  );
}

/**
 * Initialize PWA service worker
 */
function initializePWA(): void {
  if (!PWA_CONFIG.enabled) {
    console.log('PWA is disabled');
    return;
  }

  pwaService
    .register()
    .then(() => {
      console.log('✅ PWA initialized successfully');
    })
    .catch((error) => {
      console.error('❌ PWA initialization failed:', error);
    });
}

/**
 * Application initialization
 */
function initializeApp(): void {
  // Render the application
  render(() => <App />, root!);

  // Initialize PWA
  initializePWA();

  // Log application info in development
  if (import.meta.env.DEV) {
    console.log('🚀 Application started in development mode');
    console.log('API Base URL:', import.meta.env.VITE_API_BASE_URL || 'http://localhost:8000');
  }
}

// Start the application when DOM is ready
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', initializeApp);
} else {
  initializeApp();
}