/**
 * PWA Service - Service Worker registration and management
 * Implements singleton pattern for centralized PWA lifecycle control
 */
export class PWAService {
  private static instance: PWAService;
  private registration: ServiceWorkerRegistration | null = null;
  private updateAvailable = false;
  private updateCallbacks: Set<() => void> = new Set();

  private constructor() {}

  public static getInstance(): PWAService {
    if (!PWAService.instance) {
      PWAService.instance = new PWAService();
    }
    return PWAService.instance;
  }

  /**
   * Register the service worker
   * @returns Promise that resolves when registration is complete
   */
  async register(): Promise<void> {
    if (!('serviceWorker' in navigator)) {
      console.log('🚫 Service Worker is not supported in this browser');
      return;
    }

    try {
      console.log('🔄 Registering Service Worker...');

      this.registration = await navigator.serviceWorker.register('/sw.js', {
        scope: '/'
      });

      console.log('✅ Service Worker registered with scope:', this.registration.scope);

      // Set up event handlers for updates
      this.setupUpdateHandlers();

      // Check for updates immediately
      this.checkForUpdates();

    } catch (error) {
      console.error('❌ Service Worker registration failed:', error);
    }
  }

  /**
   * Set up event handlers for service worker updates
   * @private
   */
  private setupUpdateHandlers(): void {
    if (!this.registration) return;

    // New service worker found and is being installed
    this.registration.addEventListener('updatefound', () => {
      const newWorker = this.registration!.installing;
      if (!newWorker) return;

      console.log('🔄 Service Worker update found');

      newWorker.addEventListener('statechange', () => {
        if (newWorker.state === 'installed' && navigator.serviceWorker.controller) {
          // New version is ready to use
          this.updateAvailable = true;
          this.notifyUpdateAvailable();
        }
      });
    });

    // Handle messages from the service worker
    navigator.serviceWorker.addEventListener('message', (event) => {
      if (event.data && event.data.type === 'SW_UPDATE_READY') {
        this.updateAvailable = true;
        this.notifyUpdateAvailable();
      }
    });

    // Controller changed (new service worker became active)
    navigator.serviceWorker.addEventListener('controllerchange', () => {
      console.log('🔄 Service Worker updated, reloading page');
      window.location.reload();
    });
  }

  /**
   * Check for service worker updates
   * @private
   */
  private async checkForUpdates(): Promise<void> {
    if (!this.registration) return;

    try {
      await this.registration.update();
    } catch (error) {
      console.log('Service Worker update check failed:', error);
    }
  }

  /**
   * Notify that an update is available
   * Triggers registered callbacks or shows default confirmation dialog
   * @private
   */
  private notifyUpdateAvailable(): void {
    console.log('📢 Application update available');

    // Notify all registered callbacks
    if (this.updateCallbacks.size > 0) {
      this.updateCallbacks.forEach(callback => callback());
    } else {
      // Default behavior: show confirmation dialog
      if (window.confirm('An application update is available. Update now?')) {
        this.applyUpdate();
      }
    }
  }

  /**
   * Apply the waiting service worker update
   */
  public applyUpdate(): void {
    if (!this.registration || !this.updateAvailable) return;

    const waitingWorker = this.registration.waiting;
    if (waitingWorker) {
      waitingWorker.postMessage({ type: 'SKIP_WAITING' });
    }
  }

  /**
   * Register a callback to be invoked when an update is available
   * @param callback Function to call when update is ready
   */
  public onUpdateAvailable(callback: () => void): void {
    this.updateCallbacks.add(callback);
  }

  /**
   * Unregister an update callback
   * @param callback Function to remove from callbacks
   */
  public offUpdateAvailable(callback: () => void): void {
    this.updateCallbacks.delete(callback);
  }

  /**
   * Check if PWA features are supported
   * @returns true if service workers and push notifications are supported
   */
  public isPWASupported(): boolean {
    return 'serviceWorker' in navigator && 'PushManager' in window;
  }

  /**
   * Check if the app is currently installed as a PWA
   * @returns true if running in standalone mode
   */
  public isPWAInstalled(): boolean {
    return window.matchMedia('(display-mode: standalone)').matches ||
           (window.navigator as any).standalone === true;
  }

  /**
   * Check if an update is currently available
   * @returns true if an update is waiting to be applied
   */
  public isUpdateAvailable(): boolean {
    return this.updateAvailable;
  }

  /**
   * Clear all caches (useful for development)
   * @returns Promise that resolves when all caches are cleared
   */
  public async clearCache(): Promise<void> {
    if ('caches' in window) {
      const cacheNames = await caches.keys();
      await Promise.all(
        cacheNames.map(cacheName => caches.delete(cacheName))
      );
      console.log('🗑️ All caches cleared');
    }
  }

  /**
   * Unregister the service worker
   * @returns Promise that resolves when unregistration is complete
   */
  public async unregister(): Promise<boolean> {
    if (!this.registration) return false;

    try {
      const success = await this.registration.unregister();
      if (success) {
        console.log('✅ Service Worker unregistered successfully');
        this.registration = null;
        this.updateAvailable = false;
      }
      return success;
    } catch (error) {
      console.error('❌ Service Worker unregistration failed:', error);
      return false;
    }
  }
}

// Export singleton instance
export const pwaService = PWAService.getInstance();