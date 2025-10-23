import { Route, Router } from '@solidjs/router';
import AlphaRibbon from './components/AlphaRibbon';
import AppErrorBoundary from './components/ErrorBoundary';
import ToastContainer from './components/ToastContainer';
import { CatchAllRoute, ProtectedRoute, PublicRoute } from './components/routes';
import { AuthProvider } from './contexts/auth.context';
import { I18nProvider } from './contexts/i18n.context';
import HomePage from './pages/home';
import LoginPage from './pages/login';
import ProjectPage from './pages/project';
import SettingsPage from './pages/settings';
import { analyticsService } from './services/analytics.service';
import './styles/app.css';

function App() {
  // Track app initialization
  analyticsService.trackAppInitialized();
  return (
    <AppErrorBoundary>
      <I18nProvider>
        <AuthProvider>
          <Router>
            <Route
              path="/login"
              component={() => (
                <PublicRoute>
                  <LoginPage />
                </PublicRoute>
              )}
            />
            <Route
              path="/"
              component={() => (
                <ProtectedRoute>
                  <HomePage />
                </ProtectedRoute>
              )}
            />
            <Route
              path="/project/:id"
              component={() => (
                <ProtectedRoute>
                  <ProjectPage />
                </ProtectedRoute>
              )}
            />
            <Route
              path="/settings"
              component={() => (
                <ProtectedRoute>
                  <SettingsPage />
                </ProtectedRoute>
              )}
            />
            <Route path="/*all" component={CatchAllRoute} />
          </Router>
        </AuthProvider>
      </I18nProvider>
      <ToastContainer />
      <AlphaRibbon />
    </AppErrorBoundary>
  );
}

export default App;