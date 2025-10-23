/**
 * Application routes configuration
 */

import type { RouteDefinition } from '@solidjs/router';
import { lazy } from 'solid-js';

// Lazy-loaded page components
const HomePage = lazy(() => import('../pages/HomePage'));
const DashboardPage = lazy(() => import('../pages/DashboardPage'));
const NotFoundPage = lazy(() => import('../pages/NotFoundPage'));

/**
 * Route definitions
 */
export const routes: RouteDefinition[] = [
  {
    path: '/',
    component: HomePage,
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
  HOME: '/',
  DASHBOARD: '/dashboard',
} as const;
