/// <reference types="vite/client" />

/**
 * Environment variables type definitions
 */
interface ImportMetaEnv {
  readonly VITE_API_BASE_URL?: string;
  readonly VITE_API_TIMEOUT?: string;
  readonly VITE_APP_VERSION?: string;
  readonly VITE_GOOGLE_CLIENT_ID?: string;
  readonly VITE_PWA_DEV?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
