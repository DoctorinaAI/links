/**
 * Admin Application Component
 * Main SolidJS app for admin panel with authentication
 */

import { Route, Router } from '@solidjs/router';
import { Component, onMount, Show } from 'solid-js';
import { ErrorBoundary } from './components/ErrorBoundary';
import { ToastContainer } from './components/ui/ToastContainer';
import DashboardPage from './pages/DashboardPage';
import LoginPage from './pages/LoginPage';
import NotFoundPage from './pages/NotFoundPage';
import { validateStoredToken } from './services/auth.service';
import { setupAuthInterceptors } from './services/http.auth.setup';
import { getUser, login, setAuthLoading } from './stores/auth.store';
import './styles/app.css';

/**
 * Admin app with authentication and routing
 */
const AdminApp: Component = () => {
  // Setup auth interceptors on mount
  onMount(async () => {
    setupAuthInterceptors();

    // Check for stored token and validate
    setAuthLoading(true);
    try {
      const user = await validateStoredToken();
      if (user) {
        const token = localStorage.getItem('auth_token');
        if (token) {
          login(user, token);
        }
      }
    } catch (error) {
      console.error('Token validation error:', error);
    } finally {
      setAuthLoading(false);
    }
  });

  return (
    <ErrorBoundary>
      <Show
        when={getUser()()}
        fallback={<LoginPage />}
      >
        <Router base="/admin">
          <Route path="/" component={DashboardPage} />
          <Route path="/dashboard" component={DashboardPage} />
          <Route path="*" component={NotFoundPage} />
        </Router>
      </Show>

      {/* Global toast notifications */}
      <ToastContainer />
    </ErrorBoundary>
  );
};

export default AdminApp;
