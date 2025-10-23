/**
 * Admin Panel Entry Point
 * SolidJS application for admin panel
 */

/* @refresh reload */
import { render } from 'solid-js/web';
import AdminApp from './AdminApp';
import { PWA_CONFIG } from './config/app.config';
import { pwaService } from './services/pwa.service';

const root = document.getElementById('root');

if (import.meta.env.DEV && !(root instanceof HTMLElement)) {
  throw new Error(
    'Root element not found. Did you forget to add it to your admin.html?'
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
 * Admin application initialization
 */
function initializeApp(): void {
  // Render the admin application
  render(() => <AdminApp />, root!);

  // Initialize PWA
  initializePWA();

  // Log application info in development
  if (import.meta.env.DEV) {
    console.log('🚀 Admin Panel started in development mode');
    console.log('API Base URL:', import.meta.env.VITE_API_BASE_URL || 'http://localhost:8000');
  }
}

// Start the application when DOM is ready
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', initializeApp);
} else {
  initializeApp();
}
