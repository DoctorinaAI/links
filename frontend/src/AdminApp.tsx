/**
 * Admin Application Component
 * Main SolidJS app for admin panel
 */

import { Route, Router } from '@solidjs/router';
import { Component } from 'solid-js';
import { ErrorBoundary } from './components/ErrorBoundary';
import { ToastContainer } from './components/ui/ToastContainer';
import DashboardPage from './pages/DashboardPage';
import NotFoundPage from './pages/NotFoundPage';
import './styles/app.css';

/**
 * Admin app with routing
 */
const AdminApp: Component = () => {
  return (
    <ErrorBoundary>
      <Router base="/admin">
        <Route path="/" component={DashboardPage} />
        <Route path="/dashboard" component={DashboardPage} />
        <Route path="*" component={NotFoundPage} />
      </Router>

      {/* Global toast notifications */}
      <ToastContainer />
    </ErrorBoundary>
  );
};

export default AdminApp;
