/**
 * Admin routes configuration
 */

import type { RouteDefinition } from '@solidjs/router';
import { lazy } from 'solid-js';

// Lazy-loaded page components
const DashboardPage = lazy(() => import('../pages/DashboardPage'));
const NotFoundPage = lazy(() => import('../pages/NotFoundPage'));

/**
 * Admin route definitions
 */
export const routes: RouteDefinition[] = [
  {
    path: '/',
    component: DashboardPage,
  },
  {
    path: '/dashboard',
    component: DashboardPage,
  },
  {
    path: '*',
    component: NotFoundPage,
  },
];

/**
 * Route paths (for type-safe navigation)
 */
export const ROUTES = {
  DASHBOARD: '/',
} as const;
