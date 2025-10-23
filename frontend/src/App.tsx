/**
 * Main Application Component
 * Sets up routing, error boundary, and global providers
 */

import { Route, Router } from '@solidjs/router';
import { Component } from 'solid-js';
import { ErrorBoundary } from './components/ErrorBoundary';
import { ToastContainer } from './components/ui/ToastContainer';
import DashboardPage from './pages/DashboardPage';
import HomePage from './pages/HomePage';
import NotFoundPage from './pages/NotFoundPage';
import './styles/app.css';

/**
 * App component with all providers
 */
const App: Component = () => {
  return (
    <ErrorBoundary>
      <Router>
        <Route path="/" component={HomePage} />
        <Route path="/dashboard" component={DashboardPage} />
        <Route path="*" component={NotFoundPage} />
      </Router>

      {/* Global toast notifications */}
      <ToastContainer />
    </ErrorBoundary>
  );
};

export default App;